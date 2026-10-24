# Tasks: Vendorización PandaCode v0.3.26 y auditoría de referencia Orca

## Fase 1 — Pin y auditoría

- [ ] 1.1 Actualizar el pin de PandaCode a `v0.3.26` (`800b879680bc89fda1189f726864ca1a4ed61e2a`), revisar el diff contra los archivos vendorizados y actualizar `UPSTREAM.md`, ledger, avisos y tests.
- [ ] 1.2 Auditar qué módulos de Orca se usan solo como referencia visual/UX. Si se reutiliza código o assets MIT, agregar aviso de Lovecast Inc., procedencia y tests; si no, registrar "referencia sin copia".
- [ ] 1.3 Confirmar que no se incorporan activos de PandaCode ni de Orca y actualizar `docs/assets-inventory.md` si corresponde.

## Fase 2 — UI

- [ ] 2.1 Vendorizar/adaptar `sidebar-tree` con cabecera SPDX, tests portados y entrada en el ledger.
- [ ] 2.2 Vendorizar/adaptar `task-board` y `worktrees` con cabecera SPDX, tests y ledger.
- [ ] 2.3 Vendorizar/adaptar `github*`, `diff-viewer`, `change-review` y `turn-changes` con cabecera SPDX, tests y ledger.
- [ ] 2.4 Vendorizar/adaptar `command-center` y `agent-input` con cabecera SPDX, tests y ledger.
- [ ] 2.5 Integrar los módulos vendorizados en el shell actual sin duplicar estilos ni lógica.

## Fase 3 — Servicios Rust

- [ ] 3.1 Vendorizar/adaptar `workspace.rs` conservando scoping, límites y búsqueda; tests con rutas canónicas.
- [ ] 3.2 Vendorizar/adaptar `github*.rs` y `checks.rs` detrás de `GitHostProvider`, sin dependencias de Tauri en el núcleo.
- [ ] 3.3 Vendorizar/adaptar `task_controls.rs` y `turn_changes/` con tests de persistencia y diffs.
- [ ] 3.4 Verificar que `ancleto-code-core` no incorpora `tauri`, `wgpu`, `winit` ni dependencias del host.

## Fase 4 — Verificación

- [ ] 4.1 Ejecutar `node scripts/sync-pandacode.mjs --check`, `npm run license:check`, `cargo deny check`, `npm run test` y `cargo test --workspace`.
- [ ] 4.2 Revisar que cada archivo vendorizado tenga cabecera SPDX completa y fila en `docs/provenance.md`.
- [ ] 4.3 Actualizar `THIRD_PARTY_NOTICES.md` y el About si cambia el inventario de terceros.
