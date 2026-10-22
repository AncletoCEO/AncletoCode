import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const version = process.argv[2];

if (!version || !/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(version)) {
  console.error("set-version: pasá una versión semver, ej. 0.2.0");
  process.exit(1);
}

const rootPackage = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
rootPackage.version = version;
writeFileSync(join(root, "package.json"), `${JSON.stringify(rootPackage, null, 2)}\n`);

for (const name of ["acode", "acmd"]) {
  const path = join(root, "packages", name, "package.json");
  const manifest = JSON.parse(readFileSync(path, "utf8"));
  manifest.version = version;
  for (const dependency of Object.keys(manifest.optionalDependencies ?? {})) {
    manifest.optionalDependencies[dependency] = version;
  }
  writeFileSync(path, `${JSON.stringify(manifest, null, 2)}\n`);
}

const cargoPath = join(root, "Cargo.toml");
const cargo = readFileSync(cargoPath, "utf8").replace(
  /(\[workspace\.package\][\s\S]*?version\s*=\s*")([^"]+)(")/,
  `$1${version}$3`,
);
writeFileSync(cargoPath, cargo);

const lockPath = join(root, "package-lock.json");
try {
  const lock = JSON.parse(readFileSync(lockPath, "utf8"));
  lock.version = version;
  if (lock.packages?.[""]) {
    lock.packages[""].version = version;
  }
  writeFileSync(lockPath, `${JSON.stringify(lock, null, 2)}\n`);
} catch {
  // Sin lockfile: no hay nada que propagar.
}

console.log(`set-version: ${version}`);
