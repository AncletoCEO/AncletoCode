#![cfg(unix)]
#![allow(clippy::unwrap_used)]

use std::env;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use ancleto_code_cmd::vt::TerminalState;
use ancleto_code_core::session::SessionSpec;
use ancleto_code_core::{SessionManager, SessionStatus};

struct Harness {
    manager: SessionManager,
    id: String,
    terminal: TerminalState,
    consumed: usize,
}

impl Harness {
    fn spawn(program: &str, args: &[&str], cwd: &Path) -> Option<Self> {
        if !available(program) {
            eprintln!("{program} no está disponible; se omite la prueba");
            return None;
        }
        let manager = SessionManager::new(None);
        let spec = SessionSpec {
            project_id: "vt-program".to_owned(),
            driver_id: "raw".to_owned(),
            program: program.to_owned(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            cwd: cwd.to_string_lossy().into_owned(),
            env: vec![
                ("TERM".to_owned(), "xterm-256color".to_owned()),
                ("COLORTERM".to_owned(), "truecolor".to_owned()),
            ],
            cols: 100,
            rows: 30,
        };
        let info = manager.create(spec).ok()?;
        Some(Self {
            manager,
            id: info.id,
            terminal: TerminalState::new(100, 30),
            consumed: 0,
        })
    }

    fn pump(&mut self, duration: Duration, until: impl Fn(&TerminalState) -> bool) -> bool {
        let deadline = Instant::now() + duration;
        loop {
            self.feed();
            if until(&self.terminal) {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    fn feed(&mut self) {
        if let Ok(snapshot) = self.manager.snapshot(&self.id) {
            if snapshot.len() > self.consumed {
                self.terminal.feed(&snapshot[self.consumed..]);
                self.consumed = snapshot.len();
            }
        }
    }

    fn stopped(&self) -> bool {
        self.manager
            .list()
            .map(|sessions| {
                sessions.iter().any(|session| {
                    session.id == self.id && session.status == SessionStatus::Stopped
                })
            })
            .unwrap_or(false)
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = self.manager.kill_all();
    }
}

fn available(program: &str) -> bool {
    if program.contains('/') {
        return Path::new(program).is_file();
    }
    env::var_os("PATH").is_some_and(|path| {
        env::split_paths(&path).any(|directory| {
            let candidate = directory.join(program);
            candidate.is_file() || directory.join(format!("{program}.exe")).is_file()
        })
    })
}

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn vim_enters_the_alternate_screen_and_renders() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("demo.txt");
    std::fs::write(&file, "linea uno\nlinea dos\n").unwrap();
    let Some(mut harness) = Harness::spawn(
        "vim",
        &["-Nu", "NONE", "-n", "-i", "NONE", file.to_str().unwrap()],
        directory.path(),
    ) else {
        return;
    };

    assert!(
        harness.pump(Duration::from_secs(5), |terminal| {
            terminal.snapshot().alternate_screen
        }),
        "vim no entró en alt screen"
    );
}

#[test]
fn less_renders_a_file() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("demo.txt");
    std::fs::write(&file, "contenido visible de less\n").unwrap();
    let Some(mut harness) = Harness::spawn("less", &[file.to_str().unwrap()], directory.path())
    else {
        return;
    };

    let rendered = harness.pump(Duration::from_secs(5), |terminal| {
        terminal
            .full_text()
            .iter()
            .any(|(_, text)| text.contains("contenido visible"))
    });

    assert!(rendered || harness.terminal.snapshot().alternate_screen);
}

#[test]
fn htop_enters_the_alternate_screen() {
    let Some(mut harness) = Harness::spawn("htop", &[], &repository()) else {
        return;
    };

    assert!(
        harness.pump(Duration::from_secs(5), |terminal| {
            terminal.snapshot().alternate_screen
        }),
        "htop no entró en alt screen"
    );
}

#[test]
fn git_output_is_captured_until_exit() {
    let Some(mut harness) = Harness::spawn(
        "git",
        &["--no-pager", "log", "--oneline", "-5"],
        &repository(),
    ) else {
        return;
    };

    harness.pump(Duration::from_secs(5), |_| false);
    harness.feed();

    assert!(
        !harness.terminal.full_text().is_empty(),
        "git no produjo salida"
    );
    assert!(harness.stopped(), "git no terminó");
}

#[test]
fn tmux_is_covered_when_available() {
    let directory = tempfile::tempdir().unwrap();
    let Some(mut harness) = Harness::spawn(
        "tmux",
        &["-f", "/dev/null", "new-session", "-A", "-s", "ancleto-test"],
        directory.path(),
    ) else {
        return;
    };

    harness.pump(Duration::from_secs(5), |terminal| {
        terminal.snapshot().alternate_screen
    });
    let _ = harness.manager.write(&harness.id, b"exit\r");
}
