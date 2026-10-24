## ADDED Requirements

### Requirement: Activación atómica del runtime

El sistema SHALL activar una versión del runtime Ancleto solo después de verificar su
integridad y sin sobrescribir la versión activa.

#### Scenario: Update válido

- **WHEN** hay una versión nueva con checksum válido
- **THEN** se descarga y verifica fuera del directorio activo
- **AND** se activa de forma atómica
- **AND** Ajustes muestra la versión nueva

#### Scenario: Integridad inválida

- **WHEN** el checksum o el tarball no son válidos
- **THEN** el update se rechaza
- **AND** la versión activa sigue operativa
- **AND** se muestra un error accionable

### Requirement: Rollback ante fallo

El sistema SHALL restaurar la versión anterior si la activación o el primer uso del
runtime falla.

#### Scenario: Fallo de activación

- **WHEN** la activación del update falla
- **THEN** el runtime vuelve a la versión anterior
- **AND** los proyectos siguen abriéndose

#### Scenario: Red caída

- **WHEN** no hay red para consultar o descargar un update
- **THEN** el estado se reporta como no disponible
- **AND** la versión incluida sigue funcionando

### Requirement: Estado visible del runtime

El sistema SHALL mostrar en Ajustes la versión activa, la incluida y el error si lo
hay.

#### Scenario: Runtime degradado

- **WHEN** el runtime activo falla
- **THEN** Ajustes muestra el error y la versión incluida como fallback
- **AND** el usuario puede volver a la versión incluida
