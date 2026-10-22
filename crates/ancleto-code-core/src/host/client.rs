use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

use interprocess::local_socket::traits::Stream as _;
use interprocess::local_socket::Stream;
use serde_json::{json, Value};

use super::protocol::{
    HostStatus, RpcError, RpcNotification, RpcRequest, RpcResponse, PROTOCOL_VERSION,
};
use super::transport::{
    connect_stream, ensure_state_dir, read_message, read_token, write_message, HostPaths,
};
use crate::domain::{CoreError, Result};
use crate::drivers::resolve_binary;
use crate::persistence::SavedSession;
use crate::session::{SessionBackend, SessionInfo, SessionSpec};

type Pending = Mutex<HashMap<u64, Sender<std::result::Result<Value, RpcError>>>>;

#[derive(Clone)]
pub struct HostClient {
    inner: Arc<ClientInner>,
}

struct ClientInner {
    stream: Arc<Stream>,
    pending: Pending,
    write_lock: Mutex<()>,
    next_id: AtomicU64,
    events: Mutex<Vec<Sender<RpcNotification>>>,
    attached: Mutex<HashSet<String>>,
    closed: AtomicBool,
    pid: AtomicU64,
}

impl HostClient {
    pub fn connect(paths: &HostPaths) -> Result<Self> {
        let token = read_token(paths)
            .map_err(|error| CoreError::Host(format!("token del host: {error}")))?;
        let stream = connect_stream(paths)
            .map_err(|error| CoreError::Host(format!("no se pudo conectar al host: {error}")))?;

        let inner = Arc::new(ClientInner {
            stream: Arc::new(stream),
            pending: Mutex::new(HashMap::new()),
            write_lock: Mutex::new(()),
            next_id: AtomicU64::new(1),
            events: Mutex::new(Vec::new()),
            attached: Mutex::new(HashSet::new()),
            closed: AtomicBool::new(false),
            pid: AtomicU64::new(0),
        });
        Self::spawn_reader(Arc::clone(&inner.stream), Arc::downgrade(&inner));
        let client = Self { inner };

        let result = client.request(
            "host.handshake",
            json!({ "protocolVersion": PROTOCOL_VERSION, "token": token }),
        )?;
        let version = result
            .get("protocolVersion")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if version != u64::from(PROTOCOL_VERSION) {
            return Err(CoreError::Host(format!(
                "el host habla el protocolo {version}, se esperaba {PROTOCOL_VERSION}"
            )));
        }
        client.inner.pid.store(
            result.get("pid").and_then(Value::as_u64).unwrap_or(0),
            Ordering::SeqCst,
        );

        Ok(client)
    }

    pub fn connect_or_start(paths: &HostPaths, database: &Path) -> Result<Self> {
        Self::start_with(paths, database, None, 30 * 60 * 1000)
    }

    pub fn start_with(
        paths: &HostPaths,
        database: &Path,
        host_binary: Option<&Path>,
        idle_ms: u64,
    ) -> Result<Self> {
        if let Ok(client) = Self::connect(paths) {
            return Ok(client);
        }

        ensure_state_dir(paths)
            .map_err(|error| CoreError::Host(format!("directorio de estado: {error}")))?;
        let binary = host_binary
            .map(Path::to_path_buf)
            .or_else(find_host_binary)
            .ok_or_else(|| CoreError::Host("no se encontró ancleto-code-host".to_owned()))?;

        for _attempt in 0..2 {
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&paths.lock)
            {
                Ok(mut file) => {
                    let _ = writeln!(file, "{}", std::process::id());
                    let _ = file.flush();

                    Command::new(&binary)
                        .arg("start")
                        .arg("--db")
                        .arg(database)
                        .arg("--state-dir")
                        .arg(&paths.state_dir)
                        .arg("--idle-ms")
                        .arg(idle_ms.to_string())
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .map_err(|error| {
                            CoreError::Host(format!("no se pudo iniciar el host: {error}"))
                        })?;

                    if let Some(client) = wait_for_host(paths, Duration::from_secs(8)) {
                        return Ok(client);
                    }
                    return Err(CoreError::Host(
                        "el host no respondió después de iniciarse".to_owned(),
                    ));
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    if let Some(client) = wait_for_host(paths, Duration::from_secs(3)) {
                        return Ok(client);
                    }
                    let _ = std::fs::remove_file(&paths.lock);
                }
                Err(error) => {
                    return Err(CoreError::Host(format!(
                        "no se pudo tomar el lock del host: {error}"
                    )))
                }
            }
        }

