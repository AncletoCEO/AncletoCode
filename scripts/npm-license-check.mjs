import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const lockPath = join(root, "package-lock.json");

const allowed = new Set([
  "0BSD",
  "Apache-2.0",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "CC0-1.0",
  "ISC",
  "MIT",
  "MPL-2.0",
  "Unicode-3.0",
  "Unlicense",
  "Zlib",
]);

if (!existsSync(lockPath)) {
  console.error("license:check:npm requiere package-lock.json");
  process.exit(1);
}

const lock = JSON.parse(readFileSync(lockPath, "utf8"));
const packages = lock.packages ?? {};
const failures = [];
let checked = 0;

function normalizeLicense(value) {
  if (typeof value === "string") {
    return value;
  }
  if (value && typeof value === "object" && typeof value.type === "string") {
    return value.type;
  }
  return undefined;
}

function isAllowed(expression) {
  const tokens = expression
    .replace(/[()]/g, " ")
    .split(/\s+(?:OR|AND|WITH)\s+/i)
    .map((token) => token.trim())
    .filter(Boolean);

  return tokens.length > 0 && tokens.every((token) => allowed.has(token));
}

for (const [packagePath, entry] of Object.entries(packages)) {
  if (!packagePath.startsWith("node_modules/")) {
    continue;
  }

  if (entry.dev === true) {
    continue;
  }

  checked += 1;
  const name = packagePath.slice("node_modules/".length);
  const license = normalizeLicense(entry.license);

  if (!license || license === "UNLICENSED") {
    failures.push(`${name}: licencia ausente o UNLICENSED`);
    continue;
  }

  if (!isAllowed(license)) {
    failures.push(`${name}: licencia no permitida (${license})`);
  }
}

if (failures.length > 0) {
  console.error(`license:check:npm revisó ${checked} paquetes y encontró problemas:`);
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log(`license:check:npm OK (${checked} paquetes)`);
