# AncletoCode

Workspace de agentes de código con **dos versiones**: la app de escritorio **AncletoCode** (Tauri 2) y **AncletoCMD**, un emulador de terminal nativo, rápido y GPU-acelerado que además funciona como gestor de proyectos y sesiones. Ambas comparten el mismo núcleo, la misma configuración y las mismas sesiones, junto con una CLI headless para automatización.

> **Estado:** `0.0.1`, primer release de prueba para Linux y Windows. La revisión técnica de licencias está completada; para un release estable se recomienda revisión externa. Ver [`docs/release-plan.md`](docs/release-plan.md) y [`docs/release-notes/0.0.1.md`](docs/release-notes/0.0.1.md).

> **Alcance de los próximos releases (0.2+):** sólo la app de escritorio
> AncletoCode. AncletoCMD, la CLI `acode`, el host como artefacto separado y npm
> quedan congelados por ahora. La app evoluciona hacia un híbrido visual
> PandaCode + Orca, con Git/worktrees, GitHub/GitLab y release Linux vía AppImage.
> Los artefactos 0.0.1 documentados abajo siguen disponibles como históricos.


## Qué va a hacer

AncletoCode es un *workplace* para trabajar con varios agentes de código a la vez, organizados por proyecto, sin saltar entre terminales. Se puede usar desde la app de escritorio, desde AncletoCMD o desde la CLI, sobre el mismo estado.

### Dos versiones y una CLI

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

### Para usar un release 0.0.1

- **Linux x86_64:** glibc compatible con Ubuntu 22.04 o posterior si usás el
  `.deb`; el tarball y el paquete AUR son la opción para otras distribuciones.
  La app necesita WebKitGTK 4.1, GTK 3, libsoup 3 y librsvg.
- **Windows 10/11 x64:** WebView2 (incluido en Windows 11 y en la mayoría de
  Windows 10 actualizados). No hace falta Node.js para usar la app.
- **AncletoCMD en Linux:** Wayland o X11, Vulkan/OpenGL y las dependencias del
  `PKGBUILD` (`libxkbcommon`, `vulkan-icd-loader`, `fontconfig`, `freetype2`,
  `wayland`, `libx11`).
- **CLI y host:** no requieren Node.js. Node.js ≥ 24 sólo es necesario para
  `acode spec` y `acode writer`; Node.js ≥ 22 para el driver de Command Code.
- **Agentes:** instalá y autenticá por separado las CLIs que quieras usar
  (`claude`, `opencode`, `agy`, `cmd`). `acode doctor` muestra qué encontró.
- **macOS:** no hay artefactos en 0.0.1.

### Para compilar

- Rust estable y las dependencias de Tauri 2 de tu plataforma.
- Node.js ≥ 24 y npm para el frontend y los scripts de release.
- En Linux, además: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`,
  `librsvg2-dev`, `patchelf` y un toolchain de C/C++.
- En Arch, los grupos `base-devel` y las dependencias que declara el `PKGBUILD`.
- Para compilar con Ancleto embebido: `npm run prepare:ancleto` descarga el runtime
  Node y el paquete `@ancleto/spec` incluido. El release lo ejecuta antes de
  `tauri build`.

## Instalación y uso por plataforma

Todos los artefactos están en
[Releases v0.0.1](https://github.com/AncletoCEO/AncletoCode/releases/tag/v0.0.1).
Verificá la integridad con `SHA256SUMS` antes de instalar.

### Linux — Debian/Ubuntu (`.deb`)

```bash
sudo apt install ./AncletoCode_0.0.1_amd64.deb
ancleto-code-desktop
```

También aparece en el menú de aplicaciones como **AncletoCode**. El `.deb` no
incluye el host de sesiones; para sesiones persistentes usá el tarball de la CLI
o el `PKGBUILD` de Arch.

### Linux — Arch/EndeavourOS (PKGBUILD)

El release incluye `ancleto-aur-0.0.1.tar.gz` con tres `PKGBUILD`. Todavía no
están publicados en AUR; se compilan desde el release:

```bash
sudo pacman -S --needed base-devel
mkdir -p ~/ancleto-aur
tar -xzf ~/Downloads/ancleto-aur-0.0.1.tar.gz -C ~/ancleto-aur

# 1) Primero la CLI: aporta acode y ancleto-code-host
cd ~/ancleto-aur/ancleto-code-cli-bin
makepkg -si

# 2) Después la app de escritorio
cd ~/ancleto-aur/ancleto-code-bin
makepkg -si
```

Para AncletoCMD:

```bash
cd ~/ancleto-aur/ancleto-cmd-bin
makepkg -si
```

El orden importa: `ancleto-code-bin` y `ancleto-cmd-bin` dependen de
`ancleto-code-cli-bin`, que todavía no está publicado en AUR. Por eso `pacman`
no puede resolver esa dependencia si instalás primero la app.

`ancleto-code-bin` instala la app y depende de `ancleto-code-cli-bin`, que aporta
`acode` y `ancleto-code-host`.

### Linux — tarballs (cualquier distribución)

```bash
# App de escritorio + host
tar -xzf AncletoCode-0.0.1-linux-x86_64.tar.gz -C /tmp/ancleto-code
sudo install -Dm755 /tmp/ancleto-code/ancleto-code-desktop /usr/local/bin/ancleto-code-desktop
sudo install -Dm755 /tmp/ancleto-code/ancleto-code-host /usr/local/bin/ancleto-code-host
sudo install -Dm644 /tmp/ancleto-code/ancleto-code.desktop /usr/share/applications/ancleto-code.desktop
for size in 32 128 256; do
  sudo install -Dm644 "/tmp/ancleto-code/icons/${size}x${size}.png" \
    "/usr/share/icons/hicolor/${size}x${size}/apps/ancleto-code.png"
