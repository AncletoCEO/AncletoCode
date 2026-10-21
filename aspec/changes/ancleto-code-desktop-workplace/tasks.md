# Tasks: AncletoCode — desktop workplace para agentes de código

## Fase 1 — Bootstrap del proyecto

- [ ] 1.1 Inicializar Tauri 2 con frontend React 19 + TypeScript + Vite y fijar versiones.
- [ ] 1.2 Configurar `tsconfig` estricto, ESLint/Prettier y scripts `dev`, `build`, `test`, `typecheck`, `lint`.
- [ ] 1.3 Configurar `cargo fmt`/`clippy` y estructura de módulos Rust (`commands`, `session`, `drivers`, `orch`, `providers`, `ide`, `persistence`).
- [ ] 1.4 Verificar arranque de la app de escritorio en Windows (`npm run tauri dev`).

## Fase 2 — Proyectos y persistencia

- [ ] 2.1 Modelo `Project` + comandos `project_add`, `project_list`, `project_remove`, `project_open`.
- [ ] 2.2 Persistencia SQLite de proyectos y configuración en `app_config_dir`, con migración inicial.
- [ ] 2.3 Comando `project_tree` para el árbol de archivos del proyecto.
- [ ] 2.4 `projectsStore` y sidebar de proyectos con alta/baja/apertura.
- [ ] 2.5 Tests de persistencia (Rust) y del store (Vitest).

## Fase 3 — Motor de sesiones PTY

- [ ] 3.1 `SessionManager` con `portable-pty`: crear, escribir, redimensionar y matar sesiones.
- [ ] 3.2 Ring buffer por sesión y `session_snapshot` para rehidratar la terminal.
- [ ] 3.3 Eventos `session://output/{id}`, `session://state/{id}`, `session://exit/{id}`.
- [ ] 3.4 Cierre ordenado de todas las sesiones al salir (sin huérfanos).
- [ ] 3.5 `TerminalView` con xterm.js, snapshot y resize.
- [ ] 3.6 Tests de ciclo de vida con un proceso de prueba y verificación de no-huérfanos.

## Fase 4 — Drivers de CLIs

- [ ] 4.1 Contrato `AgentDriver` con capacidades (`interactivePty`, `oneShotPrompt`, `structuredOutput`, `resumeSession`), `DriverRegistry` y fallback `raw`.
- [ ] 4.2 Driver `claude`: detección (`claude --version`), arranque interactivo y one-shot (`-p`) con salida JSON.
- [ ] 4.3 Driver `opencode`: detección (`opencode --version`), arranque interactivo y one-shot (`run`).
- [ ] 4.4 Driver `agy` (Antigravity CLI): detección (`agy --version`), TUI interactiva y headless (`agy -p ... --output-format json`), con `--continue`/`--conversation` cuando aplique.
- [ ] 4.5 Driver `cmd` (Command Code): detección (bin `cmd` del paquete `command-code`), TUI interactiva y headless (`cmd -p ... --output-format json`), con `--continue`/`--resume` y `--max-turns` cuando aplique.
- [ ] 4.6 Comando `drivers_list` con estado `available`/`missing`/`error` y versión, reflejado en la UI.
- [ ] 4.7 Tests de registry, detección, construcción de comandos y parser de señales (incluido NDJSON de `cmd` y JSON de `agy`).

## Fase 5 — Integración con Antigravity IDE

- [ ] 5.1 Acción de proyecto "abrir en Antigravity" usando el IDE instalado.
- [ ] 5.2 Sincronizar assets de host vía `@ancleto/spec --agent antigravity` (`.agents/skills`, `.agents/agents`, `.agents/mcp_config.json` no destructivo).
- [ ] 5.3 Tests del comando de apertura y del merge no destructivo del MCP de workspace.

## Fase 6 — Tabs por proyecto y por agente

- [ ] 6.1 `sessionsStore` y modelo de tabs proyecto → sesiones.
- [ ] 6.2 `ProjectTabs` con indicador de sesiones activas y de estado de providers.
- [ ] 6.3 `AgentTabs` con dot de estado (`ready`, `busy`, `waiting_input`, `stopped`, `crashed`).
- [ ] 6.4 Alta de sesión eligiendo driver disponible y `cwd` del proyecto.
- [ ] 6.5 Acciones por tab: renombrar, cerrar, matar, reiniciar.
- [ ] 6.6 Tests Vitest de stores y de estado de tabs.

## Fase 7 — Interacción entre dos agentes

- [ ] 7.1 Modelo `Conversation` y persistencia de turnos (origen, destino, texto, modo, duración, timestamps).
- [ ] 7.2 Estrategia one-shot estructurada por turno, con continuidad por resume/continue cuando el driver la soporte.
- [ ] 7.3 Fallback de inyección PTY con espera de quiescencia y señal `Ready`.
- [ ] 7.4 Límites: `maxTurns`, timeout por turno, detección de repetición, pausa/reanudación y aborto.
- [ ] 7.5 Comandos `conversation_start`, `conversation_pause`, `conversation_resume`, `conversation_abort` y eventos de turno.
- [ ] 7.6 `ConversationPanel` con selector de dos sesiones, objetivo, contador y controles.
- [ ] 7.7 Tests de la máquina de estados con drivers simulados (one-shot y PTY).

