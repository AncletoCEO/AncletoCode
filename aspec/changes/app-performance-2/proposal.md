# Proposal: Optimización de rutas calientes, persistencia y arranque (app-performance-2)

## Problem

La auditoría técnica post-`0.1.0` evidenció cuatro cuellos de botella críticos que degradan la responsividad de la app de escritorio:

1. **Bloqueo masivo en el arranque por detección síncrona de drivers:**
   `drivers_list` (`src-tauri/src/commands/mod.rs:216`) es un comando sincrónico que ejecuta `Command::new(&path).arg("--version").output()` para cada driver registrado. En el sistema actual, invocaciones como `commandcode --version` demoran entre 3,3 y 3,8 segundos. Además, `driversStore.ts` carece de deduplicación de peticiones concurrentes y es invocado simultáneamente por `DesktopShell.tsx`, `AgentTabs.tsx` e `Inspector.tsx`, congelando el hilo principal de la UI entre 10 y 11 segundos durante el primer render. Adicionalmente, `driver_models` repite esta detección redundante.
2. **Comandos IPC sincrónicos con I/O y allowlist imprecisa:**
   24 de los 79 comandos de Tauri continúan ejecutándose de forma síncrona en el hilo principal (`body_blocking`). Varios de ellos realizan operaciones de SQLite (`conversation_*`, `project_*`, `settings_*`) y procesos externos (`drivers_list`). La allowlist de `docs/performance.md` y la guarda `scripts/check-blocking-commands.mjs` no detectan el I/O transitivo dentro del cuerpo de las funciones invocadas.
3. **Contención y pérdida silenciosa en persistencia SQLite:**
   La aplicación abre dos conexiones independientes `Store` sobre el mismo archivo `.sqlite` (`ProjectService` y `Orchestrator`) sin `busy_timeout` (default 0 en rusqlite). Ante contención de escrituras, la llamada falla inmediatamente con `database is locked`, y `orch/mod.rs:261` descarta el error con `let _ = store.add_turn(&turn)` mientras sigue emitiendo el evento `Turn`, provocando que la UI muestre turnos que jamás se persistieron. Asimismo, `Store::open` ejecuta el batch DDL en cada apertura sin verificación de versión.
4. **Polling redundante O(n) en la vista de conversación:**
   `ConversationPanel.tsx:154` mantiene un intervalo de 1500 ms que solicita la lista completa de turnos y la conversación entera (`conversation_turns` y `conversation_get`), recalculando deduplicación y ordenamiento con `Set` sobre todo el historial. Con 800 turnos sintéticos, cada poll transfiere más de 600 KB de JSON en el hilo principal, a pesar de que los eventos reactivos IPC (`conversation://turn/*`) ya proveen los datos en tiempo real.

## Proposed change

Implementar el change `app-performance-2` sobre la spec `performance` atacando los 4 frentes:

1. **`driver-detection-cached`**: Mover la detección a segundo plano con timeout estricto de 3 s por binario, caché con TTL en memoria y deduplicación de llamadas en frontend (`driversStore`). Reusar el path resuelto en `driver_models`.
2. **`ipc-command-io-contract`**: Pasar comandos con I/O a `async` + `spawn_blocking`, actualizar la allowlist y robustecer la guarda de CI para validar I/O transitivo.
3. **`sqlite-single-writer`**: Compartir una única instancia `Arc<Mutex<Store>>` entre servicios, configurar `PRAGMA busy_timeout = 5000`, gatear DDL con `schema_migrations` y propagar errores de persistencia de turnos.
4. **`conversation-stream-incremental`**: Eliminar el polling de 1,5 s en `ConversationPanel.tsx`, alimentando la lista por eventos reactivos `O(1)` y creando el comando incremental `conversation_turns_since` para reconciliación bajo demanda.

## Scope

- **In scope:**
  - `src-tauri/src/commands/mod.rs` (migración a async y endpoint `conversation_turns_since`).
  - `crates/ancleto-code-core/src/drivers/mod.rs` (caché y timeout de detección).
  - `crates/ancleto-code-core/src/persistence/mod.rs` (`busy_timeout`, migraciones gateadas).
  - `crates/ancleto-code-core/src/orch/mod.rs` (propagación de errores en guardado de turnos).
  - `src/stores/driversStore.ts` y `src/components/ConversationPanel.tsx` (deduplicación y consumo reactivo).
  - `docs/performance.md` y `scripts/check-blocking-commands.mjs` (allowlist y guarda).
- **Out of scope:**
  - Cambios en el renderer de terminal de `acmd` (reservado para change de TUI).
  - Modificación del esquema de base de datos o migración de tablas existentes.

## Risks

- **Driver desactualizado en caché:** Si el usuario instala un CLI durante la sesión, la caché podría reportarlo como `missing`. *Mitigación:* TTL de 60 s y acción explícita de refresco desde Ajustes.
- **Pérdida de eventos IPC de conversación durante reconexión:** *Mitigación:* Reconciliación con `conversation_turns_since` al recuperar el foco de la ventana o tras 10 s de inactividad de eventos mientras el estado sea `running`.
