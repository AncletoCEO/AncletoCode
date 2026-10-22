use ancleto_code_core::{Result, Store};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const SETTING_PREFIX: &str = "terminal.layout.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaneKind {
    Shell,
    Agent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pane {
    pub id: String,
    pub session_id: Option<String>,
    pub kind: PaneKind,
    pub title: String,
    pub zoom: f32,
    pub scrollback_lines: usize,
}

impl Pane {
    pub fn new(kind: PaneKind, title: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            session_id: None,
            kind,
            title: title.into(),
            zoom: 1.0,
            scrollback_lines: 10_000,
        }
    }

    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SplitNode {
    Pane {
        pane: Pane,
    },
    Split {
        axis: Axis,
        ratio: f32,
        first: Box<SplitNode>,
        second: Box<SplitNode>,
    },
}

impl SplitNode {
    pub fn pane(pane: Pane) -> Self {
        Self::Pane { pane }
    }

    pub fn split(&mut self, target: &str, axis: Axis) -> Option<String> {
        match self {
            SplitNode::Pane { pane } if pane.id == target => {
                let new_pane = Pane::new(pane.kind, pane.title.clone());
                let id = new_pane.id.clone();
                let first = Box::new(SplitNode::Pane { pane: pane.clone() });
                let second = Box::new(SplitNode::Pane { pane: new_pane });
                *self = SplitNode::Split {
                    axis,
                    ratio: 0.5,
                    first,
                    second,
                };
                Some(id)
            }
            SplitNode::Pane { .. } => None,
            SplitNode::Split { first, second, .. } => first
                .split(target, axis)
                .or_else(|| second.split(target, axis)),
        }
    }

    pub fn remove_pane(&mut self, target: &str) -> bool {
        match self {
            SplitNode::Pane { pane } => pane.id == target,
            SplitNode::Split { first, second, .. } => {
                if matches!(first.as_ref(), SplitNode::Pane { pane } if pane.id == target) {
                    *self = *second.clone();
                    return true;
                }
                if matches!(second.as_ref(), SplitNode::Pane { pane } if pane.id == target) {
                    *self = *first.clone();
                    return true;
                }
                first.remove_pane(target) || second.remove_pane(target)
            }
        }
    }

    pub fn adjust_ratio(&mut self, target: &str, axis: Axis, delta: f32) -> bool {
        match self {
            SplitNode::Pane { .. } => false,
            SplitNode::Split {
                axis: node_axis,
                ratio,
                first,
                second,
            } => {
                if first.adjust_ratio(target, axis, delta)
                    || second.adjust_ratio(target, axis, delta)
                {
                    return true;
                }
                if *node_axis == axis && (contains(first, target) || contains(second, target)) {
                    *ratio = (*ratio + delta).clamp(0.1, 0.9);
                    return true;
                }
                false
            }
        }
    }

    pub fn set_ratio_for(&mut self, target: &str, ratio: f32) -> bool {
        let ratio = ratio.clamp(0.1, 0.9);
        match self {
            SplitNode::Pane { .. } => false,
            SplitNode::Split {
                first,
                second,
                ratio: current,
                ..
            } => {
                if contains(first, target) || contains(second, target) {
                    if matches!(first.as_ref(), SplitNode::Pane { pane } if pane.id == target)
                        || matches!(second.as_ref(), SplitNode::Pane { pane } if pane.id == target)
                    {
                        *current = ratio;
                        return true;
                    }
                    return first.set_ratio_for(target, ratio)
                        || second.set_ratio_for(target, ratio);
                }
                false
            }
        }
    }

    pub fn contains(&self, pane_id: &str) -> bool {
        contains(self, pane_id)
    }

    pub fn panes(&self) -> Vec<&Pane> {
        let mut panes = Vec::new();
        collect(self, &mut panes);
        panes
    }

    pub fn first_pane(&self) -> &Pane {
        match self {
            SplitNode::Pane { pane } => pane,
            SplitNode::Split { first, .. } => first.first_pane(),
        }
    }
}

fn contains(node: &SplitNode, target: &str) -> bool {
    match node {
        SplitNode::Pane { pane } => pane.id == target,
        SplitNode::Split { first, second, .. } => {
            contains(first, target) || contains(second, target)
        }
    }
}

fn find_pane<'a>(node: &'a SplitNode, id: &str) -> Option<&'a Pane> {
    match node {
        SplitNode::Pane { pane } => (pane.id == id).then_some(pane),
        SplitNode::Split { first, second, .. } => {
            find_pane(first, id).or_else(|| find_pane(second, id))
        }
    }
}

