# Design: Release desktop-only y empaquetado multiplataforma

## Approach

Se usa el bundler de Tauri como única fuente de artefactos. El workflow se reduce a
una matriz Linux/Windows con `tauri build`, y cada job publica solo lo que la app
necesita. La verificación se separa en tres capas: build reproducible, instalación
en máquina limpia y smoke funcional.

```
push tag ──► build linux (AppImage + deb) ──► checksums ──┐
         └─► build windows (nsis/msi/zip) ──► checksums ──┴─► release draft
                                                              │
                                        smoke manual (Linux/Windows) ◄─┘
```

## Architecture

- **Targets**: `src-tauri/tauri.conf.json` declara `bundle.targets` por plataforma;
  no se agregan targets de CMD/CLI.
- **Workflow**: `.github/workflows/release.yml` se recorta a dos jobs desktop. Se
  elimina cualquier paso de `packages/acode`, `packages/acmd`, npm publish o AUR.
  Se conserva el paso de `SHA256SUMS` y la publicación de la fuente correspondiente.
- **AppImage**: se valida en Arch/EndeavourOS con FUSE y con
  `--appimage-extract-and-run`. La documentación indica qué hacer si FUSE no está
  disponible en el sistema.
- **Windows**: verificación con `npm run tauri dev` antes del empaquetado, y
  checklist post-instalación: About con atribución de fork, arranque, detección de
  drivers y creación de una sesión.
- **Notas de release**: plantilla con matriz (Linux AppImage/DEB, Windows
  NSIS/MSI/ZIP), limitaciones conocidas y resultado del smoke.
- **Smoke E2E**: checklist manual con dos proyectos, sesiones de los cuatro drivers,
  detach/attach, conversación con `maxTurns` y aborto, writer y Git/worktrees.

## Validation

```bash
npm run build
cargo build --manifest-path src-tauri/Cargo.toml
npm run tauri build -- --bundles appimage,deb   # Linux
npm run tauri build -- --bundles nsis,msi       # Windows
```

- Instalar el artefacto en una máquina limpia y ejecutar el checklist de arranque.
- Verificar que `SHA256SUMS` cubre todos los artefactos publicados.
- Confirmar que el workflow no referencia CMD/CLI/npm/AUR.
- Ejecutar el smoke end-to-end y adjuntar el resultado a las notas de release.
