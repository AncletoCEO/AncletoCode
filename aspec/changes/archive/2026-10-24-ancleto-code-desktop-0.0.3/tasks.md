# Tasks: Desktop 0.0.3

> **Cierre de alcance — 2026-10-24.** Usage se entregó en `app-performance`. El resto
> se separó en changes propios para que este change quede completo:
>
> | Bloque | Destino |
> |---|---|
> | GitHub review y merge (#21) | `github-provider-prs` |
> | Worktree y Task/Thread (#22) | `workspace-shell-y-modelo` |
> | Updates firmados (#23) | `app-updater-firmado` |
> | Validación Windows (#24) | `desktop-release-packaging` |
> | Hardening Ancleto (#25.1–25.4) | `ancleto-runtime-e2e` |
> | Release 0.0.3 (#25.5) | `desktop-release-packaging` |

## 1. Usage por proveedor (#20) — entregado por `app-performance`

- [x] 1.1 Definir contrato `UsageProvider` y estados `ready/unavailable/signedOut/unsupported/error`.
- [x] 1.2 Adapter OpenCode con cuota de plan además de tokens/costo local.
- [x] 1.3 Adapter Claude Code con estado `unsupported`/`unavailable` (el CLI no expone cuotas de cuenta).
- [x] 1.4 Adapter Antigravity con cuota o estado local disponible.
- [x] 1.5 Adapter Command Code con cuota/plan si está expuesto.
- [x] 1.6 Cache, timeouts, refresco manual y tests de parsers con fixtures.
- [x] 1.7 UI: estado, fuente, reset y timestamp; nunca mostrar 0% sin dato.
