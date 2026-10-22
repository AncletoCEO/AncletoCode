# PandaCode upstream

Código de terceros reutilizado por AncletoCode bajo la PandaCode License (MIT con
requisito de atribución). El texto completo está en [`LICENSE`](LICENSE).

- **Repositorio**: https://github.com/aicodeking/pandacode
- **Commit fijado**: `e9f89891fd23518848765aef8219dbf39feac6e0`
- **Versión**: 0.3.13
- **Fecha del commit**: 2026-10-21
- **Licencia**: PandaCode License (MIT + atribución de fork); GitHub la reporta como `NOASSERTION`.

## Archivos

La lista se actualiza al vendorizar cada módulo. La columna "Estado" distingue lo ya
incorporado de lo planificado.

| Ruta upstream | Ruta local | Nivel | Estado |
|---|---|---|---|
| `src-tauri/src/providers/windows_job.rs` | `third_party/pandacode/src-tauri/src/providers/windows_job.rs` | A | vendorizado |
| `src-tauri/src/providers/text_stream.rs` | `third_party/pandacode/src-tauri/src/providers/text_stream.rs` | A | vendorizado |
| `src-tauri/src/providers.rs` (extracción `Preview`/`limited`) | `third_party/pandacode/src-tauri/src/providers/support.rs` | B | vendorizado |
| `src-tauri/src/terminal.rs` | `third_party/pandacode/src-tauri/src/terminal.rs` | B | planificado |
| `src-tauri/src/workspace.rs` | `third_party/pandacode/src-tauri/src/workspace.rs` | B | planificado |
| `src-tauri/src/workspace/file_search.rs` | `third_party/pandacode/src-tauri/src/workspace/file_search.rs` | B | planificado |
| `src-tauri/src/appearance.rs` | `third_party/pandacode/src-tauri/src/appearance.rs` | B | planificado |
| `src-tauri/src/file_preview.rs` | `third_party/pandacode/src-tauri/src/file_preview.rs` | B (opcional) | planificado |
| `src-tauri/src/vsix_themes.rs` | `third_party/pandacode/src-tauri/src/vsix_themes.rs` | B (opcional) | planificado |
| `src/color.ts` | `third_party/pandacode/src/color.ts` | A | vendorizado |
| `src/palette-core.ts` | `third_party/pandacode/src/palette-core.ts` | A | vendorizado |
| `src/theme.ts` | `third_party/pandacode/src/theme.ts` | B | planificado |
| `src/preferences.ts` | `third_party/pandacode/src/preferences.ts` | B | planificado |
| `src/appearance.ts` | `third_party/pandacode/src/appearance.ts` | B | planificado |
| `src/terminal.ts` | `third_party/pandacode/src/terminal.ts` | B | planificado |
| `src/terminal-panel.ts` | `third_party/pandacode/src/terminal-panel.ts` | B | planificado |
| `src/transcript-scroll.ts` | `third_party/pandacode/src/transcript-scroll.ts` | A | vendorizado |
| `src/streaming-markdown.ts` | `third_party/pandacode/src/streaming-markdown.ts` | B | planificado |
| `src/ui-size.ts` | `third_party/pandacode/src/ui-size.ts` | B | planificado |
| `src/dom.ts` (subset Markdown/iconos) | `third_party/pandacode/src/dom.ts` | B | planificado |
| `src/vendor/enriched-markdown/md4c.js` | `third_party/pandacode/src/vendor/enriched-markdown/md4c.js` | A | vendorizado |
| `src/vendor/enriched-markdown/md4c.d.ts` | `third_party/pandacode/src/vendor/enriched-markdown/md4c.d.ts` | A | vendorizado |
| `src/vendor/enriched-markdown/README.md` | `third_party/pandacode/src/vendor/enriched-markdown/README.md` | A | vendorizado |
| `src/enriched-markdown.ts` | `third_party/pandacode/src/enriched-markdown.ts` | B | vendorizado |
| `src/streaming-markdown.ts` | `third_party/pandacode/src/streaming-markdown.ts` | B | vendorizado |

## Cabecera obligatoria

Cada archivo copiado o adaptado debe empezar con:

```text
SPDX-License-Identifier: MIT
SPDX-FileCopyrightText: 2026 PandaCode contributors
Source: https://github.com/aicodeking/pandacode/blob/<sha>/<ruta>
Modified by AncletoCode: <resumen>
```

En Rust y TypeScript usar `//`; en CSS usar `/* ... */`.

## Sincronización

1. Ejecutar `node scripts/sync-pandacode.mjs` (descarga el commit fijado y muestra el diff).
2. Revisar cada diferencia; no reemplazar archivos sin entender el cambio upstream.
3. Actualizar el commit, la fecha y `docs/provenance.md`.
4. Ejecutar `npm run license:check`, `cargo deny check`, `npm run test` y `cargo test`.

## Licencias de dependencias

Los módulos reutilizados arrastran crates (`windows`, `portable-pty`, `ignore`, `zip`,
`reqwest`, etc.) y paquetes npm (`@xterm/xterm`, `@xterm/addon-fit`, `dompurify`,
`katex`, `marked`). Sus licencias se auditan y se listan en `THIRD_PARTY_NOTICES.md`.
