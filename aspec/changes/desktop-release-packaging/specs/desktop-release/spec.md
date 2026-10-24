## ADDED Requirements

### Requirement: Empaquetado desktop-only

El sistema SHALL publicar únicamente artefactos de la app de escritorio para Linux y
Windows, sin binarios de AncletoCMD ni de la CLI.

#### Scenario: Artefactos de Linux

- **WHEN** se ejecuta el build de release en Linux
- **THEN** se generan AppImage y `.deb` de la app
- **AND** no se genera ningún artefacto de AncletoCMD o de la CLI

#### Scenario: Artefactos de Windows

- **WHEN** se ejecuta el build de release en Windows
- **THEN** se generan NSIS, MSI y un ZIP portable de la app
- **AND** el ZIP portable no requiere instalación

#### Scenario: Checksums

- **WHEN** se publican los artefactos de una versión
- **THEN** `SHA256SUMS` cubre todos los archivos publicados
- **AND** la fuente correspondiente queda disponible junto al release

### Requirement: AppImage portable en Arch

El sistema SHALL proveer una AppImage x86_64 ejecutable con FUSE y sin FUSE.

#### Scenario: Ejecución con FUSE

- **WHEN** el usuario ejecuta la AppImage en Arch/EndeavourOS con FUSE disponible
- **THEN** la app inicia y muestra la ventana principal

#### Scenario: Fallback sin FUSE

- **WHEN** FUSE no está disponible
- **THEN** la app inicia con `--appimage-extract-and-run`
- **AND** la documentación de instalación explica el fallback

### Requirement: Verificación de arranque en Windows

El sistema SHALL arrancar en Windows con las dependencias documentadas.

#### Scenario: Arranque en desarrollo

- **WHEN** se ejecuta `npm run tauri dev` en Windows
- **THEN** la ventana inicia sin errores de dependencias
- **AND** la pantalla About muestra la atribución de fork

#### Scenario: Instalación limpia

- **WHEN** se instala el artefacto en una máquina Windows sin la app
- **THEN** la app arranca, detecta drivers y permite crear una sesión

### Requirement: Smoke end-to-end de escritorio

El sistema SHALL completar el flujo principal de la app antes de publicar una
versión.

#### Scenario: Flujo completo

- **WHEN** se ejecuta el smoke con dos proyectos
- **THEN** se abren sesiones de `claude`, `opencode`, `agy` y `cmd`
- **AND** detach/attach conserva la sesión
- **AND** la conversación agente-a-agente respeta `maxTurns` y aborta
- **AND** writer genera la vista humana y Git/worktrees funcionan

#### Scenario: Release bloqueado por smoke fallido

- **WHEN** el smoke end-to-end falla
- **THEN** la versión no se publica
- **AND** el fallo queda registrado en las notas de release
