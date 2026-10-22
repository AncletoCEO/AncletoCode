# Third-Party Notices

AncletoCode se distribuye bajo la GNU Affero General Public License v3.0 (ver
[`LICENSE`](LICENSE)). Este archivo reúne los avisos de las obras de terceros
incorporadas al proyecto.

## PandaCode

AncletoCode reutiliza código de PandaCode. La declaración exigida por su licencia es:

> **This is a fork of PandaCode** — https://github.com/aicodeking/pandacode

- **Repositorio**: https://github.com/aicodeking/pandacode
- **Commit**: `e9f89891fd23518848765aef8219dbf39feac6e0` (v0.3.13)
- **Archivos incorporados**: ver [`third_party/pandacode/UPSTREAM.md`](third_party/pandacode/UPSTREAM.md)
  y [`docs/provenance.md`](docs/provenance.md).
- **Licencia**: PandaCode License (MIT License with an attribution requirement),
  reproducida en [`third_party/pandacode/LICENSE`](third_party/pandacode/LICENSE).

### PandaCode License (MIT License with an attribution requirement)

Copyright (c) 2026 PandaCode contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

If a modified version of the Software, or any work based on it (a "fork"), is
used commercially or released publicly in any form (including as source code,
a binary, an app, a hosted service or a website), the fork must state "This is
a fork of PandaCode" together with a link to
https://github.com/aicodeking/pandacode. The statement must appear in the
fork's README or main documentation, and in a place its users can readily see,
such as its About screen, website or store listing.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Enriched Markdown (md4c/WASM)

Si se vendoriza `src/vendor/enriched-markdown/` desde PandaCode, el parser
corresponde a Enriched Markdown, publicado bajo licencia MIT por Software Mansion.

- **Proyecto**: https://github.com/software-mansion/enriched-markdown
- **Licencia**: MIT
- **Requisito**: conservar el aviso de copyright y el texto de permiso al
  redistribuir el binario WASM o su wrapper.

El texto completo de la licencia se incorporará junto al módulo vendorizado en
`third_party/pandacode/src/vendor/enriched-markdown/LICENSE` al ejecutar la tarea
0B.8.

## Dependencias

Las licencias de los crates de Rust y de los paquetes npm se auditan con
`cargo deny` y el revisor de licencias npm. El reporte generado se incorpora a este
archivo o se referencia desde aquí antes de cada release.
