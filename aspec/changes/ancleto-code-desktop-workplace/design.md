# Design: AncletoCode — workspace de agentes para escritorio y terminal

## Approach

AncletoCode se organiza como un **Cargo workspace** con un núcleo compartido y **dos versiones de producto** más una superficie de automatización:

1. **Núcleo en Rust** (`crates/ancleto-code-core`): dueño de todo lo que toca el sistema operativo — proyectos en disco, procesos PTY, detección de binarios, lectura de artefactos, ejecución de las CLIs de Ancleto y persistencia. Expone un modelo tipado y eventos.
2. **Host de sesiones** (`crates/ancleto-code-host`): proceso local que posee los PTY y los expone por IPC a todos los clientes. Permite detach/attach y sesiones compartidas entre versiones.
3. **Versiones y superficie**:
   - **AncletoCode (escritorio)**: `crates/ancleto-code-tauri`, ventana Tauri con frontend React 19 + TypeScript + Vite.
   - **AncletoCMD (terminal)**: `crates/ancleto-code-cmd`, emulador de terminal nativo con `winit` + `wgpu`/`glyphon` y parsing VT con `alacritty_terminal`/`vte`, que además funciona como gestor de proyectos y sesiones.
   - **CLI headless**: `crates/ancleto-code-cli`, comandos `clap` para scripts, CI y administración.

Reglas estructurales:

- **Un solo núcleo, tres consumidores.** Ninguna regla de negocio se implementa dos veces. La app Tauri, AncletoCMD y la CLI son consumidores delgados del core y del host.
- **AncletoCMD es un emulador, no una TUI.** Tiene ventana propia y render GPU; no se ejecuta dentro de otra terminal ni depende de un webview.
- **El PTY es la fuente de verdad** de cada sesión de agente y de cada shell. Los drivers describen arranque y señales; si el parseo falla, la UI cae a terminal cruda sin romper la sesión.
- **Las CLIs de Ancleto son la fuente de verdad de sus datos.** La app no reimplementa memoria, discovery, grafo ni frescura: invoca `ancleto` y `ancleto-writer` y lee el contrato publicado (`.ancletorc`, `.ancleto/memory.db` vía CLI, `.ancleto/writer/manifest.json` + `check`).
- **Idioma:** el idioma natural de artefactos es de `@ancleto/spec` (`.ancletorc` → `language`). El "language pack" de writer es de **lenguajes de programación** y se trata como dependencia opcional.
- **La CLI es headless por diseño.** No depende del webview, de `winit` ni de Tauri; se puede ejecutar por SSH, en un servidor Linux o en un job de CI.


## Architecture

```
┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐
│ Desktop (React)     │  │ AncletoCMD          │  │ CLI (clap)          │
│ Sidebar · Tabs      │  │ winit · wgpu        │  │ run · session · ... │
│ Panels · Settings   │  │ tabs · splits       │  │ --json · CI         │
└──────────▲──────────┘  └──────────▲──────────┘  └──────────▲──────────┘
           │ Tauri IPC              │ JSON-RPC               │ JSON-RPC
┌──────────┴──────────┐  ┌──────────┴──────────┐  ┌──────────┴──────────┐
│ ancleto-code-tauri  │  │ ancleto-code-cmd    │  │ ancleto-code-cli    │
│ commands · events   │  │ renderer · layout   │  │ commands · pipeline │
└──────────▲──────────┘  └──────────▲──────────┘  └──────────▲──────────┘
           └───────────────┬─────────┴────────────────────────┘
                           │ IPC local (named pipe / Unix socket + token)
                 ┌─────────┴──────────┐
                 │ ancleto-code-host  │  dueño de PTY · ring buffers
                 │ session registry   │  attach/detach · reconciliación
                 └─────────┬──────────┘
                           │
                 ┌─────────┴──────────────────────────────────────┐
                 │ ancleto-code-core                              │
                 │  domain · projects · drivers · orchestrator    │
                 │  providers (spec/writer/node) · persistence    │
                 │  SQLite WAL · PTY · child processes            │
                 └────────────────────────────────────────────────┘
```

### Modelo de dominio

- `Project { id, name, rootPath, createdAt, lastOpenedAt }`
- `AgentSession { id, projectId, driverId, cwd, status, mode, ownerHostId, startedAt, exitCode? }`
  - `status ∈ { starting, ready, busy, waiting_input, detached, stopped, crashed }`
  - `mode ∈ { interactive, oneShot }`
  - `attachedClients[] ∈ { desktop, cmd, cli }` (varios clientes pueden mirar la misma sesión)
- `AgentDriverDescriptor { id, displayName, binary, capabilities, detect }`
  - `capabilities ∈ { interactivePty, oneShotPrompt, structuredOutput, resumeSession, headlessJson }`
- `Conversation { id, participants[sessionA, sessionB], objective, maxTurns, turnBudgetMs, state, turns[] }`
- `SpecProjectState { initialized, cliVersion, host, tier, language, discovery: {status, impact}, activeChanges[], memoryRulesCount }`
- `WriterState { status, cliVersion, schemaVersion, freshness, stats, languagePack: {installed, enabled, languages[]} }`
- `ToolStatus { available | missing | incompatible | error, version?, binaryPath?, installHint }`
- `SessionHost { id, pid, endpoint, tokenPath, protocolVersion, startedAt, idleTimeoutMs }`
- `TerminalPane { id, projectId, sessionId?, kind: shell | agent, shell?: {program, args}, scrollbackLines, title }`
- `TerminalLayout { projectId, tabs[], panes[{ paneId, split, focused, size }] }`
- `ShellProfile { id, displayName, program, args, env, oscIntegration: bool }`

### Host local de sesiones

El host es lo que permite elegir superficie sin perder el trabajo en curso.

