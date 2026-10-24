# Tasks: Release desktop-only y empaquetado multiplataforma

## Fase 1 — Verificación previa

- [ ] 1.1 Verificar el arranque de la app de escritorio en Windows con `npm run tauri dev`; registrar dependencias y resultado.
- [ ] 1.2 Revisar `tauri.conf.json` y dejar solo los targets de escritorio por plataforma.

## Fase 2 — Empaquetado

- [ ] 2.1 Empaquetar **solo la app de escritorio** en Linux: AppImage + `.deb`; verificar instalación, About y arranque.
- [ ] 2.2 Empaquetar **solo la app de escritorio** en Windows: NSIS, MSI y ZIP portable; verificar instalación, About y arranque.
- [ ] 2.3 Generar `SHA256SUMS` para todos los artefactos y publicar la fuente correspondiente.
- [ ] 2.4 Generar AppImage x86_64 para Arch/EndeavourOS, probar ejecución con FUSE y fallback `--appimage-extract-and-run`.
- [ ] 2.5 Documentar requisitos de instalación por plataforma y limitaciones conocidas (firma Windows, FUSE).

## Fase 3 — Workflow y documentación

- [ ] 3.1 Recortar el workflow de release a la app de escritorio: quitar CMD, CLI, paquetes npm y AUR del camino de publicación.
- [ ] 3.2 Hacer que el workflow falle si intenta publicar un artefacto fuera de la matriz desktop-only.
- [ ] 3.3 Actualizar README y notas de release con la matriz desktop-only (Linux AppImage/DEB, Windows NSIS/MSI/portable; macOS diferido).

## Fase 4 — Smoke end-to-end

- [ ] 4.1 Smoke manual **solo de la app de escritorio**: dos proyectos, sesiones `claude`/`opencode`/`agy`/`cmd`, detach/attach, conversación agente-a-agente, aborto, `ancleto-writer build` + vista humana y Git/worktrees.
- [ ] 4.2 Registrar el resultado del smoke en las notas de release y cerrar el checklist de instalación.
