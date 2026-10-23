use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::{CoreError, Result};
use crate::drivers::{resolve_binary, ToolStatus};
use crate::providers::node::NodeRuntime;
use crate::providers::spec::{CommandOutcome, ToolReport};

const DEFAULT_OUTPUT_DIR: &str = ".ancleto/writer";
const DEFAULT_LANGUAGE_PACK: &str = "@ancleto/writer-languages";
const MANIFEST_SCHEMA_VERSION: u32 = 1;
const INSTALL_HINT: &str =
    "npm install @ancleto/writer @ancleto/writer-languages --legacy-peer-deps";

const FACT_FILES: [&str; 13] = [
    "cycles",
    "layers",
    "communities",
    "god-nodes",
    "bridges",
    "dead-code",
    "doc-coverage",
    "hotspots",
    "blast-radius",
    "unsupported",
    "unresolved",
    "externals",
    "skipped",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriterConfig {
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default = "default_language_pack_enabled")]
    pub language_pack_enabled: bool,
    #[serde(default = "default_language_pack_module")]
    pub language_pack_module: String,
}

impl Default for WriterConfig {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            language_pack_enabled: default_language_pack_enabled(),
            language_pack_module: default_language_pack_module(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriterCheck {
    pub status: String,
    #[serde(default)]
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriterManifest {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub writer_version: Option<String>,
    #[serde(default)]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub git_commit: Option<String>,
    #[serde(default)]
    pub stats: Value,
    #[serde(default)]
    pub outputs: Vec<Value>,
}

impl WriterManifest {
    pub fn compatible(&self) -> bool {
        self.schema_version == MANIFEST_SCHEMA_VERSION
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphSummary {
    pub nodes: usize,
    pub edges: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguagePack {
    pub installed: bool,
    pub enabled: bool,
    pub module: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriterState {
    pub tool: ToolReport,
    pub check: Option<WriterCheck>,
    pub manifest: Option<WriterManifest>,
    pub compatible: bool,
    pub graph: Option<GraphSummary>,
    pub facts: BTreeMap<String, Value>,
    pub language_pack: LanguagePack,
    pub output_dir: String,
    pub human_dir: String,
}

pub struct WriterProvider {
    node: NodeRuntime,
    binary: Option<PathBuf>,
    script: Option<PathBuf>,
    version: Option<String>,
}

impl WriterProvider {
    pub fn detect(root: &Path) -> Self {
        let node = NodeRuntime::detect();
        let (binary, script) = match resolve_writer_runtime(root) {
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

    pub fn state(&self, root: &Path) -> Result<WriterState> {
        let config = read_config(root)?;
        let output_dir = config.output_dir.clone();
        let manifest = read_manifest(root, &output_dir)?;
        let compatible = manifest
            .as_ref()
            .map(WriterManifest::compatible)
            .unwrap_or(false);
        let (graph, facts) = if compatible {
            (
                read_graph_summary(root, &output_dir),
                read_facts(root, &output_dir),
            )
        } else {
            (None, BTreeMap::new())
        };

        let check = if self.binary.is_some() {
            self.check(root).ok()
        } else {
            None
        };

        Ok(WriterState {
            tool: self.tool(),
            check,
            manifest,
            compatible,
            graph,
            facts,
            language_pack: language_pack_status(root, &config),
            human_dir: format!("{output_dir}/human"),
            output_dir,
        })
    }

    pub fn init(&self, root: &Path) -> Result<CommandOutcome> {
        self.run(root, &strings(&["init"]))
    }

    pub fn build(&self, root: &Path) -> Result<CommandOutcome> {
        self.run(root, &strings(&["build", "--json"]))
    }

    pub fn check(&self, root: &Path) -> Result<WriterCheck> {
        let outcome = self.run(root, &strings(&["check", "--json"]))?;
        serde_json::from_str(outcome.stdout.trim())
            .map_err(|error| CoreError::InvalidInput(format!("writer check inválido: {error}")))
    }

    pub fn lint(&self, root: &Path) -> Result<CommandOutcome> {
        self.run(root, &strings(&["lint", "--json"]))
    }

    pub fn impact(&self, root: &Path, files: &[String]) -> Result<CommandOutcome> {
        let mut args = strings(&["impact"]);
        args.extend(files.iter().cloned());
        args.push("--json".to_owned());
        self.run(root, &args)
    }

    pub fn human(&self, root: &Path) -> Result<CommandOutcome> {
        self.run(root, &strings(&["human", "--json", "--interactive"]))
    }

    pub fn doctor(&self, root: &Path) -> Result<CommandOutcome> {
        self.run(root, &strings(&["doctor", "--json"]))
    }

    pub fn set_language_pack(
        &self,
        root: &Path,
        enabled: bool,
        module: Option<&str>,
    ) -> Result<WriterConfig> {
        let config_path = root.join(".ancleto-writer.json");
        let mut value = if config_path.is_file() {
            let text = fs::read_to_string(&config_path)?;
            serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}'))
                .map_err(|error| CoreError::InvalidInput(format!("config inválida: {error}")))?
        } else {
            serde_json::json!({ "schemaVersion": 1 })
        };

        if !value.is_object() {
            return Err(CoreError::InvalidInput(
                ".ancleto-writer.json debe ser un objeto".to_owned(),
            ));
        }

        let module = module.unwrap_or(DEFAULT_LANGUAGE_PACK).to_owned();
        value["languagePack"] = serde_json::json!({
            "enabled": enabled,
            "module": module,
        });

        let text = serde_json::to_string_pretty(&value)
            .map_err(|error| CoreError::InvalidInput(error.to_string()))?;
        fs::write(&config_path, format!("{text}\n"))?;
        read_config(root)
    }

    pub fn run(&self, root: &Path, args: &[String]) -> Result<CommandOutcome> {
        let binary = self
            .binary
            .as_ref()
            .ok_or_else(|| CoreError::Process("ancleto-writer no está instalado".to_owned()))?;

        let mut command = Command::new(binary);
        if let Some(script) = &self.script {
            command.arg(script);
        }
        let output = command
            .args(args)
            .current_dir(root)
            .output()
            .map_err(|error| {
                CoreError::Process(format!("no se pudo ejecutar ancleto-writer: {error}"))
            })?;

        Ok(CommandOutcome {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

pub fn read_config(root: &Path) -> Result<WriterConfig> {
    let path = root.join(".ancleto-writer.json");
    if !path.is_file() {
        return Ok(WriterConfig::default());
    }

    let text = fs::read_to_string(path)?;
    let value: Value =
        serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|error| {
            CoreError::InvalidInput(format!(".ancleto-writer.json inválido: {error}"))
        })?;

    let language_pack = value.get("languagePack");
    Ok(WriterConfig {
        output_dir: value
            .get("outputDir")
            .and_then(Value::as_str)
            .unwrap_or(DEFAULT_OUTPUT_DIR)
            .to_owned(),
        language_pack_enabled: language_pack
            .and_then(|pack| pack.get("enabled"))
            .and_then(Value::as_bool)
            .unwrap_or(true),
        language_pack_module: language_pack
            .and_then(|pack| pack.get("module"))
            .and_then(Value::as_str)
            .unwrap_or(DEFAULT_LANGUAGE_PACK)
            .to_owned(),
    })
}

pub fn read_manifest(root: &Path, output_dir: &str) -> Result<Option<WriterManifest>> {
    let path = root.join(output_dir).join("manifest.json");
    if !path.is_file() {
        return Ok(None);
    }

    let text = fs::read_to_string(path)?;
    let manifest = serde_json::from_str(text.trim_start_matches('\u{feff}'))
        .map_err(|error| CoreError::InvalidInput(format!("manifest.json inválido: {error}")))?;
    Ok(Some(manifest))
}

pub fn read_graph_summary(root: &Path, output_dir: &str) -> Option<GraphSummary> {
    let path = root.join(output_dir).join("graph.json");
    let text = fs::read_to_string(path).ok()?;
    let value: Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()?;
    Some(GraphSummary {
        nodes: value
            .get("nodes")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        edges: value
            .get("edges")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
    })
}

pub fn read_facts(root: &Path, output_dir: &str) -> BTreeMap<String, Value> {
    let facts_dir = root.join(output_dir).join("facts");
    let mut facts = BTreeMap::new();

    for name in FACT_FILES {
        let path = facts_dir.join(format!("{name}.json"));
        if let Ok(text) = fs::read_to_string(path) {
            if let Ok(value) = serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')) {
                facts.insert(name.to_owned(), value);
            }
        }
    }

    facts
}

pub fn language_pack_status(root: &Path, config: &WriterConfig) -> LanguagePack {
    let installed = root
        .join("node_modules")
        .join(&config.language_pack_module)
        .join("package.json")
        .is_file();

    LanguagePack {
        installed,
        enabled: config.language_pack_enabled,
        module: config.language_pack_module.clone(),
    }
}

fn resolve_writer_runtime(root: &Path) -> Option<(PathBuf, Option<PathBuf>)> {
    let node = env::var_os("ANCLETO_WRITER_NODE").map(PathBuf::from);
    let script = env::var_os("ANCLETO_WRITER_JS").map(PathBuf::from);
    if let (Some(node), Some(script)) = (node, script) {
        if node.is_file() && script.is_file() {
            return Some((node, Some(script)));
        }
    }
    resolve_writer_binary(root).map(|binary| (binary, None))
}

fn resolve_writer_binary(root: &Path) -> Option<PathBuf> {
    let local = root
        .join("node_modules")
        .join(".bin")
        .join(if cfg!(windows) {
            "ancleto-writer.cmd"
        } else {
            "ancleto-writer"
        });
    if local.is_file() {
        return Some(local);
    }
    if let Some(global) = resolve_binary("ancleto-writer") {
        return Some(global);
    }
    if let Some(sidecar) = env::var_os("ANCLETO_WRITER_BIN").map(PathBuf::from) {
        if sidecar.is_file() {
            return Some(sidecar);
        }
    }
    let executable = env::current_exe().ok()?;
    let sidecar = executable.parent()?.join(if cfg!(windows) {
        "ancleto-writer.exe"
    } else {
        "ancleto-writer"
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
        .find(|line| line.trim_start().starts_with("ancleto-writer"))
        .or_else(|| text.lines().find(|line| !line.trim().is_empty()))
        .map(|line| line.trim().to_owned())
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn default_output_dir() -> String {
    DEFAULT_OUTPUT_DIR.to_owned()
}

fn default_language_pack_enabled() -> bool {
    true
}

fn default_language_pack_module() -> String {
    DEFAULT_LANGUAGE_PACK.to_owned()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn fixture() -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join(".ancleto-writer.json"),
            r#"{
              "schemaVersion": 1,
              "outputDir": ".ancleto/writer",
              "languagePack": { "enabled": false, "module": "@ancleto/writer-languages" }
            }"#,
        )
        .unwrap();
        let writer = directory.path().join(".ancleto").join("writer");
        fs::create_dir_all(writer.join("facts")).unwrap();
        fs::write(
            writer.join("manifest.json"),
            r#"{
              "schemaVersion": 1,
              "writerVersion": "0.4.0",
              "gitCommit": "abc123",
              "stats": { "files": 3, "edges": 2, "nodes": 4 },
              "outputs": []
            }"#,
        )
        .unwrap();
        fs::write(
            writer.join("facts").join("god-nodes.json"),
            r#"[{"path":"src/lib.ts","score":0.9}]"#,
        )
        .unwrap();
        fs::write(writer.join("facts").join("cycles.json"), r#"{"cycles":[]}"#).unwrap();
        fs::write(
            writer.join("graph.json"),
            r#"{"nodes":[{},{}],"edges":[{}]}"#,
        )
        .unwrap();
        directory
    }

    #[test]
    fn reads_config_manifest_and_facts_from_fixtures() {
        let project = fixture();

        let config = read_config(project.path()).unwrap();
        assert_eq!(config.output_dir, ".ancleto/writer");
        assert!(!config.language_pack_enabled);

        let manifest = read_manifest(project.path(), &config.output_dir)
            .unwrap()
            .unwrap();
        assert!(manifest.compatible());
        assert_eq!(manifest.writer_version.as_deref(), Some("0.4.0"));
        assert_eq!(manifest.stats["files"], 3);

        let facts = read_facts(project.path(), &config.output_dir);
        assert_eq!(facts.len(), 2);
        assert!(facts.contains_key("god-nodes"));

        let graph = read_graph_summary(project.path(), &config.output_dir).unwrap();
        assert_eq!(graph.nodes, 2);
        assert_eq!(graph.edges, 1);
    }

    #[test]
    fn flags_incompatible_schemas_and_missing_facts() {
        let directory = tempfile::tempdir().unwrap();
        let writer = directory.path().join(".ancleto").join("writer");
        fs::create_dir_all(&writer).unwrap();
        fs::write(
            writer.join("manifest.json"),
            r#"{"schemaVersion":99,"stats":{}}"#,
        )
        .unwrap();

        let manifest = read_manifest(directory.path(), ".ancleto/writer")
            .unwrap()
            .unwrap();

        assert!(!manifest.compatible());
        assert!(read_facts(directory.path(), ".ancleto/writer").is_empty());
    }

    #[test]
    fn reports_the_language_pack_presence() {
        let project = fixture();
        let config = read_config(project.path()).unwrap();
        assert!(!language_pack_status(project.path(), &config).installed);

        let pack = project
            .path()
            .join("node_modules")
            .join("@ancleto")
            .join("writer-languages");
        fs::create_dir_all(&pack).unwrap();
        fs::write(pack.join("package.json"), "{}").unwrap();

        let status = language_pack_status(project.path(), &config);
        assert!(status.installed);
        assert!(!status.enabled);
    }

    #[test]
    fn toggles_the_language_pack_without_dropping_other_keys() {
        let project = fixture();
        let provider = WriterProvider {
            node: NodeRuntime::detect(),
            binary: None,
            script: None,
            version: None,
        };

        let config = provider
            .set_language_pack(project.path(), true, Some("@ancleto/writer-languages"))
            .unwrap();
        assert!(config.language_pack_enabled);
        assert_eq!(config.output_dir, ".ancleto/writer");

        let text = fs::read_to_string(project.path().join(".ancleto-writer.json")).unwrap();
        let value: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["schemaVersion"], 1);
        assert_eq!(value["languagePack"]["enabled"], true);
    }

    #[test]
    fn parses_check_fixtures() {
        let check: WriterCheck =
            serde_json::from_str(r#"{"status":"stale","exitCode":3}"#).unwrap();
        assert_eq!(check.status, "stale");
        assert_eq!(check.exit_code, 3);
    }
}
