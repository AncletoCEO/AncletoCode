# AncletoCode Desktop — sistema de diseño 0.1

## Norte

Un workspace de agentes que se sienta **calmo como PandaCode** y **organizado como Orca**: superficies limpias, jerarquía clara, poca decoración, mucha información útil y teclado primero. La identidad es Ancleto; no se copian logos, wallpapers, screenshots, iconos ni temas de PandaCode u Orca.

Referencias de decisión:

- **PandaCode:** densidad contenida, navegación por proyectos, inspector contextual, composer siempre visible, estética de herramienta nativa.
- **Orca:** flotas de agentes, tareas con worktree, tabs/paneles, diff review, branch chip, command palette, inspector de cambios.
- **Ancleto:** spec, writer, conversación agente-a-agente y sesiones persistentes como columnas vertebrales del producto.

## Layout

```text
┌──────────────┬──────────────────────────────────────────────┬───────────────┐
│              │  breadcrumb · branch · search · actions       │               │
│  navegación  ├──────────────────────────────────────────────┤   inspector   │
│  proyectos   │  tabs de proyecto / sesión                   │  contexto     │
│  archivos    │                                              │  cambios      │
│              │  thread / terminal / panel Ancleto            │  git          │
│              │                                              │  equipo       │
│              ├──────────────────────────────────────────────┤               │
│  ajustes     │  composer                                    │               │
└──────────────┴──────────────────────────────────────────────┴───────────────┘
```

- Sidebar de **240 px** con proyectos, archivos y accesos.
- Inspector derecho de **340 px**, colapsable.
- Composer fijo abajo, con estado de sesión, driver y acciones.
- En ventanas angostas el inspector se oculta y la sidebar pasa a modo compacto.

## Tokens visuales

- **Superficies:** base casi negra, panel elevado, borde de 1 px con bajo contraste, radios de 8–12 px.
- **Acento:** un solo color de acción; por defecto el acento del tema activo. No usar el acento para decorar todo.
- **Tipografía:** sans del sistema para UI; IBM Plex Mono para código/terminal. Tamaños 11–13 px en densidad compacta y 13–15 px en cómoda.
- **Espaciado:** escala de 4 px; filas de 28–32 px; separación generosa entre secciones, no entre cada control.
- **Estados:** `ready` verde, `busy` azul, `waiting` ámbar, `crashed` rojo; siempre con texto además del color.
- **Movimiento:** 120–180 ms, sólo para aparición de paneles, hover y command palette. Respetar `prefers-reduced-motion`.
- **Sombras:** mínimas; la jerarquía se construye con superficies y bordes.

## Componentes

### Sidebar

Marca Ancleto, selector de proyecto, navegación (`Workspace`, `Tareas`, `Git`, `Conocimiento`), lista de proyectos, árbol de archivos colapsable y ajustes. No muestra drivers globales como lista permanente: eso vive en el inspector.

### Top bar

Breadcrumb proyecto / sesión, branch chip (con estado dirty/ahead/behind cuando exista Git), buscador/command palette, acciones de nueva sesión, tema y ajustes. Altura 44 px.

### Thread

Tabs de sesión con punto de estado, terminal o conversación, y empty state cuando no hay sesión. El contenido nunca queda debajo del composer.

### Composer

Textarea autoexpandible, selector de driver, botón de adjuntar, estado del host y enviar. `Ctrl/Cmd+Enter` envía; si no hay sesión, ofrece crearla. No inventa un modelo si el driver no lo declara.

### Inspector

Tabs: `Contexto`, `Cambios`, `Git`, `Equipo`.

- Contexto: ruta, estado SDD, spec/writer, sesiones.
- Cambios: resumen del turno, diff y staging (cuando Git esté implementado).
- Git: rama actual, worktree, PR/MR y acciones seguras.
- Equipo: drivers disponibles, sesiones y conversación.

### Command palette

`Ctrl/Cmd+K`: proyectos, sesiones, ramas, worktrees, PRs/MRs, temas y acciones. Búsqueda difusa, teclado completo, sin dependencia de mouse.

## Personalización

El usuario puede cambiar, y se persiste por workspace/proyecto:

- Tema preset o custom (incluye import/export JSON).
- Acento independiente del tema.
- Densidad: cómoda / compacta.
- Escala tipográfica.
- Radio de bordes.
- Opacidad y transparencia de paneles/ventana.
- Fondo: sólido, degradado, imagen o patrón, con blur/dim.
- Inspector abierto/cerrado y ancho de sidebar.
- Movimiento reducido.

Los ajustes rápidos viven en `Ajustes`; la personalización avanzada de temas/fondos reutiliza el sistema de apariencia existente.

## Reglas de interacción

- Toda acción destructiva (cerrar sesión, borrar worktree, descartar cambios) pide confirmación explícita.
- El estado del agente se muestra sin bloquear la terminal.
- Los errores son accionables: qué falló, qué comando ejecutar y cómo reintentar.
- Los empty states explican el siguiente paso, no sólo “no hay datos”.
- No se muestran acciones de Git/GitHub/GitLab que todavía no existen; se marcan como próximas o se ocultan.

## Accesibilidad y rendimiento

- Contraste AA para texto y estados; foco visible; navegación completa por teclado.
- `aria-label` en iconos y controles; diálogos con título y restauración de foco.
- Virtualizar listas largas y diffs; evitar animar layout.
- Probar WebView2 y WebKitGTK; screenshots de regresión en Windows y Linux.

## Estado actual

El primer shell visual está implementado sobre el core existente: sidebar de
proyectos y archivos, top bar, tabs de sesión, thread/terminal, inspector,
composer, command palette, ajustes de tema/acento/densidad/radio/tipografía y
vista de apariencia avanzada.

![Shell inicial de AncletoCode](desktop-shell-0.1.png)

![Ajustes, temas y Ancleto embebido](desktop-settings-0.1.png)

## Implementación

Tauri 2 + React 19 + TypeScript + Tailwind 4 + Radix/shadcn + Zustand. El core Rust y el host de sesiones no se reescriben en esta fase.