fn find_pane_mut<'a>(node: &'a mut SplitNode, id: &str) -> Option<&'a mut Pane> {
    match node {
        SplitNode::Pane { pane } => (pane.id == id).then_some(pane),
        SplitNode::Split { first, second, .. } => {
            find_pane_mut(first, id).or_else(|| find_pane_mut(second, id))
        }
    }
}

fn collect<'a>(node: &'a SplitNode, panes: &mut Vec<&'a Pane>) {
    match node {
        SplitNode::Pane { pane } => panes.push(pane),
        SplitNode::Split { first, second, .. } => {
            collect(first, panes);
            collect(second, panes);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub id: String,
    pub title: String,
    pub root: SplitNode,
    pub focused_pane: String,
}

impl Tab {
    pub fn new(title: impl Into<String>, pane: Pane) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            focused_pane: pane.id.clone(),
            root: SplitNode::pane(pane),
        }
    }

    pub fn pane_ids(&self) -> Vec<String> {
        self.root
            .panes()
            .into_iter()
            .map(|pane| pane.id.clone())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalLayout {
    pub project_id: String,
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
}

impl TerminalLayout {
    pub fn new(project_id: impl Into<String>, first_pane: Pane) -> Self {
        let tab = Tab::new(first_pane.title.clone(), first_pane);
        Self {
            project_id: project_id.into(),
            tabs: vec![tab],
            active_tab: 0,
        }
    }

    pub fn active(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab)
    }

    pub fn tab(&self, id: &str) -> Option<&Tab> {
        self.tabs.iter().find(|tab| tab.id == id)
    }

    pub fn new_tab(&mut self, title: impl Into<String>, pane: Pane) -> String {
        let tab = Tab::new(title, pane);
        let id = tab.id.clone();
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        id
    }

    pub fn close_tab(&mut self, id: &str) -> bool {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == id) else {
            return false;
        };
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            self.active_tab = 0;
        } else if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        } else if index < self.active_tab {
            self.active_tab -= 1;
        }
        true
    }

    pub fn select_tab(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.active_tab = index;
            true
        } else {
            false
        }
    }

    pub fn split(&mut self, tab_id: &str, pane_id: &str, axis: Axis) -> Option<String> {
        let index = self.tabs.iter().position(|tab| tab.id == tab_id)?;
        let new_pane = self.tabs[index].root.split(pane_id, axis)?;
        self.tabs[index].focused_pane = new_pane.clone();
        Some(new_pane)
    }

    pub fn close_pane(&mut self, tab_id: &str, pane_id: &str) -> bool {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == tab_id) else {
            return false;
        };
        let tab = &mut self.tabs[index];
        let is_root = matches!(&tab.root, SplitNode::Pane { pane } if pane.id == pane_id);
        if is_root {
            return self.close_tab(tab_id);
        }
        if !tab.root.remove_pane(pane_id) {
            return false;
        }
        if tab.focused_pane == pane_id {
            tab.focused_pane = tab.root.first_pane().id.clone();
        }
        true
    }

    pub fn focus_pane(&mut self, tab_id: &str, pane_id: &str) -> bool {
        let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) else {
            return false;
        };
        if !tab.root.contains(pane_id) {
            return false;
        }
        tab.focused_pane = pane_id.to_owned();
        true
    }

    pub fn next_pane(&mut self, tab_id: &str) -> Option<String> {
        let tab = self.tabs.iter_mut().find(|tab| tab.id == tab_id)?;
        let ids = tab.pane_ids();
        if ids.is_empty() {
            return None;
        }
        let position = ids
            .iter()
            .position(|id| id == &tab.focused_pane)
            .unwrap_or(0);
        let next = ids[(position + 1) % ids.len()].clone();
        tab.focused_pane = next.clone();
        Some(next)
    }

    pub fn adjust_split_ratio(
        &mut self,
        tab_id: &str,
        pane_id: &str,
        axis: Axis,
        delta: f32,
    ) -> bool {
        let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) else {
            return false;
        };
        tab.root.adjust_ratio(pane_id, axis, delta)
    }

    pub fn swap_sessions(&mut self, tab_id: &str, first: &str, second: &str) -> bool {
        let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) else {
            return false;
        };
        if first == second || !tab.root.contains(first) || !tab.root.contains(second) {
            return false;
        }
        let Some(source) = find_pane(&tab.root, first).cloned() else {
            return false;
        };
        let Some(target) = find_pane(&tab.root, second).cloned() else {
            return false;
        };
        if let Some(pane) = find_pane_mut(&mut tab.root, first) {
            pane.session_id = target.session_id;
            pane.title = target.title;
            pane.kind = target.kind;
        }
        if let Some(pane) = find_pane_mut(&mut tab.root, second) {
            pane.session_id = source.session_id;
            pane.title = source.title;
            pane.kind = source.kind;
        }
        true
    }

    pub fn resize_split(&mut self, tab_id: &str, pane_id: &str, ratio: f32) -> bool {
        let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) else {
            return false;
        };
        tab.root.set_ratio_for(pane_id, ratio)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|error| ancleto_code_core::CoreError::InvalidInput(error.to_string()))
    }

    pub fn from_json(text: &str) -> Result<Self> {
        serde_json::from_str(text)
            .map_err(|error| ancleto_code_core::CoreError::InvalidInput(error.to_string()))
    }
}

