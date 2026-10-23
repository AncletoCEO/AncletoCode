use std::fs::{self, File};
use std::path::{Path, PathBuf};

use base64::Engine;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use tar::Archive;

const REGISTRY_LATEST: &str = "https://registry.npmjs.org/@ancleto/spec/latest";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AncletoRuntimeStatus {
    pub active_version: Option<String>,
    pub active_path: Option<String>,
    pub bundled_version: Option<String>,
    pub node_path: Option<String>,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub error: Option<String>,
}

#[derive(Deserialize)]
struct RegistryPackage {
    version: String,
    dist: RegistryDist,
}

#[derive(Deserialize)]
struct RegistryDist {
    tarball: String,
    integrity: String,
}

fn registry_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("AncletoCode")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| error.to_string())
}

fn latest_package() -> Result<RegistryPackage, String> {
    registry_client()?
        .get(REGISTRY_LATEST)
        .send()
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.json::<RegistryPackage>())
        .map_err(|error| format!("no se pudo consultar @ancleto/spec: {error}"))
}

fn package_version(package_dir: &Path) -> Option<String> {
    let text = fs::read_to_string(package_dir.join("package.json")).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    value.get("version")?.as_str().map(str::to_owned)
}

fn bundled_script(resource_dir: &Path) -> Option<PathBuf> {
    bundled_script_at(resource_dir, "package/src/cli/index.js")
}

fn bundled_writer_script(resource_dir: &Path) -> Option<PathBuf> {
    bundled_script_at(resource_dir, "writer/package/src/cli/index.js")
}

fn bundled_script_at(resource_dir: &Path, relative: &str) -> Option<PathBuf> {
    let mut candidates = vec![
        resource_dir.join("resources/ancleto").join(relative),
        resource_dir.join("ancleto").join(relative),
    ];
    if let Ok(workspace) = std::env::current_dir() {
        candidates.push(workspace.join("src-tauri/resources/ancleto").join(relative));
    }
    candidates.into_iter().find(|path| path.is_file())
}

fn node_binary() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("ANCLETO_SPEC_NODE").map(PathBuf::from) {
        if path.is_file() {
            return Some(path);
        }
    }
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(PathBuf::from));
    if let Some(dir) = &exe_dir {
        for name in ["ancleto-node.exe", "ancleto-node"] {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        if let Ok(entries) = fs::read_dir(dir.join("binaries")) {
            if let Some(path) = entries.flatten().map(|entry| entry.path()).find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("ancleto-node"))
            }) {
                return Some(path);
            }
        }
    }
    let workspace = std::env::current_dir().ok()?;
    fs::read_dir(workspace.join("src-tauri/binaries"))
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("ancleto-node"))
        })
}

fn active_from_marker(config_dir: &Path) -> Option<PathBuf> {
    let version = fs::read_to_string(config_dir.join("ancleto/active")).ok()?;
    let script = config_dir
        .join("ancleto/versions")
        .join(version.trim())
        .join("package/src/cli/index.js");
    script.is_file().then_some(script)
}

fn active_script(config_dir: &Path, resource_dir: &Path) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("ANCLETO_SPEC_JS").map(PathBuf::from) {
        if path.is_file() {
            return Some(path);
        }
    }
    active_from_marker(config_dir).or_else(|| bundled_script(resource_dir))
}

fn activate(config_dir: &Path, script: &Path) -> Result<(), String> {
    let package = script
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or_else(|| "ruta de @ancleto/spec inválida".to_owned())?;
    let version = package_version(package)
        .ok_or_else(|| "no pude leer la versión de @ancleto/spec".to_owned())?;
    let node = node_binary().ok_or_else(|| "no encontré el runtime Node incluido".to_owned())?;
    fs::create_dir_all(config_dir.join("ancleto")).map_err(|error| error.to_string())?;
    fs::write(config_dir.join("ancleto/active"), format!("{version}\n"))
        .map_err(|error| error.to_string())?;
    std::env::set_var("ANCLETO_SPEC_NODE", node);
    std::env::set_var("ANCLETO_SPEC_JS", script);
    Ok(())
}

pub fn configure(config_dir: &Path, resource_dir: &Path) {
    if let Some(script) = active_script(config_dir, resource_dir) {
        if activate(config_dir, &script).is_ok() {
            if let Some(node) = node_binary() {
                let _ = expose_node_on_path(config_dir, &node);
                if let Some(writer) = bundled_writer_script(resource_dir) {
                    std::env::set_var("ANCLETO_WRITER_NODE", &node);
                    std::env::set_var("ANCLETO_WRITER_JS", writer);
                }
            }
        }
    }
}

