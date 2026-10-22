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
    fn hooks_emit_osc_133_marks() {
        for id in ["bash", "zsh", "fish", "powershell"] {
            let script = hook_script(id).unwrap();
            assert!(script.contains("133;"), "hook de {id} sin OSC 133");
        }
        assert!(hook_script("cmd").is_none());
    }
}
