use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub interactive_pty: bool,
    pub one_shot_prompt: bool,
    pub structured_output: bool,
    pub resume_session: bool,
    pub headless_json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDriverDescriptor {
    pub id: String,
    pub display_name: String,
    pub binary: String,
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolStatus {
    Available,
    Missing,
    Incompatible,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverStatus {
    pub id: String,
    pub display_name: String,
    pub binary: String,
    pub status: ToolStatus,
    pub version: Option<String>,
    pub binary_path: Option<String>,
    pub install_hint: Option<String>,
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone)]
pub struct SpawnContext {
    pub program: String,
    pub args: Vec<String>,
    pub continue_session: bool,
    pub max_turns: Option<u32>,
}

impl Default for SpawnContext {
    fn default() -> Self {
        Self {
            program: "sh".to_owned(),
            args: Vec::new(),
            continue_session: false,
            max_turns: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentSignal {
    Text(String),
    MessageComplete { text: String },
    Ready,
    Error(String),
}

#[derive(Debug, Default)]
pub struct ParseState {
    buffer: Vec<u8>,
}

pub trait AgentDriver: Send + Sync {
    fn descriptor(&self) -> AgentDriverDescriptor;
    fn detect(&self) -> DriverStatus;
    fn spawn(&self, context: &SpawnContext) -> CommandSpec;
    fn one_shot(&self, context: &SpawnContext, prompt: &str) -> Option<CommandSpec>;
    fn parse(&self, chunk: &[u8], state: &mut ParseState) -> Vec<AgentSignal>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OneShotKind {
    PromptJson,
    RunPositional,
}

struct CliDriver {
    descriptor: AgentDriverDescriptor,
    one_shot_kind: OneShotKind,
    supports_continue: bool,
    supports_max_turns: bool,
    install_hint: &'static str,
}

impl CliDriver {
    fn new(
        descriptor: AgentDriverDescriptor,
        one_shot_kind: OneShotKind,
        supports_continue: bool,
        supports_max_turns: bool,
        install_hint: &'static str,
    ) -> Self {
        Self {
            descriptor,
            one_shot_kind,
            supports_continue,
            supports_max_turns,
            install_hint,
        }
    }
}

impl AgentDriver for CliDriver {
    fn descriptor(&self) -> AgentDriverDescriptor {
        self.descriptor.clone()
    }

    fn detect(&self) -> DriverStatus {
        detect_binary(&self.descriptor, Some(self.install_hint.to_owned()))
    }

    fn spawn(&self, _context: &SpawnContext) -> CommandSpec {
        CommandSpec {
            program: self.descriptor.binary.clone(),
            args: Vec::new(),
        }
    }

    fn one_shot(&self, context: &SpawnContext, prompt: &str) -> Option<CommandSpec> {
        let mut args = match self.one_shot_kind {
            OneShotKind::PromptJson => vec![
                "-p".to_owned(),
                prompt.to_owned(),
                "--output-format".to_owned(),
                "json".to_owned(),
            ],
            OneShotKind::RunPositional => vec!["run".to_owned(), prompt.to_owned()],
        };

        if self.supports_continue && context.continue_session {
            args.push("--continue".to_owned());
        }
        if self.supports_max_turns {
            if let Some(max_turns) = context.max_turns {
                args.push("--max-turns".to_owned());
                args.push(max_turns.to_string());
            }
        }

        Some(CommandSpec {
            program: self.descriptor.binary.clone(),
            args,
        })
    }

    fn parse(&self, chunk: &[u8], state: &mut ParseState) -> Vec<AgentSignal> {
        parse_chunk(chunk, state)
    }
}

struct RawDriver {
    descriptor: AgentDriverDescriptor,
}

impl AgentDriver for RawDriver {
    fn descriptor(&self) -> AgentDriverDescriptor {
        self.descriptor.clone()
    }

    fn detect(&self) -> DriverStatus {
        DriverStatus {
            id: self.descriptor.id.clone(),
            display_name: self.descriptor.display_name.clone(),
            binary: self.descriptor.binary.clone(),
            status: ToolStatus::Available,
            version: None,
            binary_path: None,
            install_hint: None,
            capabilities: self.descriptor.capabilities.clone(),
        }
    }

    fn spawn(&self, context: &SpawnContext) -> CommandSpec {
        CommandSpec {
            program: context.program.clone(),
            args: context.args.clone(),
        }
    }

    fn one_shot(&self, context: &SpawnContext, prompt: &str) -> Option<CommandSpec> {
        Some(CommandSpec {
            program: context.program.clone(),
            args: vec![prompt.to_owned()],
        })
    }

    fn parse(&self, chunk: &[u8], state: &mut ParseState) -> Vec<AgentSignal> {
        parse_chunk(chunk, state)
    }
}

#[derive(Clone)]
pub struct DriverRegistry {
    drivers: HashMap<String, Arc<dyn AgentDriver>>,
    fallback: Arc<dyn AgentDriver>,
}

impl Default for DriverRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DriverRegistry {
    pub fn new() -> Self {
        let fallback: Arc<dyn AgentDriver> = Arc::new(RawDriver {
            descriptor: AgentDriverDescriptor {
                id: "raw".to_owned(),
                display_name: "Raw".to_owned(),
                binary: String::new(),
                capabilities: Capabilities {
                    interactive_pty: true,
                    one_shot_prompt: true,
                    structured_output: false,
                    resume_session: false,
                    headless_json: false,
                },
            },
        });

        let mut registry = Self {
            drivers: HashMap::new(),
            fallback,
        };

        registry.register(Arc::new(CliDriver::new(
            descriptor(
                "claude",
                "Claude Code",
                "claude",
                capabilities(true, true, true, true, true),
            ),
            OneShotKind::PromptJson,
            false,
            false,
            "npm install -g @anthropic-ai/claude-code",
        )));
        registry.register(Arc::new(CliDriver::new(
            descriptor(
                "opencode",
                "OpenCode",
                "opencode",
                capabilities(true, true, false, true, false),
            ),
            OneShotKind::RunPositional,
            false,
            false,
            "npm install -g opencode-ai",
        )));
        registry.register(Arc::new(CliDriver::new(
            descriptor(
                "agy",
                "Antigravity CLI",
                "agy",
                capabilities(true, true, true, true, true),
            ),
            OneShotKind::PromptJson,
            true,
            false,
            "Instalá Antigravity CLI (agy) y agregalo al PATH",
        )));
        registry.register(Arc::new(CliDriver::new(
            descriptor(
                "cmd",
                "Command Code",
                "commandcode",
                capabilities(true, true, true, true, true),
            ),
            OneShotKind::PromptJson,
            true,
            true,
            "npm install -g command-code",
        )));

        registry
    }

    pub fn register(&mut self, driver: Arc<dyn AgentDriver>) {
        self.drivers.insert(driver.descriptor().id, driver);
    }

    pub fn resolve(&self, id: &str) -> Arc<dyn AgentDriver> {
        self.drivers
            .get(id)
            .cloned()
            .unwrap_or_else(|| self.fallback.clone())
    }

    pub fn descriptors(&self) -> Vec<AgentDriverDescriptor> {
        let mut descriptors: Vec<_> = self
            .drivers
            .values()
            .map(|driver| driver.descriptor())
            .collect();
        descriptors.push(self.fallback.descriptor());
        descriptors.sort_by(|left, right| left.id.cmp(&right.id));
        descriptors
    }

    pub fn detect_all(&self) -> Vec<DriverStatus> {
        let mut statuses: Vec<_> = self
            .drivers
            .values()
            .map(|driver| driver.detect())
            .collect();
        statuses.sort_by(|left, right| left.id.cmp(&right.id));
        statuses
    }
}

fn capabilities(
    interactive_pty: bool,
    one_shot_prompt: bool,
    structured_output: bool,
    resume_session: bool,
    headless_json: bool,
) -> Capabilities {
    Capabilities {
        interactive_pty,
        one_shot_prompt,
        structured_output,
        resume_session,
        headless_json,
    }
}

fn descriptor(
    id: &str,
    display_name: &str,
    binary: &str,
    capabilities: Capabilities,
) -> AgentDriverDescriptor {
    AgentDriverDescriptor {
        id: id.to_owned(),
        display_name: display_name.to_owned(),
        binary: binary.to_owned(),
        capabilities,
    }
}

pub fn detect_binary(
    descriptor: &AgentDriverDescriptor,
    install_hint: Option<String>,
) -> DriverStatus {
    let binary_path = resolve_binary(&descriptor.binary);
    let mut status = DriverStatus {
        id: descriptor.id.clone(),
        display_name: descriptor.display_name.clone(),
        binary: descriptor.binary.clone(),
        status: ToolStatus::Missing,
        version: None,
        binary_path: binary_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
        install_hint,
        capabilities: descriptor.capabilities.clone(),
    };

    let Some(path) = binary_path else {
        return status;
    };

    match Command::new(&path).arg("--version").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            status.version = first_line(&stdout).or_else(|| first_line(&stderr));
            status.status = if output.status.success() {
                ToolStatus::Available
            } else {
                ToolStatus::Error
            };
        }
        Err(_) => status.status = ToolStatus::Error,
    }

    status
}

fn first_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
}

fn resolve_binary(binary: &str) -> Option<PathBuf> {
    if binary.is_empty() {
        return None;
    }
    let candidate = Path::new(binary);
    if candidate.components().count() > 1 {
        return candidate.is_file().then(|| candidate.to_path_buf());
    }

    let path = env::var_os("PATH")?;
    for directory in env::split_paths(&path) {
        let full = directory.join(binary);
        if full.is_file() {
            return Some(full);
        }
        #[cfg(windows)]
        for extension in ["exe", "cmd", "bat"] {
            let with_extension = directory.join(format!("{binary}.{extension}"));
            if with_extension.is_file() {
                return Some(with_extension);
            }
        }
    }
    None
}

fn parse_chunk(chunk: &[u8], state: &mut ParseState) -> Vec<AgentSignal> {
    state.buffer.extend_from_slice(chunk);

    let mut signals = Vec::new();
    while let Some(index) = state.buffer.iter().position(|byte| *byte == b'\n') {
        let line: Vec<u8> = state.buffer.drain(..=index).collect();
        let text = String::from_utf8_lossy(&line);
        let trimmed = text.trim_end_matches(['\r', '\n']);
        if trimmed.trim().is_empty() {
            continue;
        }
        signals.extend(parse_line(trimmed));
    }
    signals
}

fn parse_line(line: &str) -> Vec<AgentSignal> {
    let trimmed = line.trim();

    if let Ok(Value::Object(object)) = serde_json::from_str::<Value>(trimmed) {
        if let Some(error) = object.get("error").and_then(Value::as_str) {
            return vec![AgentSignal::Error(error.to_owned())];
        }

        if object.get("type").and_then(Value::as_str) == Some("result") {
            if let Some(result) = object.get("result").and_then(Value::as_str) {
                return vec![AgentSignal::MessageComplete {
                    text: result.to_owned(),
                }];
            }
        }

        if let Some(result) = object.get("result").and_then(Value::as_str) {
            return vec![AgentSignal::MessageComplete {
                text: result.to_owned(),
            }];
        }

        if let Some(text) = object
            .get("delta")
            .or_else(|| object.get("text"))
            .or_else(|| object.get("content"))
            .and_then(Value::as_str)
        {
            return vec![AgentSignal::Text(text.to_owned())];
        }

        return vec![AgentSignal::Text(trimmed.to_owned())];
    }

    if trimmed.eq_ignore_ascii_case("ready") {
        return vec![AgentSignal::Ready];
    }

    vec![AgentSignal::Text(trimmed.to_owned())]
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn registry_exposes_the_expected_drivers_and_raw_fallback() {
        let registry = DriverRegistry::new();
        let ids: Vec<String> = registry
            .descriptors()
            .into_iter()
            .map(|descriptor| descriptor.id)
            .collect();

        assert_eq!(ids, vec!["agy", "claude", "cmd", "opencode", "raw"]);
        assert_eq!(registry.resolve("desconocido").descriptor().id, "raw");
    }

    #[test]
    fn builds_interactive_commands_from_the_binary() {
        let registry = DriverRegistry::new();
        let context = SpawnContext::default();

        assert_eq!(registry.resolve("claude").spawn(&context).program, "claude");
        assert_eq!(registry.resolve("agy").spawn(&context).program, "agy");
        assert_eq!(
            registry.resolve("cmd").spawn(&context).program,
            "commandcode"
        );
    }

    #[test]
    fn builds_one_shot_commands_with_continuation_and_limits() {
        let registry = DriverRegistry::new();
        let context = SpawnContext {
            continue_session: true,
            max_turns: Some(4),
            ..SpawnContext::default()
        };

        let claude = registry
            .resolve("claude")
            .one_shot(&context, "hola")
            .unwrap();
        assert_eq!(claude.program, "claude");
        assert_eq!(claude.args, ["-p", "hola", "--output-format", "json"]);

        let opencode = registry
            .resolve("opencode")
            .one_shot(&context, "hola")
            .unwrap();
        assert_eq!(opencode.args, ["run", "hola"]);

        let cmd = registry.resolve("cmd").one_shot(&context, "hola").unwrap();
        assert_eq!(
            cmd.args,
            [
                "-p",
                "hola",
                "--output-format",
                "json",
                "--continue",
                "--max-turns",
                "4"
            ]
        );

        let agy = registry.resolve("agy").one_shot(&context, "hola").unwrap();
        assert_eq!(
            agy.args,
            ["-p", "hola", "--output-format", "json", "--continue"]
        );
    }

    #[test]
    fn parses_ndjson_frames_incrementally() {
        let registry = DriverRegistry::new();
        let driver = registry.resolve("cmd");
        let mut state = ParseState::default();

        let first = driver.parse(b"{\"text\":\"uno\"}\n{\"type\":\"result\",", &mut state);
        assert_eq!(first, vec![AgentSignal::Text("uno".to_owned())]);

        let second = driver.parse(b"\"result\":\"fin\"}\n", &mut state);
        assert_eq!(
            second,
            vec![AgentSignal::MessageComplete {
                text: "fin".to_owned()
            }]
        );
    }

    #[test]
    fn parses_agy_json_and_error_frames() {
        let registry = DriverRegistry::new();
        let driver = registry.resolve("agy");
        let mut state = ParseState::default();

        assert_eq!(
            driver.parse(b"{\"result\":\"listo\"}\n", &mut state),
            vec![AgentSignal::MessageComplete {
                text: "listo".to_owned()
            }]
        );
        assert_eq!(
            driver.parse(b"{\"error\":\"sin cuota\"}\n", &mut state),
            vec![AgentSignal::Error("sin cuota".to_owned())]
        );
    }

    #[test]
    fn falls_back_to_raw_text_and_ready_markers() {
        let registry = DriverRegistry::new();
        let driver = registry.resolve("desconocido");
        let mut state = ParseState::default();

        assert_eq!(
            driver.parse(b"hola\nready\n", &mut state),
            vec![AgentSignal::Text("hola".to_owned()), AgentSignal::Ready]
        );
    }

    #[test]
    fn detects_installed_and_missing_binaries() {
        let detected = detect_binary(
            &descriptor(
                "true",
                "True",
                "true",
                capabilities(true, true, false, false, false),
            ),
            None,
        );
        assert_eq!(detected.status, ToolStatus::Available, "{detected:?}");
        assert!(detected.binary_path.is_some());

        let missing = detect_binary(
            &descriptor(
                "missing",
                "Missing",
                "ancleto-no-existe-123",
                capabilities(true, false, false, false, false),
            ),
            Some("npm install x".to_owned()),
        );
        assert_eq!(missing.status, ToolStatus::Missing);
        assert!(missing.binary_path.is_none());
    }
}
