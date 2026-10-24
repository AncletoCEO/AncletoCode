# Tasks: Rendimiento de la app — pestaña Uso responsiva y comandos no bloqueantes

## Fase 0 — Baseline y guardas de medición

- [x] 0.1 Instrumentar `usage_report` con tiempos por proveedor y `duration_ms` total en el snapshot (temporal, se conserva como campo de diagnóstico).
- [x] 0.2 Registrar baseline en `docs/performance.md`: tiempos medidos de `opencode stats`, cuota OpenCode Go, `agy -p /usage` y los 3 endpoints de Command Code, en caliente y en frío.
- [x] 0.3 Documentar el presupuesto de latencia y bundle con los umbrales del proposal y el procedimiento de medición manual.
- [x] 0.4 Agregar `scripts/check-blocking-commands.mjs` en modo reporte (sin fallar) y volcar el inventario actual de comandos sync con IO bloqueante.
- [x] 0.5 Verificar que `cargo test --workspace` y `npm run test` pasan antes de tocar código de producción.

## Fase 1 — Pestaña Uso responsiva

### Backend

- [x] 1.1 Partir `src-tauri/src/usage.rs` en `usage/{mod,providers,runner,cache,service}.rs` sin cambiar la salida de `report()`; tests existentes intactos.
- [x] 1.2 Definir `trait UsageProvider` con `id()`/`collect()` e implementarlo para opencode, claude, antigravity y commandcode.
- [x] 1.3 Convertir `usage_report` a `async fn` y ejecutar la recolección completa en `tauri::async_runtime::spawn_blocking`.
- [x] 1.4 Implementar el runner paralelo con `std::thread::scope`, orden estable por `provider` y join acotado por el presupuesto.
- [x] 1.5 Implementar watchdog de procesos (`kill` + `wait`) para `opencode stats` y `agy`; bajar `--print-timeout` de 20 s a 12 s y el watchdog a 14 s.
- [x] 1.6 Bajar el timeout HTTP de OpenCode Go y Command Code de 15 s a 5 s, manteniendo un `Client` por proveedor por refresco.
- [x] 1.7 Implementar `UsageService` con snapshot en memoria, `refreshing: AtomicBool` y *single-flight*.
- [x] 1.8 Implementar `cache.rs`: serialización versionada, lectura tolerante, TTL 5 min, escritura atómica y permisos `0600` en Unix.
- [x] 1.9 Definir el contrato `UsageSnapshot` (`providers`, `checkedAt`, `stale`, `refreshing`, `durationMs`) y ajustar el comando para devolverlo.
- [x] 1.10 Emitir `usage://provider` por proveedor y `usage://done` al cerrar el refresco, con `index` estable.
- [x] 1.11 Registrar `UsageService` en `AppState` y eliminar el camino sync anterior de `usage_report`.

### Frontend

- [x] 1.12 Actualizar `usageStore` al nuevo contrato: `load(force)`, merge por `provider`, `applyProvider`, `applyDone`, `stale`, `refreshing`, `checkedAt`.
- [x] 1.13 Suscribirse a `usage://provider` y `usage://done` una sola vez con cleanup; ignorar eventos duplicados o de un refresco anterior.
- [x] 1.14 En `UsageView`, no re-consultar si el snapshot está fresco; `Actualizar` llama `load(true)` y se deshabilita mientras `refreshing`.
- [x] 1.15 Mostrar skeletons por tarjeta durante el refresco y el texto de frescura ("actualizado hace X min"); `aria-busy` en la grilla.
- [x] 1.16 Cubrir con tests: TTL/frescura, merge por proveedor, evento duplicado, doble montaje (`StrictMode`) produce un solo refresco.
- [x] 1.17 Cubrir con tests de Rust: caché (round-trip, corrupta, versión, permisos), *single-flight*, orden estable y watchdog.
- [x] 1.18 Test que verifica que la caché serializada no contiene `apiKey`, `key`, `token`, `authorization` ni `Bearer`.
- [x] 1.19 Verificación manual: abrir Uso en frío y en caliente, comprobar primer proveedor < 2 s, `durationMs` p50 < 8 s y ventana responsiva.

