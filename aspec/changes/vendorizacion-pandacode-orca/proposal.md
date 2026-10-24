# Proposal: Vendorización PandaCode v0.3.26 y auditoría de referencia Orca

## Problem

El pin de PandaCode quedó en `v0.3.13` y la UI de escritorio todavía usa piezas
propias donde el upstream ya tiene módulos probados (sidebar-tree, task-board,
worktrees, github, diff-viewer, change-review, turn-changes, command-center,
agent-input). Además, la referencia visual de Orca no fue auditada formalmente: no
está registrado qué se tomó como referencia y qué se reutilizó, y la obligación de
atribución de Lovecast Inc. no está resuelta.

## Proposed change

- Actualizar el pin de PandaCode a `v0.3.26`
  (`800b879680bc89fda1189f726864ca1a4ed61e2a`), revisar el diff contra lo vendorizado
  y actualizar `UPSTREAM.md`, `docs/provenance.md`, avisos y tests.
- Auditar los módulos de Orca usados como referencia visual/UX. Si se reutiliza
  código o assets MIT, agregar aviso de Lovecast Inc., procedencia y tests; si no,
  registrar "referencia sin copia" en el ledger.
- Vendorizar/adaptar los módulos de UI seleccionados de PandaCode con cabecera SPDX,
  tests portados y entrada en el ledger.
- Vendorizar/adaptar los servicios Rust necesarios (`workspace.rs`, `github*.rs`,
  `task_controls.rs`, `turn_changes/`, `checks.rs`) manteniendo el núcleo
  independiente de Tauri.

## Scope

**In scope**

- Actualización de pin y sincronización revisada del código vendorizado.
- Auditoría y registro de la referencia Orca.
- Vendorización de UI y de servicios Rust con tests, cabeceras y ledger.
- Verificación de licencias y de la frontera del núcleo.

**Out of scope**

- Copiar activos (wallpapers, logos, screenshots, iconos, temas, fuentes).
- Reemplazar archivos vendorizados sin revisar el diff.
- Incorporar subsistemas de PandaCode fuera del alcance del producto.
- Reclasificar licencias o relicenciar código de terceros.

## Dependencias

- Se aplica después de archivar `ancleto-code-desktop-workplace`, que introduce la
  capability `license-compliance` en las specs principales.

## Risks

- **Diff grande entre v0.3.13 y v0.3.26**: se revisa archivo por archivo y no se
  reemplaza en bloque; `scripts/sync:pandacode` muestra el diff.
- **Acoplamiento a stores/providers de PandaCode**: los módulos Nivel B se adaptan al
  contrato propio y el diff queda en el ledger.
- **Dependencias nuevas**: cada crate o paquete incorporado pasa por
  `npm run license:check` y `cargo deny check`.
- **Frontera del núcleo**: los servicios vendorizados que dependan de Tauri quedan en
  `src-tauri`, nunca en `ancleto-code-core`.

## Success criteria

- `UPSTREAM.md`, `docs/provenance.md` y `THIRD_PARTY_NOTICES.md` reflejan el pin
  `v0.3.26` y todos los módulos incorporados.
- Cada archivo vendorizado tiene cabecera SPDX con commit y ruta, y sus tests pasan.
- La auditoría de Orca queda registrada con una decisión explícita por módulo.
- `npm run license:check`, `cargo deny check`, `npm run test` y `cargo test` en verde.
