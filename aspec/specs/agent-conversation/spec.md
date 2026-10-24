# Spec: agent-conversation

## Purpose

Conversación entre dos agentes de código sobre el mismo proyecto, con turnos, límites, pausa y aborto.

## Requirements

### Requirement: Conversación entre dos agentes

El sistema SHALL permitir vincular dos sesiones de agente en una conversación con turnos alternados.

#### Scenario: Iniciar una conversación

- **WHEN** el usuario elige dos sesiones y define un objetivo
- **THEN** el sistema envía el objetivo al primer agente
- **AND** la conversación pasa a estado `running` con el contador de turnos en cero

#### Scenario: Turno del agente A hacia el agente B

- **WHEN** el primer agente termina su turno
- **THEN** el sistema extrae el mensaje producido y lo inyecta en la sesión del segundo agente
- **AND** registra el turno con origen, destino, modo, texto y timestamp

#### Scenario: Agentes de proyectos distintos

- **WHEN** el usuario vincula sesiones pertenecientes a proyectos distintos
- **THEN** el sistema permite la conversación manteniendo cada sesión en su propio `cwd`

### Requirement: Selección de estrategia por turno

El sistema SHALL usar ejecución one-shot estructurada cuando el driver la soporte y SHALL recurrir a la inyección en el PTY interactivo como fallback.

#### Scenario: Driver con one-shot estructurado

- **WHEN** el driver del agente declara `oneShotPrompt` y `structuredOutput`
- **THEN** cada turno se ejecuta como un proceso one-shot con salida parseable
- **AND** se usa la continuidad de sesión del CLI cuando está disponible

#### Scenario: Driver sólo interactivo

- **WHEN** el driver no declara `oneShotPrompt`
- **THEN** el sistema inyecta el mensaje en la sesión PTY
- **AND** espera la señal de fin de turno antes de continuar

### Requirement: Límites y control de la conversación

El sistema SHALL aplicar límites de seguridad y permitir el control manual de la conversación.

#### Scenario: Límite de turnos alcanzado

- **WHEN** la conversación alcanza el máximo de turnos configurado
- **THEN** el sistema detiene la conversación con estado `completed`
- **AND** deja de inyectar mensajes en ambas sesiones

#### Scenario: Aborto manual

- **WHEN** el usuario aborta la conversación
- **THEN** el sistema detiene la conversación con estado `aborted`
- **AND** las sesiones de agente permanecen activas

#### Scenario: Repetición detectada

- **WHEN** dos turnos consecutivos del mismo origen son equivalentes
- **THEN** el sistema detiene la conversación para evitar un bucle
- **AND** reporta el motivo de la detención

#### Scenario: Timeout de turno

- **WHEN** un turno supera el tiempo máximo configurado
- **THEN** el sistema marca la conversación como `failed`
- **AND** notifica al usuario sin matar las sesiones

### Requirement: Observabilidad de la conversación

El sistema SHALL mostrar la conversación en curso como una secuencia legible de turnos.

#### Scenario: Seguir la conversación

- **WHEN** la conversación está `running`
- **THEN** el panel muestra cada turno a medida que ocurre, indicando qué agente habló
- **AND** permite pausar y reanudar

#### Scenario: Historial persistido

- **WHEN** el usuario reabre una conversación terminada
- **THEN** el sistema muestra sus turnos persistidos con su estado final
