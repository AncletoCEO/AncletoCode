use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellProfile {
    pub id: String,
    pub display_name: String,
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub osc_integration: bool,
}

impl ShellProfile {
    fn new(
        id: &str,
        display_name: &str,
        program: &str,
        args: &[&str],
        osc_integration: bool,
    ) -> Self {
        Self {
            id: id.to_owned(),
            display_name: display_name.to_owned(),
            program: program.to_owned(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            env: Vec::new(),
            osc_integration,
        }
    }
}

pub fn default_profiles() -> Vec<ShellProfile> {
    #[cfg(windows)]
    {
        vec![
            ShellProfile::new("cmd", "Command Prompt", "cmd.exe", &[], false),
            ShellProfile::new(
                "powershell",
                "PowerShell",
                "powershell.exe",
                &["-NoLogo"],
                true,
            ),
            ShellProfile::new("wsl", "WSL", "wsl.exe", &[], true),
        ]
    }
    #[cfg(not(windows))]
    {
        vec![
            ShellProfile::new("bash", "bash", "bash", &["-l"], true),
            ShellProfile::new("zsh", "zsh", "zsh", &["-l"], true),
            ShellProfile::new("fish", "fish", "fish", &["-l"], true),
            ShellProfile::new("sh", "sh", "sh", &[], false),
        ]
    }
}

pub fn profile(id: &str) -> Option<ShellProfile> {
    default_profiles()
        .into_iter()
        .find(|profile| profile.id == id)
}

const MARKER_START: &str = "# >>> ancleto osc133 >>>";
const MARKER_END: &str = "# <<< ancleto osc133 <<<";

pub fn install_hooks(profile_id: &str) -> std::io::Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    install_hooks_to(profile_id, &home)
}

pub fn install_hooks_to(profile_id: &str, home: &Path) -> std::io::Result<PathBuf> {
    let (path, snippet) = match profile_id {
        "bash" => (home.join(".bashrc"), BASH_HOOKS),
        "zsh" => (home.join(".zshrc"), ZSH_HOOKS),
        "fish" => {
            let config = std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config"));
            (config.join("fish").join("config.fish"), FISH_HOOKS)
        }
        "powershell" => (
            home.join(".config")
                .join("powershell")
                .join("Microsoft.PowerShell_profile.ps1"),
            POWERSHELL_HOOKS,
        ),
        "wsl" => (home.join(".bashrc"), BASH_HOOKS),
        other => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("shell sin integración OSC 133: {other}"),
            ))
        }
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let block = format!("{MARKER_START}\n{snippet}{MARKER_END}\n");
    let updated = if let (Some(start), Some(end)) =
        (existing.find(MARKER_START), existing.find(MARKER_END))
    {
        let end = end + MARKER_END.len();
        format!(
            "{}{}{}",
            &existing[..start],
            block,
            existing[end..].trim_start_matches('\n')
        )
    } else if existing.trim().is_empty() {
        block
    } else {
        format!("{}\n{block}", existing.trim_end())
    };
    fs::write(&path, updated)?;
    Ok(path)
}

pub fn hook_script(id: &str) -> Option<&'static str> {
    match id {
        "bash" => Some(BASH_HOOKS),
        "zsh" => Some(ZSH_HOOKS),
        "fish" => Some(FISH_HOOKS),
        "powershell" => Some(POWERSHELL_HOOKS),
        "wsl" => Some(BASH_HOOKS),
        _ => None,
    }
}

pub const BASH_HOOKS: &str = r#"__ancleto_prompt() {
  local status=$?
  printf '\033]133;D;%s\007' "$status"
  printf '\033]133;A\007'
}
__ancleto_preexec() {
  printf '\033]133;B\007'
}
trap '__ancleto_preexec' DEBUG
PROMPT_COMMAND="__ancleto_prompt${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
"#;

pub const ZSH_HOOKS: &str = r#"__ancleto_preexec() { printf '\033]133;B\007' }
__ancleto_precmd() {
  local status=$?
  printf '\033]133;D;%s\007' "$status"
  printf '\033]133;A\007'
}
preexec_functions+=(__ancleto_preexec)
precmd_functions+=(__ancleto_precmd)
"#;

pub const FISH_HOOKS: &str = r#"function __ancleto_preexec --on-event fish_preexec
  printf '\033]133;B\007'
end
function __ancleto_postexec --on-event fish_postexec
  printf '\033]133;D;%s\007' $status
  printf '\033]133;A\007'
end
"#;

pub const POWERSHELL_HOOKS: &str = r#"function global:prompt {
  $status = if ($?) { 0 } else { 1 }
  [Console]::Write("`e]133;D;$status`a")
  [Console]::Write("`e]133;A`a")
  "PS $($executionContext.SessionState.Path.CurrentLocation)> "
}
"#;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn provides_platform_profiles() {
        let profiles = default_profiles();
        assert!(!profiles.is_empty());
        assert!(profiles.iter().all(|profile| !profile.program.is_empty()));
        assert!(profile("bash").is_some() || profile("cmd").is_some());
    }

    #[test]
    fn installs_hooks_idempotently_and_keeps_user_content() {
        let home = tempfile::tempdir().unwrap();
        fs::write(home.path().join(".bashrc"), "alias ll='ls -l'\n").unwrap();

        let path = install_hooks_to("bash", home.path()).unwrap();
        let first = fs::read_to_string(&path).unwrap();
        assert!(first.contains("alias ll="));
        assert!(first.contains("133;A"));

        let path = install_hooks_to("bash", home.path()).unwrap();
        let second = fs::read_to_string(&path).unwrap();
        assert_eq!(
            first.matches("ancleto osc133").count(),
            second.matches("ancleto osc133").count()
        );
    }

    #[test]
    fn hooks_emit_osc_133_marks() {
        for id in ["bash", "zsh", "fish", "powershell"] {
            let script = hook_script(id).unwrap();
            assert!(script.contains("133;"), "hook de {id} sin OSC 133");
        }
        assert!(hook_script("cmd").is_none());
    }
}
