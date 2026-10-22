use std::env;
use std::io::{IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use ancleto_code_core::drivers::{AgentSignal, ParseState, SpawnContext, ToolStatus};
use ancleto_code_core::host::transport::default_database_path;
use ancleto_code_core::providers::{NodeRuntime, SpecProvider, ToolReport, WriterProvider};
use ancleto_code_core::{
    ConversationState, DriverRegistry, HostClient, HostPaths, Orchestrator, ProjectService,
    SessionBackend, StartConversation, Store,
};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use serde::Serialize;
use serde_json::{json, Value};

const SCHEMA_VERSION: u32 = 1;
const EXIT_OK: i32 = 0;
const EXIT_USAGE: i32 = 2;
const EXIT_MISSING: i32 = 3;
const EXIT_SESSION: i32 = 4;
const EXIT_INCOMPATIBLE: i32 = 5;
const EXIT_CANCELLED: i32 = 6;

static CANCELLED: AtomicBool = AtomicBool::new(false);

#[derive(Parser)]
#[command(
    name = "acode",
    bin_name = "acode",
    version,
    about = "CLI headless de AncletoCode",
    disable_help_subcommand = false
)]
struct Cli {
    #[arg(long, global = true, help = "Salida JSON versionada")]
    json: bool,
    #[arg(long, global = true, help = "Sin prompts interactivos")]
    no_input: bool,
    #[arg(long, global = true, help = "Ejecutar sin host de sesiones")]
    no_host: bool,
    #[arg(long, global = true, help = "Proyecto por id o ruta")]
    project: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Clone)]
enum Commands {
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    Agents,
    Doctor,
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },
    Run {
        prompt: String,
        #[arg(long)]
        driver: Option<String>,
    },
    Converse {
        #[arg(long)]
        a: String,
        #[arg(long)]
        b: String,
        #[arg(long)]
        objective: String,
        #[arg(long, default_value_t = 6)]
        max_turns: u32,
        #[arg(long, default_value_t = 60_000)]
        turn_budget_ms: u64,
    },
    Spec {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Writer {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Host {
        #[command(subcommand)]
        action: HostAction,
    },
    Ui,
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Subcommand, Clone)]
enum ProjectAction {
    Add { path: String },
    List,
    Remove { id: String },
    Open { id: String },
}

#[derive(Subcommand, Clone)]
enum SessionAction {
    Start {
        #[arg(long)]
        driver: String,
    },
    List,
    Attach {
        id: String,
    },
    Send {
        id: String,
        text: Vec<String>,
    },
    Kill {
        id: String,
    },
}

#[derive(Subcommand, Clone)]
enum HostAction {
    Start,
    Stop,
    Status,
}

struct Output {
    data: Value,
    human: String,
}

struct CliError {
    exit: i32,
    message: String,
    hint: Option<String>,
}

#[derive(Serialize)]
struct Envelope {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ErrorBody>,
}

#[derive(Serialize)]
struct ErrorBody {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<String>,
}

fn cli_error(exit: i32, message: impl Into<String>) -> CliError {
    CliError {
        exit,
        message: message.into(),
        hint: None,
    }
}

fn cli_error_with_hint(exit: i32, message: impl Into<String>, hint: impl Into<String>) -> CliError {
    CliError {
        exit,
        message: message.into(),
        hint: Some(hint.into()),
    }
}

pub fn run() -> i32 {
    let cli = Cli::parse();
    let _ = ctrlc::set_handler(|| {
        CANCELLED.store(true, Ordering::SeqCst);
    });

    let Some(command) = cli.command.clone() else {
        let mut command = Cli::command();
        let _ = command.print_help();
        println!();
        return EXIT_OK;
    };

    let result = dispatch(&cli, command);
    emit(cli.json, result)
}

