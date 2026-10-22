use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ancleto_code_core::host::{
    ensure_token, listen_stream, read_message, read_token, write_message, HostClient, HostPaths,
    RpcNotification, RpcRequest, RpcResponse, PROTOCOL_VERSION,
};
use ancleto_code_core::session::{SessionEvent, SessionManager, SessionSpec, SessionStatus};
use ancleto_code_core::{SavedSession, Store};
use interprocess::local_socket::Stream;
use serde::Deserialize;
use serde_json::{json, Value};

const DEFAULT_IDLE_MS: u64 = 30 * 60 * 1000;

type ClientSenders = Arc<Mutex<Vec<(u64, Sender<Value>)>>>;

struct Shared {
    sessions: SessionManager,
    store: Mutex<Store>,
    clients: ClientSenders,
    attached: Mutex<HashMap<String, HashSet<u64>>>,
    next_client: AtomicU64,
    last_activity: Mutex<Instant>,
    idle_ms: u64,
    paths: HostPaths,
    shutting_down: AtomicBool,
}

struct Options {
    command: String,
    db: PathBuf,
    state_dir: Option<PathBuf>,
    idle_ms: u64,
}

fn main() {
    let options = match parse_args() {
        Ok(options) => options,
        Err(message) => {
            eprintln!("ancleto-code-host: {message}");
            std::process::exit(2);
        }
    };

    let paths = HostPaths::discover(options.state_dir.as_deref());
    match options.command.as_str() {
        "status" => status(&paths),
        "stop" => stop(&paths),
        _ => start(&paths, options),
    }
}

fn parse_args() -> Result<Options, String> {
    let mut command = "start".to_owned();
    let mut db = None;
    let mut state_dir = None;
    let mut idle_ms = DEFAULT_IDLE_MS;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "start" | "status" | "stop" => command = arg,
            "--db" => db = args.next().map(PathBuf::from),
            "--state-dir" => state_dir = args.next().map(PathBuf::from),
            "--idle-ms" => {
                idle_ms = args
                    .next()
                    .ok_or_else(|| "--idle-ms necesita un valor".to_owned())?
                    .parse()
                    .map_err(|_| "--idle-ms inválido".to_owned())?;
            }
            other => return Err(format!("opción desconocida: {other}")),
        }
    }

    Ok(Options {
        command,
        db: db.unwrap_or_else(ancleto_code_core::host::transport::default_database_path),
        state_dir,
        idle_ms,
    })
}

fn status(paths: &HostPaths) {
    match HostClient::connect(paths) {
        Ok(client) => match client.status() {
            Ok(status) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&status).unwrap_or_default()
                );
            }
            Err(error) => {
                eprintln!("ancleto-code-host: {error}");
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("ancleto-code-host: {error}");
            std::process::exit(1);
        }
    }
}

