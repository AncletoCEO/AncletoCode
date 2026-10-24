# Tasks: Shell visual completo y modelo Task/Thread/Worktree

## Fase 1 — Shell y layout

- [ ] 1.1 Implementar el shell visual: navegación lateral por proyecto/tarea, workspace central con threads/tabs, inspector derecho (contexto, cambios, Git) y composer inferior.
- [ ] 1.2 Implementar paneles redimensionables y persistir el layout por workspace con fallback a un layout por defecto.

## Fase 2 — Modelo

- [ ] 2.1 Extender `Task`/`Thread`/`Worktree`: rama y worktree por tarea, estado, proyecto, agente, PR/MR y resultado; migraciones SQLite idempotentes.
- [ ] 2.2 Recuperar tareas, threads y worktrees tras reiniciar la app, con tests de ida y vuelta y migración desde una base previa.

## Fase 3 — Worktrees

- [ ] 3.1 Completar `WorktreeManager`: crear, listar y eliminar worktrees; detectar colisiones de rama y de path.
- [ ] 3.2 Abrir el proyecto en el worktree correcto y limpiar sin perder cambios (árbol limpio o confirmación explícita).
- [ ] 3.3 Tests con repos bare temporales: colisión, rama en uso, eliminación con cambios sin commit y eliminación limpia.

## Fase 4 — Command palette

- [ ] 4.1 Command palette y quick switcher: proyectos, tareas, ramas, worktrees, PRs/MRs, comandos Git y acciones Ancleto.
- [ ] 4.2 Atajos configurables y persistidos; tests de búsqueda y selección.
