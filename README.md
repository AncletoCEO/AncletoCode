# AncletoCode

Workspace de agentes de código con **dos versiones**: la app de escritorio **AncletoCode** (Tauri 2) y **AncletoCMD**, un emulador de terminal nativo, rápido y GPU-acelerado que además funciona como gestor de proyectos y sesiones. Ambas comparten el mismo núcleo, la misma configuración y las mismas sesiones, junto con una CLI headless para automatización.

> **Estado:** `0.0.2`, release desktop-only para Linux y Windows. Incluye UI
> híbrida PandaCode + Orca, Git/worktrees, GitHub y Ancleto SDD embebido. La
> revisión técnica de licencias está completada; para un release estable se
> recomienda revisión legal externa. Ver [`docs/release-plan.md`](docs/release-plan.md)
> y [`docs/release-notes/0.0.2.md`](docs/release-notes/0.0.2.md).
>
> AncletoCMD, CLI `acode`, npm y AUR quedan fuera de este release.


## Qué va a hacer

AncletoCode es un *workplace* para trabajar con varios agentes de código a la vez, organizados por proyecto, sin saltar entre terminales. Se puede usar desde la app de escritorio, desde AncletoCMD o desde la CLI, sobre el mismo estado.

### Visión de producto: dos versiones y una CLI (CMD/CLI fuera de este release)

| Versión | Superficie | Para quién |
|---|---|---|
| **AncletoCode** | App de escritorio con sidebar, tabs, paneles y xterm.js | Quien quiere una vista completa del proyecto, el SDD y el conocimiento |
| **AncletoCMD** | Emulador de terminal nativo con ventana propia, tabs y splits | Quien vive en la terminal y quiere velocidad, sin webview |
| **CLI `acode`** | Comandos headless con `--json` | Scripts, CI, servidores y automatización |

- Las tres superficies usan el mismo núcleo Rust, la misma base SQLite y el mismo **host de sesiones**.
- Cerrar la app o AncletoCMD hace *detach*: los agentes y las terminales siguen vivos y se pueden reenganchar.
- Una sesión creada en AncletoCMD es visible desde la app y desde la CLI, y viceversa.

### Proyectos y tabs

- Vista por proyecto con ruta, árbol de archivos, estado SDD y estado de conocimiento.
- **Tabs de primer nivel por proyecto** y, dentro de cada proyecto, **una tab por sesión de agente**.
- Estado en vivo por sesión: `starting`, `ready`, `busy`, `waiting_input`, `detached`, `stopped`, `crashed`.
- Las sesiones siguen corriendo al cambiar de tab y al cerrar la interfaz. Si el host se reinicia, sus procesos mueren: las sesiones se marcan `crashed` al arrancar y se pueden recrear con el mismo proyecto y driver.

### AncletoCMD: la versión terminal

AncletoCMD es una **terminal real**, no una TUI dentro de otra terminal. La referencia de experiencia es Warp; la de rendimiento, Alacritty/Kitty.

- **Ventana nativa** con `winit` y render GPU (`wgpu` + `glyphon`), parsing VT con `alacritty_terminal`/`vte` y PTY vía ConPTY/PTY nativo.
- **Emulación completa**: 16/256/truecolor, alt screen, mouse, bracketed paste, OSC 8, wide chars, emoji, combinación, reflow al redimensionar y scrollback de 10k líneas.
- **Gestor de terminales**: sidebar de proyectos, tabs, splits horizontales/verticales, búsqueda en scrollback, selección/copia, command palette y atajos configurables.
- **Sesiones de agente** en paneles, con detach/attach y el mismo estado que la app.
- **Integración de shell** con marcas OSC 133 en bash, zsh, fish y PowerShell: navegación entre comandos, exit status y copia de la salida de un comando. En `cmd.exe` hay fallback heurístico.
- **Temas compartidos** con la app (sólo paleta y tipografía, sin transparencias ni wallpapers).
- **Seguridad**: OSC 52 pide confirmación antes de escribir el clipboard y los hyperlinks OSC 8 no se abren solos.
- **Rendimiento objetivo**: p99 de tecla a frame < 30 ms, idle < 1 % de CPU y arranque < 300 ms, con fallback a render por software si no hay GPU compatible.
- **Plataformas**: Windows (ConPTY, `cmd`, PowerShell, WSL), Linux y macOS, x64 y arm64.

