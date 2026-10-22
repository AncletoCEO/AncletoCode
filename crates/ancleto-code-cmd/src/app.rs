use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use ancleto_code_core::host::HostPaths;
use ancleto_code_core::providers::{SpecProvider, ToolReport, WriterProvider};
use ancleto_code_core::{
    ConversationState, DriverRegistry, DriverStatus, HostClient, Orchestrator, Project,
    ProjectService, SavedSession, SessionBackend, SessionSpec, SessionStatus, StartConversation,
    Store, Turn,
};
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

use crate::keymap::{Action, Keymap};
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
const SIDEBAR_WIDTH: f32 = 220.0;

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

struct LineRender {
    text: String,
    buffer: Buffer,
}

struct PaneRender {
    lines: Vec<LineRender>,
    revision: u64,
    columns: usize,
    screen_lines: usize,
    width: f32,
    height: f32,
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
    KillSession(String),
    Clipboard(String),
    Hyperlink(String),
}

#[derive(Debug, Clone)]
enum Overlay {
    Palette(PaletteState),
    Spec(TextPanel),
    Writer(TextPanel),
    Conversation(ConversationView),
}

#[derive(Debug, Clone)]
struct PaletteState {
    query: String,
    selected: usize,
}

#[derive(Debug, Clone)]
struct TextPanel {
    lines: Vec<String>,
}

#[derive(Debug, Clone)]
struct ConversationView {
    session_a: Option<String>,
    session_b: Option<String>,
    objective: String,
    conversation_id: Option<String>,
    turns: Vec<Turn>,
    state: Option<ConversationState>,
    reason: Option<String>,
    error: Option<String>,
    sessions: Vec<(String, String, SessionStatus)>,
    focus: usize,
}

#[derive(Clone)]
enum PaletteAction {
    NewTab,
    SplitHorizontal,
    SplitVertical,
    ClosePanel,
    Copy,
    Paste,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    KillAll,
    SpecPanel,
    WriterPanel,
    ConversationPanel,
    Quit,
}

#[derive(Debug, Clone)]
enum SidebarAction {
    SelectProject {
        id: String,
        root: String,
    },
    AttachSession(String),
    NewAgent {
        project_id: String,
        root: String,
        driver_id: String,
        program: String,
    },
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
    keymap: Keymap,
    focused: bool,
    activity: std::collections::HashSet<String>,
    marks_seen: HashMap<String, usize>,
    projects: Vec<Project>,
    saved_sessions: Vec<SavedSession>,
    live_status: HashMap<String, SessionStatus>,
    drivers: Vec<DriverStatus>,
    providers: HashMap<String, (ToolReport, ToolReport)>,
    sidebar_buffer: Buffer,
    sidebar_lines: Vec<(String, Option<SidebarAction>)>,
    last_sidebar_refresh: std::time::Instant,
    overlay: Option<Overlay>,
    overlay_buffer: Buffer,
    orchestrator: Orchestrator,
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
        font_system.db_mut().load_font_data(
            include_bytes!("../../../public/fonts/IBMPlexMono-Regular.ttf").to_vec(),
        );
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
        let keymap = Keymap::load(&store);

        let paths = HostPaths::discover(None);
        let client = Arc::new(
            HostClient::connect_or_start(
                &paths,
                &ancleto_code_core::host::transport::default_database_path(),
            )
            .map_err(|error| format!("no se pudo conectar al host: {error}"))?,
        );