pub fn save_layout(store: &Store, layout: &TerminalLayout) -> Result<()> {
    store.set_setting(
        &format!("{SETTING_PREFIX}{}", layout.project_id),
        &layout.to_json()?,
    )
}

pub fn load_layout(store: &Store, project_id: &str) -> Result<Option<TerminalLayout>> {
    store
        .get_setting(&format!("{SETTING_PREFIX}{project_id}"))?
        .map(|text| TerminalLayout::from_json(&text))
        .transpose()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn layout() -> TerminalLayout {
        TerminalLayout::new(
            "project-1",
            Pane::new(PaneKind::Shell, "Shell").with_session("session-1"),
        )
    }

    #[test]
    fn splits_focuses_and_closes_panes() {
        let mut layout = layout();
        let tab_id = layout.tabs[0].id.clone();
        let pane_id = layout.tabs[0].focused_pane.clone();

        let right = layout.split(&tab_id, &pane_id, Axis::Horizontal).unwrap();
        assert_eq!(layout.tabs[0].root.panes().len(), 2);
        assert_eq!(layout.tabs[0].focused_pane, right);

        let down = layout.split(&tab_id, &right, Axis::Vertical).unwrap();
        assert_eq!(layout.tabs[0].root.panes().len(), 3);

        assert!(layout.focus_pane(&tab_id, &pane_id));
        assert_eq!(layout.tabs[0].focused_pane, pane_id);

        assert!(layout.resize_split(&tab_id, &right, 0.7));
        assert!(layout.close_pane(&tab_id, &down));
        assert_eq!(layout.tabs[0].root.panes().len(), 2);
    }

    #[test]
    fn closes_the_last_pane_with_its_tab() {
        let mut layout = layout();
        let tab_id = layout.tabs[0].id.clone();
        let pane_id = layout.tabs[0].focused_pane.clone();

        assert!(layout.close_pane(&tab_id, &pane_id));
        assert!(layout.tabs.is_empty());
    }

    #[test]
    fn cycles_pane_focus() {
        let mut layout = layout();
        let tab_id = layout.tabs[0].id.clone();
        let first = layout.tabs[0].focused_pane.clone();
        layout.split(&tab_id, &first, Axis::Horizontal).unwrap();
        let current = layout.tabs[0].focused_pane.clone();

        let next = layout.next_pane(&tab_id).unwrap();
        assert_ne!(next, current);
        let back = layout.next_pane(&tab_id).unwrap();
        assert_eq!(back, current);
    }

    #[test]
    fn adjusts_split_ratios_and_swaps_sessions() {
        let mut layout = layout();
        let tab_id = layout.tabs[0].id.clone();
        let first = layout.tabs[0].focused_pane.clone();
        let right = layout.split(&tab_id, &first, Axis::Horizontal).unwrap();

        assert!(layout.adjust_split_ratio(&tab_id, &right, Axis::Horizontal, 0.1));
        let initial = find_pane(&layout.tabs[0].root, &first)
            .unwrap()
            .session_id
            .clone();

        assert!(layout.swap_sessions(&tab_id, &first, &right));
        let after = find_pane(&layout.tabs[0].root, &first)
            .unwrap()
            .session_id
            .clone();
        assert_ne!(after, initial);
    }

    #[test]
    fn persists_and_restores_the_layout_per_project() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(&directory.path().join("state.sqlite")).unwrap();
        let mut layout = layout();
        let tab_id = layout.tabs[0].id.clone();
        let pane = layout.tabs[0].focused_pane.clone();
        layout.split(&tab_id, &pane, Axis::Horizontal).unwrap();

        save_layout(&store, &layout).unwrap();
        let restored = load_layout(&store, "project-1").unwrap().unwrap();

        assert_eq!(restored, layout);
        assert!(load_layout(&store, "other").unwrap().is_none());
    }

    #[test]
    fn serializes_to_json() {
        let layout = layout();
        let json = layout.to_json().unwrap();
        let parsed = TerminalLayout::from_json(&json).unwrap();
        assert_eq!(parsed, layout);
    }
}
