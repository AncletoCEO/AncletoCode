import { chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

function argument(name, fallback) {
  const index = process.argv.indexOf(name);
  return index === -1 ? fallback : process.argv[index + 1];
}

const version = argument("--version", null);
const platform = argument("--platform", "linux-x86_64");
const product = argument("--product", "desktop");
const binaryDir = resolve(argument("--binary-dir", join(root, "target", "release")));
const outDir = resolve(argument("--out", join(root, "dist")));

if (!version) {
  console.error("package-desktop-tarball: falta --version");
  process.exit(1);
}
if (platform.startsWith("win32")) {
  console.error("package-desktop-tarball: usá el paso de PowerShell para Windows");
  process.exit(1);
}

const products = {
  desktop: { name: "AncletoCode", binaries: ["ancleto-code-desktop", "ancleto-code-host"] },
  cmd: { name: "AncletoCMD", binaries: ["acmd", "ancleto-cmd", "ancleto-code-host"] },
  cli: { name: "acode", binaries: ["acode", "ancleto-code", "ancleto-code-host"] },
};
const selected = products[product];
if (!selected) {
  console.error(`package-desktop-tarball: producto desconocido ${product}`);
  process.exit(1);
}

const staging = mkdtempSync(join(tmpdir(), "ancleto-tarball-"));
try {
  for (const binary of selected.binaries) {
    const source = join(binaryDir, binary);
    if (!existsSync(source)) {
      throw new Error(`falta el binario ${source}`);
    }
    const destination = join(staging, binary);
    copyFileSync(source, destination);
    chmodSync(destination, 0o755);
  }

  copyFileSync(join(root, "LICENSE"), join(staging, "LICENSE"));
  copyFileSync(join(root, "THIRD_PARTY_NOTICES.md"), join(staging, "THIRD_PARTY_NOTICES.md"));

  if (product === "desktop") {
    copyFileSync(
      join(root, "packaging", "linux", "ancleto-code.desktop"),
      join(staging, "ancleto-code.desktop"),
    );
    const icons = join(staging, "icons");
    mkdirSync(icons, { recursive: true });
    for (const [source, name] of [
      ["32x32.png", "32x32.png"],
      ["128x128.png", "128x128.png"],
      ["128x128@2x.png", "256x256.png"],
    ]) {
      copyFileSync(join(root, "src-tauri", "icons", source), join(icons, name));
    }
  }

  mkdirSync(outDir, { recursive: true });
  const archive = join(outDir, `${selected.name}-${version}-${platform}.tar.gz`);
  execFileSync("tar", ["-czf", archive, "-C", staging, "."], { stdio: "inherit" });
  console.log(`package-desktop-tarball: ${archive}`);
} finally {
  rmSync(staging, { recursive: true, force: true });
}
