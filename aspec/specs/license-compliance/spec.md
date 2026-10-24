# Spec: license-compliance

## Purpose

Cumplimiento de licencias, procedencia y atribución del código incorporado.

## Requirements

### Requirement: Análisis de referencia documentado

El sistema SHALL mantener documentado el análisis de repositorios de referencia y la decisión tomada para cada feature comparable.

#### Scenario: Comparar con un repositorio de referencia

- **WHEN** se analiza un repositorio de referencia para una feature del change
- **THEN** el análisis registra repositorio, licencia detectada, features comparables y decisión (adoptar código, adaptar código, reimplementar o no adoptar)
- **AND** la decisión queda vinculada a la feature correspondiente del change
- **AND** si la decisión es adoptar o adaptar código, se indica el nivel A/B y el commit de origen

#### Scenario: Referencia sin reutilización de código

- **WHEN** la decisión es reutilizar sólo comportamiento o arquitectura
- **THEN** el registro indica explícitamente que no se copió código ni activos
- **AND** no se añaden avisos de copyright de ese repositorio como si fuera código incorporado

### Requirement: Procedencia de código y activos incorporados

El sistema SHALL registrar la procedencia de todo código, documento o activo de terceros que se incorpore al proyecto.

#### Scenario: Incorporar código de terceros

- **WHEN** se incorpora un archivo o porción sustancial de un repositorio de terceros
- **THEN** el sistema exige una entrada en el ledger de procedencia con repositorio, commit, ruta, licencia, autor y modificaciones
- **AND** el archivo derivado conserva su aviso de copyright y se marca con SPDX
- **AND** la licencia completa se incluye en `THIRD_PARTY_NOTICES.md`

#### Scenario: Incorporar un activo binario

- **WHEN** se incorpora una imagen, fuente, tema, icono o sonido de terceros
- **THEN** el ledger registra su origen y licencia
- **AND** el sistema verifica que la licencia permita redistribución en la app
- **AND** rechaza el activo si no lo permite

### Requirement: Reutilización vendorizada

El sistema SHALL incorporar el código reutilizado bajo `third_party/pandacode/`, preservando su ruta de origen y su trazabilidad.

#### Scenario: Vendorizar un módulo

- **WHEN** se incorpora un módulo de PandaCode
- **THEN** el archivo vive en `third_party/pandacode/<ruta-original>` con una cabecera SPDX que indica licencia, autor, commit y modificaciones
- **AND** figura en `third_party/pandacode/UPSTREAM.md` y en `docs/provenance.md`
- **AND** sus tests se portan y se ejecutan en CI

#### Scenario: Adaptar un módulo

- **WHEN** un archivo se modifica respecto del upstream
- **THEN** la cabecera y el ledger describen el cambio
- **AND** el diff contra el commit fijado coincide con lo declarado
- **AND** la adaptación no elimina avisos de copyright ni condiciones de licencia

#### Scenario: Sincronizar con upstream

- **WHEN** se actualiza el commit de PandaCode
- **THEN** se revisa el diff de los archivos fijados
- **AND** se actualizan commit, ledger, avisos y pruebas antes de mezclar el cambio

### Requirement: Aviso de licencia de terceros

El sistema SHALL distribuir los avisos de copyright y las licencias completas del código de terceros incorporado.

#### Scenario: Distribuir una build con código de terceros

- **WHEN** se genera un binario o paquete que contiene código de terceros
- **THEN** `THIRD_PARTY_NOTICES.md` incluye el texto completo de cada licencia y sus avisos de copyright
- **AND** el aviso es accesible desde el repositorio y desde la app

#### Scenario: Dependencia vendorizada

- **WHEN** se vendoriza una dependencia
- **THEN** se conserva el archivo de licencia original junto al código vendorizado
- **AND** la entrada en `THIRD_PARTY_NOTICES.md` identifica la versión o commit

### Requirement: Atribución de obra derivada