fn expose_node_on_path(config_dir: &Path, node: &Path) -> Result<(), String> {
    let bin = config_dir.join("ancleto/bin");
    fs::create_dir_all(&bin).map_err(|error| error.to_string())?;
    let target = bin.join(if cfg!(windows) { "node.exe" } else { "node" });
    if !target.is_file() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(node, &target).map_err(|error| error.to_string())?;
        #[cfg(windows)]
        fs::copy(node, &target).map_err(|error| error.to_string())?;
    }
    let current = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![bin];
    paths.extend(std::env::split_paths(&current));
    let joined = std::env::join_paths(paths).map_err(|error| error.to_string())?;
    std::env::set_var("PATH", joined);
    Ok(())
}

pub fn status(config_dir: &Path, resource_dir: &Path, check_remote: bool) -> AncletoRuntimeStatus {
    let active = active_script(config_dir, resource_dir);
    let bundled = bundled_script(resource_dir);
    let active_version = active
        .as_ref()
        .and_then(|script| script.parent()?.parent()?.parent())
        .and_then(package_version);
    let bundled_version = bundled
        .as_ref()
        .and_then(|script| script.parent()?.parent()?.parent())
        .and_then(package_version);
    let mut latest_version = None;
    let mut error = None;
    if check_remote {
        match latest_package() {
            Ok(package) => latest_version = Some(package.version),
            Err(message) => error = Some(message),
        }
    }
    let update_available = match (&latest_version, &active_version) {
        (Some(latest), Some(active)) => latest != active,
        (Some(_), None) => true,
        _ => false,
    };
    AncletoRuntimeStatus {
        active_version,
        active_path: active.map(|path| path.to_string_lossy().into_owned()),
        bundled_version,
        node_path: node_binary().map(|path| path.to_string_lossy().into_owned()),
        latest_version,
        update_available,
        error,
    }
}

pub fn update(config_dir: &Path, resource_dir: &Path) -> Result<AncletoRuntimeStatus, String> {
    let package = latest_package()?;
    let client = registry_client()?;
    let bytes = client
        .get(&package.dist.tarball)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("no se pudo descargar @ancleto/spec: {error}"))?
        .bytes()
        .map_err(|error| error.to_string())?;

    let (algorithm, expected) = package
        .dist
        .integrity
        .split_once('-')
        .ok_or_else(|| "integridad de npm inválida".to_owned())?;
    if algorithm != "sha512" {
        return Err(format!("algoritmo de integridad no soportado: {algorithm}"));
    }
    let actual = base64::engine::general_purpose::STANDARD.encode(Sha512::digest(&bytes));
    if actual != expected {
        return Err("la integridad del paquete @ancleto/spec no coincide".to_owned());
    }

    let versions = config_dir.join("ancleto/versions");
    let target = versions.join(&package.version);
    let staging = versions.join(format!(".{}-partial", package.version));
    fs::create_dir_all(&versions).map_err(|error| error.to_string())?;
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
    let archive_path = staging.join("package.tgz");
    fs::write(&archive_path, &bytes).map_err(|error| error.to_string())?;
    let file = File::open(&archive_path).map_err(|error| error.to_string())?;
    let mut archive = Archive::new(GzDecoder::new(file));
    archive
        .unpack(&staging)
        .map_err(|error| error.to_string())?;
    let script = staging.join("package/src/cli/index.js");
    if !script.is_file() {
        return Err("el paquete descargado no contiene el CLI de Ancleto".to_owned());
    }
    let _ = fs::remove_dir_all(&target);
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;
    fs::rename(staging.join("package"), target.join("package"))
        .map_err(|error| format!("no se pudo activar @ancleto/spec: {error}"))?;
    let _ = fs::remove_dir_all(&staging);
    activate(config_dir, &target.join("package/src/cli/index.js"))?;
    Ok(status(config_dir, resource_dir, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_the_bundled_package_version() {
        let resource_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        if !resource_dir
            .join("resources/ancleto/package/src/cli/index.js")
            .is_file()
        {
            return;
        }
        let config_dir =
            std::env::temp_dir().join(format!("ancleto-runtime-test-{}", std::process::id()));
        let status = status(&config_dir, &resource_dir, false);
        assert_eq!(status.bundled_version.as_deref(), Some("0.14.0"));
        assert!(status.active_version.is_some());
        let _ = std::fs::remove_dir_all(config_dir);
    }
}
