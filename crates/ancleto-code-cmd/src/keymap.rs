use std::collections::HashMap;

use ancleto_code_core::{Result, Store};
use serde::{Deserialize, Serialize};
use winit::event::KeyEvent;
use winit::keyboard::{Key, ModifiersState, NamedKey};

pub const SETTING_KEY: &str = "terminal.keymap";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    NewTab,
    ClosePanel,
    SplitHorizontal,
    SplitVertical,
    Copy,
    Paste,
    Quit,
    KillAll,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    NextPanel,
    PreviousTab,
    NextTab,
    ResizeLeft,
    ResizeRight,
    ResizeUp,
    ResizeDown,
    SwapPanel,
    PreviousCommand,
    NextCommand,
    CopyCommandOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Keymap {
    pub new_tab: String,
    pub close_panel: String,
    pub split_horizontal: String,
    pub split_vertical: String,
    pub copy: String,
    pub paste: String,
    pub quit: String,
    pub kill_all: String,
    pub zoom_in: String,
    pub zoom_out: String,
    pub zoom_reset: String,
    pub next_panel: String,
    pub previous_tab: String,
    pub next_tab: String,
    pub resize_left: String,
    pub resize_right: String,
    pub resize_up: String,
    pub resize_down: String,
    pub swap_panel: String,
    pub previous_command: String,
    pub next_command: String,
    pub copy_command_output: String,
}

impl Default for Keymap {
    fn default() -> Self {
        Self {
            new_tab: "ctrl+shift+t".to_owned(),
            close_panel: "ctrl+shift+w".to_owned(),
            split_horizontal: "ctrl+shift+e".to_owned(),
            split_vertical: "ctrl+shift+d".to_owned(),
            copy: "ctrl+shift+c".to_owned(),
            paste: "ctrl+shift+v".to_owned(),
            quit: "ctrl+shift+q".to_owned(),
            kill_all: "ctrl+shift+k".to_owned(),
            zoom_in: "ctrl+shift+plus".to_owned(),
            zoom_out: "ctrl+shift+minus".to_owned(),
            zoom_reset: "ctrl+shift+0".to_owned(),
            next_panel: "ctrl+tab".to_owned(),
            previous_tab: "ctrl+pageup".to_owned(),
            next_tab: "ctrl+pagedown".to_owned(),
            resize_left: "ctrl+shift+alt+left".to_owned(),
            resize_right: "ctrl+shift+alt+right".to_owned(),
            resize_up: "ctrl+shift+alt+up".to_owned(),
            resize_down: "ctrl+shift+alt+down".to_owned(),
            swap_panel: "ctrl+shift+alt+m".to_owned(),
            previous_command: "ctrl+shift+up".to_owned(),
            next_command: "ctrl+shift+down".to_owned(),
            copy_command_output: "ctrl+shift+o".to_owned(),
        }
    }
}

impl Keymap {
    pub fn load(store: &Store) -> Self {
        let mut keymap = Self::default();
        if let Ok(Some(text)) = store.get_setting(SETTING_KEY) {
            if let Ok(overrides) = serde_json::from_str::<HashMap<String, String>>(&text) {
                keymap.apply(&overrides);
            }
        }
        keymap
    }

    pub fn apply(&mut self, overrides: &HashMap<String, String>) {
        for (name, chord) in overrides {
            let target = match name.as_str() {
                "newTab" => Some(&mut self.new_tab),
                "closePanel" => Some(&mut self.close_panel),
                "splitHorizontal" => Some(&mut self.split_horizontal),
                "splitVertical" => Some(&mut self.split_vertical),
                "copy" => Some(&mut self.copy),
                "paste" => Some(&mut self.paste),
                "quit" => Some(&mut self.quit),
                "killAll" => Some(&mut self.kill_all),
                "zoomIn" => Some(&mut self.zoom_in),
                "zoomOut" => Some(&mut self.zoom_out),
                "zoomReset" => Some(&mut self.zoom_reset),
                "nextPanel" => Some(&mut self.next_panel),
                "previousTab" => Some(&mut self.previous_tab),
                "nextTab" => Some(&mut self.next_tab),
                "resizeLeft" => Some(&mut self.resize_left),
                "resizeRight" => Some(&mut self.resize_right),
                "resizeUp" => Some(&mut self.resize_up),
                "resizeDown" => Some(&mut self.resize_down),
                "swapPanel" => Some(&mut self.swap_panel),
                "previousCommand" => Some(&mut self.previous_command),
                "nextCommand" => Some(&mut self.next_command),
                "copyCommandOutput" => Some(&mut self.copy_command_output),
                _ => None,
            };
            if let Some(target) = target {
                *target = chord.clone();
            }
        }
    }

    pub fn action_for(&self, event: &KeyEvent, modifiers: ModifiersState) -> Option<Action> {
        self.entries()
            .into_iter()
            .find(|(_, chord)| matches_chord(chord, event, modifiers))
            .map(|(action, _)| action)
    }

