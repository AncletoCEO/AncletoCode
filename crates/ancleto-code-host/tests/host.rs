#![cfg(unix)]
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use ancleto_code_core::host::{
    connect_stream, read_message, read_token, write_message, HostClient, HostPaths, RpcRequest,
};
use ancleto_code_core::session::{SessionBackend, SessionSpec, SessionStatus};
use serde_json::json;

const BINARY: &str = env!("CARGO_BIN_EXE_ancleto-code-host");

struct RunningHost {
    child: Child,
    directory: tempfile::TempDir,
}

impl RunningHost {
    fn start() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let state = directory.path().join("state");
        let db = directory.path().join("ancleto-code.sqlite");
        let child = Command::new(BINARY)
            .arg("start")
            .arg("--db")
            .arg(&db)
            .arg("--state-dir")
            .arg(&state)
            .arg("--idle-ms")
            .arg("0")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let host = Self { child, directory };
        wait_for_connect(&host.paths(), Duration::from_secs(10));
        host
    }

    fn paths(&self) -> HostPaths {
        HostPaths::discover(Some(&self.directory.path().join("state")))
    }

    fn db(&self) -> PathBuf {
        self.directory.path().join("ancleto-code.sqlite")
    }

    fn connect(&self) -> HostClient {
        HostClient::connect(&self.paths()).unwrap()
    }
}

impl Drop for RunningHost {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn wait_for_connect(paths: &HostPaths, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if HostClient::connect(paths).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("el host no aceptó la conexión a tiempo");
}

fn wait_until(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        thread::sleep(Duration::from_millis(25));
    }
    false
}

fn cat_spec(cwd: &Path) -> SessionSpec {
    SessionSpec {
        project_id: "project-1".to_owned(),
        driver_id: "raw".to_owned(),
        program: "/bin/cat".to_owned(),
        args: Vec::new(),
        cwd: cwd.to_string_lossy().into_owned(),
        env: Vec::new(),
        cols: 80,
        rows: 24,
    }
}

#[test]
fn rejects_invalid_tokens_and_incompatible_versions() {
    let host = RunningHost::start();
    let paths = host.paths();
    let token = read_token(&paths).unwrap();

    let stream = connect_stream(&paths).unwrap();
    let request = RpcRequest::new(
        1,
        "host.handshake",
        json!({ "protocolVersion": 1, "token": "incorrecto" }),
    );
    let mut writer = &stream;
    write_message(&mut writer, &json!(request)).unwrap();
    let mut reader = &stream;
    let response = read_message(&mut reader).unwrap();
    assert_eq!(response["error"]["code"], -32001);

    let stream = connect_stream(&paths).unwrap();
    let request = RpcRequest::new(
        2,
        "host.handshake",
        json!({ "protocolVersion": 999, "token": token }),
    );
    let mut writer = &stream;
    write_message(&mut writer, &json!(request)).unwrap();
    let mut reader = &stream;
    let response = read_message(&mut reader).unwrap();
    assert_eq!(response["error"]["code"], -32002);
}

#[test]
fn creates_sessions_and_detaches_without_killing() {
    let host = RunningHost::start();
    let client = host.connect();

    let info = client
        .create_session(cat_spec(host.directory.path()))
        .unwrap();
    client.write_session(&info.id, b"hola\n").unwrap();
    assert!(wait_until(|| {
        client
            .snapshot_session(&info.id)
            .map(|buffer| String::from_utf8_lossy(&buffer).contains("hola"))
            .unwrap_or(false)
    }));

    let attached = client.attach(&info.id).unwrap();
    assert!(String::from_utf8_lossy(&attached).contains("hola"));

    client.detach(&info.id).unwrap();
    assert!(wait_until(|| {
        client
            .list_sessions()
            .map(|sessions| {
                sessions.iter().any(|session| {
                    session.id == info.id && session.status == SessionStatus::Detached
                })
            })
            .unwrap_or(false)
    }));

    let reattached = client.attach(&info.id).unwrap();
    assert!(String::from_utf8_lossy(&reattached).contains("hola"));
    assert!(wait_until(|| {
        client
            .list_sessions()
            .map(|sessions| {
                sessions
                    .iter()
                    .any(|session| session.id == info.id && session.status == SessionStatus::Ready)
            })
            .unwrap_or(false)
    }));

    client.kill_session(&info.id).unwrap();
    assert!(wait_until(|| {
        client
            .list_sessions()
            .map(|sessions| {
                sessions.iter().any(|session| {
                    session.id == info.id && session.status == SessionStatus::Stopped
                })
            })
            .unwrap_or(false)
    }));

    client.shutdown().unwrap();
    assert!(wait_until(|| HostClient::connect(&host.paths()).is_err()));
}

#[test]
fn reconciles_sessions_after_a_host_crash() {
    let mut host = RunningHost::start();
    let client = host.connect();
    let info = client
        .create_session(cat_spec(host.directory.path()))
        .unwrap();
    drop(client);

    let child = Command::new("kill")
        .arg("-9")
        .arg(host.child.id().to_string())
        .status()
        .unwrap();
    assert!(child.success());
    let _ = host.child.wait();

    let state = host.directory.path().join("state");
    let db = host.db();
    let restarted = Command::new(BINARY)
        .arg("start")
        .arg("--db")
        .arg(&db)
        .arg("--state-dir")
        .arg(&state)
        .arg("--idle-ms")
        .arg("0")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let paths = HostPaths::discover(Some(&state));
    wait_for_connect(&paths, Duration::from_secs(10));
    let client = HostClient::connect(&paths).unwrap();
    let saved = client.saved_sessions().unwrap();
    let recovered = saved.iter().find(|session| session.id == info.id).unwrap();
    assert_eq!(recovered.status, SessionStatus::Crashed);
    assert_eq!(recovered.spec.cwd, cat_spec(host.directory.path()).cwd);

    client.shutdown().unwrap();
    let mut restarted = restarted;
    let _ = restarted.kill();
    let _ = restarted.wait();
}

#[test]
fn concurrent_autostart_converges_on_one_host() {
    let directory = tempfile::tempdir().unwrap();
    let state = directory.path().join("state");
    let db = directory.path().join("ancleto-code.sqlite");
    let paths = HostPaths::discover(Some(&state));
    let binary = PathBuf::from(BINARY);

    let spawn = |paths: HostPaths, db: PathBuf, binary: PathBuf| {
        thread::spawn(move || {
            HostClient::start_with(&paths, &db, Some(binary.as_path()), 0)
                .map(|client| client.pid())
        })
    };

    let first = spawn(paths.clone(), db.clone(), binary.clone());
    let second = spawn(paths.clone(), db, binary);

    let first = first.join().unwrap().unwrap();
    let second = second.join().unwrap().unwrap();
    assert!(first > 0);
    assert_eq!(first, second);

    let client = HostClient::connect(&paths).unwrap();
    client.shutdown().unwrap();
    assert!(wait_until(|| HostClient::connect(&paths).is_err()));
}

#[test]
fn events_are_broadcast_to_subscribers() {
    let host = RunningHost::start();
    let client = Arc::new(host.connect());
    let events = client.events();
    let info = client
        .create_session(cat_spec(host.directory.path()))
        .unwrap();

    client.write_session(&info.id, b"evento\n").unwrap();

    assert!(wait_until(|| {
        let mut saw_output = false;
        while let Ok(notification) = events.try_recv() {
            if notification.method == "session.output"
                && notification.params["id"] == info.id.as_str()
            {
                saw_output = true;
            }
        }
        saw_output
    }));
}
