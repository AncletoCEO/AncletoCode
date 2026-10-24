# Design: Rendimiento de la app — pestaña Uso responsiva y comandos no bloqueantes

## Approach

Tres decisiones estructurales, cada una con su justificación:

1. **Ningún trabajo bloqueante en el hilo principal.** El comando pasa a `async fn` y
   todo el cuerpo bloqueante corre en `tauri::async_runtime::spawn_blocking`. Tauri
   ejecuta los comandos sync en el hilo principal; `async` los mueve al runtime, pero
   bloquear un worker del runtime también es incorrecto, de ahí `spawn_blocking`
   explícito. No se agrega `tokio` como dependencia directa: se usa el runtime que ya
   expone `tauri`.
2. **El último reporte es un activo, no un efecto de montaje.** Caché en disco +
   `stale-while-revalidate`: la UI nunca espera para pintar; el refresco es un evento
   del sistema, no una condición de render.
3. **Progreso por proveedor, no por lote.** Cada proveedor resuelve a su ritmo y
   publica su tarjeta. Un proveedor caído o lento no retrasa a los demás ni cambia su
   estado a error global.

Se descartaron: (a) cachear solo en memoria — se pierde al cerrar la app y el primer
arranque sigue costando 25–35 s; (b) bajar timeouts sin paralelizar — el total sigue
siendo la suma; (c) mover la lógica al núcleo — `usage` es un adaptador de la app
(lee credenciales del usuario y habla APIs de proveedores), no una regla de negocio
compartida, y el núcleo no debe depender de `reqwest`.

## Architecture

### Backend: módulo `usage/`

`src-tauri/src/usage.rs` se parte sin cambiar contratos públicos:

```
src-tauri/src/usage/
  mod.rs        tipos de dominio + fachada `report()`
  providers.rs  trait UsageProvider + 4 implementaciones (opencode, claude, gy, commandcode)
  runner.rs     ejecución paralela, eventos por proveedor, watchdog de timeouts
  cache.rs      snapshot en disco, TTL, escritura atómica, permisos
  service.rs    estado compartido: single-flight + snapshot vigente
```

```rust
pub trait UsageProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn collect(&self) -> ProviderUsage;
}

pub struct UsageService {
    snapshot: Mutex<Option<CachedSnapshot>>,
    refreshing: AtomicBool,
    providers: Vec<Box<dyn UsageProvider>>,
    cache_path: PathBuf,
    emit: Arc<dyn Fn(UsageEvent) + Send + Sync>,
}
```

`UsageProvider` permite inyectar proveedores falsos en tests sin red ni procesos.

### Contrato IPC

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshot {
    pub providers: Vec<ProviderUsage>,  // orden estable por id
    pub checked_at: u64,                // 0 = nunca
    pub stale: bool,                    // checked_at + TTL < now
    pub refreshing: bool,
    pub duration_ms: Option<u64>,       // último refresco completo
}

#[tauri::command]
pub async fn usage_report(
    state: State<'_, AppState>,
    force: bool,
) -> Result<UsageSnapshot, String>;
```

Semántica:

```
usage_report(force=false)
  ├─ snapshot en memoria válido (checked_at + 5min > now)  → devuelve ya, sin red
  ├─ snapshot en disco válido                              → carga, devuelve ya
  └─ sin snapshot o vencido
       ├─ devuelve snapshot actual (posiblemente vacío) + refreshing=true
       └─ lanza refresco single-flight en background

usage_report(force=true) → idéntico, pero lanza refresco aunque esté fresco
```

El refresco en background emite:

- `usage://provider` con `{ provider, usage, index }` al terminar cada proveedor.
- `usage://done` con el `UsageSnapshot` completo y `duration_ms`.

El frontend también recibe el snapshot de la invocación, así que un evento perdido
por desmontaje no pierde datos: al volver a montar, el store ya tiene el snapshot.

### Runner paralelo con presupuesto

```
                 ┌─ hilo opencode  (stats watchdog 14s + quota HTTP 5s)
usage_report ────┼─ hilo claude    (resolve_binary, < 10 ms)
 (spawn_blocking)├─ hilo agy       (print-timeout 12s + watchdog 14s)
                 └─ hilo commandcode (3 HTTP en un Client, 5s c/u, 1 reintento no)
                        │
                        ├─ emit(usage://provider) por cada uno
                        └─ join → ordenar por id → cache.put → emit(usage://done)
```

- Paralelismo con `std::thread::scope`: no hace falta un runtime async para trabajo
  bloqueante, y el `AppHandle` clonado es `Send + Sync` para emitir desde cada hilo.
- Presupuesto: el total es `max(proveedor)`, no la suma. Objetivo p95 ≤ 15 s con
  `agy` como techo real (9–13 s medidos).
- El mismo `reqwest::blocking::Client` se reutiliza para los tres endpoints de
  Command Code (ya ocurre) y para OpenCode Go se construye uno por proveedor; ambos
  se crean una vez por refresco, no por request.

### Caché

`<app_config_dir>/usage-cache.json`, escritura atómica (`usage-cache.json.tmp` +
`rename`) para no dejar un JSON a medias si se corta la energía:

