# Design: AncletoCode — desktop workplace para agentes de código

## Approach

Aplicación Tauri 2 con dos capas separadas:

1. **Núcleo en Rust** (proceso principal de Tauri): dueño de todo lo que toca el sistema operativo — proyectos en disco, procesos PTY, detección de binarios, lectura de artefactos y ejecución de las CLIs de Ancleto. Expone comandos y eventos tipados.
2. **Frontend web** (React 19 + TypeScript + Vite): estado de UI y presentación. No ejecuta procesos ni accede al filesystem.

Reglas estructurales:

- **El PTY es la fuente de verdad** de cada sesión de agente. Los drivers describen arranque y señales; si el parseo falla, la UI cae a terminal cruda sin romper la sesión.
- **Las CLIs de Ancleto son la fuente de verdad de sus datos.** La app no reimplementa memoria, discovery, grafo ni frescura: invoca `ancleto` y `ancleto-writer` y lee el contrato publicado (`.ancletorc`, `.ancleto/memory.db` vía CLI, `.ancleto/writer/manifest.json` + `check`).
- **Idioma:** el idioma natural de artefactos es de `@ancleto/spec` (`.ancletorc` → `language`). El "language pack" de writer es de **lenguajes de programación** y se trata como dependencia opcional.

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│ Frontend (React + TS + Vite)                                 │
│  Sidebar · ProjectTabs · AgentTabs · TerminalView            │
│  ConversationPanel · KnowledgePanel · SpecPanel · Settings   │
│  stores: projects · sessions · conversations · providers     │
└──────────────▲───────────────────────────┬───────────────────┘
               │ eventos                   │ invoke
┌──────────────┴───────────────────────────▼───────────────────┐
│ Núcleo Rust (Tauri 2)                                        │
│  commands/   projects · sessions · drivers · conversations   │
│              spec · writer · ide                             │
│  session/    SessionManager (PTY + ring buffer + estado)     │
│  drivers/    AgentDriver + claude · opencode · agy · cmd     │
│  orch/       ConversationOrchestrator                        │
│  providers/  AncletoSpec · AncletoWriter · NodeRuntime       │
│  ide/        AntigravityIde (open project + sync assets)     │
│  persistence/ SQLite local (proyectos, sesiones, config)     │
└──────────────────────────────────────────────────────────────┘
```

### Modelo de dominio

- `Project { id, name, rootPath, createdAt, lastOpenedAt }`
- `AgentSession { id, projectId, driverId, cwd, status, mode, startedAt, exitCode? }`
  - `status ∈ { starting, ready, busy, waiting_input, stopped, crashed }`
  - `mode ∈ { interactive, oneShot }`
- `AgentDriverDescriptor { id, displayName, binary, capabilities, detect }`
  - `capabilities ∈ { interactivePty, oneShotPrompt, structuredOutput, resumeSession, headlessJson }`
- `Conversation { id, participants[sessionA, sessionB], objective, maxTurns, turnBudgetMs, state, turns[] }`
- `SpecProjectState { initialized, cliVersion, host, tier, language, discovery: {status, impact}, activeChanges[], memoryRulesCount }`
- `WriterState { status, cliVersion, schemaVersion, freshness, stats, languagePack: {installed, enabled, languages[]} }`
- `ToolStatus { available | missing | incompatible | error, version?, binaryPath?, installHint }`

### Motor de sesiones (PTY)

- `portable-pty` con `cwd` del proyecto y `CommandSpec` provisto por el driver.
- Hilo lector por sesión con dos consumidores: ring buffer (rehidratación) y parser opcional de señales.
- Comandos: `session_create`, `session_write`, `session_resize`, `session_kill`, `session_snapshot`.
- Eventos: `session://output/{id}`, `session://state/{id}`, `session://exit/{id}`.
- Cierre ordenado de todas las sesiones al salir; sin procesos huérfanos.

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

### UI

- Layout de tres zonas: sidebar de proyectos, área central de tabs (proyecto → agentes), panel derecho contextual (árbol, conocimiento, SDD o conversación).
- `ProjectTabs`: primer nivel por proyecto, con indicador de sesiones activas y de estado de providers.
- `AgentTabs`: una tab por sesión con dot de estado.
- `TerminalView`: xterm.js enganchado a `session://output`.
- `KnowledgePanel`: stats del manifest, god nodes, hotspots, ciclos, capas, blast radius, unsupported y acceso a la vista humana.
- `SpecPanel`: cambios activos, memoria, discovery/impact, `specs check`, `doctor`.
- `ConversationPanel`: dos participantes, objetivo, contador de turnos, pausa/aborto.
- Settings por proyecto: host de spec, tier, `language`, overrides de binarios, límites de conversación y language pack de writer.

