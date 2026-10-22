use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use ancleto_code_core::host::HostPaths;
use ancleto_code_core::{HostClient, ProjectService, SessionBackend, SessionSpec, Store};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowId};

use crate::layout::{load_layout, save_layout, Axis, Pane, PaneKind, SplitNode, TerminalLayout};
use crate::profiles::{default_profiles, ShellProfile};
use crate::render::{rgba, Rect, RectRenderer};
use crate::session::TerminalSession;
use crate::theme::{self, Theme};
use crate::vt::{Selection, UiEvent};

const TITLE_BAR: f32 = 28.0;
const STATUS_BAR: f32 = 20.0;
const PADDING: f32 = 3.0;
const MIN_COLS: usize = 10;
const MIN_ROWS: usize = 3;

#[derive(Debug, Clone, Default)]
pub struct CmdOptions {
    pub project: Option<String>,
    pub shell: Option<String>,
    pub theme: Option<String>,
    pub kill_on_close: bool,
}

#[derive(Debug, Clone)]
enum UserEvent {
    Wakeup,
}

struct PaneRender {
    buffer: Buffer,
    revision: u64,
    columns: usize,
    lines: usize,
    selection: Option<Selection>,
}

struct PaneLayout {
    pane_id: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    focused: bool,
}

enum Modal {
    KillAll,
    ClosePane(String),
    Clipboard(String),
    Hyperlink(String),
}

pub fn run(options: CmdOptions) -> Result<(), String> {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .map_err(|error| error.to_string())?;
    let proxy = event_loop.create_proxy();
    let mut application = Application {
        options,
        state: None,
        proxy,
    };
    event_loop
        .run_app(&mut application)
        .map_err(|error| error.to_string())
}

struct Application {
    options: CmdOptions,
    state: Option<AppState>,
    proxy: EventLoopProxy<UserEvent>,
}

impl ApplicationHandler<UserEvent> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("AncletoCMD")
            .with_inner_size(LogicalSize::new(1280.0, 760.0))
            .with_min_inner_size(LogicalSize::new(640.0, 400.0));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                eprintln!("acmd: no se pudo crear la ventana: {error}");
                event_loop.exit();
                return;
            }
        };

        match AppState::new(
            Arc::clone(&window),
            event_loop,
            &self.options,
            self.proxy.clone(),
        ) {
            Ok(state) => self.state = Some(state),
            Err(error) => {
                eprintln!("acmd: {error}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        state.handle_event(event_loop, event);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        if let Some(state) = self.state.as_mut() {
            let UserEvent::Wakeup = event;
            state.window.request_redraw();
        }
    }
}

struct AppState {
    window: Arc<Window>,
    _instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    rects: RectRenderer,

    font_system: FontSystem,
    swash_cache: SwashCache,
    _cache: Cache,
    viewport: Viewport,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
    chrome_buffer: Buffer,
    status_buffer: Buffer,
    modal_buffer: Buffer,

    store: Store,
    client: Arc<HostClient>,
    project_id: String,
    project_root: String,
    profile: ShellProfile,
    theme: Theme,

    layout: TerminalLayout,
    sessions: HashMap<String, TerminalSession>,
    renders: HashMap<String, PaneRender>,
    panes: Vec<PaneLayout>,

    font_size: f32,
    cell_width: f32,
    cell_height: f32,
    scale_factor: f64,
    modifiers: ModifiersState,
    dragging: Option<(String, i32, usize)>,
    last_click: Option<(std::time::Instant, i32, usize, u8)>,
    kill_on_close: bool,
    focused: bool,
    activity: std::collections::HashSet<String>,
    marks_seen: HashMap<String, usize>,
    cursor: PhysicalPosition<f64>,
    modal: Option<Modal>,
    status: Option<String>,
    proxy: EventLoopProxy<UserEvent>,
}

