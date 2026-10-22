import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const failures = [];

const rootPackage = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const expected = rootPackage.version;
const cargo = readFileSync(join(root, "Cargo.toml"), "utf8");
const cargoVersion = /\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"/.exec(cargo)?.[1];

if (cargoVersion !== expected) {
  failures.push(`Cargo.toml workspace=${cargoVersion ?? "?"} vs package.json=${expected}`);
}

for (const name of ["acode", "acmd"]) {
  const manifestPath = join(root, "packages", name, "package.json");
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  if (manifest.version !== expected) {
    failures.push(`${name} package=${manifest.version} vs ${expected}`);
  }
  for (const [dependency, version] of Object.entries(manifest.optionalDependencies ?? {})) {
    if (version !== expected) {
      failures.push(`${name} optionalDependency ${dependency}=${version} vs ${expected}`);
    }
  }
}

const tauri = JSON.parse(readFileSync(join(root, "src-tauri", "tauri.conf.json"), "utf8"));
if (tauri.version !== expected) {
  failures.push(`tauri.conf.json=${tauri.version} vs ${expected}`);
}
const about = readFileSync(join(root, "src", "components", "AboutDialog.tsx"), "utf8");
if (!about.includes(`Versión ${expected}`)) {
  failures.push(`AboutDialog no muestra la versión ${expected}`);
}

const binary = process.argv[2] ?? join(root, "target", "debug", "acode");
if (existsSync(binary)) {
  const output = execFileSync(binary, ["--version"], { encoding: "utf8" }).trim();
  const reported = output.split(/\s+/).at(-1);
  if (reported !== expected) {
    failures.push(`acode --version=${reported} vs ${expected}`);
  }
}

if (failures.length > 0) {
  console.error("check:versions encontró problemas:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}
console.log(`check:versions OK (${expected})`);
