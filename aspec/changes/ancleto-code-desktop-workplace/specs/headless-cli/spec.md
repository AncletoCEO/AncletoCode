## ADDED Requirements

### Requirement: Distribución npm multiplataforma

El sistema SHALL distribuir la CLI mediante un paquete npm instalable globalmente que resuelva un binario nativo para la plataforma del usuario.

#### Scenario: Instalación global en Windows

- **WHEN** el usuario ejecuta `npm install -g @ancleto/code` en Windows
- **THEN** queda disponible el comando `acode` en `cmd` y PowerShell
- **AND** `acode --version` informa la versión del paquete instalado

#### Scenario: Instalación global en Linux

- **WHEN** el usuario ejecuta `npm install -g @ancleto/code` en Linux x64 o arm64
- **THEN** el launcher resuelve el binario correspondiente a la arquitectura y a la libc
- **AND** `acode --version` funciona sin requerir Node después del arranque del binario

#### Scenario: Plataforma no soportada

- **WHEN** la plataforma o arquitectura no tiene binario publicado
- **THEN** el launcher falla con un mensaje que indica la plataforma detectada y las soportadas
- **AND** no ejecuta un binario de otra plataforma

### Requirement: Integridad del binario distribuido

El launcher SHALL verificar el checksum del binario antes de ejecutarlo.

#### Scenario: Checksum inválido

- **WHEN** el hash del binario no coincide con el publicado
- **THEN** el launcher no ejecuta el binario
- **AND** devuelve el código de salida de estado incompatible con un mensaje accionable

### Requirement: Comandos headless

La CLI SHALL ofrecer comandos no interactivos para proyectos, agentes, sesiones, conversación, providers y diagnóstico.

#### Scenario: Ejecutar un agente en modo one-shot

- **WHEN** el usuario ejecuta `acode run "<prompt>"` con un driver y proyecto
- **THEN** la CLI ejecuta el agente sin abrir interfaz gráfica
- **AND** emite la respuesta y devuelve `0` si el turno termina correctamente

#### Scenario: Listar sesiones compartidas

- **WHEN** el usuario ejecuta `acode session list`
- **THEN** la CLI muestra las sesiones del host con proyecto, driver y estado
- **AND** incluye las sesiones creadas desde la app o AncletoCMD

#### Scenario: Diagnóstico

- **WHEN** el usuario ejecuta `acode doctor`
- **THEN** la CLI informa el estado de Node, los drivers, los providers, el host y la persistencia
- **AND** sugiere el comando de instalación para cada dependencia ausente

### Requirement: Salida para automatización

La CLI SHALL emitir salida legible por máquina cuando se solicita JSON y SHALL usar códigos de salida estables.

#### Scenario: Salida JSON

- **WHEN** el usuario ejecuta un comando con `--json`
- **THEN** la CLI escribe en stdout un objeto con `schemaVersion`, `ok` y `data` o `error`
- **AND** no intercala texto decorativo ni secuencias ANSI en stdout

#### Scenario: Error de dependencia

- **WHEN** falta Node, un driver o un provider requerido
- **THEN** la CLI devuelve el código de salida de dependencia ausente
- **AND** el error JSON identifica la dependencia y el comando de instalación sugerido

#### Scenario: Ejecución en CI

- **WHEN** la salida no es un TTY
- **THEN** la CLI no emite secuencias ANSI ni solicita entrada interactiva
- **AND** los comandos que requieren confirmación fallan si no se pasa el flag explícito

### Requirement: Operación sin host

La CLI SHALL poder ejecutar operaciones one-shot sin host local cuando se solicita explícitamente.

#### Scenario: Host deshabilitado

- **WHEN** el usuario ejecuta un comando one-shot con `--no-host`
- **THEN** la CLI ejecuta el proceso en modo in-process sin crear sesión persistente
- **AND** no afecta el estado de las sesiones administradas por el host
