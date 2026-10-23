# Plan de release


## Próximo release 0.2 — desktop-only (vigente)

El producto de release es **sólo AncletoCode de escritorio**. Quedan fuera npm,
AncletoCMD, la CLI `acode`, el host como artefacto separado y AUR.

| Plataforma | Artefactos 0.2 |
|---|---|
| Linux x86_64 | **AppImage**, `.deb` |
| Arch/EndeavourOS | AppImage (probar FUSE y `--appimage-extract-and-run`) |
| Windows x64 | NSIS, MSI, ZIP portable |
| macOS | Diferido |

La UI objetivo es un híbrido PandaCode + Orca con Git/worktrees, branch
management y GitHub/GitLab. Las tareas están en la Fase 16 de `tasks.md`.

## Versión 0.0.1 — primer release de prueba

`0.0.1` es un release **internal/beta para testers**, no el release público
estable. Incluye Linux y Windows para que el desarrollo de Windows pueda instalar,
probar y reportar.

> **Gate legal 14.12:** revisión técnica de compatibilidad completada en
> `docs/compliance/license-review-0.0.1.md`; para un release estable se recomienda
> revisión legal externa. Este release se publica bajo aprobación manual del
> entorno `release` de GitHub.

## Matriz de artefactos

| Plataforma | Artefactos 0.0.1 | Estado |
|---|---|---|
| Linux x86_64 glibc | `.deb`, `.tar.gz`, AUR generado | Se construye en Ubuntu 22.04 (glibc base) |
| Arch/EndeavourOS | `ancleto-code-bin`, `ancleto-cmd-bin`, `ancleto-code-cli-bin` | PKGBUILDs generados con checksums del release |
| Windows x64 | NSIS, MSI y ZIP portable | Para verificación del dev de Windows |
| npm (Linux/Win/macOS) | `@ancleto/code`, `@ancleto/cmd` | **Pendiente** para un release posterior; workflow `npm-publish.yml` separado |
| Linux ARM64 | npm `linux-arm64-gnu` | Incluido en npm; sin instalador de escritorio en 0.0.1 |
| macOS | npm; DMG no incluido | Fuera de 0.0.1 |

## Reglas de construcción

- **No publicar binarios compilados en Arch.** El entorno de desarrollo actual es
  EndeavourOS con glibc 2.44; un `.deb` construido ahí no corre en Ubuntu/Debian.
  Los paquetes Linux se construyen en `ubuntu-22.04` y se prueban en Debian 12,
  Ubuntu 24.04 y Arch.
- **Arch usa tarball/AUR**, no el `.deb`.
- El escritorio se instala como `ancleto-code-desktop` para no colisionar con la
  CLI `ancleto-code`/`acode`.
- La app requiere `webkit2gtk-4.1`, `gtk3`, `libsoup3` y `librsvg`; AncletoCMD
  requiere `libxkbcommon`, `vulkan-icd-loader`, `fontconfig`, `freetype2`,
  `wayland` y `libx11`.
- La fuente IBM Plex Mono va empaquetada; los emoji dependen de la fuente del
  sistema (`noto-fonts-emoji` en Arch).

## Pipeline

1. **desktop-linux:** Ubuntu 22.04; `npm run tauri -- build --bundles deb,appimage`;
   artefactos `.deb` y `.AppImage` a GitHub Actions.
2. **desktop-windows:** `windows-latest`; NSIS + MSI y ZIP portable con la app y
   el host.
3. **release:** descarga artefactos, agrega la fuente correspondiente, genera
   `SHA256SUMS` y publica el GitHub Release con las notas de
   `docs/release-notes/<versión>.md`.
4. **Aprobación:** el job de publicación usa el entorno `release`; configurar
   revisores obligatorios en GitHub.
5. **Fuera del pipeline:** AncletoCMD, CLI `acode`, npm y AUR. AUR queda como
   posibilidad futura, no como requisito de release.

## Verificación en Arch

- Ejecutar la AppImage en Arch/EndeavourOS con FUSE y con
  `--appimage-extract-and-run`.
- Smoke de la app en **X11 y Wayland nativo** (no solo XWayland).
- Verificar `.deb` en Debian/Ubuntu y AppImage en Arch; no publicar binarios
  compilados localmente con glibc más nueva que la base de CI.
- AUR no se publica en esta etapa.

## Gates

- `cargo fmt/clippy/test`, `npm run lint/typecheck/test`, licencias, `cargo deny`,
  perf y smoke npm en verde.
- **14.12** revisión legal completada (bloqueante para release público).
- **1.4** verificado en Windows.
- Instaladores NSIS/MSI y ZIP portable probados en Windows.
- `.deb` y AppImage probados en Linux/Arch; checksums generados.
- Documentar rollback: conservar tag y artefactos anteriores.

## Estado de tareas

Pendientes de la nueva etapa desktop-only: 1.4, 14.10, 14.13–14.15 y 15.3.
La revisión técnica de licencias ya está completada; para un release estable se
recomienda revisión legal externa.
