# Spec: ancleto-spec

## Purpose

Integración con la CLI `@ancleto/spec` como fuente de verdad del SDD.

## Requirements

### Requirement: Integración con `@ancleto/spec`

El sistema SHALL consumir `@ancleto/spec` (CLI `ancleto`, alias `aspec`) como orquestador SDD del proyecto, sin reimplementar su lógica.

#### Scenario: Detección del CLI

- **WHEN** el sistema inicia
- **THEN** resuelve `ancleto` en el proyecto, en el entorno global o en un sidecar
- **AND** verifica Node ≥ 24
- **AND** muestra versión y estado `available`/`missing`/`error`

#### Scenario: Inicialización del proyecto

- **WHEN** el usuario inicializa el proyecto desde la app
- **THEN** el sistema ejecuta `ancleto init` con host, tier e idioma elegidos
- **AND** refleja el estado resultante de `.ancletorc`

#### Scenario: Explorar changes activos

- **WHEN** el usuario abre el panel SDD
- **THEN** el sistema lista los changes de `aspec/changes`
- **AND** muestra al menos nombre y artefactos presentes

#### Scenario: Memoria del proyecto

- **WHEN** el usuario consulta la memoria
- **THEN** el sistema la lee exclusivamente a través de la CLI (`memory list --json`, `memory context`)
- **AND** no abre ni modifica `.ancleto/memory.db` directamente

### Requirement: Estado de discovery

El sistema SHALL exponer el estado del technical seed y su impacto.

#### Scenario: Seed desactualizado

- **WHEN** `discovery --check` reporta `STALE` o `PARTIAL`
- **THEN** el sistema muestra el estado y el `impact` (`none`/`minor`/`material`)
- **AND** sólo ofrece regenerar, sin ejecutarlo automáticamente

#### Scenario: Regeneración aprobada

- **WHEN** el usuario aprueba regenerar el seed
- **THEN** el sistema ejecuta `discovery`
- **AND** actualiza el estado mostrado

### Requirement: Configuración de host, tier e idioma

El sistema SHALL permitir configurar el host de instalación, el tier de costo y el idioma de los artefactos.

#### Scenario: Cambiar host

- **WHEN** el usuario cambia el host entre `opencode`, `claude`, `vscode`, `antigravity`, `cursor`, `roo`, `copilot` o `commandcode`
- **THEN** el sistema aplica la instalación correspondiente sin borrar los layouts previos

#### Scenario: Cambiar tier

- **WHEN** el usuario selecciona `normal`, `minimo` o `gratis`
- **THEN** el sistema lo aplica en la configuración del proyecto y lo muestra en el panel

#### Scenario: Idioma de artefactos

- **WHEN** el usuario selecciona `auto`, `es`, `en` o `pt`
- **THEN** el sistema lo persiste en `.ancletorc` → `language`
- **AND** no altera los keywords canónicos ni los nombres de archivos
