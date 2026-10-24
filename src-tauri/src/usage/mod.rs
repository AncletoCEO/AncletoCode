mod cache;
mod providers;
mod runner;
mod service;

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub use cache::{cache_path, is_stale, CachedSnapshot, CACHE_TTL_SECS};
pub use providers::default_providers;
pub use runner::collect_parallel;
pub use service::{Emitter, UsageService};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetric {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    pub label: String,
    pub used_percent: f64,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderUsage {
    pub provider: String,
    pub label: String,
    pub status: String,
    pub message: Option<String>,
    pub source: Option<String>,
    pub cost: Option<f64>,
    pub tokens: Option<u64>,
    pub sessions: Option<u64>,
    pub details: Vec<UsageMetric>,
    pub windows: Vec<UsageWindow>,
    pub checked_at: u64,
    #[serde(default)]
    pub duration_ms: Option<u64>,
}

impl ProviderUsage {
    pub fn failed(provider: &str, label: &str, message: impl Into<String>) -> Self {
        Self {
            provider: provider.to_owned(),
            label: label.to_owned(),
            status: "error".to_owned(),
            message: Some(message.into()),
            source: None,
            cost: None,
            tokens: None,
            sessions: None,
            details: Vec::new(),
            windows: Vec::new(),
            checked_at: now_secs(),
            duration_ms: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshot {
    pub providers: Vec<ProviderUsage>,
    pub checked_at: u64,
    pub stale: bool,
    pub refreshing: bool,
    pub duration_ms: Option<u64>,
}

pub(crate) fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum UsageEvent {
    #[serde(rename = "provider")]
    Provider { index: usize, usage: ProviderUsage },
    #[serde(rename = "done")]
    Done { snapshot: UsageSnapshot },
}
