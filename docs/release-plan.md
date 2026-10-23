# Plan de release

## Versión 0.0.3 — desktop-only

`0.0.3` es un release **interno/beta** para validar la app de escritorio en Linux
y Windows. El release anterior fue retirado; este es el primero del nuevo
producto desktop-only.

## Matriz de artefactos

| Plataforma | Artefactos | Estado |
|---|---|---|
| Linux x86_64 | `.deb`, `.AppImage` | Build en Ubuntu 22.04 |
| Arch/EndeavourOS | AppImage | Probar FUSE y `--appimage-extract-and-run` |
| Windows x64 | NSIS, MSI, ZIP portable | Build en CI; falta smoke manual |
| Fuente | `AncletoCode-0.0.3-source.tar.gz` | Generada desde el tag |
| Integridad | `SHA256SUMS` | Generado en CI |

Fuera del release: AncletoCMD, CLI `acode`, npm y AUR.

## Pipeline

1. **preflight:** versión, licencias, formato, tests y clippy.
2. **desktop-linux:** Ubuntu 22.04; prepara Node + `@ancleto/spec`; construye
   `.deb` y AppImage.
3. **desktop-windows:** Windows; prepara runtime; construye NSIS/MSI y ZIP
   portable.
4. **release:** fuente, checksums y GitHub Release con
   `docs/release-notes/0.0.3.md`.
5. **Aprobación:** entorno `release` de GitHub.

## Verificación

- `cargo fmt/clippy/test`, `npm run lint/typecheck/test/build` y licencias.
- `.deb` en Debian/Ubuntu y AppImage en Arch/EndeavourOS.
- Instalador NSIS/MSI y ZIP portable en Windows.
- Ancleto embebido: `ancleto --version`, actualización y fallback offline.
- Chequeo de updates al iniciar.

## Limitaciones de esta etapa

- Windows aún sin validación manual de maintainers.
- GitLab/`glab` fuera de alcance por ahora; review/merge de PRs pendiente.
- AUR, npm, CLI y AncletoCMD fuera de alcance.

La revisión técnica de licencias está completada; antes de un release estable se
recomienda revisión legal externa.