- **Propiedad**: un único host por usuario; es el dueño de los `portable-pty`, los ring buffers y el parser de señales. Los clientes no crean PTY, sólo piden operaciones.
- **Transporte**: JSON-RPC 2.0 con framing por longitud sobre named pipe `\\.\pipe\ancleto-code-<hash>` en Windows y Unix domain socket en `$XDG_RUNTIME_DIR/ancleto-code/host.sock` (0700) en Linux/macOS, con fallback a `~/.local/state/ancleto-code/`.
- **Seguridad local**: token aleatorio por instalación en archivo `0600`; el cliente debe presentarlo en el handshake. Sin TCP, sin escucha en red, sin autenticación remota.
- **Autostart**: los clientes arrancan el host si no responde (con lock file para evitar carreras), y el host se apaga por inactividad configurable sólo si no hay sesiones activas.
- **Métodos**: `session.create|write|resize|kill|snapshot|list|attach|detach`, `project.*`, `conversation.*`, `providers.*`, `host.status`. Eventos: `session.output`, `session.state`, `session.exit`, `conversation.turn`.
- **Detach/attach**: al desconectarse un cliente, la sesión pasa a `detached` pero el PTY sigue vivo; al reconectar se reenvía el snapshot del ring buffer y el stream continúa.
- **Caída del host**: los PTY mueren con el proceso; al arrancar, el host reconcilia las sesiones `starting|ready|busy|waiting_input|detached` como `crashed` y ofrece recrearlas con el mismo `cwd` y driver.
- **Compatibilidad**: `protocolVersion` en el handshake; cliente y host de versiones distintas fallan con mensaje explícito y `acode doctor` indica cómo actualizar.
- **Fallback**: si el host no está disponible, `acode run` y los comandos one-shot pueden operar in-process con `--no-host` (sin attach ni persistencia de sesión interactiva).

### Motor de sesiones (PTY)

- `portable-pty` con `cwd` del proyecto y `CommandSpec` provisto por el driver; ConPTY en Windows y PTY nativo en Linux/macOS.
- Hilo lector por sesión con dos consumidores: ring buffer (rehidratación) y parser opcional de señales.
- Comandos expuestos por el host: `session_create`, `session_write`, `session_resize`, `session_kill`, `session_snapshot`.
- Eventos del host: `session.output`, `session.state`, `session.exit`; la app Tauri los reemite como `session://output/{id}`, `session://state/{id}`, `session://exit/{id}`.
- Cierre ordenado de las sesiones al apagar el host; sin procesos huérfanos. Cerrar AncletoCMD o la app no mata sesiones: sólo hace detach.

### Drivers

```rust
trait AgentDriver: Send + Sync {
    fn descriptor(&self) -> AgentDriverDescriptor;
    fn detect(&self) -> DetectionResult;
    fn spawn(&self, ctx: &SpawnContext) -> CommandSpec;          // modo interactivo
    fn one_shot(&self, ctx: &SpawnContext, prompt: &str) -> Option<CommandSpec>;
    fn parse(&self, chunk: &[u8], st: &mut ParseState) -> Vec<AgentSignal>;
}
```

| Driver | Binario | Interactivo | One-shot | Notas |
|---|---|---|---|---|
| `claude` | `claude` | sí (TUI) | sí (`-p`, salida JSON) | detección `claude --version` |
| `opencode` | `opencode` | sí (TUI) | sí (`run`) | detección `opencode --version` |
| `agy` | `agy` | sí (TUI) | sí (`agy -p ... --output-format json`) | binario Go, auto-updater propio; `--continue`/`--conversation` para continuidad |
| `cmd` | `cmd` (`command-code`, `commandcode`, `cmdc`) | sí (TUI Ink) | sí (`cmd -p --output-format json`, NDJSON) | paquete npm `command-code`, Node ≥ 22; `--continue`/`--resume`, `--max-turns`, `--yolo` |

- `agy` es el **Antigravity CLI**; no existe un segundo CLI `antigravity`.
- `cmd` es **Command Code**; su paquete npm es `command-code` y expone los bins `cmd`, `cmdc`, `commandcode` y `command-code`.
- **Antigravity IDE** se modela como `ide/antigravity`: acción "abrir proyecto en Antigravity" y sincronización de assets de host (`@ancleto/spec --agent antigravity` → `.agents/skills`, `.agents/agents`, `.agents/mcp_config.json`). No crea sesión PTY.
- Registry de drivers; sumar uno nuevo no toca el frontend.
- Fallback `raw` si el parseo no reconoce la salida.

### Conversación agente-a-agente

- Dos estrategias, elegidas por capacidad del driver:
  1. **One-shot estructurado** (preferido): cada turno es un proceso one-shot con salida JSON cuando el driver lo soporta (`agy --output-format json`, `claude -p --output-format json`, `cmd -p --output-format json` — NDJSON con frame final `result`), y continuidad por `resume`/`continue`/`conversation id` cuando está disponible.
  2. **Inyección PTY** (fallback): escritura del mensaje en la sesión interactiva y espera de quiescencia + señal `Ready`.
- `ConversationOrchestrator` mantiene la máquina de estados: envía a A → espera fin de turno → extrae mensaje → inyecta en B → repite.
- Guardrails: `maxTurns` (default 6), timeout por turno, detección de turnos idénticos consecutivos, aborto manual, pausa/reanudación.
- Turnos persistidos con origen, destino, texto, modo, duración y timestamps.
- El panel muestra las dos voces y el motivo de finalización.

### Proveedor `@ancleto/spec`

`AncletoSpec` encapsula la CLI `ancleto` (alias `aspec`), Node ≥ 24 y cero dependencias.

- **Resolución del binario**: `node_modules/.bin/ancleto` del proyecto → global → sidecar empaquetado.
- **Comandos de proyecto**: `init` (`--agent`, `--tier`, `--lang`, `--exclude`, `--profile`), `install --project`, `update`, `upgrade`, `check`, `doctor`, `specs check --json`.
- **Lectura de estado**: `.ancletorc` (host, tier, `language`, `discovery.exclude`), `aspec/changes` (changes activos), `.ancleto/working-context.md`, `.ancleto/memory.db` **sólo vía CLI** (`ancleto memory list --json`, `ancleto memory context`).
- **Discovery**: `ancleto discovery --check` para `READY/STALE/PARTIAL/MISSING` e `impact` (`none|minor|material`); la UI ofrece regenerar sólo con aprobación explícita.
- **Hosts soportados por spec**: `opencode` (default), `claude`, `vscode`, `antigravity`, `cursor`, `roo`, `copilot`, `commandcode`. El host se elige por proyecto y determina dónde se materializan skills/agents/commands.
- **Tiers**: `normal`, `minimo`, `gratis`, visibles en Settings del proyecto.
- **MCP**: la app muestra el estado de `ancleto-memory` y de los MCP configurados por host; no levanta el servidor MCP por su cuenta.
- La app **no reescribe** `aspec/`, `AGENTS.md`, `PRODUCT.md` ni `memory.db`: usa los comandos de la CLI.

