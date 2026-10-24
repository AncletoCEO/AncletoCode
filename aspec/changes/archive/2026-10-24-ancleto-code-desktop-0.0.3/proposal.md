# Proposal: AncletoCode Desktop 0.0.3 — cuotas de usage por proveedor

## Problem

La app no mostraba cuotas reales de usage por proveedor: no había contrato común,
adapters, caché ni estados explícitos, con riesgo de mostrar 0% cuando no hay datos.
Este change entrega esa pieza (#20). Las demás previstas para 0.0.3 se separaron en
changes propios (ver Split de alcance).

## Scope

| Issue | Entrega |
|---|---|
| [#20](https://github.com/AncletoCEO/AncletoCode/issues/20) | Cuotas de usage por proveedor |
| [#21](https://github.com/AncletoCEO/AncletoCode/issues/21) | GitHub PR review, checks y merge |
| [#22](https://github.com/AncletoCEO/AncletoCode/issues/22) | Abrir worktree y completar Task/Thread |
| [#23](https://github.com/AncletoCEO/AncletoCode/issues/23) | Updates firmados e instalación desde la app |
| [#24](https://github.com/AncletoCEO/AncletoCode/issues/24) | Validación manual de Windows |
| [#25](https://github.com/AncletoCEO/AncletoCode/issues/25) | Hardening de update/rollback de Ancleto |

## Split de alcance — 2026-10-24

`#20` (usage) se entregó en `app-performance`. Los demás bloques se movieron a changes
independientes: `#21` a `github-provider-prs`, `#22` a `workspace-shell-y-modelo`,
`#23` a `app-updater-firmado`, `#24` y `#25.5` a `desktop-release-packaging`, y
`#25.1–25.4` a `ancleto-runtime-e2e`. Este change queda limitado a la entrega de usage
y completo para archivar.

## In scope

- Adapters de cuota con estados explícitos y sin inventar porcentajes.
- Fuente, reset, timestamp y estado por proveedor en la UI.
- Caché, timeouts, refresco manual y tests de parsers con fixtures.
- Providers sin datos visibles como estado, nunca como 0%.

## Out of scope

- GitHub review/merge (#21): `github-provider-prs`.
- Worktree y Task/Thread (#22): `workspace-shell-y-modelo`.
- Updates firmados (#23): `app-updater-firmado`.
- Validación Windows (#24) y release (#25.5): `desktop-release-packaging`.
- Hardening de Ancleto embebido (#25.1–25.4): `ancleto-runtime-e2e`.
- GitLab y otros hosts Git, AncletoCMD, CLI `acode`, npm, AUR y macOS.

## Success criteria

- Los proveedores muestran cuota o estado explícito con fuente y timestamp.
- OpenCode, Antigravity y Command Code reportan datos reales; Claude queda como
  `unsupported`/`unavailable` porque su CLI no expone cuotas.
- Un proveedor sin datos no muestra 0% y el resto del reporte sigue disponible.
