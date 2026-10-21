# AncletoCode

Desktop workplace para agentes de código, construido con **Tauri 2**. Reúne proyectos, sesiones de agentes y el ecosistema Ancleto en una sola aplicación de escritorio con estética de terminal.

> **Estado:** especificación. El proyecto está en fase de propuesta: todavía no hay implementación. Los artefactos de diseño viven en [`aspec/changes/ancleto-code-desktop-workplace/`](aspec/changes/ancleto-code-desktop-workplace/).

## Qué va a hacer

AncletoCode es un *workplace* para trabajar con varios agentes de código a la vez, organizados por proyecto, sin saltar entre terminales.

### Proyectos y tabs

- Vista por proyecto con ruta, árbol de archivos, estado SDD y estado de conocimiento.
- **Tabs de primer nivel por proyecto** y, dentro de cada proyecto, **una tab por sesión de agente**.
- Estado en vivo por sesión: `starting`, `ready`, `busy`, `waiting_input`, `stopped`, `crashed`.
- Las sesiones de un proyecto siguen corriendo al cambiar de tab.

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
- La app no reimplementa ninguna de estas capacidades: las consume como fuente de verdad. Tampoco escribe en `aspec/`, `AGENTS.md`, `PRODUCT.md` ni en `.ancleto/writer/`.

### Apariencia CLI

- Estética de terminal: tipografía monoespaciada, densidad compacta, glifos de prompt y statusline.
- **Transparencias** de ventana y paneles (Mica/Acrylic/Blur en Windows, vibrancy en macOS, compositor en Linux) con fallback CSS y modo opaco.
- **Colores configurables** mediante temas y tokens semánticos; la paleta ANSI del terminal se deriva del tema.
- **Fondos configurables**: sólido, degradado, imagen con blur/dim o patrón sutil.
- Precedencia global → workspace → proyecto, con validación de contraste WCAG AA.
- La configuración de apariencia se guarda en la app, nunca en el repositorio del proyecto.

## Arquitectura prevista

```
┌──────────────────────────────────────────────────────┐
│ Frontend: React 19 + TypeScript + Vite               │
│ Sidebar · Tabs · xterm.js · Paneles                  │
└───────────────▲────────────────────┬─────────────────┘
                │ eventos            │ comandos
┌───────────────┴────────────────────▼─────────────────┐
│ Núcleo Rust (Tauri 2)                                │
│ Sesiones PTY · Drivers · Conversaciones              │
│ Providers @ancleto/spec y @ancleto/writer            │
│ Integración Antigravity IDE · SQLite local           │
└──────────────────────────────────────────────────────┘
```

- **Rust**: procesos PTY (`portable-pty`), detección de binarios, ejecución de CLIs, persistencia y seguridad.
- **Frontend**: UI y estado; no ejecuta procesos ni accede al filesystem.
- **Local-first**: sin nube, sin telemetría y sin sincronización remota.

## Requisitos (previstos)

- Rust estable y las dependencias de Tauri 2 para tu plataforma.
- Node.js ≥ 24 para `@ancleto/spec` y `@ancleto/writer`.
- Node.js ≥ 22 para Command Code.
- Las CLIs de agente que quieras usar, ya instaladas y autenticadas.

## Uso (previsto)

```bash
npm install
npm run tauri dev
```

Estos comandos todavía no existen: se crearán durante la implementación.

## Documentación

- Propuesta: [`aspec/changes/ancleto-code-desktop-workplace/proposal.md`](aspec/changes/ancleto-code-desktop-workplace/proposal.md)
- Diseño: [`aspec/changes/ancleto-code-desktop-workplace/design.md`](aspec/changes/ancleto-code-desktop-workplace/design.md)
- Tareas: [`aspec/changes/ancleto-code-desktop-workplace/tasks.md`](aspec/changes/ancleto-code-desktop-workplace/tasks.md)
- Especificaciones: [`aspec/changes/ancleto-code-desktop-workplace/specs/`](aspec/changes/ancleto-code-desktop-workplace/specs/)

## Licencia

AncletoCode se distribuye bajo **GNU Affero General Public License v3.0 only** (`AGPL-3.0-only`). Ver [`LICENSE`](LICENSE).

Esto implica, en resumen:

- Podés usar, estudiar, modificar y redistribuir el software.
- Si distribuís una versión modificada, **el trabajo derivado debe licenciarse bajo la AGPL v3** y su código fuente debe quedar disponible.
- Si ofrecés una versión modificada como servicio de red, también tenés que ofrecer su código fuente a los usuarios (sección 13 de la licencia).
- El software se entrega **"AS IS"**, sin garantías de ningún tipo, y con limitación de responsabilidad (secciones 15 y 16).

No existe una licencia que obligue a *cualquier* uso derivado a ser open source en todos los escenarios: la AGPL aplica al acto de distribuir o de ofrecer el software por red. Para uso interno no distribuido, la licencia no obliga a publicar. Esta nota es un resumen informal, no asesoramiento legal; el texto de [`LICENSE`](LICENSE) es el que rige.

## Contribuir

El proyecto usa el flujo SDD de `@ancleto/spec`. Los cambios se proponen en `aspec/changes/` y se implementan después de aprobar propuesta, diseño y tareas. Al enviar una contribución aceptás que se distribuya bajo `AGPL-3.0-only`.
