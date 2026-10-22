use std::fs;
use std::path::Path;
use std::process::Command;

use serde::Serialize;

use crate::domain::{CoreError, Result};
use crate::drivers::CommandSpec;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub fn antigravity_open_command(root: &str) -> CommandSpec {
    if cfg!(target_os = "macos") {
        CommandSpec {
            program: "open".to_owned(),
            args: vec!["-a".to_owned(), "Antigravity".to_owned(), root.to_owned()],
        }
    } else if cfg!(target_os = "windows") {
        CommandSpec {
            program: "Antigravity.exe".to_owned(),
            args: vec![root.to_owned()],
        }
    } else {
        CommandSpec {
            program: "antigravity".to_owned(),
            args: vec![root.to_owned()],
        }
    }
}

pub fn spec_agent_command(root: &str) -> CommandSpec {
    CommandSpec {
        program: resolve_spec_binary(root),
        args: vec![
            "init".to_owned(),
            "--agent".to_owned(),
            "antigravity".to_owned(),
        ],
    }
}

fn resolve_spec_binary(root: &str) -> String {
    let local = Path::new(root)
        .join("node_modules")
        .join(".bin")
        .join(if cfg!(windows) {
            "ancleto.cmd"
        } else {
            "ancleto"
        });
    if fs::metadata(&local).is_ok() {
        local.to_string_lossy().into_owned()
    } else {
        "ancleto".to_owned()
    }
}

pub fn open_in_antigravity(root: &str) -> Result<()> {
    let command = antigravity_open_command(root);
    Command::new(&command.program)
        .args(&command.args)
        .spawn()
        .map_err(|error| CoreError::Process(format!("no se pudo abrir Antigravity: {error}")))?;
    Ok(())
}

pub fn sync_antigravity_assets(root: &str) -> Result<SyncResult> {
    let command = spec_agent_command(root);
    let output = Command::new(&command.program)
        .args(&command.args)
        .current_dir(root)
        .output()
        .map_err(|error| {
            CoreError::Process(format!("no se pudo ejecutar {}: {error}", command.program))
        })?;

    Ok(SyncResult {
        success: output.status.success(),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn builds_the_platform_open_command() {
        let command = antigravity_open_command("/tmp/demo");

        if cfg!(target_os = "macos") {
            assert_eq!(command.program, "open");
            assert_eq!(command.args, ["-a", "Antigravity", "/tmp/demo"]);
        } else if cfg!(target_os = "windows") {
            assert_eq!(command.program, "Antigravity.exe");
            assert_eq!(command.args, ["/tmp/demo"]);
        } else {
            assert_eq!(command.program, "antigravity");
            assert_eq!(command.args, ["/tmp/demo"]);
        }
    }

    #[test]
    fn prefers_the_local_ancleto_binary_for_sync() {
        let directory = tempfile::tempdir().unwrap();
        let bin = directory.path().join("node_modules").join(".bin");
        std::fs::create_dir_all(&bin).unwrap();
        let local = bin.join(if cfg!(windows) {
            "ancleto.cmd"
        } else {
            "ancleto"
        });
        std::fs::write(&local, "").unwrap();

        let command = spec_agent_command(&directory.path().to_string_lossy());

        assert_eq!(command.program, local.to_string_lossy());
        assert_eq!(command.args, ["init", "--agent", "antigravity"]);
    }

    #[test]
    fn falls_back_to_the_path_binary_for_sync() {
        let command = spec_agent_command("/tmp/sin-node-modules");

        assert_eq!(command.program, "ancleto");
        assert_eq!(command.args, ["init", "--agent", "antigravity"]);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod mcp_tests {
    use crate::drivers::ToolStatus;
    use crate::providers::{InitOptions, SpecProvider};
    use serde_json::Value;
    use std::fs;
    use std::path::Path;

    #[test]
    fn ancleto_init_merges_antigravity_mcp_without_destroying_keys() {
        let project = tempfile::tempdir().unwrap();
        let agents = project.path().join(".agents");
        fs::create_dir_all(&agents).unwrap();
        fs::write(
            agents.join("mcp_config.json"),
            r#"{
              "mcpServers": { "existente": { "command": "echo", "args": ["hola"] } },
              "otraClave": true
            }"#,
        )
        .unwrap();

        let provider = SpecProvider::detect(project.path());
        if provider.tool().status != ToolStatus::Available {
            eprintln!("ancleto no disponible; se omite la prueba de merge MCP");
            return;
        }

        let outcome = provider
            .init(
                project.path(),
                &InitOptions {
                    agent: Some("antigravity".to_owned()),
                    tier: Some("minimo".to_owned()),
                    language: Some("es".to_owned()),
                    exclude: Vec::new(),
                    profile: Some("general".to_owned()),
                },
            )
            .unwrap();
        assert!(outcome.success, "init falló: {}", outcome.stderr);

        let config: Value = serde_json::from_str(
            &fs::read_to_string(Path::new(project.path()).join(".agents/mcp_config.json")).unwrap(),
        )
        .unwrap();

        assert_eq!(config["otraClave"], true);
        assert_eq!(config["mcpServers"]["existente"]["command"], "echo");
        assert_eq!(config["mcpServers"]["existente"]["args"][0], "hola");
        let servers = config["mcpServers"].as_object().unwrap();
        assert!(
            servers.len() > 1,
            "no se agregó ningún servidor MCP: {servers:?}"
        );
        assert!(servers.keys().any(|name| name != "existente"));
    }
}
