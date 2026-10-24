# Design: Git local completo (rebase, ramas y revisión de cambios)

## Approach

El `GitService` sigue usando el binario `git` con argv y sin shell intermedio. Las
operaciones nuevas conservan ese contrato y devuelven `GitActionOutcome` con estado y
mensaje accionable. La UI nunca ejecuta Git directamente: invoca comandos `async` y
refresca el estado después de cada mutación.

```
UI (branch panel / diff) ──IPC async──► GitService (core, argv)
                                            │
                    status/diff/rebase/merge ◄┘
                                            │
                    resultado + estado nuevo ──► store
```

## Architecture

- **Rebase**: `git rebase <base>` con `GIT_EDITOR=true` para no bloquear; si hay
  conflicto, se devuelve el listado de archivos y se ofrece abortar o continuar.
- **Ramas**: el panel consume `git for-each-ref` y `git status` para ahead/behind,
  dirty state y worktree asociado (`git worktree list --porcelain`).
- **Diff**: se parsea el diff unificado en archivos y hunks; el split se renderiza
  desde el mismo modelo. El staging por hunk usa `git apply --cached` con el hunk
  seleccionado.
- **Comentarios por línea**: se guardan en memoria con archivo, línea y hash del diff;
  se envían al agente como texto estructurado. Si el diff cambia, se marcan como
  obsoletos.
- **Seguridad**: toda acción destructiva (reset, checkout con cambios, borrar rama,
  rebase) exige confirmación explícita; nunca se fuerza push.

## Validation

```bash
cargo test -p ancleto-code-core git
cargo clippy --workspace --all-targets
npm run test
npm run typecheck
npm run check:commands
```

- Repos temporales: rebase limpio, conflicto y aborto; rama en uso; borrar rama con
  cambios.
- Diff: parseo de archivos/hunks, staging parcial y descarte por diff obsoleto.
- UI: panel de ramas con ahead/behind y comentarios por línea.
