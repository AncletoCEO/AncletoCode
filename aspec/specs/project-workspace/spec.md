# Spec: project-workspace

## Purpose

Gestión de proyectos, tabs y estado del ecosistema por proyecto.

## Requirements

### Requirement: Gestión de proyectos

El sistema SHALL permitir registrar, listar y quitar proyectos asociados a una ruta local, persistiendo el registro entre reinicios de la aplicación.

#### Scenario: Alta de un proyecto

- **WHEN** el usuario agrega una ruta existente como proyecto
- **THEN** el proyecto aparece en la lista con nombre derivado de la carpeta y su ruta absoluta
- **AND** el proyecto se conserva tras reiniciar la aplicación

#### Scenario: Alta de una ruta inexistente

- **WHEN** el usuario intenta agregar una ruta que no existe
- **THEN** el sistema rechaza la operación y muestra un error explicativo
- **AND** no se crea ningún proyecto

### Requirement: Vista de proyecto

El sistema SHALL mostrar una vista por proyecto que incluya su árbol de archivos y el estado de sus sesiones activas.

#### Scenario: Abrir un proyecto

- **WHEN** el usuario selecciona un proyecto desde el sidebar
- **THEN** se muestra su vista con la raíz del proyecto y el árbol de archivos
- **AND** se marcan como activas las sesiones de agente que sigan corriendo

### Requirement: Tabs por proyecto

El sistema SHALL organizar el área de trabajo con tabs de primer nivel, una por cada proyecto abierto.

#### Scenario: Cambiar de proyecto

- **WHEN** el usuario selecciona la tab de otro proyecto
- **THEN** se muestra la vista de ese proyecto con sus propias tabs de agente
- **AND** las sesiones del proyecto anterior siguen ejecutándose en segundo plano

#### Scenario: Indicador de actividad

- **WHEN** un proyecto tiene al menos una sesión de agente activa
- **THEN** su tab muestra un indicador de actividad
- **AND** el indicador desaparece cuando no queda ninguna sesión activa

### Requirement: Estado del ecosistema por proyecto

El sistema SHALL mostrar por proyecto el estado de `@ancleto/spec` y de `@ancleto/writer`.

#### Scenario: Proyecto con ecosistema completo

- **WHEN** ambas CLIs están disponibles y el proyecto está inicializado
- **THEN** el proyecto muestra el host, el tier, el idioma y la frescura del conocimiento

#### Scenario: Proyecto sin inicializar

- **WHEN** el proyecto no tiene `.ancletorc`
- **THEN** el sistema lo indica y ofrece inicializarlo con `@ancleto/spec`
- **AND** no ejecuta la inicialización sin confirmación