impl AppState {
    fn new(
        window: Arc<Window>,
        event_loop: &ActiveEventLoop,
        options: &CmdOptions,
        proxy: EventLoopProxy<UserEvent>,
    ) -> Result<Self, String> {
        let size = window.inner_size();
        let scale_factor = window.scale_factor();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(event_loop.owned_display_handle()),
        ));
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
            ..Default::default()
        }))
        .or_else(|_| {
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: true,
                compatible_surface: None,
                ..Default::default()
            }))
        })
        .map_err(|error| format!("no hay adaptador gráfico utilizable: {error}"))?;

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .map_err(|error| format!("no se pudo crear el dispositivo GPU: {error}"))?;

        let surface = instance
            .create_surface(Arc::clone(&window))
            .map_err(|error| format!("no se pudo crear la superficie: {error}"))?;
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };
        surface.configure(&device, &surface_config);

        let mut font_system = FontSystem::new();
        let (cell_width, cell_height) = measure_cell(&mut font_system, 13.0);
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, surface_config.format);
        let text_renderer =
            TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);
        let scale = scale_factor as f32;
        let chrome_buffer = Buffer::new(
            &mut font_system,
            Metrics::new(13.0 * scale, cell_height * scale),
        );
        let status_buffer = Buffer::new(&mut font_system, Metrics::new(11.0 * scale, 16.0 * scale));
        let modal_buffer = Buffer::new(&mut font_system, Metrics::new(14.0 * scale, 20.0 * scale));

        let store = Store::open(&ancleto_code_core::host::transport::default_database_path())
            .map_err(|error| format!("no se pudo abrir la base: {error}"))?;
        let (project_id, project_root) = resolve_project(options.project.as_deref())?;
        let profile = resolve_profile(options.shell.as_deref())?;
        let theme = theme::load(&store, options.theme.as_deref());

        let paths = HostPaths::discover(None);
        let client = Arc::new(
            HostClient::connect_or_start(
                &paths,
                &ancleto_code_core::host::transport::default_database_path(),
            )
            .map_err(|error| format!("no se pudo conectar al host: {error}"))?,
        );

        let rects = RectRenderer::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb);
        let mut state = Self {
            window,
            _instance: instance,
            surface,
            device,
            queue,
            surface_config,
            rects,
            font_system,
            swash_cache,
            _cache: cache,
            viewport,
            atlas,
            text_renderer,
            chrome_buffer,
            status_buffer,
            modal_buffer,
            store,
            client,
            project_id,
            project_root,
            profile,
            theme,
            layout: TerminalLayout::new("placeholder", Pane::new(PaneKind::Shell, "Shell")),
            sessions: HashMap::new(),
            renders: HashMap::new(),
            panes: Vec::new(),
            font_size: 13.0,
            cell_width,
            cell_height,
            scale_factor,
            modifiers: ModifiersState::empty(),
            dragging: None,
            last_click: None,
            kill_on_close: options.kill_on_close,
            focused: true,
            activity: std::collections::HashSet::new(),
            marks_seen: HashMap::new(),
            cursor: PhysicalPosition::new(0.0, 0.0),
            modal: None,
            status: None,
            proxy,
        };

        let restored = load_layout(&state.store, &state.project_id)
            .ok()
            .flatten()
            .filter(|layout| !layout.tabs.is_empty());
        if let Some(layout) = restored {
            state.layout = layout;
            state.ensure_sessions()?;
        } else {
            let pane = state.create_session()?;
            state.layout = TerminalLayout::new(state.project_id.clone(), pane);
        }

        state.update_chrome();
        Ok(state)
    }

    fn create_session(&mut self) -> Result<Pane, String> {
        let columns = 80;
        let lines = 24;
        let spec = SessionSpec {
            project_id: self.project_id.clone(),
            driver_id: "shell".to_owned(),
            program: self.profile.program.clone(),
            args: self.profile.args.clone(),
            cwd: self.project_root.clone(),
            env: self.profile.env.clone(),
            cols: columns as u16,
            rows: lines as u16,
        };
        let info = self
            .client
            .create_session(spec)
            .map_err(|error| error.to_string())?;
        let proxy = self.proxy.clone();
        let wake: crate::session::Wake = Arc::new(move || {
            let _ = proxy.send_event(UserEvent::Wakeup);
        });
        let session = TerminalSession::attach_with_wake(
            Arc::clone(&self.client),
            info.id.clone(),
            columns,
            lines,
            Some(wake),
        )
        .map_err(|error| error.to_string())?;

        let pane = Pane::new(PaneKind::Shell, self.profile.display_name.clone())
            .with_session(info.id.clone());
        self.sessions.insert(pane.id.clone(), session);
        Ok(pane)
    }

    fn ensure_sessions(&mut self) -> Result<(), String> {
        let saved = self
            .client
            .saved_sessions()
            .unwrap_or_default()
            .into_iter()
            .filter(|session| {
                session.status != ancleto_code_core::SessionStatus::Crashed
                    && session.status != ancleto_code_core::SessionStatus::Stopped
            })
            .map(|session| session.id)
            .collect::<std::collections::HashSet<_>>();
        let mut missing = Vec::new();
        for tab in &mut self.layout.tabs {
            collect_panes_mut(&mut tab.root, &mut |pane| {
                let alive = pane
                    .session_id
                    .as_ref()
                    .is_some_and(|id| saved.contains(id));
                if !alive {
                    missing.push(pane.id.clone());
                }
            });
        }

        for pane_id in missing {
            let replacement = self.create_session()?;
            let session_id = replacement.session_id.clone();
            self.sessions.remove(&replacement.id);
            for tab in &mut self.layout.tabs {
                if let Some(pane) = find_pane_mut(&mut tab.root, &pane_id) {
                    pane.session_id = session_id.clone();
                }
            }
        }

        let panes: Vec<(String, Option<String>)> = self
            .layout
            .tabs
            .iter()
            .flat_map(|tab| tab.root.panes())
            .map(|pane| (pane.id.clone(), pane.session_id.clone()))
            .collect();
        for (pane_id, session_id) in panes {
            let Some(session_id) = session_id else {
                continue;
            };
            if self.sessions.contains_key(&pane_id) {
                continue;
            }
            let proxy = self.proxy.clone();
            let wake: crate::session::Wake = Arc::new(move || {
                let _ = proxy.send_event(UserEvent::Wakeup);
            });
            match TerminalSession::attach_with_wake(
                Arc::clone(&self.client),
                session_id,
                80,
                24,
                Some(wake),
            ) {
                Ok(session) => {
                    self.sessions.insert(pane_id, session);
                }
                Err(error) => {
                    self.status = Some(format!("no se pudo reattach: {error}"));
                }
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                if self.kill_on_close {
                    self.modal = Some(Modal::KillAll);
                    self.update_chrome();
                    self.window.request_redraw();
                } else {
                    self.shutdown(event_loop, false);
                }
            }
            WindowEvent::Resized(size) => {
                self.resize_surface(size);
                self.window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
                self.recompute_metrics();
                self.window.request_redraw();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::Focused(focused) => {
                self.focused = focused;
                if focused {
                    self.activity.clear();
                }
                self.update_chrome();
                self.window.request_redraw();
            }
            WindowEvent::Ime(Ime::Commit(text)) => {
                self.write_to_focus(text.as_bytes());
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                self.handle_key(event_loop, &event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.handle_mouse_button(state, button);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = position;
                self.handle_cursor_moved(position);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_wheel(delta);
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.redraw() {
                    self.status = Some(error);
                }
            }
            _ => {}
        }
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &winit::event::KeyEvent) {
        if let Some(modal) = &self.modal {
            let key = event
                .logical_key
                .to_text()
                .unwrap_or_default()
                .to_lowercase();
            let confirm = key == "y" || key == "s";
            let cancel = key == "n" || matches!(event.logical_key, Key::Named(NamedKey::Escape));
            match modal {
                Modal::KillAll if confirm => self.shutdown(event_loop, true),
                Modal::KillAll if cancel => self.modal = None,
                Modal::ClosePane(pane_id) if confirm => {
                    let pane_id = pane_id.clone();
                    self.close_pane(event_loop, &pane_id, true);
                }
                Modal::ClosePane(_) if cancel => self.modal = None,
                Modal::Clipboard(text) if confirm => {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(text.clone());
                    }
                    self.modal = None;
                }
                Modal::Clipboard(_) if cancel => self.modal = None,
                Modal::Hyperlink(url) if confirm => {
                    let _ = open::that(url);
                    self.modal = None;
                }
                Modal::Hyperlink(_) if cancel => self.modal = None,
                _ => {}
            }
            self.update_chrome();
            self.window.request_redraw();
            return;
        }

        let control = self.modifiers.control_key();
        let shift = self.modifiers.shift_key();

        if control && shift {
            if let Key::Character(text) = &event.logical_key {
                match text.to_lowercase().as_str() {
                    "t" => {
                        if let Ok(pane) = self.create_session() {
                            self.layout.new_tab(self.profile.display_name.clone(), pane);
                            self.update_layout_geometry();
                        }
                        return;
                    }
                    "w" => {
                        if let Some(pane) = self.focused_pane_id() {
                            self.modal = Some(Modal::ClosePane(pane));
                            self.update_chrome();
                            self.window.request_redraw();
                        }
                        return;
                    }
                    "e" | "d" => {
                        let axis = if text.eq_ignore_ascii_case("e") {
                            Axis::Horizontal
                        } else {
                            Axis::Vertical
                        };
                        self.split_focused(axis);
                        return;
                    }
                    "c" => {
                        self.copy_selection();
                        return;
                    }
                    "v" => {
                        self.paste();
                        return;
                    }
                    "q" => {
                        self.shutdown(event_loop, false);
                        return;
                    }
                    "k" => {
                        self.modal = Some(Modal::KillAll);
                        self.update_chrome();
                        self.window.request_redraw();
                        return;
                    }
                    "+" | "=" => {
                        self.zoom(1.0);
                        return;
                    }
                    "-" => {
                        self.zoom(-1.0);
                        return;
                    }
                    "0" => {
                        self.font_size = 13.0;
                        self.recompute_metrics();
                        return;
                    }
                    _ => {}
                }
            }
        }

        if control && !shift && matches!(event.logical_key, Key::Named(NamedKey::PageUp)) {
            let index = self.layout.active_tab.saturating_sub(1);
            self.layout.select_tab(index);
            if let Some(tab) = self.layout.tabs.get(index) {
                self.activity.remove(&tab.id.clone());
            }
            self.update_layout_geometry();
            self.update_chrome();
            self.window.request_redraw();
            return;
        }
        if control && !shift && matches!(event.logical_key, Key::Named(NamedKey::PageDown)) {
            let index = (self.layout.active_tab + 1).min(self.layout.tabs.len().saturating_sub(1));
            self.layout.select_tab(index);
            if let Some(tab) = self.layout.tabs.get(index) {
                self.activity.remove(&tab.id.clone());
            }
            self.update_layout_geometry();
            self.update_chrome();
            self.window.request_redraw();
            return;
        }
        if control && shift && matches!(event.logical_key, Key::Named(NamedKey::PageUp)) {
            self.scroll_focus(-10);
            return;
        }
        if control && shift && matches!(event.logical_key, Key::Named(NamedKey::PageDown)) {
            self.scroll_focus(10);
            return;
        }
        if matches!(event.logical_key, Key::Named(NamedKey::Tab)) && control {
            if let Some(tab_id) = self.layout.active().map(|tab| tab.id.clone()) {
                self.layout.next_pane(&tab_id);
                self.window.request_redraw();
            }
            return;
        }

        if let Some(bytes) = encode_key(event, self.modifiers) {
            self.write_to_focus(&bytes);
        }
    }

    fn split_focused(&mut self, axis: Axis) {
        let Some(pane_id) = self.focused_pane_id() else {
            return;
        };
        let tab_id = self.active_tab_id();
        let Ok(pane) = self.create_session() else {
            return;
        };
        let Some(new_id) = self.layout.split(&tab_id, &pane_id, axis) else {
            if let Some(session_id) = &pane.session_id {
                let _ = self.client.kill_session(session_id);
            }
            self.sessions.remove(&pane.id);
            self.status = Some("no se pudo dividir el pane".to_owned());
            self.update_chrome();
            self.window.request_redraw();
            return;
        };
        if let Some(target) = find_pane_mut_by_id(&mut self.layout, &new_id) {
            target.session_id = pane.session_id.clone();
            target.title = pane.title.clone();
        }
        if let Some(session) = self.sessions.remove(&pane.id) {
            self.sessions.insert(new_id, session);
        }
        self.update_layout_geometry();
        self.update_chrome();
        self.window.request_redraw();
    }

    fn handle_mouse_button(&mut self, state: ElementState, button: MouseButton) {
        let Some(focused) = self.focused_pane_id() else {
            return;
        };
        let cursor = self.cursor;
        let Some((line, column)) = self.cell_at(&focused, cursor) else {
            return;
        };

        let mode = self
            .sessions
            .get(&focused)
            .and_then(|session| session.snapshot())
            .map(|snapshot| (snapshot.mouse_reporting, snapshot.sgr_mouse))
            .unwrap_or((false, false));

        if mode.0 {
            let button_code = match button {
                MouseButton::Left => 0,
                MouseButton::Middle => 1,
                MouseButton::Right => 2,
                _ => return,
            };
            let pressed = state == ElementState::Pressed;
            if let Some(session) = self.sessions.get(&focused) {
                let modifiers = mouse_modifiers(self.modifiers);
                if let Some(bytes) =
                    session.mouse_report(button_code, column, line as usize, pressed, modifiers)
                {
                    let _ = session.write(&bytes);
                }
            }
            return;
        }

        if button == MouseButton::Left {
            match state {
                ElementState::Pressed => {
                    if self.modifiers.control_key() {
                        if let Some(url) = self
                            .sessions
                            .get(&focused)
                            .and_then(|session| session.hyperlink_at(line, column))
                        {
                            self.modal = Some(Modal::Hyperlink(url));
                            self.update_chrome();
                            self.window.request_redraw();
                            return;
                        }
                    }
                    let now = std::time::Instant::now();
                    let count = match self.last_click {
                        Some((time, last_line, last_column, count))
                            if time.elapsed().as_millis() < 400
                                && last_line == line
                                && last_column == column =>
                        {
                            (count % 3) + 1
                        }
                        _ => 1,
                    };
                    self.last_click = Some((now, line, column, count));
                    self.dragging = Some((focused.clone(), line, column));
                    if let Some(render) = self.renders.get_mut(&focused) {
                        render.selection = Some(match count {
                            2 => word_selection(&self.sessions, &focused, line, column),
                            3 => line_selection(&self.sessions, &focused, line, column),
                            _ => Selection {
                                start_line: line,
                                start_column: column,
                                end_line: line,
                                end_column: column,
                            },
                        });
                    }
                }
                ElementState::Released => {
                    self.dragging = None;
                }
            }
            self.window.request_redraw();
        }
    }

    fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        let Some((pane_id, _, _)) = &self.dragging else {
            return;
        };
        let pane_id = pane_id.clone();
        if let Some((line, column)) = self.cell_at(&pane_id, position) {
            if let Some(render) = self.renders.get_mut(&pane_id) {
                if let Some(selection) = &mut render.selection {
                    selection.end_line = line;
                    selection.end_column = column;
                }
            }
            self.window.request_redraw();
        }
    }

    fn handle_wheel(&mut self, delta: MouseScrollDelta) {
        let amount = match delta {
            MouseScrollDelta::LineDelta(_, y) => y.round() as i32 * 3,
            MouseScrollDelta::PixelDelta(position) => position.y.round() as i32 / 20,
        }
        .clamp(-30, 30);
        if amount != 0 {
            self.scroll_focus(amount);
        }
    }

    fn scroll_focus(&mut self, delta: i32) {
        if let Some(pane_id) = self.focused_pane_id() {
            if let Some(session) = self.sessions.get(&pane_id) {
                session.scroll(delta);
            }
            self.window.request_redraw();
        }
    }

    fn write_to_focus(&mut self, bytes: &[u8]) {
        if let Some(pane_id) = self.focused_pane_id() {
            if let Some(session) = self.sessions.get(&pane_id) {
                let _ = session.write(bytes);
            }
        }
    }

    fn paste(&mut self) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if let Ok(text) = clipboard.get_text() {
                if let Some(pane_id) = self.focused_pane_id() {
                    if let Some(session) = self.sessions.get(&pane_id) {
                        let bytes = session.bracketed_paste(&text);
                        let _ = session.write(&bytes);
                    }
                }
            }
        }
    }

    fn copy_selection(&mut self) {
        let Some(pane_id) = self.focused_pane_id() else {
            return;
        };
        let Some(selection) = self
            .renders
            .get(&pane_id)
            .and_then(|render| render.selection.clone())
        else {
            return;
        };
        if let Some(session) = self.sessions.get(&pane_id) {
            let text = session.selection_text(&selection);
            if !text.is_empty() {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(text);
                }
            }
        }
    }

    fn zoom(&mut self, delta: f32) {
        self.font_size = (self.font_size + delta).clamp(8.0, 32.0);
        self.recompute_metrics();
    }

    fn recompute_metrics(&mut self) {
        let (width, height) = measure_cell(&mut self.font_system, self.font_size);
        self.cell_width = width;
        self.cell_height = height;
        let scale = self.scale_factor as f32;
        self.chrome_buffer
            .set_metrics(Metrics::new(13.0 * scale, self.cell_height * scale));
        self.status_buffer
            .set_metrics(Metrics::new(11.0 * scale, 16.0 * scale));
        self.modal_buffer
            .set_metrics(Metrics::new(14.0 * scale, 20.0 * scale));
        self.renders.clear();
        self.update_layout_geometry();
        self.resize_surface(self.window.inner_size());
        self.window.request_redraw();
    }

    fn resize_surface(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.width = size.width.max(1);
        self.surface_config.height = size.height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    fn focused_pane_id(&self) -> Option<String> {
        self.layout
            .active()
            .map(|tab| tab.focused_pane.clone())
            .or_else(|| self.layout.tabs.first().map(|tab| tab.focused_pane.clone()))
    }

    fn active_tab_id(&self) -> String {
        self.layout
            .active()
            .map(|tab| tab.id.clone())
            .unwrap_or_default()
    }

    fn cell_at(&self, pane_id: &str, position: PhysicalPosition<f64>) -> Option<(i32, usize)> {
        let layout = self.panes.iter().find(|pane| pane.pane_id == pane_id)?;
        let snapshot = self.sessions.get(pane_id)?.snapshot()?;
        let x = position.x as f32 / self.scale_factor as f32 - layout.x - PADDING;
        let y = position.y as f32 / self.scale_factor as f32 - layout.y - PADDING;
        if x < 0.0 || y < 0.0 {
            return None;
        }
        let column = (x / self.cell_width) as usize;
        let row = (y / self.cell_height) as usize;
        let line = snapshot.rows.get(row)?.line;
        Some((line, column.min(snapshot.columns.saturating_sub(1))))
    }

    fn update_layout_geometry(&mut self) {
        let width = self.surface_config.width as f32 / self.scale_factor as f32;
        let height = self.surface_config.height as f32 / self.scale_factor as f32;
        let area = Rect2 {
            x: 0.0,
            y: TITLE_BAR,
            width,
            height: (height - TITLE_BAR - STATUS_BAR).max(1.0),
        };
        let tab = match self.layout.active() {
            Some(tab) => tab,
            None => return,
        };
        let mut panes = Vec::new();
        collect_geometry(&tab.root, area, &tab.focused_pane, &mut panes);
        self.panes = panes;

        for pane in &self.panes {
            let columns = ((pane.width - PADDING * 2.0) / self.cell_width)
                .floor()
                .max(MIN_COLS as f32) as usize;
            let lines = ((pane.height - PADDING * 2.0) / self.cell_height)
                .floor()
                .max(MIN_ROWS as f32) as usize;
            if let Some(session) = self.sessions.get(&pane.pane_id) {
                if session.columns() != columns || session.lines() != lines {
                    let _ = session.resize(columns, lines);
                }
            }
        }
    }

    fn update_chrome(&mut self) {
        let title = self
            .layout
            .tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| {
                let marker = if self.activity.contains(&tab.id) {
                    " ●"
                } else {
                    ""
                };
                if index == self.layout.active_tab {
                    format!("▶ {}{marker}", tab.title)
                } else {
                    format!("  {}{marker}", tab.title)
                }
            })
            .collect::<Vec<_>>()
            .join("   ");
        set_buffer_text(&mut self.font_system, &mut self.chrome_buffer, &title);

        let project = Path::new(&self.project_root)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.project_root.clone());
        let panes = self.panes.len();
        let status = self.status.clone().unwrap_or_else(|| {
            format!(
                "{} · {} panes · {} pt · Ctrl+Shift+T nueva tab · Q salir",
                project, panes, self.font_size
            )
        });
        set_buffer_text(&mut self.font_system, &mut self.status_buffer, &status);
        self.window.set_title(&format!("AncletoCMD — {project}"));

        let modal = match &self.modal {
            Some(Modal::KillAll) => "¿Matar todas las sesiones y salir? [s/n]",
            Some(Modal::ClosePane(_)) => "¿Cerrar el pane y terminar su sesión? [s/n]",
            Some(Modal::Clipboard(_)) => {
                "Un programa quiere escribir el portapapeles (OSC 52). ¿Permitir? [s/n]"
            }
            Some(Modal::Hyperlink(_)) => "¿Abrir el hyperlink OSC 8? [s/n]",
            None => "",
        };
        set_buffer_text(&mut self.font_system, &mut self.modal_buffer, modal);
    }

    fn close_pane(&mut self, event_loop: &ActiveEventLoop, pane_id: &str, kill: bool) {
        let tab_id = self.active_tab_id();
        if let Some(session) = self.sessions.remove(pane_id) {
            if kill {
                let _ = self.client.kill_session(session.session_id());
            }
        }
        self.renders.remove(pane_id);
        self.modal = None;
        if !self.layout.close_pane(&tab_id, pane_id) {
            self.shutdown(event_loop, kill);
            return;
        }
        self.update_layout_geometry();
        self.update_chrome();
        self.window.request_redraw();
    }

    fn shutdown(&mut self, event_loop: &ActiveEventLoop, kill: bool) {
        for tab in &self.layout.tabs {
            for pane in tab.root.panes() {
                if let Some(session_id) = &pane.session_id {
                    if kill {
                        let _ = self.client.kill_session(session_id);
                    }
                }
            }
        }
        let _ = save_layout(&self.store, &self.layout);
        self.sessions.clear();
        event_loop.exit();
    }

    fn redraw(&mut self) -> Result<(), String> {
        self.update_layout_geometry();
        self.update_pane_buffers();
        self.detect_activity();
        self.update_chrome();

        let width = self.surface_config.width as f32;
        let height = self.surface_config.height as f32;
        let scale = self.scale_factor as f32;

        self.viewport.update(
            &self.queue,
            Resolution {
                width: self.surface_config.width,
                height: self.surface_config.height,
            },
        );

        let mut rects = Vec::new();
        let mut text_areas: Vec<PaneText> = Vec::new();

        for pane in &self.panes {
            let x = pane.x * scale;
            let y = pane.y * scale;
            let w = pane.width * scale;
            let h = pane.height * scale;
            rects.push(Rect {
                x,
                y,
                width: w,
                height: h,
                color: rgba(self.theme.background, 1.0),
            });
            if pane.focused {
                rects.push(Rect {
                    x,
                    y,
                    width: w,
                    height: 1.0,
                    color: rgba(self.theme.accent, 0.9),
                });
            }

            if let Some(render) = self.renders.get(&pane.pane_id) {
                if let Some(selection) = &render.selection {
                    if let Some(session) = self.sessions.get(&pane.pane_id) {
                        if let Some(snapshot) = session.snapshot() {
                            for (row_index, row) in snapshot.rows.iter().enumerate() {
                                if row.line < selection.start_line || row.line > selection.end_line
                                {
                                    continue;
                                }
                                let start = if row.line == selection.start_line {
                                    selection.start_column
                                } else {
                                    0
                                };
                                let end = if row.line == selection.end_line {
                                    selection.end_column + 1
                                } else {
                                    snapshot.columns
                                };
                                let left =
                                    x + PADDING * scale + start as f32 * self.cell_width * scale;
                                let width_sel =
                                    end.saturating_sub(start) as f32 * self.cell_width * scale;
                                rects.push(Rect {
                                    x: left,
                                    y: y + PADDING * scale
                                        + row_index as f32 * self.cell_height * scale,
                                    width: width_sel,
                                    height: self.cell_height * scale,
                                    color: rgba(self.theme.selection, 0.6),
                                });
                            }
                            if pane.focused && snapshot.cursor.visible {
                                if let Some(row) = snapshot
                                    .rows
                                    .iter()
                                    .position(|row| row.line == snapshot.cursor.line)
                                {
                                    rects.push(Rect {
                                        x: x + PADDING * scale
                                            + snapshot.cursor.column as f32
                                                * self.cell_width
                                                * scale,
                                        y: y + PADDING * scale
                                            + row as f32 * self.cell_height * scale,
                                        width: self.cell_width * scale,
                                        height: self.cell_height * scale,
                                        color: rgba(self.theme.cursor, 0.7),
                                    });
                                }
                            }
                        }
                    }
                }

                text_areas.push(PaneText {
                    pane_id: pane.pane_id.clone(),
                    left: (pane.x + PADDING) * scale,
                    top: (pane.y + PADDING) * scale,
                    right: ((pane.x + pane.width) * scale) as i32,
                    bottom: ((pane.y + pane.height) * scale) as i32,
                });
            }
        }

        let modal_rects = if self.modal.is_some() {
            vec![
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width,
                    height,
                    color: [0.0, 0.0, 0.0, 0.55],
                },
                Rect {
                    x: (width / 2.0 - 280.0 * scale).max(8.0),
                    y: height / 2.0 - 28.0 * scale,
                    width: 560.0 * scale,
                    height: 56.0 * scale,
                    color: rgba(self.theme.background, 0.96),
                },
            ]
        } else {
            Vec::new()
        };
        rects.extend(modal_rects);
        let chrome_rects = vec![
            Rect {
                x: 0.0,
                y: 0.0,
                width,
                height: TITLE_BAR * scale,
                color: rgba(self.theme.background, 1.0),
            },
            Rect {
                x: 0.0,
                y: height - STATUS_BAR * scale,
                width,
                height: STATUS_BAR * scale,
                color: rgba(self.theme.border, 0.9),
            },
        ];
        rects.extend(chrome_rects);
        let count = self
            .rects
            .prepare(&self.device, &self.queue, &rects, width, height);

        let chrome_text = TextArea {
            buffer: &self.chrome_buffer,
            left: 8.0 * scale,
            top: 5.0 * scale,
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: (TITLE_BAR * scale) as i32,
            },
            default_color: Color::rgb(
                self.theme.accent.0,
                self.theme.accent.1,
                self.theme.accent.2,
            ),
            custom_glyphs: &[],
        };
        let status_text = TextArea {
            buffer: &self.status_buffer,
            left: 8.0 * scale,
            top: height - STATUS_BAR * scale + 2.0 * scale,
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: (height - STATUS_BAR * scale) as i32,
                right: width as i32,
                bottom: height as i32,
            },
            default_color: Color::rgb(
                self.theme.foreground.0,
                self.theme.foreground.1,
                self.theme.foreground.2,
            ),
            custom_glyphs: &[],
        };
        let modal_width = self
            .modal_buffer
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0_f32, f32::max);
        let modal_text = TextArea {
            buffer: &self.modal_buffer,
            left: ((width - modal_width) / 2.0).max(8.0),
            top: height / 2.0 - 10.0 * scale,
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            },
            default_color: Color::rgb(
                self.theme.danger.0,
                self.theme.danger.1,
                self.theme.danger.2,
            ),
            custom_glyphs: &[],
        };

        let mut areas: Vec<TextArea<'_>> = Vec::new();
        for pane in &text_areas {
            if let Some(render) = self.renders.get(&pane.pane_id) {
                areas.push(TextArea {
                    buffer: &render.buffer,
                    left: pane.left,
                    top: pane.top,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: pane.left as i32,
                        top: pane.top as i32,
                        right: pane.right,
                        bottom: pane.bottom,
                    },
                    default_color: Color::rgb(
                        self.theme.foreground.0,
                        self.theme.foreground.1,
                        self.theme.foreground.2,
                    ),
                    custom_glyphs: &[],
                });
            }
        }
        areas.push(chrome_text);
        areas.push(status_text);
        if self.modal.is_some() {
            areas.push(modal_text);
        }

        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                areas,
                &mut self.swash_cache,
            )
            .map_err(|error| error.to_string())?;

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                self.window.request_redraw();
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Suboptimal(_)
            | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.surface_config);
                self.window.request_redraw();
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("error de validación de la superficie".to_owned())
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ancleto-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.theme.background.0 as f64 / 255.0,
                            g: self.theme.background.1 as f64 / 255.0,
                            b: self.theme.background.2 as f64 / 255.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.rects.render(&mut pass, count);
            self.text_renderer
                .render(&self.atlas, &self.viewport, &mut pass)
                .map_err(|error| error.to_string())?;
        }
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
        self.atlas.trim();

        self.collect_session_events();
        Ok(())
    }

    fn detect_activity(&mut self) {
        let active_tab = self.layout.active().map(|tab| tab.id.clone());
        let mut attention = false;
        let mut active = Vec::new();
        for (pane_id, session) in &self.sessions {
            let marks = session.mark_count();
            let previous = self.marks_seen.insert(pane_id.clone(), marks).unwrap_or(0);
            if marks > previous {
                if let Some(tab_id) = tab_for_pane(&self.layout, pane_id) {
                    if Some(tab_id.clone()) != active_tab {
                        self.activity.insert(tab_id);
                        active.push(pane_id.clone());
                    }
                }
                attention = true;
            }
        }
        if attention && !self.focused {
            self.window
                .request_user_attention(Some(winit::window::UserAttentionType::Informational));
        }
        if !active.is_empty() {
            self.update_chrome();
        }
    }

    fn update_pane_buffers(&mut self) {
        for pane in &self.panes {
            let Some(session) = self.sessions.get(&pane.pane_id) else {
                continue;
            };
            let revision = session.revision();
            let columns = session.columns();
            let lines = session.lines();
            let Some(snapshot) = session.snapshot() else {
                continue;
            };
            let needs_update = self
                .renders
                .get(&pane.pane_id)
                .map(|render| {
                    render.revision != revision
                        || render.columns != columns
                        || render.lines != lines
                })
                .unwrap_or(true);
            if !needs_update {
                continue;
            }

            let text = snapshot
                .rows
                .iter()
                .map(|row| row.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let (width, height) = (
                (pane.width - PADDING * 2.0) * self.scale_factor as f32,
                (pane.height - PADDING * 2.0) * self.scale_factor as f32,
            );
            let scale = self.scale_factor as f32;
            if !self.renders.contains_key(&pane.pane_id) {
                let buffer = Buffer::new(
                    &mut self.font_system,
                    Metrics::new(self.font_size * scale, self.cell_height * scale),
                );
                self.renders.insert(
                    pane.pane_id.clone(),
                    PaneRender {
                        buffer,
                        revision: 0,
                        columns: 0,
                        lines: 0,
                        selection: None,
                    },
                );
            }
            let Some(entry) = self.renders.get_mut(&pane.pane_id) else {
                continue;
            };
            entry.buffer.set_metrics(Metrics::new(
                self.font_size * scale,
                self.cell_height * scale,
            ));
            entry.buffer.set_size(Some(width), Some(height));
            entry.buffer.set_text(
                &text,
                &Attrs::new().family(Family::Monospace),
                Shaping::Basic,
                None,
            );
            entry
                .buffer
                .shape_until_scroll(&mut self.font_system, false);
            entry.revision = revision;
            entry.columns = columns;
            entry.lines = lines;
        }
    }

    fn collect_session_events(&mut self) {
        let panes: Vec<String> = self.sessions.keys().cloned().collect();
        for pane_id in panes {
            let Some(session) = self.sessions.get_mut(&pane_id) else {
                continue;
            };
            for event in session.take_events() {
                match event {
                    UiEvent::Bell => {
                        self.window.request_user_attention(Some(
                            winit::window::UserAttentionType::Informational,
                        ));
                    }
                    UiEvent::ClipboardStore(text) => {
                        self.modal = Some(Modal::Clipboard(text));
                        self.update_chrome();
                    }
                    UiEvent::Title(_) | UiEvent::PtyWrite(_) | UiEvent::Exit => {}
                }
            }
        }
    }
}

