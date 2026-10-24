use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use ancleto_code_core::drivers::resolve_binary;

use super::{now_secs, ProviderUsage, UsageMetric, UsageWindow};

const OPENCODE_GO_USAGE_URL: &str = "https://opencode.ai/zen/go/v1/usage";
const COMMAND_CODE_API_URL: &str = "https://api.commandcode.ai";
const HTTP_TIMEOUT: Duration = Duration::from_secs(5);
const PROCESS_TIMEOUT: Duration = Duration::from_secs(14);

pub trait UsageProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str {
        self.id()
    }
    fn collect(&self) -> ProviderUsage;
}

pub fn default_providers() -> Vec<Box<dyn UsageProvider>> {
    vec![
        Box::new(OpencodeProvider),
        Box::new(ClaudeProvider),
        Box::new(AntigravityProvider),
        Box::new(CommandCodeProvider),
    ]
}

pub struct OpencodeProvider;

impl UsageProvider for OpencodeProvider {
    fn id(&self) -> &'static str {
        "opencode"
    }

    fn label(&self) -> &'static str {
        "OpenCode"
    }

    fn collect(&self) -> ProviderUsage {
        opencode_usage()
    }
}

pub struct ClaudeProvider;

impl UsageProvider for ClaudeProvider {
    fn id(&self) -> &'static str {
        "claude"
    }

    fn label(&self) -> &'static str {
        "Claude Code"
    }

    fn collect(&self) -> ProviderUsage {
        static_provider("claude", "Claude Code", "claude")
    }
}

pub struct AntigravityProvider;

impl UsageProvider for AntigravityProvider {
    fn id(&self) -> &'static str {
        "antigravity"
    }

    fn label(&self) -> &'static str {
        "Antigravity"
    }

    fn collect(&self) -> ProviderUsage {
        antigravity_usage()
    }
}

pub struct CommandCodeProvider;

impl UsageProvider for CommandCodeProvider {
    fn id(&self) -> &'static str {
        "commandcode"
    }

    fn label(&self) -> &'static str {
        "Command Code"
    }

    fn collect(&self) -> ProviderUsage {
        command_code_usage()
    }
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

fn terminate(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let pid = child.id() as i32;
        // SAFETY: pid belongs to the process group created with process_group(0).
        unsafe {
            libc::kill(-pid, libc::SIGKILL);
        }
    }
    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }
    let _ = child.wait();
}

fn read_all(mut stream: Option<impl Read>) -> Vec<u8> {
    let mut buffer = Vec::new();
    if let Some(stream) = stream.as_mut() {
        let _ = stream.read_to_end(&mut buffer);
    }
    buffer
}

pub(crate) fn run_with_timeout(mut command: Command, timeout: Duration) -> Result<Output, String> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .spawn()
        .map_err(|error| format!("no se pudo ejecutar el comando: {error}"))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_reader = std::thread::spawn(move || read_all(stdout));
    let stderr_reader = std::thread::spawn(move || read_all(stderr));
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                terminate(&mut child);
                return Err(format!(
                    "excedió el tiempo límite de {} s",
                    timeout.as_secs()
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(error) => {
                terminate(&mut child);
                return Err(error.to_string());
            }
        }
    };
    Ok(Output {
        status,
        stdout: stdout_reader.join().unwrap_or_default(),
        stderr: stderr_reader.join().unwrap_or_default(),
    })
}

fn command_code_auth_key() -> Option<String> {
    if let Ok(key) = std::env::var("COMMAND_CODE_API_KEY") {
        if !key.trim().is_empty() {
            return Some(key);
        }
    }
    let mut candidates = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(PathBuf::from(home).join(".commandcode/auth.json"));
    }
    if cfg!(windows) {
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            candidates.push(PathBuf::from(profile).join(".commandcode/auth.json"));
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
            .get("apiKey")
            .and_then(|key| key.as_str())
            .filter(|key| !key.trim().is_empty())
        {
            return Some(key.to_owned());
        }
    }
    None
}

fn command_code_get(
    client: &reqwest::blocking::Client,
    key: &str,
    endpoint: &str,
) -> Result<serde_json::Value, String> {
    let response = client
        .get(format!("{COMMAND_CODE_API_URL}{endpoint}"))
        .header("Authorization", format!("Bearer {key}"))
        .header("Accept", "application/json")
        .send()
        .map_err(|_| "No se pudo consultar la API de Command Code.".to_owned())?;
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err("HTTP 401".to_owned());
    }
    if !response.status().is_success() {
        return Err(format!("Command Code API respondió {}.", response.status()));
    }
    response
        .json::<serde_json::Value>()
        .map_err(|_| "Respuesta inválida de Command Code API.".to_owned())
}

