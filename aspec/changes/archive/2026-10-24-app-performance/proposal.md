# Proposal: Rendimiento de la app — pestaña Uso responsiva y comandos no bloqueantes

## Problem

Abrir la pestaña **Uso** tarda decenas de segundos y, durante ese tiempo, la ventana
completa queda sin responder. La causa no está en el render: `UsageView.tsx` son
cuatro tarjetas. El costo está en el backend y en cómo se ejecuta.

`usage_report` (`src-tauri/src/commands/mod.rs`) es un comando **sincrónico** y
`report()` (`src-tauri/src/usage.rs`) recolecta los cuatro proveedores **en serie**,
sin caché y sin reutilizar el resultado entre visitas. En esta máquina
(Linux, 2026-10-24) medido con los binarios reales:

| Llamada | Medición |
|---|---|
| `opencode stats --days 30` (proceso) | 1,5 s caliente / 6,5 s frío |
| OpenCode Go quota (HTTP) | 2,5 s (DNS ~3 s) |
| `agy -p /usage` (proceso, `--print-timeout 20s`) | 8,6 – 13,4 s |
| Command Code: credits + subscriptions + summary (3 HTTP secuenciales, mismo `Client`) | 4,9 + 3,5 + 4,0 ≈ 12,4 s |
| **Total secuencial** | **~25 – 35 s** |

Sobre eso, cuatro amplificadores:

1. **Todos los comandos Tauri son sincrónicos** (`0/80` con `async fn`). Un comando
   sync corre en el hilo principal: mientras dura, la ventana no repinta ni procesa
   eventos. No es "lento": es una UI congelada.
2. **Caché inexistente.** `UsageView` se desmonta al cambiar de vista y su
   `useEffect` vuelve a disparar el reporte completo en cada visita
   (`src/shell/UsageView.tsx`, `src/stores/usageStore.ts`).
3. **Todo-o-nada.** La grilla se pinta vacía hasta que termina el último proveedor;
   el primero podría mostrarse al segundo 1.
4. **Sin presupuesto de tiempo.** Timeouts de 15 s por request HTTP y 20 s para
   `agy`; el peor caso supera los 60 s. En desarrollo, `StrictMode` duplica el efecto
   y lanza dos reportes concurrentes.

El mismo patrón afecta a otros comandos con procesos o red (`git_*`, `spec_*`,
`writer_*`, `project_tree`, `session_snapshot`): cambiar de panel también congela la
ventana. Y en el arranque, el bundle inicial importa KaTeX CSS globalmente y monta
todas las vistas sin *code-splitting*.

## Proposed change

Tres fases, en orden de impacto:

**Fase 1 — Pestaña Uso (el caso reportado).**

- `usage_report` pasa a `async` y ejecuta la recolección en `spawn_blocking`.
- Los proveedores se recolectan en paralelo (un hilo por proveedor) y cada uno
  emite un evento `usage://provider` al terminar, para pintado progresivo.
- Caché en disco (`usage-cache.json` en `configDir`, escritura atómica, `0600`) con
  *stale-while-revalidate*: al entrar se sirve el último reporte al instante y, si
  está vencido, se refresca en background.
- *Single-flight*: un solo refresco a la vez (cubre doble click y `StrictMode`).
- Presupuesto de latencia: HTTP 5 s, `agy` y `opencode stats` con watchdog de 14 s
  y `kill`; refresco completo acotado a ≤ 15 s en el peor caso.
- El frontend deja de re-consultar si el reporte está fresco y muestra skeleton por
  proveedor mientras llega cada evento.

**Fase 2 — Barrido del patrón bloqueante.**

- Regla explícita y verificable: todo comando que haga *process spawn*, red o IO de
  disco sobre N archivos se declara `async` y mueve el trabajo a `spawn_blocking`.
- Se convierten los comandos de mayor uso: `git_*`, `spec_status`/`spec_discovery_*`,
  `writer_status`, `project_tree`, `project_search`, `session_snapshot`,
  `driver_models`.
- Guarda estática (`scripts/check-blocking-commands.mjs`) sobre los comandos y sus
  módulos, con allowlist justificada.

**Fase 3 — Arranque y bundle.**

- KaTeX CSS sale del chunk inicial y se carga solo cuando se renderiza matemática.
- Vistas y `TerminalView` (xterm) con `React.lazy` + `Suspense`.
- `checkUpdates()` se difiere al primer idle después del primer frame.
- Auditoría del `setup()` de Tauri (`HostClient::connect_or_start`) para no retrasar
  la creación de la ventana.

## Scope

**In scope**

- Latencia y responsividad de la pestaña Uso (backend + store + vista).
- Caché con frescura explícita y refresco manual.
- Eventos de progreso por proveedor.
- Conversión a `async`/`spawn_blocking` de los comandos bloqueantes enumerados.
- Presupuesto de bundle inicial y carga diferida de vistas y KaTeX CSS.
- Tests unitarios de caché, *single-flight*, timeouts y ausencia de secretos.
- Guarda estática de comandos bloqueantes en CI.

**Out of scope**

- Rediseño visual de la pestaña Uso o de otros paneles.
- Telemetría, métricas remotas o servicios de observabilidad.
- Nuevos proveedores de uso.
- AncletoCMD, CLI `acode` y empaquetado.
- Reescribir el host de sesiones o el motor PTY.

## Risks

- **Datos de uso obsoletos se muestran como actuales**: `checkedAt` visible,
  TTL de 5 min, botón "Actualizar" que fuerza, y estado `stale` en la UI.
- **Eventos perdidos al desmontar la vista**: el comando devuelve siempre un snapshot
  completo y el store hace merge por `provider`, no append ciego.
- **Credenciales filtradas a la caché**: la caché solo guarda datos derivados; test
  que escanea el JSON contra claves conocidas de auth y permisos `0600` en Unix.
- **Procesos colgados sin timeout nativo**: watchdog por hilo que hace `kill` +
  `wait` y reporta `status: "error"` con mensaje accionable.
- **Convertir comandos a `async` con `Mutex` de `AppState`**: riesgo de sostener el
  lock a través de un `await`; regla de diseño: el lock se toma y se suelta dentro
  del `spawn_blocking`.
- **Code-splitting rompe tests o el render inicial**: presupuesto medido en build y
  *fallback* `Suspense` con el mismo layout para evitar saltos.
- **Cambios de concurrencia alteran el orden de las tarjetas**: el runner ordena por
  índice estable de proveedor antes de publicar el snapshot.

## Success criteria

- Abrir Uso con caché válida: contenido visible en < 100 ms y sin bloqueo del hilo
  principal.
- Sin caché: primer proveedor visible en < 2 s; reporte completo acotado a ≤ 15 s
  (p95) con `agy` como techo real; p50 < 12 s en red normal.
- Ningún frame > 100 ms durante un refresco (ventana siempre responsiva).
- `usage-cache.json` no contiene claves de API ni tokens.
- Ningún comando que haga spawn, red o IO pesado permanece sincrónico sin
  justificación en allowlist.
- Bundle inicial gzip ≤ 180 KB (hoy 228 KB) y KaTeX CSS fuera del chunk inicial.
