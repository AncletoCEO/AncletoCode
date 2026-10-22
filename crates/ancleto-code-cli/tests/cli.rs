#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant};

const ACODE: &str = env!("CARGO_BIN_EXE_acode");

struct Sandbox {
    directory: tempfile::TempDir,
    host_binary: PathBuf,
}

impl Sandbox {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let acode = PathBuf::from(ACODE);
        let target = acode.parent().unwrap().to_path_buf();
        let host_binary = target.join(if cfg!(windows) {
            "ancleto-code-host.exe"
        } else {
            "ancleto-code-host"
        });
        if !host_binary.is_file() {
            let status =
                Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()))
                    .args(["build", "-p", "ancleto-code-host"])
                    .status()
                    .unwrap();
            assert!(status.success(), "no se pudo compilar el host");
        }
        Self {
            directory,
            host_binary,
        }
    }

    fn home(&self) -> PathBuf {
        self.directory.path().join("home")
    }

    fn state(&self) -> PathBuf {
        self.directory.path().join("state")
    }

    fn run(&self, args: &[&str]) -> Output {
        self.run_with_env(args, &[])
    }

    fn run_with_env(&self, args: &[&str], env: &[(&str, &str)]) -> Output {
        let mut command = Command::new(ACODE);
        command
            .args(args)
            .env("ANCLETO_CODE_HOME", self.home())
            .env("ANCLETO_CODE_STATE", self.state())
            .env("ANCLETO_HOST_BIN", &self.host_binary)
            .env_remove("ANCLETO_SPEC_BIN");
        for (key, value) in env {
            command.env(key, value);
        }
        command.output().unwrap()
    }

    fn json(&self, args: &[&str]) -> (i32, serde_json::Value) {
        let output = self.run(args);
        let text = String::from_utf8_lossy(&output.stdout);
        let value = serde_json::from_str(text.trim()).unwrap_or_else(|_| {
            panic!(
                "salida no JSON: {text}\nstderr: {}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
        (output.status.code().unwrap_or(-1), value)
    }
}

fn wait_for(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        thread::sleep(Duration::from_millis(50));
    }
    false
}

#[test]
fn agents_returns_versioned_json() {
    let sandbox = Sandbox::new();
    let (code, value) = sandbox.json(&["agents", "--json"]);

    assert_eq!(code, 0);
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["ok"], true);
    let drivers = value["data"].as_array().unwrap();
    assert!(drivers.iter().any(|driver| driver["id"] == "claude"));
}

