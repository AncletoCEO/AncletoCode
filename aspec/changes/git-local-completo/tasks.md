# Tasks: Git local completo (rebase, ramas y revisión de cambios)

## Fase 1 — Rebase

- [ ] 1.1 Agregar `rebase` al `GitService` con `GIT_EDITOR` no interactivo y confirmación previa.
- [ ] 1.2 Implementar detección de conflictos, listado de archivos y acciones continuar/abortar.
- [ ] 1.3 Tests con repos temporales: rebase limpio, conflicto y aborto.

## Fase 2 — Ramas

- [ ] 2.1 Completar el branch chip: rama actual, ahead/behind y dirty state.
- [ ] 2.2 Completar el panel de ramas: listar, crear, cambiar y eliminar con confirmación; worktree asociado.
- [ ] 2.3 Tests de stores y componentes para el panel de ramas.

## Fase 3 — Revisión de cambios

- [ ] 3.1 Implementar diff split además del unificado, con archivos agrupados por turno.
- [ ] 3.2 Implementar staging por hunk y refresco del estado tras cada operación.
- [ ] 3.3 Implementar comentarios por línea con envío al agente y descarte cuando el diff cambia.
- [ ] 3.4 Tests de parseo de hunks, staging parcial y comentarios obsoletos.