### Proveedor `@ancleto/writer`

`AncletoWriter` encapsula `ancleto-writer` (Node ≥ 24, core sin dependencias de runtime).

- **Resolución**: local del proyecto → global → sidecar.
- **Comandos**: `init`, `build`, `check`, `lint`, `impact`, `human`, `doctor`; todos los que exponen `--json` se consumen en JSON.
- **Frescura**: se usa `ancleto-writer check --json` como autoridad; la app **no recalcula** el fingerprint ni el schema. Si `schemaVersion` es incompatible, el panel degrada a "conocimiento no disponible".
- **Panel de conocimiento** (lectura de `.ancleto/writer/`): `manifest.json` (versión, commit, stats, hashes), `graph.json` (nodos/aristas), `facts/*` (cycles, bridges, communities, god-nodes, layers, hotspots, blast-radius, dead-code, doc-coverage, unsupported).
- **Vista humana**: ejecuta `ancleto-writer human` y muestra `human/index.html` / `human/interactive.html` embebidos vía protocolo de assets local (sin red, sin CDN); alternativa: abrir en navegador.
- **Documentos para agentes**: expone `agent/INDEX.md`, `API.md`, `SYMBOLS.md`, `GOTCHAS.md`, `ARCHITECTURE.md` con acción "adjuntar al contexto".
- **Language pack**: detecta `@ancleto/writer-languages` (11 lenguajes vía tree-sitter, versión independiente). Estado `installed/absent`, toggle `languagePack.enabled` y `module` en `.ancleto-writer.json`. Si falta o falla, el core degrada a JS/TS y lo reporta en `facts/unsupported.json`; la app sugiere `npm install @ancleto/writer @ancleto/writer-languages` y advierte sobre peers con `--legacy-peer-deps`.
- **Idioma**: writer no tiene idioma natural; no se expone ningún selector de idioma en su panel.

### Integración con el IDE Antigravity

- Acción de proyecto `open_in_antigravity` usando el binario/instalador de Antigravity.
- Sincronización de assets con `@ancleto/spec --agent antigravity` (`.agents/skills/<n>/SKILL.md`, `.agents/agents/<n>.md`, `.agents/mcp_config.json` no destructivo).
- No se intenta embeber la app Antigravity: se delega en el IDE instalado.

### CLI `acode`, AncletoCMD y empaquetado npm

#### Workspace

```text
crates/
  ancleto-code-core/     # lib: domain, pty, drivers, orchestrator, providers, persistence
  ancleto-code-host/     # bin: local session host (JSON-RPC over pipe/socket)
  ancleto-code-cli/      # bin: clap command tree (headless)
  ancleto-code-cmd/      # bin: AncletoCMD terminal emulator/manager (winit + wgpu)
  ancleto-code-tauri/    # bin: Tauri desktop shell + React assets
packages/
  acode/                 # npm launcher @ancleto/code (bin: acode, ancleto-code)
  acmd/                  # npm launcher @ancleto/cmd  (bin: acmd, ancleto-cmd)
  acmd-win32-x64/        # binario + checksum, optionalDependency
  acmd-win32-arm64/
  acmd-linux-x64-gnu/
  acmd-linux-arm64-gnu/
  acmd-darwin-x64/
  acmd-darwin-arm64/
  acode-win32-x64/       # binarios de la CLI (+ host)
  acode-win32-arm64/
  acode-linux-x64-gnu/
  acode-linux-arm64-gnu/
  acode-linux-x64-musl/
  acode-linux-arm64-musl/
  acode-darwin-x64/
  acode-darwin-arm64/
```

- **Dependencias permitidas**: `core` no conoce `host`, `cli`, `cmd` ni `tauri`; `host` depende de `core`; `cli` y `cmd` dependen de `core` + cliente IPC; `tauri` depende de `core` + cliente IPC. Un test de CI falla si aparece una dependencia inversa o si `core` importa `tauri`/`winit`/`wgpu`.
- **Binarios**: `acode` (CLI + host auxiliar) y `acmd` (AncletoCMD) son separados para que la terminal no arrastre dependencias gráficas en un servidor y para que la CLI sea pequeña. El host se empaqueta con ambos.
- **Escritorio**: la app Tauri se distribuye con instaladores (MSI/NSIS, DMG, AppImage/DEB) y no depende de npm.

#### Contrato de comandos (`acode`)

```text
acode                         # ayuda (no abre GUI)
acode run "<prompt>" [--driver D] [--project P] [--json] [--no-input]
acode project add <path> | list | remove <id> | open <id>
acode agents [--json]
acode session start --project P --driver D [--json]
acode session list [--json] | attach <id> | send <id> "<text>" | kill <id>
acode converse --a <id> --b <id> --objective "..." [--max-turns N] [--json]
acode spec  <init|install|update|upgrade|check|doctor|specs-check|discovery|memory> [...]
acode writer <init|build|check|lint|impact|human|doctor> [...]
acode host start | stop | status
acode doctor [--json]
acode ui                      # abrir la app de escritorio instalada
```

