# Performance — presupuesto, medición y allowlist de comandos

Documento operativo del change `app-performance`. Registra el baseline medido, el
presupuesto vigente, cómo se mide y qué comandos pueden quedar sincrónicos.

## Baseline (2026-10-24, Linux, red doméstica)

Pestaña Uso con los binarios y sesiones reales, recolección secuencial y sin caché:

| Llamada | Caliente | Frío |
|---|---|---|
| `opencode stats --days 30` | 1,5 s | 6,5 s |
| Cuota OpenCode Go (HTTP) | 2,5 s (DNS ~3 s) | 2,5 s |
| `agy -p /usage` (timeout 20 s) | 8,6 – 13,4 s | 13,4 s |
| Command Code: credits + subscriptions + summary | 4,9 + 3,5 + 4,0 s | 12,4 s |
| **Total serial** | **~25 s** | **~35 s** |

Los 80 comandos del backend eran sincrónicos, así que todo ese tiempo corría en el
hilo principal: la ventana quedaba congelada y el reporte se recalculaba completo en
cada visita a la pestaña.

Bundle inicial (build previo a este change):

| Activo | Raw | gzip |
|---|---|---|
| `index-*.js` | 808.696 B | 227.886 B |
| `katex-*.js` (diferido) | 258.935 B | 76.758 B |
| `md4c-*.js` (diferido) | 157.780 B | 53.934 B |
| `index-*.css` | 102.612 B | 20.190 B |

KaTeX CSS se importaba globalmente y arrastraba 39 archivos de fuentes al arranque.

## Post-implementación (2026-10-24, misma máquina)

Refresco real en paralelo, con `UsageService` y watchdog:

```text
provider claude: 0 ms (unavailable)
provider commandcode: 1921 ms (ready)
provider opencode: 4395 ms (ready)
provider antigravity: 9576 ms (ready)
total: 9576 ms, 4 proveedores
cache abierta: 80 µs + snapshot 3 µs
```

- El refresco pasó de la suma (~25–35 s) al máximo por proveedor (~10 s).
- Abrir la pestaña con caché válida cuesta ~0,1 ms de backend; el costo restante es
  el render de cuatro tarjetas.
- `agy` domina el presupuesto; su `--print-timeout` es 12 s y el watchdog 14 s porque
  en frío tarda 8,6–13,4 s. En una segunda corrida llegó a 14,0 s y el watchdog lo
  cortó con estado `error`; el refresco igual terminó acotado y el resto de los
  proveedores no se vio afectado.

Bundle después de diferir vistas, xterm y KaTeX CSS:

| Activo | Raw | gzip | Carga |
|---|---|---|---|
| `index-*.js` | 390.690 B | 119.870 B | arranque |
| `index-*.css` | 68.310 B | 11.670 B | arranque |
| `jsx-runtime-*.js` | 9.940 B | 3.770 B | arranque |
| `TerminalView-*.js` (xterm) | 333.380 B | 83.990 B | al abrir una sesión |
| `ConversationPanel-*.js` | 49.200 B | 18.410 B | al abrir conversación |
| `katex-*.js` | 258.930 B | 77.670 B | al renderizar matemática |
| `katex-*.css` | 29.800 B | 7.970 B | al renderizar matemática |
| `md4c-*.js` | 157.780 B | 54.310 B | al renderizar markdown |

El chunk inicial bajó de 227,9 KB a 119,9 KB gzip (−47 %) y `index.html` ya no
referencia KaTeX ni sus fuentes.

## Presupuesto vigente

| Métrica | Objetivo |
|---|---|
| Apertura con caché válida | < 100 ms, sin red ni procesos |
| Primer proveedor visible sin caché | < 2 s |
| Refresco completo p50 / p95 | < 12 s / < 15 s |
| Bloqueo del hilo principal | ningún frame > 100 ms |
| HTTP por request | 5 s |
| Proceso (`opencode stats`, `agy`) | watchdog 14 s |
| Bundle inicial JS (gzip) | ≤ 180 KB |

## Procedimiento de medición

1. Refresco real y caché:

   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml --test usage_latency \
     -- --ignored --nocapture
   ```

   El test usa proveedores reales, escribe la caché en un directorio temporal y
   reporta duración por proveedor, total y tiempo de apertura desde caché. No corre
   en la suite normal (`#[ignore]`).

