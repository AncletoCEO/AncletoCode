## ADDED Requirements

### Requirement: Emulador de terminal nativo

El sistema SHALL proveer AncletoCMD como una aplicación de terminal con ventana propia y emulación VT, sin depender de un webview ni de otra terminal.

#### Scenario: Ejecutar un programa interactivo

- **WHEN** el usuario ejecuta `vim`, `htop` o `tmux` en un pane de AncletoCMD
- **THEN** el programa se renderiza correctamente con alt screen, posicionamiento de cursor y colores
- **AND** la entrada del teclado y el redimensionado llegan al proceso

#### Scenario: Texto internacional

- **WHEN** la salida contiene caracteres anchos, emoji o marcas combinadas
- **THEN** AncletoCMD los renderiza sin desalinear la grilla
- **AND** el reflow al redimensionar conserva el contenido

### Requirement: Gestor de proyectos, tabs y paneles

AncletoCMD SHALL organizar terminales y sesiones de agente en proyectos, tabs y splits persistentes.

#### Scenario: Abrir dos proyectos

- **WHEN** el usuario abre dos proyectos en AncletoCMD
- **THEN** cada proyecto conserva sus propias tabs, splits y sesiones
- **AND** puede alternar entre ellos desde el sidebar o la paleta de comandos

#### Scenario: Restaurar layout

- **WHEN** el usuario cierra y vuelve a abrir AncletoCMD
- **THEN** se restauran tabs, splits, foco y panes del proyecto
- **AND** los panes cuyas sesiones siguen vivas se reenganchan al host

#### Scenario: Crear una sesión de agente

- **WHEN** el usuario crea una sesión de agente desde AncletoCMD
- **THEN** elige un driver disponible y la sesión se abre en un pane
- **AND** la sesión es visible y adjuntable desde la app de escritorio y la CLI

### Requirement: Sesiones persistentes

AncletoCMD SHALL hacer detach de sus sesiones al cerrar la ventana en lugar de terminarlas.

#### Scenario: Cierre de ventana

- **WHEN** el usuario cierra la ventana con terminales y agentes activos
- **THEN** las sesiones pasan a `detached` y siguen ejecutándose en el host
- **AND** la opción "cerrar y matar sesiones" requiere confirmación explícita

### Requirement: Integración de shell

AncletoCMD SHALL integrarse con los shells soportados para marcar prompts, comandos y resultados.

#### Scenario: Shell con integración

- **WHEN** el shell emite las marcas OSC 133 al ejecutar comandos
- **THEN** AncletoCMD permite navegar al comando anterior o siguiente
- **AND** muestra el resultado y permite copiar la salida de un comando completo

#### Scenario: Shell sin integración

- **WHEN** el shell no soporta las marcas o el usuario desactiva la integración
- **THEN** AncletoCMD sigue funcionando como terminal estándar
- **AND** no inventa bloques ni altera la salida del programa

### Requirement: Rendimiento de terminal

AncletoCMD SHALL mantener una latencia de entrada y un consumo de recursos comparables a terminales nativos rápidos.

#### Scenario: Entrada bajo carga

- **WHEN** un pane recibe salida continua mientras el usuario escribe en otro pane
- **THEN** la entrada se refleja con una latencia p99 menor a 30 ms
- **AND** el uso de CPU en reposo se mantiene por debajo del 1 %

#### Scenario: Sin GPU compatible

- **WHEN** no hay un adaptador gráfico utilizable o el contexto GPU falla
- **THEN** AncletoCMD recurre a render por software
- **AND** informa la degradación sin cerrar la aplicación

### Requirement: Temas y accesibilidad

AncletoCMD SHALL usar los manifiestos de tema de AncletoCode limitados a paleta y tipografía.

#### Scenario: Cambiar de tema

- **WHEN** el usuario cambia el tema activo
- **THEN** AncletoCMD actualiza colores y contraste en todas las tabs
- **AND** no aplica transparencias ni wallpapers

#### Scenario: Tema de bajo contraste

- **WHEN** un tema no cumple el contraste mínimo
- **THEN** AncletoCMD advierte y ofrece un preset accesible
- **AND** permite continuar con el tema elegido

### Requirement: Seguridad de la terminal

AncletoCMD SHALL tratar la salida de los programas como no confiable.

#### Scenario: Escritura de clipboard por OSC 52

- **WHEN** un programa solicita escribir el clipboard mediante OSC 52
- **THEN** AncletoCMD pide confirmación al usuario
- **AND** rechaza la operación si no hay confirmación

#### Scenario: Hyperlink OSC 8

- **WHEN** la salida contiene un hyperlink OSC 8
- **THEN** AncletoCMD no lo abre automáticamente
- **AND** requiere una acción explícita del usuario para abrirlo

### Requirement: Distribución y plataformas

El sistema SHALL distribuir AncletoCMD mediante npm y SHALL soportar Windows, Linux y macOS en x64 y arm64.

#### Scenario: Instalación con npm

- **WHEN** el usuario ejecuta `npm install -g @ancleto/cmd`
- **THEN** queda disponible el comando `acmd`
- **AND** el launcher resuelve el binario de su plataforma y verifica su integridad

#### Scenario: Windows con shells nativos

- **WHEN** el usuario abre AncletoCMD en Windows
- **THEN** puede iniciar terminales con `cmd`, PowerShell y WSL
- **AND** el cierre de la ventana no termina las sesiones activas

#### Scenario: Linux y macOS

- **WHEN** el usuario abre AncletoCMD en Linux o macOS
- **THEN** puede iniciar terminales con bash, zsh o fish
- **AND** funcionan HiDPI, clipboard y atajos documentados
