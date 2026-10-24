use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ancleto_code_core::drivers::resolve_binary;

const OPENCODE_GO_USAGE_URL: &str = "https://opencode.ai/zen/go/v1/usage";

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetric {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    pub label: String,
    pub used_percent: f64,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
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
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn resolved(binary: &str) -> Option<PathBuf> {
    resolve_binary(binary)
}

fn compact_number(value: &str) -> Option<f64> {
    let value = value.trim().trim_end_matches('|').trim();
    let (number, multiplier) = match value.chars().last()? {
        'K' | 'k' => (&value[..value.len() - 1], 1_000.0),
        'M' | 'm' => (&value[..value.len() - 1], 1_000_000.0),
        'B' | 'b' => (&value[..value.len() - 1], 1_000_000_000.0),
        _ => (value, 1.0),
    };
    number
        .replace(',', "")
        .parse::<f64>()
        .ok()
        .map(|parsed| parsed * multiplier)
}

fn field<'a>(text: &'a str, label: &str) -> Option<&'a str> {
    text.lines().find_map(|line| {
        let line = line.trim().trim_matches('│').trim();
        let rest = line.strip_prefix(label)?;
        (!rest.trim().is_empty()).then_some(rest.trim())
    })
}

fn opencode_auth_key() -> Option<String> {
    let mut candidates = Vec::new();
    if let Some(data) = std::env::var_os("XDG_DATA_HOME") {
        candidates.push(PathBuf::from(data).join("opencode/auth.json"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(PathBuf::from(home).join(".local/share/opencode/auth.json"));
    }
    if cfg!(windows) {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            candidates.push(PathBuf::from(appdata).join("opencode/auth.json"));
        }
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            candidates.push(PathBuf::from(local).join("opencode/auth.json"));
        }
    }
    for path in candidates {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        if let Some(key) = value
            .get("opencode-go")
            .and_then(|entry| entry.get("key"))
            .and_then(|key| key.as_str())
            .filter(|key| !key.trim().is_empty())
        {
            return Some(key.to_owned());
        }
    }
    None
}

fn opencode_quota() -> (Vec<UsageWindow>, Option<String>) {
    let Some(key) = opencode_auth_key() else {
        return (Vec::new(), None);
    };
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(client) => client,
        Err(error) => return (Vec::new(), Some(error.to_string())),
    };
    let response = match client
        .get(OPENCODE_GO_USAGE_URL)
        .header("Authorization", format!("Bearer {key}"))
        .header("Accept", "application/json")
        .send()
    {
        Ok(response) => response,
        Err(_) => {
            return (
                Vec::new(),
                Some("No se pudo consultar la cuota de OpenCode Go.".to_owned()),
            )
        }
    };
    if !response.status().is_success() {
        return (
            Vec::new(),
            Some(format!(
                "OpenCode Go usage respondió {}.",
                response.status()
            )),
        );
    }
    let Ok(body) = response.json::<serde_json::Value>() else {
        return (
            Vec::new(),
            Some("Respuesta de usage de OpenCode inválida.".to_owned()),
        );
    };
    let mut windows = Vec::new();
    if let Some(usage) = body.get("usage") {
        for (key, label) in [
            ("rolling", "Rolling"),
            ("weekly", "Weekly"),
            ("monthly", "Monthly"),
        ] {
            let Some(entry) = usage.get(key) else {
                continue;
            };
            let Some(percent) = entry.get("percent").and_then(|value| value.as_f64()) else {
                continue;
            };
            windows.push(UsageWindow {
                label: label.to_owned(),
                used_percent: percent.clamp(0.0, 100.0),
                resets_at: entry
                    .get("resetsAt")
                    .and_then(|value| value.as_str())
                    .map(str::to_owned),
            });
        }
    }
    (windows, None)
}

