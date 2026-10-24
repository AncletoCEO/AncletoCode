# Spec: appearance

## Purpose

Sistema de apariencia: temas, transparencias, fondos y accesibilidad.

## Requirements

### Requirement: Apariencia CLI

El sistema SHALL presentar una interfaz con estética de terminal, tipografía monoespaciada y densidad compacta en todas sus superficies.

#### Scenario: Identidad visual consistente

- **WHEN** el usuario abre cualquier proyecto
- **THEN** sidebar, tabs, terminal y paneles usan la tipografía monoespaciada y los glifos de prompt del tema activo
- **AND** los estados se comunican con color y glifos, sin depender de iconografía decorativa

### Requirement: Transparencias configurables

El sistema SHALL permitir transparencia de ventana y de paneles con controles independientes y fallback cuando el sistema operativo no soporta efectos nativos.

#### Scenario: Efecto nativo disponible

- **WHEN** la plataforma soporta Mica, Acrylic, Blur o vibrancy
- **THEN** el sistema aplica el efecto nativo elegido
- **AND** permite ajustar la opacidad de ventana, de paneles y del terminal por separado

#### Scenario: Efecto no soportado

- **WHEN** la plataforma no soporta el efecto solicitado
- **THEN** el sistema degrada a un fondo translúcido CSS
- **AND** informa el fallback sin bloquear la aplicación

#### Scenario: Modo opaco

- **WHEN** el usuario activa el modo opaco
- **THEN** el sistema desactiva todas las transparencias y el blur
- **AND** persiste la elección

### Requirement: Colores configurables

El sistema SHALL exponer un sistema de tokens de color editable mediante temas.

#### Scenario: Cambiar de tema

- **WHEN** el usuario elige un tema preset
- **THEN** la UI y la paleta ANSI del terminal se actualizan en vivo
- **AND** la elección se persiste

#### Scenario: Tema personalizado

- **WHEN** el usuario crea, duplica o edita un tema
- **THEN** el sistema permite ajustar tokens de fondo, texto, acentos y estados
- **AND** exporta o importa el tema como `theme.json`

### Requirement: Fondos configurables

El sistema SHALL permitir configurar el fondo de la aplicación y del terminal.

#### Scenario: Fondo con imagen

- **WHEN** el usuario elige una imagen de fondo
- **THEN** el sistema permite cover, contain o tile, con controles de desenfoque y oscurecimiento
- **AND** copia la imagen al directorio de datos de la aplicación sin escribirla en el repositorio

#### Scenario: Fondo por proyecto

- **WHEN** un proyecto define un fondo propio
- **THEN** ese proyecto lo usa por encima de la configuración global
- **AND** el resto de los proyectos conserva el suyo

### Requirement: Precedencia y persistencia de apariencia

El sistema SHALL resolver la apariencia con precedencia global → workspace → proyecto y persistirla fuera de los archivos del ecosistema Ancleto.

#### Scenario: Override de proyecto

- **WHEN** el proyecto define tema, transparencia o fondo
- **THEN** esos valores prevalecen sobre los globales
- **AND** al quitarlos se vuelve a la configuración heredada

#### Scenario: Sin contaminar el repositorio

- **WHEN** el usuario cambia la apariencia
- **THEN** el sistema no escribe en `.ancletorc` ni en `.ancleto-writer.json`
- **AND** guarda la configuración en la base local de la aplicación

### Requirement: Legibilidad y accesibilidad

El sistema SHALL garantizar legibilidad mínima sobre cualquier tema o fondo.

#### Scenario: Contraste insuficiente

- **WHEN** un tema o fondo produce contraste menor a WCAG AA para texto
- **THEN** el sistema advierte y ofrece corregir el token o aplicar un scrim
- **AND** no impide guardar el tema si el usuario lo confirma

#### Scenario: Preferencias de accesibilidad

- **WHEN** el usuario activa reducción de movimiento o modo opaco
- **THEN** el sistema desactiva animaciones y transparencias decorativas
