use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::ProviderUsage;

pub const CACHE_TTL_SECS: u64 = 300;
const CACHE_VERSION: u32 = 1;
const CACHE_FILE: &str = "usage-cache.json";
const TEMP_FILE: &str = "usage-cache.json.tmp";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedSnapshot {
    pub version: u32,
    pub checked_at: u64,
    pub providers: Vec<ProviderUsage>,
}

impl CachedSnapshot {
    pub fn new(checked_at: u64, providers: Vec<ProviderUsage>) -> Self {
        Self {
            version: CACHE_VERSION,
            checked_at,
            providers,
        }
    }
}

pub fn cache_path(config_dir: &Path) -> PathBuf {
    config_dir.join(CACHE_FILE)
}

pub fn is_stale(checked_at: u64, now: u64) -> bool {
    checked_at == 0 || now.saturating_sub(checked_at) > CACHE_TTL_SECS
}

pub fn read(path: &Path) -> Option<CachedSnapshot> {
    let text = fs::read_to_string(path).ok()?;
    let snapshot = serde_json::from_str::<CachedSnapshot>(&text).ok()?;
    (snapshot.version == CACHE_VERSION).then_some(snapshot)
}

pub fn write(path: &Path, snapshot: &CachedSnapshot) -> std::io::Result<()> {
    let Some(directory) = path.parent() else {
        return Ok(());
    };
    fs::create_dir_all(directory)?;
    let temp = path.with_file_name(TEMP_FILE);
    let mut file = open_private(&temp)?;
    let payload = serde_json::to_vec_pretty(snapshot).map_err(std::io::Error::other)?;
    file.write_all(&payload)?;
    file.flush()?;
    drop(file);
    fs::rename(&temp, path)
}

#[cfg(unix)]
fn open_private(path: &Path) -> std::io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(path)
}

#[cfg(not(unix))]
fn open_private(path: &Path) -> std::io::Result<fs::File> {
    fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage::{UsageMetric, UsageWindow};

    fn sample_provider() -> ProviderUsage {
        ProviderUsage {
            provider: "opencode".to_owned(),
            label: "OpenCode".to_owned(),
            status: "ready".to_owned(),
            message: Some("ok".to_owned()),
            source: Some("opencode stats".to_owned()),
            cost: Some(1.5),
            tokens: Some(42),
            sessions: Some(2),
            details: vec![UsageMetric {
                label: "Input".to_owned(),
                value: "42".to_owned(),
            }],
            windows: vec![UsageWindow {
                label: "Weekly".to_owned(),
                used_percent: 12.5,
                resets_at: Some("2026-10-30T00:00:00Z".to_owned()),
            }],
            checked_at: 1_700_000_000,
            duration_ms: Some(120),
        }
    }

    fn temp_dir(label: &str) -> PathBuf {
        let unique = format!(
            "ancleto-usage-{label}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        );
        let dir = std::env::temp_dir().join(unique);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn round_trips_snapshots() {
        let dir = temp_dir("roundtrip");
        let path = cache_path(&dir);
        let snapshot = CachedSnapshot::new(1_700_000_000, vec![sample_provider()]);
        write(&path, &snapshot).expect("write");
        let read_back = read(&path).expect("read");
        assert_eq!(read_back.checked_at, snapshot.checked_at);
        assert_eq!(read_back.providers.len(), 1);
        assert_eq!(read_back.providers[0].provider, "opencode");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ignores_corrupt_or_unknown_versions() {
        let dir = temp_dir("corrupt");
        let path = cache_path(&dir);
        fs::write(&path, "{not json").expect("write corrupt");
        assert!(read(&path).is_none());
        fs::write(&path, r#"{"version":99,"checkedAt":1,"providers":[]}"#).expect("write version");
        assert!(read(&path).is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn computes_staleness_from_ttl() {
        assert!(is_stale(0, 1_000));
        assert!(!is_stale(1_000, 1_000 + CACHE_TTL_SECS));
        assert!(is_stale(1_000, 1_000 + CACHE_TTL_SECS + 1));
    }

    #[test]
    fn cache_never_contains_credentials() {
        let dir = temp_dir("secrets");
        let path = cache_path(&dir);
        let mut provider = sample_provider();
        provider.message = Some("sin datos de sesión".to_owned());
        write(&path, &CachedSnapshot::new(1, vec![provider])).expect("write");
        let text = fs::read_to_string(&path).expect("read");
        for needle in ["apiKey", "Bearer", "authorization", "\"key\"", "\"token\""] {
            assert!(!text.contains(needle), "la caché contiene {needle}");
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn cache_file_is_private_on_unix() {
        use std::os::unix::fs::PermissionsExt;
        let dir = temp_dir("permissions");
        let path = cache_path(&dir);
        write(&path, &CachedSnapshot::new(1, vec![])).expect("write");
        let mode = fs::metadata(&path).expect("metadata").permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
        let _ = fs::remove_dir_all(&dir);
    }
}