- **Salida**: texto legible por defecto; `--json` emite un sobre `{ "schemaVersion": 1, "ok": bool, "data": ..., "error": ... }` por stdout y deja logs en stderr. Sin ANSI si `--json`, `--no-input`, `NO_COLOR` o `stdout` no es TTY.
- **Códigos de salida**: `0` ok, `2` uso inválido, `3` dependencia ausente (`node`, driver, provider), `4` error de sesión/host, `5` estado incompatible (schema/protocolo), `6` cancelado por el usuario.
- **Señales**: `Ctrl+C` en un comando one-shot cancela el proceso hijo y devuelve `6`; `SIGTERM` cierra ordenadamente y hace detach sin matar sesiones del host.
- **Rutas de estado**: `%APPDATA%\AncletoCode` y `%LOCALAPPDATA%\AncletoCode` en Windows; `$XDG_CONFIG_HOME/ancleto-code` y `$XDG_STATE_HOME/ancleto-code` en Linux; `~/Library/Application Support/AncletoCode` en macOS. Override con `ANCLETO_CODE_HOME` y `ANCLETO_CODE_STATE`.
- **Windows**: los argumentos se pasan como argv al proceso, nunca por una shell intermedia; se habilita VT/UTF-8 cuando la consola lo permite y se prueba en `cmd.exe` y PowerShell 5.1/7. npm genera los shims `acode.cmd`/`acode.ps1`.
- **Linux/macOS**: build musl estático para las variantes `-musl`; sin dependencia de Node, webkit ni GPU. Clipboard por OSC 52 cuando no hay integración nativa.

#### Distribución npm

- `packages/acode/package.json`: `bin: { "acode": "bin/acode.js", "ancleto-code": "bin/acode.js" }`, `optionalDependencies` por plataforma y `engines.node >= 18` sólo para el launcher.
- `packages/acmd/package.json`: `bin: { "acmd": "bin/acmd.js", "ancleto-cmd": "bin/acmd.js" }`, `optionalDependencies` con los binarios de AncletoCMD.
- **Launcher** (`bin/acode.js` / `bin/acmd.js`): detecta `process.platform` + `process.arch` + libc (`musl`/`gnu`), resuelve el paquete instalado, verifica que el binario exista y sea ejecutable, y lo lanza con `spawn` heredando stdio, señales y código de salida. No implementa lógica de producto.
- Si el paquete opcional falta: mensaje accionable con plataforma detectada, comando de instalación manual y enlace a releases; `acode doctor` reporta la causa.
- **Integridad**: cada paquete de plataforma publica `checksums.json`; el launcher verifica el hash antes de ejecutar y falla con código `5` si no coincide.
- **Versión única**: la versión del workspace se propaga a todos los paquetes en el publish; un test compara `acode --version` y `acmd --version` con los `package.json`.
- **Publicación**: CI construye la matriz en runners nativos (Windows, Ubuntu gnu y musl, macOS x64/arm64) y publica con `npm publish --provenance`; sin auto-update en runtime.
- **Licencias en el paquete**: incluir `LICENSE`, `THIRD_PARTY_NOTICES.md` y la declaración de fork de PandaCode en ambos paquetes.

### AncletoCMD — emulador de terminal y gestor

AncletoCMD es la **versión terminal** de AncletoCode: una terminal real, rápida y GPU-acelerada, con la organización por proyectos y agentes de un gestor. La referencia de experiencia es Warp; la referencia de rendimiento es Alacritty/Kitty.

#### Stack y arquitectura interna

- **Ventana y eventos**: `winit` con bucle de eventos propio, DPI por monitor y soporte de multi-ventana opcional (una ventana = un workspace).
- **Render**: `wgpu` para compositing y `glyphon` (cosmic-text) para shaping/rasterizado de texto; fallback a render por software cuando no hay adaptador compatible o la GPU falla.
- **Motor VT**: `alacritty_terminal` (`Term` + `vte::Parser`) alimentado por el stream del host. No se escribe un parser de escape propio.
- **PTY**: `portable-pty` en el host (ConPTY en Windows, PTY en Linux/macOS). AncletoCMD es cliente del host, igual que la app y la CLI.
- **Modelo de ventana**: árbol de layout con tabs y splits; cada panel referencia un `TerminalPane` (shell o agente) y su `sessionId`.
- **Salida a pantalla**: daño parcial por frame, sin repintar la grilla completa; coalescing de output del host por frame.
- **Objetivos de rendimiento**: p99 de tecla a frame < 30 ms, idle < 1 % de CPU, arranque < 300 ms, scrollback de 10.000 líneas por defecto configurable.

#### Terminal

- **Emulación**: secuencias ANSI/VT, 16/256/truecolor, alt screen, scroll regions, cursor shapes, mouse reporting, bracketed paste, focus events, title, bell y OSC 8 hyperlinks.
- **Texto**: wide chars, emoji (fuente fallback), combining marks, bidi best-effort, ligaduras opcionales y reflow al redimensionar.
- **Fuentes**: JetBrains Mono o IBM Plex Mono empaquetada, cadena de fallback del sistema, tamaños y zoom por ventana.
- **Scrollback**: buffer circular eficiente, búsqueda incremental con resaltado, selección por carácter/palabra/línea y copia con OSC 52 o clipboard nativo.
- **Entrada**: IME en Windows/macOS/Linux, dead keys, layouts no latinos, pegado con bracketed paste y atajos configurables.
- **Seguridad**: OSC 52 pide confirmación, hyperlinks requieren modificador, títulos saneados y ningún escape ejecuta comandos.

#### Gestor de proyectos y agentes

- **Sidebar de proyectos**: proyectos ordenados por recencia, indicador de sesiones activas, estado de providers y apertura de una terminal en la raíz del proyecto.
- **Sesiones de agente**: creación con selector de driver (disponibles/`missing`), attach/detach, estado en vivo (`ready`, `busy`, `waiting_input`, `crashed`) y las mismas reglas de conversación agente-a-agente que la app.
- **Tabs y splits**: tabs por proyecto, splits horizontales/verticales, mover/redimensionar, cambiar foco y restaurar layout (`TerminalLayout`) al reabrir.
- **Command palette** (`Ctrl+Shift+P`): abrir proyecto, nueva terminal, nueva sesión de agente, cambiar tema, buscar en scrollback, ver estado de spec/writer.
- **Paneles laterales**: spec (cambios, memoria, discovery, `specs check`, `doctor`), writer (stats, frescura, hechos) y conversación, sin salir de la terminal.
- **Integración de shell**: hooks opt-in que emiten OSC 133 (`A` prompt, `B` inicio de comando, `C` fin de comando, `D` exit status) para bash, zsh, fish y PowerShell; navegación entre comandos, copia de la salida de un comando y marcador de error en la gutter. En `cmd.exe` y shells sin hooks se usa detección heurística de prompt y se degrada sin romper nada.
- **Historial**: búsqueda en historial de shell y comandos recientes; sugerencias basadas en historial local, sin IA ni envío de datos.