fn emit(json_mode: bool, result: Result<Output, CliError>) -> i32 {
    match result {
        Ok(output) => {
            if json_mode {
                let envelope = Envelope {
                    schema_version: SCHEMA_VERSION,
                    ok: true,
                    data: Some(output.data),
                    error: None,
                };
                println!("{}", serde_json::to_string(&envelope).unwrap_or_default());
            } else if !output.human.is_empty() {
                println!("{}", output.human);
            }
            EXIT_OK
        }
        Err(error) => {
            let body = ErrorBody {
                code: error_code(error.exit).to_owned(),
                message: error.message.clone(),
                hint: error.hint.clone(),
            };
            if json_mode {
                let envelope = Envelope {
                    schema_version: SCHEMA_VERSION,
                    ok: false,
                    data: None,
                    error: Some(body),
                };
                println!("{}", serde_json::to_string(&envelope).unwrap_or_default());
            } else {
                eprintln!("acode: {}", error.message);
                if let Some(hint) = &error.hint {
                    eprintln!("  {hint}");
                }
            }
            error.exit
        }
    }
}

fn error_code(exit: i32) -> &'static str {
    match exit {
        EXIT_USAGE => "invalid_usage",
        EXIT_MISSING => "missing_dependency",
        EXIT_SESSION => "session_error",
        EXIT_INCOMPATIBLE => "incompatible_state",
        EXIT_CANCELLED => "cancelled",
        _ => "error",
    }
}

fn dispatch(cli: &Cli, command: Commands) -> Result<Output, CliError> {
    match command {
        Commands::Project { action } => project_command(action, cli.json),
        Commands::Agents => agents_command(),
        Commands::Doctor => doctor_command(),
        Commands::Session { action } => session_command(action, cli),
        Commands::Run { prompt, driver } => run_command(&prompt, driver.as_deref(), cli),
        Commands::Converse {
            a,
            b,
            objective,
            max_turns,
            turn_budget_ms,
        } => converse_command(&a, &b, &objective, max_turns, turn_budget_ms, cli),
        Commands::Spec { args } => provider_command(Provider::Spec, &args, cli, false),
        Commands::Writer { args } => provider_command(Provider::Writer, &args, cli, true),
        Commands::Host { action } => host_command(action),
        Commands::Ui => ui_command(),
        Commands::Completions { shell } => completions_command(shell),
    }
}

fn project_command(action: ProjectAction, _json: bool) -> Result<Output, CliError> {
    let service = ProjectService::open(&default_database_path())
        .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;

    match action {
        ProjectAction::Add { path } => {
            let project = service
                .add(&path)
                .map_err(|error| cli_error(EXIT_USAGE, error.to_string()))?;
            Ok(Output {
                data: serde_json::to_value(&project).unwrap_or(Value::Null),
                human: format!("{} {} {}", project.id, project.name, project.root_path),
            })
        }
        ProjectAction::List => {
            let projects = service
                .list()
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
            let human = projects
                .iter()
                .map(|project| format!("{} {} {}", project.id, project.name, project.root_path))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(Output {
                data: serde_json::to_value(&projects).unwrap_or(Value::Null),
                human,
            })
        }
        ProjectAction::Remove { id } => {
            let removed = service
                .remove(&id)
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
            if !removed {
                return Err(cli_error(
                    EXIT_SESSION,
                    format!("proyecto no encontrado: {id}"),
                ));
            }
            Ok(Output {
                data: json!({ "removed": id }),
                human: format!("proyecto eliminado: {id}"),
            })
        }
        ProjectAction::Open { id } => {
            let project = service
                .open_project(&id)
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
            Ok(Output {
                data: serde_json::to_value(&project).unwrap_or(Value::Null),
                human: format!("{} {}", project.name, project.root_path),
            })
        }
    }
}

