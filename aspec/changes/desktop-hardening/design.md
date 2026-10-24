# Design: Hardening del escritorio (paneles, accesibilidad, tests y rollout)

## Approach

El hardening se trata como una capa transversal: no cambia el modelo de dominio, pero
define cómo se integra, se mide y se prueba. La integración de paneles se hace por
composición (vistas del mismo shell), la accesibilidad por contratos de teclado/foco y
la performance por virtualización con presupuesto medido.

```
paneles Ancleto ──► shell (vistas) ──► tests de regresión
                          │
        a11y (foco/teclado/contraste) ──► snapshots
                          │
        virtualización ──► presupuesto de render
```

## Architecture

- **Integración**: `spec`, `writer`, `conversación` y `sesiones` se montan como vistas
  del shell; el estado sigue viniendo de las CLIs/host. No se duplica lógica.
- **Accesibilidad**: orden de foco explícito, atajos documentados, contraste AA
  verificado con los tokens de `appearance`, `prefers-reduced-motion` respetado y
  `aria-live` para estados de sesión/refresh.
- **Virtualización**: listas de archivos, tareas, diffs y transcript con ventana
  virtual; se conserva el anclaje al mensaje visible.
- **Presupuesto de render**: se mide con PerformanceObserver en desarrollo y se
  documenta el umbral en `docs/performance.md`; una lista de 5.000 entradas no debe
  producir tareas largas > 100 ms.
- **Tests**: fixtures de repos bare/worktrees/conflictos, `gh` falso, stores y
  componentes con Testing Library, y snapshots de las vistas principales.
- **Rollout**: feature flags para GitHub y paneles nuevos, con default, dueño y fecha
  de revisión; guía de Git/worktrees/providers y checklist de release.

## Validation

```bash
npm run test
npm run typecheck
npm run lint
cargo test --workspace
npm run check:commands
```

- Tests de integración de paneles: acciones equivalentes a las vistas actuales.
- Auditoría de teclado: recorrer toda la app sin mouse y verificar foco visible.
- Perfil de rendimiento con listas y diffs largos; comparar contra el presupuesto.
- Revisión de snapshots visuales de las vistas principales.