fn opencode_usage() -> ProviderUsage {
    let mut usage = ProviderUsage {
        provider: "opencode".to_owned(),
        label: "OpenCode".to_owned(),
        status: "unavailable".to_owned(),
        message: None,
        source: Some("opencode stats + OpenCode Go".to_owned()),
        cost: None,
        tokens: None,
        sessions: None,
        details: Vec::new(),
        windows: Vec::new(),
        checked_at: now(),
    };
    let Some(opencode) = resolved("opencode") else {
        usage.status = "unavailable".to_owned();
        usage.message = Some("OpenCode no está instalado.".to_owned());
        return usage;
    };
    let stats_error = match Command::new(opencode)
        .args(["stats", "--days", "30"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout);
            usage.cost = field(&text, "Total Cost").and_then(|value| {
                value
                    .split('$')
                    .next_back()
                    .and_then(|value| value.trim().parse::<f64>().ok())
            });
            usage.tokens = field(&text, "Input")
                .and_then(compact_number)
                .zip(field(&text, "Output").and_then(compact_number))
                .zip(field(&text, "Cache Read").and_then(compact_number))
                .map(|((input, output), cache)| (input + output + cache) as u64);
            usage.sessions = field(&text, "Sessions").and_then(|value| value.parse().ok());
            usage.details = [
                ("Sesiones", usage.sessions),
                (
                    "Input",
                    field(&text, "Input")
                        .and_then(compact_number)
                        .map(|v| v as u64),
                ),
                (
                    "Output",
                    field(&text, "Output")
                        .and_then(compact_number)
                        .map(|v| v as u64),
                ),
                (
                    "Cache read",
                    field(&text, "Cache Read")
                        .and_then(compact_number)
                        .map(|v| v as u64),
                ),
            ]
            .into_iter()
            .filter_map(|(label, value)| {
                value.map(|value| UsageMetric {
                    label: label.to_owned(),
                    value: value.to_string(),
                })
            })
            .collect();
            None
        }
        Ok(output) => Some(String::from_utf8_lossy(&output.stderr).trim().to_owned()),
        Err(error) => Some(error.to_string()),
    };
    let (windows, quota_error) = opencode_quota();
    usage.windows = windows;
    if usage.windows.is_empty() && usage.details.is_empty() {
        usage.status = "unavailable".to_owned();
        usage.message = Some(stats_error.or(quota_error).unwrap_or_else(|| {
            "OpenCode no devolvió métricas ni cuota para la cuenta actual.".to_owned()
        }));
    } else {
        usage.status = "ready".to_owned();
        usage.message = quota_error.or(stats_error);
    }
    usage
}

fn static_provider(provider: &str, label: &str, binary: &str) -> ProviderUsage {
    let installed = resolved(binary).is_some();
    ProviderUsage {
        provider: provider.to_owned(),
        label: label.to_owned(),
        status: if installed {
            "unsupported"
        } else {
            "unavailable"
        }
        .to_owned(),
        message: Some(if installed {
            "El CLI no expone cuotas de cuenta; se puede agregar un adapter específico.".to_owned()
        } else {
            format!("{label} no está instalado.")
        }),
        source: None,
        cost: None,
        tokens: None,
        sessions: None,
        details: Vec::new(),
        windows: Vec::new(),
        checked_at: now(),
    }
}

#[derive(serde::Deserialize)]
struct AgyBucket {
    name: Option<String>,
    remaining_fraction: Option<f64>,
    reset_time: Option<String>,
    #[serde(default)]
    disabled: bool,
}

#[derive(serde::Deserialize)]
struct AgyGroup {
    name: Option<String>,
    buckets: Vec<AgyBucket>,
}

#[derive(serde::Deserialize)]
struct AgyData {
    groups: Vec<AgyGroup>,
    description: Option<String>,
}

#[derive(serde::Deserialize)]
struct AgyCommand {
    data: Option<AgyData>,
}

#[derive(serde::Deserialize)]
struct AgyEnvelope {
    status: String,
    command: Option<AgyCommand>,
}