struct Rect2 {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

struct PaneText {
    pane_id: String,
    left: f32,
    top: f32,
    right: i32,
    bottom: i32,
}

fn collect_geometry(node: &SplitNode, rect: Rect2, focused: &str, out: &mut Vec<PaneLayout>) {
    match node {
        SplitNode::Pane { pane } => out.push(PaneLayout {
            pane_id: pane.id.clone(),
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            focused: pane.id == focused,
        }),
        SplitNode::Split {
            axis,
            ratio,
            first,
            second,
        } => {
            let ratio = ratio.clamp(0.1, 0.9);
            match axis {
                Axis::Horizontal => {
                    let left = rect.width * ratio - 1.0;
                    collect_geometry(
                        first,
                        Rect2 {
                            x: rect.x,
                            y: rect.y,
                            width: left.max(1.0),
                            height: rect.height,
                        },
                        focused,
                        out,
                    );
                    collect_geometry(
                        second,
                        Rect2 {
                            x: rect.x + left + 2.0,
                            y: rect.y,
                            width: (rect.width - left - 2.0).max(1.0),
                            height: rect.height,
                        },
                        focused,
                        out,
                    );
                }
                Axis::Vertical => {
                    let top = rect.height * ratio - 1.0;
                    collect_geometry(
                        first,
                        Rect2 {
                            x: rect.x,
                            y: rect.y,
                            width: rect.width,
                            height: top.max(1.0),
                        },
                        focused,
                        out,
                    );
                    collect_geometry(
                        second,
                        Rect2 {
                            x: rect.x,
                            y: rect.y + top + 2.0,
                            width: rect.width,
                            height: (rect.height - top - 2.0).max(1.0),
                        },
                        focused,
                        out,
                    );
                }
            }
        }
    }
}

fn tab_for_pane(layout: &TerminalLayout, pane_id: &str) -> Option<String> {
    layout
        .tabs
        .iter()
        .find(|tab| tab.root.contains(pane_id))
        .map(|tab| tab.id.clone())
}

fn collect_panes_mut(node: &mut SplitNode, callback: &mut impl FnMut(&mut Pane)) {
    match node {
        SplitNode::Pane { pane } => callback(pane),
        SplitNode::Split { first, second, .. } => {
            collect_panes_mut(first, callback);
            collect_panes_mut(second, callback);
        }
    }
}

fn find_pane_mut<'a>(node: &'a mut SplitNode, pane_id: &str) -> Option<&'a mut Pane> {
    match node {
        SplitNode::Pane { pane } => (pane.id == pane_id).then_some(pane),
        SplitNode::Split { first, second, .. } => {
            find_pane_mut(first, pane_id).or_else(|| find_pane_mut(second, pane_id))
        }
    }
}

