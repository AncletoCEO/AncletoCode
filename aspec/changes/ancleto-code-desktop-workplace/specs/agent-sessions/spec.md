## ADDED Requirements

### Requirement: Sesiones de agente mediante PTY

El sistema SHALL ejecutar cada sesión de agente dentro de un pseudo-terminal con el directorio de trabajo del proyecto seleccionado.

#### Scenario: Crear una sesión

- **WHEN** el usuario crea una sesión eligiendo un driver disponible y un proyecto
- **THEN** el sistema lanza el proceso del CLI en un PTY con el `cwd` del proyecto
- **AND** la sesión queda asociada al proyecto con estado `starting`

#### Scenario: Cierre ordenado de la aplicación

- **WHEN** el usuario cierra la aplicación con sesiones activas
- **THEN** el sistema termina todos los procesos PTY hijos
- **AND** no quedan procesos huérfanos

#### Scenario: Rehidratar una terminal

- **WHEN** el usuario vuelve a una tab de sesión ya iniciada
- **THEN** la terminal muestra el buffer reciente de salida de esa sesión
- **AND** la sesión sigue aceptando entrada

### Requirement: Tabs por agente

El sistema SHALL mostrar dentro de cada proyecto una tab por sesión de agente, con su estado en vivo.

#### Scenario: Estado de la sesión

- **WHEN** la sesión cambia de estado entre `ready`, `busy`, `waiting_input`, `stopped` y `crashed`
- **THEN** la tab de esa sesión actualiza su indicador en consecuencia

#### Scenario: Cerrar una tab

- **WHEN** el usuario cierra la tab de una sesión activa
- **THEN** el sistema solicita confirmación de que se terminará el proceso
- **AND** al confirmar, la sesión pasa a `stopped` y la tab se elimina

### Requirement: Entrada y salida interactiva

El sistema SHALL transmitir la entrada del usuario al proceso de la sesión y emitir su salida en tiempo real.

#### Scenario: Escribir en la sesión

- **WHEN** el usuario escribe texto o comandos en la terminal de una sesión activa
- **THEN** los bytes llegan al proceso del CLI

#### Scenario: Redimensionar la terminal

- **WHEN** cambia el tamaño del área de terminal
- **THEN** el sistema envía el nuevo tamaño al PTY de la sesión
