import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const coreRoot = join(root, "crates", "ancleto-code-core");
const manifestPath = join(coreRoot, "Cargo.toml");
const failures = [];

const forbiddenCrates = new Set([
  "ancleto-code",
  "ancleto-code-cmd",
  "ancleto-code-host",
  "ancleto-code-tauri",
  "glyphon",
  "tauri",
  "tauri-build",
  "wgpu",
  "winit",
]);

const forbiddenImports = [
  "ancleto_code_cmd",
  "ancleto_code_host",
  "glyphon::",
  "tauri::",
  "use tauri",
  "use wgpu",
  "use winit",
  "wgpu::",
  "winit::",
];

if (!existsSync(manifestPath)) {
  console.error("check:core-deps no encontró crates/ancleto-code-core/Cargo.toml");
  process.exit(1);
}

const manifest = readFileSync(manifestPath, "utf8");
const dependencySections = new Set(["dependencies", "dev-dependencies", "build-dependencies"]);
let currentSection = "";

for (const rawLine of manifest.split(/\r?\n/)) {
  const line = rawLine.trim();
  const sectionMatch = /^\[([^\]]+)\]$/.exec(line);

  if (sectionMatch) {
    currentSection = sectionMatch[1] ?? "";
    continue;
  }

  if (!dependencySections.has(currentSection) || line.length === 0 || line.startsWith("#")) {
    continue;
  }

  const nameMatch = /^"?([A-Za-z0-9_-]+)"?\s*=/.exec(line);
  if (!nameMatch) {
    continue;
  }

  const crateName = nameMatch[1];
  if (crateName && forbiddenCrates.has(crateName)) {
    failures.push(`El core declara una dependencia prohibida: ${crateName}`);
  }
}

function walk(directory) {
  const files = [];
  for (const name of readdirSync(directory)) {
    const absolutePath = join(directory, name);
    if (statSync(absolutePath).isDirectory()) {
      files.push(...walk(absolutePath));
    } else if (absolutePath.endsWith(".rs")) {
      files.push(absolutePath);
    }
  }
  return files;
}

const sourceRoot = join(coreRoot, "src");
if (existsSync(sourceRoot)) {
  for (const file of walk(sourceRoot)) {
    const contents = readFileSync(file, "utf8");
    for (const forbidden of forbiddenImports) {
      if (contents.includes(forbidden)) {
        failures.push(`${file.slice(root.length + 1)} importa o menciona ${forbidden}`);
      }
    }
  }
}

if (failures.length > 0) {
  console.error("check:core-deps encontró violaciones de frontera:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log("check:core-deps OK");