fn command_code_usage() -> ProviderUsage {
    let mut usage = static_provider("commandcode", "Command Code", "cmd");
    let Some(key) = command_code_auth_key() else {
        usage.status = "signedOut".to_owned();
        usage.message = Some("Command Code no tiene una API key guardada.".to_owned());
        return usage;
    };
    let client = match reqwest::blocking::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            usage.status = "error".to_owned();
            usage.message = Some(error.to_string());
            return usage;
        }
    };
    let credits = match command_code_get(&client, &key, "/alpha/billing/credits") {
        Ok(value) => value,
        Err(error) if error == "HTTP 401" => {
            usage.status = "signedOut".to_owned();
            usage.message = Some("La API key de Command Code no es válida.".to_owned());
            return usage;
        }
        Err(error) => {
            usage.status = "error".to_owned();
            usage.message = Some(error);
            return usage;
        }
    };
    let subscription = command_code_get(&client, &key, "/alpha/billing/subscriptions").ok();
    let summary = command_code_get(&client, &key, "/alpha/usage/summary").ok();
    usage.source = Some("Command Code API".to_owned());

    if let Some(limits) = credits.get("windowLimits") {
        for (key, label) in [("fiveHour", "5-hour"), ("weekly", "Weekly")] {
            let Some(window) = limits.get(key) else {
                continue;
            };
            let used = window.get("used").and_then(|value| value.as_f64());
            let cap = window.get("cap").and_then(|value| value.as_f64());
            if let (Some(used), Some(cap)) = (used, cap) {
                if cap > 0.0 {
                    usage.windows.push(UsageWindow {
                        label: label.to_owned(),
                        used_percent: (used / cap * 100.0).clamp(0.0, 100.0),
                        resets_at: window
                            .get("resetAt")
                            .and_then(|value| value.as_u64())
                            .map(|value| value.to_string()),
                    });
                }
            }
        }
    }
    let total_cost = summary
        .as_ref()
        .and_then(|value| value.get("totalCost"))
        .and_then(|value| value.as_f64());
    let remaining = credits
        .get("credits")
        .and_then(|value| value.get("monthlyCredits"))
        .and_then(|value| value.as_f64());
    if let (Some(used), Some(remaining)) = (total_cost, remaining) {
        let cap = used + remaining.max(0.0);
        if cap > 0.0 {
            usage.windows.push(UsageWindow {
                label: "Monthly".to_owned(),
                used_percent: (used / cap * 100.0).clamp(0.0, 100.0),
                resets_at: subscription
                    .as_ref()
                    .and_then(|value| value.pointer("/data/currentPeriodEnd"))
                    .and_then(|value| value.as_str())
                    .map(str::to_owned),
            });
            usage.details.push(UsageMetric {
                label: "Gasto".to_owned(),
                value: format!("${used:.2} / ${cap:.2}"),
            });
        }
    }
    if let Some(plan_id) = subscription
        .as_ref()
        .and_then(|value| value.pointer("/data/planId"))
        .and_then(|value| value.as_str())
    {
        let plan = plan_id
            .rsplit('-')
            .next()
            .unwrap_or(plan_id)
            .to_ascii_uppercase();
        usage.details.insert(
            0,
            UsageMetric {
                label: "Plan".to_owned(),
                value: format!("{plan} Plan"),
            },
        );
    }
    if let Some(total) = summary
        .as_ref()
        .and_then(|value| value.get("totalCount"))
        .and_then(|value| value.as_u64())
    {
        usage.details.push(UsageMetric {
            label: "Requests este mes".to_owned(),
            value: total.to_string(),
        });
    }
    if usage.windows.is_empty() {
        usage.status = "error".to_owned();
        usage.message = Some("Command Code no devolvió límites de uso.".to_owned());
    } else {
        usage.status = "ready".to_owned();
        usage.message = None;
    }
    usage
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
        .timeout(HTTP_TIMEOUT)
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
        checked_at: now_secs(),
        duration_ms: None,
    };
    let Some(opencode) = resolved("opencode") else {
        usage.status = "unavailable".to_owned();
        usage.message = Some("OpenCode no está instalado.".to_owned());
        return usage;
    };
    let stats = {
        let mut command = Command::new(opencode);
        command.args(["stats", "--days", "30"]);
        run_with_timeout(command, PROCESS_TIMEOUT)
    };
    let stats_error = match stats {
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
        Err(error) => Some(error),
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
        checked_at: now_secs(),
        duration_ms: None,
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
    let mut command = Command::new(agy);
    command
        .args([
            "-p",
            "/usage",
            "--output-format",
            "json",
            "--print-timeout",
            "12s",
        ])
        .env("AGY_CLI_DISABLE_AUTO_UPDATE", "true");
    let output = match run_with_timeout(command, PROCESS_TIMEOUT) {
        Ok(output) => output,
        Err(error) => {
            usage.status = "error".to_owned();
            usage.message = Some(format!("No se pudo ejecutar agy /usage: {error}"));
            return usage;
        }
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

    #[cfg(unix)]
    #[test]
    fn watchdog_kills_processes_that_exceed_the_budget() {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 30"]);
        let started = Instant::now();
        let result = run_with_timeout(command, Duration::from_millis(150));
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[cfg(unix)]
    #[test]
    fn watchdog_kills_the_whole_process_group() {
        let pidfile = std::env::temp_dir().join(format!(
            "ancleto-watchdog-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let script = format!("sleep 30 & echo $! > '{}'; wait", pidfile.display());
        let mut command = Command::new("sh");
        command.args(["-c", &script]);
        let started = Instant::now();
        let result = run_with_timeout(command, Duration::from_millis(200));
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(3));
        let pid = std::fs::read_to_string(&pidfile)
            .expect("pidfile")
            .trim()
            .parse::<i32>()
            .expect("pid");
        std::thread::sleep(Duration::from_millis(100));
        // SAFETY: signal 0 only checks whether the process group member is alive.
        let alive = unsafe { libc::kill(pid, 0) } == 0;
        assert!(!alive, "el proceso nieto sobrevivió al watchdog");
        let _ = std::fs::remove_file(&pidfile);
    }

    #[cfg(unix)]
    #[test]
    fn watchdog_returns_output_of_fast_processes() {
        let mut command = Command::new("sh");
        command.args(["-c", "printf hola"]);
        let output = run_with_timeout(command, Duration::from_secs(5)).expect("output");
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "hola");
    }
}
