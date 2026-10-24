# Design: Backlog diferido (CLI, AncletoCMD, npm y hosts Git alternativos)

## Approach

No hay diseño técnico que ejecutar: este change es un registro de trabajo fuera de
alcance. Cada ítem conserva su descripción original y agrega la condición que lo
reabre. No se crean delta specs porque no se compromete comportamiento nuevo.

## Architecture

- **Reapertura**: un ítem vuelve al camino activo cuando se cumple su condición
  (decisión de release de CLI/CMD, soporte de un host Git, publicación npm).
- **Trazabilidad**: cada tarea referencia su fase original del change
  `ancleto-code-desktop-workplace` para no perder contexto.
- **Sin specs**: al no haber comportamiento especificado, este change no tiene
  `specs/`; el motivo queda registrado en el proposal.

## Validation

- Revisión manual: el `tasks.md` del change activo no contiene tareas diferidas.
- Verificar que cada ítem del backlog declara su condición de reapertura.