fn stop(paths: &HostPaths) {
    match HostClient::connect(paths) {
        Ok(client) => match client.shutdown() {
            Ok(()) => println!("ancleto-code-host: detenido"),
            Err(error) => {
                eprintln!("ancleto-code-host: {error}");
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("ancleto-code-host: {error}");
            std::process::exit(1);
        }
    }
}

fn start(paths: &HostPaths, options: Options) {
    if let Err(error) = ensure_token(paths) {
        eprintln!("ancleto-code-host: no se pudo preparar el token: {error}");
        std::process::exit(1);
    }
    let token = match read_token(paths) {
        Ok(token) => token,
        Err(error) => {
            eprintln!("ancleto-code-host: no se pudo leer el token: {error}");
            std::process::exit(1);
        }
    };

    if HostClient::connect(paths).is_ok() {
        eprintln!("ancleto-code-host: ya hay un host en ejecución");
        std::process::exit(0);
    }

    #[cfg(unix)]
    {
        let socket_path = std::path::Path::new(&paths.socket);
        if socket_path.exists() {
            let _ = std::fs::remove_file(socket_path);
        }
    }

    let lock = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&paths.lock);
    let mut lock = match lock {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if HostClient::connect(paths).is_ok() {
                eprintln!("ancleto-code-host: ya hay un host en ejecución");
                std::process::exit(0);
            }
            let _ = std::fs::remove_file(&paths.lock);
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&paths.lock)
            {
                Ok(file) => file,
                Err(error) => {
                    eprintln!("ancleto-code-host: lock ocupado: {error}");
                    std::process::exit(1);
                }
            }
        }
        Err(error) => {
            eprintln!("ancleto-code-host: no se pudo crear el lock: {error}");
            std::process::exit(1);
        }
    };
    let _ = writeln!(lock, "{}", std::process::id());
    let _ = lock.flush();

    let store = match Store::open(&options.db) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("ancleto-code-host: no se pudo abrir la base: {error}");
            std::process::exit(1);
        }
    };
    if let Ok(count) = store.crash_active_sessions() {
        if count > 0 {
            eprintln!("ancleto-code-host: {count} sesión(es) marcadas como crashed");
        }
    }

    let clients: ClientSenders = Arc::new(Mutex::new(Vec::new()));
    let event_clients = Arc::clone(&clients);
    let sessions = SessionManager::new(Some(Arc::new(move |event| {
        broadcast(&event_clients, notification(event));
    })));

    let shared = Arc::new(Shared {
        sessions,
        store: Mutex::new(store),
        clients,
        attached: Mutex::new(HashMap::new()),
        next_client: AtomicU64::new(1),
        last_activity: Mutex::new(Instant::now()),
        idle_ms: options.idle_ms,
        paths: paths.clone(),
        shutting_down: AtomicBool::new(false),
    });

    let listener = match listen_stream(paths) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("ancleto-code-host: no se pudo escuchar: {error}");
            let _ = std::fs::remove_file(&paths.lock);
            std::process::exit(1);
        }
    };

    eprintln!(
        "ancleto-code-host {} escuchando (pid {})",
        env!("CARGO_PKG_VERSION"),
        std::process::id()
    );

    spawn_idle_watchdog(Arc::clone(&shared));

    let signal_shared = Arc::clone(&shared);
    let _ = ctrlc::set_handler(move || {
        request_shutdown(&signal_shared);
    });

    for incoming in listener {
        match incoming {
            Ok(stream) => {
                let shared = Arc::clone(&shared);
                let token = token.clone();
                thread::spawn(move || handle_client(shared, stream, token));
            }
            Err(error) => {
                if shared.shutting_down.load(Ordering::SeqCst) {
                    break;
                }
                eprintln!("ancleto-code-host: conexión fallida: {error}");
            }
        }
    }
}

fn spawn_idle_watchdog(shared: Arc<Shared>) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        if shared.shutting_down.load(Ordering::SeqCst) {
            return;
        }
        if shared.idle_ms == 0 {
            continue;
        }

        let clients = shared
            .clients
            .lock()
            .map(|clients| clients.len())
            .unwrap_or(0);
        let sessions = shared.sessions.list().unwrap_or_default();
        let active = sessions.iter().any(|session| {
            !matches!(
                session.status,
                SessionStatus::Stopped | SessionStatus::Crashed
            )
        });
        let idle = shared
            .last_activity
            .lock()
            .map(|instant| instant.elapsed())
            .unwrap_or_default();

        if clients == 0 && !active && idle >= Duration::from_millis(shared.idle_ms) {
            request_shutdown(&shared);
            return;
        }
    });
}

