## ADDED Requirements

### Requirement: Compatibilidad con CLIs de código

El sistema SHALL soportar como mínimo los CLIs `claude code`, `opencode`, `agy` (Antigravity CLI) y `cmd` (Command Code) mediante un contrato de driver común, y SHALL tratar a Antigravity como integración de IDE y no como un CLI adicional.

#### Scenario: Detección de un CLI instalado

- **WHEN** el sistema verifica los drivers al iniciar
- **THEN** marca como `available` cada CLI cuyo binario se resuelve en el `PATH` o en una ruta configurada
- **AND** muestra su versión cuando el CLI la expone

#### Scenario: Detección de un CLI ausente

- **WHEN** el binario de un driver no se encuentra
- **THEN** el driver se marca como `missing`
- **AND** no se ofrece como opción para crear una sesión

#### Scenario: Antigravity como IDE

- **WHEN** el usuario elige la integración de Antigravity
- **THEN** el sistema abre el proyecto en el IDE Antigravity instalado
- **AND** no crea una sesión PTY, porque `agy` es el único CLI de esa familia

#### Scenario: Command Code

- **WHEN** el sistema verifica el driver de Command Code
- **THEN** resuelve el binario `cmd` del paquete npm `command-code`
- **AND** lo marca como `available` con su versión, o `missing` si no está instalado

### Requirement: Modos de ejecución del agente

El sistema SHALL declarar por driver las capacidades de ejecución interactiva y one-shot, y SHALL elegir el modo según la operación.

#### Scenario: Sesión interactiva

- **WHEN** el usuario abre una tab de agente
- **THEN** el sistema lanza el CLI en modo interactivo dentro de un PTY

#### Scenario: Ejecución one-shot con salida estructurada

- **WHEN** una operación requiere un resultado parseable y el driver declara `oneShotPrompt`
- **THEN** el sistema ejecuta el CLI en modo one-shot pidiendo salida estructurada cuando esté disponible
- **AND** no abre una tab de terminal para esa ejecución

### Requirement: Degradación a modo crudo

El sistema SHALL mantener la sesión operativa cuando la interpretación de la salida del CLI falla.

#### Scenario: Salida no reconocida

- **WHEN** el parser de un driver no reconoce la salida del proceso
- **THEN** la sesión continúa mostrando la salida cruda en la terminal
- **AND** el estado de la sesión se mantiene consistente sin cambiar a `crashed`

### Requirement: Extensibilidad de drivers

El sistema SHALL permitir agregar un driver nuevo sin modificar el frontend.

#### Scenario: Registro de un driver adicional

- **WHEN** se registra un nuevo driver en el registro del núcleo
- **THEN** aparece automáticamente en las opciones de creación de sesión
- **AND** reutiliza el motor PTY y las tabs existentes
