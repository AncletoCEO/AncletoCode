use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{CoreError, Result};

pub const DEFAULT_SCROLLBACK_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Starting,
    Ready,
    Busy,
    WaitingInput,
    Detached,
    Stopped,
    Crashed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSpec {
    pub project_id: String,
    pub driver_id: String,
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub cwd: String,
    #[serde(default)]
    pub env: Vec<(String, String)>,
    #[serde(default = "default_cols")]
    pub cols: u16,
    #[serde(default = "default_rows")]
    pub rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub id: String,
    pub project_id: String,
    pub driver_id: String,
    pub status: SessionStatus,
    pub pid: Option<u32>,
    pub cwd: String,
    pub buffer_bytes: usize,
}

#[derive(Debug, Clone)]
pub enum SessionEvent {
    Output { id: String, data: Vec<u8> },
    State { id: String, status: SessionStatus },
    Exit { id: String, exit_code: Option<i32> },
}

pub type Sink = Arc<dyn Fn(SessionEvent) + Send + Sync>;

pub trait SessionBackend: Send + Sync {
    fn create_session(&self, spec: SessionSpec) -> Result<SessionInfo>;
    fn write_session(&self, id: &str, data: &[u8]) -> Result<()>;
    fn resize_session(&self, id: &str, cols: u16, rows: u16) -> Result<()>;
    fn kill_session(&self, id: &str) -> Result<()>;
    fn kill_all_sessions(&self) -> Result<()>;
    fn snapshot_session(&self, id: &str) -> Result<Vec<u8>>;
    fn list_sessions(&self) -> Result<Vec<SessionInfo>>;
}

impl SessionBackend for SessionManager {
    fn create_session(&self, spec: SessionSpec) -> Result<SessionInfo> {
        self.create(spec)
    }

    fn write_session(&self, id: &str, data: &[u8]) -> Result<()> {
        self.write(id, data)
    }

    fn resize_session(&self, id: &str, cols: u16, rows: u16) -> Result<()> {
        self.resize(id, cols, rows)
    }

    fn kill_session(&self, id: &str) -> Result<()> {
        self.kill(id)
    }

    fn kill_all_sessions(&self) -> Result<()> {
        self.kill_all()
    }

    fn snapshot_session(&self, id: &str) -> Result<Vec<u8>> {
        self.snapshot(id)
    }

    fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        self.list()
    }
}

struct SessionEntry {
    info: SessionInfo,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    killer: Box<dyn ChildKiller + Send + Sync>,
    buffer: RingBuffer,
}

struct Inner {
    sessions: Mutex<HashMap<String, SessionEntry>>,
    sink: Option<Sink>,
}

#[derive(Clone)]
pub struct SessionManager {
    inner: Arc<Inner>,
}

impl SessionManager {
    pub fn new(sink: Option<Sink>) -> Self {
        Self {
            inner: Arc::new(Inner {
                sessions: Mutex::new(HashMap::new()),
                sink,
            }),
        }
    }

    pub fn create(&self, spec: SessionSpec) -> Result<SessionInfo> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: spec.rows.max(1),
                cols: spec.cols.max(1),
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(pty_error)?;

        let mut command = CommandBuilder::new(&spec.program);
        command.args(&spec.args);
        command.cwd(&spec.cwd);
        for (key, value) in &spec.env {
            command.env(key, value);
        }

        let mut child = pair.slave.spawn_command(command).map_err(pty_error)?;
        drop(pair.slave);

        let reader = pair.master.try_clone_reader().map_err(pty_error)?;
        let writer = pair.master.take_writer().map_err(pty_error)?;
        let killer = child.clone_killer();
        let id = Uuid::new_v4().to_string();
        let info = SessionInfo {
            id: id.clone(),
            project_id: spec.project_id,
            driver_id: spec.driver_id,
            status: SessionStatus::Ready,
            pid: child.process_id(),
            cwd: spec.cwd.clone(),
            buffer_bytes: 0,
        };

        let entry = SessionEntry {
            info: info.clone(),
            master: pair.master,
            writer,
            killer,
            buffer: RingBuffer::new(DEFAULT_SCROLLBACK_BYTES),
        };

        self.sessions()?.insert(id.clone(), entry);
        self.emit(SessionEvent::State {
            id: id.clone(),
            status: SessionStatus::Ready,
        });

