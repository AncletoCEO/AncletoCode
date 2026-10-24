# Design: Vendorización PandaCode v0.3.26 y auditoría de referencia Orca

## Approach

Se aplica el flujo de vendorización de `AGENTS.md` sin excepciones: fijar upstream,
copiar preservando ruta, cabecera SPDX, ledger, tests portados y verificación. La
auditoría de Orca es una decisión separada: cada módulo se clasifica como
"referencia sin copia" o "reutilizado con aviso", y esa clasificación se registra
antes de escribir código.

```
upstream fijado ──► sync (diff) ──► clasificación A/B/C ──► copia/adaptación
                                                             │
                        SPDX + ledger + tests ◄──────────────┘
                                    │
                     license:check + cargo deny + suites
```

## Architecture

- **Pin**: `third_party/pandacode/UPSTREAM.md` guarda repo, commit `800b879...`,
  versión `v0.3.26` y fecha. Ningún archivo se actualiza sin revisar el diff contra
  el commit anterior.
- **Módulos UI**: se incorporan bajo `third_party/pandacode/<ruta-original>` y se
  consumen desde `src/` mediante wrappers; no se edita el original sin registrar el
  cambio en el ledger.
- **Servicios Rust**: `workspace.rs`, `github*.rs`, `task_controls.rs`,
  `turn_changes/` y `checks.rs` se adaptan en `src-tauri`; el núcleo
  `ancleto-code-core` sigue sin depender de `tauri`.
- **Orca**: si un módulo se reutiliza, se agrega el aviso de Lovecast Inc. (MIT) en
  `THIRD_PARTY_NOTICES.md` y su fila en `docs/provenance.md`; si solo inspira el
  diseño, se registra "referencia sin copia" con fecha y módulo.
- **Tests**: cada módulo portado trae sus tests upstream con la misma cabecera y se
  ejecuta en CI.
- **Activos**: prohibido copiar imágenes, logos, temas, iconos o fuentes; el
  inventario de `docs/assets-inventory.md` no cambia por este trabajo.

## Validation

```bash
node scripts/sync-pandacode.mjs --check
npm run license:check
cargo deny --manifest-path src-tauri/Cargo.toml check licenses advisories
npm run test
cargo test --workspace
npm run check:commands
```

- Revisar que cada archivo nuevo tenga la cabecera SPDX completa (licencia,
  copyright, source con commit y ruta, resumen de modificación).
- Verificar que `docs/provenance.md` tenga una fila por archivo vendorizado.
- Confirmar con un test de grafo que `ancleto-code-core` no incorpora Tauri.