### CLI headless y sesiones persistentes

- **`acode`** ejecuta agentes, proyectos, sesiones, conversaciones y providers sin abrir interfaz gráfica.
- **Host local de sesiones** (`ancleto-code-host`): dueño de los PTY, con IPC por named pipe/Unix socket, token por usuario y autostart bajo demanda.
- **Salida para scripts**: `--json` con `schemaVersion`, `--no-input`, ausencia de ANSI cuando no hay TTY y códigos de salida estables (`0` ok, `2` uso inválido, `3` dependencia ausente, `4` error de sesión, `5` estado incompatible, `6` cancelado).
- **Modo sin host**: `acode run --no-host` ejecuta un one-shot in-process sin crear sesión persistente.

### CLIs de código soportadas

| CLI | Binario | Interactivo | One-shot |
|---|---|---|---|
| Claude Code | `claude` | TUI | `claude -p` con salida JSON |
| OpenCode | `opencode` | TUI | `opencode run` |
| Antigravity CLI | `agy` | TUI | `agy -p --output-format json` |
| Command Code | `cmd` | TUI | `cmd -p --output-format json` (NDJSON) |

- `agy` es el CLI de Antigravity; **Antigravity IDE** se integra aparte (abrir proyecto y sincronizar skills), no como sesión PTY.
- Command Code se instala con `npm i -g command-code` y requiere Node ≥ 22.
- Todos los drivers corren sobre un motor PTY común; si el parseo de salida falla, la sesión sigue funcionando en modo crudo.

### Dos agentes interactuando

- Se eligen dos sesiones (del mismo proyecto o de proyectos distintos) y un objetivo.
- La conversación alterna turnos con **límite configurable**, timeout por turno, detección de repetición, pausa y aborto manual.
- Prefiere ejecución one-shot estructurada por turno y usa inyección PTY como fallback.
- Cada turno queda registrado con origen, destino, modo, texto, duración y timestamps.

### Ecosistema Ancleto

