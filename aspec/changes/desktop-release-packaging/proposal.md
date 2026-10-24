# Proposal: Release desktop-only y empaquetado multiplataforma

## Problem

La app de escritorio funciona en desarrollo, pero todavía no tiene un camino de
release cerrado: el workflow publica artefactos de AncletoCMD, CLI y paquetes npm
que esta etapa no entrega; faltan AppImage, `.deb`, NSIS/MSI/ZIP; no hay verificación
de arranque en Windows ni un smoke end-to-end reproducible. Sin eso no se puede
declarar una versión instalable de AncletoCode.

## Proposed change

- Verificar el arranque de la app en Windows con `npm run tauri dev` y registrar el
  checklist de dependencias (WebView2, Node del host, binarios de agentes).
- Empaquetar **solo la app de escritorio**: AppImage + `.deb` en Linux y
  NSIS/MSI/ZIP portable en Windows, con `SHA256SUMS`.
- Probar la AppImage x86_64 en Arch/EndeavourOS con FUSE y con
  `--appimage-extract-and-run`, y documentar la instalación.
- Recortar el workflow de release: fuera CMD, CLI, npm y AUR; se conserva la fuente
  correspondiente y los checksums.
- Actualizar README y notas de release con la matriz desktop-only.
- Ejecutar un smoke manual end-to-end de la app: dos proyectos, sesiones
  `claude`/`opencode`/`agy`/`cmd`, detach/attach, conversación agente-a-agente,
  aborto, writer (`build` y vista humana) y Git/worktrees.

## Scope

**In scope**

- Release Linux (AppImage, `.deb`) y Windows (NSIS, MSI, ZIP portable).
- CI de build y publicación desktop-only, con checksums y fuente correspondiente.
- Verificación de instalación, About y arranque en ambos sistemas.
- Smoke end-to-end de la app de escritorio.
- Documentación de instalación y matriz de release.

**Out of scope**

- Paquetes npm `@ancleto/code` y `@ancleto/cmd`, y su matriz de publicación.
- AUR y otros gestores de paquetes.
- AncletoCMD y la CLI como artefactos de release.
- macOS (diferido) y auto-update del binario.

## Risks

- **Firma y notarización en Windows**: sin firma el instalador muestra advertencias;
  se documenta como limitación conocida hasta tener certificado.
- **Dependencias del sistema en Linux**: WebKitGTK y FUSE varían por distro; el
  release declara requisitos y prueba el fallback de AppImage sin FUSE.
- **Artefactos mezclados**: el workflow recortado debe fallar si aparece un target de
  CMD/CLI/npm en el camino de publicación.
- **Smoke manual no reproducible**: se documenta un checklist con datos concretos
  (proyectos, sesiones, acciones) y resultados esperados.

## Success criteria

- `tauri build` genera AppImage y `.deb` en Linux, y NSIS/MSI/ZIP en Windows.
- La AppImage arranca con FUSE y con `--appimage-extract-and-run`.
- El workflow no publica artefactos fuera de la app de escritorio.
- El smoke end-to-end pasa completo y queda registrado en las notas de release.
