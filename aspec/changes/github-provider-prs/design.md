# Design: GitHostProvider y GitHub nativo (PRs, checks y review)

## Approach

Se define una interfaz `GitHostProvider` en el núcleo con capacidades declaradas y
operaciones tipadas. GitHub se implementa ejecutando `gh` por argv, sin shell
intermedio y sin persistir credenciales: la autenticación vive en `gh`. La UI
consume el provider y deshabilita lo que no está disponible.

```
GitService (rama/diff) ──┐
                         ├──► GitHostProvider (traits) ──► GitHubCliProvider ──► gh
Tarea / Thread ──────────┘             │
                        capacidades ◄──┴──► UI (PR panel, checks, review)
```

## Architecture

- **Contrato**: `GitHostProvider { id, detect(), capabilities(), pull_requests(),
  issues(), checks(), create_pull_request(), review(), merge() }`; errores
  `unavailable`/`unauthorized`/`unsupported` con mensaje accionable.
- **Detección**: `gh auth status --json`; se expone login y host sin exponer tokens.
- **PR desde tarea**: la tarea aporta rama, base, título y cuerpo; el provider crea el
  PR y guarda `pr_url` en el modelo; el thread muestra estado y checks.
- **Review/merge**: review con comentario y evento (`approve`, `comment`,
  `request-changes`); merge solo si no hay conflictos y los checks requeridos pasan,
  con confirmación explícita.
- **Comandos IPC**: `async` + `spawn_blocking` por usar procesos y red.
- **Sin secretos**: nada de tokens en settings, logs ni caché; los errores de `gh` se
  muestran sanitizados.

## Validation

```bash
cargo test -p ancleto-code-core github
cargo test --manifest-path src-tauri/Cargo.toml
npm run test
npm run typecheck
npm run check:commands
```

- Tests con un `gh` falso (script de fixture): login, sin login, sin permisos, PRs,
  issues, checks, review y merge rechazado por checks.
- Tests de UI: PR enlazado a tarea, capacidades deshabilitadas y confirmación de merge.
- Smoke manual con un repo real y `gh` autenticado.