fn handle_client(shared: Arc<Shared>, stream: Stream, token: String) {
    let stream = Arc::new(stream);
    let client_id = shared.next_client.fetch_add(1, Ordering::SeqCst);
    touch(&shared);

    let handshake = {
        let mut reader = &*stream;
        read_message(&mut reader)
    };
    let request: RpcRequest = match handshake.and_then(|value| {
        serde_json::from_value(value)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }) {
        Ok(request) => request,
        Err(_) => return,
    };

    if request.method != "host.handshake" {
        respond_direct(
            &stream,
            RpcResponse::error(Some(request.id), -32600, "handshake requerido"),
        );
        return;
    }

    #[derive(Deserialize)]
    struct Handshake {
        #[serde(rename = "protocolVersion")]
        protocol_version: u32,
        token: String,
    }

    let handshake: Handshake = match serde_json::from_value(request.params.clone()) {
        Ok(handshake) => handshake,
        Err(error) => {
            respond_direct(
                &stream,
                RpcResponse::error(Some(request.id), -32600, error.to_string()),
            );
            return;
        }
    };

    if handshake.token != token {
        respond_direct(
            &stream,
            RpcResponse::error(Some(request.id), -32001, "token inválido"),
        );
        return;
    }
    if handshake.protocol_version != PROTOCOL_VERSION {
        respond_direct(
            &stream,
            RpcResponse::error(
                Some(request.id),
                -32002,
                format!(
                    "protocolo incompatible: {} (se esperaba {PROTOCOL_VERSION})",
                    handshake.protocol_version
                ),
            ),
        );
        return;
    }

    let (sender, receiver) = mpsc::channel::<Value>();
    if let Ok(mut clients) = shared.clients.lock() {
        clients.push((client_id, sender.clone()));
    }

    let writer_stream = Arc::clone(&stream);
    thread::spawn(move || {
        let mut writer = &*writer_stream;
        while let Ok(value) = receiver.recv() {
            if write_message(&mut writer, &value).is_err() {
                break;
            }
        }
    });

    let _ = sender.send(response_value(RpcResponse::ok(
        request.id,
        json!({ "protocolVersion": PROTOCOL_VERSION, "pid": std::process::id() }),
    )));

    loop {
        let message = {
            let mut reader = &*stream;
            read_message(&mut reader)
        };
        let value = match message {
            Ok(value) => value,
            Err(_) => break,
        };
        let request: RpcRequest = match serde_json::from_value(value) {
            Ok(request) => request,
            Err(error) => {
                let _ = sender.send(response_value(RpcResponse::error(
                    None,
                    -32600,
                    error.to_string(),
                )));
                continue;
            }
        };
        touch(&shared);
        let response = dispatch(&shared, client_id, request);
        let _ = sender.send(response_value(response));
        if shared.shutting_down.load(Ordering::SeqCst) {
            break;
        }
    }

    detach_client(&shared, client_id);
    if let Ok(mut clients) = shared.clients.lock() {
        clients.retain(|(id, _)| *id != client_id);
    }
    touch(&shared);
}

