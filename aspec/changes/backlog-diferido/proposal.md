# Proposal: Backlog diferido (CLI, AncletoCMD, npm y hosts Git alternativos)

## Problem

El alcance vigente es la app de escritorio. Varias tareas quedaron explícitamente
diferidas y hoy viven mezcladas con el trabajo activo, lo que ensucia el estado del
change principal y hace difícil saber qué está realmente en curso.

## Proposed change

Reunir en un único change de backlog las tareas diferidas, sin ejecutarlas:

- Verificación de argv/VT de la CLI headless en Windows.
- Tests de conformidad VT y smoke de AncletoCMD.
- Publicación de paquetes npm por plataforma y su matriz.
- Smoke de instalación global de `@ancleto/code` y `@ancleto/cmd`.
- GitLab, Bitbucket/Azure y remoto genérico.

## Scope

**In scope**

- Registrar las tareas diferidas con su condición de reapertura.
- Mantenerlas fuera del camino de release de la app de escritorio.

**Out of scope**

- Ejecutar cualquiera de estas tareas.
- Cambiar el alcance del producto sin una decisión explícita.

## Risks

- **Backlog olvidado**: cada ítem declara qué evento lo reabre (nuevo alcance, release
  de CLI/CMD, soporte de otro host).
- **Confusión con trabajo activo**: el change no tiene delta specs ni se aplica hasta
  que se apruebe su reapertura.

## Success criteria

- Ninguna tarea diferida aparece en el change activo.
- Cada ítem del backlog tiene condición de reapertura y dueño de decisión.
