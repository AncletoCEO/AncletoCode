use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::domain::{CoreError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverModel {
    pub id: String,
    pub label: String,
}

pub fn list(driver_id: &str, binary: &str) -> Result<Vec<DriverModel>> {
    let args = match driver_id {
        "agy" | "opencode" => vec!["models"],
        "cmd" | "commandcode" => vec!["--list-models"],
        _ => return Ok(Vec::new()),
    };
    let output = Command::new(binary)
        .args(args)
        .output()
        .map_err(|error| CoreError::Process(format!("no se pudo listar modelos: {error}")))?;
    if !output.status.success() {
        return Err(CoreError::Process(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(match driver_id {
        "agy" => parse_agy(&text),
        "opencode" => parse_lines(&text),
        "cmd" | "commandcode" => parse_cmd(&text),
        _ => Vec::new(),
    })
}

fn parse_agy(text: &str) -> Vec<DriverModel> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            let (id, label) = line.split_once('\t')?;
            let id = id.trim();
            if id.is_empty() || id.contains(' ') {
                return None;
            }
            Some(DriverModel {
                id: id.to_owned(),
                label: label.trim().to_owned(),
            })
        })
        .collect()
}

fn parse_lines(text: &str) -> Vec<DriverModel> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.contains(' '))
        .map(|line| DriverModel {
            id: line.to_owned(),
            label: line.to_owned(),
        })
        .collect()
}

fn parse_cmd(text: &str) -> Vec<DriverModel> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty()
                || line.starts_with("Available models")
                || line.starts_with("Open Source")
                || line.starts_with("Proprietary")
                || line == "Models"
            {
                return None;
            }
            let mut parts = line.split_whitespace();
            let id = parts.next()?;
            if !id.contains('/') {
                return None;
            }
            Some(DriverModel {
                id: id.to_owned(),
                label: parts.collect::<Vec<_>>().join(" "),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_agy_models() {
        let models = parse_agy(
            "Fetching available models...\ngemini-4-argon-medium\tGemini 4 Argon (Medium)\n",
        );
        assert_eq!(models[0].id, "gemini-4-argon-medium");
        assert_eq!(models[0].label, "Gemini 4 Argon (Medium)");
    }

    #[test]
    fn parses_command_code_models() {
        let models = parse_cmd(
            "Available models\n\nOpen Source\n\ndeepseek/deepseek-v4-flash  fast reasoning\n",
        );
        assert_eq!(models[0].id, "deepseek/deepseek-v4-flash");
        assert_eq!(models[0].label, "fast reasoning");
    }
}
