use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use uuid::Uuid;

use crate::domain::{Conversation, ConversationState, CoreError, Result, Turn, TurnMode};
use crate::drivers::{
    resolve_binary, AgentDriver, AgentSignal, CommandSpec, DriverRegistry, ParseState, SpawnContext,
};
use crate::persistence::Store;
use crate::session::{SessionBackend, SessionInfo};

const DEFAULT_MAX_TURNS: u32 = 6;
const MAX_TURNS_LIMIT: u32 = 50;
const DEFAULT_TURN_BUDGET_MS: u64 = 60_000;
const MIN_TURN_BUDGET_MS: u64 = 1_000;
const MAX_TURN_BUDGET_MS: u64 = 600_000;
const POLL_INTERVAL_MS: u64 = 25;

type ProcessOutput = (Vec<u8>, Vec<u8>, Option<std::process::ExitStatus>);
const QUIESCENCE_MS: u64 = 400;

pub type ConversationSink = Arc<dyn Fn(ConversationEvent) + Send + Sync>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConversationEvent {
    Turn {
        conversation_id: String,
        turn: Turn,
    },
    TurnStarted {
        conversation_id: String,
        index: u32,
        from_session: String,
        to_session: String,
        model: Option<String>,
    },
    State {
        conversation_id: String,
        state: ConversationState,
        reason: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct StartConversation {
    pub session_a: String,
    pub session_b: String,
    pub objective: String,
    pub max_turns: Option<u32>,
    pub turn_budget_ms: Option<u64>,
    pub model_a: Option<String>,
    pub model_b: Option<String>,
    pub allow_tools: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Control {
    Running,
    Paused,
    Aborted,
}

#[derive(Debug)]
enum TurnError {
    Timeout,
    Aborted,
    Failed(String),
}

struct Inner {
    store: Mutex<Store>,
    sessions: Arc<dyn SessionBackend>,
    drivers: DriverRegistry,
    sink: Option<ConversationSink>,
    controls: Mutex<HashMap<String, Control>>,
}

#[derive(Clone)]
pub struct Orchestrator {
    inner: Arc<Inner>,
}

impl Orchestrator {
    pub fn new(
        store: Store,
        sessions: Arc<dyn SessionBackend>,
        drivers: DriverRegistry,
        sink: Option<ConversationSink>,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                store: Mutex::new(store),
                sessions,
                drivers,
                sink,
                controls: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub fn start(&self, request: StartConversation) -> Result<Conversation> {
        if request.objective.trim().is_empty() {
            return Err(CoreError::InvalidInput(
                "el objetivo no puede estar vacío".to_owned(),
            ));
        }
        if request.session_a == request.session_b {
            return Err(CoreError::InvalidInput(
                "la conversación necesita dos sesiones distintas".to_owned(),
            ));
        }

        let sessions = self.inner.sessions.list_sessions()?;
        for id in [&request.session_a, &request.session_b] {
            if !sessions.iter().any(|session| &session.id == id) {
                return Err(CoreError::SessionNotFound(id.clone()));
            }
        }

        let conversation = Conversation {
            id: Uuid::new_v4().to_string(),
            session_a: request.session_a,
            session_b: request.session_b,
            objective: request.objective,
            max_turns: request
                .max_turns
                .unwrap_or(DEFAULT_MAX_TURNS)
                .clamp(1, MAX_TURNS_LIMIT),
            turn_budget_ms: request
                .turn_budget_ms
                .unwrap_or(DEFAULT_TURN_BUDGET_MS)
                .clamp(MIN_TURN_BUDGET_MS, MAX_TURN_BUDGET_MS),
            state: ConversationState::Running,
            reason: None,
            created_at: unix_timestamp(),
            model_a: request.model_a.filter(|value| !value.trim().is_empty()),
            model_b: request.model_b.filter(|value| !value.trim().is_empty()),
            allow_tools: request.allow_tools,
        };

        self.store()?.create_conversation(&conversation)?;
        self.control_insert(&conversation.id, Control::Running);
        self.emit(ConversationEvent::State {
            conversation_id: conversation.id.clone(),
            state: ConversationState::Running,
            reason: None,
        });

        let orchestrator = self.clone();
        let running = conversation.clone();
        thread::spawn(move || orchestrator.run(running));

        Ok(conversation)
    }

    pub fn conversation(&self, id: &str) -> Result<Conversation> {
        self.store()?
            .get_conversation(id)?
            .ok_or_else(|| CoreError::InvalidInput(format!("conversación no encontrada: {id}")))
    }

    pub fn conversations(&self) -> Result<Vec<Conversation>> {
        self.store()?.list_conversations()
    }

    pub fn turns(&self, id: &str) -> Result<Vec<Turn>> {
        self.store()?.list_turns(id)
    }

    pub fn pause(&self, id: &str) -> Result<ConversationState> {
        self.control_insert(id, Control::Paused);
        self.set_state(id, ConversationState::Paused, Some("paused"))
    }

    pub fn resume(&self, id: &str) -> Result<ConversationState> {
        self.control_insert(id, Control::Running);
        self.set_state(id, ConversationState::Running, None)
    }

    pub fn abort(&self, id: &str) -> Result<ConversationState> {
        self.control_insert(id, Control::Aborted);
        self.set_state(id, ConversationState::Aborted, Some("aborted"))
    }

    fn run(&self, conversation: Conversation) {
        let id = conversation.id.clone();
        let sessions = match self.inner.sessions.list_sessions() {
            Ok(sessions) => sessions,
            Err(error) => {
                self.finalize(&id, ConversationState::Failed, &error.to_string());
                return;
            }
        };
        let (Some(first), Some(second)) = (
            sessions
                .iter()
                .find(|session| session.id == conversation.session_a)
                .cloned(),
            sessions
                .iter()
                .find(|session| session.id == conversation.session_b)
                .cloned(),
        ) else {
            self.finalize(&id, ConversationState::Failed, "sesión no encontrada");
            return;
        };

        let mut current = first;
        let mut other = second;
        let mut last_message: Option<String> = None;
        let mut previous_by_session: HashMap<String, String> = HashMap::new();

        for index in 0..conversation.max_turns {
            if !self.wait_running(&id) {
                self.finalize(&id, ConversationState::Aborted, "aborted");
                return;
            }

            let prompt = conversation_prompt(
                &conversation,
                &current,
                &other,
                index,
                last_message.as_deref(),
            );
            let started = Instant::now();
            let model = if current.id == conversation.session_a {
                conversation.model_a.as_deref()
            } else {
                conversation.model_b.as_deref()
            };
            self.emit(ConversationEvent::TurnStarted {
                conversation_id: id.clone(),
                index: index + 1,
                from_session: current.id.clone(),
                to_session: other.id.clone(),
                model: model.map(str::to_owned),
            });

            match self.run_turn(&conversation, &current, model, &prompt, index) {
                Ok((text, mode)) => {
                    let turn = Turn {
                        conversation_id: id.clone(),
                        index,
                        from_session: current.id.clone(),
                        to_session: other.id.clone(),
                        text: text.clone(),
                        mode,
                        duration_ms: elapsed_ms(started),
                        created_at: unix_timestamp(),
                    };

                    if let Ok(store) = self.store() {
                        let _ = store.add_turn(&turn);
                    }
                    self.emit(ConversationEvent::Turn {
                        conversation_id: id.clone(),
                        turn,
                    });

                    let normalized = normalize(&text);
                    if previous_by_session
                        .get(&current.id)
                        .is_some_and(|previous| previous == &normalized)
                    {
                        self.finalize(&id, ConversationState::Failed, "repetition");
                        return;
                    }
                    previous_by_session.insert(current.id.clone(), normalized);
                    last_message = Some(text);
                    std::mem::swap(&mut current, &mut other);
                }
                Err(TurnError::Aborted) => {
                    self.finalize(&id, ConversationState::Aborted, "aborted");
                    return;
                }
                Err(TurnError::Timeout) => {
                    self.finalize(&id, ConversationState::Failed, "timeout");
                    return;
                }
                Err(TurnError::Failed(reason)) => {
                    self.finalize(&id, ConversationState::Failed, &reason);
                    return;
                }
            }
        }

        self.finalize(&id, ConversationState::Completed, "max_turns");
    }

    fn run_turn(
        &self,
        conversation: &Conversation,
        current: &SessionInfo,
        model: Option<&str>,
        prompt: &str,
        index: u32,
    ) -> std::result::Result<(String, TurnMode), TurnError> {
        let driver = self.inner.drivers.resolve(&current.driver_id);
        let capabilities = driver.descriptor().capabilities;
        let context = SpawnContext {
            program: resolve_program(&current.program, driver.descriptor().binary.as_str()),
            args: args_with_permissions(
                args_with_model(&current.args, model, &current.driver_id),
                conversation.allow_tools,
                &current.driver_id,
            ),
            continue_session: index > 0,
            ..SpawnContext::default()
        };

        if capabilities.one_shot_prompt && driver.descriptor().id != "raw" {
            if let Some(spec) = driver.one_shot(&context, prompt) {
                let (stdout, stderr, status) =
                    self.run_process(conversation, &spec, &current.cwd)?;
                let text = extract_text(driver.as_ref(), &stdout);
                let detail = String::from_utf8_lossy(&stderr).trim().to_owned();
                if !status.map(|status| status.success()).unwrap_or(true) {
                    return Err(TurnError::Failed(if detail.is_empty() {
                        "El agente terminó con error sin explicar el motivo.".to_owned()
                    } else {
                        detail
                    }));
                }
                if text.is_empty() {
                    return Err(TurnError::Failed(if detail.is_empty() {
                        "El agente terminó sin producir una respuesta. Si fue por permisos headless, configurá la regla del CLI o usá una sesión interactiva.".to_owned()
                    } else {
                        detail
                    }));
                }
                return Ok((text, TurnMode::OneShot));
            }
        }

        let text = self.run_pty(conversation, &current.id, prompt, driver.as_ref())?;
        Ok((text, TurnMode::Pty))
    }

    fn run_process(
        &self,
        conversation: &Conversation,
        spec: &CommandSpec,
        cwd: &str,
    ) -> std::result::Result<ProcessOutput, TurnError> {
        let mut child = Command::new(&spec.program)
            .args(&spec.args)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                TurnError::Failed(format!("no se pudo ejecutar {}: {error}", spec.program))
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| TurnError::Failed("sin stdout".to_owned()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| TurnError::Failed("sin stderr".to_owned()))?;
        let out_handle = thread::spawn(move || read_all(stdout));
        let err_handle = thread::spawn(move || read_all(stderr));
        let deadline = Instant::now() + Duration::from_millis(conversation.turn_budget_ms);

        let exit_status = loop {
            if self.control(&conversation.id) == Control::Aborted {
                let _ = child.kill();
                let _ = child.wait();
                return Err(TurnError::Aborted);
            }

            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) => {}
                Err(error) => {
                    let _ = child.kill();
                    let _ = out_handle.join();
                    let _ = err_handle.join();
                    return Err(TurnError::Failed(format!("falló el proceso: {error}")));
                }
            }

            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                let _ = out_handle.join();
                let _ = err_handle.join();
                return Err(TurnError::Timeout);
            }

            thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
        };

        let stdout = out_handle.join().unwrap_or_default();
        let stderr = err_handle.join().unwrap_or_default();
        Ok((stdout, stderr, exit_status))
    }

    fn run_pty(
        &self,
        conversation: &Conversation,
        session_id: &str,
        prompt: &str,
        driver: &dyn AgentDriver,
    ) -> std::result::Result<String, TurnError> {
        let baseline = self
            .inner
            .sessions
            .snapshot_session(session_id)
            .map_err(|error| TurnError::Failed(error.to_string()))?
            .len();
        self.inner
            .sessions
            .write_session(session_id, format!("{prompt}\n").as_bytes())
            .map_err(|error| TurnError::Failed(error.to_string()))?;

        let deadline = Instant::now() + Duration::from_millis(conversation.turn_budget_ms);
        let mut consumed = baseline;
        let mut last_growth = Instant::now();
        let mut parse_state = ParseState::default();

        loop {
            if self.control(&conversation.id) == Control::Aborted {
                return Err(TurnError::Aborted);
            }
            if Instant::now() >= deadline {
                return Err(TurnError::Timeout);
            }

            thread::sleep(Duration::from_millis(100));
            let snapshot = self
                .inner
                .sessions
                .snapshot_session(session_id)
                .map_err(|error| TurnError::Failed(error.to_string()))?;

            if snapshot.len() > consumed {
                let fresh = snapshot[consumed..].to_vec();
                consumed = snapshot.len();
                last_growth = Instant::now();
                if driver
                    .parse(&fresh, &mut parse_state)
                    .iter()
                    .any(|signal| matches!(signal, AgentSignal::Ready))
                {
                    break;
                }
            }

            if consumed > baseline && last_growth.elapsed() >= Duration::from_millis(QUIESCENCE_MS)
            {
                break;
            }
        }

        let snapshot = self
            .inner
            .sessions
            .snapshot_session(session_id)
            .map_err(|error| TurnError::Failed(error.to_string()))?;
        Ok(String::from_utf8_lossy(&snapshot[baseline..])
            .trim()
            .to_owned())
    }

    fn wait_running(&self, id: &str) -> bool {
        loop {
            match self.control(id) {
                Control::Running => return true,
                Control::Aborted => return false,
                Control::Paused => thread::sleep(Duration::from_millis(100)),
            }
        }
    }

    fn control(&self, id: &str) -> Control {
        self.inner
            .controls
            .lock()
            .ok()
            .and_then(|controls| controls.get(id).copied())
            .unwrap_or(Control::Running)
    }

    fn control_insert(&self, id: &str, control: Control) {
        if let Ok(mut controls) = self.inner.controls.lock() {
            controls.insert(id.to_owned(), control);
        }
    }

    fn set_state(
        &self,
        id: &str,
        state: ConversationState,
        reason: Option<&str>,
    ) -> Result<ConversationState> {
        self.store()?.update_conversation_state(id, state, reason)?;
        self.emit(ConversationEvent::State {
            conversation_id: id.to_owned(),
            state,
            reason: reason.map(str::to_owned),
        });
        Ok(state)
    }

    fn finalize(&self, id: &str, state: ConversationState, reason: &str) {
        if let Ok(store) = self.store() {
            let _ = store.update_conversation_state(id, state, Some(reason));
        }
        if let Ok(mut controls) = self.inner.controls.lock() {
            controls.remove(id);
        }
        self.emit(ConversationEvent::State {
            conversation_id: id.to_owned(),
            state,
            reason: Some(reason.to_owned()),
        });
    }

    fn store(&self) -> Result<std::sync::MutexGuard<'_, Store>> {
        self.inner
            .store
            .lock()
            .map_err(|_| CoreError::Process("el store de conversaciones está bloqueado".to_owned()))
    }

    fn emit(&self, event: ConversationEvent) {
        if let Some(sink) = &self.inner.sink {
            sink(event);
        }
    }
}

