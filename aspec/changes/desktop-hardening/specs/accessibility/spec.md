## ADDED Requirements

### Requirement: Operación completa por teclado

La aplicación SHALL permitir operar las vistas principales sin mouse, con foco
visible y orden predecible.

#### Scenario: Recorrido por teclado

- **WHEN** el usuario recorre la aplicación solo con Tab y atajos
- **THEN** alcanza sidebar, tabs, workspace, inspector y composer
- **AND** el foco siempre es visible y no queda atrapado

#### Scenario: Paleta y diálogos

- **WHEN** se abre el command palette o un diálogo
- **THEN** el foco entra en el elemento correcto
- **AND** Escape cierra y devuelve el foco al origen

### Requirement: Contraste y movimiento reducido

La aplicación SHALL cumplir contraste AA y respetar la preferencia de movimiento
reducido.

#### Scenario: Tema con contraste insuficiente

- **WHEN** un tema o fondo no cumple contraste AA para texto o estados
- **THEN** la aplicación lo advierte
- **AND** ofrece una alternativa legible

#### Scenario: Movimiento reducido

- **WHEN** el sistema declara `prefers-reduced-motion`
- **THEN** las animaciones no esenciales se desactivan
- **AND** la interfaz sigue siendo comprensible

### Requirement: Anuncio de estados

La aplicación SHALL anunciar cambios de estado relevantes a tecnologías de asistencia.

#### Scenario: Refresco de uso

- **WHEN** comienza o termina un refresco de datos
- **THEN** el estado se anuncia con `aria-live`
- **AND** no se interrumpe la lectura en curso
