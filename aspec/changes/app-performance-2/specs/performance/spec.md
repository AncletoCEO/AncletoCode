# Spec: Performance (Delta)

## MODIFIED Requirements

### Requirement: Comandos IPC no bloqueantes

The system SHALL ejecutar fuera del hilo principal todo comando que inicie procesos,
use la red o realice IO de disco sobre un conjunto de archivos o bases de datos SQLite.

#### Scenario: Detección de drivers no congela la interfaz

- **WHEN** el frontend solicita la lista de drivers disponibles
- **THEN** la operación se ejecuta de forma asíncrona fuera del hilo principal
- **AND** ningún frame supera 100 ms durante la invocación

#### Scenario: Consultas de persistencia de tareas y proyectos

- **WHEN** se invocan comandos de listado o actualización de tareas o proyectos
- **THEN** las operaciones de SQLite corren en un hilo de trabajo dedicado
- **AND** el hilo principal de la UI no sufre bloqueo

### Requirement: Presupuesto de arranque y bundle

The system SHALL cargar de forma diferida el código y procesar las dependencias externas
sin retrasar la primera pintura interactiva del shell.

#### Scenario: Shell interactivo en arranque

- **WHEN** la aplicación se inicia con o sin sesiones previas
- **THEN** el shell se vuelve interactivo en menos de 800 ms
- **AND** la detección de drivers o reconciliación de sesiones no bloquea el primer frame

## ADDED Requirements

### Requirement: Detección de drivers en segundo plano con caché

The system SHALL cachear los resultados de detección de drivers en memoria con un TTL
y limitar el tiempo máximo de ejecución por binario externo.

#### Scenario: Acceso con caché vigente

- **WHEN** se solicita la lista de drivers y la caché tiene menos de 60 segundos
- **THEN** la respuesta se sirve en menos de 20 ms sin invocar subprocesos

#### Scenario: Timeout por binario colgado o lento

- **WHEN** un ejecutable de driver excede 3 segundos al responder `--version`
- **THEN** el subproceso se cancela
- **AND** el driver se marca en estado `missing` o `error` sin retrasar al resto

### Requirement: Persistencia transaccional de sesiones y turnos

The system SHALL asegurar la persistencia atómica de turnos y propagar errores de base
de datos para evitar desincronizaciones con los eventos de la interfaz.

#### Scenario: Escritura concurrente bajo contención

- **WHEN** dos procesos o servicios intentan escribir en la base de datos simultáneamente
- **THEN** la conexión espera hasta 5000 ms antes de fallar con error de bloqueo
- **AND** no se pierden registros de turnos de conversación

#### Scenario: Fallo de persistencia aborta emisión

- **WHEN** la inserción de un turno en SQLite falla
- **THEN** el orquestador propaga el error
- **AND** no se emite el evento de turno exitoso a la interfaz

### Requirement: Stream incremental de conversación

The system SHALL actualizar la transcripción de conversación de manera reactiva mediante
eventos IPC y proveer reconciliación incremental por cursor.

#### Scenario: Emisión reactiva sin polling

- **WHEN** una conversación se encuentra en ejecución activa
- **THEN** la interfaz incorpora los nuevos turnos mediante los eventos `conversation://turn`
- **AND** no se ejecutan consultas periódicas de polling de todo el historial

#### Scenario: Reconciliación incremental tras reconexión

- **WHEN** la ventana recupera el foco o se detecta inactividad tras 10 segundos en ejecución
- **THEN** se consulta `conversation_turns_since` enviando el último índice conocido
- **AND** la transferencia de datos no excede 32 KB en conversaciones de hasta 800 turnos
