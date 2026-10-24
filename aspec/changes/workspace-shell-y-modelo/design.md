# Design: Shell visual completo y modelo Task/Thread/Worktree

## Approach

El shell se divide en cuatro superficies estables (sidebar, workspace, inspector,
composer) y el layout se persiste como datos, no como estado implícito de React. El
modelo de dominio vive en `ancleto-code-core`; la UI consume comandos IPC y nunca
escribe SQLite por su cuenta. Las operaciones de worktree se ejecutan en el núcleo
con `git` por argv y devuelven un resultado accionable.

```
┌ sidebar ─┐┌──────────── workspace ────────────┐┌ inspector ┐
│ proyecto ││ ProjectTabs → AgentTabs → Thread ││ contexto  │
│ tareas   ││ ┌ terminal / panels ───────────┐ ││ cambios   │
│ sesiones ││ └──────────────────────────────┘ ││ git       │
└──────────┘└────────── composer ─────────────┘└───────────┘
```

## Architecture

- **Layout**: tres zonas redimensionables; tamaños y visibilidad en SQLite por
  workspace, con layout por defecto si el registro no existe o está corrupto.
- **Modelo**: `Task { id, project_id, title, status, agent, branch, worktree_path,
  pr_url, result }`; `Thread` referencia una tarea y sus sesiones. Migraciones
  idempotentes en el núcleo con tests de ida y vuelta.
- **Worktrees**: `WorktreeManager` en el núcleo; crea con `git worktree add`, detecta
  colisión de rama/path, abre el proyecto en el worktree y elimina solo con árbol
  limpio o confirmación. Test con repos bare temporales.
- **Palette**: command palette con búsqueda por proyecto, tarea, rama, worktree, PR y
  acción; quick switcher para proyectos y tareas. Los atajos se guardan en settings.
- **Frontera**: comandos IPC que tocan disco o procesos son `async` +
  `spawn_blocking`, según `docs/performance.md`.

## Validation

```bash
npm run test
npm run typecheck
cargo test -p ancleto-code-core
cargo clippy --workspace --all-targets
npm run check:commands
```

- Tests de modelo: crear tarea con worktree, recuperar tras reinicio, migración desde
  base previa.
- Tests de worktree: colisión de path, rama en uso, eliminación con cambios sin
  commit y eliminación limpia.
- Tests de UI: layout persistente, apertura/cierre de paneles y búsqueda en palette.
- Smoke manual: crear tarea, abrir worktree, cambiar de proyecto y reiniciar.
