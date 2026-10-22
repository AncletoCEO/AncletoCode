# Plan de release

## Versión 0.0.1 — primer release de prueba

`0.0.1` es un release **internal/beta para testers**, no el release público
estable. Incluye Linux y Windows para que el desarrollo de Windows pueda instalar,
probar y reportar.

> **Gate legal 14.12 abierto.** Antes de un release público estable hay que
> completar la revisión de AGPL-3.0 + la condición de atribución de PandaCode.
> Este release se publica bajo aprobación manual del entorno `release` de GitHub.

## Matriz de artefactos

| Plataforma | Artefactos 0.0.1 | Estado |
|---|---|---|
| Linux x86_64 glibc | `.deb`, `.tar.gz`, AUR generado | Se construye en Ubuntu 22.04 (glibc base) |
| Arch/EndeavourOS | `ancleto-code-bin`, `ancleto-cmd-bin`, `ancleto-code-cli-bin` | PKGBUILDs generados con checksums del release |
| Windows x64 | NSIS, MSI y ZIP portable | Para verificación del dev de Windows |
| npm (Linux/Win/macOS) | `@ancleto/code`, `@ancleto/cmd` | Binarios por plataforma + launcher |
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

1. **npm:** matriz nativa para CLI, host y AncletoCMD; publica paquetes por
   plataforma y los meta paquetes.
2. **desktop-linux:** Ubuntu 22.04; `npm run tauri -- build --bundles deb`;
   tarballs de escritorio, AncletoCMD y CLI/host; artefactos a GitHub Actions.
3. **desktop-windows:** `windows-latest`; NSIS + MSI y ZIP portable.
4. **release:** descarga artefactos, genera `SHA256SUMS`, genera PKGBUILDs AUR
   con checksums reales y publica el GitHub Release con las notas de
   `docs/release-notes/<versión>.md`.
5. **Aprobación:** los jobs de publicación usan el entorno `release`; configurar
   revisores obligatorios en GitHub para el gate legal.

## Verificación en Arch

- Construir desde fuente en `archlinux:latest` con `base-devel`, `rust`,
  `nodejs`, `npm`, `webkit2gtk-4.1`, `gtk3`, `libsoup3`, `librsvg`.
- Smoke de la app y AncletoCMD en **X11 y Wayland nativo** (no solo XWayland).
- CLI/host estáticos musl: `acode doctor`, host, sesión, detach/attach.
- AUR: `makepkg --printsrcinfo`, `namcap`, instalación y desinstalación limpias.

## Gates

- `cargo fmt/clippy/test`, `npm run lint/typecheck/test`, licencias, `cargo deny`,
  perf y smoke npm en verde.
- **14.12** revisión legal completada (bloqueante para release público).
- **1.4 / 12.11 / 14.8** verificados en Windows.
- **13.14** smoke Windows/macOS; Linux ya cubierto por tests de vim/less/htop/git.
- Checksums y PKGBUILDs generados; probar instalación/desinstalación en cada
  formato.
- Documentar rollback: conservar tag y artefactos anteriores.

## Estado de tareas

Pendientes al preparar 0.0.1: 1.4, 12.11, 13.14, 14.8, 14.10, 14.12, 15.3.
Las tareas de Windows y la revisión legal son las que condicionan el paso de
beta interna a release estable.
