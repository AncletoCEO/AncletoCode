import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { spawn } from "node:child_process";

const require = createRequire(import.meta.url);

export const SUPPORTED = {
  acode: [
    "linux-x64-gnu",
    "linux-arm64-gnu",
    "linux-x64-musl",
    "linux-arm64-musl",
    "win32-x64",
    "win32-arm64",
    "darwin-x64",
    "darwin-arm64",
  ],
  acmd: [
    "linux-x64-gnu",
    "linux-arm64-gnu",
    "win32-x64",
    "win32-arm64",
    "darwin-x64",
    "darwin-arm64",
  ],
};

export const PACKAGES = {
  acode: "@ancleto/code",
  acmd: "@ancleto/cmd",
};

export const HOST_BINARY = "ancleto-code-host";

export function detectLibc() {
  if (process.platform !== "linux") {
    return null;
  }
  try {
    const report = process.report?.getReport();
    if (report?.header?.glibcVersionRuntime) {
      return "gnu";
    }
  } catch {
    // Sin reporte: se intenta por el loader.
  }
  try {
    if (readdirSync("/lib").some((name) => name.startsWith("ld-musl"))) {
      return "musl";
    }
  } catch {
    // Directorio /lib no legible.
  }
  return "gnu";
}

export function platformKey(platform = process.platform, arch = process.arch, libc = detectLibc()) {
  if (platform === "linux") {
    return `linux-${arch}-${libc ?? "gnu"}`;
  }
  return `${platform}-${arch}`;
}

export function platformPackage(binary, key) {
  return `${PACKAGES[binary]}-${key}`;
}

export function binaryName(binary, platform = process.platform) {
  return platform === "win32" ? `${binary}.exe` : binary;
}

export function sha256(buffer) {
  return createHash("sha256").update(buffer).digest("hex");
}

export function verifyChecksum(path, expected) {
  return sha256(readFileSync(path)) === expected;
}

export function missingPlatformMessage(binary, key) {
  const supported = SUPPORTED[binary].join(", ");
  return [
    `No hay binario de ${PACKAGES[binary]} para ${key}.`,
    `Plataformas soportadas: ${supported}.`,
    `Instalación manual: npm install -g ${PACKAGES[binary]}@latest --include=optional`,
    `Binarios publicados: https://github.com/AncletoCEO/AncletoCode/releases`,
  ].join("\n");
}

export function resolvePlatformDir(binary, key) {
  const name = platformPackage(binary, key);
  try {
    return dirname(require.resolve(`${name}/package.json`));
  } catch {
    return null;
  }
}

export function launch(binary, argv = process.argv.slice(2), env = process.env) {
  return launchBinary(binary, argv, env, { verifyHost: true });
}

function launchBinary(binary, argv, env, { verifyHost }) {
  const key = platformKey();
  if (!SUPPORTED[binary].includes(key)) {
    process.stderr.write(`acode: ${missingPlatformMessage(binary, key)}\n`);
    process.exit(3);
  }

  const directory = resolvePlatformDir(binary, key);
  if (!directory) {
    process.stderr.write(
      `acode: falta el paquete opcional ${platformPackage(binary, key)}.\n${missingPlatformMessage(binary, key)}\n`,
    );
    process.exit(3);
  }

  const checksumsPath = join(directory, "checksums.json");
  const checksums = existsSync(checksumsPath)
    ? JSON.parse(readFileSync(checksumsPath, "utf8"))
    : {};

  const executable = resolveExecutable(directory, binary, key, checksums, binary);
  if (verifyHost) {
    resolveExecutable(directory, HOST_BINARY, key, checksums, binary);
  }

  const child = spawn(executable, argv, { stdio: "inherit", env, windowsHide: false });
  for (const signal of ["SIGINT", "SIGTERM"]) {
    process.on(signal, () => {
      child.kill(signal);
    });
  }
  child.on("error", (error) => {
    process.stderr.write(`acode: no se pudo ejecutar ${executable}: ${error.message}\n`);
    process.exit(4);
  });
  child.on("exit", (code, signal) => {
    if (signal) {
      process.exit(128 + signalNumber(signal));
    }
    process.exit(code ?? 1);
  });
}

function resolveExecutable(directory, name, key, checksums, binary) {
  const file = join(
    directory,
    binaryName(name, key.startsWith("win32") ? "win32" : process.platform),
  );
  if (!existsSync(file)) {
    process.stderr.write(
      `acode: falta ${file} en ${platformPackage(binary, key)}.\n${missingPlatformMessage(binary, key)}\n`,
    );
    process.exit(3);
  }
  const expected =
    checksums[binaryName(name, key.startsWith("win32") ? "win32" : process.platform)];
  if (!expected) {
    process.stderr.write(
      `acode: el paquete ${platformPackage(binary, key)} no publica checksum para ${name}.\n`,
    );
    process.exit(5);
  }
  if (!verifyChecksum(file, expected)) {
    process.stderr.write(
      `acode: checksum inválido para ${name}.\nReinstalá con: npm install -g ${PACKAGES[binary]}@latest --include=optional\n`,
    );
    process.exit(5);
  }
  return file;
}

function signalNumber(signal) {
  return { SIGINT: 2, SIGTERM: 15, SIGHUP: 1, SIGKILL: 9 }[signal] ?? 1;
}
