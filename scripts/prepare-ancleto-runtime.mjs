import { createHash } from "node:crypto";
import { chmod, copyFile, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import extract from "extract-zip";
import * as tar from "tar";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const cacheDir = join(root, ".cache", "ancleto-runtime");
const binariesDir = join(root, "src-tauri", "binaries");
const resourcesDir = join(root, "src-tauri", "resources", "ancleto");
const nodeVersion = process.env.ANCLETO_NODE_VERSION ?? "v24.22.1";
const force = process.argv.includes("--force");

function targetTriple() {
  const configured = process.env.TAURI_ENV_TARGET_TRIPLE ?? process.env.CARGO_BUILD_TARGET;
  if (configured) {
    return configured;
  }
  const output = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
  const host = output.match(/^host: (.+)$/m)?.[1];
  if (!host) {
    throw new Error("no pude detectar el target triple de Rust");
  }
  return host;
}

function nodeArchive(triple) {
  const platform = triple.includes("windows")
    ? "win"
    : triple.includes("darwin")
      ? "darwin"
      : "linux";
  const arch = triple.startsWith("aarch64") ? "arm64" : "x64";
  const extension = platform === "win" ? "zip" : "tar.gz";
  return {
    platform,
    name: `node-${nodeVersion}-${platform}-${arch}.${extension}`,
    url: `https://nodejs.org/dist/${nodeVersion}/node-${nodeVersion}-${platform}-${arch}.${extension}`,
  };
}

async function download(url, destination) {
  if (existsSync(destination) && !force) {
    return destination;
  }
  console.log(`descargando ${url}`);
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`falló la descarga (${response.status}): ${url}`);
  }
  await mkdir(dirname(destination), { recursive: true });
  await writeFile(destination, Buffer.from(await response.arrayBuffer()));
  return destination;
}

function sha512Base64(buffer) {
  return createHash("sha512").update(buffer).digest("base64");
}

async function prepareNode(triple) {
  const archive = nodeArchive(triple);
  const file = join(cacheDir, archive.name);
  await download(archive.url, file);
  const extractDir = join(cacheDir, archive.name.replace(/\.(zip|tar\.gz)$/, ""));
  if (!existsSync(extractDir) || force) {
    await rm(extractDir, { recursive: true, force: true });
    await mkdir(extractDir, { recursive: true });
    if (archive.name.endsWith(".zip")) {
      await extract(file, { dir: extractDir });
    } else {
      await tar.x({ file, cwd: extractDir });
    }
  }
  const bundleDir = join(
    extractDir,
    `node-${nodeVersion}-${archive.platform}-${archive.name.includes("arm64") ? "arm64" : "x64"}`,
  );
  const nodeName = archive.platform === "win" ? "node.exe" : "node";
  const source = join(bundleDir, archive.platform === "win" ? nodeName : join("bin", nodeName));
  if (!existsSync(source)) {
    throw new Error(`no encontré ${source} en la distribución de Node`);
  }
  await mkdir(binariesDir, { recursive: true });
  const destination = join(
    binariesDir,
    `ancleto-node-${triple}${archive.platform === "win" ? ".exe" : ""}`,
  );
  await copyFile(source, destination);
  if (archive.platform !== "win") {
    await chmod(destination, 0o755);
  }
  console.log(`node listo: ${destination}`);
}

async function prepareSpec() {
  const latest = await (await fetch("https://registry.npmjs.org/@ancleto/spec/latest")).json();
  if (!latest.version || !latest.dist?.tarball || !latest.dist?.integrity) {
    throw new Error("respuesta inválida del registry de npm");
  }
  const versionFile = join(resourcesDir, "version.txt");
  const packageDir = join(resourcesDir, "package");
  if (!force && existsSync(versionFile) && existsSync(join(packageDir, "package.json"))) {
    const installed = (await readFile(versionFile, "utf8")).trim();
    if (installed === latest.version) {
      console.log(`@ancleto/spec ${latest.version} ya está incluido`);
      return;
    }
  }

  const archive = join(cacheDir, `ancleto-spec-${latest.version}.tgz`);
  await download(latest.dist.tarball, archive);
  const bytes = await readFile(archive);
  const [algorithm, expected] = latest.dist.integrity.split("-", 2);
  if (algorithm !== "sha512" || sha512Base64(bytes) !== expected) {
    throw new Error("la integridad del tarball de @ancleto/spec no coincide");
  }
  const staging = join(cacheDir, `ancleto-spec-${latest.version}`);
  await rm(staging, { recursive: true, force: true });
  await mkdir(staging, { recursive: true });
  await tar.x({ file: archive, cwd: staging });
  await rm(packageDir, { recursive: true, force: true });
  await mkdir(resourcesDir, { recursive: true });
  await copyTree(join(staging, "package"), packageDir);
  await writeFile(versionFile, `${latest.version}\n`);
  console.log(`@ancleto/spec ${latest.version} incluido en ${packageDir}`);
}


async function prepareWriter() {
  const latest = await (await fetch("https://registry.npmjs.org/@ancleto/writer/latest")).json();
  if (!latest.version || !latest.dist?.tarball || !latest.dist?.integrity) {
    throw new Error("respuesta inválida del registry para @ancleto/writer");
  }
  const writerDir = join(resourcesDir, "writer");
  const versionFile = join(writerDir, "version.txt");
  const packageDir = join(writerDir, "package");
  if (!force && existsSync(versionFile) && existsSync(join(packageDir, "src/cli/index.js"))) {
    const installed = (await readFile(versionFile, "utf8")).trim();
    if (installed === latest.version) {
      console.log(`@ancleto/writer ${latest.version} ya está incluido`);
      return;
    }
  }

  const archive = join(cacheDir, `ancleto-writer-${latest.version}.tgz`);
  await download(latest.dist.tarball, archive);
  const bytes = await readFile(archive);
  const [algorithm, expected] = latest.dist.integrity.split("-", 2);
  if (algorithm !== "sha512" || sha512Base64(bytes) !== expected) {
    throw new Error("la integridad del tarball de @ancleto/writer no coincide");
  }
  const staging = join(cacheDir, `ancleto-writer-${latest.version}`);
  await rm(staging, { recursive: true, force: true });
  await mkdir(staging, { recursive: true });
  await tar.x({ file: archive, cwd: staging });
  await rm(packageDir, { recursive: true, force: true });
  await mkdir(writerDir, { recursive: true });
  await copyTree(join(staging, "package"), packageDir);
  await writeFile(versionFile, `${latest.version}\n`);
  console.log(`@ancleto/writer ${latest.version} incluido en ${packageDir}`);
}

async function copyTree(source, destination) {
  const { cp } = await import("node:fs/promises");
  await cp(source, destination, { recursive: true });
}

try {
  const triple = targetTriple();
  console.log(`target: ${triple}`);
  await prepareNode(triple);
  await prepareSpec();
  await prepareWriter();
} catch (error) {
  console.error(`prepare-ancleto-runtime: ${error instanceof Error ? error.message : error}`);
  process.exit(1);
}