- **`@ancleto/spec`** (`ancleto` / `aspec`): inicialización del proyecto, host de instalación, tier (`normal`, `minimo`, `gratis`), idioma de artefactos (`auto`, `es`, `en`, `pt`), explorador de `aspec/changes`, estado de discovery con `impact`, memoria del proyecto y `specs check`.
- **`@ancleto/writer`** (`ancleto-writer`): motor de conocimiento estático y determinista. Expone `build`, `check`, `lint`, `impact`, `human` y `doctor`, junto con el grafo de código, god nodes, hotspots, ciclos, capas, blast radius y la vista humana HTML.
- **`@ancleto/writer-languages`**: pack **opcional de lenguajes de programación** (Python, Go, Rust, C#, Java, C, C++, PHP, Ruby, Swift y Kotlin). No es un paquete de idiomas naturales; el idioma de los artefactos se configura en `@ancleto/spec`.
- La app y AncletoCMD no reimplementan ninguna de estas capacidades: las consumen como fuente de verdad. Tampoco escriben en `aspec/`, `AGENTS.md`, `PRODUCT.md` ni en `.ancleto/writer/`.

### Apariencia CLI

- Estética de terminal: tipografía monoespaciada, densidad compacta, glifos de prompt y statusline.
- **Transparencias** de ventana y paneles (Mica/Acrylic/Blur en Windows, vibrancy en macOS, compositor en Linux) con fallback CSS y modo opaco. Exclusivo de la app de escritorio.
- **Colores configurables** mediante temas y tokens semánticos; la paleta ANSI de xterm, la de AncletoCMD y la de la UI se derivan del mismo tema.
- **Fondos configurables** (sólido, degradado, imagen con blur/dim o patrón sutil) en la app; AncletoCMD prioriza latencia y sólo usa paleta y tipografía.
- Precedencia global → workspace → proyecto, con validación de contraste WCAG AA.
- La configuración de apariencia se guarda en la app, nunca en el repositorio del proyecto.

## Arquitectura

Es un Cargo workspace: un núcleo compartido, un host de sesiones y tres consumidores.

```
┌────────────────────┐  ┌────────────────────┐  ┌────────────────────┐
│ AncletoCode        │  │ AncletoCMD         │  │ CLI `acode`        │
│ React + Tauri      │  │ winit + wgpu       │  │ clap + --json      │
└─────────▲──────────┘  └─────────▲──────────┘  └─────────▲──────────┘
          └───────────────┬───────┴───────────────────────┘
                          │ IPC local (pipe / Unix socket + token)
                ┌─────────┴──────────┐
                │ Host de sesiones   │  PTY · ring buffers · attach
                └─────────┬──────────┘
                          │
                ┌─────────┴──────────────────────────────┐
                │ Núcleo Rust (`ancleto-code-core`)      │
                │ Sesiones · Drivers · Conversaciones    │
                │ Providers spec/writer · SQLite         │
                └────────────────────────────────────────┘
```

- **Núcleo Rust**: procesos PTY (`portable-pty`), detección de binarios, ejecución de CLIs, persistencia y seguridad. No conoce Tauri, `winit` ni la CLI.
- **Host**: único dueño de los PTY; las tres superficies se conectan como clientes y comparten sesiones.
- **AncletoCode**: frontend React que no ejecuta procesos ni accede al filesystem.
- **AncletoCMD**: emulador de terminal nativo que renderiza el stream del host.
- **CLI**: comandos headless para scripts, CI y administración.
- **Local-first**: sin nube, sin telemetría y sin sincronización remota.

## Requisitos

### Para usar el release

- **Linux x86_64:** WebKitGTK 4.1, GTK 3, libsoup 3 y librsvg. El `.deb` se
  construye sobre Ubuntu 22.04; en Arch/EndeavourOS usá la **AppImage**.
- **Windows 10/11 x64:** WebView2. No hace falta instalar Node ni npm: Ancleto
  SDD viaja incluido en la app.
- **Agentes:** las CLIs que quieras usar (`claude`, `opencode`, `agy`, `cmd`) se
  instalan y autentican por separado. `acode`/npm/AncletoCMD **no forman parte
  de este release**.
- **macOS:** diferido.

### Para compilar

```bash
npm ci
npm run prepare:ancleto   # Node + @ancleto/spec embebidos
npm run tauri dev
```

El release se construye con `npm run tauri -- build --bundles deb,appimage` en
Linux y `--bundles nsis,msi` en Windows.

## Instalación 0.0.2

Todos los artefactos están en
[Releases v0.0.2](https://github.com/AncletoCEO/AncletoCode/releases/tag/v0.0.2).
Verificá `SHA256SUMS` antes de instalar.

### Linux — Debian/Ubuntu

```bash
sudo apt install ./AncletoCode_0.0.2_amd64.deb
ancleto-code-desktop
```

### Linux — Arch/EndeavourOS y otras distros

```bash
chmod +x AncletoCode_0.0.2_amd64.AppImage
./AncletoCode_0.0.2_amd64.AppImage
```

Si FUSE no está disponible:

```bash
./AncletoCode_0.0.2_amd64.AppImage --appimage-extract-and-run
```

### Windows 10/11 x64

1. Descargá `AncletoCode_0.0.2_x64-setup.exe` (NSIS) o
   `AncletoCode_0.0.2_x64_en-US.msi`.
2. Si SmartScreen advierte por ser una build interna sin firma, verificá el hash
   y elegí **Más información → Ejecutar de todos modos**.
3. Abrí **AncletoCode** desde el menú Inicio.

El **ZIP portable** incluye la app y el host:

```powershell
Expand-Archive .\AncletoCode-0.0.2-windows-x64-portable.zip -DestinationPath .\AncletoCode
.\AncletoCode\ancleto-code-desktop.exe
```

### macOS

No hay artefactos en 0.0.2.

## Uso

### Proyectos y sesiones

1. Abrí **AncletoCode**.
2. Agregá un proyecto con su ruta local.
3. Creá una sesión de agente desde el thread y elegí driver.
4. Cerrar la ventana hace *detach*: las sesiones siguen en el host.

### Tareas y Git

- **Tareas** abre el board por proyecto.
- **Iniciar** una tarea crea su rama `task/...` y un worktree real.
- El inspector **Git** muestra rama, cambios, worktrees, remoto y PRs/issues de
  GitHub.
- Podés hacer fetch, pull, push, stage, commit y merge desde la app.
- GitHub usa `gh`; GitLab/`glab` queda pendiente en esta build.

### Ancleto SDD y actualizaciones

- **Ajustes → Ancleto SDD** muestra la versión incluida (`0.14.0`) y permite
  actualizarla desde npm con verificación de integridad y rollback.
- Al iniciar, la app consulta GitHub Releases y muestra un aviso si hay una
  versión nueva.
- El runtime Node y `@ancleto/spec` están dentro de los instaladores; no hace
  falta Node del sistema.

### Verificación

```bash
# Linux
sha256sum -c SHA256SUMS
```

```powershell
# Windows PowerShell
Get-FileHash .\AncletoCode_0.0.2_x64-setup.exe -Algorithm SHA256
```

## Limitaciones de 0.0.2

- Windows no fue validado manualmente por maintainers en esta build.
- GitLab y otros hosts quedan fuera de alcance por ahora. Checks/review de PRs y
  abrir el proyecto desde el worktree siguen pendientes.
- AncletoCMD, CLI `acode`, npm y AUR están fuera de este release.
- La atribución de fork y la revisión técnica de licencias están incluidas; para
  un release estable se recomienda revisión legal externa.

## Documentación

- Propuesta: [`aspec/changes/ancleto-code-desktop-workplace/proposal.md`](aspec/changes/ancleto-code-desktop-workplace/proposal.md)
- Diseño: [`aspec/changes/ancleto-code-desktop-workplace/design.md`](aspec/changes/ancleto-code-desktop-workplace/design.md)
- Tareas: [`aspec/changes/ancleto-code-desktop-workplace/tasks.md`](aspec/changes/ancleto-code-desktop-workplace/tasks.md)
- Especificaciones: [`aspec/changes/ancleto-code-desktop-workplace/specs/`](aspec/changes/ancleto-code-desktop-workplace/specs/)
- Revisión de compatibilidad de licencias: [`docs/compliance/license-review-0.0.1.md`](docs/compliance/license-review-0.0.1.md)
- Sistema de diseño del nuevo desktop: [`docs/design/desktop-ui.md`](docs/design/desktop-ui.md)

## Licencia

AncletoCode se distribuye bajo **GNU Affero General Public License v3.0 only** (`AGPL-3.0-only`). Ver [`LICENSE`](LICENSE).

Esto implica, en resumen:

- Podés usar, estudiar, modificar y redistribuir el software.
- Si distribuís una versión modificada, **el trabajo derivado debe licenciarse bajo la AGPL v3** y su código fuente debe quedar disponible.
- Si ofrecés una versión modificada como servicio de red, también tenés que ofrecer su código fuente a los usuarios (sección 13 de la licencia).
- El software se entrega **"AS IS"**, sin garantías de ningún tipo, y con limitación de responsabilidad (secciones 15 y 16).

No existe una licencia que obligue a *cualquier* uso derivado a ser open source en todos los escenarios: la AGPL aplica al acto de distribuir o de ofrecer el software por red. Para uso interno no distribuido, la licencia no obliga a publicar. Esta nota es un resumen informal, no asesoramiento legal; el texto de [`LICENSE`](LICENSE) es el que rige.

## Licencias y atribuciones

**This is a fork of PandaCode** — https://github.com/aicodeking/pandacode

AncletoCode reutiliza código de [PandaCode](https://github.com/aicodeking/pandacode), fijado en el commit `e9f89891fd23518848765aef8219dbf39feac6e0` (v0.3.13), bajo la PandaCode License (MIT con requisito de atribución de fork).

- Texto de la licencia: [`third_party/pandacode/LICENSE`](third_party/pandacode/LICENSE).
- Avisos completos: [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
- Archivos reutilizados y sincronización: [`third_party/pandacode/UPSTREAM.md`](third_party/pandacode/UPSTREAM.md).
- Procedencia y modificaciones: [`docs/provenance.md`](docs/provenance.md).

La declaración de fork aparece en la pantalla **About** de la app de escritorio y en el cuerpo del GitHub Release. No se incluirán wallpapers, temas, iconos, screenshots ni logos de PandaCode; los activos de terceros que se incorporen quedarán registrados con su licencia en `THIRD_PARTY_NOTICES.md`. Las dependencias del emulador (`alacritty_terminal`/`vte`, `winit`, `wgpu`, `glyphon`, `cosmic-text`) se auditan con el resto.

## Contribuir

El proyecto usa el flujo SDD de `@ancleto/spec`. Los cambios se proponen en `aspec/changes/` y se implementan después de aprobar propuesta, diseño y tareas. Al enviar una contribución aceptás que se distribuya bajo `AGPL-3.0-only`.
