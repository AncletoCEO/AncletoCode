# Spec: ancleto-writer

## Purpose

Integración con `@ancleto/writer`, su panel de conocimiento y vista humana.

## Requirements

### Requirement: Integración con `@ancleto/writer`

El sistema SHALL consumir `@ancleto/writer` como motor de conocimiento estático del proyecto y exponer sus artefactos sin reimplementar el análisis.

#### Scenario: Detección del CLI

- **WHEN** el sistema inicia
- **THEN** resuelve `ancleto-writer` en el proyecto, en el entorno global o en un sidecar
- **AND** muestra versión y estado `available`/`missing`/`incompatible`

#### Scenario: Construir el conocimiento del proyecto

- **WHEN** el usuario ejecuta un build desde el panel
- **THEN** el sistema corre `ancleto-writer build`
- **AND** reporta el resultado y los códigos de salida de la CLI

#### Scenario: Frescura del grafo

- **WHEN** el usuario consulta el estado del conocimiento
- **THEN** el sistema usa `ancleto-writer check --json` como autoridad de frescura y compatibilidad
- **AND** no recalcula el fingerprint por su cuenta

### Requirement: Panel de conocimiento

El sistema SHALL mostrar los hechos publicados en `.ancleto/writer/`.

#### Scenario: Leer el manifest

- **WHEN** existe un `manifest.json` con `schemaVersion` compatible
- **THEN** el sistema muestra versión de writer, commit, estadísticas y hashes

#### Scenario: Explorar hechos

- **WHEN** el usuario abre el panel de conocimiento
- **THEN** el sistema muestra ciclos, capas, comunidades, god nodes, hotspots, blast radius, código sin uso y cobertura de documentación
- **AND** permite consultar el impacto de un archivo con `impact`

#### Scenario: Esquema incompatible

- **WHEN** el `schemaVersion` publicado no es compatible
- **THEN** el sistema marca el conocimiento como `incompatible`
- **AND** no intenta interpretar los artefactos

### Requirement: Vista humana

El sistema SHALL ofrecer la proyección humana generada por writer.

#### Scenario: Generar la vista humana

- **WHEN** el usuario ejecuta `human` desde el panel
- **THEN** el sistema genera `human/OVERVIEW.md`, `human/ARCHITECTURE.md`, `human/data.json` y los HTML
- **AND** permite verlos embebidos o en el navegador

### Requirement: Language pack de lenguajes de programación

El sistema SHALL tratar `@ancleto/writer-languages` como un pack opcional de lenguajes de programación, no como idioma natural.

#### Scenario: Pack instalado

- **WHEN** el pack está instalado y habilitado
- **THEN** el sistema muestra los lenguajes soportados además de JS/TS
- **AND** permite desactivarlo con `languagePack.enabled = false` en `.ancleto-writer.json`

#### Scenario: Pack ausente

- **WHEN** el pack no está instalado o falla
- **THEN** el sistema informa que el core degrada a JS/TS
- **AND** muestra las extensiones no soportadas reportadas en `facts/unsupported.json`
- **AND** sugiere el comando de instalación sin ejecutarlo

### Requirement: Independencia del idioma natural

El sistema SHALL NOT presentar opciones de idioma natural en el panel de writer.

#### Scenario: Configurar idioma

- **WHEN** el usuario busca cambiar el idioma de los artefactos
- **THEN** el sistema lo dirige a la configuración de `@ancleto/spec`
- **AND** no escribe claves de idioma en `.ancleto-writer.json`
