# Proposal: Updates firmados de la app (descarga, verificación y rollback)

## Problem

La app solo consulta GitHub Releases para avisar que hay una versión nueva; no hay
instalación desde la app, ni firma, ni verificación de integridad, ni rollback. Un
update fallido o manipulado podría dejar la app inutilizable o instalar un artefacto
no confiable. Tampoco existe una política de canales stable/prerelease ni de rotación
de claves.

## Proposed change

- Configurar el updater de Tauri con firma obligatoria, endpoint por canal
  (stable/prerelease) y clave pública embebida.
- Implementar descarga con progreso, verificación de firma y rollback ante fallo.
- Instalar en Windows y Linux con confirmación explícita si hay sesiones activas.
- Cubrir con tests firma inválida, descarga fallida, red caída y fallback.
- Documentar generación, custodia y rotación de claves; la privada vive solo en CI.

## Scope

**In scope**

- `tauri-plugin-updater` (Rust + JS), manifiesto de update firmado y endpoint.
- Canales stable/prerelease y selección explícita.
- Descarga con progreso, verificación y rollback.
- Confirmación con sesiones activas.
- Tests de integridad y documentación de claves/rotación.

**Out of scope**

- Actualización de `@ancleto/spec` (va en `ancleto-runtime-e2e`).
- Publicación de artefactos (va en `desktop-release-packaging`).
- Auto-update silencioso sin consentimiento.
- Actualización de dependencias del sistema.

## Dependencies

- Requiere los artefactos y checksums de `desktop-release-packaging` para publicar el
  manifiesto de update.

## Risks

- **Clave privada filtrada**: solo se versiona la pública; la privada vive en el
  secret store de CI y se documenta la rotación.
- **Update manipulado**: la firma se verifica antes de ejecutar; sin firma válida no
  se instala.
- **Update interrumpido**: la instalación es atómica y conserva la versión anterior
  hasta completar; si falla, se restaura.
- **Pérdida de sesiones**: si hay sesiones activas, la instalación exige confirmación
  y explica que se van a cerrar.
- **Canal equivocado**: el canal se muestra y se persiste; prerelease no es el default.

## Success criteria

- Un update firmado se descarga con progreso, se verifica y se instala.
- Una firma inválida o descarga fallida no modifica la instalación actual.
- Con sesiones activas la instalación pide confirmación.
- La documentación cubre claves, canales y rotación.
