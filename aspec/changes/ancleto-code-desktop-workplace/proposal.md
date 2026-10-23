# Proposal: AncletoCode — workspace de agentes para escritorio y terminal

## Actualización de alcance — 2026-10-22

El alcance de release cambia: **sólo se implementa y publica AncletoCode de
escritorio**. Quedan fuera de esta etapa AncletoCMD, la CLI `acode`, el host como
artefacto separado y la distribución por npm. El código existente se conserva y
el host sigue siendo una dependencia interna de la app para sesiones
persistentes, pero no se empaqueta ni se valida como producto independiente.

El norte de producto pasa a ser un **híbrido visual y funcional entre PandaCode y
Orca IDE**:

- De **PandaCode** se reutiliza código y se adopta su enfoque calmo, denso y
  teclado-primero; se actualiza la referencia a `v0.3.26`
  (`800b879680bc89fda1189f726864ca1a4ed61e2a`).
- De **Orca** (`v1.4.237`, commit
  `071747853f5995ee233b36751613f09a60007c2b`, MIT) se toma la organización visual
  y de flujo: flotas de agentes, tareas con worktree propio, revisión de cambios,
  Git/GitHub/GitLab y command palette. No se copian logos, activos ni código sin
  vendorizarlo y atribuirlo a Lovecast Inc.
- Se conservan los diferenciadores Ancleto: `@ancleto/spec`, `@ancleto/writer`,
  conversación agente-a-agente y sesiones persistentes.

El release Linux usa **AppImage** (además de `.deb`) para Arch/EndeavourOS y otras
distribuciones; Windows mantiene NSIS/MSI/ZIP portable. macOS se difiere.

## Problem

Hoy los agentes de código se usan desde terminales sueltas, una por ventana y sin contexto compartido. Esto obliga a saltar entre consolas para alternar proyectos, se pierde de vista qué agente corre en cada carpeta, y no existe forma nativa de que dos agentes colaboren sobre el mismo trabajo. Además, el ecosistema Ancleto sólo se consume por CLI (`ancleto`, `ancleto-writer`), sin una experiencia de escritorio que exponga su memoria, su grafo de conocimiento y su estado de frescura.

Esa experiencia de escritorio, sola, deja afuera tres escenarios reales: servidores Linux sin entorno gráfico, sesiones remotas por SSH y automatizaciones de CI que necesitan lanzar un agente, una conversación o un `specs check` sin abrir una ventana. El usuario ya instala otras herramientas del ecosistema con `npm i -g` y espera la misma fricción cero para AncletoCode, tanto en `cmd`/PowerShell de Windows como en la shell de Linux. Sin una superficie CLI, cada automatización tendría que reinventar la detección de agentes, el ciclo de vida de procesos y la lectura de estado que la app ya resuelve.

Hay además un perfil de usuario que quiere **una terminal de verdad**, no una ventana de app: prefiere vivir en la terminal, con paneles, tabs y atajos, y dejar las sesiones de agente corriendo aunque cierre la interfaz. Hoy Warp cubre parte de esa experiencia pero es lento y no conoce el proyecto ni el SDD; los emuladores rápidos no son gestores de agentes ni de proyectos. AncletoCode debe ofrecer una **segunda versión de producto, AncletoCMD**: un emulador de terminal nativo, rápido y GPU-acelerado, que además organice proyectos y sesiones de agente como un gestor de terminales, sin exigir webview.

## Aclaraciones de alcance (corrección sobre el pedido inicial)

La investigación de los paquetes y de los CLIs deja cuatro precisiones que cambian el diseño:

