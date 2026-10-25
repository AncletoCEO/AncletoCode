# Tasks: Optimización de rutas calientes, persistencia y arranque (app-performance-2)

## Fase 1 — Detección de drivers y arranque

- [x] 1.1 Agregar caché en memoria con TTL de 60 s y timeout de 3 s por binario en `crates/ancleto-code-core/src/drivers/mod.rs`.
- [x] 1.2 Convertir `drivers_list` en comando `async` con `run_blocking` en `src-tauri/src/commands/mod.rs`.
- [x] 1.3 Reusar la información de caché en `driver_models` para evitar invocar `detect_all()` nuevamente.
- [x] 1.4 Deduplicar peticiones concurrentes en `src/stores/driversStore.ts` con una promesa única en vuelo.
- [x] 1.5 Test unitario de `drivers_list`: verificar respuesta ≤ 20 ms con caché y timeout ≤ 3 s ante binario lento.

## Fase 2 — Persistencia SQLite y eliminación de contención

- [x] 2.1 Configurar `PRAGMA busy_timeout = 5000;` en `crates/ancleto-code-core/src/persistence/mod.rs`.
- [x] 2.2 Gatear la ejecución de sentencias DDL en `Store::open` consultando `schema_migrations`.
- [x] 2.3 Unificar la instancia de base de datos en `src-tauri/src/lib.rs` compartiendo un único `Arc<Mutex<Store>>` entre `AppState`, `ProjectService` y `Orchestrator`.
- [x] 2.4 Propagar el error en `crates/ancleto-code-core/src/orch/mod.rs` si `store.add_turn()` falla, interrumpiendo la emisión del evento para asegurar consistencia.
- [x] 2.5 Test de estrés de concurrencia: dos writers concurrentes sobre `Store`, verificando cero fallos `SQLITE_BUSY` y cero turnos perdidos.

## Fase 3 — Flujo incremental de conversación

- [x] 3.1 Implementar el método `list_turns_since(conversation_id, since_index)` en `crates/ancleto-code-core/src/persistence/mod.rs` aprovechando el índice PK.
- [x] 3.2 Exponer el comando IPC `conversation_turns_since` en `src-tauri/src/commands/mod.rs`.
- [x] 3.3 Eliminar el intervalo de polling periódico de 1500 ms en `src/components/ConversationPanel.tsx`.
- [x] 3.4 Conectar el flujo reactivo a eventos IPC y añadir reconciliación incremental por reconexión/foco y watchdog de inactividad (10 s).
- [x] 3.5 Test de rendimiento de `conversation_turns_since`: verificar tiempo ≤ 2 ms y transferencia ≤ 32 KB en histórico de 800 turnos.

## Fase 4 — Comandos asíncronos y guardas de CI

- [x] 4.1 Migrar los comandos restantes con I/O (`project_list`, `settings_get`, `settings_put`, `task_*`, `conversation_*`) a `pub async fn` con `run_blocking`.
- [x] 4.2 Actualizar la tabla de allowlist en `docs/performance.md`, documentando con precisión el comportamiento real de cada comando.
- [x] 4.3 Robustecer `scripts/check-blocking-commands.mjs` para validar I/O transitivo hacia `Store` y `Command`.
- [x] 4.4 Ejecutar validaciones completas: `cargo test --workspace`, `npm run lint`, `npm run check:commands` y `npm run build`.
