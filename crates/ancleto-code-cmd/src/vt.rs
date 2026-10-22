use std::sync::{Arc, Mutex};

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::vte::ansi::{Color, NamedColor, Processor};
use serde::{Deserialize, Serialize};

struct TermSize {
    columns: usize,
    lines: usize,
}

impl Dimensions for TermSize {
    fn total_lines(&self) -> usize {
        self.lines
    }

    fn screen_lines(&self) -> usize {
        self.lines
    }

    fn columns(&self) -> usize {
        self.columns
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CellColor {
    Named(String),
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellSnapshot {
    pub text: String,
    pub foreground: CellColor,
    pub background: CellColor,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
    pub hidden: bool,
    pub wide: bool,
    pub hyperlink: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineSnapshot {
    pub line: i32,
    pub text: String,
    pub cells: Vec<CellSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorSnapshot {
    pub line: i32,
    pub column: usize,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSnapshot {
    pub columns: usize,
    pub lines: usize,
    pub rows: Vec<LineSnapshot>,
    pub cursor: CursorSnapshot,
    pub display_offset: usize,
    pub bracketed_paste: bool,
    pub mouse_reporting: bool,
    pub sgr_mouse: bool,
    pub alternate_screen: bool,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    pub start_line: i32,
    pub start_column: usize,
    pub end_line: i32,
    pub end_column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellMarkKind {
    PromptStart,
    CommandStart,
    CommandExecuted,
    CommandFinished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellMark {
    pub kind: ShellMarkKind,
    pub line: i32,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UiEvent {
    Title(String),
    Bell,
    ClipboardStore(String),
    PtyWrite(String),
    Exit,
}

struct Proxy {
    events: Arc<Mutex<Vec<UiEvent>>>,
}

impl EventListener for Proxy {
    fn send_event(&self, event: Event) {
        let mapped = match event {
            Event::Title(title) => UiEvent::Title(sanitize_title(&title)),
            Event::ResetTitle => UiEvent::Title("AncletoCMD".to_owned()),
            Event::Bell => UiEvent::Bell,
            Event::ClipboardStore(_, text) => UiEvent::ClipboardStore(text),
            Event::PtyWrite(text) => UiEvent::PtyWrite(text),
            Event::Exit => UiEvent::Exit,
            _ => return,
        };
        if let Ok(mut events) = self.events.lock() {
            events.push(mapped);
        }
    }
}

pub struct TerminalState {
    term: Term<Proxy>,
    processor: Processor,
    columns: usize,
    lines: usize,
    events: Arc<Mutex<Vec<UiEvent>>>,
    title: String,
    marks: Vec<ShellMark>,
    osc_buffer: Vec<u8>,
    revision: u64,
}

impl TerminalState {
    pub fn new(columns: usize, lines: usize) -> Self {
        let columns = columns.max(1);
        let lines = lines.max(1);
        let events = Arc::new(Mutex::new(Vec::new()));
        let config = Config {
            scrolling_history: 10_000,
            ..Config::default()
        };
        let proxy = Proxy {
            events: Arc::clone(&events),
        };
        let term = Term::new(config, &TermSize { columns, lines }, proxy);

        Self {
            term,
            processor: Processor::new(),
            columns,
            lines,
            events,
            title: "AncletoCMD".to_owned(),
            marks: Vec::new(),
            osc_buffer: Vec::new(),
            revision: 0,
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        self.processor.advance(&mut self.term, bytes);
        self.revision = self.revision.wrapping_add(1);
        self.scan_shell_marks(bytes);
        if let Ok(mut events) = self.events.lock() {
            for event in events.iter() {
                if let UiEvent::Title(title) = event {
                    self.title = title.clone();
                }
            }
            events.retain(|event| !matches!(event, UiEvent::Title(_)));
        }
    }

    pub fn resize(&mut self, columns: usize, lines: usize) {
        self.columns = columns.max(1);
        self.lines = lines.max(1);
        self.term.resize(TermSize {
            columns: self.columns,
            lines: self.lines,
        });
    }

    pub fn columns(&self) -> usize {
        self.columns
    }

    pub fn lines(&self) -> usize {
        self.lines
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn snapshot(&self) -> TerminalSnapshot {
        let mode = *self.term.mode();
        let content = self.term.renderable_content();
        let mut rows: Vec<LineSnapshot> = Vec::new();

        for indexed in content.display_iter {
            let line = indexed.point.line.0;
            let cell = indexed.cell;
            let snapshot = cell_snapshot(cell);

            if rows.last().map(|row| row.line) != Some(line) {
                rows.push(LineSnapshot {
                    line,
                    text: String::new(),
                    cells: Vec::new(),
                });
            }
            let row = rows.last_mut().expect("fila recién creada");
            if !cell
                .flags
                .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER)
            {
                row.text.push_str(&snapshot.text);
            }
            row.cells.push(snapshot);
        }

        for row in &mut rows {
            row.text = row.text.trim_end().to_owned();
        }

        let cursor = self.term.grid().cursor.point;
        TerminalSnapshot {
            columns: self.columns,
            lines: self.lines,
            rows,
            cursor: CursorSnapshot {
                line: cursor.line.0,
                column: cursor.column.0,
                visible: mode.contains(TermMode::SHOW_CURSOR),
            },
            display_offset: content.display_offset,
            bracketed_paste: mode.contains(TermMode::BRACKETED_PASTE),
            mouse_reporting: mode.contains(TermMode::MOUSE_REPORT_CLICK)
                || mode.contains(TermMode::MOUSE_MOTION),
            sgr_mouse: mode.contains(TermMode::SGR_MOUSE),
            alternate_screen: mode.contains(TermMode::ALT_SCREEN),
            title: self.title.clone(),
        }
    }

    pub fn scroll(&mut self, delta: i32) {
        self.term.scroll_display(Scroll::Delta(delta));
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn scroll_to(&mut self, position: Scroll) {
        self.term.scroll_display(position);
    }

    pub fn full_text(&self) -> Vec<(i32, String)> {
        let mut lines = Vec::new();
        let grid = self.term.grid();
        let top = grid.topmost_line().0;
        let bottom = grid.bottommost_line().0;
        for line in top..=bottom {
            let text = self.line_text(line);
            if !text.is_empty() {
                lines.push((line, text));
            }
        }
        lines
    }

    pub fn search(&self, query: &str, case_sensitive: bool) -> Vec<(i32, usize)> {
        let needle = if case_sensitive {
            query.to_owned()
        } else {
            query.to_lowercase()
        };
        if needle.is_empty() {
            return Vec::new();
        }

        let mut matches = Vec::new();
        for (line, text) in self.full_text() {
            let haystack = if case_sensitive {
                text.clone()
            } else {
                text.to_lowercase()
            };
            let mut offset = 0;
            while let Some(found) = haystack[offset..].find(&needle) {
                let column = haystack[..offset + found].chars().count();
                matches.push((line, column));
                offset += found + needle.len().max(1);
                if offset >= haystack.len() {
                    break;
                }
            }
        }
        matches
    }

    pub fn incremental_search(
        &mut self,
        query: &str,
        from_line: i32,
        case_sensitive: bool,
    ) -> Option<(i32, usize)> {
        let matches = self.search(query, case_sensitive);
        let found = *matches
            .iter()
            .find(|(line, _)| *line >= from_line)
            .or_else(|| matches.first())?;
        let bottom = self.term.grid().bottommost_line().0;
        self.scroll(bottom - found.0);
        Some(found)
    }

    pub fn selection_text(&self, selection: &Selection) -> String {
        let (start_line, end_line) = if selection.start_line <= selection.end_line {
            (selection.start_line, selection.end_line)
        } else {
            (selection.end_line, selection.start_line)
        };
        let mut out = String::new();
        for line in start_line..=end_line {
            let text = self.line_text(line);
            let chars: Vec<char> = text.chars().collect();
            let start = if line == start_line {
                selection.start_column.min(chars.len())
            } else {
                0
            };
            let end = if line == end_line {
                (selection.end_column + 1).min(chars.len())
            } else {
                chars.len()
            };
            if start < end {
                out.extend(&chars[start..end]);
            }
            if line < end_line {
                out.push('\n');
            }
        }
        out
    }

    pub fn hyperlink_at(&self, line: i32, column: usize) -> Option<String> {
        let grid = self.term.grid();
        if line < grid.topmost_line().0 || line > grid.bottommost_line().0 || column >= self.columns
        {
            return None;
        }
        let cell = &grid[Line(line)][Column(column)];
        cell.hyperlink().map(|link| link.uri().to_owned())
    }

    pub fn take_events(&mut self) -> Vec<UiEvent> {
        self.events
            .lock()
            .map(|mut events| std::mem::take(&mut *events))
            .unwrap_or_default()
    }

    pub fn marks(&self) -> &[ShellMark] {
        &self.marks
    }

    pub fn previous_command_line(&self, from: i32) -> Option<i32> {
        self.marks
            .iter()
            .rev()
            .find(|mark| mark.kind == ShellMarkKind::CommandFinished && mark.line < from)
            .map(|mark| mark.line)
    }

    pub fn next_command_line(&self, from: i32) -> Option<i32> {
        self.marks
            .iter()
            .find(|mark| mark.kind == ShellMarkKind::CommandStart && mark.line > from)
            .map(|mark| mark.line)
    }

    pub fn last_exit_code(&self) -> Option<i32> {
        self.marks
            .iter()
            .rev()
            .find(|mark| mark.kind == ShellMarkKind::CommandFinished)
            .and_then(|mark| mark.exit_code)
    }

    pub fn bracketed_paste(&self, text: &str) -> Vec<u8> {
        if self.term.mode().contains(TermMode::BRACKETED_PASTE) {
            let mut bytes = Vec::with_capacity(text.len() + 12);
            bytes.extend_from_slice(b"\x1b[200~");
            bytes.extend_from_slice(text.as_bytes());
            bytes.extend_from_slice(b"\x1b[201~");
            bytes
        } else {
            text.as_bytes().to_vec()
        }
    }

    pub fn mouse_report(
        &self,
        button: u8,
        column: usize,
        row: usize,
        pressed: bool,
        modifiers: u8,
    ) -> Option<Vec<u8>> {
        let mode = self.term.mode();
        if !mode.intersects(TermMode::MOUSE_REPORT_CLICK | TermMode::MOUSE_MOTION) {
            return None;
        }

        let x = column.saturating_add(1) as u32;
        let y = row.saturating_add(1) as u32;
        if mode.contains(TermMode::SGR_MOUSE) {
            let code = button + modifiers + u8::from(!pressed);
            Some(format!("\x1b[<{code};{x};{y}{}", if pressed { 'M' } else { 'm' }).into_bytes())
        } else {
            let code = button + modifiers + if pressed { 0 } else { 3 };
            Some(vec![
                0x1b,
                b'[',
                b'M',
                32 + code,
                32 + u8::try_from(x).unwrap_or(1),
                32 + u8::try_from(y).unwrap_or(1),
            ])
        }
    }

    fn line_text(&self, line: i32) -> String {
        let grid = self.term.grid();
        if line < grid.topmost_line().0 || line > grid.bottommost_line().0 {
            return String::new();
        }
        let row = &grid[Line(line)];
        let mut text = String::new();
        for column in 0..self.columns {
            let cell = &row[Column(column)];
            if cell
                .flags
                .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER)
            {
                continue;
            }
            text.push(cell.c);
            if let Some(zerowidth) = cell.zerowidth() {
                text.extend(zerowidth);
            }
        }
        text.trim_end().to_owned()
    }

    fn scan_shell_marks(&mut self, bytes: &[u8]) {
        self.osc_buffer.extend_from_slice(bytes);
        const PREFIX: &[u8] = b"\x1b]133;";

        loop {
            let Some(start) = find(&self.osc_buffer, PREFIX) else {
                if self.osc_buffer.len() > 4096 {
                    self.osc_buffer.clear();
                }
                return;
            };
            let rest = &self.osc_buffer[start + PREFIX.len()..];
            let Some(end) = rest
                .iter()
                .position(|byte| *byte == 0x07 || (*byte == 0x1b))
            else {
                self.osc_buffer.drain(..start);
                return;
            };
            let payload = String::from_utf8_lossy(&rest[..end]).into_owned();
            let consumed = start + PREFIX.len() + end + 1;
            self.osc_buffer.drain(..consumed);

            let line = self.term.grid().cursor.point.line.0;
            let (kind, exit_code) = match payload.as_bytes().first().copied() {
                Some(b'A') => (Some(ShellMarkKind::PromptStart), None),
                Some(b'B') => (Some(ShellMarkKind::CommandStart), None),
                Some(b'C') => (Some(ShellMarkKind::CommandExecuted), None),
                Some(b'D') => {
                    let code = payload
                        .strip_prefix("D;")
                        .and_then(|value| value.parse().ok());
                    (Some(ShellMarkKind::CommandFinished), code)
                }
                _ => (None, None),
            };
            if let Some(kind) = kind {
                self.marks.push(ShellMark {
                    kind,
                    line,
                    exit_code,
                });
                if self.marks.len() > 2000 {
                    self.marks.drain(..500);
                }
            }
        }
    }
}

fn cell_snapshot(cell: &Cell) -> CellSnapshot {
    let mut text = String::new();
    if !cell
        .flags
        .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER)
    {
        text.push(cell.c);
    }
    if let Some(zerowidth) = cell.zerowidth() {
        text.extend(zerowidth);
    }

    CellSnapshot {
        text,
        foreground: color(cell.fg),
        background: color(cell.bg),
        bold: cell.flags.contains(Flags::BOLD),
        italic: cell.flags.contains(Flags::ITALIC),
        underline: cell.flags.intersects(Flags::ALL_UNDERLINES),
        inverse: cell.flags.contains(Flags::INVERSE),
        hidden: cell.flags.contains(Flags::HIDDEN),
        wide: cell.flags.contains(Flags::WIDE_CHAR),
        hyperlink: cell.hyperlink().map(|link| link.uri().to_owned()),
    }
}

fn color(color: Color) -> CellColor {
    match color {
        Color::Named(named) => CellColor::Named(named_name(named).to_owned()),
        Color::Indexed(index) => CellColor::Indexed(index),
        Color::Spec(rgb) => CellColor::Rgb(rgb.r, rgb.g, rgb.b),
    }
}

fn named_name(color: NamedColor) -> &'static str {
    match color {
        NamedColor::Black => "black",
        NamedColor::Red => "red",
        NamedColor::Green => "green",
        NamedColor::Yellow => "yellow",
        NamedColor::Blue => "blue",
        NamedColor::Magenta => "magenta",
        NamedColor::Cyan => "cyan",
        NamedColor::White => "white",
        NamedColor::BrightBlack => "brightBlack",
        NamedColor::BrightRed => "brightRed",
        NamedColor::BrightGreen => "brightGreen",
        NamedColor::BrightYellow => "brightYellow",
        NamedColor::BrightBlue => "brightBlue",
        NamedColor::BrightMagenta => "brightMagenta",
        NamedColor::BrightCyan => "brightCyan",
        NamedColor::BrightWhite => "brightWhite",
        NamedColor::Foreground => "foreground",
        NamedColor::Background => "background",
        NamedColor::Cursor => "cursor",
        NamedColor::DimBlack => "dimBlack",
        NamedColor::DimRed => "dimRed",
        NamedColor::DimGreen => "dimGreen",
        NamedColor::DimYellow => "dimYellow",
        NamedColor::DimBlue => "dimBlue",
        NamedColor::DimMagenta => "dimMagenta",
        NamedColor::DimCyan => "dimCyan",
        NamedColor::DimWhite => "dimWhite",
        NamedColor::BrightForeground => "brightForeground",
        NamedColor::DimForeground => "dimForeground",
    }
}

fn sanitize_title(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .filter(|ch| !ch.is_control())
        .take(200)
        .collect();
    if cleaned.trim().is_empty() {
        "AncletoCMD".to_owned()
    } else {
        cleaned
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn state(columns: usize, lines: usize) -> TerminalState {
        TerminalState::new(columns, lines)
    }

    fn row_text(state: &TerminalState, line: i32) -> String {
        state
            .snapshot()
            .rows
            .into_iter()
            .find(|row| row.line == line)
            .map(|row| row.text)
            .unwrap_or_default()
    }

    #[test]
    fn renders_plain_text_and_tracks_the_cursor() {
        let mut terminal = state(20, 4);
        terminal.feed(b"hola\r\nmundo");

        assert_eq!(row_text(&terminal, 0), "hola");
        assert_eq!(row_text(&terminal, 1), "mundo");
        let snapshot = terminal.snapshot();
        assert_eq!(snapshot.cursor.line, 1);
        assert_eq!(snapshot.cursor.column, 5);
        assert!(snapshot.cursor.visible);
    }

    #[test]
    fn enters_and_leaves_the_alternate_screen() {
        let mut terminal = state(20, 3);
        terminal.feed(b"principal");
        terminal.feed(b"\x1b[?1049h\x1b[H");
        assert!(terminal.snapshot().alternate_screen);
        terminal.feed(b"alterna");
        assert_eq!(row_text(&terminal, 0), "alterna");
        terminal.feed(b"\x1b[?1049l");
        assert!(!terminal.snapshot().alternate_screen);
        assert!(row_text(&terminal, 0).contains("principal"));
    }

    #[test]
    fn tracks_bracketed_paste_and_mouse_modes() {
        let mut terminal = state(40, 10);
        assert!(!terminal.snapshot().bracketed_paste);

        terminal.feed(b"\x1b[?2004h\x1b[?1000h\x1b[?1006h");
        let snapshot = terminal.snapshot();
        assert!(snapshot.bracketed_paste);
        assert!(snapshot.mouse_reporting);
        assert!(snapshot.sgr_mouse);

        let pasted = terminal.bracketed_paste("hola");
        assert_eq!(pasted, b"\x1b[200~hola\x1b[201~");

        let report = terminal.mouse_report(0, 4, 2, true, 0).unwrap();
        assert_eq!(report, b"\x1b[<0;5;3M");

        terminal.feed(b"\x1b[?2004l\x1b[?1000l\x1b[?1006l");
        assert_eq!(terminal.bracketed_paste("hola"), b"hola");
        assert!(terminal.mouse_report(0, 1, 1, true, 0).is_none());
    }

    #[test]
    fn preserves_wide_chars_emoji_and_combining_marks() {
        let mut terminal = state(20, 3);
        terminal.feed("界🦀e\u{301}".as_bytes());

        let text = row_text(&terminal, 0);
        assert!(text.starts_with("界🦀e"));
        assert_eq!(text.chars().count(), 4);
        let snapshot = terminal.snapshot();
        let row = snapshot.rows.iter().find(|row| row.line == 0).unwrap();
        assert!(row.cells.iter().any(|cell| cell.wide));
        assert!(row.cells.iter().any(|cell| cell.text.contains('\u{301}')));
    }

    #[test]
    fn reflows_on_resize() {
        let mut terminal = state(10, 4);
        terminal.feed(b"abcdefghijklmnopqrst");
        terminal.resize(20, 4);

        let reconstructed = terminal
            .full_text()
            .into_iter()
            .map(|(_, text)| text)
            .collect::<String>();
        assert!(reconstructed.contains("abcdefghij"));
        assert!(reconstructed.contains("klmnopqrst"));
    }

    #[test]
    fn searches_scrollback_and_selects_text() {
        let mut terminal = state(20, 3);
        terminal.feed(b"uno\r\ndos\r\ntres\r\ncuatro\r\ncinco");

        let matches = terminal.search("dos", false);
        assert!(matches.iter().any(|(_, column)| *column == 0));

        terminal.feed(b"\r\n");
        terminal.scroll(-10);
        let full = terminal.full_text();
        assert!(full.iter().any(|(_, text)| text == "dos"));

        let mut simple = state(20, 3);
        simple.feed(b"uno");
        let selection = Selection {
            start_line: 0,
            start_column: 0,
            end_line: 0,
            end_column: 2,
        };
        assert_eq!(simple.selection_text(&selection), "uno");
    }

    #[test]
    fn exposes_clipboard_title_hyperlink_and_bell_events() {
        let mut terminal = state(30, 3);
        terminal.feed(b"\x1b]0;Mi \x01titulo\x07");
        assert_eq!(terminal.title(), "Mi titulo");

        terminal.feed(b"\x07");
        terminal.feed(b"\x1b]52;c;aG9sYQ==\x07");
        let events = terminal.take_events();
        assert!(events.iter().any(|event| matches!(event, UiEvent::Bell)));
        assert!(events
            .iter()
            .any(|event| matches!(event, UiEvent::ClipboardStore(text) if text == "hola")));

        terminal.feed(b"\x1b]8;;https://example.com\x1b\\link\x1b]8;;\x1b\\");
        assert_eq!(
            terminal.hyperlink_at(0, 0).as_deref(),
            Some("https://example.com")
        );
    }

    #[test]
    fn parses_osc_133_marks_across_chunks() {
        let mut terminal = state(20, 4);
        terminal.feed(b"\x1b]133;A\x07\x1b]13");
        terminal.feed(b"3;B\x07prompt\r\n");
        terminal.feed(b"\x1b]133;D;42\x07");

        let marks = terminal.marks();
        assert_eq!(marks[0].kind, ShellMarkKind::PromptStart);
        assert_eq!(marks[1].kind, ShellMarkKind::CommandStart);
        assert_eq!(marks[2].kind, ShellMarkKind::CommandFinished);
        assert_eq!(terminal.last_exit_code(), Some(42));
    }
}