2. Caché en la app instalada: `<app_config_dir>/usage-cache.json`. En Linux es
   `~/.config/com.ancleto.code/usage-cache.json`. Debe ser JSON legible, sin claves de
   API y con permisos `0600`.

3. Bundle:

   ```bash
   npm run build
   for f in dist/assets/index-*.js dist/assets/index-*.css; do
     printf '%s raw=%s gzip=%s\n' "$f" "$(stat -c%s "$f")" "$(gzip -c "$f" | wc -c)"
   done
   ```

4. Responsividad: con la app abierta, grabar un Performance profile en el webview
   durante un refresco y verificar que no haya tareas largas > 100 ms atribuibles al
   IPC. Complementario: `time` de la pestaña no debe coincidir con el total del
   refresco.

5. Arranque: `setup()` registra un aviso cuando `HostClient::connect_or_start` supera
   150 ms (`ancleto-code: el host tardó N ms en conectar`). Mientras no haya una
   medición real que lo supere, el host queda en el arranque; si aparece el aviso, el
   paso siguiente es moverlo a inicialización perezosa en el primer `session_create`.

6. Vista de Uso (automatizado, sin display):

   ```bash
   npx vitest run src/shell/UsageView.test.tsx
   ```

   Cubre skeletons por proveedor pendiente, pintado progresivo por evento, datos
   vencidos visibles durante el refresco, refresco forzado desde el botón, ausencia de
   re-consulta con snapshot fresco y error sin perder los datos previos.

7. Arranque real bajo Xvfb:

   ```bash
   npm run smoke:app
   ```

   Levanta el binario con la dist compilada durante 10 s y falla ante panic, error de
   librerías o salida inesperada. Reporta el aviso de host si superó 150 ms.

   Limitación: no hay automatización de input real sobre el webview. La responsividad
   durante un refresco se sostiene con el diseño `async` + `spawn_blocking`, la guarda
   `check:commands` y los tests de vista; una prueba con mouse/teclado queda como
   verificación exploratoria manual.

## Regla de comandos

Registrada también en `AGENTS.md`:

> Todo comando Tauri que inicie procesos, use red o haga IO de disco sobre un
> conjunto de archivos se declara `async` y ejecuta el cuerpo bloqueante en
> `tauri::async_runtime::spawn_blocking`. Los locks de `AppState` se toman y sueltan
> dentro del cierre bloqueante, nunca alrededor de un `await`.

Guarda:

```bash
node scripts/check-blocking-commands.mjs          # reporte
node scripts/check-blocking-commands.mjs --check  # falla si hay violaciones
```

La guarda lee la tabla de allowlist de este documento. Una excepción sin fila acá
hace fallar el check.

## Allowlist de comandos sincrónicos

| Comando | Motivo |
|---|---|
| `core_version` | Devuelve una constante; sin IO. |
| `session_write` | Escritura PTY en el camino de las teclas; debe resolverse en el hilo de IPC sin cola adicional. |
| `session_resize` | Aviso de resize al PTY; operación corta de control. |
| `session_kill` | Señal de cierre al PTY; operación corta de control. |
| `session_create` | Creación de sesión en el backend; el arranque del PTY es asíncrono aguas abajo. |
| `session_list` | Lista en memoria del backend de sesiones. |
| `project_add` | Alta en SQLite local; sin escaneo de árbol. |
| `project_list` | Lista en memoria/SQLite de proyectos registrados. |
| `project_remove` | Baja en SQLite local. |
| `project_open` | Lookup y persistencia de la selección; el escaneo es `project_tree`. |
| `settings_get` | Lectura de una clave en SQLite. |
| `settings_put` | Escritura de una clave en SQLite. |
| `task_create` | Inserción de una fila en SQLite. |
| `task_update` | Actualización de una fila en SQLite. |
| `task_delete` | Borrado de una fila en SQLite. |
| `task_start` | Actualización de estado de una tarea en SQLite. |
| `drivers_list` | Resolución de binarios en PATH; sin spawn. |
| `appearance_native_effect` | Efecto nativo de ventana; requiere el hilo principal. |
| `conversation_start` | Orquestador en memoria que delega a hilos propios. |
| `conversation_pause` | Cambio de estado en memoria. |
| `conversation_resume` | Cambio de estado en memoria. |
| `conversation_abort` | Cambio de estado en memoria y señal a hilos existentes. |
| `conversation_get` | Lectura en memoria. |
| `conversation_turns` | Lectura en memoria. |
| `conversation_list` | Lectura en memoria. |
