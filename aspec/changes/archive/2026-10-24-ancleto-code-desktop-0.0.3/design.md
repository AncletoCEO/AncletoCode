> **Diseño histórico — 2026-10-24.** Las secciones de GitHub, Task/Thread/Worktree,
> Updates y Ancleto embebido se movieron a `github-provider-prs`,
> `workspace-shell-y-modelo`, `app-updater-firmado` y `ancleto-runtime-e2e`
> respectivamente. La sección de Usage describe lo entregado en este change.

# Design: Desktop 0.0.3

## Usage

Un `UsageProvider` por CLI con contrato común: `status`, `windows`, `cost`,
`source`, `checkedAt`, `message`. Las fuentes son endpoints de cuota del proveedor
o estado local de la CLI; nunca inferencia. Cache por proveedor y timeout por
adapter. Los porcentajes se muestran como **usado** y los datos ausentes como
`—`, nunca como 0%.

## GitHub

Extender el servicio Git actual con operaciones `gh` para checks, review y merge.
La UI sólo habilita merge cuando los checks requeridos están verdes y el usuario
confirma. Cada operación devuelve un outcome accionable; GitLab no se implementa.

## Task/Thread/Worktree

`Task` ya persiste rama/worktree/sesión. Se agrega la selección del worktree como
contexto activo del proyecto, threads por tarea y reconciliación al reabrir.
La limpieza de worktrees exige confirmación si hay cambios sin commit.

## Updates

El chequeo actual de GitHub Releases se mantiene. La instalación usa el updater de
Tauri con firma, descarga verificada, progreso y rollback. Ancleto se actualiza
por separado desde npm con integridad y fallback al paquete incluido.

## Ancleto embebido

El runtime Node y `@ancleto/spec` viajan en la app. La actualización descarga a un
directorio versionado, verifica SHA-512 y activa atómicamente; si falla, conserva
la versión anterior. Tests con fixture local y modo offline.