        let manager = self.clone();
        let reader_id = id.clone();
        thread::spawn(move || {
            let mut reader = reader;
            let mut chunk = [0u8; 8192];
            loop {
                match reader.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(count) => manager.push_output(&reader_id, &chunk[..count]),
                    Err(_) => break,
                }
            }
        });

        let manager = self.clone();
        let watcher_id = id.clone();
        thread::spawn(move || loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    manager.finish(&watcher_id, i32::try_from(status.exit_code()).ok());
                    break;
                }
                Ok(None) => thread::sleep(Duration::from_millis(50)),
                Err(_) => {
                    manager.crash(&watcher_id);
                    break;
                }
            }
        });

        Ok(info)
    }

    pub fn write(&self, id: &str, data: &[u8]) -> Result<()> {
        let mut sessions = self.sessions()?;
        let entry = sessions
            .get_mut(id)
            .ok_or_else(|| CoreError::SessionNotFound(id.to_owned()))?;
        entry.writer.write_all(data).map_err(pty_error)?;
        entry.writer.flush().map_err(pty_error)
    }

    pub fn resize(&self, id: &str, cols: u16, rows: u16) -> Result<()> {
        let sessions = self.sessions()?;
        let entry = sessions
            .get(id)
            .ok_or_else(|| CoreError::SessionNotFound(id.to_owned()))?;
        entry
            .master
            .resize(PtySize {
                rows: rows.max(1),
                cols: cols.max(1),
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(pty_error)
    }

    pub fn kill(&self, id: &str) -> Result<()> {
        {
            let mut sessions = self.sessions()?;
            let entry = sessions
                .get_mut(id)
                .ok_or_else(|| CoreError::SessionNotFound(id.to_owned()))?;
            entry.killer.kill().map_err(pty_error)?;
            entry.info.status = SessionStatus::Stopped;
        }

        self.emit(SessionEvent::State {
            id: id.to_owned(),
            status: SessionStatus::Stopped,
        });
        Ok(())
    }

    pub fn set_status(&self, id: &str, status: SessionStatus) -> Result<()> {
        {
            let mut sessions = self.sessions()?;
            let entry = sessions
                .get_mut(id)
                .ok_or_else(|| CoreError::SessionNotFound(id.to_owned()))?;
            entry.info.status = status;
        }
        self.emit(SessionEvent::State {
            id: id.to_owned(),
            status,
        });
        Ok(())
    }

    pub fn kill_all(&self) -> Result<()> {
        let mut stopped = Vec::new();
        {
            let mut sessions = self.sessions()?;
            for (id, entry) in sessions.iter_mut() {
                if !matches!(
                    entry.info.status,
                    SessionStatus::Stopped | SessionStatus::Crashed
                ) {
                    let _ = entry.killer.kill();
                    entry.info.status = SessionStatus::Stopped;
                    stopped.push(id.clone());
                }
            }
        }

        for id in stopped {
            self.emit(SessionEvent::State {
                id,
                status: SessionStatus::Stopped,
            });
        }
        Ok(())
    }

    pub fn snapshot(&self, id: &str) -> Result<Vec<u8>> {
        let sessions = self.sessions()?;
        sessions
            .get(id)
            .map(|entry| entry.buffer.snapshot())
            .ok_or_else(|| CoreError::SessionNotFound(id.to_owned()))
    }

    pub fn list(&self) -> Result<Vec<SessionInfo>> {
        let sessions = self.sessions()?;
        let mut infos: Vec<SessionInfo> = sessions
            .values()
            .map(|entry| {
                let mut info = entry.info.clone();
                info.buffer_bytes = entry.buffer.len();
                info
            })
            .collect();
        infos.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(infos)
    }

    fn sessions(&self) -> Result<MutexGuard<'_, HashMap<String, SessionEntry>>> {
        self.inner
            .sessions
            .lock()
            .map_err(|_| CoreError::Pty("el estado de sesiones está bloqueado".to_owned()))
    }

    fn emit(&self, event: SessionEvent) {
        if let Some(sink) = &self.inner.sink {
            sink(event);
        }
    }

    fn push_output(&self, id: &str, data: &[u8]) {
        if let Ok(mut sessions) = self.sessions() {
            if let Some(entry) = sessions.get_mut(id) {
                entry.buffer.push(data);
            }
        }
        self.emit(SessionEvent::Output {
            id: id.to_owned(),
            data: data.to_vec(),
        });
    }

    fn finish(&self, id: &str, exit_code: Option<i32>) {
        {
            let Ok(mut sessions) = self.sessions() else {
                return;
            };
            if let Some(entry) = sessions.get_mut(id) {
                entry.info.status = SessionStatus::Stopped;
            }
        }
        self.emit(SessionEvent::Exit {
            id: id.to_owned(),
            exit_code,
        });
        self.emit(SessionEvent::State {
            id: id.to_owned(),
            status: SessionStatus::Stopped,
        });
    }

    fn crash(&self, id: &str) {
        {
            let Ok(mut sessions) = self.sessions() else {
                return;
            };
            if let Some(entry) = sessions.get_mut(id) {
                entry.info.status = SessionStatus::Crashed;
            }
        }
        self.emit(SessionEvent::State {
            id: id.to_owned(),
            status: SessionStatus::Crashed,
        });
    }
}