```json
{
  "version": 1,
  "checkedAt": 1792800000,
  "providers": [ { "provider": "opencode", "...": "..." } ]
}
```

- Solo campos derivados de `ProviderUsage` (costos, tokens, ventanas, mensajes).
  Nunca credenciales, headers ni rutas de auth.
- Lectura tolerante: JSON corrupto o `version` desconocida → se trata como ausente.
- Permisos `0600` en Unix (`std::os::unix::fs::PermissionsExt`).
- TTL 5 min: las cuotas cambian lento; el botón "Actualizar" fuerza.

### Frontend

`src/stores/usageStore.ts`:

```ts
type UsageState = {
  providers: ProviderUsage[];   // merge por provider
  checkedAt: number;
  stale: boolean;
  refreshing: boolean;
  error: string | null;
  load: (force?: boolean) => Promise<void>;
  applyProvider: (usage: ProviderUsage) => void;  // evento
  applyDone: (snapshot: UsageSnapshot) => void;   // evento
};
```

- `UsageView` llama `load(false)` al montar: con caché fresca retorna al instante.
- Suscripción a `usage://provider` y `usage://done` con `listen` + cleanup; se
  registra una vez, no por render.
- Skeletons por tarjeta mientras `refreshing` y el proveedor no llegó.
- `checkedAt` visible ("actualizado hace X"), botón deshabilitado mientras
  `refreshing`, `aria-busy` en la grilla.

### Fase 2: regla de comandos y guarda

Regla: **un comando Tauri que spawnee procesos, use red o haga IO de disco sobre N
archivos se declara `async` y mueve el cuerpo a `spawn_blocking`.**

Conversión por comando: se conserva la firma, el `State` se clona (`Arc`-based) y el
lock se toma dentro del cierre bloqueante, nunca alrededor de un `await`.

Guarda `scripts/check-blocking-commands.mjs`: recorre `src-tauri/src/commands/*.rs`,
detecta bloques `#[tauri::command]` + `fn` sin `async`, extrae el nombre y busca en
todo `src-tauri/src` si ese comando referencia `Command::new`, `reqwest`,
`std::fs::` o `std::process`. Reporta `archivo:línea` y falla si no está en la
allowlist (`docs/performance.md`), que exige justificación escrita.

### Fase 3: arranque y bundle

- `main.tsx` deja de importar `katex/dist/katex.min.css` globalmente; un módulo
  `src/lib/math-styles.ts` hace `import("katex/dist/katex.min.css")` una sola vez
  antes del primer `typesetMath`. Vite lo emite como chunk CSS aparte. No se modifica
  el archivo vendorizado (evita tocar cabecera SPDX y ledger).
- `React.lazy` para `SpecPanel`, `WriterPanel`, `ConversationPanel`,
  `AppearancePanel`, `TasksView` y `TerminalView`, con `Suspense` que reutiliza el
  layout vacío actual para no introducir saltos.
- `checkUpdates()` se difiere con `requestIdleCallback` (fallback `setTimeout 0`)
  después del primer frame; el TTL de 12 h ya existe.
- Medición del `setup()`: instrumentar `HostClient::connect_or_start` con tiempos y,
  si supera 150 ms, moverlo a inicialización perezosa en el primer `session_create`.

## Validation

**Automática**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
npm run lint
npm run typecheck
npm run test
npm run license:check
node scripts/check-blocking-commands.mjs
```

Tests nuevos (sin red ni procesos reales):

- `cache`: round-trip, TTL (`stale`/fresco), JSON corrupto, versión desconocida,
  escritura atómica, permisos `0600`.
- `service`: *single-flight* (dos `load` concurrentes producen un solo refresco),
  snapshot servido antes de que termine el refresco.
- `runner`: orden estable con proveedores que terminan fuera de orden; un proveedor
  que falla no altera el estado de los demás; watchdog mata un hijo que duerme más
  que el presupuesto.
- `secrets`: la caché serializada no contiene `apiKey`, `key`, `token`,
  `authorization`, `Bearer`.
- Frontend: el store hace merge por `provider`, ignora eventos duplicados y no
  re-consulta si `checkedAt` está dentro del TTL.

**Manual / perf (documentado en `docs/performance.md`)**

- Con caché válida: abrir Uso y verificar contenido < 100 ms y ausencia de long
  tasks en DevTools Performance.
- Sin caché: medir primer proveedor (< 2 s) y `duration_ms` del snapshot
  (p50 < 12 s, p95 < 15 s), con `agy` disponible y sin él.
- Con un proveedor simulado lento/caído (test de integración con proveedor falso):
  las otras tarjetas se pintan y la ventana sigue respondiendo.
- Bundle: `vite build` y comparar gzip del chunk inicial contra el presupuesto
  (≤ 180 KB) y ausencia de `KaTeX_*.woff2` en la carga inicial.

**Criterio de cierre de cada fase**

- Fase 1 no se cierra sin: UI responsiva durante refresco, caché sin secretos y
  single-flight probado.
- Fase 2 no se cierra sin: guarda en CI en verde y allowlist con justificación.
- Fase 3 no se cierra sin: presupuesto de bundle medido y arranque sin regresión.
