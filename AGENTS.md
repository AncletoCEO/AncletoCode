# AGENTS.md — convenciones de AncletoCode

AncletoCode es un workspace de agentes de código para escritorio y terminal. El
trabajo se organiza con el flujo SDD de `@ancleto/spec`: los cambios viven en
`aspec/changes/<nombre>/` y se implementan desde su `tasks.md`.

## Licencias y procedencia

El proyecto es `AGPL-3.0-only` y es un fork de PandaCode. Toda contribución se
distribuye bajo la AGPL y conserva los avisos de terceros.

- Aviso de licencia de terceros: `THIRD_PARTY_NOTICES.md`.
- Ledger de código incorporado: `docs/provenance.md`.
- Upstream fijado y procedimiento de sync: `third_party/pandacode/UPSTREAM.md`.
- Inventario de activos: `docs/assets-inventory.md`.

### Cabecera SPDX obligatoria

Todo archivo vendorizado o adaptado de `third_party/pandacode/` debe empezar con
esta cabecera, en el comentario del lenguaje del archivo:

```text
SPDX-License-Identifier: MIT
SPDX-FileCopyrightText: 2026 PandaCode contributors
Source: https://github.com/aicodeking/pandacode/blob/<sha>/<ruta>
Modified by AncletoCode: <resumen del cambio>
```

- En Rust, TypeScript, JavaScript y JSON con comentarios: `//`.
- En CSS: `/* ... */`.
- En HTML: `<!-- ... -->`.
- El `<sha>` es el commit fijado en `third_party/pandacode/UPSTREAM.md`; no se usa
  un hash móvil.
- `<ruta>` es la ruta original del archivo en el repositorio upstream.
- `<resumen del cambio>` describe la adaptación; si no hubo cambios, se escribe
  `Ninguna`.
- Los tests portados llevan la misma cabecera.

### Alta de un archivo vendorizado

1. Copiar el archivo preservando su ruta original bajo
   `third_party/pandacode/<ruta-original>`.
2. Agregar la cabecera SPDX con el commit fijado.
3. Agregar la fila correspondiente en `docs/provenance.md` y en
   `third_party/pandacode/UPSTREAM.md`, cambiando el estado a `vendorizado`.
4. Portar los tests upstream del módulo.
5. Ejecutar `npm run license:check`, `cargo deny check`, `npm run test` y
   `cargo test` antes de commitear.

Un archivo adaptado conserva el aviso de copyright original y registra el diff en
el ledger. Nunca se reemplaza un archivo vendorizado por una versión nueva sin
revisar el diff y actualizar commit, ledger y avisos.

### Activos

No se incorporan wallpapers, temas, iconos, screenshots, logos ni fuentes de
PandaCode o de proveedores. Los activos propios se generan y los de terceros sólo
entran si su licencia permite redistribución, con origen y licencia registrados en
`docs/assets-inventory.md` y, si corresponde, en `THIRD_PARTY_NOTICES.md`.

## Código

- Rust: `cargo fmt` y `cargo clippy` sin warnings nuevos. El núcleo
  `ancleto-code-core` no depende de `tauri`, `winit`, `wgpu`, del host, de la CLI
  ni de AncletoCMD.
- TypeScript: modo `strict`; sin `any` implícito. Prettier y ESLint se ejecutan
  antes del commit.
- Comandos IPC: todo `#[tauri::command]` que inicie procesos, use red o haga IO de
  disco sobre N archivos es `async` y ejecuta el cuerpo bloqueante en
  `tauri::async_runtime::spawn_blocking`; los locks no se sostienen a través de un
  `await`. Las excepciones se justifican en `docs/performance.md` y las verifica
  `npm run check:commands`.
- Reglas de negocio en el núcleo compartido. La app, AncletoCMD y la CLI son
  consumidores delgados.
- El PTY es la fuente de verdad de las sesiones; el parseo nunca bloquea la
  operación.
- Las CLIs `ancleto` y `ancleto-writer` son la fuente de verdad de spec, memoria,
  discovery y grafo. La app no las reimplementa ni escribe en `aspec/`,
  `.ancletorc` ni `.ancleto-writer.json` para estado propio.
- No se agregan comentarios explicativos salvo que aporten una decisión no obvia.

## Verificación

Antes de dar una tarea por terminada:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
npm run lint
npm run typecheck
npm run test
npm run license:check
```

Los comandos que todavía no existan se habilitan cuando la fase correspondiente
los agregue.

## Commits

Conventional Commits, en inglés o español pero consistente, con alcance cuando
aplique (`feat(core):`, `fix(host):`, `docs(compliance):`). Un commit no mezcla
cambios de compliance con refactors de producto.
