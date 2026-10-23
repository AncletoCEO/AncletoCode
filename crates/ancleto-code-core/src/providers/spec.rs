use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::{CoreError, Result};
use crate::drivers::{resolve_binary, ToolStatus};
use crate::providers::node::NodeRuntime;

const INSTALL_HINT: &str = "npm install -g @ancleto/spec";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolReport {
    pub status: ToolStatus,
    pub version: Option<String>,
    pub binary_path: Option<String>,
    pub install_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveChange {
    pub name: String,
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    pub initialized: bool,
    pub host: Option<String>,
    pub tier: Option<String>,
    pub language: Option<String>,
    pub discovery_exclude: Vec<String>,
    pub discovery_output_dir: Option<String>,
    pub active_changes: Vec<ActiveChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecProjectState {
    pub initialized: bool,
    pub host: Option<String>,
    pub tier: Option<String>,
    pub language: Option<String>,
    pub discovery_exclude: Vec<String>,
    pub discovery_output_dir: Option<String>,
    pub active_changes: Vec<ActiveChange>,
    pub tool: ToolReport,
    pub node: NodeRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandOutcome {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitOptions {
    pub agent: Option<String>,
    pub tier: Option<String>,
    pub language: Option<String>,
    pub exclude: Vec<String>,
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryState {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub state: String,
    pub impact: String,
    #[serde(default)]
    pub recommended_action: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub missing_docs: Vec<String>,
    #[serde(default)]
    pub changed_areas: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecsCheck {
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub scanned: u32,
    #[serde(default)]
    pub non_canonical: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNode {
    pub memory_key: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub scope: String,
    pub content: String,
    #[serde(default)]
    pub status: Option<String>,
}

pub struct SpecProvider {
    node: NodeRuntime,
    binary: Option<PathBuf>,
    script: Option<PathBuf>,
    version: Option<String>,
}

impl SpecProvider {
    pub fn detect(root: &Path) -> Self {
        let node = NodeRuntime::detect();
        let (binary, script) = match resolve_spec_runtime(root) {
            Some((binary, script)) => (Some(binary), script),
            None => (None, None),
        };
        let version = binary
            .as_ref()
            .and_then(|path| version_of(path, script.as_deref()));
        Self {
            node,
            binary,
            script,
            version,
        }
    }

    pub fn node(&self) -> &NodeRuntime {
        &self.node
    }

    pub fn tool(&self) -> ToolReport {
        let status = if self.binary.is_none() {
            ToolStatus::Missing
        } else if !self.node.available || !self.node.compatible {
            ToolStatus::Incompatible
        } else if self.version.is_some() {
            ToolStatus::Available
        } else {
            ToolStatus::Error
        };

        ToolReport {
            status,
            version: self.version.clone(),
            binary_path: self
                .script
                .as_ref()
                .or(self.binary.as_ref())
                .map(|path| path.to_string_lossy().into_owned()),
            install_hint: Some(INSTALL_HINT.to_owned()),
        }
    }

    pub fn project_state(&self, root: &Path) -> Result<SpecProjectState> {
        let settings = read_project_settings(root)?;
        Ok(SpecProjectState {
            initialized: settings.initialized,
            host: settings.host,
            tier: settings.tier,
            language: settings.language,
            discovery_exclude: settings.discovery_exclude,
            discovery_output_dir: settings.discovery_output_dir,
            active_changes: settings.active_changes,
            tool: self.tool(),
            node: self.node.clone(),
        })
    }

    pub fn init(&self, root: &Path, options: &InitOptions) -> Result<CommandOutcome> {
        let args = init_arguments(options);
        self.run(root, &args)
    }

    pub fn install(&self, root: &Path, options: &InitOptions) -> Result<CommandOutcome> {
        let args = install_arguments(root, options);
        self.run(root, &args)
    }

    pub fn update(&self, root: &Path) -> Result<CommandOutcome> {
        self.run(root, &strings(&["update", "--project", root_str(root)]))
    }

    pub fn upgrade(&self, root: &Path) -> Result<CommandOutcome> {
        self.run(root, &strings(&["upgrade"]))
    }

    pub fn check(&self, root: &Path) -> Result<CommandOutcome> {
        self.run(root, &strings(&["check"]))
    }

    pub fn doctor(&self, root: &Path) -> Result<CommandOutcome> {
        self.run(root, &strings(&["doctor"]))
    }

    pub fn specs_check(&self, root: &Path, change: Option<&str>) -> Result<SpecsCheck> {
        let mut args = strings(&["specs", "check", "--json"]);
        if let Some(change) = change {
            args.push("--change".to_owned());
            args.push(change.to_owned());
        }
        let outcome = self.run(root, &args)?;
        parse_specs_check(&outcome.stdout)
    }

    pub fn discovery_check(&self, root: &Path) -> Result<DiscoveryState> {
        let outcome = self.run(root, &strings(&["discovery", "--check"]))?;
        parse_discovery(&outcome.stdout)
    }

    pub fn discovery_generate(&self, root: &Path) -> Result<CommandOutcome> {
        self.run(root, &strings(&["discovery"]))
    }

    pub fn memory_list(&self, root: &Path) -> Result<Vec<MemoryNode>> {
        let outcome = self.run(root, &strings(&["memory", "list", "--json"]))?;
        parse_memory_list(&outcome.stdout)
    }

    pub fn memory_context(&self, root: &Path) -> Result<String> {
        let outcome = self.run(root, &strings(&["memory", "context"]))?;
        Ok(outcome.stdout.trim().to_owned())
    }

    pub fn run(&self, root: &Path, args: &[String]) -> Result<CommandOutcome> {
        let binary = self
            .binary
            .as_ref()
            .ok_or_else(|| CoreError::Process("ancleto no está instalado".to_owned()))?;

        let mut command = Command::new(binary);
        if let Some(script) = &self.script {
            command.arg(script);
        }
        let output = command
            .args(args)
            .current_dir(root)
            .output()
            .map_err(|error| CoreError::Process(format!("no se pudo ejecutar ancleto: {error}")))?;

        Ok(CommandOutcome {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

pub fn init_arguments(options: &InitOptions) -> Vec<String> {
    let mut args = strings(&["init"]);
    push_options(&mut args, options);
    args
}

pub fn install_arguments(root: &Path, options: &InitOptions) -> Vec<String> {
    let mut args = strings(&["install", "--project", root_str(root)]);
    push_options(&mut args, options);
    args
}

fn push_options(args: &mut Vec<String>, options: &InitOptions) {
    if let Some(agent) = &options.agent {
        args.push("--agent".to_owned());
        args.push(agent.clone());
    }
    if let Some(tier) = &options.tier {
        args.push("--tier".to_owned());
        args.push(tier.clone());
    }
    if let Some(language) = &options.language {
        args.push("--lang".to_owned());
        args.push(language.clone());
    }
    if !options.exclude.is_empty() {
        args.push("--exclude".to_owned());
        args.push(options.exclude.join(","));
    }
    if let Some(profile) = &options.profile {
        args.push("--profile".to_owned());
        args.push(profile.clone());
    }
}

pub fn read_project_settings(root: &Path) -> Result<ProjectSettings> {
    let manifest_path = root.join(".ancletorc");
    if !manifest_path.is_file() {
        return Ok(ProjectSettings {
            initialized: false,
            host: None,
            tier: read_tier_file(root),
            language: None,
            discovery_exclude: Vec::new(),
            discovery_output_dir: None,
            active_changes: scan_changes(root),
        });
    }

    let text = fs::read_to_string(&manifest_path)?;
    let value: Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))
        .map_err(|error| CoreError::InvalidInput(format!(".ancletorc inválido: {error}")))?;

    let host = value
        .get("agent")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let tier = value
        .get("tier")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| read_tier_file(root));
    let language = value
        .get("language")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let discovery = value.get("discovery");
    let discovery_exclude = discovery
        .and_then(|discovery| discovery.get("exclude"))
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let discovery_output_dir = discovery
        .and_then(|discovery| discovery.get("outputDir"))
        .and_then(Value::as_str)
        .map(str::to_owned);

    Ok(ProjectSettings {
        initialized: true,
        host,
        tier,
        language,
        discovery_exclude,
        discovery_output_dir,
        active_changes: scan_changes(root),
    })
}

pub fn parse_discovery(text: &str) -> Result<DiscoveryState> {
    serde_json::from_str(text.trim())
        .map_err(|error| CoreError::InvalidInput(format!("discovery inválido: {error}")))
}

pub fn parse_specs_check(text: &str) -> Result<SpecsCheck> {
    serde_json::from_str(text.trim())
        .map_err(|error| CoreError::InvalidInput(format!("specs check inválido: {error}")))
}

pub fn parse_memory_list(text: &str) -> Result<Vec<MemoryNode>> {
    serde_json::from_str(text.trim())
        .map_err(|error| CoreError::InvalidInput(format!("memoria inválida: {error}")))
}

fn scan_changes(root: &Path) -> Vec<ActiveChange> {
    let changes_dir = root.join("aspec").join("changes");
    let Ok(entries) = fs::read_dir(changes_dir) else {
        return Vec::new();
    };

    let mut changes = Vec::new();
    for entry in entries.flatten() {
        if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let mut artifacts = Vec::new();
        for artifact in ["proposal.md", "design.md", "tasks.md"] {
            if entry.path().join(artifact).is_file() {
                artifacts.push(artifact.to_owned());
            }
        }
        if entry.path().join("specs").is_dir() {
            artifacts.push("specs".to_owned());
        }
        changes.push(ActiveChange { name, artifacts });
    }

    changes.sort_by(|left, right| left.name.cmp(&right.name));
    changes
}

fn read_tier_file(root: &Path) -> Option<String> {
    fs::read_to_string(root.join(".opencode").join(".ancleto-tier"))
        .ok()
        .map(|tier| tier.trim().to_owned())
        .filter(|tier| !tier.is_empty())
}

fn resolve_spec_runtime(root: &Path) -> Option<(PathBuf, Option<PathBuf>)> {
    let node = env::var_os("ANCLETO_SPEC_NODE").map(PathBuf::from);
    let script = env::var_os("ANCLETO_SPEC_JS").map(PathBuf::from);
    if let (Some(node), Some(script)) = (node, script) {
        if node.is_file() && script.is_file() {
            return Some((node, Some(script)));
        }
    }
    resolve_spec_binary(root).map(|binary| (binary, None))
}

fn resolve_spec_binary(root: &Path) -> Option<PathBuf> {
    let local = root
        .join("node_modules")
        .join(".bin")
        .join(if cfg!(windows) {
            "ancleto.cmd"
        } else {
            "ancleto"
        });
    if local.is_file() {
        return Some(local);
    }
    if let Some(global) = resolve_binary("ancleto") {
        return Some(global);
    }
    if let Some(sidecar) = env::var_os("ANCLETO_SPEC_BIN").map(PathBuf::from) {
        if sidecar.is_file() {
            return Some(sidecar);
        }
    }
    let executable = env::current_exe().ok()?;
    let sidecar = executable.parent()?.join(if cfg!(windows) {
        "ancleto.exe"
    } else {
        "ancleto"
    });
    sidecar.is_file().then_some(sidecar)
}

fn version_of(path: &Path, script: Option<&Path>) -> Option<String> {
    let mut command = Command::new(path);
    if let Some(script) = script {
        command.arg(script);
    }
    let output = command.arg("--version").output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
}

fn root_str(root: &Path) -> &str {
    root.to_str().unwrap_or(".")
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn default_schema_version() -> u32 {
    1
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture_project() -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join(".ancletorc"),
            r#"{
              "schemaVersion": 1,
              "agent": "antigravity",
              "language": "es",
              "discovery": { "outputDir": "docs/technical-discovery", "exclude": ["**/*.png", "docs"] }
            }"#,
        )
        .unwrap();
        fs::create_dir_all(directory.path().join(".opencode")).unwrap();
        fs::write(
            directory.path().join(".opencode").join(".ancleto-tier"),
            "minimo\n",
        )
        .unwrap();
        let change = directory
            .path()
            .join("aspec")
            .join("changes")
            .join("add-auth");
        fs::create_dir_all(change.join("specs")).unwrap();
        fs::write(change.join("proposal.md"), "# Proposal\n").unwrap();
        fs::write(change.join("tasks.md"), "- [ ] 1.1\n").unwrap();
        directory
    }

    #[test]
    fn reads_settings_and_active_changes_from_fixtures() {
        let project = fixture_project();

        let settings = read_project_settings(project.path()).unwrap();

        assert!(settings.initialized);
        assert_eq!(settings.host.as_deref(), Some("antigravity"));
        assert_eq!(settings.tier.as_deref(), Some("minimo"));
        assert_eq!(settings.language.as_deref(), Some("es"));
        assert_eq!(settings.discovery_exclude, ["**/*.png", "docs"]);
        assert_eq!(settings.active_changes.len(), 1);
        assert_eq!(settings.active_changes[0].name, "add-auth");
        assert_eq!(
            settings.active_changes[0].artifacts,
            ["proposal.md", "tasks.md", "specs"]
        );
    }

    #[test]
    fn reports_an_uninitialized_project() {
        let directory = tempfile::tempdir().unwrap();

        let settings = read_project_settings(directory.path()).unwrap();

        assert!(!settings.initialized);
        assert!(settings.host.is_none());
        assert!(settings.active_changes.is_empty());
    }

    #[test]
    fn builds_init_and_install_arguments() {
        let options = InitOptions {
            agent: Some("antigravity".to_owned()),
            tier: Some("gratis".to_owned()),
            language: Some("pt".to_owned()),
            exclude: vec!["**/*.png".to_owned(), "docs".to_owned()],
            profile: Some("test".to_owned()),
        };

        assert_eq!(
            init_arguments(&options),
            [
                "init",
                "--agent",
                "antigravity",
                "--tier",
                "gratis",
                "--lang",
                "pt",
                "--exclude",
                "**/*.png,docs",
                "--profile",
                "test"
            ]
        );

        let install = install_arguments(Path::new("/tmp/demo"), &options);
        assert_eq!(
            &install[..4],
            ["install", "--project", "/tmp/demo", "--agent"]
        );
    }

    #[test]
    fn parses_discovery_fixtures() {
        let discovery = parse_discovery(
            r#"{
              "schemaVersion": 2,
              "state": "STALE",
              "impact": "material",
              "recommendedAction": "regenerate",
              "message": "Seed desactualizado",
              "missingDocs": ["units/_map.md"],
              "changedAreas": ["src/lib.rs"]
            }"#,
        )
        .unwrap();

        assert_eq!(discovery.schema_version, 2);
        assert_eq!(discovery.state, "STALE");
        assert_eq!(discovery.impact, "material");
        assert_eq!(discovery.recommended_action.as_deref(), Some("regenerate"));
        assert_eq!(discovery.missing_docs, ["units/_map.md"]);
    }

    #[test]
    fn parses_specs_check_and_memory_fixtures() {
        let check = parse_specs_check(r#"{"ok":true,"scanned":12,"nonCanonical":[]}"#).unwrap();
        assert!(check.ok);
        assert_eq!(check.scanned, 12);
        assert!(check.non_canonical.is_empty());

        let nodes = parse_memory_list(
            r#"[{
              "memory_key": "api-errors",
              "type": "decision",
              "scope": "project",
              "content": "Formato común",
              "status": "active"
            }]"#,
        )
        .unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].memory_key, "api-errors");
        assert_eq!(nodes[0].kind, "decision");
    }

    #[test]
    fn detects_the_tool_status_without_spawning_it() {
        let directory = tempfile::tempdir().unwrap();
        let missing = SpecProvider {
            node: NodeRuntime {
                available: true,
                version: Some("v24.0.0".to_owned()),
                binary_path: Some("/usr/bin/node".to_owned()),
                compatible: true,
            },
            binary: None,
            script: None,
            version: None,
        };
        assert_eq!(missing.tool().status, ToolStatus::Missing);

        let incompatible = SpecProvider {
            node: NodeRuntime {
                available: true,
                version: Some("v22.0.0".to_owned()),
                binary_path: Some("/usr/bin/node".to_owned()),
                compatible: false,
            },
            binary: Some(directory.path().join("ancleto")),
            script: None,
            version: Some("ancleto 0.14.0".to_owned()),
        };
        assert_eq!(incompatible.tool().status, ToolStatus::Incompatible);
    }
}
