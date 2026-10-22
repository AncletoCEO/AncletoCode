# Revisión de compatibilidad de licencias — 0.0.1

**Fecha:** 22 de octubre de 2026
**Alcance:** distribución 0.0.1 por GitHub Release (Linux x86_64 y Windows x64).
**Naturaleza:** revisión técnica de licencias y avisos. No es asesoría legal formal.

## Conclusión

**Las licencias son compatibles para distribuir 0.0.1.** No hay conflicto entre
AGPL-3.0-only y la PandaCode License (MIT + atribución de fork), ni con las
dependencias auditadas.

Que el producto no sea comercial **no elimina las obligaciones**: la cláusula de
PandaCode se activa también cuando el fork se libera públicamente, y la AGPL
aplica a la distribución de binarios y fuente. Tampoco las agrava: el uso
comercial está permitido por ambas licencias. La atribución ya se cumple y se
mantiene.

## Licencias entrantes

| Componente | Licencia | Compatibilidad |
|---|---|---|
| Código original de AncletoCode | AGPL-3.0-only | Licencia del proyecto |
| PandaCode | PandaCode License: MIT + atribución de fork | Compatible; se conservan MIT y la atribución |
| Enriched Markdown / md4c WASM | MIT | Compatible; aviso completo incluido |
| IBM Plex Mono | SIL OFL 1.1 | Compatible para empaquetar la fuente; no se vende por separado |
| Dependencias Rust | Apache-2.0, MIT, MPL-2.0, BSL-1.0, BSD, ISC, Zlib, etc. | Compatibles; allowlist de `deny.toml` |
| Dependencias npm de producción | Permisivas (MIT/ISC/Apache/BSD) | Compatibles; `license:check:npm` |

## Análisis de PandaCode

La PandaCode License contiene el texto MIT y agrega esta condición: si una
versión modificada o un fork se usa comercialmente o se libera públicamente, debe
decir **"This is a fork of PandaCode"** con un enlace a
`https://github.com/aicodeking/pandacode`, en el README o documentación principal
y en un lugar visible para los usuarios, como About.

### Por qué es compatible con AGPL-3.0-only

1. **Base MIT:** MIT es una licencia permisiva compatible con GPL/AGPL. Se puede
   incorporar el código a una obra mayor distribuida bajo AGPL, conservando el
   aviso de copyright y el texto de permiso.
2. **Condición de atribución:** no restringe usar, estudiar, modificar ni
   redistribuir; sólo exige un aviso. GPLv3 §7(b) —aplicable a AGPL por
   referencia— permite términos adicionales que exijan preservar avisos legales
   razonables o atribuciones de autor en el material o en los avisos del trabajo
   que lo contiene. La declaración de fork es exactamente una atribución de autor.
3. **No hay relicenciamiento del código de PandaCode:** los archivos derivados
   siguen bajo la PandaCode License. La distribución combinada se ofrece bajo
   AGPL-3.0-only para el código propio, con los avisos de MIT/PandaCode
   preservados. No se presenta el código de PandaCode como AGPL puro.
4. **AGPL §13:** la app es local y no ofrece interacción remota por red. Si en el
   futuro se ofrece como servicio, habrá que ofrecer la Corresponding Source a
   los usuarios remotos.

La condición de atribución se cumple en:

- `README.md` — declaración exacta y enlace.
- `src/components/AboutDialog.tsx` — pantalla About de la app.
- Cuerpo del GitHub Release `v0.0.1`.
- `packages/acode/README.md` y `packages/acmd/README.md` — paquetes npm futuros.
- `THIRD_PARTY_NOTICES.md`, `docs/provenance.md` y
  `third_party/pandacode/UPSTREAM.md` — avisos y trazabilidad.

### Activos y marcas

No se incorporaron wallpapers, temas, iconos, screenshots, logos ni fuentes de
PandaCode. La app usa tipografía IBM Plex Mono con su licencia OFL y activos
propios. No se sugiere afiliación o endoso de PandaCode.

## Obligaciones verificadas

- [x] `LICENSE` raíz con AGPL-3.0-only.
- [x] `THIRD_PARTY_NOTICES.md` con el texto completo de PandaCode, MIT, OFL,
      Apache-2.0 y BSL-1.0.
- [x] `third_party/pandacode/LICENSE` con la licencia upstream.
- [x] Cabeceras SPDX en los archivos vendorizados; 0 archivos sin cabecera.
- [x] Ledger de procedencia con commit, ruta, autor y modificaciones.
- [x] Declaración de fork en README, About, release y READMEs de paquetes.
- [x] Sin activos ni marcas de PandaCode.
- [x] `deny.toml` permite BSL-1.0 y documenta el advisory de `ttf-parser`.
- [x] `cargo deny check licenses advisories` OK.
- [x] `npm run license:check`, `license:check:npm` y `check:core-deps` OK.
- [x] Fuente correspondiente: el workflow genera
      `AncletoCode-0.0.1-source.tar.gz` desde el mismo commit del build.

## Riesgos residuales

- **`ttf-parser` no mantenido (RUSTSEC-2026-0192):** es transitiva de
  `cosmic-text`/`glyphon` y `winit`; no hay actualización compatible con el stack
  fijado. Es un riesgo de mantenimiento, no de licencia. Queda ignorado con
  justificación en `deny.toml` y debe revisarse al actualizar el stack gráfico.
- **Fuente del release:** el asset de fuente se genera desde la etiqueta con la
  que se ejecuta el workflow; no se mantiene una copia manual separada.
- **Revisión legal formal:** esta revisión cubre compatibilidad y avisos, no
  sustituye una opinión legal si en el futuro hay una versión comercial o un
  contrato con terceros. Para 0.0.1 interno/beta no se identificó un blocker.
- **Avisos de dependencias MIT/Apache transitivas:** se incluye el texto de las
  licencias y la tabla de dependencias; si se publica en npm o se redistribuye
  masivamente, conviene generar un bundle de avisos por dependencia con
  `cargo-about` antes de un release estable.

## Veredicto

**Aprobado para 0.0.1** como release interno/beta, con atribución visible y
avisos completos. Antes de un release estable o comercial se recomienda una
revisión legal externa y regenerar el bundle de avisos de dependencias.