1. **`agy` y `antigravity` no son dos CLIs distintos.** `agy` es el **Antigravity CLI** (binario Go, sucesor de Gemini CLI, con TUI y modo headless `agy -p`). `antigravity` es la **superficie IDE/desktop** de Antigravity, no un CLI de código. Por lo tanto hay **cuatro drivers PTY** (`claude`, `opencode`, `agy`, `cmd`) y una **integración de IDE** separada (`antigravity`), que no abre una sesión de agente sino que abre el proyecto y sincroniza los assets de skills del host.
2. **Command Code es `cmd`.** Su paquete npm es `command-code` (Node ≥ 22) y expone los bins `cmd`, `cmdc`, `commandcode` y `command-code`; tiene TUI interactiva y modo headless `cmd -p --output-format json` (NDJSON), con `--continue`/`--resume`. Se integra como cuarto driver.
3. **`@ancleto/writer` no es un motor de idiomas.** Es un **motor de conocimiento estático determinista y offline**: escanea el repo, construye un grafo de código y publica hechos en `.ancleto/writer/` (`manifest.json`, `graph.json`, `symbols.json`, `facts/*`, `agent/*`, y una vista humana `human/*`). No tiene LLM ni consume tokens. Su "paquete de idiomas" es `@ancleto/writer-languages`, un **pack opcional de lenguajes de programación** (Python, Go, Rust, C#, Java, C, C++, PHP, Ruby, Swift, Kotlin vía tree-sitter), no i18n.
4. **El idioma natural de los artefactos pertenece a `@ancleto/spec`**, no a writer: se configura en `.ancletorc` (`language: auto | es | en | pt`) y los keywords (`Requirement`, `Scenario`, `SHALL`, `WHEN`/`THEN`/`AND`) nunca se traducen.

## Análisis de referencia y reutilización de PandaCode

Se analizó [aicodeking/pandacode](https://github.com/aicodeking/pandacode) en el commit `e9f89891fd23518848765aef8219dbf39feac6e0` (v0.3.13), un workspace de escritorio para agentes de código con el mismo problema y una arquitectura muy cercana (Tauri 2 + Rust + webview nativo + SQLite). La conclusión es que **conviene reutilizar código** en las áreas donde PandaCode ya resolvió problemas difíciles (PTY, limpieza de procesos, buffers acotados, temas, contraste y Markdown), siempre bajo la puerta de cumplimiento de este change.

**Licencia detectada:** el repositorio declara la **PandaCode License (MIT License with an attribution requirement)**; GitHub la reporta como `NOASSERTION` porque no es una licencia SPDX estándar. Sus obligaciones son:

1. **MIT base**: conservar el aviso de copyright y el texto de permiso en todas las copias o porciones sustanciales del software.
2. **Atribución de fork**: si una versión modificada o una obra basada en PandaCode se usa comercialmente o se publica de cualquier forma (código, binario, app, servicio o sitio), debe declarar **"This is a fork of PandaCode"** con enlace a `https://github.com/aicodeking/pandacode`, en el README o documentación principal y en un lugar visible para el usuario (About, sitio o store listing). **Como este change incorpora código, la atribución deja de ser condicional y pasa a ser obligatoria.**
3. **Sin endoso implícito**: los nombres y logos de los proveedores pertenecen a sus dueños.

**Implicación para AncletoCode:** el proyecto es **AGPL-3.0**. Incorporar código MIT es compatible en la dirección entrante, pero la condición de atribución viaja con el código y debe preservarse en la distribución combinada. Al incorporar código, el proyecto pasa a ser una obra basada en PandaCode: se conservan `LICENSE` (AGPL-3.0), los avisos MIT y la declaración de fork. La convivencia se documenta como término adicional permitido por AGPL §7 y se somete a revisión legal antes del primer release público.

**Decisión de alcance:** se reutiliza el código de PandaCode que mapea a las features del MVP, no todo el repositorio. La incorporación es selectiva, vendorizada, con procedencia registrada y cabeceras SPDX; los subsistemas fuera de alcance (bots, cloud, VMs, board, simuladores, equipos arquitecto/workers) quedan como referencia, sin copia; Git/worktrees/GitHub/GitLab pasan a estar en alcance 0.2. Los activos (wallpapers, logos, screenshots, iconos, fuentes y temas) no se reutilizan.

### Estrategia de reutilización

- **Nivel A — adoptar con cambios mínimos**: módulos autocontenidos de alta calidad que no dependen del modelo de dominio de PandaCode. Se copian bajo `third_party/pandacode/`, con cabecera SPDX, commit de origen y tests portados.
- **Nivel B — adaptar**: módulos valiosos pero acoplados a eventos, stores o providers de PandaCode. Se copian y se adaptan a nuestro contrato (`AgentDriver`, `SessionManager`, stores React), registrando el diff en `docs/provenance.md`.
- **Nivel C — referencia sin copia**: patrones de esquema/migraciones, capa ACP, orquestación de equipos, colas, turn changes y subsistemas fuera del MVP. Se leen para decisiones de diseño y se citan en el ledger.
- **No reutilizable**: wallpapers, temas, logos de proveedores, screenshots, iconos, marca y textos de documentación. Nuestros activos se generan o se obtienen de fuentes con licencia compatible.

### Comparación de features

| Área de AncletoCode | Enfoque de PandaCode | Decisión para AncletoCode |
|---|---|---|
| Shell de escritorio | Tauri 2, Rust, TypeScript DOM plano, SQLite, sin Node embebido | Reusar el patrón de webview nativo + núcleo Rust + SQLite; mantener React 19 por decisión propia |
| Proveedores | Interfaz normalizada de eventos con adapters por protocolo (Claude stream-json, Codex app-server, ACP para Kimi/OpenCode/Antigravity/Grok), proceso por turno y resume por ID | Reusar `text_stream.rs` y `windows_job.rs`; mantener el contrato PTY-first con one-shot estructurado y dejar ACP como referencia futura |
| Antigravity | Distingue el CLI `agy` del servidor ACP `agy_acp_server` y no confunde el TUI con ACP | Reusar la distinción: `agy` es driver PTY; el IDE es integración separada |
| Ciclo de vida | Proceso sólo durante el turno, grupos de procesos, límites de salida y stderr, cierre limpio | Reusar `terminal.rs` (adaptado), `windows_job.rs` y `text_stream.rs` para buffers acotados y cierre ordenado |
| Workspace | Proyectos, archivos, Git, worktrees, branch chip y browser embebido | Reusar/adaptar `workspace.rs` y `file_search.rs`; Git/worktrees/branch chip pasan a estar en alcance 0.2; browser queda fuera |
| Tabs y navegación | Sidebar global agrupada por recencia; cada tarea es un thread | Reusar `transcript-scroll.ts` y `ui-size.ts`; mantener tabs proyecto → agente con indicadores propios |
| SDD y conocimiento | Sin equivalente: no consume `aspec` ni `ancleto-writer` | Diferenciador propio; se conserva el diseño de providers Ancleto y no se reutiliza código |
| Aprobaciones | Barra de aprobaciones con approve/deny y agrupación | Fuera del MVP: cada CLI conserva su modelo de permisos dentro del PTY; sólo referencia |
| Tareas y cola | Board durable, cola de mensajes, steering y aprobaciones | Fuera del MVP; se documenta como evolución posible y no se copia |
| Equipos de agentes | Arquitecto + hasta tres workers con perfiles y watchdog | Nuestra conversación de dos agentes es el MVP; el patrón arquitecto/workers queda como referencia, sin copia |
| Transparencia | Vidrio nativo en macOS, vibrancy, fallback opaco, reduced transparency/high contrast, sin shaders ni JS por frame | Reusar `appearance.rs` y `appearance.ts` para macOS/fallback; implementar Mica/Acrylic en Windows con la API de Tauri |
| Temas y fondos | Wallpapers originales generados, schedules, acentos derivados, importación de temas VS Code, tests de contraste | Reusar `color.ts`, `palette-core.ts` y `vsix_themes.rs`; adaptar la UI; **no** copiar wallpapers, temas ni arte |
| Persistencia | SQLite en app data, paginación acotada, migraciones versionadas | Reusar los patrones de migración y paginación; no copiar `storage.rs` completo |
| Seguridad | Rutas canónicas, argv sin shell, Markdown saneado, CSP, secretos en keychain, sin telemetría | Reusar `workspace.rs` y `file_preview.rs` para rutas/lecturas; adoptar Markdown saneado y CSP en el MVP |

## Proposed change

Construir **AncletoCode**: una aplicación de escritorio en **Tauri 2** que funciona como *workplace* de agentes de código.

- **Vista de proyecto**: cada proyecto abierto es un workspace con su ruta, su árbol de archivos, su estado SDD y su estado de conocimiento.
- **Tabs por proyecto y por agente**: pestañas de primer nivel por proyecto y, dentro de cada proyecto, una pestaña por sesión de agente.
- **Cuatro drivers PTY**: `claude code` (`claude`), `opencode` (`opencode`), `agy` (Antigravity CLI) y `cmd` (Command Code), con detección de binario, versión y capacidades declaradas (TUI interactiva y modo one-shot/headless cuando existe).
- **Integración con Antigravity IDE**: abrir el proyecto en la app Antigravity y materializar sus assets (`.agents/skills`, `.agents/agents`, `.agents/mcp_config.json`) vía `@ancleto/spec --agent antigravity`.
- **Interacción entre dos agentes**: conversación por turnos entre dos sesiones, usando modo one-shot estructurado cuando el driver lo soporta y PTY interactivo como fallback.
- **Proveedor `@ancleto/spec`**: inicialización/actualización del proyecto (host, tier, idioma), explorador de `aspec/`, estado de discovery con `impact`, memoria del proyecto, `specs check` y `doctor`.
- **Proveedor `@ancleto/writer`**: build/check/lint/impact/human/doctor, panel de conocimiento (god nodes, hotspots, ciclos, capas, blast radius) y vista humana embebida; detección y control opcional del pack `@ancleto/writer-languages`.
- **Apariencia CLI**: estética de terminal (tipografía monoespaciada, chrome tipo prompt, densidad compacta) con **transparencias reales de ventana y de paneles**, y **colores y fondos configurables** por tema y por proyecto.
- **CLI headless `ancleto-code`** (alias `acode`): la misma funcionalidad de proyectos, sesiones, drivers, conversación y providers, ejecutable desde `cmd`/PowerShell de Windows y desde la shell de Linux o macOS, instalable con `npm i -g @ancleto/code`.
- **AncletoCMD**: segunda versión del producto, un **emulador de terminal nativo** con render GPU, tabs, splits, paneles, scrollback y búsqueda, más un **gestor de terminales** con sidebar de proyectos y sesiones de agente, paleta de comandos y atajos. Es la alternativa rápida a Warp, sin webview y con integración Ancleto.
- **Núcleo compartido**: extraer el dominio, el motor PTY, los drivers, el orquestador, los providers y la persistencia a un crate reutilizable por la app de escritorio, AncletoCMD y el binario CLI, sin duplicar lógica ni contratos.
- **Sesiones persistentes**: un host local liviano es dueño de los PTY y permite que AncletoCMD se cierre y se vuelva a adjuntar sin matar a los agentes ni a los shells; la app de escritorio y la CLI se conectan al mismo host.
- **Elección de producto**: el usuario puede usar la app gráfica AncletoCode, AncletoCMD o la CLI headless sobre el mismo proyecto, la misma configuración y las mismas sesiones, sin migrar nada.
- **Experiencia de terminal**: subcomandos no interactivos con `--json` y códigos de salida estables para scripts y CI, además de `attach` con streaming en vivo.
- **Distribución npm multiplataforma**: un paquete `@ancleto/code` con launcher Node y binarios nativos por plataforma como `optionalDependencies`, al estilo de `opencode`/`esbuild`, con checksums y verificación de plataforma.
- **Estado compartido**: las dos versiones y la CLI usan la misma base SQLite y la misma configuración, de modo que proyectos, sesiones y conversaciones se ven desde todas.
- **Paridad de plataformas**: Windows x64/arm64 con ConPTY y `cmd`/PowerShell; Linux x64/arm64 (incluido musl estático); macOS x64/arm64.
- **Reutilización de PandaCode bajo compliance**: incorporación selectiva de módulos de PTY, procesos, workspace, temas, contraste, Markdown y utilidades de UI, con procedencia, avisos y atribución obligatoria.
- **Cumplimiento de licencia y procedencia**: política de reutilización, registro de procedencia, `third_party/pandacode/LICENSE`, `THIRD_PARTY_NOTICES.md`, atribución en README/About y requisitos verificables.

### Dos versiones del producto, un solo núcleo

**AncletoCode (escritorio)** y **AncletoCMD (terminal)** son dos versiones del mismo producto, como la app de escritorio y la CLI de `opencode`. La CLI headless no es una tercera versión de usuario sino la superficie de automatización que ambas comparten.

- **Un solo núcleo, tres consumidores**: el código Rust de dominio, PTY, drivers, orquestador, providers y persistencia se mueve a `crates/ancleto-code-core`; la app Tauri (`crates/ancleto-code-tauri`), AncletoCMD (`crates/ancleto-code-cmd`) y el binario CLI (`crates/ancleto-code-cli`) pasan a ser consumidores delgados. Ni React, ni AncletoCMD, ni la CLI implementan reglas de negocio.
- **Host de sesiones**: un proceso local (`ancleto-code-host`) es el único dueño de los PTY. Habla JSON-RPC por named pipe en Windows y Unix domain socket en Linux/macOS, con token por usuario. Todos los clientes se conectan al mismo host, de modo que una sesión abierta en AncletoCMD se ve y se adjunta desde la app, y una sesión lanzada por la CLI sobrevive al cierre de la terminal. Sin host disponible, la CLI puede operar en modo one-shot in-process (`--no-host`).
- **AncletoCMD es un emulador, no una TUI**: ventana nativa propia con `winit` y render GPU (`wgpu` + `glyphon`/`cosmic-text`), parsing VT con `alacritty_terminal`/`vte`. No se ejecuta dentro de otra terminal ni depende de un webview.
- **AncletoCMD como gestor**: además de shells arbitrarios (`cmd`, PowerShell, WSL en Windows; bash/zsh/fish en Linux/macOS), organiza proyectos, tabs, splits y sesiones de agente, con paleta de comandos, búsqueda y persistencia de layout.
- **Por qué binario nativo y no una CLI en Node**: el motor PTY, la limpieza de procesos y la detección de binarios ya existen en Rust y son la parte difícil. Reescribirlos en Node duplicaría lógica y crearía dos fuentes de verdad. npm actúa como **canal de distribución**, igual que `opencode` o `esbuild`: un launcher resuelve el binario de la plataforma y lo ejecuta.
- **Requisito de runtime**: AncletoCMD y `acode` no requieren Node para arrancar; Node ≥ 24 sigue siendo necesario sólo para las CLIs del ecosistema Ancleto (`ancleto`, `ancleto-writer`) que la herramienta invoca.
- **Comandos previstos**:

  | Comando | Propósito |
  |---|---|
  | `ancleto-cmd` / `acmd` | Abrir AncletoCMD; `--project`, `--shell`, `--attach <session>`, `--theme` |
  | `acode` | Ayuda y comandos headless (no abre la GUI) |
  | `acode run "<prompt>"` | Ejecución one-shot con un driver; `--json`, `--driver`, `--project` |
  | `acode project add\|list\|remove\|open` | Gestión de proyectos compartida entre superficies |
  | `acode agents` | Detección de drivers y capacidades |
  | `acode session start\|list\|attach\|send\|kill` | Ciclo de vida PTY y attach a una sesión |
  | `acode converse` | Conversación agente-a-agente con límites y streaming |
  | `acode spec <cmd>` / `acode writer <cmd>` | Passthrough normalizado de los providers Ancleto |
  | `acode doctor` | Estado de Node, CLIs, providers, plataforma, host y persistencia |
  | `acode host start\|stop\|status` | Control del host de sesiones |
  | `acode ui` | Abrir la app de escritorio si está instalada |

- **Contrato para automatización**: toda salida legible por máquina va por `--json`, sin ANSI cuando la salida no es un TTY, sin prompts interactivos cuando se pasa `--no-input`, y con códigos de salida documentados (`0` ok, `2` uso inválido, `3` dependencia ausente, `4` error de sesión, `5` estado incompatible, `6` cancelado).
- **Concurrencia de superficies**: la app y AncletoCMD comparten el mismo host y la misma base SQLite en WAL, con el host como escritor único; una sesión creada en cualquiera es visible y adjuntable desde las demás. Si el host muere, las sesiones se marcan `crashed` en el siguiente arranque y ambas versiones ofrecen recrearlas.
- **Alcance de AncletoCMD**: emulación VT completa, tabs y splits, paneles, scrollback con búsqueda, selección y copia, mouse, hyperlinks OSC 8, bracketed paste, integración de shell con marcas OSC 133 (prompt/comando/salida), paleta de comandos, sidebar de proyectos, sesiones de agente y temas de la app. Gestión avanzada de bloques tipo Warp (acciones por bloque, workflows, autocomplete con IA) queda fuera del MVP.
- **Compatibilidad de terminales**: AncletoCMD es la terminal, así que el soporte se mide en Windows (ConPTY, `cmd`, PowerShell, WSL; Windows Terminal no aplica) y Linux/macOS (PTY, shells comunes, IME y clipboard). No hay dependencia de `TERM` del host.

## Scope

> **Vigencia 2026-10-22:** el alcance de release es sólo la app de escritorio.
> Los bullets de CLI, AncletoCMD y npm se conservan como diseño futuro y no son
> criterio de aceptación de esta etapa. Git/worktrees/GitHub/GitLab y AppImage
> pasan a estar en alcance.

**In scope**

- Shell Tauri 2 con backend Rust y frontend React + TypeScript + Vite.
- Núcleo Rust compartido en un Cargo workspace (`ancleto-code-core`) consumido por la app Tauri, AncletoCMD y el binario CLI.
- CLI `ancleto-code` (alias `acode`) con comandos de proyectos, agentes, sesiones, conversación, providers y `doctor`, más `attach`.
- **AncletoCMD**: ventana nativa con `winit` + `wgpu`/`glyphon`, parsing VT (`alacritty_terminal`/`vte`), PTY vía ConPTY/PTY nativo, tabs, splits, paneles, scrollback, búsqueda, selección/copia, mouse, hyperlinks, bracketed paste, IME y paleta de comandos.
- **Gestor en AncletoCMD**: sidebar de proyectos, sesiones de agente, creación/renombrado/cierre, detach/attach, persistencia de layout y paneles de estado de spec/writer.
- **Integración de shell**: marcas OSC 133 (prompt, inicio/fin de comando, exit status), navegación entre comandos y copia de la salida de un comando.
- **Temas compartidos**: paletas ANSI derivadas de los manifiestos de tema de la app, sin transparencias ni wallpapers.
- **Rendimiento**: render GPU por frame, scrollback eficiente, latencia de input medida y fallback de render por software cuando no hay GPU compatible.
- Host local de sesiones (`ancleto-code-host`) con IPC por named pipe/Unix socket, token por usuario, autostart, límite de inactividad y recuperación de sesiones huérfanas.
- Salida `--json`, comportamiento no interactivo (`--no-input`), detección de TTY y códigos de salida estables para scripts y CI.
- Paquetes npm `@ancleto/cmd` (AncletoCMD) y `@ancleto/code` (CLI) con launcher y binarios por plataforma en `optionalDependencies`, checksums y verificación de plataforma.
- Matriz de builds: Windows x64/arm64 (ConPTY, `cmd`, PowerShell y WSL), Linux x64/arm64 (gnu; musl estático sólo para la CLI) y macOS x64/arm64.
- Persistencia compartida entre superficies: misma base SQLite en WAL, misma configuración y reconciliación de sesiones huérfanas.
- Tests de paridad entre app, AncletoCMD y CLI sobre el mismo núcleo, y smoke tests de instalación global en Windows y Linux.
- Documentación de instalación y referencia de comandos en `README.md`, más `acode --help` completo y completions de shell.
- Motor de sesiones PTY con ciclo de vida completo y cierre ordenado.
- Drivers: `claude`, `opencode`, `agy` y `cmd` (Command Code), con perfil de arranque, detección y capacidades.
- Integración de IDE Antigravity como acción de proyecto (no como driver PTY).
- Tabs de proyecto y de agente con estado en vivo.
- Conversación agente-a-agente con turnos, límites, aborto y registro.
- Proveedor `@ancleto/spec`: init/install/update, lectura de `.ancletorc`, cambios, memoria, discovery, `specs check`, `doctor`, tiers y `--lang`.
- Proveedor `@ancleto/writer`: `init`, `build`, `check`, `lint`, `impact`, `human`, `doctor`; lectura de `manifest.json`/`graph.json`/`facts/*`; vista humana embebida; detección del language pack.
- Detección de Node ≥ 24 y de cada binario, con estado `available`/`missing`/`error` y comando de instalación sugerido.
- Sistema de apariencia: tokens de color, temas incluidos, fondos (sólido, degradado, imagen o wallpaper de terminal), transparencia/opacidad de ventana y paneles, y persistencia por proyecto.
- Análisis comparativo de PandaCode y política de reutilización, con registro de qué se adoptó, qué se adaptó y qué se reimplementó.
- Incorporación selectiva de código de PandaCode en `third_party/pandacode/`: núcleo PTY, limpieza de procesos, buffers de stream, workspace/path scoping, temas/contraste, markdown y utilidades de UI.
- Portado de tests de los módulos reutilizados, con cabeceras SPDX y cobertura equivalente.
- Artefactos de cumplimiento: `third_party/pandacode/LICENSE`, `THIRD_PARTY_NOTICES.md`, `docs/provenance.md`, sección de licencias y atribuciones en `README.md`, línea de atribución obligatoria en el About, y checklist de auditoría de activos.
- Verificación automatizada de que las obligaciones de atribución, los avisos de licencia y las dependencias cumplen.

**Out of scope**

- AncletoCMD, CLI `acode`, host como artefacto separado y paquetes npm durante esta etapa (código conservado, release diferido).

- Ejecución remota, multiusuario o sincronización en la nube.
- Edición colaborativa en tiempo real entre humanos.
- Marketplace de agentes o plugins de terceros.
- Otros CLIs fuera de `claude`, `opencode`, `agy` y `cmd` (el contrato de driver queda abierto).
- Reimplementar la memoria, el discovery o el grafo dentro de la app: se consumen las CLIs de Ancleto como fuente de verdad.
- Instalar dependencias automáticamente (sólo se sugiere el comando npm).
- Autenticación de las CLIs (se delega en cada CLI ya autenticada en la máquina).
- Copiar código o activos de PandaCode fuera del flujo vendorizado y sin la puerta de cumplimiento.
- Incorporar subsistemas completos fuera del MVP (bots, cloud, VMs, board, simuladores, teams) aunque su código esté disponible. GitHub/GitLab **sí** forman parte del nuevo alcance de escritorio.
- Reclasificar la licencia del proyecto o relicenciar porciones de terceros sin conservar sus avisos.
- **Bloques avanzados estilo Warp**: acciones por bloque, workflows, autocomplete con IA, compartir bloques y Agent Mode propio. El MVP incluye marcas OSC 133 y navegación/copia por comando, no la capa de producto completa de Warp.
- **Emulador de terminal dentro de la app de escritorio**: AncletoCMD es la versión terminal; la app usa xterm.js para las sesiones, no un emulador nativo embebido.
- Modo servidor o daemon de red (`serve`, HTTP, WebSocket), terminal web y clientes de editor; el host local por IPC sí forma parte del alcance.
- Servidor remoto, SSH multiusuario o ejecución en contenedores gestionados por la herramienta.
- Gestores de paquetes distintos de npm (Homebrew, Scoop, Chocolatey, AUR, winget) y auto-actualización del binario.
- Plataformas fuera de la matriz: Windows 32-bit, Linux armv7, FreeBSD, Android/Termux.
- Embeber la app de escritorio dentro de AncletoCMD/CLI o viceversa; `acode ui` sólo lanza la app instalada.

## Risks

- **`agy` es un binario Go con TUI y auto-updater propio y puede cambiar sin aviso**: contrato de driver desacoplado, modo headless (`agy -p --output-format json`) como camino preferido para turnos, y fallback a PTY crudo.
- **Parsear TUIs es frágil**: el PTY es la fuente de verdad; el parseo sólo alimenta estado/UX y nunca bloquea la operación.
- **Duplicar la lógica de frescura del grafo**: la app no recalcula fingerprints; usa `ancleto-writer check --json` y respeta `schemaVersion`.
- **Acoplamiento a formatos de artefactos de writer (schemaVersion 1)**: encapsular la lectura en un adapter y degradar a "conocimiento no disponible" si el esquema es incompatible.
- **Variabilidad del pack de lenguajes (peers nativos de tree-sitter, versión independiente)**: tratarlo como opcional, detectar su ausencia y no requerirlo para operar.
- **Bucles o costo descontrolado en la conversación agente-a-agente**: límite duro de turnos, timeout por turno, detección de repetición y aborto manual.
- **Dependencia de Node ≥ 24 para todo el ecosistema Ancleto**: chequeo explícito al iniciar y estado `missing` accionable.
- **Crecimiento de alcance del MVP**: cerrar contrato de driver, orquestador y adapters de providers antes de sumar integraciones.
- **Transparencia real y rendimiento en Windows (Mica/Acrylic/blur)**: usar efectos nativos con fallback a capas CSS translúcidas, permitir desactivar el blur por GPU y medir el costo de render del terminal.
- **Legibilidad con fondos e imágenes personalizados**: validar contraste de texto/estado contra el fondo parado, ofrecer presets accesibles y ajuste automático de luminosidad.
- **Deriva del código vendorizado**: fijar el commit, mantener `third_party/pandacode/UPSTREAM.md` y un script de sync/diff; no editar sin registrar el diff.
- **Avisos de licencia desactualizados al agregar un módulo o dependencia**: hacer que `license:check` y la auditoría de dependencias sean bloqueantes en CI.
- **Atribución de fork omitida en una release pública**: declaración obligatoria en README y About, verificada por CI antes de cada release.
- **Incompatibilidad entre AGPL-3.0 del proyecto y la condición adicional de PandaCode**: tratar la atribución como término adicional permitido por AGPL §7, conservar ambos avisos y completar revisión legal antes del primer release.
- **Dependencias transitivas con licencias incompatibles**: auditar con `cargo deny` y un revisor de licencias npm; bloquear el release si aparece una licencia no permitida.
- **Sobreacoplamiento al modelo de PandaCode**: reutilizar módulos acotados detrás de adapters propios; no importar stores, tipos ni providers completos.
- **Reutilización accidental de activos no cubiertos o de marcas de proveedores**: inventario de activos originales, verificación de hashes/origen y prohibición de logos/wallpapers de terceros.
- **Extracción incompleta del núcleo y duplicación de lógica**: cerrar la extracción a `ancleto-code-core` antes de escribir la CLI, con tests que prohíban dependencias de Tauri en el core y un único owner por regla de negocio.
- **Diferencias de ConPTY y de terminal entre `cmd`, PowerShell, WSL y shells de Linux**: matriz de smoke tests por shell, tests de PTY multiplataforma y detección de capacidades en runtime.
- **Fallos de `optionalDependencies` en npm (proxy, plataforma no soportada, instalación parcial)**: launcher con mensaje accionable, verificación de checksum, `acode doctor` que reporta la causa y fallback claro a instalación manual del binario.
- **Binarios nativos bloqueados por antivirus o SmartScreen en Windows**: firma de código cuando sea posible, documentación de excepciones y build reproducible.
- **Incompatibilidad glibc/musl en Linux para los binarios GUI**: publicar builds ligados a glibc para AncletoCMD y musl sólo para la CLI; validar en distros glibc y musl.
- **Deriva de versión entre launchers npm, binarios por plataforma y app de escritorio**: versión única derivada del workspace, publicación atómica y test que compara `acode --version` y `ancmd --version` con los paquetes.
- **Acceso concurrente de CLI, AncletoCMD y app a la misma SQLite**: WAL, timeouts acotados, migraciones idempotentes y recuperación de sesiones huérfanas al iniciar cualquiera de las superficies.
- **Complejidad del host de sesiones y del IPC local**: protocolo versionado, socket/pipe con permisos del usuario y token, autostart idempotente, límite de inactividad y tests de compatibilidad entre versiones de cliente y host.
- **Divergencia de comportamiento entre AncletoCMD y la app gráfica**: toda regla vive en el core y se comparte por el host; AncletoCMD sólo renderiza, y los tests de paridad comparan estados de ambos clientes sobre el mismo host.
- **Corrección del emulador VT (secuencias de escape, wide chars, emoji, combinación, reflow)**: apoyarse en `alacritty_terminal`/`vte` en lugar de escribir el parser, con suite de conformidad y fixtures de programas reales (vim, tmux, htop, git).
- **GPU y drivers gráficos heterogéneos en Windows/Linux/macOS**: detectar adaptador y capacidades, fallback a render por software, limitar efectos y probar en iGPU/dGPU y sesiones remotas.
- **Latencia de input y render por debajo de Warp**: medir p50/p99 de tecla a frame, render por frame con daño parcial, evitar reparses innecesarios y presupuestar memoria del scrollback.
- **IME y teclado internacional en Windows/Linux/macOS**: probar composición, dead keys, layouts no latinos y pegado con bracketed paste; no interceptar atajos del sistema.
- **Integración de shell incompleta (cmd, PowerShell, bash, zsh, fish, WSL)**: marcas OSC 133 con fallback a detección heurística, y degradar sin romper la terminal cuando el shell no coopera.
- **Seguridad de la terminal (OSC 52, OSC 8, title, clipboard, hyperlinks)**: sanear secuencias peligrosas, preguntar antes de escribir clipboard y no ejecutar hyperlinks automáticamente.
- **Complejidad de una ventana nativa propia (DPI, multi-monitor, accesibilidad)**: usar `winit` con escala por monitor, tamaños configurables y auditoría de contraste; la accesibilidad de lector de pantalla se documenta como limitada en el MVP.
- **Pérdida de sesiones al reiniciar AncletoCMD o al caerse el host**: detach sin matar sesiones, estado `crashed` reconciliado al arrancar y opción de recrear sesión con el mismo `cwd` y driver.
- **Conflictos de atajos con el shell y el sistema**: keybindings configurables, detección de leader y documentación por plataforma.
- **Contratos de automatización rotos por cambios de salida o de códigos de salida**: esquemas JSON versionados y tests de contrato en CI.
- **Problemas de encoding y quoting en `cmd`/PowerShell (UTF-8, CRLF, comillas)**: usar VT/UTF-8 cuando esté disponible, escapar argumentos sin shell intermedio y probar rutas con espacios y caracteres no ASCII.