#### Ventanas y experiencia

- **Atajos**: nueva tab, cerrar tab, split vertical/horizontal, foco siguiente, command palette, búsqueda, zoom de fuente y detach. Todos configurables y con documentación por plataforma.
- **Notificaciones**: campana visual, marca en tab y notificación del sistema cuando un comando largo termina con la ventana en segundo plano.
- **Temas**: manifiestos JSON compartidos con la app; AncletoCMD sólo usa paleta y tipografía (sin transparencias ni wallpapers). Incluye presets accesibles y contraste AA.
- **Cerrar la ventana**: hace detach; las sesiones siguen en el host. "Cerrar y matar sesiones" existe con confirmación explícita.
- **Windows**: ConPTY, `cmd`, PowerShell 5.1/7 y WSL como `ShellProfile`; manejo de `Ctrl+C` y cierre sin matar sesiones. Sin dependencia de Windows Terminal.
- **Linux/macOS**: bash/zsh/fish, Wayland y X11, HiDPI, clipboard nativo y fallback OSC 52. Builds glibc para la GUI.
- **Accesibilidad**: contraste, zoom, navegación por teclado completa y compatibilidad limitada con lectores de pantalla documentada; la CLI sigue siendo la vía accesible para automatización.

#### Paridad y límites

- AncletoCMD consume los mismos casos de uso del core que la app: sesiones, drivers, conversación, providers y persistencia. No reimplementa permisos, límites ni lectura de artefactos.
- **Licencias**: el emulador no reutiliza código de PandaCode; usa crates de terceros (`alacritty_terminal`/`vte`, `winit`, `wgpu`, `glyphon`, `cosmic-text`) con sus propias licencias, auditadas y registradas en `THIRD_PARTY_NOTICES.md`.
- Fuera del MVP: bloques con acciones por comando (más allá de OSC 133), workflows, autocomplete con IA, compartir bloques, modo agente propio y sincronización en la nube.

### Referencia PandaCode, reutilización y cumplimiento de licencia

PandaCode se usa como **referencia de comportamiento y como fuente de código reutilizable**. El upstream se fija en el commit `e9f89891fd23518848765aef8219dbf39feac6e0` (v0.3.13). La regla es: **todo lo que mapea a una feature del MVP se reutiliza** si el módulo es razonablemente desacoplado; lo que pertenece a subsistemas fuera del MVP queda como referencia.

#### Política de reutilización

- **Nivel A — adoptar**: módulos autocontenidos que se copian casi textuales, con cabecera SPDX, commit de origen y tests portados.
- **Nivel B — adaptar**: módulos valiosos pero acoplados a eventos, stores o providers de PandaCode; se copian, se adaptan a nuestro contrato y se registra el diff.
- **Nivel C — referencia**: patrones de esquema, migraciones, capas de protocolo, orquestación y subsistemas fuera del MVP; se leen y se citan, no se copian.
- **Prohibido**: copiar activos (wallpapers, logos, screenshots, iconos, fuentes, temas), textos de documentación o marca; usar código sin cabecera, sin entrada en el ledger o sin aviso de licencia.
- **Fuente única**: el código reutilizado vive bajo `third_party/pandacode/` con la ruta original; nuestro código lo importa o lo envuelve, nunca lo edita sin registrar el diff.

#### Inventario de reutilización

