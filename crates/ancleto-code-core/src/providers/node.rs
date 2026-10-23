use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::drivers::resolve_binary;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeRuntime {
    pub available: bool,
    pub version: Option<String>,
    pub binary_path: Option<String>,
    pub compatible: bool,
}

impl NodeRuntime {
    pub fn detect() -> Self {
        let path = std::env::var_os("ANCLETO_SPEC_NODE")
            .map(std::path::PathBuf::from)
            .filter(|path| path.is_file())
            .or_else(|| resolve_binary("node"));
        let Some(path) = path else {
            return Self {
                available: false,
                version: None,
                binary_path: None,
                compatible: false,
            };
        };

        let output = Command::new(&path).arg("--version").output();
        let version = output.ok().and_then(|output| {
            let text = String::from_utf8_lossy(&output.stdout);
            text.lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .map(str::to_owned)
        });
        let compatible = version
            .as_deref()
            .and_then(major_version)
            .is_some_and(|major| major >= 24);

        Self {
            available: true,
            version,
            binary_path: Some(path.to_string_lossy().into_owned()),
            compatible,
        }
    }
}

fn major_version(version: &str) -> Option<u32> {
    version
        .trim_start_matches('v')
        .split('.')
        .next()
        .and_then(|major| major.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_major_version() {
        assert_eq!(major_version("v24.21.0"), Some(24));
        assert_eq!(major_version("24.1.0"), Some(24));
        assert_eq!(major_version("v22.9.0"), Some(22));
        assert_eq!(major_version("sin version"), None);
    }
}