## Fase 8 — Proveedor `@ancleto/spec`

- [ ] 8.1 `NodeRuntime` y resolución de binarios en orden local → global → sidecar, con chequeo de Node ≥ 24.
- [ ] 8.2 Comandos de proyecto: `init` (`--agent`, `--tier`, `--lang`, `--exclude`, `--profile`), `install --project`, `update`, `upgrade`, `check`, `doctor`.
- [ ] 8.3 Lectura de `.ancletorc` (host, tier, `language`, `discovery.exclude`) y de `aspec/changes` activos.
- [ ] 8.4 Memoria del proyecto vía CLI (`ancleto memory list --json`, `ancleto memory context`); sin acceso directo a `memory.db`.
- [ ] 8.5 Estado de discovery (`ancleto discovery --check`) con `impact` y regeneración sólo con aprobación.
- [ ] 8.6 `specs check --json` y `doctor` en el `SpecPanel`.
- [ ] 8.7 Settings por proyecto: host, tier (`normal`/`minimo`/`gratis`) e idioma (`auto`/`es`/`en`/`pt`).
- [ ] 8.8 Tests de adapter con fixtures de `.ancletorc`, cambios, discovery y salidas JSON.

## Fase 9 — Proveedor `@ancleto/writer`

- [ ] 9.1 Resolución de `ancleto-writer` y detección de versión; estado `missing`/`incompatible`.
- [ ] 9.2 Comandos `init`, `build`, `check`, `lint`, `impact`, `human`, `doctor` con manejo de `--json` y códigos de salida 0–5.
- [ ] 9.3 Frescura autoritativa vía `check --json`; no recalcular fingerprint ni asumir `schemaVersion`.
- [ ] 9.4 Lectura de `manifest.json`, `graph.json` y `facts/*` (ciclos, capas, comunidades, god nodes, hotspots, blast radius, dead code, doc coverage, unsupported).
- [ ] 9.5 `KnowledgePanel` con stats y hechos; documentos `agent/*` con acción "adjuntar al contexto".
- [ ] 9.6 Vista humana: ejecutar `human` y embeber `index.html`/`interactive.html` vía asset protocol; alternativa "abrir en navegador".
- [ ] 9.7 Detección de `@ancleto/writer-languages` (pack de lenguajes de programación), toggle `languagePack.enabled`/`module` en `.ancleto-writer.json` y estado en `facts/unsupported.json`.
- [ ] 9.8 Estado `missing` con comando npm sugerido (incluida nota `--legacy-peer-deps` para peers de tree-sitter).
- [ ] 9.9 Tests del adapter con fixtures de manifest/graph/facts, `schemaVersion` incompatible y pack ausente.

## Fase 10 — Apariencia CLI, transparencia y temas

- [ ] 10.1 Definir la identidad CLI: tipografía monoespaciada empaquetada, densidad, glifos de prompt y statusline.
- [ ] 10.2 Implementar el sistema de tokens semánticos y derivar la paleta ANSI de xterm desde el tema activo.
- [ ] 10.3 Incluir temas preset (Ancleto Dark, Carbon, Solarized, Nord, Dracula, Light) como manifiestos JSON con preview en vivo.
- [ ] 10.4 Editor de temas: crear, duplicar, importar y exportar `theme.json`.
- [ ] 10.5 Transparencia de ventana con efectos nativos (Mica/Acrylic/Blur en Windows, vibrancy en macOS, compositor en Linux) y fallback CSS.
- [ ] 10.6 Transparencia de paneles con `backdrop-filter` y controles separados de opacidad (ventana, paneles, terminal) más "modo opaco".
- [ ] 10.7 Configuración de fondos: sólido, degradado, imagen (cover/contain/tile) con blur/dim y patrón opcional; copiar imágenes al directorio de datos de la app.
- [ ] 10.8 Precedencia y persistencia global → workspace → proyecto, sin escribir en `.ancletorc` ni `.ancleto-writer.json`.
- [ ] 10.9 Validación de contraste WCAG AA, scrim automático sobre imágenes y soporte de `prefers-reduced-motion`/modo opaco.
- [ ] 10.10 Tests de resolución de temas, contraste, export/import y persistencia por proyecto.

## Fase 11 — Hardening y cierre

- [ ] 11.1 Persistencia de configuración: idioma, host, tier, límites de conversación, overrides de binarios, preferencias de writer y apariencia.
- [ ] 11.2 Atajos: nueva sesión, cambiar proyecto, matar sesión, iniciar conversación, rebuild de writer, cambiar tema.
- [ ] 11.3 Smoke manual end-to-end: dos proyectos, sesiones `claude`/`opencode`/`agy`/`cmd`, conversación agente-a-agente, aborto, build + vista humana de writer.
- [ ] 11.4 Verificar escenarios de degradación (CLI ausente, Node ausente, sin language pack, salida no parseable, efectos de transparencia no soportados).
- [ ] 11.5 `cargo test`, `cargo clippy`, `npm run test`, `npm run typecheck` y `npm run lint` en verde.
- [ ] 11.6 Empaquetado (`npm run tauri build`) y verificación de cierre limpio de procesos.