## Fase 2 — Barrido de comandos bloqueantes

- [x] 2.1 Convertir `git_status`, `git_branches`, `git_diff`, `git_worktrees`, `git_remote`, `git_pull_requests`, `git_issues` a `async` + `spawn_blocking`, sin sostener locks a través de `await`.
- [x] 2.2 Convertir `spec_status`, `spec_discovery_check`, `spec_discovery_generate`, `spec_specs_check`, `spec_memory_list`, `spec_memory_context` a `async` + `spawn_blocking`.
- [x] 2.3 Convertir `writer_status`, `writer_check`, `writer_impact`, `writer_agent_doc`, `writer_languages_status` a `async` + `spawn_blocking`.
- [x] 2.4 Convertir `project_tree`, `project_search`, `session_snapshot`, `driver_models`, `task_list` a `async` + `spawn_blocking`.
- [x] 2.5 Revisar que ningún `Wgpu`/`AppState` `Mutex` se mantenga tomado a través de un `await`; ajustar con bloques acotados.
- [x] 2.6 Endurecer `check-blocking-commands.mjs`: falla en CI si aparece un comando sync con red, spawn o IO pesado fuera de la allowlist.
- [x] 2.7 Crear la allowlist en `docs/performance.md` con justificación por excepción (p. ej. comandos triviales de un solo archivo).
- [x] 2.8 Agregar la regla de comandos no bloqueantes a `AGENTS.md` (sección Código) y enlazarla en `docs/performance.md`.
- [x] 2.9 Verificación manual: cambiar entre paneles de Git, Spec, Writer y Workspace con la ventana respondiendo durante la carga.

## Fase 3 — Arranque y bundle

- [x] 3.1 Medir el bundle inicial actual (`vite build`, gzip del chunk `index-*.js` y CSS) y registrar el baseline en `docs/performance.md`.
- [x] 3.2 Quitar `katex/dist/katex.min.css` de `main.tsx` e inyectarlo desde `src/lib/math-styles.ts` con `import()` la primera vez que se renderiza matemática.
- [x] 3.3 Verificar que `KaTeX_*.woff2` no se descargan en el arranque y que la matemática conserva estilos.
- [x] 3.4 Envolver `SpecPanel`, `WriterPanel`, `ConversationPanel`, `AppearancePanel` y `TasksView` en `React.lazy` + `Suspense` con el layout vacío actual.
- [x] 3.5 Envolver `TerminalView` (xterm) en `React.lazy` + `Suspense` para sacar xterm del chunk inicial.
- [x] 3.6 Diferir `checkUpdates()` a `requestIdleCallback` (fallback `setTimeout 0`) después del primer frame.
- [x] 3.7 Instrumentar `HostClient::connect_or_start` en `setup()`; si supera 150 ms, moverlo a inicialización perezosa en el primer `session_create`.
- [x] 3.8 Comprobar que el chunk inicial gzip queda ≤ 180 KB y que los tests de UI siguen pasando con los límites de `Suspense`.

## Fase 4 — Cierre

- [x] 4.1 Ejecutar la verificación completa del proyecto (`cargo fmt --check`, `clippy`, `cargo test`, `lint`, `typecheck`, `npm test`, `license:check`, guarda de comandos).
- [x] 4.2 Repetir la medición end-to-end de Uso y de cambio de paneles; comparar contra el baseline de 0.2.
- [x] 4.3 Actualizar `docs/performance.md` con resultados finales, umbrales y procedimiento de regresión.
- [x] 4.4 Revisar el diff completo contra el scope del proposal y dejar fuera cualquier cambio no relacionado.
