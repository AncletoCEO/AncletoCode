use ancleto_code_core::Store;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub background: (u8, u8, u8),
    pub foreground: (u8, u8, u8),
    pub accent: (u8, u8, u8),
    pub selection: (u8, u8, u8),
    pub cursor: (u8, u8, u8),
    pub border: (u8, u8, u8),
    pub danger: (u8, u8, u8),
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: (13, 17, 23),
            foreground: (230, 237, 243),
            accent: (88, 166, 255),
            selection: (38, 79, 120),
            cursor: (88, 166, 255),
            border: (48, 54, 61),
            danger: (248, 81, 73),
        }
    }
}

pub fn load(store: &Store, requested: Option<&str>) -> Theme {
    let global = store
        .get_setting("appearance.global")
        .ok()
        .flatten()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok());
    let theme_id = requested.map(str::to_owned).or_else(|| {
        global
            .as_ref()
            .and_then(|value| value.get("themeId"))
            .and_then(Value::as_str)
            .map(str::to_owned)
    });

    if let Some(id) = theme_id {
        if let Some(theme) = store
            .get_setting("appearance.themes")
            .ok()
            .flatten()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .and_then(|value| custom_theme(&value, &id))
        {
            return theme;
        }
        if let Some(theme) = preset(&id) {
            return theme;
        }
    }

    Theme::default()
}

fn custom_theme(value: &Value, id: &str) -> Option<Theme> {
    let themes = value.as_array()?;
    let manifest = themes
        .iter()
        .find(|entry| entry.get("id").and_then(Value::as_str) == Some(id))?;
    let tokens = manifest.get("tokens")?;
    let get = |name: &str, fallback: (u8, u8, u8)| {
        tokens
            .get(name)
            .and_then(Value::as_str)
            .and_then(hex)
            .unwrap_or(fallback)
    };
    let default = Theme::default();
    Some(Theme {
        background: get("bgBase", default.background),
        foreground: get("fg", default.foreground),
        accent: get("accent", default.accent),
        selection: get("selection", default.selection),
        cursor: get("cursor", default.cursor),
        border: get("border", default.border),
        danger: get("danger", default.danger),
    })
}

fn preset(id: &str) -> Option<Theme> {
    let theme = match id {
        "ancleto-dark" => Theme::default(),
        "carbon" => Theme {
            background: (22, 22, 22),
            foreground: (244, 244, 244),
            accent: (120, 169, 255),
            selection: (0, 67, 206),
            cursor: (120, 169, 255),
            border: (57, 57, 57),
            danger: (250, 77, 86),
        },
        "solarized" => Theme {
            background: (0, 43, 54),
            foreground: (238, 232, 213),
            accent: (42, 161, 152),
            selection: (11, 79, 92),
            cursor: (42, 161, 152),
            border: (15, 74, 90),
            danger: (220, 50, 47),
        },
        "nord" => Theme {
            background: (46, 52, 64),
            foreground: (236, 239, 244),
            accent: (136, 192, 208),
            selection: (76, 86, 106),
            cursor: (136, 192, 208),
            border: (76, 86, 106),
            danger: (191, 97, 106),
        },
        "dracula" => Theme {
            background: (40, 42, 54),
            foreground: (248, 248, 242),
            accent: (189, 147, 249),
            selection: (68, 71, 90),
            cursor: (189, 147, 249),
            border: (98, 114, 164),
            danger: (255, 85, 85),
        },
        "ancleto-light" => Theme {
            background: (246, 248, 250),
            foreground: (31, 35, 40),
            accent: (5, 80, 174),
            selection: (182, 215, 255),
            cursor: (5, 80, 174),
            border: (208, 215, 222),
            danger: (207, 34, 46),
        },
        _ => return None,
    };
    Some(theme)
}

fn hex(value: &str) -> Option<(u8, u8, u8)> {
    let value = value.trim().trim_start_matches('#');
    if value.len() != 6 {
        return None;
    }
    Some((
        u8::from_str_radix(&value[0..2], 16).ok()?,
        u8::from_str_radix(&value[2..4], 16).ok()?,
        u8::from_str_radix(&value[4..6], 16).ok()?,
    ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_colors() {
        assert_eq!(hex("#58a6ff"), Some((88, 166, 255)));
        assert_eq!(hex("58a6ff"), Some((88, 166, 255)));
        assert_eq!(hex("zzz"), None);
    }

    #[test]
    fn falls_back_to_the_default_theme() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(&directory.path().join("state.sqlite")).unwrap();
        assert_eq!(load(&store, None), Theme::default());
        assert_eq!(load(&store, Some("desconocido")), Theme::default());
    }

    #[test]
    fn reads_presets_and_custom_themes() {
        assert_eq!(preset("carbon").unwrap().background, (22, 22, 22));

        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(&directory.path().join("state.sqlite")).unwrap();
        let themes = serde_json::json!([{
            "id": "mia",
            "tokens": { "bgBase": "#010203", "fg": "#fefdfc", "accent": "#112233" }
        }]);
        store
            .set_setting("appearance.themes", &themes.to_string())
            .unwrap();
        let theme = load(&store, Some("mia"));
        assert_eq!(theme.background, (1, 2, 3));
        assert_eq!(theme.foreground, (254, 253, 252));
    }
}
