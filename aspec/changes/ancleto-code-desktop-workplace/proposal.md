# Proposal: AncletoCode — desktop workplace para agentes de código

## Problem

Hoy los agentes de código se usan desde terminales sueltas, una por ventana y sin contexto compartido. Esto obliga a saltar entre consolas para alternar proyectos, se pierde de vista qué agente corre en cada carpeta, y no existe forma nativa de que dos agentes colaboren sobre el mismo trabajo. Además, el ecosistema Ancleto sólo se consume por CLI (`ancleto`, `ancleto-writer`), sin una experiencia de escritorio que exponga su memoria, su grafo de conocimiento y su estado de frescura.

## Aclaraciones de alcance (corrección sobre el pedido inicial)

La investigación de los paquetes y de los CLIs deja cuatro precisiones que cambian el diseño:

1. **`agy` y `antigravity` no son dos CLIs distintos.** `agy` es el **Antigravity CLI** (binario Go, sucesor de Gemini CLI, con TUI y modo headless `agy -p`). `antigravity` es la **superficie IDE/desktop** de Antigravity, no un CLI de código. Por lo tanto hay **cuatro drivers PTY** (`claude`, `opencode`, `agy`, `cmd`) y una **integración de IDE** separada (`antigravity`), que no abre una sesión de agente sino que abre el proyecto y sincroniza los assets de skills del host.
2. **Command Code es `cmd`.** Su paquete npm es `command-code` (Node ≥ 22) y expone los bins `cmd`, `cmdc`, `commandcode` y `command-code`; tiene TUI interactiva y modo headless `cmd -p --output-format json` (NDJSON), con `--continue`/`--resume`. Se integra como cuarto driver.
3. **`@ancleto/writer` no es un motor de idiomas.** Es un **motor de conocimiento estático determinista y offline**: escanea el repo, construye un grafo de código y publica hechos en `.ancleto/writer/` (`manifest.json`, `graph.json`, `symbols.json`, `facts/*`, `agent/*`, y una vista humana `human/*`). No tiene LLM ni consume tokens. Su "paquete de idiomas" es `@ancleto/writer-languages`, un **pack opcional de lenguajes de programación** (Python, Go, Rust, C#, Java, C, C++, PHP, Ruby, Swift, Kotlin vía tree-sitter), no i18n.
4. **El idioma natural de los artefactos pertenece a `@ancleto/spec`**, no a writer: se configura en `.ancletorc` (`language: auto | es | en | pt`) y los keywords (`Requirement`, `Scenario`, `SHALL`, `WHEN`/`THEN`/`AND`) nunca se traducen.

## Proposed change

Construir **AncletoCode**: una aplicación de escritorio en **Tauri 2** que funciona como *workplace* de agentes de código.

- **Vista de proyecto**: cada proyecto abierto es un workspace con su ruta, su árbol de archivos, su estado SDD y su estado de conocimiento.
- **Tabs por proyecto y por agente**: pestañas de primer nivel por proyecto y, dentro de cada proyecto, una pestaña por sesión de agente.
- **Cuatro drivers PTY**: `claude code` (`claude`), `opencode` (`opencode`), `agy` (Antigravity CLI) y `cmd` (Command Code), con detección de binario, versión y capacidades declaradas (TUI interactiva y modo one-shot/headless cuando existe).
- **Integración con Antigravity IDE**: abrir el proyecto en la app Antigravity y materializar sus assets (`.agents/skills`, `.agents/agents`, `.agents/mcp_config.json`) vía `@ancleto/spec --agent antigravity`.
- **Interacción entre dos agentes**: conversación por turnos entre dos sesiones, usando modo one-shot estructurado cuando el driver lo soporta y PTY interactivo como fallback.
- **Proveedor `@ancleto/spec`**: inicialización/actualización del proyecto (host, tier, idioma), explorador de `aspec/`, estado de discovery con `impact`, memoria del proyecto, `specs check` y `doctor`.
- **Proveedor `@ancleto/writer`**: build/check/lint/impact/human/doctor, panel de conocimiento (god nodes, hotspots, ciclos, capas, blast radius) y vista humana embebida; detección y control opcional del pack `@ancleto/writer-languages`.
- **Apariencia CLI**: estética de terminal (tipografía monoespaciada, chrome tipo prompt, densidad compacta) con **transparencias reales de ventana y de paneles**, y **colores y fondos configurables** por tema y por proyecto.

## Scope

**In scope**

- Shell Tauri 2 con backend Rust y frontend React + TypeScript + Vite.
- Gestión de proyectos y persistencia (SQLite local).
- Motor de sesiones PTY con ciclo de vida completo y cierre ordenado.
- Drivers: `claude`, `opencode`, `agy` y `cmd` (Command Code), con perfil de arranque, detección y capacidades.
- Integración de IDE Antigravity como acción de proyecto (no como driver PTY).
- Tabs de proyecto y de agente con estado en vivo.
- Conversación agente-a-agente con turnos, límites, aborto y registro.
- Proveedor `@ancleto/spec`: init/install/update, lectura de `.ancletorc`, cambios, memoria, discovery, `specs check`, `doctor`, tiers y `--lang`.
- Proveedor `@ancleto/writer`: `init`, `build`, `check`, `lint`, `impact`, `human`, `doctor`; lectura de `manifest.json`/`graph.json`/`facts/*`; vista humana embebida; detección del language pack.
- Detección de Node ≥ 24 y de cada binario, con estado `available`/`missing`/`error` y comando de instalación sugerido.
- Sistema de apariencia: tokens de color, temas incluidos, fondos (sólido, degradado, imagen o wallpaper de terminal), transparencia/opacidad de ventana y paneles, y persistencia por proyecto.

**Out of scope**

- Ejecución remota, multiusuario o sincronización en la nube.
- Edición colaborativa en tiempo real entre humanos.
- Marketplace de agentes o plugins de terceros.
- Otros CLIs fuera de `claude`, `opencode`, `agy` y `cmd` (el contrato de driver queda abierto).
- Reimplementar la memoria, el discovery o el grafo dentro de la app: se consumen las CLIs de Ancleto como fuente de verdad.
- Instalar dependencias automáticamente (sólo se sugiere el comando npm).
- Autenticación de las CLIs (se delega en cada CLI ya autenticada en la máquina).

## Risks

- **`agy` es un binario Go con TUI y auto-updater propio y puede cambiar sin aviso**: contrato de driver desacoplado, modo headless (`agy -p --output-format json`) como camino preferido para turnos, y fallback a PTY crudo.
- **Parsear TUIs es frágil**: el PTY es la fuente de verdad; el parseo sólo alimenta estado/UX y nunca bloquea la operación.
- **Duplicar la lógica de frescura del grafo**: la app no recalcula fingerprints; usa `ancleto-writer check --json` y respeta `schemaVersion`.
- **Acoplamiento a formatos de artefactos de writer (schemaVersion 1)**: encapsular la lectura en un adapter y degradar a "conocimiento no disponible" si el esquema es incompatible.
- **Variabilidad del pack de lenguajes (peers nativos de tree-sitter, versión independiente)**: tratarlo como opcional, detectar su ausencia y no requerirlo para operar.
- **Bucles o costo descontrolado en la conversación agente-a-agente**: límite duro de turnos, timeout por turno, detección de repetición y aborto manual.
- **Dependencia de Node ≥ 24 para todo el ecosistema Ancleto**: chequeo explícito al iniciar y estado `missing` accionable.
- **Crecimiento de alcance del MVP**: cerrar contrato de driver, orquestador y adapters de providers antes de sumar integraciones.
- **Transparencia real y rendimiento en Windows (Mica/Acrylic/blur)**: usar efectos nativos con fallback a capas CSS translúcidas, permitir desactivar el blur por GPU y medir el costo de render del terminal.
- **Legibilidad con fondos e imágenes personalizados**: validar contraste de texto/estado contra el fondo parado, ofrecer presets accesibles y ajuste automático de luminosidad.