| Módulo upstream | Ruta upstream | Uso en AncletoCode | Estrategia | Dependencias / notas |
|---|---|---|---|---|
| Windows Job Objects | `src-tauri/src/providers/windows_job.rs` | Matar el árbol de procesos de cada PTY y one-shot en Windows | A: casi textual, con tests | crate `windows` (`Win32_System_JobObjects`); `Drop` termina descendientes |
| Reconciliación de streams | `src-tauri/src/providers/text_stream.rs` | Unir deltas NDJSON/JSON con snapshots finales sin duplicar texto | A: casi textual; extraer el helper `limited` | Sin dependencias externas; caché acotada a 128 mensajes |
| Motor PTY | `src-tauri/src/terminal.rs` | Ciclo de vida de terminales, writer Linux no bloqueante, `Drop` y limpieza | B: adaptar eventos/comandos y el modelo de sesión | `portable-pty`, `tauri`, `windows_job` |
| Workspace y path scoping | `src-tauri/src/workspace.rs` + `workspace/file_search.rs` | Rutas canónicas, árbol acotado, búsqueda de archivos y captura de procesos | B: quitar Git/worktrees; conservar scoping y límites | crate `ignore`; límites de 128 KiB por archivo, 5.000 entradas |
| Preview de archivos | `src-tauri/src/file_preview.rs` | Enlaces y preview local seguro dentro del proyecto | B: opcional para el panel de archivos | `tokio`; valida ruta léxica y canónica, bloquea dotfiles/claves |
| Temas VS Code | `src-tauri/src/vsix_themes.rs` | Importar `.vsix`/Open VSX y mapear colores al tema | B: opcional, si se habilita import de temas VS Code | `zip`, `reqwest`, `serde`; sólo conserva el mapa `colors` |
| Apariencia nativa | `src-tauri/src/appearance.rs` | Material de ventana en macOS, fallback sólido y preferencias de accesibilidad | B: conservar macOS/fallback; agregar Mica/Acrylic en Windows | `objc2` en macOS; `tauri`; su implementación no-macOS es sólida |
| Toolkit de color | `src/color.ts` | sRGB/OKLCH, mezcla, luminancia y contraste WCAG | A: casi textual | Sin dependencias |
| Derivación de paleta | `src/palette-core.ts` | Tokens desde un acento, mapeo de temas VS Code y color dominante | A: casi textual, renombrando tokens a los nuestros | `color.ts` |
| Tema y migración | `src/theme.ts` + `src/preferences.ts` | Toggle claro/oscuro y migración de preferencias | B: adaptar claves y persistencia | `localStorage` |
| Sincronización de apariencia | `src/appearance.ts` | Coalescing de IPC, `prefers-reduced-transparency` y alto contraste | B: adaptar al bridge Tauri y a nuestro store | `@tauri-apps/api` |
| Terminal UI | `src/terminal.ts` + `src/terminal-panel.ts` | xterm.js, buffers, tabs y rehidratación | B: adaptar a `session://` y a React | `@xterm/xterm`, `@xterm/addon-fit` |
| Markdown incremental | `src/streaming-markdown.ts` | Pintar respuestas del agente sin reflows | B: adaptar a nuestro transcript | `dom.ts`, `file-links.ts`, `transcript-scroll.ts` |
| Pipeline Markdown | `src/dom.ts` (subset `markdownHtml` + iconos) | Render saneado de Markdown y utilidades de UI | B: extraer el subset, no copiar el archivo completo | `dompurify`, `katex`, `marked`, `enriched-markdown` |
| Parser Enriched Markdown | `src/vendor/enriched-markdown/` | Parser md4c/WASM para Markdown rico | A: vendorizar con su licencia MIT (Software Mansion) | Integridad del tarball y cambio de export documentados upstream |
| Scroll de transcript | `src/transcript-scroll.ts` | Anclar el mensaje visible al insertar contenido arriba | A: casi textual | Sin dependencias |
| Escalado de interfaz | `src/ui-size.ts` | Zoom de toda la interfaz y atajos | B: adaptar a React y a nuestras preferencias | `@tauri-apps/api/webview` |
| Esquema y migraciones | `src-tauri/src/storage.rs` | Patrones de migración, paginación y límites | C: referencia, no copiar | Acoplado a sesiones/mensajes/architect |
| Capa ACP/providers | `src-tauri/src/providers.rs`, `providers/acp.rs`, `providers/launch.rs` | Contratos de protocolo, discovery de binarios y frames acotados | C: referencia; evaluar ACP como capacidad futura | Acoplado a su modelo de evento |
| Equipos/agentes | `src-tauri/src/architect*`, `message_queue.rs`, `turn_changes.rs` | Orquestación, watchdog, colas y diffs | C: referencia para la conversación de dos agentes | Fuera del MVP |

#### Vendorización y flujo de incorporación

1. **Fijar upstream**: commit `e9f89891fd23518848765aef8219dbf39feac6e0`; registrar repo, versión y fecha en `third_party/pandacode/UPSTREAM.md`.
2. **Copiar preservando la ruta**: `third_party/pandacode/<ruta-original>`; nunca mezclar un archivo adaptado sin dejar el original identificable.
3. **Cabecera obligatoria** en cada archivo:
   ```text
   SPDX-License-Identifier: MIT
   SPDX-FileCopyrightText: 2026 PandaCode contributors
   Source: https://github.com/aicodeking/pandacode/blob/<sha>/<ruta>
   Modified by AncletoCode: <resumen del cambio>
   ```
   En Rust/TS con `//`; en CSS con `/* ... */`.
4. **Ledger**: agregar la fila en `docs/provenance.md` con ruta local, ruta upstream, licencia, autor, commit y resumen de modificaciones.
5. **Tests**: portar los tests upstream del módulo, con la misma cabecera, y ejecutarlos en CI.
6. **Verificación**: `npm run license:check`, `cargo deny check`, `npm run test` y `cargo test`.
7. **Sync**: `scripts/sync-pandacode.mjs` descarga los archivos fijados y muestra el diff; una actualización de upstream requiere revisión y actualización de commit/ledger, nunca un reemplazo silencioso.

#### Obligaciones de licencia

La licencia de PandaCode es MIT con una condición adicional de atribución para forks. Al incorporar código, se aplican todas:

1. **Aviso MIT**: incluir el copyright y el texto de permiso completos en `third_party/pandacode/LICENSE` y en `THIRD_PARTY_NOTICES.md`; el aviso viaja en cada copia o porción sustancial.
2. **Atribución de fork (obligatoria)**: incluir exactamente **"This is a fork of PandaCode"** con enlace a `https://github.com/aicodeking/pandacode` en:
   - `README.md` o documentación principal;
   - la pantalla **About** de la app;
   - el sitio o store listing si existe.
3. **Sin endoso**: no usar logos de proveedores ni marcas de PandaCode de forma que sugiera afiliación.
4. **Compatibilidad AGPL-3.0**: AncletoCode mantiene su `LICENSE` AGPL-3.0; la distribución combinada conserva los avisos MIT y la condición de atribución. La condición adicional se documenta como término permitido por AGPL §7 (preservación de avisos) y se somete a revisión legal antes del primer release.
5. **Activos**: los wallpapers, temas, iconos, screenshots y logos son obras separadas; no se copian. Si se usara un activo, se registra su licencia específica y se verifica que permita redistribución.
6. **Dependencias**: las licencias de los crates y paquetes npm que arrastran los módulos reutilizados se auditan y se incluyen en los avisos.

#### Artefactos de cumplimiento

- `third_party/pandacode/LICENSE`: texto completo de la PandaCode License.
- `third_party/pandacode/UPSTREAM.md`: repo, commit, versión, archivos vendorizados y procedimiento de sync.
- `THIRD_PARTY_NOTICES.md` en la raíz: PandaCode, Software Mansion (md4c/Enriched Markdown) y cualquier dependencia vendorizada.
- `docs/provenance.md`: ledger de referencias y código incorporado, con commit, ruta, licencia, autor y modificaciones.
- `README.md`: sección "Licencias y atribuciones" con la declaración de fork y el enlace.
- About de la app: declaración de fork y enlace a PandaCode.
- `scripts/license-check.mjs` (`npm run license:check`): valida `LICENSE`, `THIRD_PARTY_NOTICES.md`, la sección del README, la declaración de fork y las cabeceras SPDX de `third_party/pandacode/`.
- `scripts/sync-pandacode.mjs` y `deny.toml`: sincronización con upstream y auditoría de licencias Rust.
- `specs/license-compliance/spec.md`: requisitos verificables del change.

