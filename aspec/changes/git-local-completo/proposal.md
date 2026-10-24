# Proposal: Git local completo (rebase, ramas y revisión de cambios)

## Problem

El servicio Git local ya cubre status, ramas, diff, staging, commit, fetch/pull/push,
merge y worktrees, pero faltan tres piezas para el flujo diario: `rebase` con manejo
de conflictos, un panel de ramas completo (ahead/behind, dirty state, worktree
asociado) y la revisión de cambios con diff split, staging por hunk y comentarios por
línea enviados al agente.

## Proposed change

- Agregar `rebase` al servicio Git con confirmación y manejo de conflictos.
- Completar el branch chip y el panel de ramas: rama actual, ahead/behind, dirty
  state, cambiar/crear rama, worktree asociado y acciones seguras.
- Completar la revisión de cambios: diff unificado y split, archivos por turno,
  staging por hunk, comentarios por línea y envío del comentario al agente.

## Scope

**In scope**

- `rebase` local y resolución asistida de conflictos.
- Panel de ramas y branch chip.
- Diff split, staging por hunk y comentarios por línea.

**Out of scope**

- Fuerza push y reescritura de historia remota.
- Rebase interactivo con edición de commits (squash/reword) en esta etapa.
- Revisión de PRs en GitHub (va en `github-provider-prs`).
- GitLab, Bitbucket y Azure.

## Risks

- **Pérdida de trabajo en rebase**: se exige confirmación, se aborta con
  `git rebase --abort` ante error y se testea con repos temporales.
- **Conflictos no resueltos**: la UI muestra archivos en conflicto y no permite
  continuar hasta resolverlos.
- **Staging por hunk inconsistente**: se valida contra el diff vigente y se refresca
  el estado después de cada operación.
- **Comentarios huérfanos**: cada comentario referencia archivo, línea y commit/diff
  del momento; si el diff cambia, se descarta con aviso.

## Success criteria

- Rebase con y sin conflictos funciona y se puede abortar.
- El panel de ramas muestra ahead/behind, dirty state y worktree asociado.
- El diff permite split, staging por hunk y comentarios por línea enviados al agente.
- Tests de Git con repos temporales y de stores/componentes en verde.
