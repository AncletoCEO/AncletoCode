## ADDED Requirements

### Requirement: Tareas con thread y worktree

El sistema SHALL modelar cada tarea con su thread, rama, worktree, agente, PR/MR y
resultado, persistiendo la relación en SQLite.

#### Scenario: Crear tarea con worktree

- **WHEN** el usuario crea una tarea y elige crear un worktree
- **THEN** se registra la tarea con su rama y path de worktree
- **AND** la relación sobrevive al reinicio de la aplicación

#### Scenario: Tarea sin worktree

- **WHEN** el usuario crea una tarea sin worktree
- **THEN** la tarea trabaja sobre el árbol principal del proyecto
- **AND** no se crea ninguna rama

### Requirement: Gestión segura de worktrees

El sistema SHALL crear, abrir y eliminar worktrees sin perder cambios del usuario.

#### Scenario: Colisión de worktree

- **WHEN** el path o la rama del worktree ya están en uso
- **THEN** el sistema rechaza la operación con un mensaje accionable
- **AND** no modifica el repositorio

#### Scenario: Eliminar con cambios sin commit

- **WHEN** el usuario intenta eliminar un worktree con cambios sin commit
- **THEN** el sistema exige confirmación explícita
- **AND** si el usuario cancela, el worktree permanece intacto

#### Scenario: Abrir el proyecto en su worktree

- **WHEN** el usuario abre una tarea con worktree
- **THEN** las sesiones nuevas usan el path del worktree como directorio de trabajo