#[test]
fn unknown_command_exits_with_usage_code() {
    let sandbox = Sandbox::new();
    let output = sandbox.run(&["comando-inexistente"]);

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn project_add_list_and_remove_share_state() {
    let sandbox = Sandbox::new();
    let project = sandbox.directory.path().join("demo");
    std::fs::create_dir_all(&project).unwrap();

    let (code, added) = sandbox.json(&["project", "add", project.to_str().unwrap(), "--json"]);
    assert_eq!(code, 0);
    assert_eq!(added["data"]["name"], "demo");
    let id = added["data"]["id"].as_str().unwrap().to_owned();

    let (code, listed) = sandbox.json(&["project", "list", "--json"]);
    assert_eq!(code, 0);
    assert!(listed["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["id"] == id));

    let (code, _) = sandbox.json(&["project", "remove", &id, "--json"]);
    assert_eq!(code, 0);

    let (_, listed) = sandbox.json(&["project", "list", "--json"]);
    assert!(listed["data"].as_array().unwrap().is_empty());
}

#[test]
fn missing_driver_returns_dependency_error() {
    let sandbox = Sandbox::new();

    let (code, value) = sandbox.json(&["run", "hola", "--driver", "no-existe", "--json"]);

    assert_eq!(code, 3);
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "missing_dependency");
    assert!(value["error"]["message"]
        .as_str()
        .unwrap()
        .contains("no-existe"));
}

#[test]
fn doctor_reports_node_drivers_and_platform() {
    let sandbox = Sandbox::new();
    let (code, value) = sandbox.json(&["doctor", "--json"]);

    assert_eq!(code, 0);
    assert!(value["data"]["node"]["version"].is_string());
    assert!(value["data"]["drivers"].is_array());
    assert!(value["data"]["platform"]["os"].is_string());
}

#[test]
fn completions_print_a_script() {
    let sandbox = Sandbox::new();
    let output = sandbox.run(&["completions", "bash"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("acode"));
}

#[test]
fn host_start_status_and_stop() {
    let sandbox = Sandbox::new();
    let (code, started) = sandbox.json(&["host", "start", "--json"]);
    assert_eq!(code, 0, "{started}");
    let pid = started["data"]["pid"].as_u64().unwrap();

    let (code, status) = sandbox.json(&["host", "status", "--json"]);
    assert_eq!(code, 0);
    assert_eq!(status["data"]["pid"].as_u64(), Some(pid));

    let (code, _) = sandbox.json(&["host", "stop", "--json"]);
    assert_eq!(code, 0);

    assert!(wait_for(|| {
        sandbox.run(&["host", "status", "--json"]).status.code() == Some(4)
    }));
}

#[test]
fn session_start_list_and_kill_work_through_the_host() {
    let sandbox = Sandbox::new();
    let project = sandbox.directory.path().join("proj");
    std::fs::create_dir_all(&project).unwrap();
    let (_, added) = sandbox.json(&["project", "add", project.to_str().unwrap(), "--json"]);
    let id = added["data"]["id"].as_str().unwrap().to_owned();

    let agy = sandbox.run(&["agents", "--json"]);
    let agents: serde_json::Value = serde_json::from_slice(&agy.stdout).unwrap();
    let available = agents["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|driver| driver["status"] == "available")
        .and_then(|driver| driver["id"].as_str());
    let Some(driver) = available else {
        return;
    };

    let (code, created) = sandbox.json(&[
        "session",
        "start",
        "--driver",
        driver,
        "--project",
        &id,
        "--json",
    ]);
    assert_eq!(code, 0, "{created}");
    let session_id = created["data"]["id"].as_str().unwrap().to_owned();

    let (code, listed) = sandbox.json(&["session", "list", "--json"]);
    assert_eq!(code, 0);
    assert!(listed["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|session| session["id"] == session_id));

    let (code, _) = sandbox.json(&["session", "kill", &session_id, "--json"]);
    assert_eq!(code, 0);

    let _ = sandbox.run(&["host", "stop", "--json"]);
}

#[test]
fn spec_passthrough_returns_an_envelope_or_a_dependency_error() {
    let sandbox = Sandbox::new();
    let (code, value) = sandbox.json(&["--json", "spec", "specs", "check"]);

    assert!(code == 0 || code == 3, "código inesperado: {code} {value}");
    assert_eq!(value["schemaVersion"], 1);
}

#[test]
fn host_status_without_a_host_exits_with_session_error() {
    let sandbox = Sandbox::new();
    let output = sandbox.run(&["--json", "host", "status"]);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(4));
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "session_error");
}

#[test]
fn spec_passthrough_without_the_cli_returns_missing_dependency() {
    let sandbox = Sandbox::new();
    let output = sandbox.run_with_env(&["--json", "spec", "init"], &[("PATH", "/usr/bin:/bin")]);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(3), "{value}");
    assert_eq!(value["error"]["code"], "missing_dependency");
}

#[cfg(unix)]
#[test]
fn spec_without_node_is_incompatible() {
    use std::os::unix::fs::symlink;

    let sandbox = Sandbox::new();
    let path = std::env::var_os("PATH").unwrap_or_default();
    let Some(ancleto) = std::env::split_paths(&path)
        .map(|directory| directory.join("ancleto"))
        .find(|candidate| candidate.is_file())
    else {
        eprintln!("ancleto no está en PATH; se omite la prueba de Node ausente");
        return;
    };
    let bin = sandbox.directory.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    symlink(&ancleto, bin.join("ancleto")).unwrap();

    let output = sandbox.run_with_env(
        &["--json", "spec", "init"],
        &[("PATH", bin.to_str().unwrap())],
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(5), "{value}");
    assert_eq!(value["error"]["code"], "incompatible_state");
}
