## ADDED Requirements

### Requirement: Rebase con manejo de conflictos

El sistema SHALL permitir rebasar la rama actual sobre una base elegida, con
confirmación previa y sin interacción de editor.

#### Scenario: Rebase sin conflictos

- **WHEN** el usuario confirma un rebase sobre una base actualizada
- **THEN** la rama local queda rebasada y el estado se refresca
- **AND** no se reescribe ninguna rama remota

#### Scenario: Rebase con conflictos

- **WHEN** el rebase encuentra conflictos
- **THEN** el sistema lista los archivos en conflicto
- **AND** ofrece continuar o abortar
- **AND** abortar deja la rama en su estado original

### Requirement: Panel de ramas

El sistema SHALL mostrar la rama actual con su estado respecto del remoto y el
worktree asociado.

#### Scenario: Cambio de rama

- **WHEN** el usuario cambia de rama desde el panel
- **THEN** el estado del árbol se valida antes de la operación
- **AND** si hay cambios incompatibles se exige confirmación explícita

#### Scenario: Rama con worktree

- **WHEN** una rama está asociada a un worktree
- **THEN** el panel lo indica y enlaza a la tarea correspondiente

### Requirement: Revisión de cambios con comentarios

El sistema SHALL permitir revisar el diff por archivo y por hunk, preparar hunks y
dejar comentarios por línea para el agente.

#### Scenario: Staging por hunk

- **WHEN** el usuario selecciona un hunk del diff
- **THEN** ese hunk se agrega al índice sin incluir los demás cambios del archivo

#### Scenario: Comentario por línea

- **WHEN** el usuario deja un comentario en una línea del diff
- **THEN** el comentario se envía al agente con archivo y línea
- **AND** si el diff cambia, el comentario se marca como obsoleto
