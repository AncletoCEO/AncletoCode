# Tasks: AncletoCode — workspace de agentes para escritorio y terminal

## Issues de GitHub

Cada issue corresponde a un bloque de features y está asociado a las tareas de este archivo.

| Issue | Feature | Fases / tareas |
|---|---|---|
| [#1](https://github.com/AncletoCEO/AncletoCode/issues/1) | Compliance de licencias y vendorización | Fase 0 (0.1–0.8), Fase 1 (1.5–1.7), Fase 3B (3B.1–3B.13), Fase 14 (14.11–14.12) |
| [#2](https://github.com/AncletoCEO/AncletoCode/issues/2) | Workspace Cargo y núcleo compartido | Fase 1 (1.1–1.4), Fase 1B (1B.1–1B.5) |
| [#3](https://github.com/AncletoCEO/AncletoCode/issues/3) | Proyectos y persistencia | Fase 2 (2.1–2.5) |
| [#4](https://github.com/AncletoCEO/AncletoCode/issues/4) | Motor de sesiones PTY | Fase 3 (3.1–3.6) |
| [#5](https://github.com/AncletoCEO/AncletoCode/issues/5) | Drivers de agentes | Fase 4 (4.1–4.7) |
| [#6](https://github.com/AncletoCEO/AncletoCode/issues/6) | Integración con Antigravity IDE | Fase 5 (5.1–5.3) |
| [#7](https://github.com/AncletoCEO/AncletoCode/issues/7) | Tabs por proyecto y por agente | Fase 6 (6.1–6.6) |
| [#8](https://github.com/AncletoCEO/AncletoCode/issues/8) | Conversación agente-a-agente | Fase 7 (7.1–7.7) |
| [#9](https://github.com/AncletoCEO/AncletoCode/issues/9) | Proveedor `@ancleto/spec` | Fase 8 (8.1–8.8) |
| [#10](https://github.com/AncletoCEO/AncletoCode/issues/10) | Proveedor `@ancleto/writer` | Fase 9 (9.1–9.9) |
| [#11](https://github.com/AncletoCEO/AncletoCode/issues/11) | Apariencia, transparencia y temas | Fase 10 (10.1–10.10) |
| [#12](https://github.com/AncletoCEO/AncletoCode/issues/12) | Host local de sesiones | Fase 11 (11.1–11.8) |
| [#13](https://github.com/AncletoCEO/AncletoCode/issues/13) | CLI headless `acode` | Fase 12 (12.1–12.13) |
| [#14](https://github.com/AncletoCEO/AncletoCode/issues/14) | AncletoCMD | Fase 13 (13.1–13.15) |
| [#15](https://github.com/AncletoCEO/AncletoCode/issues/15) | Empaquetado npm y release | Fase 14 (14.1–14.10) |
| [#16](https://github.com/AncletoCEO/AncletoCode/issues/16) | Hardening y cierre | Fase 15 (15.1–15.6) |

## Fase 0 — Cumplimiento de licencia y vendorización (bloqueante)

Issue: [#1](https://github.com/AncletoCEO/AncletoCode/issues/1)

- [x] 0.1 Crear `third_party/pandacode/LICENSE` con el texto completo de la PandaCode License.
- [x] 0.2 Crear `third_party/pandacode/UPSTREAM.md` con repo, commit `e9f89891fd23518848765aef8219dbf39feac6e0`, versión 0.3.13, fecha, archivos vendorizados y procedimiento de sync.
- [x] 0.3 Crear `THIRD_PARTY_NOTICES.md` con PandaCode, Software Mansion (md4c/Enriched Markdown) y las dependencias vendorizadas que se adopten.
- [x] 0.4 Crear `docs/provenance.md` con el ledger de módulos reutilizados: ruta local, ruta upstream, commit, licencia, autor y modificaciones.
- [x] 0.5 Agregar la sección "Licencias y atribuciones" en `README.md` con la declaración obligatoria "This is a fork of PandaCode" y el enlace.
- [x] 0.6 Documentar la convención de cabeceras SPDX para archivos vendorizados y adaptados en `AGENTS.md`/guía de contribución.
- [x] 0.7 Auditar el árbol contra nombres y hashes de PandaCode para detectar copias accidentales de código o activos; documentar el inventario de activos.
- [x] 0.8 Verificar que no se incluyen wallpapers, temas, iconos, screenshots ni logos de PandaCode ni de proveedores sin licencia que permita redistribución.

## Fase 1 — Bootstrap del proyecto

Issue: [#2](https://github.com/AncletoCEO/AncletoCode/issues/2)

- [x] 1.1 Inicializar Tauri 2 con frontend React 19 + TypeScript + Vite y fijar versiones.
- [x] 1.2 Configurar `tsconfig` estricto, ESLint/Prettier y scripts `dev`, `build`, `test`, `typecheck`, `lint`.
- [x] 1.3 Configurar `cargo fmt`/`clippy` y estructura de módulos Rust (`commands`, `session`, `drivers`, `orch`, `providers`, `ide`, `persistence`).
- [ ] 1.4 Verificar arranque de la app de escritorio en Windows (`npm run tauri dev`).
- [x] 1.5 Implementar la declaración "This is a fork of PandaCode" + enlace en la pantalla About de la app.
- [x] 1.6 Añadir `npm run license:check` (`scripts/license-check.mjs`): valida `LICENSE`, `THIRD_PARTY_NOTICES.md`, la sección del README, la declaración de fork, las cabeceras SPDX y el ledger.
- [x] 1.7 Añadir `deny.toml` + `cargo deny check licenses advisories` y un revisor de licencias npm para dependencias directas y transitivas.

## Fase 1B — Extracción del núcleo compartido

Issue: [#2](https://github.com/AncletoCEO/AncletoCode/issues/2)

- [x] 1B.1 Convertir el proyecto en Cargo workspace con `crates/ancleto-code-core`, `ancleto-code-host`, `ancleto-code-cli`, `ancleto-code-cmd` y `ancleto-code-tauri`.
- [x] 1B.2 Mover dominio, proyectos, sesiones PTY, drivers, orquestador, providers y persistencia a `ancleto-code-core` sin dependencias de Tauri, host, CLI ni AncletoCMD.
- [x] 1B.3 Adaptar la app Tauri para consumir `ancleto-code-core` y mantener sus comandos y eventos.
- [x] 1B.4 Añadir un test de CI que falle ante dependencias inversas o imports de `tauri`/`winit`/`wgpu` dentro del core.
- [x] 1B.5 Ejecutar los tests existentes del core y de la app sin regresiones tras la extracción.

## Fase 2 — Proyectos y persistencia

Issue: [#3](https://github.com/AncletoCEO/AncletoCode/issues/3)

- [x] 2.1 Modelo `Project` + comandos `project_add`, `project_list`, `project_remove`, `project_open`.
- [x] 2.2 Persistencia SQLite de proyectos y configuración en `app_config_dir`, con migración inicial.
- [x] 2.3 Comando `project_tree` para el árbol de archivos del proyecto.
- [x] 2.4 `projectsStore` y sidebar de proyectos con alta/baja/apertura.
- [x] 2.5 Tests de persistencia (Rust) y del store (Vitest).

## Fase 3 — Motor de sesiones PTY

Issue: [#4](https://github.com/AncletoCEO/AncletoCode/issues/4)

- [x] 3.1 `SessionManager` con `portable-pty`: crear, escribir, redimensionar y matar sesiones.
- [x] 3.2 Ring buffer por sesión y `session_snapshot` para rehidratar la terminal.
- [x] 3.3 Eventos `session://output/{id}`, `session://state/{id}`, `session://exit/{id}`.
- [x] 3.4 Cierre ordenado de todas las sesiones al salir (sin huérfanos).
- [x] 3.5 `TerminalView` con xterm.js, snapshot y resize.
- [x] 3.6 Tests de ciclo de vida con un proceso de prueba y verificación de no-huérfanos.

## Fase 3B — Incorporación de código reutilizable de PandaCode

Issue: [#1](https://github.com/AncletoCEO/AncletoCode/issues/1)

- [x] 3B.1 Vendorizar `src-tauri/src/providers/windows_job.rs` (Nivel A: casi textual) con cabecera SPDX y tests de terminación de descendientes.
- [x] 3B.2 Vendorizar `src-tauri/src/providers/text_stream.rs` (Nivel A) y extraer el helper `limited`; portar sus tests de deltas/snapshots.
- [ ] 3B.3 Adaptar `src-tauri/src/terminal.rs` (Nivel B) a `SessionManager`, eventos `session://` y `windows_job`; portar tests de ciclo de vida y no-huérfanos.
- [ ] 3B.4 Adaptar `src-tauri/src/workspace.rs` y `workspace/file_search.rs` (Nivel B) sin Git/worktrees: rutas canónicas, árbol acotado, búsqueda y captura de procesos.
- [x] 3B.5 Vendorizar `src/color.ts` y `src/palette-core.ts` (Nivel A), renombrar tokens y portar tests de contraste/acentos.
- [ ] 3B.6 Adaptar `src/theme.ts`, `src/preferences.ts`, `src/appearance.ts` y `src-tauri/src/appearance.rs` (Nivel B) a nuestro store y bridge; sumar Mica/Acrylic en Windows.
- [ ] 3B.7 Adaptar `src/terminal.ts` y `src/terminal-panel.ts` (Nivel B) a xterm.js, tabs y eventos de sesión.
- [ ] 3B.8 Vendorizar `src/vendor/enriched-markdown/` (MIT, Software Mansion) y adaptar el subset de `src/dom.ts` + `src/streaming-markdown.ts` con DOMPurify/KaTeX.
- [ ] 3B.9 Vendorizar `src/transcript-scroll.ts` (Nivel A) y adaptar `src/ui-size.ts` (Nivel B) al frontend React.
- [ ] 3B.10 Opcional: adaptar `src-tauri/src/file_preview.rs`, `src-tauri/src/vsix_themes.rs` y `src/theme-settings.ts` si se habilitan preview de archivos e import de temas VS Code.
- [x] 3B.11 Crear `scripts/sync-pandacode.mjs` para descargar los archivos fijados y mostrar el diff contra `third_party/pandacode/`.
- [x] 3B.12 Verificar que cada archivo vendorizado tiene cabecera, entrada en `docs/provenance.md` y figura en `UPSTREAM.md`; integrar la verificación en CI.
- [ ] 3B.13 Ejecutar los tests portados y los del proyecto (`cargo test`, `npm run test`) sin regresiones.

## Fase 4 — Drivers de CLIs

Issue: [#5](https://github.com/AncletoCEO/AncletoCode/issues/5)

- [x] 4.1 Contrato `AgentDriver` con capacidades (`interactivePty`, `oneShotPrompt`, `structuredOutput`, `resumeSession`), `DriverRegistry` y fallback `raw`.
- [x] 4.2 Driver `claude`: detección (`claude --version`), arranque interactivo y one-shot (`-p`) con salida JSON.
- [x] 4.3 Driver `opencode`: detección (`opencode --version`), arranque interactivo y one-shot (`run`).
- [x] 4.4 Driver `agy` (Antigravity CLI): detección (`agy --version`), TUI interactiva y headless (`agy -p ... --output-format json`), con `--continue`/`--conversation` cuando aplique.
- [x] 4.5 Driver `cmd` (Command Code): detección (bin `cmd` del paquete `command-code`), TUI interactiva y headless (`cmd -p ... --output-format json`), con `--continue`/`--resume` y `--max-turns` cuando aplique.
- [x] 4.6 Comando `drivers_list` con estado `available`/`missing`/`error` y versión, reflejado en la UI.
- [x] 4.7 Tests de registry, detección, construcción de comandos y parser de señales (incluido NDJSON de `cmd` y JSON de `agy`).

## Fase 5 — Integración con Antigravity IDE

Issue: [#6](https://github.com/AncletoCEO/AncletoCode/issues/6)

- [ ] 5.1 Acción de proyecto "abrir en Antigravity" usando el IDE instalado.
- [ ] 5.2 Sincronizar assets de host vía `@ancleto/spec --agent antigravity` (`.agents/skills`, `.agents/agents`, `.agents/mcp_config.json` no destructivo).
- [ ] 5.3 Tests del comando de apertura y del merge no destructivo del MCP de workspace.

## Fase 6 — Tabs por proyecto y por agente

Issue: [#7](https://github.com/AncletoCEO/AncletoCode/issues/7)

- [x] 6.1 `sessionsStore` y modelo de tabs proyecto → sesiones.
- [x] 6.2 `ProjectTabs` con indicador de sesiones activas y de estado de providers.
- [x] 6.3 `AgentTabs` con dot de estado (`ready`, `busy`, `waiting_input`, `stopped`, `crashed`).
- [x] 6.4 Alta de sesión eligiendo driver disponible y `cwd` del proyecto.
- [x] 6.5 Acciones por tab: renombrar, cerrar, matar, reiniciar.
- [x] 6.6 Tests Vitest de stores y de estado de tabs.

## Fase 7 — Interacción entre dos agentes

Issue: [#8](https://github.com/AncletoCEO/AncletoCode/issues/8)

- [ ] 7.1 Modelo `Conversation` y persistencia de turnos (origen, destino, texto, modo, duración, timestamps).
- [ ] 7.2 Estrategia one-shot estructurada por turno, con continuidad por resume/continue cuando el driver la soporte.
- [ ] 7.3 Fallback de inyección PTY con espera de quiescencia y señal `Ready`.
- [ ] 7.4 Límites: `maxTurns`, timeout por turno, detección de repetición, pausa/reanudación y aborto.
- [ ] 7.5 Comandos `conversation_start`, `conversation_pause`, `conversation_resume`, `conversation_abort` y eventos de turno.
- [ ] 7.6 `ConversationPanel` con selector de dos sesiones, objetivo, contador y controles.
- [ ] 7.7 Tests de la máquina de estados con drivers simulados (one-shot y PTY).

## Fase 8 — Proveedor `@ancleto/spec`

Issue: [#9](https://github.com/AncletoCEO/AncletoCode/issues/9)

- [ ] 8.1 `NodeRuntime` y resolución de binarios en orden local → global → sidecar, con chequeo de Node ≥ 24.
- [ ] 8.2 Comandos de proyecto: `init` (`--agent`, `--tier`, `--lang`, `--exclude`, `--profile`), `install --project`, `update`, `upgrade`, `check`, `doctor`.
- [ ] 8.3 Lectura de `.ancletorc` (host, tier, `language`, `discovery.exclude`) y de `aspec/changes` activos.
- [ ] 8.4 Memoria del proyecto vía CLI (`ancleto memory list --json`, `ancleto memory context`); sin acceso directo a `memory.db`.
- [ ] 8.5 Estado de discovery (`ancleto discovery --check`) con `impact` y regeneración sólo con aprobación.
- [ ] 8.6 `specs check --json` y `doctor` en el `SpecPanel`.
- [ ] 8.7 Settings por proyecto: host, tier (`normal`/`minimo`/`gratis`) e idioma (`auto`/`es`/`en`/`pt`).
- [ ] 8.8 Tests de adapter con fixtures de `.ancletorc`, cambios, discovery y salidas JSON.

## Fase 9 — Proveedor `@ancleto/writer`

Issue: [#10](https://github.com/AncletoCEO/AncletoCode/issues/10)

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

Issue: [#11](https://github.com/AncletoCEO/AncletoCode/issues/11)

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

## Fase 11 — Host local de sesiones

Issue: [#12](https://github.com/AncletoCEO/AncletoCode/issues/12)

- [ ] 11.1 Definir el protocolo JSON-RPC versionado (métodos, eventos, errores y `protocolVersion`) en `ancleto-code-host`.
- [ ] 11.2 Implementar transporte por named pipe en Windows y Unix domain socket en Linux/macOS, con permisos `0700` y token `0600`.
- [ ] 11.3 Implementar autostart con lock file, `host start|stop|status`, límite de inactividad y apagado ordenado.
- [ ] 11.4 Mover la propiedad de los PTY y los ring buffers al host; exponer `session.create|write|resize|kill|snapshot|list|attach|detach`.
- [ ] 11.5 Implementar detach sin matar la sesión y reattach con snapshot del buffer.
- [ ] 11.6 Reconciliar sesiones huérfanas a `crashed` al arrancar el host y ofrecer recreación con el mismo `cwd` y driver.
- [ ] 11.7 Conectar la app Tauri al host y conservar sus eventos `session://` como adaptador.
- [ ] 11.8 Tests de handshake, token inválido, versión incompatible, autostart concurrente, detach/reattach y caída del host.

## Fase 12 — CLI headless `acode`

Issue: [#13](https://github.com/AncletoCEO/AncletoCode/issues/13)

- [ ] 12.1 Estructura `clap` con bins `acode`/`ancleto-code`, `--help`, `--version` y árbol de subcomandos.
- [ ] 12.2 Sobre JSON versionado (`schemaVersion`, `ok`, `data`, `error`), detección de TTY, `NO_COLOR` y `--no-input`.
- [ ] 12.3 Códigos de salida `0/2/3/4/5/6` y manejo de `Ctrl+C`/`SIGTERM` sin dejar procesos hijos.
- [ ] 12.4 Comandos `project add|list|remove|open` sobre el estado compartido.
- [ ] 12.5 Comandos `agents` y `doctor` con Node ≥ 24, drivers, providers, plataforma, terminal y host.
- [ ] 12.6 Comandos `session start|list|attach|send|kill` contra el host y fallback in-process con `--no-host`.
- [ ] 12.7 `acode run` one-shot con `--driver`, `--project`, salida texto/JSON y streaming en TTY.
- [ ] 12.8 `acode converse` con objetivo, `maxTurns`, timeout, streaming y motivo de finalización.
- [ ] 12.9 `acode spec` y `acode writer` como passthrough normalizado de los providers.
- [ ] 12.10 `acode host start|stop|status` y `acode ui` para abrir la app instalada.
- [ ] 12.11 Windows: argv sin shell intermedia, VT/UTF-8 cuando esté disponible y pruebas en `cmd.exe` y PowerShell.
- [ ] 12.12 Completions de shell y documentación de comandos en `README.md`.
- [ ] 12.13 Tests de contrato de salida JSON, códigos de salida y cancelación.

## Fase 13 — AncletoCMD: emulador de terminal y gestor

Issue: [#14](https://github.com/AncletoCEO/AncletoCode/issues/14)

- [ ] 13.1 Crear `crates/ancleto-code-cmd`: ventana `winit`, superficie `wgpu`, render de texto con `glyphon`/`cosmic-text` y fallback de render por software.
- [ ] 13.2 Motor VT con `alacritty_terminal`/`vte` alimentado por el stream del host; soporte de alt screen, scroll regions, cursor, mouse y bracketed paste.
- [ ] 13.3 Grilla y scrollback: buffer circular de 10k líneas, wide chars, emoji, combining, reflow al redimensionar y búsqueda incremental.
- [ ] 13.4 Render por frame con daño parcial, frame pacing, DPI por monitor, multi-monitor, fuentes empaquetadas, fallback y zoom.
- [ ] 13.5 Layout: tabs y árbol de splits, foco, mover/redimensionar y multi-ventana opcional.
- [ ] 13.6 Entrada: teclado, IME, dead keys, layouts no latinos, mouse reporting, bracketed paste y atajos configurables.
- [ ] 13.7 Selección, copia (OSC 52/nativo), hyperlinks OSC 8, títulos saneados y confirmación para escrituras de clipboard.
- [ ] 13.8 Sidebar de proyectos, sesiones de agente, creación con selector de driver, estado en vivo, detach/attach y kill con confirmación.
- [ ] 13.9 Command palette (`Ctrl+Shift+P`) y paneles de spec, writer y conversación agente-a-agente.
- [ ] 13.10 Shell profiles (`cmd`, PowerShell, WSL, bash, zsh, fish) y hooks OSC 133 opt-in con navegación entre comandos, copia de salida y exit status; fallback heurístico en shells sin hooks.
- [ ] 13.11 Notificaciones de fin de comando, bell, indicador de tab y foco.
- [ ] 13.12 Persistencia de `TerminalLayout` y temas compartidos en SQLite; restauración al reabrir y respeto de la configuración global.
- [ ] 13.13 Benchmark de rendimiento (tecla a frame p99, idle CPU, arranque, throughput) y umbrales en CI.
- [ ] 13.14 Tests de conformidad VT con fixtures y programas reales (vim, tmux, htop, git, less), y smoke tests en Windows (`cmd`, PowerShell, WSL), Linux y macOS.
- [ ] 13.15 Manejo de cierre de ventana con detach, restauración de sesiones y "cerrar y matar sesiones" con confirmación.

## Fase 14 — Empaquetado npm y release multiplataforma

Issue: [#15](https://github.com/AncletoCEO/AncletoCode/issues/15)

- [ ] 14.1 Crear `packages/acode` (`acode`/`ancleto-code`) y `packages/acmd` (`acmd`/`ancleto-cmd`) con launchers Node mínimos.
- [ ] 14.2 Launcher: detección de plataforma/arquitectura/libc, resolución del paquete opcional, verificación de checksum y `spawn` con stdio y señales heredados.
- [ ] 14.3 Publicar paquetes por plataforma con `checksums.json`: CLI `acode` (win32 x64/arm64; linux gnu/musl x64/arm64; darwin x64/arm64) y AncletoCMD `acmd` (win32 x64/arm64; linux gnu x64/arm64; darwin x64/arm64).
- [ ] 14.4 Matriz de CI en runners nativos: musl estático sólo para la CLI y glibc para la GUI; `npm publish --provenance`.
- [ ] 14.5 Propagar una versión única a todos los paquetes y testear `acode --version` y `acmd --version` contra los `package.json`.
- [ ] 14.6 Mensajes accionables para `optionalDependencies` ausentes, checksum inválido y plataforma no soportada.
- [ ] 14.7 Incluir `LICENSE`, `THIRD_PARTY_NOTICES.md` y la declaración de fork de PandaCode en los paquetes publicados.
- [ ] 14.8 Smoke test de `npm i -g @ancleto/code` y `npm i -g @ancleto/cmd` en Windows (`cmd`/PowerShell) y Linux; abrir AncletoCMD y ejecutar `acode doctor`.
- [ ] 14.9 Documentar instalación, actualización manual y desinstalación en `README.md`.
- [ ] 14.10 Empaquetar la app de escritorio (MSI/NSIS, DMG, AppImage/DEB) y verificar `acode ui`.
- [ ] 14.11 Auditar las licencias de las dependencias gráficas y de terminal (`alacritty_terminal`/`vte`, `winit`, `wgpu`, `glyphon`, `cosmic-text`) y registrarlas en los avisos.
- [ ] 14.12 Revisión legal de la convivencia AGPL-3.0 + condición de atribución antes del primer release público.

## Fase 15 — Hardening y cierre

Issue: [#16](https://github.com/AncletoCEO/AncletoCode/issues/16)

- [ ] 15.1 Persistencia de configuración: idioma, host, tier, límites de conversación, overrides de binarios, preferencias de writer, apariencia y layout de AncletoCMD.
- [ ] 15.2 Atajos: nueva sesión, cambiar proyecto, matar sesión, iniciar conversación, rebuild de writer, cambiar tema y equivalentes en AncletoCMD.
- [ ] 15.3 Smoke manual end-to-end en las dos versiones: dos proyectos, sesiones `claude`/`opencode`/`agy`/`cmd`, detach/attach, conversación agente-a-agente, aborto, build + vista humana de writer.
- [ ] 15.4 Verificar escenarios de degradación (CLI ausente, Node ausente, host caído, GPU no disponible, sin language pack, salida no parseable, efectos de transparencia no soportados).
- [ ] 15.5 `cargo test`, `cargo clippy`, `npm run test`, `npm run typecheck` y `npm run lint` en verde.
- [ ] 15.6 Empaquetado (`npm run tauri build`, matriz npm) y verificación de cierre limpio de procesos.
