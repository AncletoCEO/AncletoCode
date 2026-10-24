# Proposal: GitHostProvider y GitHub nativo (PRs, checks y review)

## Problem

La integración con GitHub está a medio camino: se detecta el remoto y se listan PRs,
pero faltan autenticación y capacidades declaradas por el provider, listado de issues
y checks, review y merge seguro, y crear un PR/MR desde una tarea. Sin eso el flujo
"tarea → rama → PR → review → merge" no se puede completar desde la app.

## Proposed change

- Completar la abstracción `GitHostProvider` con GitHub como único proveedor activo,
  detección de `gh`, login y capacidades (`issues`, `checks`, `review`, `merge`).
- Completar GitHub con `gh`: listar repos/PRs/issues/checks, abrir PR, revisar y
  mergear; enlazar PR con tarea y rama.
- Crear PR/MR desde una tarea: rama, base, título/cuerpo, reviewers, issue link y
  checks, con estado visible en el thread.

## Scope

**In scope**

- `GitHostProvider` con capacidades y errores accionables.
- GitHub mediante `gh` (sin guardar tokens en la app).
- PRs vinculados a tareas, checks, review y merge.
- Estado del PR en el thread.

**Out of scope**

- GitLab, Bitbucket y Azure (diferidos).
- Autenticación propia con OAuth o tokens propios.
- Merge automático sin confirmación.
- Revisión de código con IA o comentarios inline en GitHub.

## Risks

- **`gh` ausente o sin login**: el provider se declara `unavailable` y guía el login;
  nunca se pide un token a la app.
- **Permisos insuficientes**: las capacidades se detectan y las acciones no
  disponibles se deshabilitan con explicación.
- **Merge accidental**: el merge exige confirmación y verifica checks y conflictos.
- **Datos obsoletos**: el estado del PR se refresca al abrir el thread y después de
  cada acción.

## Success criteria

- El provider declara login, repo y capacidades reales.
- Se listan issues, checks y PRs; se abre un PR desde una tarea.
- Review y merge funcionan con confirmación y validación de checks.
- Tests con `gh` mockeado y stores/componentes en verde.
