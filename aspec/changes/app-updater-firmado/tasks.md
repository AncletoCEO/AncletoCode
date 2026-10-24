# Tasks: Updates firmados de la app (descarga, verificación y rollback)

## Fase 1 — Configuración y canales

- [ ] 1.1 Integrar `tauri-plugin-updater` (Rust + JS) y configurar el endpoint y la clave pública en `tauri.conf.json`.
- [ ] 1.2 Definir canales stable/prerelease con endpoints distintos, selección persistida y visible en Ajustes.
- [ ] 1.3 Firmar los artefactos y el manifiesto en el workflow de release; la clave privada vive solo en CI.

## Fase 2 — Instalación y rollback

- [ ] 2.1 Implementar descarga con progreso y cancelación sin modificar la instalación actual.
- [ ] 2.2 Verificar la firma antes de instalar; rechazar firma inválida, manifiesto malformado o descarga incompleta.
- [ ] 2.3 Instalar con activación atómica y rollback ante fallo, mostrando el error accionable.
- [ ] 2.4 Confirmar antes de instalar si hay sesiones activas y ofrecer detach.

## Fase 3 — Tests y documentación

- [ ] 3.1 Tests de firma válida, firma inválida, manifiesto malformado, descarga interrumpida y cancelación.
- [ ] 3.2 Documentar generación, custodia y rotación de claves, canales y checklist de release firmado.
- [ ] 3.3 Smoke manual de update en Linux y Windows con un manifiesto de prueba; verificar que no hay secretos en el repo ni en logs.
