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

## IBM Plex Mono

Tipografía empaquetada para la app de escritorio y AncletoCMD.

- **Proyecto**: https://github.com/IBM/plex
- **Origen del archivo**: https://github.com/google/fonts/tree/main/ofl/ibmplexmono
- **Archivos**: `public/fonts/IBMPlexMono-Regular.ttf` y `public/fonts/OFL.txt`
- **Licencia**: SIL Open Font License 1.1

```text
Copyright © 2017 IBM Corp. with Reserved Font Name "Plex"

This Font Software is licensed under the SIL Open Font License, Version 1.1.

This license is copied below, and is also available with a FAQ at: http://scripts.sil.org/OFL


-----------------------------------------------------------
SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007
-----------------------------------------------------------

PREAMBLE
The goals of the Open Font License (OFL) are to stimulate worldwide
development of collaborative font projects, to support the font creation
efforts of academic and linguistic communities, and to provide a free and
open framework in which fonts may be shared and improved in partnership
with others.

The OFL allows the licensed fonts to be used, studied, modified and
redistributed freely as long as they are not sold by themselves. The
fonts, including any derivative works, can be bundled, embedded, 
redistributed and/or sold with any software provided that any reserved
names are not used by derivative works. The fonts and derivatives,
however, cannot be released under any other type of license. The
requirement for fonts to remain under this license does not apply
to any document created using the fonts or their derivatives.

DEFINITIONS
"Font Software" refers to the set of files released by the Copyright
Holder(s) under this license and clearly marked as such. This may
include source files, build scripts and documentation.

"Reserved Font Name" refers to any names specified as such after the
copyright statement(s).

"Original Version" refers to the collection of Font Software components as
distributed by the Copyright Holder(s).

"Modified Version" refers to any derivative made by adding to, deleting,
or substituting -- in part or in whole -- any of the components of the
Original Version, by changing formats or by porting the Font Software to a
new environment.

"Author" refers to any designer, engineer, programmer, technical
writer or other person who contributed to the Font Software.

PERMISSION & CONDITIONS
Permission is hereby granted, free of charge, to any person obtaining
a copy of the Font Software, to use, study, copy, merge, embed, modify,
redistribute, and sell modified and unmodified copies of the Font
Software, subject to the following conditions:

1) Neither the Font Software nor any of its individual components,
in Original or Modified Versions, may be sold by itself.

2) Original or Modified Versions of the Font Software may be bundled,
redistributed and/or sold with any software, provided that each copy
contains the above copyright notice and this license. These can be
included either as stand-alone text files, human-readable headers or
in the appropriate machine-readable metadata fields within text or
binary files as long as those fields can be easily viewed by the user.

3) No Modified Version of the Font Software may use the Reserved Font
Name(s) unless explicit written permission is granted by the corresponding
Copyright Holder. This restriction only applies to the primary font name as
presented to the users.

4) The name(s) of the Copyright Holder(s) or the Author(s) of the Font
Software shall not be used to promote, endorse or advertise any
Modified Version, except to acknowledge the contribution(s) of the
Copyright Holder(s) and the Author(s) or with their explicit written
permission.

5) The Font Software, modified or unmodified, in part or in whole,
must be distributed entirely under this license, and must not be
distributed under any other license. The requirement for fonts to
remain under this license does not apply to any document created
using the Font Software.

TERMINATION
This license becomes null and void if any of the above conditions are
not met.

DISCLAIMER
THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT
OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL THE
COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM
OTHER DEALINGS IN THE FONT SOFTWARE.
```

## Dependencias gráficas y de terminal

Auditoría de las dependencias directas usadas por AncletoCMD y la app de escritorio.
Todas las licencias están dentro de la allowlist de `deny.toml` y son compatibles
con AGPL-3.0.

| Dependencia | Versión | Licencia |
|---|---|---|
| `alacritty_terminal` | 0.26.0 | Apache-2.0 |
| `vte` | 0.15.0 | Apache-2.0 OR MIT |
| `winit` | 0.30.13 | Apache-2.0 |
| `wgpu` | 30.0.1 | MIT OR Apache-2.0 |
| `glyphon` | 0.12.0 | MIT OR Apache-2.0 OR Zlib |
| `cosmic-text` | 0.19.0 | MIT OR Apache-2.0 |
| `window-vibrancy` | 0.8.1 | Apache-2.0 OR MIT |
| `portable-pty` | 0.9.0 | MIT |
| `arboard` | 3.7.0 | MIT OR Apache-2.0 |
| `open` | 5.4.5 | MIT |
| `pollster` | 1.0.1 | Apache-2.0 OR MIT |
| `bytemuck` | 1.25.2 | Zlib OR Apache-2.0 OR MIT |
| `interprocess` | 2.4.6 | 0BSD OR Apache-2.0 |

El detalle completo de licencias transitivas se verifica con `cargo deny check
licenses advisories` antes de cada release.
