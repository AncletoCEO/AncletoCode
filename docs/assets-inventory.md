# Inventario y auditoría de activos

Este inventario registra los activos binarios o creativos (imágenes, fuentes,
temas, iconos, sonidos, logos y screenshots) que se incorporan al repositorio, con
su origen y licencia. También deja constancia de las auditorías de copia
accidental de código o activos de PandaCode.

## Activos incorporados

| Activo | Ruta | Origen | Licencia | Permite redistribución |
|---|---|---|---|---|
| Icono de la app | `src-tauri/icon.svg`, `src-tauri/icons/` | Generado para AncletoCode | AGPL-3.0-only | Sí (obra propia) |

No se incluyen wallpapers, temas, iconos, screenshots, logos ni fuentes de
PandaCode, de proveedores o de terceros.

Cuando se incorpore un activo propio o de terceros, se agrega una fila con su
origen, licencia y verificación de redistribución, y se refleja en
`THIRD_PARTY_NOTICES.md` si corresponde.

## Auditoría del 2026-10-21

- **Alcance**: árbol completo del repositorio, excluyendo `.git/`.
- **Método**: listado de archivos, detección por extensión de activos
  (`png`, `jpg`, `jpeg`, `webp`, `svg`, `gif`, `ico`, `icns`, `ttf`, `otf`,
  `woff`, `woff2`, `mp3`, `wav`) y verificación de tipo de archivo binario.
- **Resultado**: no se encontraron activos ni archivos binarios. Todos los
  archivos del árbol son texto (documentación, licencias y artefactos SDD).
- **Código de PandaCode**: no hay copias de código upstream fuera de
  `third_party/pandacode/`; ese directorio sólo contiene la licencia y el
  registro de upstream. El código vendorizado se audita al incorporarse
  (Fase 3B), comparando rutas, commit y diff contra
  `e9f89891fd23518848765aef8219dbf39feac6e0`.
- **Marcas de proveedores**: no se usan logos ni marcas de PandaCode,
  Anthropic, Antigravity, Command Code u otros proveedores. Las referencias a
  CLIs son textuales y no sugieren afiliación ni endoso.

La auditoría se repite antes de cada release, junto con
`npm run license:check` y `cargo deny check`.
