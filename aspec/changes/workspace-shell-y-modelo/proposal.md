# Proposal: Shell visual completo y modelo Task/Thread/Worktree

## Problem

El shell actual cubre sidebar, topbar, thread, inspector, composer y palette de forma
parcial: falta la navegación por proyecto/tarea, el layout redimensionable y
persistente, y el command palette/quick switcher completo. El modelo `Task` existe
con estados, pero todavía no se relaciona con `Thread`, `Worktree`, agente, PR/MR ni
resultado, y el `WorktreeManager` no abre el proyecto en el worktree correcto ni
resuelve colisiones y limpieza asistida.

## Proposed change

- Completar el shell visual: navegación lateral por proyecto/tarea, workspace central
  con threads/tabs, inspector derecho (contexto, cambios, Git), composer inferior y
  paneles redimensionables con layout persistente.
- Extender el modelo `Task`/`Thread`/`Worktree`: una tarea puede tener rama y worktree
  propios, con estado, proyecto, agente, PR/MR y resultado; migraciones SQLite y
  recuperación tras reinicio.
- Completar `WorktreeManager`: crear/listar/eliminar worktrees, abrir el proyecto en
  el worktree correcto, detectar colisiones y limpiar sin perder cambios.
- Completar command palette y quick switcher: proyectos, tareas, ramas, worktrees,
  PRs/MRs, comandos Git y acciones Ancleto, con atajos configurables.

## Scope

**In scope**

- Layout del shell y su persistencia.
- Modelo de dominio Task/Thread/Worktree y migraciones.
- Gestión de worktrees con apertura y limpieza segura.
- Command palette y quick switcher.

**Out of scope**

- Rebase y revisión de diffs (van en `git-local-completo`).
- GitHub/PRs (van en `github-provider-prs`).
- Vendorización de los módulos de UI (va en `vendorizacion-pandacode-orca`).
- Multi-ventana y edición colaborativa.

## Dependencias

- La delta de `project-workspace` se sincroniza después de archivar
  `ancleto-code-desktop-workplace`, que introduce esa capability en las specs
  principales.

## Risks

- **Pérdida de cambios al limpiar worktrees**: la eliminación exige árbol limpio o
  confirmación explícita, y se testea con repos temporales.
- **Layout persistente corrupto**: se valida al cargar y se cae a un layout por
  defecto sin romper la app.
- **Duplicación de estado**: el modelo vive en el núcleo compartido; la UI solo lo
  consume, no lo reimplementa.
- **Migraciones sobre bases existentes**: se prueban con bases previas y se mantienen
  idempotentes.

## Success criteria

- El shell permite navegar por proyecto y tarea, con layout persistente tras reiniciar.
- Una tarea puede crear su rama y worktree, abrirse en él y limpiarse sin perder
  cambios.
- El command palette encuentra proyectos, tareas, ramas, worktrees y PRs.
- Tests de stores, componentes y modelo en verde.
