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

fn title_token(token: &str) -> String {
    let mut chars = token.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    if first.is_ascii_digit() {
        let mut value = token.to_owned();
        if let Some(last) = value.chars().last() {
            if last.is_ascii_alphabetic() {
                let upper = last.to_ascii_uppercase();
                value.pop();
                value.push(upper);
            }
        }
        return value;
    }
    let mut value = first.to_uppercase().collect::<String>();
    value.push_str(chars.as_str());
    value
}

fn title_case(value: &str) -> String {
    value
        .split('-')
        .filter(|part| !part.is_empty())
        .map(title_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn humanize_model_id(id: &str) -> String {
    let name = id.rsplit('/').next().unwrap_or(id);
    let lower = name.to_lowercase();
    let brands = [
        ("deepseek", "DeepSeek"),
        ("minimax", "MiniMax"),
        ("moonshotai", "Moonshot"),
        ("glm", "GLM"),
        ("mimo", "MiMo"),
        ("kimi", "Kimi"),
        ("claude", "Claude"),
        ("gemini", "Gemini"),
        ("grok", "Grok"),
        ("gpt-oss", "GPT-OSS"),
        ("gpt", "GPT"),
        ("openai", "OpenAI"),
        ("qwen", "Qwen"),
        ("ornith", "Ornith"),
        ("longcat", "LongCat"),
        ("step", "Step"),
    ];
    for (prefix, brand) in brands {
        if lower == prefix {
            return brand.to_owned();
        }
        if lower.starts_with(&format!("{prefix}-")) {
            let rest = title_case(&name[prefix.len() + 1..]);
            return if rest.is_empty() {
                brand.to_owned()
            } else {
                format!("{brand} {rest}")
            };
        }
        if prefix == "qwen" && lower.starts_with("qwen") {
            return format!("Qwen {}", title_case(&name[4..]));
        }
    }
    title_case(name)
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
                label: humanize_model_id(id),
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
    fn humanizes_command_code_ids() {
        assert_eq!(
            humanize_model_id("deepseek/deepseek-v4-flash"),
            "DeepSeek V4 Flash"
        );
        assert_eq!(humanize_model_id("xiaomi/mimo-v2.6-pro"), "MiMo V2.6 Pro");
        assert_eq!(humanize_model_id("z-ai/glm-5.3-flash"), "GLM 5.3 Flash");
        assert_eq!(humanize_model_id("moonshotai/kimi-k3"), "Kimi K3");
    }

    #[test]
    fn parses_command_code_models() {
        let models = parse_cmd(
            "Available models\n\nOpen Source\n\ndeepseek/deepseek-v4-flash  fast reasoning\n",
        );
        assert_eq!(models[0].id, "deepseek/deepseek-v4-flash");
        assert_eq!(models[0].label, "DeepSeek V4 Flash");
    }
}