    fn entries(&self) -> Vec<(Action, &str)> {
        vec![
            (Action::NewTab, self.new_tab.as_str()),
            (Action::ClosePanel, self.close_panel.as_str()),
            (Action::SplitHorizontal, self.split_horizontal.as_str()),
            (Action::SplitVertical, self.split_vertical.as_str()),
            (Action::Copy, self.copy.as_str()),
            (Action::Paste, self.paste.as_str()),
            (Action::Quit, self.quit.as_str()),
            (Action::KillAll, self.kill_all.as_str()),
            (Action::ZoomIn, self.zoom_in.as_str()),
            (Action::ZoomOut, self.zoom_out.as_str()),
            (Action::ZoomReset, self.zoom_reset.as_str()),
            (Action::NextPanel, self.next_panel.as_str()),
            (Action::PreviousTab, self.previous_tab.as_str()),
            (Action::NextTab, self.next_tab.as_str()),
            (Action::ResizeLeft, self.resize_left.as_str()),
            (Action::ResizeRight, self.resize_right.as_str()),
            (Action::ResizeUp, self.resize_up.as_str()),
            (Action::ResizeDown, self.resize_down.as_str()),
            (Action::SwapPanel, self.swap_panel.as_str()),
            (Action::PreviousCommand, self.previous_command.as_str()),
            (Action::NextCommand, self.next_command.as_str()),
            (Action::CopyCommandOutput, self.copy_command_output.as_str()),
        ]
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Chord {
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
    key: String,
}

fn parse_chord(chord: &str) -> Option<Chord> {
    let mut parsed = Chord {
        ctrl: false,
        shift: false,
        alt: false,
        meta: false,
        key: String::new(),
    };
    for token in chord.split('+') {
        match token.trim().to_lowercase().as_str() {
            "ctrl" | "control" => parsed.ctrl = true,
            "shift" => parsed.shift = true,
            "alt" | "option" => parsed.alt = true,
            "super" | "meta" | "cmd" | "command" => parsed.meta = true,
            key if !key.is_empty() => parsed.key = key.to_owned(),
            _ => {}
        }
    }
    (!parsed.key.is_empty()).then_some(parsed)
}

fn matches_chord(chord: &str, event: &KeyEvent, modifiers: ModifiersState) -> bool {
    let Some(parsed) = parse_chord(chord) else {
        return false;
    };
    if parsed.ctrl != modifiers.control_key()
        || parsed.shift != modifiers.shift_key()
        || parsed.alt != modifiers.alt_key()
        || parsed.meta != modifiers.super_key()
    {
        return false;
    }
    key_matches(&parsed.key, &event.logical_key)
}

fn key_matches(spec: &str, key: &Key) -> bool {
    match key {
        Key::Named(named) => named_name(named) == Some(spec),
        Key::Character(text) => {
            let text = text.to_lowercase();
            match spec {
                "plus" => text == "+" || text == "=",
                "minus" => text == "-" || text == "_",
                other => text == other,
            }
        }
        _ => false,
    }
}

fn named_name(key: &NamedKey) -> Option<&'static str> {
    Some(match key {
        NamedKey::Tab => "tab",
        NamedKey::PageUp => "pageup",
        NamedKey::PageDown => "pagedown",
        NamedKey::ArrowUp => "up",
        NamedKey::ArrowDown => "down",
        NamedKey::ArrowLeft => "left",
        NamedKey::ArrowRight => "right",
        NamedKey::Escape => "escape",
        NamedKey::Enter => "enter",
        NamedKey::Space => "space",
        NamedKey::Backspace => "backspace",
        NamedKey::Delete => "delete",
        NamedKey::Insert => "insert",
        NamedKey::Home => "home",
        NamedKey::End => "end",
        NamedKey::F1 => "f1",
        NamedKey::F2 => "f2",
        NamedKey::F3 => "f3",
        NamedKey::F4 => "f4",
        NamedKey::F5 => "f5",
        NamedKey::F6 => "f6",
        NamedKey::F7 => "f7",
        NamedKey::F8 => "f8",
        NamedKey::F9 => "f9",
        NamedKey::F10 => "f10",
        NamedKey::F11 => "f11",
        NamedKey::F12 => "f12",
        _ => return None,
    })
}

pub fn load(store: &Store) -> Result<Keymap> {
    Ok(Keymap::load(store))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn parses_chords() {
        let chord = parse_chord("ctrl+shift+alt+m").unwrap();
        assert!(chord.ctrl && chord.shift && chord.alt && !chord.meta);
        assert_eq!(chord.key, "m");
        assert!(parse_chord("ctrl+").is_none());
    }

    #[test]
    fn defaults_and_overrides_are_applied() {
        let mut keymap = Keymap::default();
        assert_eq!(keymap.new_tab, "ctrl+shift+t");

        keymap.apply(&HashMap::from([
            ("newTab".to_owned(), "ctrl+alt+n".to_owned()),
            ("inexistente".to_owned(), "ctrl+x".to_owned()),
        ]));
        assert_eq!(keymap.new_tab, "ctrl+alt+n");
        assert_eq!(keymap.close_panel, "ctrl+shift+w");
    }

    #[test]
    fn loads_overrides_from_settings() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(&directory.path().join("state.sqlite")).unwrap();
        store
            .set_setting(SETTING_KEY, r#"{"quit":"ctrl+alt+q"}"#)
            .unwrap();

        let keymap = Keymap::load(&store);

        assert_eq!(keymap.quit, "ctrl+alt+q");
        assert_eq!(keymap.new_tab, "ctrl+shift+t");
    }
}