        Err(CoreError::Host("no se pudo iniciar el host".to_owned()))
    }

    pub fn status(&self) -> Result<HostStatus> {
        let value = self.request("host.status", json!({}))?;
        serde_json::from_value(value)
            .map_err(|error| CoreError::Host(format!("estado inválido: {error}")))
    }

    pub fn shutdown(&self) -> Result<()> {
        self.request("host.shutdown", json!({})).map(|_| ())
    }

    pub fn events(&self) -> Receiver<RpcNotification> {
        let (sender, receiver) = mpsc::channel();
        if let Ok(mut events) = self.inner.events.lock() {
            events.push(sender);
        }
        receiver
    }

    pub fn attach(&self, id: &str) -> Result<Vec<u8>> {
        let value = self.request("session.attach", json!({ "id": id }))?;
        if let Ok(mut attached) = self.inner.attached.lock() {
            attached.insert(id.to_owned());
        }
        snapshot_from(value)
    }

    pub fn detach(&self, id: &str) -> Result<()> {
        if let Ok(mut attached) = self.inner.attached.lock() {
            attached.remove(id);
        }
        self.request("session.detach", json!({ "id": id }))
            .map(|_| ())
    }

    pub fn saved_sessions(&self) -> Result<Vec<SavedSession>> {
        let value = self.request("session.saved", json!({}))?;
        serde_json::from_value(value)
            .map_err(|error| CoreError::Host(format!("sesiones guardadas inválidas: {error}")))
    }

    pub fn pid(&self) -> u32 {
        self.inner.pid.load(Ordering::SeqCst) as u32
    }

    fn request(&self, method: &str, params: Value) -> Result<Value> {
        if self.inner.closed.load(Ordering::SeqCst) {
            return Err(CoreError::Host("el host no está conectado".to_owned()));
        }

        let id = self.inner.next_id.fetch_add(1, Ordering::SeqCst);
        let (sender, receiver) = mpsc::channel();
        self.inner
            .pending
            .lock()
            .map_err(|_| CoreError::Host("estado de peticiones bloqueado".to_owned()))?
            .insert(id, sender);

        if let Err(error) = self.write(&RpcRequest::new(id, method, params)) {
            if let Ok(mut pending) = self.inner.pending.lock() {
                pending.remove(&id);
            }
            return Err(error);
        }

        match receiver.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(error)) => Err(CoreError::Host(format!(
                "{} (código {})",
                error.message, error.code
            ))),
            Err(_) => Err(CoreError::Host("el host no respondió a tiempo".to_owned())),
        }
    }

    fn write(&self, request: &RpcRequest) -> Result<()> {
        let _guard = self
            .inner
            .write_lock
            .lock()
            .map_err(|_| CoreError::Host("escritura bloqueada".to_owned()))?;
        let payload =
            serde_json::to_value(request).map_err(|error| CoreError::Host(error.to_string()))?;
        let mut writer = &*self.inner.stream;
        write_message(&mut writer, &payload)
            .map_err(|error| CoreError::Host(format!("no se pudo escribir al host: {error}")))
    }

    fn spawn_reader(stream: Arc<Stream>, weak: Weak<ClientInner>) {
        thread::spawn(move || loop {
            let mut reader = &*stream;
            let value = match read_message(&mut reader) {
                Ok(value) => value,
                Err(_) => {
                    if let Some(inner) = weak.upgrade() {
                        inner.closed.store(true, Ordering::SeqCst);
                        inner.fail_all("el host cerró la conexión");
                    }
                    break;
                }
            };

            if let Some(id) = value.get("id").and_then(Value::as_u64) {
                let response: Option<RpcResponse> = serde_json::from_value(value).ok();
                let Some(inner) = weak.upgrade() else {
                    break;
                };
                let sender = inner
                    .pending
                    .lock()
                    .ok()
                    .and_then(|mut pending| pending.remove(&id));
                if let (Some(sender), Some(response)) = (sender, response) {
                    let result = match response.error {
                        Some(error) => Err(error),
                        None => Ok(response.result.unwrap_or(Value::Null)),
                    };
                    let _ = sender.send(result);
                }
            } else if let Ok(notification) = serde_json::from_value::<RpcNotification>(value) {
                if let Some(inner) = weak.upgrade() {
                    if let Ok(mut events) = inner.events.lock() {
                        events.retain(|sender| sender.send(notification.clone()).is_ok());
                    }
                }
            }
        });
    }
}

