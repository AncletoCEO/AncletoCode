# Proposal: AncletoCode Desktop 0.0.3 — usage, GitHub y hardening

## Problem

0.0.2 ya tiene el shell desktop-only, Git local, tareas/worktrees, GitHub básico y
Ancleto SDD embebido. Faltan las partes que convierten el MVP en un flujo diario
completo: cuotas reales de usage, revisión/merge de PRs, apertura de proyectos
desde worktrees, updates instalables y validación Windows.

## Scope

| Issue | Entrega |
|---|---|
| [#20](https://github.com/AncletoCEO/AncletoCode/issues/20) | Cuotas de usage por proveedor |
| [#21](https://github.com/AncletoCEO/AncletoCode/issues/21) | GitHub PR review, checks y merge |
| [#22](https://github.com/AncletoCEO/AncletoCode/issues/22) | Abrir worktree y completar Task/Thread |
| [#23](https://github.com/AncletoCEO/AncletoCode/issues/23) | Updates firmados e instalación desde la app |
| [#24](https://github.com/AncletoCEO/AncletoCode/issues/24) | Validación manual de Windows |
| [#25](https://github.com/AncletoCEO/AncletoCode/issues/25) | Hardening de update/rollback de Ancleto |

## In scope

- Adapters de cuota con estados explícitos y sin inventar porcentajes.
- PRs de GitHub con checks, review y merge seguro.
- Task/Thread/Worktree completo, incluyendo abrir el contexto del worktree.
- Updater firmado con rollback.
- Tests E2E de Ancleto embebido (éxito, corrupción, offline).
- Release 0.0.3 con `.deb`, AppImage, NSIS/MSI y portable.

## Out of scope

GitLab y otros hosts Git, AncletoCMD, CLI `acode`, npm, AUR y macOS.

## Success criteria

- Un usuario puede pasar de tarea a worktree, PR, review y merge sin salir de la app.
- Las cuotas muestran fuente y estado; los proveedores sin datos no muestran 0%.
- Una actualización fallida de la app o de Ancleto deja la versión anterior operativa.
- Windows pasa el checklist de #24.
