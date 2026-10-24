# Tasks: Hardening del escritorio (paneles, accesibilidad, tests y rollout)

## Fase 1 — Integración de paneles

- [ ] 1.1 Integrar los paneles Ancleto (spec, writer, conversación, sesiones) en el nuevo shell sin regresiones; el estado sigue viniendo de las CLIs.
- [ ] 1.2 Verificar que cada acción existente sigue disponible y que no se duplica lógica en la app.

## Fase 2 — Accesibilidad

- [ ] 2.1 Definir y aplicar orden de foco, navegación por teclado y foco visible en las vistas principales.
- [ ] 2.2 Verificar contraste AA de texto y estados con los tokens de apariencia.
- [ ] 2.3 Respetar `prefers-reduced-motion` y anunciar estados de sesión y refresco con `aria-live`.

## Fase 3 — Rendimiento

- [ ] 3.1 Virtualizar listas de archivos, tareas y diffs, conservando scroll y foco.
- [ ] 3.2 Medir con listas/diffs largos y registrar el presupuesto de render en `docs/performance.md`.
- [ ] 3.3 Corregir las regresiones que superen el presupuesto (tareas largas > 100 ms).

## Fase 4 — Tests y documentación

- [ ] 4.1 Fixtures de repos bare, worktrees y conflictos; providers mockeados (`gh`/`glab`).
- [ ] 4.2 Tests de stores y componentes, y snapshots visuales de las vistas principales.
- [ ] 4.3 Guía de Git/worktrees/providers, feature flags con dueño/default y checklist de release desktop-only.