fn dispatch(shared: &Arc<Shared>, client_id: u64, request: RpcRequest) -> RpcResponse {
    let id = request.id;
    match request.method.as_str() {
        "host.status" => {
            let clients = shared
                .clients
                .lock()
                .map(|clients| clients.len())
                .unwrap_or(0);
            let sessions = shared
                .sessions
                .list()
                .map(|sessions| sessions.len())
                .unwrap_or(0);
            RpcResponse::ok(
                id,
                json!({
                    "protocolVersion": PROTOCOL_VERSION,
                    "pid": std::process::id(),
                    "clients": clients,
                    "sessions": sessions,
                    "uptimeMs": 0,
                }),
            )
        }
        "host.shutdown" => {
            let shared = Arc::clone(shared);
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(100));
                request_shutdown(&shared);
            });
            RpcResponse::ok(id, json!({ "ok": true }))
        }
        "session.create" => match parse_params::<CreateParams>(request.params) {
            Ok(params) => match shared.sessions.create(params.spec.clone()) {
                Ok(info) => {
                    let saved = SavedSession {
                        id: info.id.clone(),
                        spec: params.spec,
                        status: info.status,
                        pid: info.pid,
                        created_at: unix_timestamp(),
                    };
                    if let Ok(store) = shared.store.lock() {
                        let _ = store.save_session(&saved);
                    }
                    match serde_json::to_value(&info) {
                        Ok(value) => RpcResponse::ok(id, value),
                        Err(error) => RpcResponse::error(Some(id), -32000, error.to_string()),
                    }
                }
                Err(error) => session_error(id, error),
            },
            Err(error) => RpcResponse::error(Some(id), -32600, error),
        },
        "session.write" => match parse_params::<WriteParams>(request.params) {
            Ok(params) => match shared.sessions.write(&params.id, &params.data) {
                Ok(()) => RpcResponse::ok(id, Value::Null),
                Err(error) => session_error(id, error),
            },
            Err(error) => RpcResponse::error(Some(id), -32600, error),
        },
        "session.resize" => match parse_params::<ResizeParams>(request.params) {
            Ok(params) => match shared.sessions.resize(&params.id, params.cols, params.rows) {
                Ok(()) => RpcResponse::ok(id, Value::Null),
                Err(error) => session_error(id, error),
            },
            Err(error) => RpcResponse::error(Some(id), -32600, error),
        },
        "session.kill" => match parse_params::<IdParams>(request.params) {
            Ok(params) => {
                let result = shared.sessions.kill(&params.id);
                if let Ok(store) = shared.store.lock() {
                    let _ = store.set_session_status(&params.id, SessionStatus::Stopped);
                }
                match result {
                    Ok(()) => RpcResponse::ok(id, Value::Null),
                    Err(error) => session_error(id, error),
                }
            }
            Err(error) => RpcResponse::error(Some(id), -32600, error),
        },
        "session.kill_all" => match shared.sessions.kill_all() {
            Ok(()) => {
                if let Ok(sessions) = shared.sessions.list() {
                    if let Ok(store) = shared.store.lock() {
                        for session in sessions {
                            let _ = store.set_session_status(&session.id, SessionStatus::Stopped);
                        }
                    }
                }
                RpcResponse::ok(id, Value::Null)
            }
            Err(error) => session_error(id, error),
        },
        "session.snapshot" => match parse_params::<IdParams>(request.params) {
            Ok(params) => match shared.sessions.snapshot(&params.id) {
                Ok(data) => RpcResponse::ok(id, json!({ "data": data })),
                Err(error) => session_error(id, error),
            },
            Err(error) => RpcResponse::error(Some(id), -32600, error),
        },
        "session.list" => match shared.sessions.list() {
            Ok(sessions) => match serde_json::to_value(sessions) {
                Ok(value) => RpcResponse::ok(id, value),
                Err(error) => RpcResponse::error(Some(id), -32000, error.to_string()),
            },
            Err(error) => session_error(id, error),
        },
        "session.saved" => {
            let result = shared
                .store
                .lock()
                .map_err(|_| "store bloqueado".to_owned())
                .and_then(|store| store.saved_sessions().map_err(|error| error.to_string()));
            match result.and_then(|sessions| {
                serde_json::to_value(sessions).map_err(|error| error.to_string())
            }) {
                Ok(value) => RpcResponse::ok(id, value),
                Err(error) => RpcResponse::error(Some(id), -32000, error),
            }
        }
        "session.attach" => match parse_params::<IdParams>(request.params) {
            Ok(params) => {
                let snapshot = shared.sessions.snapshot(&params.id);
                match snapshot {
                    Ok(data) => {
                        if let Ok(mut attached) = shared.attached.lock() {
                            attached
                                .entry(params.id.clone())
                                .or_default()
                                .insert(client_id);
                        }
                        if let Ok(sessions) = shared.sessions.list() {
                            if let Some(session) =
                                sessions.iter().find(|session| session.id == params.id)
                            {
                                if session.status == SessionStatus::Detached {
                                    let _ = shared
                                        .sessions
                                        .set_status(&params.id, SessionStatus::Ready);
                                    if let Ok(store) = shared.store.lock() {
                                        let _ = store
                                            .set_session_status(&params.id, SessionStatus::Ready);
                                    }
                                }
                            }
                        }
                        RpcResponse::ok(id, json!({ "data": data }))
                    }
                    Err(error) => session_error(id, error),
                }
            }
            Err(error) => RpcResponse::error(Some(id), -32600, error),
        },
        "session.detach" => match parse_params::<IdParams>(request.params) {
            Ok(params) => {
                detach_session(shared, client_id, &params.id);
                RpcResponse::ok(id, Value::Null)
            }
            Err(error) => RpcResponse::error(Some(id), -32600, error),
        },
        other => RpcResponse::error(Some(id), -32601, format!("método desconocido: {other}")),
    }
}

