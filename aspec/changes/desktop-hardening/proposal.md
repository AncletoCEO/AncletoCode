# Proposal: Hardening del escritorio (paneles, accesibilidad, tests y rollout)

## Problem

Con el shell y Git en curso, faltan las condiciones para considerarlo terminado:
los paneles Ancleto (spec, writer, conversación, sesiones) no están integrados en el
nuevo shell sin regresiones; la accesibilidad (foco, teclado, contraste, reduced
motion) no está verificada; las listas y diffs no están virtualizados; y no hay
fixtures, snapshots visuales ni documentación de rollout.

## Proposed change

- Integrar los paneles Ancleto en el nuevo shell sin regresiones, manteniendo las CLIs
  como fuente de verdad del estado.
- Accesibilidad: orden de foco, navegación por teclado, contraste AA, `prefers-reduced-motion`
  y estados anunciados.
- Rendimiento: virtualizar listas y diffs, y fijar un presupuesto de render medible.
- Tests: fixtures de repos bare, worktrees, conflictos, providers mockeados y
  snapshots visuales de las vistas principales.
- Documentación y rollout: guía de Git/worktrees/providers, feature flags y checklist
  de release desktop-only.

## Scope

**In scope**

- Integración de paneles y paridad funcional.
- Accesibilidad y virtualización.
- Fixtures, tests de stores/componentes y snapshots.
- Documentación y feature flags.

**Out of scope**

- Nuevos proveedores Git.
- Cambios de diseño visual mayor.
- Rendimiento del motor PTY (ya cubierto por su propio presupuesto).
- Telemetría o métricas remotas.

## Risks

- **Regresiones al mover paneles**: cada panel mantiene sus tests y se compara el
  comportamiento antes/después.
- **Virtualización rompe scroll o foco**: se testea con listas largas y teclado, y se
  conserva el anclaje del transcript.
- **Snapshots frágiles**: se limitan a las vistas principales y se revisan con
  criterio, no se aceptan a ciegas.
- **Feature flags olvidadas**: cada flag tiene dueño, default y fecha de revisión.

## Success criteria

- Los paneles Ancleto funcionan dentro del nuevo shell sin perder estado ni acciones.
- Navegación completa por teclado y contraste AA en las vistas principales.
- Listas y diffs largos se mantienen dentro del presupuesto de render.
- Fixtures, tests y snapshots cubren el flujo principal; guía y checklist publicadas.
