use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

const PACKAGE: &str = "@ancleto/writer-languages";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriterLanguagesStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub message: Option<String>,
}

fn prefix(config_dir: &Path) -> PathBuf {
    config_dir.join("ancleto/writer-languages")
}

fn package_dir(config_dir: &Path) -> PathBuf {
    prefix(config_dir).join("node_modules").join(PACKAGE)
}

fn status_of(config_dir: &Path) -> WriterLanguagesStatus {
    let package = package_dir(config_dir);
    let manifest = package.join("package.json");
    let version = fs::read_to_string(&manifest).ok().and_then(|text| {
        serde_json::from_str::<serde_json::Value>(&text)
            .ok()?
            .get("version")?
            .as_str()
            .map(str::to_owned)
    });
    WriterLanguagesStatus {
        installed: manifest.is_file(),
        version,
        path: prefix(config_dir).to_string_lossy().into_owned().into(),
        message: None,
    }
}

fn expose(config_dir: &Path) {
    let modules = prefix(config_dir).join("node_modules");
    if !modules.is_dir() {
        return;
    }
    let mut paths = vec![modules];
    if let Some(existing) = env::var_os("NODE_PATH") {
        paths.extend(env::split_paths(&existing));
    }
    if let Ok(joined) = env::join_paths(paths) {
        env::set_var("NODE_PATH", joined);
    }
}

pub fn configure(config_dir: &Path) {
    expose(config_dir);
}

pub fn status(config_dir: &Path) -> WriterLanguagesStatus {
    status_of(config_dir)
}

pub fn install(config_dir: &Path) -> Result<WriterLanguagesStatus, String> {
    let root = prefix(config_dir);
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let root_text = root.to_string_lossy().into_owned();

    let mut command = if cfg!(windows) {
        let mut command = Command::new("cmd");
        command.args(["/C", "npm", "install"]);
        command
    } else {
        let mut command = Command::new("npm");
        command.arg("install");
        command
    };
    let output = command
        .args([
            "--prefix",
            &root_text,
            &format!("{PACKAGE}@latest"),
            "--legacy-peer-deps",
            "--no-audit",
            "--no-fund",
        ])
        .output()
        .map_err(|error| format!("no se pudo ejecutar npm: {error}"))?;

    let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if !output.status.success() {
        return Err(if message.is_empty() {
            "npm no pudo instalar @ancleto/writer-languages".to_owned()
        } else {
            message
        });
    }
    expose(config_dir);
    let mut result = status_of(config_dir);
    if !result.installed {
        return Err("npm terminó pero no encontré @ancleto/writer-languages".to_owned());
    }
    result.message = Some(String::from_utf8_lossy(&output.stdout).trim().to_owned());
    Ok(result)
}