fn antigravity_usage() -> ProviderUsage {
    let mut usage = static_provider("antigravity", "Antigravity", "agy");
    if usage.status != "unsupported" {
        return usage;
    }
    let Some(agy) = resolved("agy") else {
        return usage;
    };
    let output = Command::new(agy)
        .args([
            "-p",
            "/usage",
            "--output-format",
            "json",
            "--print-timeout",
            "20s",
        ])
        .env("AGY_CLI_DISABLE_AUTO_UPDATE", "true")
        .output();
    let Ok(output) = output else {
        usage.status = "error".to_owned();
        usage.message = Some("No se pudo ejecutar agy /usage.".to_owned());
        return usage;
    };
    if !output.status.success() {
        usage.status = "error".to_owned();
        usage.message = Some(String::from_utf8_lossy(&output.stderr).trim().to_owned());
        return usage;
    }
    let envelope: AgyEnvelope = match serde_json::from_slice(&output.stdout) {
        Ok(value) => value,
        Err(error) => {
            usage.status = "error".to_owned();
            usage.message = Some(format!("Respuesta de agy inválida: {error}"));
            return usage;
        }
    };
    if envelope.status != "SUCCESS" {
        usage.status = "error".to_owned();
        usage.message = Some("agy no devolvió usage.".to_owned());
        return usage;
    }
    let Some(data) = envelope.command.and_then(|command| command.data) else {
        usage.status = "error".to_owned();
        usage.message = Some("agy no incluyó datos de usage.".to_owned());
        return usage;
    };
    for group in data.groups {
        let group_name = group.name.unwrap_or_else(|| "Antigravity".to_owned());
        for bucket in group.buckets {
            if bucket.disabled {
                continue;
            }
            let Some(fraction) = bucket.remaining_fraction.filter(|value| value.is_finite()) else {
                continue;
            };
            let used = ((1.0 - fraction).clamp(0.0, 1.0) * 100.0).round();
            let label = match bucket.name {
                Some(name) => format!("{group_name} · {name}"),
                None => group_name.clone(),
            };
            usage.windows.push(UsageWindow {
                label,
                used_percent: used,
                resets_at: bucket.reset_time,
            });
        }
    }
    if let Some(window) = usage
        .windows
        .iter()
        .find(|window| window.used_percent >= 60.0)
    {
        usage.details.push(UsageMetric {
            label: "Ventana más usada".to_owned(),
            value: format!("{:.0}%", window.used_percent),
        });
    }
    if !usage.windows.is_empty() {
        usage.status = "ready".to_owned();
        usage.message = data.description;
    }
    usage
}

fn command_code_usage() -> ProviderUsage {
    let Some(cmd) = resolved("cmd") else {
        return static_provider("commandcode", "Command Code", "cmd");
    };
    let mut usage = static_provider("commandcode", "Command Code", "cmd");
    usage.source = Some("cmd status".to_owned());
    let output = match Command::new(cmd).arg("status").output() {
        Ok(output) => output,
        Err(error) => {
            usage.status = "error".to_owned();
            usage.message = Some(format!("No se pudo consultar Command Code: {error}"));
            return usage;
        }
    };
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if text.contains("Not authenticated") {
        usage.status = "signedOut".to_owned();
        usage.message = Some("Command Code no está autenticado.".to_owned());
    } else if text.contains("Authenticated") {
        usage.status = "unsupported".to_owned();
        usage.message = Some(
            "Command Code está autenticado, pero su CLI no expone cuota de suscripción.".to_owned(),
        );
    } else {
        usage.message = Some("Command Code no devolvió un estado reconocible.".to_owned());
    }
    usage
}

pub fn report() -> Vec<ProviderUsage> {
    vec![
        opencode_usage(),
        static_provider("claude", "Claude Code", "claude"),
        antigravity_usage(),
        command_code_usage(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_compact_numbers() {
        assert_eq!(compact_number("44.2M"), Some(44_200_000.0));
        assert_eq!(compact_number("1,234"), Some(1_234.0));
    }

    #[test]
    fn parses_opencode_fields() {
        let text = "│ Total Cost $19.50 │\n│ Input 44.2M │\n│ Output 3.2M │\n│ Sessions 52 │";
        assert_eq!(field(text, "Total Cost"), Some("$19.50"));
        assert_eq!(
            compact_number(field(text, "Input").expect("input field")),
            Some(44_200_000.0)
        );
    }
}
