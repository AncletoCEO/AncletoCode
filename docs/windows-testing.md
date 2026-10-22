# Verificación en Windows — 0.0.1

Este release incluye Windows para que el dev de Windows pueda instalar, probar y
reportar antes de un release estable.

## Artefactos esperados

- `AncletoCode_0.0.1_x64-setup.exe` — instalador NSIS.
- `AncletoCode_0.0.1_x64_en-US.msi` — instalador MSI (si el job lo produce).
- `AncletoCode-0.0.1-windows-x64-portable.zip` — ejecutable, host local y licencias.
- Paquetes npm: `npm install -g @ancleto/code@0.0.1` y
  `npm install -g @ancleto/cmd@0.0.1`.

El instalador NSIS/MSI instala la app; para sesiones persistentes por host
instalá `@ancleto/code` (aporta `ancleto-code-host.exe`). El ZIP portable ya
incluye el host junto al ejecutable.
- `SHA256SUMS` y `PKGBUILD` de Arch adjuntos al release.

## Requisitos

- Windows 10/11 x64 con WebView2 (los instaladores lo bootstrapean si falta).
- Node.js 24 para `ancleto`/`ancleto-writer` si se prueban los paneles spec/writer.
- Al menos un CLI de agente instalado y autenticado: `claude`, `opencode`, `agy`
  o `cmd` (Command Code).
- Opcional: WSL y PowerShell 7 para las pruebas de AncletoCMD.

## Checklist

### Instalación
- [ ] Instalar con NSIS como usuario actual; desinstalar y reinstalar.
- [ ] Repetir con MSI si el artefacto existe.
- [ ] Probar el ZIP portable sin instalar.
- [ ] Verificar que `LICENSE`, `THIRD_PARTY_NOTICES.md` y la atribución de fork
      están presentes en la instalación.

### App de escritorio
- [ ] Abrir, agregar un proyecto con espacios/acentos y ver el árbol.
- [ ] Buscar archivos y abrir el proyecto en Antigravity.
- [ ] Ver `acode agents`/panel de drivers: available/missing/error.
- [ ] Crear sesiones `claude`, `opencode`, `agy` y `cmd`; tabs y estados.
- [ ] Escribir, redimensionar, matar y reiniciar una sesión.
- [ ] Cerrar la app: las sesiones quedan `detached`; reabrir y reenganchar.
- [ ] Conversación entre dos agentes: objetivo, límites, pausa, aborto.
- [ ] Paneles spec y writer; `writer build` y vista humana.
- [ ] Apariencia: temas, transparencia Mica/Acrylic, fondos, contraste, modo opaco.
- [ ] About: "This is a fork of PandaCode" con enlace.

### AncletoCMD
- [ ] Abrir con `acmd`; probar `cmd`, PowerShell y WSL.
- [ ] Tabs, splits, foco, resize, zoom, command palette.
- [ ] Sesiones de agente desde el sidebar; attach/detach y kill con confirmación.
- [ ] OSC 52 pide confirmación; hyperlinks OSC 8 no se abren solos.
- [ ] Cerrar la ventana: detach; reabrir y restaurar layout/sesiones.
- [ ] Probar en X11/Wayland si aplica (Linux) y en Windows en modo ventana.

### CLI en `cmd.exe` y PowerShell
- [ ] `acode --version` y `acode doctor`.
- [ ] `acode project add/list/remove` con rutas con espacios y Unicode.
- [ ] `acode --json agents` y `acode --json session list`.
- [ ] `acode run "hola" --driver opencode`.
- [ ] `acode host start|status|stop`.
- [ ] `acode ui` abre la app instalada.
- [ ] Ctrl+C no deja procesos hijos; cerrar la terminal no mata sesiones del host.

### Cierre limpio
- [ ] Cerrar app, AncletoCMD y host; no quedan procesos PTY/one-shot colgados.
- [ ] Revisar el Administrador de tareas ante cualquier sospecha.

## Reporte

Abrir un issue con: versión, artefacto usado, Windows, CLI/driver, pasos,
resultado esperado vs obtenido y logs (`%APPDATA%\AncletoCode`,
`%LOCALAPPDATA%\AncletoCode`). Adjuntar captura si es visual.