done
ancleto-code-desktop

# CLI + host
tar -xzf acode-0.0.1-linux-x86_64.tar.gz -C "$HOME/.local/opt/acode"
mkdir -p "$HOME/.local/bin"
ln -sf "$HOME/.local/opt/acode/acode" "$HOME/.local/bin/acode"
ln -sf "$HOME/.local/opt/acode/ancleto-code" "$HOME/.local/bin/ancleto-code"
ln -sf "$HOME/.local/opt/acode/ancleto-code-host" "$HOME/.local/bin/ancleto-code-host"

# AncletoCMD
tar -xzf AncletoCMD-0.0.1-linux-x86_64.tar.gz -C "$HOME/.local/opt/ancleto-cmd"
ln -sf "$HOME/.local/opt/ancleto-cmd/acmd" "$HOME/.local/bin/acmd"
ln -sf "$HOME/.local/opt/ancleto-cmd/ancleto-cmd" "$HOME/.local/bin/ancleto-cmd"
```

Asegurate de que `~/.local/bin` esté en `PATH`. El tarball de la app y el de la
CLI incluyen `ancleto-code-host`; el de AncletoCMD también, para que sea
autocontenido.

### Windows 10/11 x64

**Instalador (recomendado para la app):**

1. Descargá `AncletoCode_0.0.1_x64-setup.exe` (NSIS) o
   `AncletoCode_0.0.1_x64_en-US.msi`.
2. Ejecutalo. Si SmartScreen advierte por ser una build interna sin firma,
   elegí **Más información → Ejecutar de todos modos**.
3. Abrí **AncletoCode** desde el menú Inicio.

**ZIP portable (app + host):**

```powershell
Expand-Archive .\AncletoCode-0.0.1-windows-x64-portable.zip -DestinationPath .\AncletoCode
.\AncletoCode\ancleto-code-desktop.exe
```

**CLI + host en PowerShell:**

```powershell
$dest = "$env:LOCALAPPDATA\AncletoCode\bin"
New-Item -ItemType Directory -Force $dest | Out-Null
Expand-Archive .\acode-0.0.1-windows-x64.zip -DestinationPath $dest -Force
[Environment]::SetEnvironmentVariable(
  "Path",
  [Environment]::GetEnvironmentVariable("Path", "User") + ";$dest",
  "User"
)
# Cerrá y volvé a abrir la terminal para que tome el PATH
acode doctor
```

**AncletoCMD en Windows:**

```powershell
$dest = "$env:LOCALAPPDATA\AncletoCMD"
Expand-Archive .\AncletoCMD-0.0.1-windows-x64.zip -DestinationPath $dest -Force
& "$dest\acmd.exe" --project .
```

El instalador NSIS/MSI **no** incluye `ancleto-code-host.exe`; el ZIP portable y
el ZIP de la CLI sí. Si usás el instalador y querés sesiones persistentes,
agregá el host del ZIP de la CLI al `PATH` o usá el portable. El checklist
completo está en [`docs/windows-testing.md`](docs/windows-testing.md).

### macOS

No hay `.dmg` ni binarios para macOS en 0.0.1. La app y AncletoCMD se pueden
compilar desde el código, pero no es un artefacto soportado de este release.

### npm

La publicación en npm quedó **pendiente** para un release posterior. En 0.0.1
usá los tarballs/ZIP de GitHub. La matriz prevista (cuando se publique) es:

| Plataforma | `@ancleto/code` | `@ancleto/cmd` |
|---|---|---|
| Linux x64/arm64 glibc | sí | sí |
| Linux x64/arm64 musl | sí | no |
| Windows x64/arm64 | sí | sí |
| macOS x64/arm64 | sí | sí |

El launcher verifica el checksum del binario, propaga señales y, si falta el
paquete de la plataforma, indica la plataforma detectada y el comando de
instalación manual.

## Uso de las superficies

### App de escritorio

1. Abrí **AncletoCode**.
2. Agregá un proyecto con la ruta local o elegí uno existente.
3. Creá una sesión de agente desde el panel del proyecto y elegí el driver.
4. Las tabs de proyecto y de agente conservan el estado; cerrar la ventana hace
   *detach*, no mata la sesión.
5. La atribución de fork y las licencias están en **About**.

### AncletoCMD

```bash
acmd --project .
acmd --project . --theme <id>
acmd --install-hooks bash   # o zsh, fish, powershell
```

Atajos y paleta de comandos están documentados en la propia app. AncletoCMD
comparte proyectos, sesiones y temas con la app y la CLI. `--kill-on-close`
termina las sesiones al cerrar la ventana; sin esa opción quedan en el host.

### CLI `acode`

```text
acode [--json] [--no-input] [--no-host] [--project <id|ruta>] <comando>
```

| Comando | Propósito |
|---|---|
| `acode project add <ruta>` / `list` / `remove <id>` / `open <id>` | Gestión de proyectos compartida |
| `acode agents [--json]` | Detección de drivers y capacidades |
| `acode doctor [--json]` | Node, drivers, providers, host, plataforma y base |
| `acode session start --driver <id> --project <id>` | Crear una sesión PTY en el host |
| `acode session list` / `attach <id>` / `send <id> "texto"` / `kill <id>` | Ciclo de vida y attach |
| `acode run "<prompt>" --driver <id> [--project <id>]` | One-shot; streaming si la salida es TTY |
| `acode converse --a <id> --b <id> --objective "..."` | Conversación agente-a-agente con límites |
| `acode spec <init|install|update|upgrade|check|doctor|specs…>` | Passthrough de `@ancleto/spec` |
| `acode writer <init|build|check|lint|impact|human|doctor>` | Passthrough de `@ancleto/writer` |
| `acode host start` / `stop` / `status` | Control del host de sesiones |
| `acode ui` | Abrir la app de escritorio instalada |
| `acode completions <bash|zsh|fish|powershell>` | Completions de shell |

- `--json` emite un sobre `{ "schemaVersion": 1, "ok": true|false, "data"|"error" }` por stdout.
- Sin TTY no se emiten secuencias ANSI ni se piden confirmaciones.
- Códigos de salida: `0` ok, `2` uso inválido, `3` dependencia ausente, `4` error de sesión/host, `5` estado incompatible, `6` cancelado.
- `--no-host` ejecuta los one-shot in-process, sin sesión persistente.
- Las sesiones viven en el host local: cerrar la app o la CLI no mata a los agentes.

Ejemplos:

```bash
acode doctor
acode --json agents
acode project add .
acode run "explicá este módulo" --driver claude --project .
acode session start --driver opencode --project .
acode --json session list
acode completions bash >> ~/.bashrc
```

## Verificación y problemas comunes

```bash
# Linux
sha256sum -c SHA256SUMS

