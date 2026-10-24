# Design: Updates firmados de la app (descarga, verificación y rollback)

## Approach

Se adopta el updater oficial de Tauri: el manifiesto de update incluye la firma del
artefacto y la app trae embebida la clave pública. La descarga ocurre fuera del
directorio de instalación y la activación es atómica; ante cualquier fallo se conserva
la versión actual. El chequeo de GitHub Releases existente queda como aviso previo,
no como mecanismo de instalación.

```
check (canal) ──► manifiesto firmado ──► ¿firma válida?
                                             │ sí            │ no
                                     descarga + progreso     error, sin cambios
                                             │
                                  ¿sesiones activas? ── sí ──► confirmar
                                             │ no
                                     instalar (atómico) ── fallo ──► rollback
```

## Architecture

- **Plugin**: `tauri-plugin-updater` en Rust y `@tauri-apps/plugin-updater` en el
  frontend; `tauri.conf.json` declara endpoints por canal y `pubkey`.
- **Canales**: `stable` y `prerelease` con endpoints distintos; el canal activo se
  guarda en settings y se muestra en Ajustes.
- **Firma**: la clave privada solo existe en CI; el manifiesto y los artefactos se
  firman en el workflow de release. La app verifica antes de instalar.
- **Progreso y cancelación**: la descarga reporta bytes/total y permite cancelar sin
  tocar la instalación.
- **Sesiones activas**: antes de instalar se consulta el backend de sesiones; si hay
  alguna, se pide confirmación y se ofrece detach.
- **Rollback**: si la instalación falla, el instalador conserva la versión anterior;
  el estado del último intento queda visible con el error.
- **Documentación**: `docs/updates.md` con generación de claves, custodia, rotación y
  checklist de release firmado.

## Validation

```bash
cargo test --manifest-path src-tauri/Cargo.toml updater
npm run test
npm run typecheck
npm run check:commands
```

- Tests: firma válida, firma inválida, manifiesto malformado, descarga interrumpida y
  cancelación.
- Test de sesiones activas: la instalación no avanza sin confirmación.
- Smoke manual en Linux y Windows con un manifiesto de prueba y clave de test.
- Verificar que la clave privada no aparece en el repositorio ni en logs.
