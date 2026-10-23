# Tasks: Desktop 0.0.3

## 1. Usage por proveedor (#20)

- [ ] 1.1 Definir contrato `UsageProvider` y estados `ready/unavailable/signedOut/unsupported/error`.
- [ ] 1.2 Adapter OpenCode con cuota de plan además de tokens/costo local.
- [ ] 1.3 Adapter Claude Code con ventanas de la cuenta activa.
- [ ] 1.4 Adapter Antigravity con cuota o estado local disponible.
- [ ] 1.5 Adapter Command Code con cuota/plan si está expuesto.
- [ ] 1.6 Cache, timeouts, refresco manual y tests de parsers con fixtures.
- [ ] 1.7 UI: estado, fuente, reset y timestamp; nunca mostrar 0% sin dato.

## 2. GitHub review y merge (#21)

- [ ] 2.1 Listar checks/status de un PR.
- [ ] 2.2 Review approve/request changes y comentarios.
- [ ] 2.3 Merge con método configurable y confirmación.
- [ ] 2.4 Vincular PR con tarea, rama y worktree.
- [ ] 2.5 Errores de permisos, checks pendientes y conflictos.
- [ ] 2.6 Tests con `gh` mockeado y fixtures.

## 3. Worktree y Task/Thread (#22)

- [ ] 3.1 Abrir el contexto de un worktree como workspace activo.
- [ ] 3.2 Threads por tarea y asociación con sesión/agente/resultado.
- [ ] 3.3 Restauración de tareas, threads y worktrees al reabrir.
- [ ] 3.4 Limpieza asistida con cambios sin commit.
- [ ] 3.5 Tests de repos bare, worktrees huérfanos y conflictos.

## 4. Updates firmados (#23)

- [ ] 4.1 Configurar firma/endpoint de updates y canal stable/prerelease.
- [ ] 4.2 Descarga con progreso, verificación y rollback.
- [ ] 4.3 Instalación Windows/Linux con confirmación si hay sesiones activas.
- [ ] 4.4 Tests de firma inválida, descarga fallida y fallback.
- [ ] 4.5 Documentar claves y rotación.

## 5. Validación Windows (#24)

- [ ] 5.1 Instalar NSIS y MSI; abrir app y About.
- [ ] 5.2 Smoke de ZIP portable y host.
- [ ] 5.3 Proyecto, sesión PTY, tarea, worktree, Git y GitHub.
- [ ] 5.4 Ancleto embebido y updates.
- [ ] 5.5 Adjuntar evidencia y cerrar bloqueos.

## 6. Hardening Ancleto embebido (#25)

- [ ] 6.1 Test E2E de update real desde npm.
- [ ] 6.2 Test de integridad inválida y rollback.
- [ ] 6.3 Test offline con paquete incluido.
- [ ] 6.4 Errores accionables para versión incompatible.
- [ ] 6.5 Release 0.0.3 desktop-only con checksums y fuente.