El sistema SHALL declarar que es un fork de PandaCode desde que incorpora código de PandaCode, sin condiciones de uso ni de publicación.

#### Scenario: Declaración obligatoria

- **WHEN** el repositorio contiene código de PandaCode
- **THEN** el README o la documentación principal declara exactamente "This is a fork of PandaCode" con enlace a `https://github.com/aicodeking/pandacode`
- **AND** la misma declaración aparece en la pantalla About de la app
- **AND** se mantiene en el sitio o store listing si existe

#### Scenario: Retirar la última porción de código derivado

- **WHEN** se elimina todo el código de PandaCode del repositorio
- **THEN** se retira la declaración de fork
- **AND** la referencia se conserva en el ledger como inspiración de diseño

### Requirement: Activos y marcas de terceros

El sistema SHALL NOT incorporar marcas, logos, wallpapers, temas, iconos o screenshots de terceros sin licencia que permita su redistribución.

#### Scenario: Usar identidad de proveedores

- **WHEN** la app muestra un proveedor o CLI de terceros
- **THEN** usa texto o un activo propio
- **AND** no sugiere afiliación ni endoso del proveedor

#### Scenario: Reutilizar arte de referencia

- **WHEN** se necesita un wallpaper, tema o ilustración
- **THEN** se genera o se obtiene de una fuente con licencia compatible
- **AND** se registra en el ledger y en `THIRD_PARTY_NOTICES.md` si corresponde

### Requirement: Auditoría de dependencias

El sistema SHALL auditar las licencias de las dependencias directas y transitivas antes de cada release.

#### Scenario: Dependencias Rust

- **WHEN** se ejecuta `cargo deny check licenses advisories`
- **THEN** toda licencia fuera de la allowlist explícita se reporta y se revisa
- **AND** el build falla ante una licencia ausente, desconocida o incompatible con AGPL-3.0

#### Scenario: Dependencias npm

- **WHEN** se auditan las dependencias npm de producción
- **THEN** se revisan también las transitivas
- **AND** falla ante licencias ausentes, `UNLICENSED` o incompatibles con AGPL-3.0
- **AND** el resultado se refleja en `THIRD_PARTY_NOTICES.md` o en un reporte referenciado

### Requirement: Compatibilidad con la licencia del proyecto

El sistema SHALL conservar la licencia AGPL-3.0 del proyecto y los términos de las licencias entrantes en la distribución combinada.

#### Scenario: Combinar AGPL-3.0 con código MIT

- **WHEN** se incorpora código MIT al proyecto AGPL-3.0
- **THEN** la distribución conserva el aviso MIT y la licencia AGPL-3.0
- **AND** no se elimina ni se reescribe la condición de atribución de la licencia entrante

#### Scenario: Condición adicional de atribución

- **WHEN** la licencia entrante impone una condición adicional de atribución
- **THEN** el proyecto la trata como término adicional compatible y la documenta
- **AND** la revisión legal se completa antes de incorporar el código

### Requirement: Verificación automatizada de cumplimiento

El sistema SHALL verificar automáticamente la presencia de los artefactos y avisos de cumplimiento antes de publicar.

#### Scenario: Verificación en CI

- **WHEN** se ejecuta la verificación de licencias en CI
- **THEN** falla si falta `LICENSE`, `THIRD_PARTY_NOTICES.md`, `third_party/pandacode/LICENSE`, `third_party/pandacode/UPSTREAM.md`, `docs/provenance.md`, la sección de licencias del README o la declaración "This is a fork of PandaCode" con su enlace
- **AND** falla si un archivo de `third_party/pandacode/` carece de cabecera SPDX o de entrada en el ledger
- **AND** informa el archivo o la cadena faltante

#### Scenario: Auditoría de activos

- **WHEN** se audita el árbol antes de una release
- **THEN** no se encuentran coincidencias de código o activos de PandaCode sin entrada de procedencia
- **AND** toda coincidencia intencional está justificada, licenciada y registrada
