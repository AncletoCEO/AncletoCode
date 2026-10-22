# Procedencia de código y referencias

Este ledger registra todo código de terceros incorporado a AncletoCode, así como las
referencias de diseño que no se copiaron.

- **Upstream de código reutilizado**: PandaCode, commit
  `e9f89891fd23518848765aef8219dbf39feac6e0` (v0.3.13).
- **Licencia**: PandaCode License (MIT + atribución de fork), ver
  [`third_party/pandacode/LICENSE`](../third_party/pandacode/LICENSE).
- **Avisos**: [`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md).
- **Lista de archivos y sincronización**: [`third_party/pandacode/UPSTREAM.md`](../third_party/pandacode/UPSTREAM.md).

## Código incorporado

Cada archivo derivado debe tener cabecera SPDX con el commit de origen y una fila en
esta tabla. La columna "Estado" pasa a `vendorizado` cuando el archivo existe bajo
`third_party/pandacode/`.

| Ruta local | Ruta upstream | Nivel | Licencia | Autor | Modificaciones | Estado |
|---|---|---|---|---|---|---|
| `third_party/pandacode/src-tauri/src/providers/windows_job.rs` | `src-tauri/src/providers/windows_job.rs` | A | MIT | PandaCode contributors | Ninguna | vendorizado |
| `third_party/pandacode/src-tauri/src/providers/text_stream.rs` | `src-tauri/src/providers/text_stream.rs` | A | MIT | PandaCode contributors | Ninguna | vendorizado |
| `third_party/pandacode/src-tauri/src/providers/support.rs` | `src-tauri/src/providers.rs` | B | MIT | PandaCode contributors | Extracción de `Preview` y `limited` | vendorizado |
| `third_party/pandacode/src-tauri/src/terminal.rs` | `src-tauri/src/terminal.rs` | B | MIT | PandaCode contributors | Reimplementado en `SessionManager` (PTY, eventos, cierre) e integrado con `windows_job` en Windows | reimplementado + windows_job |
| `third_party/pandacode/src-tauri/src/workspace.rs` | `src-tauri/src/workspace.rs` | B | MIT | PandaCode contributors | Rutas canónicas, árbol acotado y limpieza reimplementados; búsqueda vendorizada | reimplementado |
| `third_party/pandacode/src-tauri/src/workspace/file_search.rs` | `src-tauri/src/workspace/file_search.rs` | B | MIT | PandaCode contributors | `FileEntry` del core; sin prueba de worktrees | vendorizado |
| `third_party/pandacode/src-tauri/src/appearance.rs` | `src-tauri/src/appearance.rs` | B | MIT | PandaCode contributors | Reimplementado con `window-vibrancy` y fallback CSS | reimplementado |
| `third_party/pandacode/src-tauri/src/file_preview.rs` | `src-tauri/src/file_preview.rs` | B | MIT | PandaCode contributors | Opcional no habilitado | no habilitado |
| `third_party/pandacode/src-tauri/src/vsix_themes.rs` | `src-tauri/src/vsix_themes.rs` | B | MIT | PandaCode contributors | Opcional no habilitado | no habilitado |
| `third_party/pandacode/src/color.ts` | `src/color.ts` | A | MIT | PandaCode contributors | Se agrega `@ts-nocheck` para el tsconfig estricto | vendorizado |
| `third_party/pandacode/src/palette-core.ts` | `src/palette-core.ts` | A | MIT | PandaCode contributors | Ninguna | vendorizado |
| `third_party/pandacode/src/theme.ts` | `src/theme.ts` | B | MIT | PandaCode contributors | Reimplementado en el store de apariencia | reimplementado |
| `third_party/pandacode/src/preferences.ts` | `src/preferences.ts` | B | MIT | PandaCode contributors | Reimplementado en la configuración SQLite | reimplementado |
| `third_party/pandacode/src/appearance.ts` | `src/appearance.ts` | B | MIT | PandaCode contributors | Reimplementado en el store de apariencia | reimplementado |
| `third_party/pandacode/src/terminal.ts` | `src/terminal.ts` | B | MIT | PandaCode contributors | Reimplementado en `TerminalView` (React + xterm) | reimplementado |
| `third_party/pandacode/src/terminal-panel.ts` | `src/terminal-panel.ts` | B | MIT | PandaCode contributors | Reimplementado en tabs y paneles React | reimplementado |
| `third_party/pandacode/src/transcript-scroll.ts` | `src/transcript-scroll.ts` | A | MIT | PandaCode contributors | Ninguna | vendorizado |
| `third_party/pandacode/src/streaming-markdown.ts` | `src/streaming-markdown.ts` | B | MIT | PandaCode contributors | Adaptación al transcript propio | planificado |
| `third_party/pandacode/src/ui-size.ts` | `src/ui-size.ts` | B | MIT | PandaCode contributors | Adaptación a React y preferencias | planificado |
| `third_party/pandacode/src/dom.ts` | `src/dom.ts` | B | MIT | PandaCode contributors | Sólo subset Markdown/iconos | planificado |
| `third_party/pandacode/src/vendor/enriched-markdown/md4c.js` | `src/vendor/enriched-markdown/md4c.js` | A | MIT (Software Mansion) | Software Mansion | Ninguna | vendorizado |
| `third_party/pandacode/src/vendor/enriched-markdown/md4c.d.ts` | `src/vendor/enriched-markdown/md4c.d.ts` | A | MIT (Software Mansion) | Software Mansion | Ninguna | vendorizado |
| `third_party/pandacode/src/vendor/enriched-markdown/README.md` | `src/vendor/enriched-markdown/README.md` | A | MIT (Software Mansion) | Software Mansion | Ninguna | vendorizado |
| `third_party/pandacode/src/enriched-markdown.ts` | `src/enriched-markdown.ts` | B | MIT | PandaCode contributors | KaTeX/DOMPurify propios; sin tags generativos | vendorizado |
| `third_party/pandacode/src/streaming-markdown.ts` | `src/streaming-markdown.ts` | B | MIT | PandaCode contributors | Adaptado a nuestro render; sin file-links | vendorizado |

## Referencias de diseño (sin copia)

| Fuente | Qué se tomó | Dónde se aplica |
|---|---|---|
| PandaCode `src-tauri/src/storage.rs` | Patrones de migración, paginación y límites | Diseño de persistencia |
| PandaCode `src-tauri/src/providers.rs`, `providers/acp.rs`, `providers/launch.rs` | Contratos de protocolo, discovery y frames acotados | Diseño de drivers |
| PandaCode `src-tauri/src/architect*`, `message_queue.rs`, `turn_changes.rs` | Orquestación, watchdog, colas y diffs | Diseño de conversación |
| PandaCode `docs/design.md`, `docs/ipc.md`, `docs/providers.md` | Decisiones de arquitectura | `design.md` |

## Activos

No se incorporan wallpapers, temas, iconos, screenshots, logos ni fuentes de
PandaCode. Cualquier activo propio o de terceros debe registrarse con su origen y
licencia antes de entrar al repositorio.
