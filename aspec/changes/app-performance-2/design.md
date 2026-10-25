# Design: Optimización de rutas calientes, persistencia y arranque (app-performance-2)

## Approach

El diseño aborda las 4 áreas identificadas reduciendo la sobrecarga de I/O bloqueante y el tráfico de IPC:

1. **Detección asincrónica de drivers y caché:**
   - Se modifica `DriverRegistry` para almacenar una caché de `DriverStatus` protegida por `std::sync::RwLock` con una marca de tiempo (`SystemTime`).
   - La llamada de detección se traslada a `spawn_blocking` con un timeout de 3 segundos por proceso hijo usando canales o watchdogs.
   - En frontend, `driversStore.ts` implementa un *in-flight promise deduplicator* de modo que múltiples llamadas a `loadDrivers()` en el arranque compartan la misma promesa.
   - `driver_models` consulta el `binary_path` desde la caché en lugar de ejecutar una nueva detección.

2. **Comandos IPC asíncronos y guarda transitiva:**
   - Comandos que tocan SQLite (`conversation_turns`, `conversation_get`, `conversation_list`, `project_list`, `settings_get`, `settings_put`, `task_*`) y procesos (`drivers_list`) se declaran `pub async fn` y delegan a `run_blocking`.
   - Se actualizan las descripciones en `docs/performance.md`.
   - Se ajusta `scripts/check-blocking-commands.mjs` para verificar estáticamente que los comandos sincrónicos no invoquen métodos de `Store` o `Command`.

3. **Arquitectura SQLite de conexión única y propagación:**
   - `src-tauri/src/lib.rs` inicializa un único `Arc<Mutex<Store>>` y lo inyecta en `AppState`, `ProjectService` y `Orchestrator`.
   - En `Store::open`, se añade `PRAGMA busy_timeout = 5000;` a la inicialización de la conexión.
   - Las sentencias DDL se ejecutan condicionalmente comprobando la tabla `schema_migrations`.
   - En `crates/ancleto-code-core/src/orch/mod.rs`, la persistencia del turno valida el resultado de `store.add_turn(&turn)`; si falla, se propaga el error y no se emite el evento para evitar inconsistencias.

4. **Flujo de conversación puramente reactivo:**
   - Se implementa el comando IPC `conversation_turns_since(id: String, since_index: usize) -> Result<Vec<Turn>, String>`.
   - En `ConversationPanel.tsx`, se elimina el `window.setInterval`. El hook monta con una carga inicial y se suscribe a los eventos `conversation://turn/${id}` y `conversation://state/${id}`.
   - Se añade un listener de evento `focus` en la ventana y un temporizador de watchdog de 10 s (solo si `state === 'running'` y no llegaron eventos) que consulta `conversation_turns_since(id, turns.length - 1)` para reconciliar sin re-descargar todo el histórico.

## Architecture

```
[ Frontend: DesktopShell / AgentTabs / Inspector ]
              │
              ├── loadDrivers() (Promise deduplicada) ──> [ IPC: drivers_list (async) ]
              │                                                     │
              │                                            [ DriverRegistry Cache ]
              │                                                     │ (si vencido / background)
              │                                            [ Process Pool (timeout 3s) ]
              │
[ Frontend: ConversationPanel ]
              │
              ├── Inicial: conversation_turns ──────────> [ IPC (async) ]
              ├── Eventos: conversation://turn/<id> <───── [ Orchestrator Events ]
              └── Reconciliación (focus / watchdog 10s) ─> [ IPC: conversation_turns_since ]
                                                                    │
                                                           [ Single Arc<Mutex<Store>> ]
                                                           [ PRAGMA busy_timeout=5000 ]
```

## Validation

- **`drivers_list` con caché:** p95 ≤ 20 ms.
- **Detección real:** en background, timeout de 3 s por binario, 0 bloqueo en hilo principal.
- **Comandos sync:** 0 comandos sincrónicos con I/O o procesos (guarda CI).
- **`conversation_turns_since`:** ≤ 32 KB y ≤ 2 ms p95 en 800 turnos.
- **SQLite:** 0 errores `BUSY` en test de contención con 2 writers concurrentes; 0 turnos perdidos.
- **Arranque:** shell interactivo ≤ 800 ms; ningún frame > 100 ms.