        let rects = RectRenderer::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb);
        let sidebar_buffer = Buffer::new(
            &mut font_system,
            Metrics::new(11.0 * scale_factor as f32, 16.0 * scale_factor as f32),
        );
        let overlay_buffer = Buffer::new(
            &mut font_system,
            Metrics::new(12.0 * scale_factor as f32, 18.0 * scale_factor as f32),
        );
        let orchestrator = Orchestrator::new(
            Store::open(&ancleto_code_core::host::transport::default_database_path())
                .map_err(|error| format!("no se pudo abrir la base de conversaciones: {error}"))?,
            Arc::clone(&client) as Arc<dyn SessionBackend>,
            DriverRegistry::from_store(&store),
            None,
        );
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
            keymap,
            focused: true,
            activity: std::collections::HashSet::new(),
            marks_seen: HashMap::new(),
            projects: Vec::new(),
            saved_sessions: Vec::new(),
            live_status: HashMap::new(),
            drivers: Vec::new(),
            providers: HashMap::new(),
            sidebar_buffer,
            sidebar_lines: Vec::new(),
            last_sidebar_refresh: std::time::Instant::now(),
            overlay: None,
            overlay_buffer,
            orchestrator,
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

        state.projects =
            ProjectService::open(&ancleto_code_core::host::transport::default_database_path())
                .and_then(|service| service.list())
                .unwrap_or_default();
        state.drivers = DriverRegistry::from_store(&state.store).detect_all();
        for project in state.projects.iter().take(8) {
            let spec = SpecProvider::detect(Path::new(&project.root_path)).tool();
            let writer = WriterProvider::detect(Path::new(&project.root_path)).tool();
            state.providers.insert(project.id.clone(), (spec, writer));
        }
        state.refresh_sidebar();
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
            env: terminal_env(&self.profile.env),
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
                self.handle_mouse_button(event_loop, state, button);
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

    fn open_palette(&mut self) {
        self.overlay = Some(Overlay::Palette(PaletteState {
            query: String::new(),
            selected: 0,
        }));
        self.update_overlay_buffer();
        self.window.request_redraw();
    }

    fn open_spec_panel(&mut self) {
        let root = Path::new(&self.project_root);
        let provider = SpecProvider::detect(root);
        let tool = provider.tool();
        let mut lines = vec![format!(
            "ancleto: {} {}",
            tool_label(&tool),
            tool.version.clone().unwrap_or_default()
        )];
        match provider.project_state(root) {
            Ok(state) => {
                lines.push(format!(
                    "inicializado: {}",
                    if state.initialized { "sí" } else { "no" }
                ));
                lines.push(format!(
                    "host: {} · tier: {} · idioma: {}",
                    state.host.unwrap_or_else(|| "—".to_owned()),
                    state.tier.unwrap_or_else(|| "—".to_owned()),
                    state.language.unwrap_or_else(|| "—".to_owned())
                ));
                if !state.active_changes.is_empty() {
                    lines.push(String::new());
                    lines.push("changes activos:".to_owned());
                    for change in &state.active_changes {
                        lines.push(format!(
                            "  · {} [{}]",
                            change.name,
                            change.artifacts.join(", ")
                        ));
                    }
                }
            }
            Err(error) => lines.push(format!("estado: {error}")),
        }
        if tool.status == ancleto_code_core::drivers::ToolStatus::Available {
            if let Ok(discovery) = provider.discovery_check(root) {
                lines.push(String::new());
                lines.push(format!(
                    "discovery: {} · impact {} · {}",
                    discovery.state,
                    discovery.impact,
                    discovery.message.unwrap_or_default()
                ));
            }
            if let Ok(check) = provider.specs_check(root, None) {
                lines.push(format!(
                    "specs check: {} · {} escaneados · {} no canónicos",
                    if check.ok { "ok" } else { "con errores" },
                    check.scanned,
                    check.non_canonical.len()
                ));
            }
            if let Ok(context) = provider.memory_context(root) {
                lines.push(String::new());
                lines.push("memoria:".to_owned());
                if context.trim().is_empty() {
                    lines.push("  sin reglas activas".to_owned());
                } else {
                    for line in context.lines().take(12) {
                        lines.push(format!("  {line}"));
                    }
                }
            }
        }
        self.overlay = Some(Overlay::Spec(TextPanel { lines }));
        self.update_overlay_buffer();
        self.window.request_redraw();
    }

    fn open_writer_panel(&mut self) {
        let root = Path::new(&self.project_root);
        let provider = WriterProvider::detect(root);
        let mut lines = Vec::new();
        match provider.state(root) {
            Ok(state) => {
                lines.push(format!(
                    "ancleto-writer: {} {}",
                    tool_label(&state.tool),
                    state.tool.version.clone().unwrap_or_default()
                ));
                if let Some(check) = &state.check {
                    lines.push(format!("frescura: {}", check.status));
                }
                if let Some(manifest) = &state.manifest {
                    lines.push(format!(
                        "manifest: schema {} · writer {}",
                        manifest.schema_version,
                        manifest.writer_version.clone().unwrap_or_default()
                    ));
                    lines.push(format!("stats: {}", manifest.stats));
                }
                if let Some(graph) = &state.graph {
                    lines.push(format!(
                        "grafo: {} nodos · {} aristas",
                        graph.nodes, graph.edges
                    ));
                }
                lines.push(format!(
                    "language pack: {} · {} · {}",
                    state.language_pack.module,
                    if state.language_pack.installed {
                        "instalado"
                    } else {
                        "ausente"
                    },
                    if state.language_pack.enabled {
                        "habilitado"
                    } else {
                        "deshabilitado"
                    }
                ));
                lines.push(String::new());
                lines.push("hechos:".to_owned());
                for (name, value) in &state.facts {
                    lines.push(format!("  {name}: {}", summarize_fact(value)));
                }
                if let Some(unsupported) = state.facts.get("unsupported") {
                    lines.push(format!("  unsupported: {}", summarize_fact(unsupported)));
                }
            }
            Err(error) => lines.push(format!("writer: {error}")),
        }
        self.overlay = Some(Overlay::Writer(TextPanel { lines }));
        self.update_overlay_buffer();
        self.window.request_redraw();
    }

    fn open_conversation_panel(&mut self) {
        let sessions: Vec<(String, String, SessionStatus)> = self
            .saved_sessions
            .iter()
            .filter(|session| {
                session.spec.project_id == self.project_id
                    && !matches!(
                        session.status,
                        SessionStatus::Stopped | SessionStatus::Crashed
                    )
            })
            .map(|session| {
                (
                    session.id.clone(),
                    session.spec.driver_id.clone(),
                    self.live_status
                        .get(&session.id)
                        .copied()
                        .unwrap_or(session.status),
                )
            })
            .collect();
        let session_a = sessions.first().map(|entry| entry.0.clone());
        let session_b = sessions.get(1).map(|entry| entry.0.clone());
        self.overlay = Some(Overlay::Conversation(ConversationView {
            session_a,
            session_b,
            objective: String::new(),
            conversation_id: None,
            turns: Vec::new(),
            state: None,
            reason: None,
            error: None,
            sessions,
            focus: 0,
        }));
        self.update_overlay_buffer();
        self.window.request_redraw();
    }

    fn palette_items(&self) -> Vec<(&'static str, PaletteAction)> {
        vec![
            ("Nueva tab", PaletteAction::NewTab),
            ("Dividir horizontal", PaletteAction::SplitHorizontal),
            ("Dividir vertical", PaletteAction::SplitVertical),
            ("Cerrar panel", PaletteAction::ClosePanel),
            ("Copiar selección", PaletteAction::Copy),
            ("Pegar", PaletteAction::Paste),
            ("Zoom +", PaletteAction::ZoomIn),
            ("Zoom −", PaletteAction::ZoomOut),
            ("Zoom 100%", PaletteAction::ZoomReset),
            ("Panel de spec", PaletteAction::SpecPanel),
            ("Panel de writer", PaletteAction::WriterPanel),
            (
                "Conversación agente-a-agente",
                PaletteAction::ConversationPanel,
            ),
            ("Matar todas las sesiones", PaletteAction::KillAll),
            ("Salir", PaletteAction::Quit),
        ]
    }

    fn filtered_palette(&self, query: &str) -> Vec<(&'static str, PaletteAction)> {
        let query = query.to_lowercase();
        self.palette_items()
            .into_iter()
            .filter(|(label, _)| query.is_empty() || label.to_lowercase().contains(&query))
            .collect()
    }

    fn handle_overlay_key(&mut self, event_loop: &ActiveEventLoop, event: &winit::event::KeyEvent) {
        let Some(mut overlay) = self.overlay.take() else {
            return;
        };
        let escape = matches!(event.logical_key, Key::Named(NamedKey::Escape));
        let mut keep = true;
        let mut pending_action: Option<PaletteAction> = None;
        let mut refresh_panel: Option<bool> = None;

        match &mut overlay {
            Overlay::Palette(state) => {
                if escape {
                    keep = false;
                } else {
                    match &event.logical_key {
                        Key::Named(NamedKey::Backspace) => {
                            state.query.pop();
                            state.selected = 0;
                        }
                        Key::Named(NamedKey::ArrowDown) => {
                            let count = self.filtered_palette(&state.query).len();
                            if count > 0 {
                                state.selected = (state.selected + 1) % count;
                            }
                        }
                        Key::Named(NamedKey::ArrowUp) => {
                            let count = self.filtered_palette(&state.query).len();
                            if count > 0 {
                                state.selected = (state.selected + count - 1) % count;
                            }
                        }
                        Key::Named(NamedKey::Enter) => {
                            let items = self.filtered_palette(&state.query);
                            if let Some((_, action)) = items.get(state.selected) {
                                pending_action = Some(action.clone());
                                keep = false;
                            }
                        }
                        Key::Character(text) => {
                            state.query.push_str(text);
                            state.selected = 0;
                        }
                        _ => {}
                    }
                }
            }
            Overlay::Spec(_) | Overlay::Writer(_) => {
                if escape {
                    keep = false;
                } else if matches!(&event.logical_key, Key::Character(text) if text.eq_ignore_ascii_case("r"))
                {
                    refresh_panel = Some(matches!(overlay, Overlay::Writer(_)));
                    keep = false;
                }
            }
            Overlay::Conversation(view) => {
                if escape {
                    keep = false;
                } else {
                    match &event.logical_key {
                        Key::Named(NamedKey::Tab) => view.focus = (view.focus + 1) % 3,
                        Key::Named(NamedKey::Backspace) if view.focus == 2 => {
                            view.objective.pop();
                        }
                        Key::Named(NamedKey::ArrowUp) if view.focus < 2 => {
                            shift_choice(view, -1);
                        }
                        Key::Named(NamedKey::ArrowDown) if view.focus < 2 => {
                            shift_choice(view, 1);
                        }
                        Key::Character(text) if self.modifiers.control_key() => {
                            if let Some(id) = view.conversation_id.clone() {
                                if text.eq_ignore_ascii_case("p") {
                                    let _ = self.orchestrator.pause(&id);
                                } else if text.eq_ignore_ascii_case("r") {
                                    let _ = self.orchestrator.resume(&id);
                                } else if text.eq_ignore_ascii_case("a") {
                                    let _ = self.orchestrator.abort(&id);
                                }
                            }
                        }
                        Key::Character(text) if view.focus == 2 => {
                            view.objective.push_str(text);
                        }
                        Key::Named(NamedKey::Enter) if view.focus == 2 => {
                            let (Some(a), Some(b)) =
                                (view.session_a.clone(), view.session_b.clone())
                            else {
                                view.error = Some("hacen falta dos sesiones".to_owned());
                                self.overlay = Some(overlay);
                                self.update_overlay_buffer();
                                self.window.request_redraw();
                                return;
                            };
                            if a == b || view.objective.trim().is_empty() {
                                view.error =
                                    Some("elegí dos sesiones distintas y un objetivo".to_owned());
                                self.overlay = Some(overlay);
                                self.update_overlay_buffer();
                                self.window.request_redraw();
                                return;
                            }
                            match self.orchestrator.start(StartConversation {
                                session_a: a,
                                session_b: b,
                                objective: view.objective.clone(),
                                max_turns: Some(6),
                                turn_budget_ms: Some(60_000),
                            }) {
                                Ok(conversation) => {
                                    view.conversation_id = Some(conversation.id);
                                    view.state = Some(ConversationState::Running);
                                    view.turns.clear();
                                    view.error = None;
                                }
                                Err(error) => view.error = Some(error.to_string()),
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if keep {
            self.overlay = Some(overlay);
        }
        if let Some(writer) = refresh_panel {
            if writer {
                self.open_writer_panel();
            } else {
                self.open_spec_panel();
            }
            return;
        }
        if let Some(action) = pending_action {
            self.execute_palette(event_loop, action);
            return;
        }
        self.update_overlay_buffer();
        self.window.request_redraw();
    }

    fn execute_palette(&mut self, event_loop: &ActiveEventLoop, action: PaletteAction) {
        match action {
            PaletteAction::NewTab => self.perform_action(event_loop, Action::NewTab),
            PaletteAction::SplitHorizontal => {
                self.perform_action(event_loop, Action::SplitHorizontal)
            }
            PaletteAction::SplitVertical => self.perform_action(event_loop, Action::SplitVertical),
            PaletteAction::ClosePanel => self.perform_action(event_loop, Action::ClosePanel),
            PaletteAction::Copy => self.perform_action(event_loop, Action::Copy),
            PaletteAction::Paste => self.perform_action(event_loop, Action::Paste),
            PaletteAction::ZoomIn => self.perform_action(event_loop, Action::ZoomIn),
            PaletteAction::ZoomOut => self.perform_action(event_loop, Action::ZoomOut),
            PaletteAction::ZoomReset => self.perform_action(event_loop, Action::ZoomReset),
            PaletteAction::KillAll => self.perform_action(event_loop, Action::KillAll),
            PaletteAction::Quit => self.shutdown(event_loop, false),
            PaletteAction::SpecPanel => self.open_spec_panel(),
            PaletteAction::WriterPanel => self.open_writer_panel(),
            PaletteAction::ConversationPanel => self.open_conversation_panel(),
        }
    }

    fn update_overlay_buffer(&mut self) {
        if let Some(Overlay::Conversation(view)) = &mut self.overlay {
            if let Some(id) = view.conversation_id.clone() {
                if let Ok(conversation) = self.orchestrator.conversation(&id) {
                    view.state = Some(conversation.state);
                    view.reason = conversation.reason.clone();
                }
                if let Ok(turns) = self.orchestrator.turns(&id) {
                    view.turns = turns;
                }
            }
        }

        let lines = match &self.overlay {
            None => Vec::new(),
            Some(Overlay::Palette(state)) => {
                let mut lines = vec![
                    "command palette".to_owned(),
                    format!("filtro: {}", state.query),
                    String::new(),
                ];
                for (index, (label, _)) in self
                    .filtered_palette(&state.query)
                    .iter()
                    .enumerate()
                    .take(20)
                {
                    let marker = if index == state.selected { "▶" } else { " " };
                    lines.push(format!("{marker} {label}"));
                }
                lines.push(String::new());
                lines.push("↑↓ elegir · Enter ejecutar · Esc cerrar".to_owned());
                lines
            }
            Some(Overlay::Spec(panel)) | Some(Overlay::Writer(panel)) => {
                let mut lines = panel.lines.clone();
                lines.push(String::new());
                lines.push("r recargar · Esc cerrar".to_owned());
                lines
            }
            Some(Overlay::Conversation(view)) => {
                let mut lines = vec!["conversación agente-a-agente".to_owned(), String::new()];
                lines.push("sesiones:".to_owned());
                for (index, (id, driver, status)) in view.sessions.iter().enumerate() {
                    let marker = if Some(id) == view.session_a.as_ref() {
                        "A"
                    } else if Some(id) == view.session_b.as_ref() {
                        "B"
                    } else {
                        " "
                    };
                    lines.push(format!(
                        "  [{marker}] {} · {} · {id}",
                        driver,
                        status_glyph(*status)
                    ));
                    let _ = index;
                }
                lines.push(String::new());
                lines.push(format!("objetivo: {}", view.objective));
                if let Some(state) = view.state {
                    lines.push(format!(
                        "estado: {} · {} turnos{}",
                        conversation_state_label(state),
                        view.turns.len(),
                        view.reason
                            .as_ref()
                            .map(|reason| format!(" · {reason}"))
                            .unwrap_or_default()
                    ));
                }
                if let Some(error) = &view.error {
                    lines.push(format!("error: {error}"));
                }
                lines.push(String::new());
                lines.push(
                    "Tab foco · ↑↓ sesión · Enter iniciar · Ctrl+P/R/A · Esc cerrar".to_owned(),
                );
                lines.push(String::new());
                for turn in view.turns.iter().rev().take(12).rev() {
                    lines.push(format!(
                        "#{} {} → {}: {}",
                        turn.index,
                        turn.from_session,
                        turn.to_session,
                        turn.text.lines().next().unwrap_or_default()
                    ));
                }
                lines
            }
        };

        let text = lines.join("\n");
        self.overlay_buffer.set_size(
            Some(
                (self.surface_config.width as f32 / self.scale_factor as f32 - 160.0)
                    * self.scale_factor as f32,
            ),
            None,
        );
        set_buffer_text(&mut self.font_system, &mut self.overlay_buffer, &text);
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &winit::event::KeyEvent) {
        if self.overlay.is_some() {
            self.handle_overlay_key(event_loop, event);
            return;
        }
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
                Modal::KillSession(session_id) if confirm => {
                    let session_id = session_id.clone();
                    self.close_session_panes(&session_id);
                }
                Modal::KillSession(_) if cancel => self.modal = None,
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

        if let Some(action) = self.keymap.action_for(event, self.modifiers) {
            self.perform_action(event_loop, action);
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
            self.status = Some("no se pudo dividir el panel".to_owned());
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

    fn perform_action(&mut self, event_loop: &ActiveEventLoop, action: Action) {
        match action {
            Action::NewTab => {
                if let Ok(pane) = self.create_session() {
                    self.layout.new_tab(self.profile.display_name.clone(), pane);
                    self.update_layout_geometry();
                    self.update_chrome();
                    self.window.request_redraw();
                }
            }
            Action::ClosePanel => {
                if let Some(pane) = self.focused_pane_id() {
                    self.modal = Some(Modal::ClosePane(pane));
                    self.update_chrome();
                    self.window.request_redraw();
                }
            }
            Action::SplitHorizontal => self.split_focused(Axis::Horizontal),
            Action::SplitVertical => self.split_focused(Axis::Vertical),
            Action::Copy => self.copy_selection(),
            Action::Paste => self.paste(),
            Action::Quit => self.shutdown(event_loop, false),
            Action::KillAll => {
                self.modal = Some(Modal::KillAll);
                self.update_chrome();
                self.window.request_redraw();
            }
            Action::ZoomIn => self.zoom(1.0),
            Action::ZoomOut => self.zoom(-1.0),
            Action::ZoomReset => {
                self.font_size = 13.0;
                self.recompute_metrics();
            }
            Action::NextPanel => self.focus_next_panel(),
            Action::PreviousTab => self.switch_tab(-1),
            Action::NextTab => self.switch_tab(1),
            Action::ResizeLeft => self.adjust_ratio(Axis::Horizontal, -0.05),
            Action::ResizeRight => self.adjust_ratio(Axis::Horizontal, 0.05),
            Action::ResizeUp => self.adjust_ratio(Axis::Vertical, -0.05),
            Action::ResizeDown => self.adjust_ratio(Axis::Vertical, 0.05),
            Action::SwapPanel => self.swap_with_next_panel(),
            Action::PreviousCommand => self.navigate_command(true),
            Action::NextCommand => self.navigate_command(false),
            Action::CopyCommandOutput => self.copy_command_output(),
            Action::CommandPalette => self.open_palette(),
            Action::SpecPanel => self.open_spec_panel(),
            Action::WriterPanel => self.open_writer_panel(),
            Action::ConversationPanel => self.open_conversation_panel(),
            Action::WriterBuild => self.writer_build(),
            Action::ThemeCycle => {
                self.theme = theme::cycle(&self.store);
                self.window.request_redraw();
            }
            Action::NextProject => self.select_project(1),
            Action::PreviousProject => self.select_project(-1),
        }
    }

    fn writer_build(&mut self) {
        let root = Path::new(&self.project_root);
        let provider = WriterProvider::detect(root);
        let mut lines = vec!["writer build".to_owned(), String::new()];
        match provider.build(root) {
            Ok(outcome) => {
                lines.push(format!(
                    "resultado: {}",
                    if outcome.success { "ok" } else { "falló" }
                ));
                let text = if outcome.stdout.trim().is_empty() {
                    outcome.stderr
                } else {
                    outcome.stdout
                };
                for line in text.lines().take(30) {
                    lines.push(line.to_owned());
                }
            }
            Err(error) => lines.push(format!("error: {error}")),
        }
        self.overlay = Some(Overlay::Spec(TextPanel { lines }));
        self.update_overlay_buffer();
        self.window.request_redraw();
    }

    fn select_project(&mut self, delta: i32) {
        if self.projects.is_empty() {
            return;
        }
        let current = self
            .projects
            .iter()
            .position(|project| project.id == self.project_id)
            .unwrap_or(0) as i32;
        let next = (current + delta).rem_euclid(self.projects.len() as i32) as usize;
        if let Some(project) = self.projects.get(next) {
            self.project_id = project.id.clone();
            self.project_root = project.root_path.clone();
            if let Ok(pane) = self.create_session() {
                self.layout.new_tab(self.profile.display_name.clone(), pane);
                self.refresh_sidebar();
                self.update_layout_geometry();
                self.update_chrome();
                self.window.request_redraw();
            }
        }
    }

    fn focus_next_panel(&mut self) {
        let Some(tab_id) = self.active_tab_id_opt() else {
            return;
        };
        self.layout.next_pane(&tab_id);
        self.update_chrome();
        self.window.request_redraw();
    }

    fn switch_tab(&mut self, delta: i32) {
        if self.layout.tabs.is_empty() {
            return;
        }
        let current = self.layout.active_tab as i32;
        let next = (current + delta).clamp(0, self.layout.tabs.len() as i32 - 1) as usize;
        if self.layout.select_tab(next) {
            if let Some(tab) = self.layout.tabs.get(next) {
                self.activity.remove(&tab.id.clone());
            }
            self.update_layout_geometry();
            self.update_chrome();
            self.window.request_redraw();
        }
    }

    fn adjust_ratio(&mut self, axis: Axis, delta: f32) {
        let Some(pane_id) = self.focused_pane_id() else {
            return;
        };
        let tab_id = self.active_tab_id();
        if self
            .layout
            .adjust_split_ratio(&tab_id, &pane_id, axis, delta)
        {
            self.update_layout_geometry();
            self.update_chrome();
            self.window.request_redraw();
        }
    }

    fn swap_with_next_panel(&mut self) {
        let Some(pane_id) = self.focused_pane_id() else {
            return;
        };
        let tab_id = self.active_tab_id();
        if let Some(other) = next_pane_id(&self.layout, &tab_id, &pane_id) {
            if self.layout.swap_sessions(&tab_id, &pane_id, &other) {
                self.update_layout_geometry();
                self.update_chrome();
                self.window.request_redraw();
            }
        }
    }

    fn active_tab_id_opt(&self) -> Option<String> {
        self.layout.active().map(|tab| tab.id.clone())
    }

    fn handle_mouse_button(
        &mut self,
        event_loop: &ActiveEventLoop,
        state: ElementState,
        button: MouseButton,
    ) {
        let scale = self.scale_factor as f32;
        if self.cursor.x as f32 / scale <= SIDEBAR_WIDTH {
            if state == ElementState::Pressed {
                self.handle_sidebar_click(event_loop, button);
            }
            return;
        }
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

    fn navigate_command(&mut self, previous: bool) {
        let Some(pane_id) = self.focused_pane_id() else {
            return;
        };
        let Some(session) = self.sessions.get(&pane_id) else {
            return;
        };
        let bottom = session
            .snapshot()
            .and_then(|snapshot| snapshot.rows.last().map(|row| row.line))
            .unwrap_or(0);
        let marks = session.marks();
        let mut lines: Vec<i32> = marks
            .iter()
            .filter(|mark| {
                matches!(
                    mark.kind,
                    crate::vt::ShellMarkKind::CommandFinished
                        | crate::vt::ShellMarkKind::CommandStart
                )
            })
            .map(|mark| mark.line)
            .collect();
        if lines.is_empty() {
            if let Some(snapshot) = session.snapshot() {
                lines = snapshot
                    .rows
                    .iter()
                    .filter(|row| {
                        let text = row.text.trim_end();
                        text.ends_with('$') || text.ends_with('#') || text.ends_with('❯')
                    })
                    .map(|row| row.line)
                    .collect();
            }
        }
        if lines.is_empty() {
            session.scroll(if previous { 10 } else { -10 });
        } else {
            let target = if previous {
                *lines.last().unwrap_or(&bottom)
            } else {
                *lines.first().unwrap_or(&bottom)
            };
            session.scroll(bottom - target);
        }
        self.window.request_redraw();
    }

    fn copy_command_output(&mut self) {
        let Some(pane_id) = self.focused_pane_id() else {
            return;
        };
        let Some(session) = self.sessions.get(&pane_id) else {
            return;
        };
        let marks = session.marks();
        let finished = marks
            .iter()
            .rev()
            .find(|mark| matches!(mark.kind, crate::vt::ShellMarkKind::CommandFinished));
        let Some(finished) = finished else {
            return;
        };
        let start = marks
            .iter()
            .rev()
            .find(|mark| {
                mark.line < finished.line
                    && matches!(
                        mark.kind,
                        crate::vt::ShellMarkKind::CommandStart
                            | crate::vt::ShellMarkKind::PromptStart
                    )
            })
            .map(|mark| mark.line)
            .unwrap_or(finished.line);
        let text = session.text_between(start, finished.line);
        if !text.is_empty() {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(text);
            }
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
        self.overlay_buffer
            .set_metrics(Metrics::new(12.0 * scale, 18.0 * scale));
        self.renders.clear();
        self.refresh_sidebar();
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
            x: SIDEBAR_WIDTH,
            y: TITLE_BAR,
            width: (width - SIDEBAR_WIDTH).max(1.0),
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

    fn refresh_sidebar(&mut self) {
        self.live_status = self
            .client
            .list_sessions()
            .unwrap_or_default()
            .into_iter()
            .map(|session| (session.id, session.status))
            .collect();
        self.saved_sessions = self
            .client
            .saved_sessions()
            .unwrap_or_default()
            .into_iter()
            .filter(|session| session.status != SessionStatus::Stopped)
            .collect();
        let mut lines: Vec<(String, Option<SidebarAction>)> = Vec::new();
        lines.push(("proyectos".to_owned(), None));
        for project in self.projects.iter().take(8) {
            let active = self
                .saved_sessions
                .iter()
                .filter(|session| {
                    session.spec.project_id == project.id
                        && session.status != SessionStatus::Crashed
                })
                .count();
            let glyphs = self
                .providers
                .get(&project.id)
                .map(|(spec, writer)| {
                    format!(" {}{}", provider_glyph(spec), provider_glyph(writer))
                })
                .unwrap_or_default();
            let marker = if active > 0 {
                format!(" ●{active}")
            } else {
                String::new()
            };
            lines.push((
                format!("▸ {}{}{}", project.name, marker, glyphs),
                Some(SidebarAction::SelectProject {
                    id: project.id.clone(),
                    root: project.root_path.clone(),
                }),
            ));
        }

        lines.push(("sesiones".to_owned(), None));
        for session in self.saved_sessions.iter().take(12) {
            let status = self
                .live_status
                .get(&session.id)
                .copied()
                .unwrap_or(session.status);
            let id = if session.id.len() > 8 {
                &session.id[..8]
            } else {
                &session.id
            };
            lines.push((
                format!(
                    "  {} {} · {id}",
                    status_glyph(status),
                    session.spec.driver_id
                ),
                Some(SidebarAction::AttachSession(session.id.clone())),
            ));
        }

        lines.push(("nuevo agente".to_owned(), None));
        let project_id = self.project_id.clone();
        let root = self.project_root.clone();
        for driver in &self.drivers {
            if driver.status != ancleto_code_core::drivers::ToolStatus::Available {
                continue;
            }
            lines.push((
                format!("  + {}", driver.display_name),
                Some(SidebarAction::NewAgent {
                    project_id: project_id.clone(),
                    root: root.clone(),
                    driver_id: driver.id.clone(),
                    program: driver.binary.clone(),
                }),
            ));
        }

        self.sidebar_lines = lines;
        let text = self
            .sidebar_lines
            .iter()
            .map(|(line, _)| line.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        self.sidebar_buffer.set_metrics(Metrics::new(
            11.0 * self.scale_factor as f32,
            16.0 * self.scale_factor as f32,
        ));
        self.sidebar_buffer.set_size(
            Some((SIDEBAR_WIDTH - 12.0) * self.scale_factor as f32),
            None,
        );
        set_buffer_text(&mut self.font_system, &mut self.sidebar_buffer, &text);
        self.last_sidebar_refresh = std::time::Instant::now();
    }

    fn handle_sidebar_click(&mut self, event_loop: &ActiveEventLoop, button: MouseButton) {
        let scale = self.scale_factor as f32;
        let x = self.cursor.x as f32 / scale;
        let y = self.cursor.y as f32 / scale;
        let height = self.surface_config.height as f32 / scale;
        if x > SIDEBAR_WIDTH || y < TITLE_BAR || y > height - STATUS_BAR {
            return;
        }
        let line = ((y - TITLE_BAR - PADDING) / 16.0).floor() as usize;
        let Some((_, Some(action))) = self.sidebar_lines.get(line) else {
            return;
        };
        let action = action.clone();
        match action {
            SidebarAction::SelectProject { id, root } => {
                self.project_id = id;
                self.project_root = root;
                if let Ok(pane) = self.create_session() {
                    self.layout.new_tab(self.profile.display_name.clone(), pane);
                    self.update_layout_geometry();
                }
            }
            SidebarAction::AttachSession(session_id) => {
                if button == MouseButton::Right {
                    self.modal = Some(Modal::KillSession(session_id));
                } else {
                    self.attach_session(session_id);
                }
            }
            SidebarAction::NewAgent {
                project_id,
                root,
                driver_id,
                program,
            } => {
                let spec = SessionSpec {
                    project_id,
                    driver_id: driver_id.clone(),
                    program,
                    args: Vec::new(),
                    cwd: root,
                    env: terminal_env(&[]),
                    cols: 100,
                    rows: 30,
                };
                if let Ok(info) = self.client.create_session(spec) {
                    let proxy = self.proxy.clone();
                    let wake: crate::session::Wake = Arc::new(move || {
                        let _ = proxy.send_event(UserEvent::Wakeup);
                    });
                    if let Ok(session) = TerminalSession::attach_with_wake(
                        Arc::clone(&self.client),
                        info.id.clone(),
                        100,
                        30,
                        Some(wake),
                    ) {
                        let pane = Pane::new(PaneKind::Agent, driver_id).with_session(info.id);
                        self.sessions.insert(pane.id.clone(), session);
                        self.layout.new_tab(pane.title.clone(), pane);
                        self.refresh_sidebar();
                        self.update_layout_geometry();
                        self.update_chrome();
                        self.window.request_redraw();
                    }
                }
            }
        }
        self.refresh_sidebar();
        let _ = event_loop;
        self.update_chrome();
        self.window.request_redraw();
    }

    fn attach_session(&mut self, session_id: String) {
        let mut found = None;
        for (index, tab) in self.layout.tabs.iter().enumerate() {
            for pane in tab.root.panes() {
                if pane.session_id.as_deref() == Some(session_id.as_str()) {
                    found = Some((index, tab.id.clone(), pane.id.clone()));
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }
        if let Some((index, tab_id, pane_id)) = found {
            self.layout.select_tab(index);
            self.layout.focus_pane(&tab_id, &pane_id);
            self.update_layout_geometry();
            self.update_chrome();
            self.window.request_redraw();
            return;
        }

        let Some(saved) = self
            .saved_sessions
            .iter()
            .find(|session| session.id == session_id)
            .cloned()
        else {
            return;
        };
        let proxy = self.proxy.clone();
        let wake: crate::session::Wake = Arc::new(move || {
            let _ = proxy.send_event(UserEvent::Wakeup);
        });
        if let Ok(session) = TerminalSession::attach_with_wake(
            Arc::clone(&self.client),
            session_id.clone(),
            100,
            30,
            Some(wake),
        ) {
            let pane =
                Pane::new(PaneKind::Agent, saved.spec.driver_id.clone()).with_session(session_id);
            self.sessions.insert(pane.id.clone(), session);
            self.layout.new_tab(pane.title.clone(), pane);
            self.update_layout_geometry();
            self.update_chrome();
            self.window.request_redraw();
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
        let exit = self.focused_pane_id().and_then(|pane_id| {
            self.sessions
                .get(&pane_id)
                .and_then(|session| session.last_exit_code())
        });
        let status = self.status.clone().unwrap_or_else(|| {
            let exit = exit
                .map(|code| format!(" · exit {code}"))
                .unwrap_or_default();
            let panels = if panes == 1 {
                "1 panel".to_owned()
            } else {
                format!("{panes} paneles")
            };
            format!(
                "{} · {} · {} pt{exit} · Ctrl+Shift+T nueva tab · Q salir",
                project, panels, self.font_size
            )
        });
        set_buffer_text(&mut self.font_system, &mut self.status_buffer, &status);
        self.window.set_title(&format!("AncletoCMD — {project}"));

        let modal = match &self.modal {
            Some(Modal::KillAll) => "¿Matar todas las sesiones y salir? [s/n]",
            Some(Modal::KillSession(_)) => "¿Terminar esta sesión? [s/n]",
            Some(Modal::ClosePane(_)) => "¿Cerrar el panel y terminar su sesión? [s/n]",
            Some(Modal::Clipboard(_)) => {
                "Un programa quiere escribir el portapapeles (OSC 52). ¿Permitir? [s/n]"
            }
            Some(Modal::Hyperlink(_)) => "¿Abrir el hyperlink OSC 8? [s/n]",
            None => "",
        };
        set_buffer_text(&mut self.font_system, &mut self.modal_buffer, modal);
    }

    fn close_session_panes(&mut self, session_id: &str) {
        let mut targets: Vec<(String, String)> = Vec::new();
        for tab in &self.layout.tabs {
            for pane in tab.root.panes() {
                if pane.session_id.as_deref() == Some(session_id) {
                    targets.push((tab.id.clone(), pane.id.clone()));
                }
            }
        }
        for (tab_id, pane_id) in targets {
            self.sessions.remove(&pane_id);
            self.layout.close_pane(&tab_id, &pane_id);
        }
        let _ = self.client.kill_session(session_id);
        if self.layout.tabs.is_empty() {
            if let Ok(pane) = self.create_session() {
                self.layout.new_tab(self.profile.display_name.clone(), pane);
            }
        }
        self.refresh_sidebar();
        self.update_layout_geometry();
        self.update_chrome();
        self.window.request_redraw();
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
        self.client.saved_sessions().ok();
        self.refresh_sidebar();
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
        if self.last_sidebar_refresh.elapsed().as_secs() >= 2 {
            self.refresh_sidebar();
        }
        self.update_layout_geometry();
        self.update_pane_buffers();
        self.detect_activity();
        if self.overlay.is_some() {
            self.update_overlay_buffer();
        }
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

        rects.push(Rect {
            x: 0.0,
            y: TITLE_BAR * scale,
            width: SIDEBAR_WIDTH * scale,
            height: (height / scale - TITLE_BAR - STATUS_BAR).max(0.0) * scale,
            color: rgba(self.theme.background, 0.96),
        });
        rects.push(Rect {
            x: SIDEBAR_WIDTH * scale,
            y: TITLE_BAR * scale,
            width: 1.0,
            height: (height / scale - TITLE_BAR - STATUS_BAR).max(0.0) * scale,
            color: rgba(self.theme.border, 1.0),
        });
        let overlay_geometry = if self.overlay.is_some() && self.modal.is_none() {
            let width_l = width / scale;
            let height_l = height / scale;
            let panel_width = (width_l * 0.8).min(900.0);
            let panel_height = (height_l * 0.75).min(620.0);
            let panel_x = (width_l - panel_width) / 2.0;
            let panel_y = (height_l - panel_height) / 2.0;
            rects.push(Rect {
                x: 0.0,
                y: 0.0,
                width,
                height,
                color: [0.0, 0.0, 0.0, 0.6],
            });
            rects.push(Rect {
                x: panel_x * scale,
                y: panel_y * scale,
                width: panel_width * scale,
                height: panel_height * scale,
                color: rgba(self.theme.background, 0.98),
            });
            Some((panel_x, panel_y, panel_width, panel_height))
        } else {
            None
        };
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
        if let Some((panel_x, panel_y, panel_width, panel_height)) = overlay_geometry {
            areas.push(TextArea {
                buffer: &self.overlay_buffer,
                left: (panel_x + 12.0) * scale,
                top: (panel_y + 10.0) * scale,
                scale: 1.0,
                bounds: TextBounds {
                    left: ((panel_x + 8.0) * scale) as i32,
                    top: ((panel_y + 6.0) * scale) as i32,
                    right: ((panel_x + panel_width - 8.0) * scale) as i32,
                    bottom: ((panel_y + panel_height - 6.0) * scale) as i32,
                },
                default_color: Color::rgb(
                    self.theme.foreground.0,
                    self.theme.foreground.1,
                    self.theme.foreground.2,
                ),
                custom_glyphs: &[],
            });
        }
        areas.push(TextArea {
            buffer: &self.sidebar_buffer,
            left: 6.0 * scale,
            top: (TITLE_BAR + PADDING) * scale,
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: (TITLE_BAR * scale) as i32,
                right: (SIDEBAR_WIDTH * scale) as i32,
                bottom: (height - STATUS_BAR * scale) as i32,
            },
            default_color: Color::rgb(
                self.theme.foreground.0,
                self.theme.foreground.1,
                self.theme.foreground.2,
            ),
            custom_glyphs: &[],
        });
        if overlay_geometry.is_none() {
            for pane in &text_areas {
                if let Some(render) = self.renders.get(&pane.pane_id) {
                    let line_height = self.cell_height * self.scale_factor as f32;
                    for (index, line) in render.lines.iter().enumerate() {
                        let top = pane.top + index as f32 * line_height;
                        if top >= pane.bottom as f32 {
                            break;
                        }
                        areas.push(TextArea {
                            buffer: &line.buffer,
                            left: pane.left,
                            top,
                            scale: 1.0,
                            bounds: TextBounds {
                                left: pane.left as i32,
                                top: top as i32,
                                right: pane.right,
                                bottom: (top + line_height).min(pane.bottom as f32) as i32,
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
            let screen_lines = session.lines();
            let Some(snapshot) = session.snapshot() else {
                continue;
            };
            let scale = self.scale_factor as f32;
            let width = (pane.width - PADDING * 2.0) * scale;
            let height = (pane.height - PADDING * 2.0) * scale;
            let line_height = self.cell_height * scale;
            let desired: Vec<String> = snapshot.rows.iter().map(|row| row.text.clone()).collect();

            if !self.renders.contains_key(&pane.pane_id) {
                self.renders.insert(
                    pane.pane_id.clone(),
                    PaneRender {
                        lines: Vec::new(),
                        revision: u64::MAX,
                        columns: 0,
                        screen_lines: 0,
                        width: 0.0,
                        height: 0.0,
                        selection: None,
                    },
                );
            }
            let Some(entry) = self.renders.get_mut(&pane.pane_id) else {
                continue;
            };
            let resized = (entry.width - width).abs() > 0.5 || (entry.height - height).abs() > 0.5;
            let metrics = Metrics::new(self.font_size * scale, line_height);

            while entry.lines.len() < desired.len() {
                let mut buffer = Buffer::new(&mut self.font_system, metrics);
                buffer.set_wrap(glyphon::Wrap::None);
                buffer.set_size(Some(width), Some(line_height));
                entry.lines.push(LineRender {
                    text: String::new(),
                    buffer,
                });
            }
            entry.lines.truncate(desired.len());

            let mut changed = false;
            for (index, text) in desired.iter().enumerate() {
                let line = &mut entry.lines[index];
                if line.text == *text && !resized {
                    continue;
                }
                line.buffer.set_metrics(metrics);
                line.buffer.set_size(Some(width), Some(line_height));
                line.buffer.set_text(
                    text,
                    &Attrs::new().family(Family::Name("IBM Plex Mono")),
                    Shaping::Basic,
                    None,
                );
                line.buffer.shape_until_scroll(&mut self.font_system, false);
                line.text = text.clone();
                changed = true;
            }
            if changed || resized {
                entry.revision = revision;
            }
            entry.columns = columns;
            entry.screen_lines = screen_lines;
            entry.width = width;
            entry.height = height;
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

fn next_pane_id(layout: &TerminalLayout, tab_id: &str, from: &str) -> Option<String> {
    let tab = layout.tab(tab_id)?;
    let ids = tab.pane_ids();
    if ids.is_empty() {
        return None;
    }
    let position = ids.iter().position(|id| id == from).unwrap_or(0);
    Some(ids[(position + 1) % ids.len()].clone())
}

fn tool_label(tool: &ToolReport) -> &'static str {
    match tool.status {
        ancleto_code_core::drivers::ToolStatus::Available => "available",
        ancleto_code_core::drivers::ToolStatus::Missing => "missing",
        ancleto_code_core::drivers::ToolStatus::Incompatible => "incompatible",
        ancleto_code_core::drivers::ToolStatus::Error => "error",
    }
}

fn summarize_fact(value: &serde_json::Value) -> String {
    if let Some(array) = value.as_array() {
        return array.len().to_string();
    }
    if let Some(object) = value.as_object() {
        if let Some(array) = object.values().find_map(|value| value.as_array()) {
            return array.len().to_string();
        }
        return "·".to_owned();
    }
    value.to_string()
}

fn conversation_state_label(state: ConversationState) -> &'static str {
    match state {
        ConversationState::Running => "running",
        ConversationState::Paused => "paused",
        ConversationState::Completed => "completed",
        ConversationState::Aborted => "aborted",
        ConversationState::Failed => "failed",
    }
}

fn shift_choice(view: &mut ConversationView, delta: i32) {
    let ids: Vec<String> = view.sessions.iter().map(|entry| entry.0.clone()).collect();
    if ids.is_empty() {
        return;
    }
    let target = if view.focus == 0 {
        &mut view.session_a
    } else {
        &mut view.session_b
    };
    let current = target
        .as_ref()
        .and_then(|id| ids.iter().position(|candidate| candidate == id))
        .unwrap_or(0) as i32;
    let next = (current + delta).rem_euclid(ids.len() as i32) as usize;
    *target = Some(ids[next].clone());
}

fn terminal_env(extra: &[(String, String)]) -> Vec<(String, String)> {
    let mut env = vec![
        ("TERM".to_owned(), "xterm-256color".to_owned()),
        ("COLORTERM".to_owned(), "truecolor".to_owned()),
    ];
    for (key, value) in extra {
        if key != "TERM" && key != "COLORTERM" {
            env.push((key.clone(), value.clone()));
        }
    }
    env
}

fn provider_glyph(tool: &ToolReport) -> &'static str {
    match tool.status {
        ancleto_code_core::drivers::ToolStatus::Available => "●",
        ancleto_code_core::drivers::ToolStatus::Missing => "○",
        ancleto_code_core::drivers::ToolStatus::Incompatible => "!",
        ancleto_code_core::drivers::ToolStatus::Error => "×",
    }
}

fn status_glyph(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Starting => "◌",
        SessionStatus::Ready => "●",
        SessionStatus::Busy => "◐",
        SessionStatus::WaitingInput => "◑",
        SessionStatus::Detached => "○",
        SessionStatus::Stopped => "·",
        SessionStatus::Crashed => "×",
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
        &Attrs::new().family(Family::Name("IBM Plex Mono")),
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
        &Attrs::new().family(Family::Name("IBM Plex Mono")),
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