fn find_pane_mut_by_id<'a>(layout: &'a mut TerminalLayout, pane_id: &str) -> Option<&'a mut Pane> {
    layout
        .tabs
        .iter_mut()
        .find_map(|tab| find_pane_mut(&mut tab.root, pane_id))
}

fn resolve_project(requested: Option<&str>) -> Result<(String, String), String> {
    let service =
        ProjectService::open(&ancleto_code_core::host::transport::default_database_path())
            .map_err(|error| error.to_string())?;
    let projects = service.list().map_err(|error| error.to_string())?;

    if let Some(requested) = requested {
        let path = Path::new(requested);
        if path.is_dir() {
            let canonical = path.canonicalize().map_err(|error| error.to_string())?;
            let root = canonical.to_string_lossy().into_owned();
            let id = projects
                .iter()
                .find(|project| project.root_path == root)
                .map(|project| project.id.clone())
                .unwrap_or_else(|| root.clone());
            return Ok((id, root));
        }
        if let Some(project) = projects.iter().find(|project| project.id == requested) {
            return Ok((project.id.clone(), project.root_path.clone()));
        }
        return Err(format!("proyecto no encontrado: {requested}"));
    }

    if let Some(project) = projects.first() {
        return Ok((project.id.clone(), project.root_path.clone()));
    }

    let root = std::env::current_dir()
        .map_err(|error| error.to_string())?
        .to_string_lossy()
        .into_owned();
    Ok((root.clone(), root))
}

