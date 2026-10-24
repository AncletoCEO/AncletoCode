## ADDED Requirements

### Requirement: Virtualización de listas y diffs

El sistema SHALL virtualizar listas y diffs largos para mantener el render dentro del
presupuesto.

#### Scenario: Lista grande

- **WHEN** una lista de archivos o tareas supera 1.000 entradas
- **THEN** solo se renderiza la ventana visible
- **AND** ninguna interacción produce una tarea larga mayor a 100 ms

#### Scenario: Diff extenso

- **WHEN** se abre un diff de más de 2.000 líneas
- **THEN** el diff se renderiza por ventanas
- **AND** el scroll y el foco se mantienen estables

#### Scenario: Transcript largo

- **WHEN** una conversación supera 500 turnos
- **THEN** el transcript se virtualiza
- **AND** se conserva el anclaje al mensaje visible al insertar contenido arriba