# Windows PowerShell
Get-FileHash .\acode-0.0.1-windows-x64.zip -Algorithm SHA256
```

- **`acode` no se encuentra:** agregá el directorio de instalación al `PATH` y
  abrí una terminal nueva.
- **No hay host / las sesiones no persisten:** `acode host status`; si falta el
  host, usá el ZIP portable, el ZIP de la CLI o el paquete AUR de la CLI.
- **La app no abre en Linux:** verificá WebKitGTK 4.1 y probá XWayland si estás
  en Wayland.
- **glibc incompatible:** usá el tarball o el `PKGBUILD`; el `.deb` se compila en
  Ubuntu 22.04.
- **Windows SmartScreen:** build interna sin firma; verificá el hash y elegí
  **Ejecutar de todos modos**.
- **Un agente no aparece:** `acode doctor` y comprobá que su binario esté en
  `PATH` y autenticado.

## Documentación

- Propuesta: [`aspec/changes/ancleto-code-desktop-workplace/proposal.md`](aspec/changes/ancleto-code-desktop-workplace/proposal.md)
- Diseño: [`aspec/changes/ancleto-code-desktop-workplace/design.md`](aspec/changes/ancleto-code-desktop-workplace/design.md)
- Tareas: [`aspec/changes/ancleto-code-desktop-workplace/tasks.md`](aspec/changes/ancleto-code-desktop-workplace/tasks.md)
- Especificaciones: [`aspec/changes/ancleto-code-desktop-workplace/specs/`](aspec/changes/ancleto-code-desktop-workplace/specs/)
- Revisión de compatibilidad de licencias: [`docs/compliance/license-review-0.0.1.md`](docs/compliance/license-review-0.0.1.md)
- Sistema de diseño del nuevo desktop: [`docs/design/desktop-ui.md`](docs/design/desktop-ui.md)

## Licencia

AncletoCode y AncletoCMD se distribuyen bajo **GNU Affero General Public License v3.0 only** (`AGPL-3.0-only`). Ver [`LICENSE`](LICENSE).

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

La declaración de fork aparece en la pantalla **About** de la app de escritorio y en los README de los paquetes npm. No se incluirán wallpapers, temas, iconos, screenshots ni logos de PandaCode; los activos de terceros que se incorporen quedarán registrados con su licencia en `THIRD_PARTY_NOTICES.md`. Las dependencias del emulador (`alacritty_terminal`/`vte`, `winit`, `wgpu`, `glyphon`, `cosmic-text`) se auditan con el resto.

## Contribuir

El proyecto usa el flujo SDD de `@ancleto/spec`. Los cambios se proponen en `aspec/changes/` y se implementan después de aprobar propuesta, diseño y tareas. Al enviar una contribución aceptás que se distribuya bajo `AGPL-3.0-only`.