fn read_all(mut reader: impl Read) -> Vec<u8> {
    let mut buffer = Vec::new();
    let _ = reader.read_to_end(&mut buffer);
    buffer
}

fn extract_text(driver: &dyn AgentDriver, output: &[u8]) -> String {
    let mut bytes = output.to_vec();
    bytes.push(b'\n');
    let mut state = ParseState::default();
    let mut text = String::new();

    for signal in driver.parse(&bytes, &mut state) {
        match signal {
            AgentSignal::MessageComplete { text: complete } => text = complete,
            AgentSignal::TextDelta(chunk) => text.push_str(&chunk),
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

fn conversation_prompt(
    conversation: &Conversation,
    current: &SessionInfo,
    other: &SessionInfo,
    index: u32,
    previous: Option<&str>,
) -> String {
    let turn = index + 1;
    let remaining = conversation.max_turns.saturating_sub(turn);
    let closing = if remaining == 0 {
        "Este es el último turno: cerrá con una conclusión clara y accionable.".to_owned()
    } else {
        format!("Quedan {remaining} turnos después de este.")
    };
    let previous = previous
        .map(|message| format!("Mensaje anterior de tu interlocutor:\n{message}"))
        .unwrap_or_else(|| {
            "Este es el primer turno: abrí la conversación con una propuesta concreta.".to_owned()
        });
    format!(
        "Estás participando en una conversación deliberada entre dos agentes, no en un bucle.\n         Objetivo compartido: {}\n         Vos sos {}; tu interlocutor es {}.\n         Turno {} de {}. {}\n         Respondé al objetivo y al mensaje anterior con un aporte nuevo. No repitas el texto del otro agente ni asumas que la conversación está trabada.\n\n{}",
        conversation.objective,
        short_id(&current.id),
        short_id(&other.id),
        turn,
        conversation.max_turns,
        closing,
        previous
    )
}

fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}

fn resolve_program(program: &str, fallback: &str) -> String {
    let candidate = if program.trim().is_empty() {
        fallback
    } else {
        program
    };
    if Path::new(candidate).components().count() > 1 {
        return candidate.to_owned();
    }
    resolve_binary(candidate)
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| candidate.to_owned())
}

fn args_with_permissions(mut args: Vec<String>, allow: bool, driver_id: &str) -> Vec<String> {
    if !allow {
        return args;
    }
    let flag = match driver_id {
        "agy" | "claude" => "--dangerously-skip-permissions",
        "opencode" => "--auto",
        "cmd" => "--yolo",
        _ => return args,
    };
    if !args.iter().any(|arg| arg == flag) {
        args.push(flag.to_owned());
    }
    args
}

fn args_with_model(args: &[String], model: Option<&str>, driver_id: &str) -> Vec<String> {
    let Some(model) = model.filter(|value| !value.trim().is_empty()) else {
        return args.to_vec();
    };
    let flag = if driver_id == "cmd" { "-m" } else { "--model" };
    let mut filtered = Vec::new();
    let mut skip = false;
    for arg in args {
        if skip {
            skip = false;
            continue;
        }
        if arg == "--model" || arg == "-m" {
            skip = true;
            continue;
        }
        filtered.push(arg.clone());
    }
    filtered.push(flag.to_owned());
    filtered.push(model.to_owned());
    filtered
}

fn normalize(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or_default()
}

#[cfg(all(test, unix))]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::drivers::{AgentDriverDescriptor, Capabilities, DriverStatus, ToolStatus};
    use crate::session::SessionManager;
    use crate::session::SessionSpec;
    use std::sync::Mutex;

    struct FakeDriver {
        id: String,
        program: String,
        fixed_args: Option<Vec<String>>,
    }

    impl AgentDriver for FakeDriver {
        fn descriptor(&self) -> AgentDriverDescriptor {
            AgentDriverDescriptor {
                id: self.id.clone(),
                display_name: "Fake".to_owned(),
                binary: self.program.clone(),
                capabilities: Capabilities {
                    interactive_pty: true,
                    one_shot_prompt: true,
                    structured_output: false,
                    resume_session: false,
                    headless_json: false,
                },
            }
        }

        fn detect(&self) -> DriverStatus {
            DriverStatus {
                id: self.id.clone(),
                display_name: "Fake".to_owned(),
                binary: self.program.clone(),
                status: ToolStatus::Available,
                version: Some("test".to_owned()),
                binary_path: Some(self.program.clone()),
                install_hint: None,
                capabilities: self.descriptor().capabilities,
            }
        }

        fn spawn(&self, _context: &SpawnContext) -> CommandSpec {
            CommandSpec {
                program: self.program.clone(),
                args: Vec::new(),
            }
        }

        fn one_shot(&self, _context: &SpawnContext, prompt: &str) -> Option<CommandSpec> {
            Some(CommandSpec {
                program: self.program.clone(),
                args: self
                    .fixed_args
                    .clone()
                    .unwrap_or_else(|| vec![prompt.to_owned()]),
            })
        }

        fn parse(&self, chunk: &[u8], _state: &mut ParseState) -> Vec<AgentSignal> {
            String::from_utf8_lossy(chunk)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| AgentSignal::Text(line.trim().to_owned()))
                .collect()
        }
    }

    fn setup(
        driver: FakeDriver,
    ) -> (
        tempfile::TempDir,
        Orchestrator,
        SessionManager,
        String,
        String,
    ) {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(&directory.path().join("state.sqlite")).unwrap();
        let sessions = SessionManager::new(None);
        let driver_id = driver.id.clone();
        let mut drivers = DriverRegistry::new();
        drivers.register(Arc::new(driver));

        let events = Arc::new(Mutex::new(Vec::new()));
        let recorded = events.clone();
        let sink: ConversationSink = Arc::new(move |event| recorded.lock().unwrap().push(event));
        let orchestrator =
            Orchestrator::new(store, Arc::new(sessions.clone()), drivers, Some(sink));

        let first = sessions.create(session_spec(&driver_id)).unwrap();
        let second = sessions.create(session_spec(&driver_id)).unwrap();

        (directory, orchestrator, sessions, first.id, second.id)
    }

    fn session_spec(driver_id: &str) -> SessionSpec {
        SessionSpec {
            project_id: "project-1".to_owned(),
            driver_id: driver_id.to_owned(),
            program: "/bin/cat".to_owned(),
            args: Vec::new(),
            cwd: "/tmp".to_owned(),
            env: Vec::new(),
            cols: 80,
            rows: 24,
        }
    }

    fn silent_driver() -> FakeDriver {
        FakeDriver {
            id: "silent".to_owned(),
            program: "/bin/true".to_owned(),
            fixed_args: None,
        }
    }

    fn echo_driver() -> FakeDriver {
        FakeDriver {
            id: "echo".to_owned(),
            program: "/bin/echo".to_owned(),
            fixed_args: None,
        }
    }

    fn request(a: &str, b: &str, max_turns: u32) -> StartConversation {
        StartConversation {
            session_a: a.to_owned(),
            session_b: b.to_owned(),
            objective: "objetivo".to_owned(),
            max_turns: Some(max_turns),
            turn_budget_ms: Some(5_000),
            model_a: None,
            model_b: None,
            allow_tools: false,
        }
    }

    fn wait_for_state(orchestrator: &Orchestrator, id: &str, target: ConversationState) -> bool {
        let deadline = Instant::now() + Duration::from_secs(8);
        while Instant::now() < deadline {
            if let Ok(conversation) = orchestrator.conversation(id) {
                if conversation.state == target {
                    return true;
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
        false
    }

    #[test]
    fn permission_flags_are_explicit_per_driver() {
        assert_eq!(
            args_with_permissions(Vec::new(), true, "agy"),
            vec!["--dangerously-skip-permissions"]
        );
        assert_eq!(
            args_with_permissions(Vec::new(), true, "cmd"),
            vec!["--yolo"]
        );
        assert!(args_with_permissions(Vec::new(), false, "agy").is_empty());
    }

    #[test]
    fn conversation_prompt_makes_roles_and_remaining_turns_explicit() {
        let first = SessionInfo {
            id: "session-a".to_owned(),
            project_id: "project".to_owned(),
            driver_id: "agy".to_owned(),
            status: crate::session::SessionStatus::Ready,
            pid: None,
            cwd: "/tmp".to_owned(),
            buffer_bytes: 0,
            program: "agy".to_owned(),
            args: Vec::new(),
        };
        let second = SessionInfo {
            id: "session-b".to_owned(),
            ..first.clone()
        };
        let conversation = Conversation {
            id: "c".to_owned(),
            session_a: first.id.clone(),
            session_b: second.id.clone(),
            objective: "decidir el diseño".to_owned(),
            max_turns: 3,
            turn_budget_ms: 1000,
            state: ConversationState::Running,
            reason: None,
            created_at: 0,
            model_a: None,
            model_b: None,
            allow_tools: false,
        };
        let prompt =
            conversation_prompt(&conversation, &first, &second, 2, Some("respuesta previa"));
        assert!(prompt.contains("Objetivo compartido: decidir el diseño"));
        assert!(prompt.contains("Turno 3 de 3"));
        assert!(prompt.contains("último turno"));
        assert!(prompt.contains("respuesta previa"));
    }

    #[test]
    fn completes_after_the_turn_limit_alternating_sessions() {
        let (_directory, orchestrator, sessions, first, second) = setup(echo_driver());

        let conversation = orchestrator.start(request(&first, &second, 2)).unwrap();
        assert!(wait_for_state(
            &orchestrator,
            &conversation.id,
            ConversationState::Completed
        ));

        let turns = orchestrator.turns(&conversation.id).unwrap();
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].from_session, first);
        assert_eq!(turns[1].from_session, second);
        assert_eq!(
            orchestrator
                .conversation(&conversation.id)
                .unwrap()
                .reason
                .as_deref(),
            Some("max_turns")
        );

        sessions.kill_all_sessions().unwrap();
    }

    #[test]
    fn stops_when_a_session_repeats_its_message() {
        let (_directory, orchestrator, sessions, first, second) = setup(FakeDriver {
            id: "constant".to_owned(),
            program: "/bin/echo".to_owned(),
            fixed_args: Some(vec!["igual".to_owned()]),
        });

        let conversation = orchestrator.start(request(&first, &second, 9)).unwrap();
        assert!(wait_for_state(
            &orchestrator,
            &conversation.id,
            ConversationState::Failed
        ));

        let conversation = orchestrator.conversation(&conversation.id).unwrap();
        assert_eq!(conversation.reason.as_deref(), Some("repetition"));
        assert_eq!(orchestrator.turns(&conversation.id).unwrap().len(), 3);

        sessions.kill_all_sessions().unwrap();
    }

    #[test]
    fn fails_fast_when_an_agent_produces_no_output() {
        let (_directory, orchestrator, _sessions, first, second) = setup(silent_driver());
        let conversation = orchestrator.start(request(&first, &second, 6)).unwrap();
        assert!(wait_for_state(
            &orchestrator,
            &conversation.id,
            ConversationState::Failed
        ));
        let stored = orchestrator.conversation(&conversation.id).unwrap();
        assert!(stored
            .reason
            .as_deref()
            .is_some_and(|reason| reason.contains("sin producir una respuesta")));
    }

    #[test]
    fn pauses_resumes_and_aborts() {
        let (_directory, orchestrator, sessions, first, second) = setup(FakeDriver {
            id: "sleep".to_owned(),
            program: "/bin/sleep".to_owned(),
            fixed_args: Some(vec!["1".to_owned()]),
        });

        let conversation = orchestrator.start(request(&first, &second, 4)).unwrap();

        assert_eq!(
            orchestrator.pause(&conversation.id).unwrap(),
            ConversationState::Paused
        );
        assert_eq!(
            orchestrator.conversation(&conversation.id).unwrap().state,
            ConversationState::Paused
        );

        assert_eq!(
            orchestrator.resume(&conversation.id).unwrap(),
            ConversationState::Running
        );
        assert_eq!(
            orchestrator.abort(&conversation.id).unwrap(),
            ConversationState::Aborted
        );
        assert!(wait_for_state(
            &orchestrator,
            &conversation.id,
            ConversationState::Aborted
        ));

        sessions.kill_all_sessions().unwrap();
    }

    #[test]
    fn rejects_an_empty_objective_and_the_same_session_twice() {
        let (_directory, orchestrator, sessions, first, second) = setup(echo_driver());

        let empty = orchestrator
            .start(StartConversation {
                session_a: first.clone(),
                session_b: second.clone(),
                objective: "   ".to_owned(),
                max_turns: None,
                turn_budget_ms: None,
                model_a: None,
                model_b: None,
                allow_tools: false,
            })
            .unwrap_err();
        assert!(matches!(empty, CoreError::InvalidInput(_)));

        let same = orchestrator
            .start(StartConversation {
                session_a: first.clone(),
                session_b: first,
                objective: "objetivo".to_owned(),
                max_turns: None,
                turn_budget_ms: None,
                model_a: None,
                model_b: None,
                allow_tools: false,
            })
            .unwrap_err();
        assert!(matches!(same, CoreError::InvalidInput(_)));

        sessions.kill_all_sessions().unwrap();
    }

    #[test]
    fn extract_text_uses_the_command_code_final_result() {
        let registry = crate::drivers::DriverRegistry::new();
        let driver = registry.resolve("cmd");
        let output = concat!(
            "{\"type\":\"event\",\"event\":{\"type\":\"thinking_delta\",\"delta\":\"pensando\"}}\n",
            "{\"type\":\"event\",\"event\":{\"type\":\"text_delta\",\"delta\":\"Ho\"}}\n",
            "{\"type\":\"event\",\"event\":{\"type\":\"text_delta\",\"delta\":\"la\"}}\n",
            "{\"type\":\"result\",\"subtype\":\"success\",\"finalText\":\"Hola\"}\n",
        );
        assert_eq!(extract_text(driver.as_ref(), output.as_bytes()), "Hola");
    }

    #[test]
    fn extract_text_joins_command_code_deltas_without_newlines() {
        let registry = crate::drivers::DriverRegistry::new();
        let driver = registry.resolve("cmd");
        let output = concat!(
            "{\"type\":\"event\",\"event\":{\"type\":\"text_delta\",\"delta\":\"Ho\"}}\n",
            "{\"type\":\"event\",\"event\":{\"type\":\"text_delta\",\"delta\":\"la\"}}\n",
        );
        assert_eq!(extract_text(driver.as_ref(), output.as_bytes()), "Hola");
    }
}
