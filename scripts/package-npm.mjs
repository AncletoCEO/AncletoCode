import { createHash } from "node:crypto";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

function argument(name, fallback) {
  const index = process.argv.indexOf(name);
  return index === -1 ? fallback : process.argv[index + 1];
}

const binaryDir = resolve(argument("--binary-dir", join(root, "target", "release")));
const platform = argument("--platform", null);
const version = argument(
  "--version",
  JSON.parse(readFileSync(join(root, "package.json"), "utf8")).version,
);
const product = argument("--product", "all");

if (!platform) {
  console.error("package-npm: falta --platform (ej. linux-x64-gnu)");
  process.exit(1);
}

const windows = platform.startsWith("win32");
const suffix = windows ? ".exe" : "";

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function platformFields(key) {
  const [os, cpu, libc] = key.split("-");
  return {
    os: [os],
    cpu: [cpu],
    ...(libc ? { libc: [libc === "gnu" ? "glibc" : libc] } : {}),
  };
}

function packageProduct(name, scope, binaries) {
  const directory = join(root, "packages", `${scope}-${platform}`);
  mkdirSync(directory, { recursive: true });

  const checksums = {};
  for (const binary of binaries) {
    const source = join(binaryDir, `${binary}${suffix}`);
    if (!existsSync(source)) {
      throw new Error(`package-npm: falta ${source}`);
    }
    const destination = join(directory, `${binary}${suffix}`);
    copyFileSync(source, destination);
    if (!windows) {
      chmodSync(destination, 0o755);
    }
    checksums[`${binary}${suffix}`] = sha256(destination);
  }

  writeFileSync(join(directory, "checksums.json"), `${JSON.stringify(checksums, null, 2)}\n`);
  copyFileSync(join(root, "LICENSE"), join(directory, "LICENSE"));
  copyFileSync(join(root, "THIRD_PARTY_NOTICES.md"), join(directory, "THIRD_PARTY_NOTICES.md"));

  const manifest = {
    name: `@ancleto/${scope}-${platform}`,
    version,
    description: `${name} para ${platform}`,
    license: "AGPL-3.0-only",
    homepage: "https://github.com/AncletoCEO/AncletoCode",
    repository: { type: "git", url: "git+https://github.com/AncletoCEO/AncletoCode.git" },
    ...platformFields(platform),
    files: [...Object.keys(checksums), "checksums.json", "LICENSE", "THIRD_PARTY_NOTICES.md"],
  };
  writeFileSync(join(directory, "package.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  console.log(`package-npm: ${manifest.name}@${version} en ${directory}`);
}

if (product === "all" || product === "code") {
  packageProduct("@ancleto/code", "code", ["acode", "ancleto-code-host"]);
}
if (product === "all" || product === "cmd") {
  packageProduct("@ancleto/cmd", "cmd", ["acmd"]);
}
