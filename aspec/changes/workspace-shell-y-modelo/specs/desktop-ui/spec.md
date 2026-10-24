## ADDED Requirements

### Requirement: Shell de tres zonas

La aplicación SHALL organizar el trabajo en sidebar de proyectos/tareas, workspace
central con threads/tabs e inspector contextual, con composer inferior.

#### Scenario: Navegación por tarea

- **WHEN** el usuario selecciona una tarea del sidebar
- **THEN** el workspace muestra los threads y sesiones asociados a esa tarea
- **AND** el inspector muestra el contexto de la tarea

#### Scenario: Paneles redimensionables

- **WHEN** el usuario redimensiona o cierra un panel
- **THEN** el layout se guarda por workspace
- **AND** se restaura al reiniciar la aplicación

#### Scenario: Layout inválido

- **WHEN** el layout guardado no se puede leer
- **THEN** la aplicación usa el layout por defecto
- **AND** no se pierde el estado de las sesiones

### Requirement: Command palette y quick switcher

La aplicación SHALL ofrecer un command palette que busque proyectos, tareas, ramas,
worktrees, PRs y acciones.

#### Scenario: Búsqueda por acción

- **WHEN** el usuario abre el palette y escribe el nombre de una acción
- **THEN** la acción aparece en los resultados y puede ejecutarse con Enter
- **AND** los atajos de teclado se respetan

#### Scenario: Quick switcher de proyectos

- **WHEN** el usuario abre el quick switcher
- **THEN** puede cambiar de proyecto o tarea sin usar el mouse
