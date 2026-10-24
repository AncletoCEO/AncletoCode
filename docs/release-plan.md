# Plan de release

## Versión 0.1.0 — release estable desktop-only

`0.1.0` es el primer release **estable** de AncletoCode para escritorio. Cierra el
trabajo de rendimiento y usage: la app no bloquea la interfaz durante tareas de red
o procesos, la pestaña Uso responde con caché y progreso, y el runtime Ancleto
embebido tiene actualización administrada con rollback.

El release se publica como **normal** (`prerelease: false`), con artefactos,
checksums y fuente correspondiente.

## Matriz de artefactos

| Plataforma | Artefactos | Estado |
|---|---|---|
| Linux x86_64 | `.deb`, `.AppImage` | Build en Ubuntu 22.04 |
| Arch/EndeavourOS | AppImage | Probar FUSE y `--appimage-extract-and-run` |
| Windows x64 | NSIS, MSI, ZIP portable | Build en CI; falta smoke manual |
| Fuente | `AncletoCode-0.1.0-source.tar.gz` | Generada desde el tag |
| Integridad | `SHA256SUMS` | Generado en CI |

Fuera del release: AncletoCMD, CLI `acode`, npm y AUR.

## Pipeline

1. **preflight:** versión, notas de release, licencias, formato, tests y clippy.
2. **desktop-linux:** Ubuntu 22.04; prepara Node + `@ancleto/spec`; construye
   `.deb` y AppImage.
3. **desktop-windows:** Windows; prepara runtime; construye NSIS/MSI y ZIP
   portable.
4. **release:** fuente, checksums y GitHub Release normal con
   `docs/release-notes/0.1.0.md`.
5. **Aprobación:** entorno `release` de GitHub.

## Verificación

- `cargo fmt/clippy/test`, `npm run lint/typecheck/test/build`, `check:versions`,
  `check:commands` y licencias.
- `.deb` en Debian/Ubuntu y AppImage en Arch/EndeavourOS con y sin FUSE.
- Instalador NSIS/MSI y ZIP portable en Windows.
- Uso: apertura con caché, primer proveedor y refresco acotado; ventana operable.
- Ancleto embebido: `ancleto --version`, actualización y fallback offline.
- Chequeo de updates al iniciar.

## Limitaciones de esta etapa

- Windows aún sin validación manual de maintainers.
- Actualización firmada de la app planificada; por ahora solo chequeo y aviso.
- Review/merge de PRs de GitHub en curso; listado de PRs e issues operativo.
- GitLab/`glab`, npm, AUR, CLI y AncletoCMD fuera de alcance.

La revisión técnica de licencias está completada; antes de un release estable se
recomienda revisión legal externa.