fn agents_command() -> Result<Output, CliError> {
    let registry = driver_registry();
    let statuses = registry.detect_all();
    let human = statuses
        .iter()
        .map(|status| {
            format!(
                "{:10} {:10} {}",
                status.id,
                status.status_label(),
                status.version.clone().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(Output {
        data: serde_json::to_value(&statuses).unwrap_or(Value::Null),
        human,
    })
}

fn doctor_command() -> Result<Output, CliError> {
    let node = NodeRuntime::detect();
    let drivers = driver_registry().detect_all();
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let spec = SpecProvider::detect(&cwd).tool();
    let writer = WriterProvider::detect(&cwd).tool();
    let paths = HostPaths::discover(None);
    let host = match HostClient::connect(&paths) {
        Ok(client) => match client.status() {
            Ok(status) => json!({ "available": true, "status": status }),
            Err(error) => json!({ "available": false, "reason": error.to_string() }),
        },
        Err(error) => json!({ "available": false, "reason": error.to_string() }),
    };

    let data = json!({
        "node": node,
        "drivers": drivers,
        "spec": spec,
        "writer": writer,
        "host": host,
        "platform": {
            "os": env::consts::OS,
            "arch": env::consts::ARCH,
            "family": env::consts::FAMILY,
        },
        "terminal": std::io::stdout().is_terminal(),
        "database": default_database_path().to_string_lossy(),
    });

    let mut lines = Vec::new();
    lines.push(format!(
        "node: {}",
        node.version
            .clone()
            .unwrap_or_else(|| "no disponible".to_owned())
    ));
    for driver in &drivers {
        lines.push(format!(
            "driver {}: {} {}",
            driver.id,
            driver.status_label(),
            driver.version.clone().unwrap_or_default()
        ));
    }
    lines.push(format!("spec: {}", spec.status_label()));
    lines.push(format!("writer: {}", writer.status_label()));
    lines.push(format!(
        "host: {}",
        if data["host"]["available"] == true {
            "disponible"
        } else {
            "no disponible"
        }
    ));
    lines.push(format!(
        "plataforma: {} {}",
        env::consts::OS,
        env::consts::ARCH
    ));

    Ok(Output {
        data,
        human: lines.join("\n"),
    })
}

fn session_command(action: SessionAction, cli: &Cli) -> Result<Output, CliError> {
    match action {
        SessionAction::Start { driver } => {
            let root = resolve_project_root(cli.project.as_deref())?;
            let id = cli.project.clone().unwrap_or_else(|| "cli".to_owned());
            let agent = available_driver(&driver)?;
            let descriptor = agent.descriptor();
            let spec = ancleto_code_core::SessionSpec {
                project_id: id,
                driver_id: driver,
                program: descriptor.binary,
                args: Vec::new(),
                cwd: root,
                env: Vec::new(),
                cols: 100,
                rows: 30,
            };
            let client = connect_host()?;
            let info = client
                .create_session(spec)
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
            Ok(Output {
                data: serde_json::to_value(&info).unwrap_or(Value::Null),
                human: info.id.clone(),
            })
        }
        SessionAction::List => {
            let client = connect_host()?;
            let sessions = client
                .list_sessions()
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
            let human = sessions
                .iter()
                .map(|session| {
                    format!(
                        "{} {:10} {:10} {}",
                        session.id,
                        session.project_id,
                        session.driver_id,
                        session.status_label()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            Ok(Output {
                data: serde_json::to_value(&sessions).unwrap_or(Value::Null),
                human,
            })
        }
        SessionAction::Attach { id } => attach_session(&id),
        SessionAction::Send { id, text } => {
            let client = connect_host()?;
            let mut payload = text.join(" ");
            payload.push('\n');
            client
                .write_session(&id, payload.as_bytes())
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
            Ok(Output {
                data: json!({ "sent": payload.trim_end() }),
                human: String::new(),
            })
        }
        SessionAction::Kill { id } => {
            let client = connect_host()?;
            client
                .kill_session(&id)
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
            Ok(Output {
                data: json!({ "killed": id }),
                human: format!("sesión terminada: {id}"),
            })
        }
    }
}

fn attach_session(id: &str) -> Result<Output, CliError> {
    let client = connect_host()?;
    let snapshot = client
        .attach(id)
        .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
    let events = client.events();
    let mut stdout = std::io::stdout();

    if !snapshot.is_empty() {
        let _ = stdout.write_all(&snapshot);
        let _ = stdout.flush();
    }

    loop {
        if CANCELLED.load(Ordering::SeqCst) {
            let _ = client.detach(id);
            return Err(cli_error(EXIT_CANCELLED, "attach cancelado"));
        }
        match events.recv_timeout(Duration::from_millis(200)) {
            Ok(notification) => {
                if notification.method == "session.output" && notification.params["id"] == id {
                    if let Ok(data) =
                        serde_json::from_value::<Vec<u8>>(notification.params["data"].clone())
                    {
                        let _ = stdout.write_all(&data);
                        let _ = stdout.flush();
                    }
                }
                if notification.method == "session.state"
                    && notification.params["id"] == id
                    && notification.params["status"] == "stopped"
                {
                    let _ = client.detach(id);
                    return Err(cli_error(EXIT_CANCELLED, "la sesión se detuvo"));
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                return Err(cli_error(EXIT_SESSION, "el host cerró la conexión"));
            }
        }
    }
}

fn run_command(prompt: &str, driver_id: Option<&str>, cli: &Cli) -> Result<Output, CliError> {
    if cli.no_host {
        // La ejecución one-shot no necesita host: se ejecuta in-process.
    }
    let driver_id = driver_id.unwrap_or("claude");
    let driver = available_driver(driver_id)?;
    let root = resolve_project_root(cli.project.as_deref())?;
    let spec = driver
        .one_shot(&SpawnContext::default(), prompt)
        .ok_or_else(|| {
            cli_error(
                EXIT_INCOMPATIBLE,
                format!("{driver_id} no soporta one-shot"),
            )
        })?;

    let stream = !cli.json && std::io::stdout().is_terminal() && !cli.no_input;
    if stream {
        let status = spawn_streaming(&spec, &root)?;
        if !status.success() {
            return Err(cli_error(
                EXIT_SESSION,
                format!("el agente terminó con estado {status}"),
            ));
        }
        return Ok(Output {
            data: json!({ "driver": driver_id, "streamed": true }),
            human: String::new(),
        });
    }

    let (stdout, stderr) = spawn_capture(&spec, &root)?;
    let text = extract_text(driver.as_ref(), &stdout);
    if text.is_empty() && !stderr.is_empty() {
        return Err(cli_error(
            EXIT_SESSION,
            String::from_utf8_lossy(&stderr).trim().to_owned(),
        ));
    }
    Ok(Output {
        data: json!({
            "driver": driver_id,
            "text": text,
            "raw": String::from_utf8_lossy(&stdout),
        }),
        human: text,
    })
}

fn converse_command(
    a: &str,
    b: &str,
    objective: &str,
    max_turns: u32,
    turn_budget_ms: u64,
    _cli: &Cli,
) -> Result<Output, CliError> {
    let client = Arc::new(connect_host()?);
    let backend: Arc<dyn SessionBackend> = client.clone();
    let store = Store::open(&default_database_path())
        .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
    let orchestrator = Orchestrator::new(store, backend, driver_registry(), None);
    let conversation = orchestrator
        .start(StartConversation {
            session_a: a.to_owned(),
            session_b: b.to_owned(),
            objective: objective.to_owned(),
            max_turns: Some(max_turns),
            turn_budget_ms: Some(turn_budget_ms),
        })
        .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;

    let mut printed = 0usize;
    let stream = std::io::stdout().is_terminal();
    loop {
        if CANCELLED.load(Ordering::SeqCst) {
            let _ = orchestrator.abort(&conversation.id);
        }

        let turns = orchestrator
            .turns(&conversation.id)
            .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
        if stream {
            for turn in turns.iter().skip(printed) {
                println!(
                    "#{} {} -> {}: {}",
                    turn.index, turn.from_session, turn.to_session, turn.text
                );
            }
        }
        printed = turns.len();

        let current = orchestrator
            .conversation(&conversation.id)
            .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
        if !matches!(
            current.state,
            ConversationState::Running | ConversationState::Paused
        ) {
            let exit = match current.state {
                ConversationState::Completed => EXIT_OK,
                ConversationState::Aborted => EXIT_CANCELLED,
                _ => EXIT_SESSION,
            };
            let output = Output {
                data: json!({ "conversation": current, "turns": turns }),
                human: format!(
                    "conversación {}: {} ({})",
                    conversation.id,
                    current.state_label(),
                    current.reason.clone().unwrap_or_default()
                ),
            };
            return if exit == EXIT_OK || exit == EXIT_CANCELLED {
                Ok(output)
            } else {
                Err(cli_error(exit, output.human))
            };
        }
        thread::sleep(Duration::from_millis(250));
    }
}

enum Provider {
    Spec,
    Writer,
}

fn provider_command(
    provider: Provider,
    args: &[String],
    cli: &Cli,
    json_passthrough: bool,
) -> Result<Output, CliError> {
    if args.is_empty() {
        return Err(cli_error(EXIT_USAGE, "falta el comando del provider"));
    }
    let root = resolve_project_root(cli.project.as_deref())?;
    let path = Path::new(&root);

    let outcome = match provider {
        Provider::Spec => {
            let provider = SpecProvider::detect(path);
            ensure_tool(&provider.tool(), "ancleto")?;
            provider
                .run(path, args)
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?
        }
        Provider::Writer => {
            let provider = WriterProvider::detect(path);
            ensure_tool(&provider.tool(), "ancleto-writer")?;
            provider
                .run(path, args)
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?
        }
    };

    let data = json!({
        "success": outcome.success,
        "exitCode": outcome.exit_code,
        "stdout": outcome.stdout,
        "stderr": outcome.stderr,
        "json": if json_passthrough { serde_json::from_str::<Value>(outcome.stdout.trim()).ok() } else { None },
    });
    let human = if outcome.stdout.trim().is_empty() {
        outcome.stderr.trim().to_owned()
    } else {
        outcome.stdout.trim_end().to_owned()
    };

    if outcome.success {
        Ok(Output { data, human })
    } else {
        Err(cli_error(
            normalize_provider_exit(outcome.exit_code),
            if human.is_empty() {
                "el provider falló".to_owned()
            } else {
                human
            },
        ))
    }
}

fn normalize_provider_exit(code: Option<i32>) -> i32 {
    match code {
        Some(0) => EXIT_OK,
        Some(2) => EXIT_USAGE,
        Some(3) => EXIT_MISSING,
        Some(4) => EXIT_SESSION,
        Some(5) => EXIT_INCOMPATIBLE,
        Some(6) => EXIT_CANCELLED,
        _ => EXIT_SESSION,
    }
}

fn host_command(action: HostAction) -> Result<Output, CliError> {
    let paths = HostPaths::discover(None);
    match action {
        HostAction::Start => {
            let client = HostClient::connect_or_start(&paths, &default_database_path())
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
            let status = client
                .status()
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
            Ok(Output {
                data: serde_json::to_value(&status).unwrap_or(Value::Null),
                human: format!("host iniciado (pid {})", status.pid),
            })
        }
        HostAction::Stop => {
            let client = HostClient::connect(&paths)
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
            client
                .shutdown()
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
            Ok(Output {
                data: json!({ "stopped": true }),
                human: "host detenido".to_owned(),
            })
        }
        HostAction::Status => {
            let client = HostClient::connect(&paths)
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
            let status = client
                .status()
                .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
            Ok(Output {
                data: serde_json::to_value(&status).unwrap_or(Value::Null),
                human: format!(
                    "host pid {} · {} clientes · {} sesiones",
                    status.pid, status.clients, status.sessions
                ),
            })
        }
    }
}

fn ui_command() -> Result<Output, CliError> {
    if let Some(path) = env::var_os("ANCLETO_CODE_APP") {
        let status = Command::new(path)
            .stdin(Stdio::null())
            .spawn()
            .map_err(|error| cli_error(EXIT_MISSING, error.to_string()))?;
        return Ok(Output {
            data: json!({ "launched": true, "pid": status.id() }),
            human: "app de escritorio iniciada".to_owned(),
        });
    }

    let candidates = if cfg!(target_os = "macos") {
        vec!["AncletoCode"]
    } else if cfg!(windows) {
        vec!["AncletoCode.exe"]
    } else {
        vec!["ancleto-code-desktop", "ancleto-code-app"]
    };
    for candidate in candidates {
        if Command::new(candidate).stdin(Stdio::null()).spawn().is_ok() {
            return Ok(Output {
                data: json!({ "launched": true }),
                human: "app de escritorio iniciada".to_owned(),
            });
        }
    }
    Err(cli_error_with_hint(
        EXIT_MISSING,
        "no se encontró la app de escritorio instalada",
        "definí ANCLETO_CODE_APP con la ruta al binario o instalá AncletoCode",
    ))
}

fn completions_command(shell: Shell) -> Result<Output, CliError> {
    let mut command = Cli::command();
    let mut buffer = Vec::new();
    generate(shell, &mut command, "acode", &mut buffer);
    let text = String::from_utf8_lossy(&buffer).into_owned();
    Ok(Output {
        data: json!({ "shell": format!("{shell:?}"), "script": text }),
        human: text,
    })
}

fn connect_host() -> Result<HostClient, CliError> {
    HostClient::connect_or_start(&HostPaths::discover(None), &default_database_path())
        .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))
}

fn resolve_project_root(project: Option<&str>) -> Result<String, CliError> {
    if let Some(input) = project {
        let path = Path::new(input);
        if path.is_dir() {
            return path
                .canonicalize()
                .map(|path| path.to_string_lossy().into_owned())
                .map_err(|error| cli_error(EXIT_USAGE, error.to_string()));
        }
        let service = ProjectService::open(&default_database_path())
            .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?;
        if let Some(found) = service
            .list()
            .map_err(|error| cli_error(EXIT_SESSION, error.to_string()))?
            .into_iter()
            .find(|entry| entry.id == input)
        {
            return Ok(found.root_path);
        }
        return Err(cli_error(
            EXIT_USAGE,
            format!("proyecto no encontrado: {input}"),
        ));
    }

    env::current_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(|error| cli_error(EXIT_USAGE, error.to_string()))
}

fn driver_registry() -> DriverRegistry {
    let path = default_database_path();
    if path.is_file() {
        if let Ok(store) = Store::open(&path) {
            return DriverRegistry::from_store(&store);
        }
    }
    DriverRegistry::new()
}

fn available_driver(id: &str) -> Result<Arc<dyn ancleto_code_core::AgentDriver>, CliError> {
    let registry = driver_registry();
    let driver = registry.resolve(id);
    if driver.descriptor().id != id {
        return Err(cli_error(EXIT_MISSING, format!("driver desconocido: {id}")));
    }
    let status = driver.detect();
    if status.status != ToolStatus::Available {
        return Err(cli_error_with_hint(
            EXIT_MISSING,
            format!("{id} no está disponible"),
            status
                .install_hint
                .unwrap_or_else(|| "instalá el CLI y asegurate de que esté en el PATH".to_owned()),
        ));
    }
    Ok(driver)
}

fn ensure_tool(
    tool: &ancleto_code_core::providers::ToolReport,
    name: &str,
) -> Result<(), CliError> {
    match tool.status {
        ToolStatus::Available => Ok(()),
        ToolStatus::Incompatible => Err(cli_error_with_hint(
            EXIT_INCOMPATIBLE,
            format!("{name} es incompatible (se requiere Node ≥ 24)"),
            tool.install_hint.clone().unwrap_or_default(),
        )),
        _ => Err(cli_error_with_hint(
            EXIT_MISSING,
            format!("{name} no está disponible"),
            tool.install_hint.clone().unwrap_or_default(),
        )),
    }
}

fn spawn_streaming(
    spec: &ancleto_code_core::drivers::CommandSpec,
    cwd: &str,
) -> Result<std::process::ExitStatus, CliError> {
    let mut child = Command::new(&spec.program)
        .args(&spec.args)
        .current_dir(cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| cli_error(EXIT_MISSING, error.to_string()))?;

    loop {
        if CANCELLED.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(cli_error(EXIT_CANCELLED, "ejecución cancelada"));
        }
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(error) => return Err(cli_error(EXIT_SESSION, error.to_string())),
        }
    }
}

fn spawn_capture(
    spec: &ancleto_code_core::drivers::CommandSpec,
    cwd: &str,
) -> Result<(Vec<u8>, Vec<u8>), CliError> {
    let mut child = Command::new(&spec.program)
        .args(&spec.args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| cli_error(EXIT_MISSING, error.to_string()))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| cli_error(EXIT_SESSION, "sin stdout"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| cli_error(EXIT_SESSION, "sin stderr"))?;
    let out_handle = thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stdout.read_to_end(&mut buffer);
        buffer
    });
    let err_handle = thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stderr.read_to_end(&mut buffer);
        buffer
    });

    let deadline = Instant::now() + Duration::from_secs(600);
    loop {
        if CANCELLED.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = out_handle.join();
            let _ = err_handle.join();
            return Err(cli_error(EXIT_CANCELLED, "ejecución cancelada"));
        }
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {}
            Err(error) => return Err(cli_error(EXIT_SESSION, error.to_string())),
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            return Err(cli_error(EXIT_CANCELLED, "timeout de ejecución"));
        }
        thread::sleep(Duration::from_millis(50));
    }

    Ok((
        out_handle.join().unwrap_or_default(),
        err_handle.join().unwrap_or_default(),
    ))
}

