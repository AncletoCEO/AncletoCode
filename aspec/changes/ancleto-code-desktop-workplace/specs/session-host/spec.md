## ADDED Requirements

### Requirement: Host local único dueño de las sesiones

El sistema SHALL ejecutar un host local por usuario que sea el único dueño de los PTY y del estado de sesiones, y SHALL exponerlo a la app de escritorio, a AncletoCMD y a la CLI mediante IPC local.

#### Scenario: Varios clientes sobre el mismo host

- **WHEN** la app, AncletoCMD y la CLI se conectan al host
- **THEN** todas reciben el mismo estado de sesiones y proyectos
- **AND** ninguna crea PTY fuera del host

#### Scenario: Host no disponible

- **WHEN** un cliente no encuentra el host
- **THEN** lo arranca automáticamente con un lock que evita carreras
- **AND** si el arranque falla, muestra un error accionable

### Requirement: Seguridad del IPC local

El host SHALL aceptar conexiones sólo del usuario actual mediante named pipe o Unix domain socket con permisos restringidos, y SHALL exigir un token en el handshake.

#### Scenario: Token inválido

- **WHEN** un cliente presenta un token ausente o incorrecto
- **THEN** el host rechaza la conexión sin ejecutar ninguna operación
- **AND** registra el intento sin exponer el token

#### Scenario: Sin escucha de red

- **WHEN** el host está en ejecución
- **THEN** no abre puertos TCP ni acepta conexiones remotas
- **AND** el socket o pipe sólo es accesible por el usuario propietario

### Requirement: Ciclo de vida y autostart

El host SHALL arrancar bajo demanda, apagarse por inactividad cuando no haya sesiones activas y cerrar ordenadamente sus procesos hijos al apagarse.

#### Scenario: Apagado por inactividad

- **WHEN** no quedan clientes conectados ni sesiones activas durante el período configurado
- **THEN** el host se apaga
- **AND** no deja procesos PTY huérfanos

#### Scenario: Apagado con sesiones activas

- **WHEN** el usuario detiene el host explícitamente con sesiones activas
- **THEN** el sistema pide confirmación
- **AND** al confirmar termina los procesos hijos y marca las sesiones como `stopped`

### Requirement: Detach y attach sin matar sesiones

El sistema SHALL permitir que un cliente se desconecte de una sesión sin terminar el proceso y que otro cliente se adjunte después.

#### Scenario: Cerrar AncletoCMD con terminales abiertas

- **WHEN** el usuario cierra la ventana de AncletoCMD con sesiones activas
- **THEN** las sesiones pasan a `detached` y sus procesos siguen vivos
- **AND** al reabrir AncletoCMD se restauran los paneles y el buffer reciente

#### Scenario: Attach desde otra versión

- **WHEN** la app de escritorio se adjunta a una sesión creada en AncletoCMD
- **THEN** recibe el snapshot del ring buffer
- **AND** puede escribir en la sesión y ver la salida en tiempo real

### Requirement: Reconciliación tras caída del host

El sistema SHALL detectar sesiones cuyo proceso dueño ya no existe y reconciliarlas al arrancar.

#### Scenario: Host terminado de forma abrupta

- **WHEN** el host muere sin cerrar sus sesiones
- **THEN** en el siguiente arranque las sesiones que estaban activas se marcan como `crashed`
- **AND** la app y AncletoCMD ofrecen recrearlas con el mismo `cwd` y driver

### Requirement: Protocolo versionado

El host SHALL declarar una versión de protocolo y SHALL rechazar clientes incompatibles con un error explícito.

#### Scenario: Cliente y host de versiones distintas

- **WHEN** un cliente presenta una versión de protocolo no soportada
- **THEN** el host rechaza el handshake
- **AND** el cliente indica que debe actualizar y `acode doctor` reporta la incompatibilidad