### Apariencia CLI, transparencia y temas

La identidad visual es **CLI-first**: la app se ve como una terminal de escritorio, no como un IDE.

- **Estética**: tipografía monoespaciada (JetBrains Mono o IBM Plex Mono empaquetada, fallback a la mono del sistema), densidad compacta, chrome tipo prompt (`›`, `❯`, `▍`), statusline por sesión, separadores ASCII y una barra de título con controles discretos. Sin iconografía decorativa; los estados se comunican con color y glifos.
- **Sistema de tokens**: variables CSS semánticas (`--bg-base`, `--bg-panel`, `--bg-elevated`, `--fg`, `--fg-muted`, `--accent`, `--accent-alt`, `--success`, `--warning`, `--danger`, `--border`, `--selection`, `--cursor`). La paleta ANSI de xterm se deriva del mismo tema para que terminal y UI no se contradigan.
- **Temas**: presets incluidos (Ancleto Dark, Carbon, Solarized, Nord, Dracula y un tema claro), definidos como manifiestos JSON. El usuario puede crear, duplicar, importar y exportar temas, con preview en vivo.
- **Transparencias**:
  - **Ventana**: `transparent: true` + efectos nativos vía `window-vibrancy`/API de Tauri (Mica, Acrylic o Blur en Windows; vibrancy en macOS; compositor en Linux), con detección de soporte y fallback a fondo translúcido CSS.
  - **Paneles**: `backdrop-filter` en sidebar, tabs, paneles laterales y overlays.
  - **Controles separados**: opacidad de ventana, opacidad de paneles y opacidad del fondo del terminal; además de un toggle "modo opaco" global.
  - **Rendimiento**: límite de superficies con blur simultáneas; por defecto el terminal usa fondo casi opaco (mejor throughput de xterm) y la transparencia real del terminal es opt-in.
- **Fondos**: sólido, degradado (dos o tres colores, ángulo configurable), imagen (cover/contain/tile) con controles de desenfoque y oscurecimiento, o patrón sutil (grid/scanlines) desactivado por defecto. Los archivos de imagen se copian al directorio de datos de la app, nunca al repositorio del proyecto.
- **Precedencia y persistencia**: `global → workspace → proyecto`. La apariencia se guarda en la configuración de la app (SQLite), **no** en `.ancletorc` ni en `.ancleto-writer.json`. Un tema puede exportarse como `theme.json` para compartirlo.
- **Accesibilidad**: verificación de contraste mínimo (WCAG AA) para texto y estados, scrim automático cuando hay imagen de fondo, opción de desactivar transparencia/animaciones, y aviso cuando un tema personalizado no cumple contraste.
- **Superficies afectadas**: título, sidebar, tabs, terminal, paneles de conocimiento/SDD/conversación, modales y menús contextuales.

### Persistencia

- SQLite local para proyectos, metadatos de sesiones, conversaciones y configuración.
- Los buffers de terminal viven en memoria y se descartan al cerrar la sesión.
- Nada de la app se escribe dentro de `.ancleto/writer/` ni de `aspec/`.

## Validation

- **Rust (`cargo test` + `clippy`)**:
  - registry de drivers, detección y construcción de comandos interactivo/one-shot;
  - parser de señales con fixtures (incluida salida JSON de `agy`);
  - máquina de estados del orquestador: `maxTurns`, repetición, timeout, aborto, pausa;
  - adapters de `@ancleto/spec` y `@ancleto/writer` con fixtures de `.ancletorc`, `manifest.json`, `graph.json` y `facts/*`, incluidos casos `schemaVersion` incompatible y `unsupported.json`;
  - resolución de binarios: local, global, ausente y Node < 24.
- **Frontend (`vitest` + `tsc --noEmit` + `lint`)**: stores de tabs, estado de providers, panel de conocimiento, controles de conversación y resolución de temas (precedencia global → workspace → proyecto, contraste y export/import de `theme.json`).
- **Smoke manual end-to-end**: dos proyectos → sesiones `claude`, `opencode`, `agy` y `cmd` → output en vivo, resize y kill → conversación agente-a-agente con corte por `maxTurns` y aborto → `ancleto-writer build` y `human` desde el panel.
- **Apariencia**: verificar transparencia nativa y fallback CSS en Windows, cambio de tema en vivo, fondo con imagen, contraste AA y persistencia por proyecto.
- **Degradación**: simular CLI ausente, Node ausente, writer sin language pack, `schemaVersion` incompatible y salida no parseable; la app debe mostrar `missing`/`incompatible` y modo crudo sin romper sesiones.
- **Cierre limpio**: cerrar la ventana y verificar que no quedan procesos PTY hijos ni procesos one-shot colgados.