fn extract_text(driver: &dyn ancleto_code_core::AgentDriver, output: &[u8]) -> String {
    let mut bytes = output.to_vec();
    bytes.push(b'\n');
    let mut state = ParseState::default();
    let mut text = String::new();
    for signal in driver.parse(&bytes, &mut state) {
        match signal {
            AgentSignal::MessageComplete { text: complete } => text = complete,
            AgentSignal::Text(chunk) => {
                if !text.is_empty() {
                    text.push('\n');
                }
                text.push_str(&chunk);
            }
            _ => {}
        }
    }
    text.trim().to_owned()
}

trait StatusLabel {
    fn status_label(&self) -> &'static str;
}

impl StatusLabel for ToolReport {
    fn status_label(&self) -> &'static str {
        match self.status {
            ToolStatus::Available => "available",
            ToolStatus::Missing => "missing",
            ToolStatus::Incompatible => "incompatible",
            ToolStatus::Error => "error",
        }
    }
}

impl StatusLabel for ancleto_code_core::DriverStatus {
    fn status_label(&self) -> &'static str {
        match self.status {
            ToolStatus::Available => "available",
            ToolStatus::Missing => "missing",
            ToolStatus::Incompatible => "incompatible",
            ToolStatus::Error => "error",
        }
    }
}

