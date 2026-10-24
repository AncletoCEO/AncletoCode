## ADDED Requirements

### Requirement: Provider con capacidades declaradas

El sistema SHALL exponer las capacidades del proveedor Git detectado y deshabilitar
las acciones no disponibles.

#### Scenario: GitHub sin autenticar

- **WHEN** `gh` no está autenticado
- **THEN** el provider se marca como no disponible
- **AND** la UI explica cómo iniciar sesión sin pedir un token a la aplicación

#### Scenario: Capacidades parciales

- **WHEN** el login no tiene permisos de merge
- **THEN** la acción de merge se deshabilita con una explicación
- **AND** el resto de las operaciones sigue disponible

### Requirement: PR enlazado a una tarea

El sistema SHALL crear un PR desde una tarea y mostrar su estado en el thread.

#### Scenario: Crear PR

- **WHEN** el usuario crea un PR desde una tarea con rama y base
- **THEN** el PR se crea en GitHub con título, cuerpo y reviewers indicados
- **AND** la tarea guarda la URL del PR

#### Scenario: Checks del PR

- **WHEN** la tarea tiene un PR asociado
- **THEN** el thread muestra los checks y su estado actualizado

### Requirement: Review y merge seguros

El sistema SHALL permitir revisar y mergear un PR solo con confirmación explícita y
sin conflictos ni checks fallidos.

#### Scenario: Merge bloqueado

- **WHEN** el PR tiene checks fallidos o conflictos
- **THEN** el merge no se ejecuta
- **AND** se muestra el motivo

#### Scenario: Merge confirmado

- **WHEN** el usuario confirma el merge y los checks requeridos pasan
- **THEN** el PR se mergea con la estrategia elegida
- **AND** la tarea refleja el resultado