fn resolve_profile(requested: Option<&str>) -> Result<ShellProfile, String> {
    let profiles = default_profiles();
    if let Some(requested) = requested {
        return profiles
            .into_iter()
            .find(|profile| profile.id == requested || profile.program == requested)
            .ok_or_else(|| format!("shell no soportada: {requested}"));
    }
    profiles
        .into_iter()
        .next()
        .ok_or_else(|| "no hay shells disponibles".to_owned())
}

fn set_buffer_text(font_system: &mut FontSystem, buffer: &mut Buffer, text: &str) {
    buffer.set_text(
        text,
        &Attrs::new().family(Family::Monospace),
        Shaping::Basic,
        None,
    );
    buffer.shape_until_scroll(font_system, false);
}

fn measure_cell(font_system: &mut FontSystem, font_size: f32) -> (f32, f32) {
    let metrics = Metrics::new(font_size, font_size * 1.35);
    let mut buffer = Buffer::new(font_system, metrics);
    buffer.set_size(Some(1000.0), Some(1000.0));
    buffer.set_text(
        "M",
        &Attrs::new().family(Family::Monospace),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(font_system, false);
    let run = buffer.layout_runs().next();
    let width = run
        .and_then(|run| run.glyphs.first().map(|glyph| glyph.w))
        .filter(|width| *width > 0.0)
        .unwrap_or(font_size * 0.6);
    (width, font_size * 1.35)
}

fn encode_key(event: &winit::event::KeyEvent, modifiers: ModifiersState) -> Option<Vec<u8>> {
    let control = modifiers.control_key();
    let alt = modifiers.alt_key();
    let app_cursor = false;
    let mut bytes = match &event.logical_key {
        Key::Named(NamedKey::Enter) => b"\r".to_vec(),
        Key::Named(NamedKey::Tab) => b"\t".to_vec(),
        Key::Named(NamedKey::Backspace) => vec![0x7f],
        Key::Named(NamedKey::Escape) => vec![0x1b],
        Key::Named(NamedKey::Delete) => b"\x1b[3~".to_vec(),
        Key::Named(NamedKey::Insert) => b"\x1b[2~".to_vec(),
        Key::Named(NamedKey::Home) => {
            if app_cursor {
                b"\x1bOH".to_vec()
            } else {
                b"\x1b[H".to_vec()
            }
        }
        Key::Named(NamedKey::End) => {
            if app_cursor {
                b"\x1bOF".to_vec()
            } else {
                b"\x1b[F".to_vec()
            }
        }
        Key::Named(NamedKey::ArrowUp) => b"\x1b[A".to_vec(),
        Key::Named(NamedKey::ArrowDown) => b"\x1b[B".to_vec(),
        Key::Named(NamedKey::ArrowRight) => b"\x1b[C".to_vec(),
        Key::Named(NamedKey::ArrowLeft) => b"\x1b[D".to_vec(),
        Key::Named(NamedKey::PageUp) => b"\x1b[5~".to_vec(),
        Key::Named(NamedKey::PageDown) => b"\x1b[6~".to_vec(),
        Key::Named(NamedKey::F1) => b"\x1bOP".to_vec(),
        Key::Named(NamedKey::F2) => b"\x1bOQ".to_vec(),
        Key::Named(NamedKey::F3) => b"\x1bOR".to_vec(),
        Key::Named(NamedKey::F4) => b"\x1bOS".to_vec(),
        Key::Named(NamedKey::F5) => b"\x1b[15~".to_vec(),
        Key::Named(NamedKey::F6) => b"\x1b[17~".to_vec(),
        Key::Named(NamedKey::F7) => b"\x1b[18~".to_vec(),
        Key::Named(NamedKey::F8) => b"\x1b[19~".to_vec(),
        Key::Named(NamedKey::F9) => b"\x1b[20~".to_vec(),
        Key::Named(NamedKey::F10) => b"\x1b[21~".to_vec(),
        Key::Named(NamedKey::F11) => b"\x1b[23~".to_vec(),
        Key::Named(NamedKey::F12) => b"\x1b[24~".to_vec(),
        Key::Character(text) => {
            if control {
                let character = text.chars().next()?;
                let code = match character.to_ascii_lowercase() {
                    'a'..='z' => character.to_ascii_lowercase() as u8 - b'a' + 1,
                    '[' => 0x1b,
                    '\\' => 0x1c,
                    ']' => 0x1d,
                    '^' => 0x1e,
                    '_' => 0x1f,
                    _ => return None,
                };
                vec![code]
            } else {
                text.as_bytes().to_vec()
            }
        }
        _ => return event.text.as_ref().map(|text| text.as_bytes().to_vec()),
    };

    if alt && !bytes.starts_with(&[0x1b]) {
        bytes.insert(0, 0x1b);
    }
    Some(bytes)
}

fn word_selection(
    sessions: &HashMap<String, TerminalSession>,
    pane_id: &str,
    line: i32,
    column: usize,
) -> Selection {
    let text = sessions
        .get(pane_id)
        .and_then(|session| session.snapshot())
        .and_then(|snapshot| snapshot.rows.into_iter().find(|row| row.line == line))
        .map(|row| row.text)
        .unwrap_or_default();
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Selection {
            start_line: line,
            start_column: column,
            end_line: line,
            end_column: column,
        };
    }
    let column = column.min(chars.len() - 1);
    let is_word = |ch: char| ch.is_alphanumeric() || ch == '_';
    let mut start = column;
    while start > 0 && is_word(chars[start - 1]) {
        start -= 1;
    }
    let mut end = column;
    while end + 1 < chars.len() && is_word(chars[end + 1]) {
        end += 1;
    }
    Selection {
        start_line: line,
        start_column: start,
        end_line: line,
        end_column: end,
    }
}

fn line_selection(
    sessions: &HashMap<String, TerminalSession>,
    pane_id: &str,
    line: i32,
    column: usize,
) -> Selection {
    let end = sessions
        .get(pane_id)
        .and_then(|session| session.snapshot())
        .and_then(|snapshot| snapshot.rows.into_iter().find(|row| row.line == line))
        .map(|row| row.text.chars().count().saturating_sub(1))
        .unwrap_or(column);
    Selection {
        start_line: line,
        start_column: 0,
        end_line: line,
        end_column: end.max(column),
    }
}

fn mouse_modifiers(modifiers: ModifiersState) -> u8 {
    let mut value = 0;
    if modifiers.shift_key() {
        value += 4;
    }
    if modifiers.alt_key() {
        value += 8;
    }
    if modifiers.control_key() {
        value += 16;
    }
    value
}