impl ClientInner {
    fn fail_all(&self, message: &str) {
        if let Ok(mut pending) = self.pending.lock() {
            for (_, sender) in pending.drain() {
                let _ = sender.send(Err(RpcError {
                    code: -32000,
                    message: message.to_owned(),
                }));
            }
        }
    }
}

impl SessionBackend for HostClient {
    fn create_session(&self, spec: SessionSpec) -> Result<SessionInfo> {
        let value = self.request("session.create", json!({ "spec": spec }))?;
        serde_json::from_value(value)
            .map_err(|error| CoreError::Host(format!("sesión inválida: {error}")))
    }

    fn write_session(&self, id: &str, data: &[u8]) -> Result<()> {
        self.request("session.write", json!({ "id": id, "data": data }))
            .map(|_| ())
    }

    fn resize_session(&self, id: &str, cols: u16, rows: u16) -> Result<()> {
        self.request(
            "session.resize",
            json!({ "id": id, "cols": cols, "rows": rows }),
        )
        .map(|_| ())
    }

    fn kill_session(&self, id: &str) -> Result<()> {
        self.request("session.kill", json!({ "id": id }))
            .map(|_| ())
    }

    fn kill_all_sessions(&self) -> Result<()> {
        self.request("session.kill_all", json!({})).map(|_| ())
    }

    fn snapshot_session(&self, id: &str) -> Result<Vec<u8>> {
        let value = self.request("session.snapshot", json!({ "id": id }))?;
        snapshot_from(value)
    }

    fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let value = self.request("session.list", json!({}))?;
        serde_json::from_value(value)
            .map_err(|error| CoreError::Host(format!("lista de sesiones inválida: {error}")))
    }
}

impl Drop for ClientInner {
    fn drop(&mut self) {
        let _ = self.stream.set_nonblocking(true);
        let ids: Vec<String> = self
            .attached
            .lock()
            .map(|attached| attached.iter().cloned().collect())
            .unwrap_or_default();
        for (index, id) in ids.iter().enumerate() {
            let request = RpcRequest::new(
                u64::MAX - index as u64,
                "session.detach",
                json!({ "id": id }),
            );
            if let Ok(payload) = serde_json::to_value(&request) {
                let mut writer = &*self.stream;
                let _ = write_message(&mut writer, &payload);
            }
        }
    }
}

fn snapshot_from(value: Value) -> Result<Vec<u8>> {
    let data = value.get("data").cloned().unwrap_or(Value::Null);
    serde_json::from_value(data)
        .map_err(|error| CoreError::Host(format!("snapshot inválido: {error}")))
}

fn wait_for_host(paths: &HostPaths, timeout: Duration) -> Option<HostClient> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Ok(client) = HostClient::connect(paths) {
            return Some(client);
        }
        thread::sleep(Duration::from_millis(100));
    }
    None
}

fn find_host_binary() -> Option<PathBuf> {
    if let Some(path) = env::var_os("ANCLETO_HOST_BIN") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }

    let executable = env::current_exe().ok()?;
    let sibling = executable.parent()?.join(if cfg!(windows) {
        "ancleto-code-host.exe"
    } else {
        "ancleto-code-host"
    });
    if sibling.is_file() {
        return Some(sibling);
    }

    resolve_binary("ancleto-code-host")
}