fn detach_session(shared: &Arc<Shared>, client_id: u64, id: &str) {
    let empty = {
        let Ok(mut attached) = shared.attached.lock() else {
            return;
        };
        let Some(clients) = attached.get_mut(id) else {
            return;
        };
        clients.remove(&client_id);
        let empty = clients.is_empty();
        if empty {
            attached.remove(id);
        }
        empty
    };

    if empty {
        if let Ok(sessions) = shared.sessions.list() {
            if let Some(session) = sessions.iter().find(|session| session.id == id) {
                if !matches!(
                    session.status,
                    SessionStatus::Stopped | SessionStatus::Crashed | SessionStatus::Detached
                ) {
                    let _ = shared.sessions.set_status(id, SessionStatus::Detached);
                    if let Ok(store) = shared.store.lock() {
                        let _ = store.set_session_status(id, SessionStatus::Detached);
                    }
                }
            }
        }
    }
}

fn detach_client(shared: &Arc<Shared>, client_id: u64) {
    let sessions: Vec<String> = shared
        .attached
        .lock()
        .map(|attached| {
            attached
                .iter()
                .filter(|(_, clients)| clients.contains(&client_id))
                .map(|(id, _)| id.clone())
                .collect()
        })
        .unwrap_or_default();

    for id in sessions {
        detach_session(shared, client_id, &id);
    }
}

fn request_shutdown(shared: &Arc<Shared>) {
    if shared.shutting_down.swap(true, Ordering::SeqCst) {
        return;
    }
    let _ = shared.sessions.kill_all();
    if let Ok(sessions) = shared.sessions.list() {
        if let Ok(store) = shared.store.lock() {
            for session in sessions {
                let _ = store.set_session_status(&session.id, SessionStatus::Stopped);
            }
        }
    }
    #[cfg(unix)]
    {
        let _ = std::fs::remove_file(&shared.paths.socket);
    }
    let _ = std::fs::remove_file(&shared.paths.lock);
    std::process::exit(0);
}

fn broadcast(clients: &ClientSenders, notification: RpcNotification) {
    let Ok(value) = serde_json::to_value(notification) else {
        return;
    };
    if let Ok(mut clients) = clients.lock() {
        clients.retain(|(_, sender)| sender.send(value.clone()).is_ok());
    }
}

fn notification(event: SessionEvent) -> RpcNotification {
    match event {
        SessionEvent::Output { id, data } => {
            RpcNotification::new("session.output", json!({ "id": id, "data": data }))
        }
        SessionEvent::State { id, status } => RpcNotification::new(
            "session.state",
            json!({ "id": id, "status": serde_json::to_value(status).unwrap_or(Value::Null) }),
        ),
        SessionEvent::Exit { id, exit_code } => {
            RpcNotification::new("session.exit", json!({ "id": id, "exitCode": exit_code }))
        }
    }
}

fn respond_direct(stream: &Arc<Stream>, response: RpcResponse) {
    if let Ok(value) = serde_json::to_value(response) {
        let mut writer = &**stream;
        let _ = write_message(&mut writer, &value);
    }
}

fn response_value(response: RpcResponse) -> Value {
    serde_json::to_value(response).unwrap_or_else(|_| {
        json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32000, "message": "serialización" } })
    })
}

fn parse_params<T: for<'de> Deserialize<'de>>(params: Value) -> Result<T, String> {
    serde_json::from_value(params).map_err(|error| error.to_string())
}

fn session_error(id: u64, error: ancleto_code_core::CoreError) -> RpcResponse {
    RpcResponse::error(Some(id), -32003, error.to_string())
}

fn touch(shared: &Arc<Shared>) {
    if let Ok(mut instant) = shared.last_activity.lock() {
        *instant = Instant::now();
    }
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or_default()
}

#[derive(Deserialize)]
struct CreateParams {
    spec: SessionSpec,
}

#[derive(Deserialize)]
struct IdParams {
    id: String,
}

#[derive(Deserialize)]
struct WriteParams {
    id: String,
    data: Vec<u8>,
}

#[derive(Deserialize)]
struct ResizeParams {
    id: String,
    cols: u16,
    rows: u16,
}