impl StatusLabel for ancleto_code_core::SessionInfo {
    fn status_label(&self) -> &'static str {
        self.status.label()
    }
}

impl StatusLabel for ancleto_code_core::Conversation {
    fn status_label(&self) -> &'static str {
        self.state.label()
    }
}

trait Label {
    fn label(&self) -> &'static str;
}

impl Label for ancleto_code_core::SessionStatus {
    fn label(&self) -> &'static str {
        match self {
            ancleto_code_core::SessionStatus::Starting => "starting",
            ancleto_code_core::SessionStatus::Ready => "ready",
            ancleto_code_core::SessionStatus::Busy => "busy",
            ancleto_code_core::SessionStatus::WaitingInput => "waiting_input",
            ancleto_code_core::SessionStatus::Detached => "detached",
            ancleto_code_core::SessionStatus::Stopped => "stopped",
            ancleto_code_core::SessionStatus::Crashed => "crashed",
        }
    }
}

impl Label for ConversationState {
    fn label(&self) -> &'static str {
        match self {
            ConversationState::Running => "running",
            ConversationState::Paused => "paused",
            ConversationState::Completed => "completed",
            ConversationState::Aborted => "aborted",
            ConversationState::Failed => "failed",
        }
    }
}

trait ConversationHuman {
    fn state_label(&self) -> &'static str;
}

impl ConversationHuman for ancleto_code_core::Conversation {
    fn state_label(&self) -> &'static str {
        self.state.label()
    }
}
