## ADDED Requirements

### Requirement: Update firmado y verificado

El sistema SHALL instalar una actualización solo si la firma del artefacto es válida
y corresponde a la clave pública embebida.

#### Scenario: Firma válida

- **WHEN** hay una versión nueva con manifiesto y firma válidos
- **THEN** la app descarga el artefacto y verifica la firma antes de instalar
- **AND** muestra el progreso de la descarga

#### Scenario: Firma inválida

- **WHEN** la firma no coincide o el manifiesto está malformado
- **THEN** la app no instala nada
- **AND** conserva la versión actual operativa
- **AND** muestra un error accionable

### Requirement: Canales de actualización

El sistema SHALL permitir elegir entre stable y prerelease, con stable como default.

#### Scenario: Cambio de canal

- **WHEN** el usuario selecciona un canal en Ajustes
- **THEN** el canal se persiste y se usa para el próximo chequeo
- **AND** la UI indica el canal activo

### Requirement: Instalación segura con sesiones activas

El sistema SHALL pedir confirmación antes de instalar una actualización que cierre
sesiones activas.

#### Scenario: Sesiones activas

- **WHEN** el usuario inicia una instalación y hay sesiones de agente activas
- **THEN** la app advierte que se van a cerrar
- **AND** no avanza sin confirmación explícita
- **AND** ofrece detach como alternativa

#### Scenario: Descarga cancelada

- **WHEN** el usuario cancela la descarga
- **THEN** la instalación actual no se modifica
- **AND** el estado vuelve a "update disponible"

### Requirement: Rollback ante fallo de instalación

El sistema SHALL conservar la versión anterior si la instalación falla.

#### Scenario: Fallo al instalar

- **WHEN** la instalación falla después de descargar
- **THEN** la app vuelve a la versión anterior
- **AND** el error queda visible en Ajustes