#### Auditoría de dependencias

- **Rust**: `cargo deny check licenses advisories` con `deny.toml`; allowlist explícita (MIT, Apache-2.0, BSD-2/3-Clause, ISC, Unicode-3.0, Zlib) y revisión manual de cualquier licencia nueva.
- **npm**: `license-checker --production` (o equivalente) para dependencias directas y transitivas; falla ante licencias ausentes, `UNLICENSED` o no compatibles con AGPL-3.0.
- **Assets**: inventario con origen y licencia; bloqueo de imágenes, fuentes, temas o logos sin permiso de redistribución.
- **Reporte**: el resultado se vuelca en `THIRD_PARTY_NOTICES.md` o en un archivo generado referenciado desde allí.

#### Validación de cumplimiento

- `npm run license:check` en verde y `cargo deny check` sin hallazgos bloqueantes.
- Cada archivo de `third_party/pandacode/` tiene cabecera SPDX, entrada en `docs/provenance.md` y figura en `UPSTREAM.md`.
- README y About muestran "This is a fork of PandaCode" con el enlace correcto.
- El árbol no contiene wallpapers, temas, logos, screenshots ni iconos de PandaCode.
- El diff contra el commit fijado coincide con lo declarado en el ledger.
- Revisión legal completada antes del primer release público.
- Repetir la auditoría antes de cada release.

### UI de escritorio

- Layout de tres zonas: sidebar de proyectos, área central de tabs (proyecto → agentes), panel derecho contextual (árbol, conocimiento, SDD o conversación).
- `ProjectTabs`: primer nivel por proyecto, con indicador de sesiones activas y de estado de providers.
- `AgentTabs`: una tab por sesión con dot de estado.
- `TerminalView`: xterm.js enganchado a `session://output`.
- `KnowledgePanel`: stats del manifest, god nodes, hotspots, ciclos, capas, blast radius, unsupported y acceso a la vista humana.
- `SpecPanel`: cambios activos, memoria, discovery/impact, `specs check`, `doctor`.
- `ConversationPanel`: dos participantes, objetivo, contador de turnos, pausa/aborto.
- Settings por proyecto: host de spec, tier, `language`, overrides de binarios, límites de conversación y language pack de writer.
- La app es un cliente del host: al cerrar la ventana hace detach y no mata las sesiones. Un ítem de menú permite "matar todas las sesiones" con confirmación explícita.

### Apariencia CLI, transparencia y temas

La identidad visual es **CLI-first**: la app se ve como una terminal de escritorio, no como un IDE. Esta sección aplica a la **app gráfica**; AncletoCMD usa el mismo manifiesto de tema pero sólo la paleta y la tipografía, sin transparencias ni imágenes.

- **Estética**: tipografía monoespaciada (JetBrains Mono o IBM Plex Mono empaquetada, fallback a la mono del sistema), densidad compacta, chrome tipo prompt (`›`, `❯`, `▍`), statusline por sesión, separadores ASCII y una barra de título con controles discretos. Sin iconografía decorativa; los estados se comunican con color y glifos.
- **Sistema de tokens**: variables CSS semánticas (`--bg-base`, `--bg-panel`, `--bg-elevated`, `--fg`, `--fg-muted`, `--accent`, `--accent-alt`, `--success`, `--warning`, `--danger`, `--border`, `--selection`, `--cursor`). La paleta ANSI de xterm, la de AncletoCMD y la UI se derivan del mismo tema para que no se contradigan.
- **Temas**: presets incluidos (Ancleto Dark, Carbon, Solarized, Nord, Dracula y un tema claro), definidos como manifiestos JSON. El usuario puede crear, duplicar, importar y exportar temas, con preview en vivo. AncletoCMD consume el mismo manifiesto y adapta la paleta a 16/256/truecolor según el render configurado.
- **Transparencias**:
  - **Ventana**: `transparent: true` + efectos nativos vía `window-vibrancy`/API de Tauri (Mica, Acrylic o Blur en Windows; vibrancy en macOS; compositor en Linux), con detección de soporte y fallback a fondo translúcido CSS.
  - **Paneles**: `backdrop-filter` en sidebar, tabs, paneles laterales y overlays.
  - **Controles separados**: opacidad de ventana, opacidad de paneles y opacidad del fondo del terminal; además de un toggle "modo opaco" global.
  - **Rendimiento**: límite de superficies con blur simultáneas; por defecto el terminal usa fondo casi opaco (mejor throughput de xterm) y la transparencia real del terminal es opt-in.
  - **AncletoCMD** no implementa transparencias ni efectos de ventana: su prioridad es latencia y throughput.
- **Fondos**: sólido, degradado (dos o tres colores, ángulo configurable), imagen (cover/contain/tile) con controles de desenfoque y oscurecimiento, o patrón sutil (grid/scanlines) desactivado por defecto. Los archivos de imagen se copian al directorio de datos de la app, nunca al repositorio del proyecto.
- **Precedencia y persistencia**: `global → workspace → proyecto`. La apariencia se guarda en la configuración de la app (SQLite), **no** en `.ancletorc` ni en `.ancleto-writer.json`. Un tema puede exportarse como `theme.json` para compartirlo con AncletoCMD y otros usuarios.
- **Accesibilidad**: verificación de contraste mínimo (WCAG AA) para texto y estados, scrim automático cuando hay imagen de fondo, opción de desactivar transparencia/animaciones, y aviso cuando un tema personalizado no cumple contraste.
- **Superficies afectadas**: título, sidebar, tabs, terminal, paneles de conocimiento/SDD/conversación, modales y menús contextuales.

### Persistencia

