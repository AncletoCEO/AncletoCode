# Tasks: GitHostProvider y GitHub nativo (PRs, checks y review)

## Fase 1 — Provider

- [ ] 1.1 Definir `GitHostProvider` con identificación, detección, capacidades y errores tipados (`unavailable`, `unauthorized`, `unsupported`).
- [ ] 1.2 Implementar GitHub con `gh`: detectar login y host, listar repos y exponer capacidades reales.
- [ ] 1.3 Tests con `gh` mockeado: sin binario, sin login, sin permisos y con login válido.

## Fase 2 — PRs, issues y checks

- [ ] 2.1 Listar issues y checks de un PR, con estados y mensajes de error accionables.
- [ ] 2.2 Abrir un PR y enlazarlo con tarea y rama; persistir `pr_url` en el modelo.
- [ ] 2.3 Mostrar estado y checks del PR en el thread y refrescar al abrir la tarea.

## Fase 3 — Review y merge

- [ ] 3.1 Implementar review (`approve`, `comment`, `request-changes`) con confirmación.
- [ ] 3.2 Implementar merge seguro: validar conflictos y checks requeridos, y exigir confirmación explícita.
- [ ] 3.3 Tests de review, merge rechazado y merge exitoso con `gh` mockeado.

## Fase 4 — PR desde una tarea

- [ ] 4.1 Crear PR/MR desde una tarea: rama, base, título/cuerpo, reviewers, issue link y checks.
- [ ] 4.2 Tests de UI del formulario y del estado del PR en el thread.
