import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

function argument(name, fallback) {
  const index = process.argv.indexOf(name);
  return index === -1 ? fallback : process.argv[index + 1];
}

const version = argument("--version", null);
const dist = resolve(argument("--dist", join(root, "dist")));
const url = "https://github.com/AncletoCEO/AncletoCode";

if (!version) {
  console.error("package-aur: falta --version");
  process.exit(1);
}

const sha256 = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");

const definitions = {
  "ancleto-code-bin": {
    name: "AncletoCode",
    prefix: "AncletoCode",
    desc: "Workspace de agentes de código para escritorio y terminal",
    depends: ["webkit2gtk-4.1", "gtk3", "libsoup3", "librsvg", "ancleto-code-cli-bin"],
    provides: ["ancleto-code"],
    conflicts: ["ancleto-code"],
    install: `
  install -Dm755 "\${srcdir}/ancleto-code-desktop" "\$pkgdir/usr/bin/ancleto-code-desktop"
  install -Dm644 "\${srcdir}/ancleto-code.desktop" "\$pkgdir/usr/share/applications/ancleto-code.desktop"
  install -Dm644 "\${srcdir}/icons/32x32.png" "\$pkgdir/usr/share/icons/hicolor/32x32/apps/ancleto-code.png"
  install -Dm644 "\${srcdir}/icons/128x128.png" "\$pkgdir/usr/share/icons/hicolor/128x128/apps/ancleto-code.png"
  install -Dm644 "\${srcdir}/icons/256x256.png" "\$pkgdir/usr/share/icons/hicolor/256x256/apps/ancleto-code.png"
`,
  },
  "ancleto-cmd-bin": {
    name: "AncletoCMD",
    prefix: "AncletoCMD",
    desc: "Emulador de terminal nativo y gestor de agentes",
    depends: [
      "libxkbcommon",
      "vulkan-icd-loader",
      "fontconfig",
      "freetype2",
      "wayland",
      "libx11",
      "ancleto-code-cli-bin",
    ],
    provides: ["ancleto-cmd"],
    conflicts: ["ancleto-cmd"],
    install: `
  install -Dm755 "\${srcdir}/acmd" "\$pkgdir/usr/bin/acmd"
  install -Dm755 "\${srcdir}/ancleto-cmd" "\$pkgdir/usr/bin/ancleto-cmd"
`,
  },
  "ancleto-code-cli-bin": {
    name: "acode",
    prefix: "acode",
    desc: "CLI headless y host de sesiones de AncletoCode",
    depends: [],
    provides: ["acode", "ancleto-code-cli"],
    conflicts: ["ancleto-code-cli"],
    install: `
  install -Dm755 "\${srcdir}/acode" "\$pkgdir/usr/bin/acode"
  install -Dm755 "\${srcdir}/ancleto-code" "\$pkgdir/usr/bin/ancleto-code"
  install -Dm755 "\${srcdir}/ancleto-code-host" "\$pkgdir/usr/bin/ancleto-code-host"
`,
  },
};

const architectures = [];
for (const [key, arch] of [
  ["linux-x86_64", "x86_64"],
  ["linux-aarch64", "aarch64"],
]) {
  const available = Object.values(definitions).every((definition) =>
    existsSync(join(dist, `${definition.prefix}-${version}-${key}.tar.gz`)),
  );
  if (available) architectures.push({ key, arch });
}

if (architectures.length === 0) {
  console.error(`package-aur: no hay tarballs de ${version} en ${dist}`);
  process.exit(1);
}

for (const [pkgname, definition] of Object.entries(definitions)) {
  const archLine = architectures.map((entry) => `'${entry.arch}'`).join(" ");
  const sources = architectures
    .map(
      (entry) =>
        `source_${entry.arch}=("\${url}/releases/download/v\${pkgver}/${definition.prefix}-\${pkgver}-${entry.key}.tar.gz")`,
    )
    .join("\n");
  const sums = architectures
    .map((entry) => {
      const archive = join(dist, `${definition.prefix}-${version}-${entry.key}.tar.gz`);
      return `sha256sums_${entry.arch}=('${sha256(archive)}')`;
    })
    .join("\n");

  const pkgbuild = `# Maintainer: Ancleto <release@ancleto.dev>
pkgname=${pkgname}
pkgver=${version}
pkgrel=1
pkgdesc="${definition.desc}"
arch=(${archLine})
url="${url}"
license=('AGPL-3.0-only')
depends=(${definition.depends.map((dependency) => `'${dependency}'`).join(" ")})
provides=(${definition.provides.map((provided) => `'${provided}'`).join(" ")})
conflicts=(${definition.conflicts.map((conflict) => `'${conflict}'`).join(" ")})
${sources}
${sums}
options=('!strip')

package() {
${definition.install.trimEnd()}
  install -Dm644 "\${srcdir}/LICENSE" "\$pkgdir/usr/share/licenses/\${pkgname}/LICENSE"
  install -Dm644 "\${srcdir}/THIRD_PARTY_NOTICES.md" "\$pkgdir/usr/share/licenses/\${pkgname}/THIRD_PARTY_NOTICES.md"
}
`;

  const directory = join(dist, "aur", pkgname);
  mkdirSync(directory, { recursive: true });
  writeFileSync(join(directory, "PKGBUILD"), pkgbuild);
  console.log(`package-aur: ${join(directory, "PKGBUILD")}`);
}