- SQLite local para proyectos, metadatos de sesiones, conversaciones, layouts de AncletoCMD y configuración.
- **Escritor único**: el host es el único proceso que escribe estado de sesiones; los clientes leen con WAL y hacen mutaciones a través del host. Las migraciones son idempotentes y se ejecutan al arrancar el host.
- Los buffers de terminal viven en memoria del host y se conservan mientras la sesión exista; al detach no se descartan, al matar la sesión sí.
- La configuración es compartida por las versiones; no hay archivos de configuración por superficie.
- Nada de la app se escribe dentro de `.ancleto/writer/` ni de `aspec/`.


### UI híbrida PandaCode + Orca y Git (2026-10-22)

La app de escritorio es el único producto de esta etapa. La referencia visual es
un híbrido: la calma, densidad y navegación por teclado de PandaCode, más la
organización de flotas, tareas y worktrees de Orca. No se adopta Electron ni se
copia la app de Orca; se implementa sobre Tauri + React + Rust.

**Shell.** Navegación lateral por proyectos y tareas, workspace central con
threads/tabs, inspector derecho para contexto/cambios/Git y composer inferior.
Paneles redimensionables, command palette y quick switcher. Los paneles Ancleto
(spec/writer/conversación) se integran como vistas del mismo shell.

**Git.** Un `GitService` en Rust usa el binario `git` del sistema con argv, sin
shell intermedio, para status/diff/stage/commit/branch/fetch/pull/push/merge y
worktrees. Cada tarea puede tener su propio worktree y rama. Los tokens no se
guardan en la app: la autenticación se delega a `gh`, `glab`, el credential helper
o el remoto web. Los proveedores se exponen detrás de `GitHostProvider`
(GitHub/GitLab primero; Bitbucket/Azure o remoto genérico después).

**Worktrees.** Crear un worktree por tarea permite ejecutar varios agentes en
paralelo sin pisar el árbol principal; el merge del ganador es explícito y nunca
automático. La limpieza verifica cambios sin commit antes de eliminar.

**Release.** Linux: AppImage + `.deb`; Windows: NSIS/MSI/ZIP portable. Sin npm,
AncletoCMD ni CLI en el camino de publicación. La AppImage se prueba en
Arch/EndeavourOS con FUSE y con `--appimage-extract-and-run`.

## Validation

- **Rust (`cargo test` + `clippy`)**:
  - registry de drivers, detección y construcción de comandos interactivo/one-shot;
  - parser de señales con fixtures (incluida salida JSON de `agy`);
  - máquina de estados del orquestador: `maxTurns`, repetición, timeout, aborto, pausa;
  - adapters de `@ancleto/spec` y `@ancleto/writer` con fixtures de `.ancletorc`, `manifest.json`, `graph.json` y `facts/*`, incluidos casos `schemaVersion` incompatible y `unsupported.json`;
  - resolución de binarios: local, global, ausente y Node < 24;
  - core sin dependencias de Tauri/host/cli/cmd (test de grafo de dependencias).
- **Host**: handshake con/sin token, rechazo de versión de protocolo distinta, autostart con lock, detach sin matar PTY, reattach con snapshot, caída simulada y reconciliación a `crashed`, límite de inactividad.
- **CLI**: tests de parsing de argumentos, `--json` con sobre versionado, códigos de salida `0/2/3/4/5/6`, ausencia de ANSI cuando no hay TTY, cancelación con `Ctrl+C`, todas las rutas de `--no-host`.
- **AncletoCMD**:
  - **VT**: suite de conformidad con `vttest`/fixtures y programas reales (vim, tmux, htop, git, less); wide chars, emoji, combinación, alt screen, mouse, bracketed paste y OSC 8.
  - **Layout**: árbol de tabs/splits, foco, resize, restauración de `TerminalLayout` y multi-monitor/DPI (tests headless del modelo + smoke manual).
  - **Shell integration**: OSC 133 en bash/zsh/fish/PowerShell, exit status, navegación entre comandos y fallback heurístico en `cmd.exe`.
  - **Rendimiento**: benchmark de tecla a frame (p99 < 30 ms), idle CPU, arranque, throughput de output y scrollback de 10k líneas.
  - **Robustez**: pérdida de GPU/adapter, fallback software, fuente faltante, IME y clipboard denegado.
- **Empaquetado**: matriz de build (win x64/arm64, linux gnu/musl x64/arm64, mac x64/arm64), launcher con checksum válido/inválido, paquete opcional ausente, `npm i -g` de `@ancleto/code` y `@ancleto/cmd` en Windows/PowerShell y Linux, y comparación de versión launcher/binario/app.
- **Paridad entre versiones**: crear una sesión en AncletoCMD, verla y escribir desde la app, adjuntarla desde la CLI, matarla desde la app y verificar el estado final en las tres.
- **Frontend (`vitest` + `tsc --noEmit` + `lint`)**: stores de tabs, estado de providers, panel de conocimiento, controles de conversación y resolución de temas (precedencia global → workspace → proyecto, contraste y export/import de `theme.json`).
- **Smoke manual end-to-end**: dos proyectos → sesiones `claude`, `opencode`, `agy` y `cmd` → output en vivo, resize y kill → conversación agente-a-agente con corte por `maxTurns` y aborto → `ancleto-writer build` y `human` desde el panel; repetir el flujo en AncletoCMD desde `cmd`/PowerShell y desde Linux.
- **Terminal**: verificar AncletoCMD con `cmd`, PowerShell, WSL, bash, zsh y fish; resize, mouse, paste, IME, colores y cierre/reapertura con sesiones vivas.
- **Apariencia**: verificar transparencia nativa y fallback CSS en Windows, cambio de tema en vivo, fondo con imagen, contraste AA y persistencia por proyecto.
- **Degradación**: simular CLI ausente, Node ausente, writer sin language pack, `schemaVersion` incompatible, host caído y salida no parseable; las versiones deben mostrar `missing`/`incompatible` y modo crudo sin romper sesiones.
- **Cierre limpio**: cerrar la ventana, AncletoCMD y el host, y verificar que no quedan procesos PTY hijos ni procesos one-shot colgados.
- **Cumplimiento**: `npm run license:check` y `cargo deny check` en verde, módulos vendorizados con cabecera/ledger, diff de upstream declarado, y README/About con "This is a fork of PandaCode" y el enlace.
