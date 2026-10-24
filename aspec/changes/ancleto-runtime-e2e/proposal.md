# Proposal: E2E y rollback del runtime Ancleto embebido

## Problem

La app ya incluye el runtime Node y `@ancleto/spec`, y la actualización administrada
desde Ajustes existe parcialmente. Faltan las pruebas end-to-end de instalación,
actualización y rollback, y la garantía de que un update fallido deja operativa la
versión incluida. Sin eso no se puede confiar en el runtime embebido como fuente de
verdad del SDD.

## Proposed change

- Cubrir con E2E el ciclo completo del runtime embebido: versión incluida, update
  administrado, verificación de integridad y fallback.
- Implementar y probar rollback ante fallo de descarga, checksum inválido, corrupción o
  incompatibilidad de versión.
- Verificar que la app y las CLIs usan el mismo runtime y que un fallo no bloquea la
  apertura de proyectos.

## Scope

**In scope**

- E2E del runtime embebido (instalado, actualizado, caído).
- Rollback y fallback a la versión incluida.
- Mensajes accionables y estado visible en Ajustes.

**Out of scope**

- Publicación del runtime o de `@ancleto/spec` (fuera de este change).
- Actualización de la app de escritorio en sí.
- Instalación automática de dependencias del sistema.

## Risks

- **Update parcial deja el runtime inutilizable**: escritura atómica y verificación
  antes de activar; si falla, se conserva la versión anterior.
- **Checksum o integridad inválida**: se rechaza el paquete y se reporta sin borrar la
  versión activa.
- **Divergencia entre app y CLI**: ambas consumen el runtime resuelto por el mismo
  módulo; se testea el mismo resultado.
- **Red ausente**: el estado se muestra como offline y la versión incluida sigue
  funcionando.

## Success criteria

- El E2E cubre instalación, update exitoso, update fallido y rollback.
- Un update fallido deja la versión anterior operativa.
- Ajustes muestra versión activa, incluida y error de forma clara.
- Tests de integración en verde sin depender de red real.