struct RingBuffer {
    data: VecDeque<u8>,
    capacity: usize,
}

impl RingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn push(&mut self, bytes: &[u8]) {
        self.data.extend(bytes.iter().copied());
        while self.data.len() > self.capacity {
            self.data.pop_front();
        }
    }

    fn snapshot(&self) -> Vec<u8> {
        self.data.iter().copied().collect()
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

fn pty_error(error: impl std::fmt::Display) -> CoreError {
    CoreError::Pty(error.to_string())
}

fn default_cols() -> u16 {
    80
}

fn default_rows() -> u16 {
    24
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer_keeps_the_tail() {
        let mut buffer = RingBuffer::new(4);
        buffer.push(b"abcdef");

        assert_eq!(buffer.snapshot(), b"cdef");
        assert_eq!(buffer.len(), 4);
    }

    #[cfg(unix)]
    fn manager() -> (SessionManager, Arc<Mutex<Vec<SessionEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let recorded = events.clone();
        let sink: Sink = Arc::new(move |event| recorded.lock().unwrap().push(event));
        (SessionManager::new(Some(sink)), events)
    }

    #[cfg(unix)]
    fn spec(program: &str, args: &[&str]) -> SessionSpec {
        SessionSpec {
            project_id: "project-1".to_owned(),
            driver_id: "raw".to_owned(),
            program: program.to_owned(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            cwd: "/tmp".to_owned(),
            env: Vec::new(),
            cols: 80,
            rows: 24,
        }
    }

    #[cfg(unix)]
    fn wait_for(condition: impl Fn() -> bool) -> bool {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            if condition() {
                return true;
            }
            thread::sleep(Duration::from_millis(25));
        }
        false
    }

    #[cfg(unix)]
    #[test]
    fn captures_output_and_reaches_stopped() {
        let (manager, _events) = manager();
        let info = manager
            .create(spec("/bin/sh", &["-c", "printf hola"]))
            .unwrap();

        let saw_output = wait_for(|| {
            manager
                .snapshot(&info.id)
                .map(|buffer| String::from_utf8_lossy(&buffer).contains("hola"))
                .unwrap_or(false)
        });
        let stopped = wait_for(|| {
            manager
                .list()
                .map(|sessions| {
                    sessions
                        .iter()
                        .any(|session| session.status == SessionStatus::Stopped)
                })
                .unwrap_or(false)
        });

        assert!(saw_output);
        assert!(stopped);
    }

    #[cfg(unix)]
    #[test]
    fn writes_to_an_interactive_session() {
        let (manager, _events) = manager();
        let info = manager.create(spec("/bin/cat", &[])).unwrap();

        manager.write(&info.id, b"hola\n").unwrap();

        let saw_echo = wait_for(|| {
            manager
                .snapshot(&info.id)
                .map(|buffer| String::from_utf8_lossy(&buffer).contains("hola"))
                .unwrap_or(false)
        });
        assert!(saw_echo);

        manager.kill(&info.id).unwrap();
        assert_eq!(manager.list().unwrap()[0].status, SessionStatus::Stopped);
    }

    #[cfg(unix)]
    #[test]
    fn kill_all_stops_every_session() {
        let (manager, _events) = manager();
        let first = manager.create(spec("/bin/cat", &[])).unwrap();
        let second = manager.create(spec("/bin/cat", &[])).unwrap();

        manager.kill_all().unwrap();

        let sessions = manager.list().unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions
            .iter()
            .all(|session| session.status == SessionStatus::Stopped));
        assert!(manager.snapshot(&first.id).is_ok());
        assert!(manager.snapshot(&second.id).is_ok());

        #[cfg(target_os = "linux")]
        for pid in [first.pid, second.pid].into_iter().flatten() {
            assert!(wait_for(
                || !std::path::Path::new(&format!("/proc/{pid}")).exists()
            ));
        }
    }
}
