import { copyFile, mkdir } from "node:fs/promises";
import { existsSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const triple =
  process.env.TAURI_ENV_TARGET_TRIPLE ??
  process.env.CARGO_BUILD_TARGET ??
  execFileSync("rustc", ["-vV"], { encoding: "utf8" }).match(/^host: (.+)$/m)?.[1];
if (!triple) throw new Error("no pude detectar el target triple");
const name = triple.includes("windows") ? "ancleto-code-host.exe" : "ancleto-code-host";
const candidates = [
  join(root, "target", triple, "release", name),
  join(root, "target", "release", name),
  join(root, "target", triple, "debug", name),
  join(root, "target", "debug", name),
];
const source = candidates.find(existsSync);
if (!source) throw new Error("compilá ancleto-code-host antes de preparar el sidecar");
await mkdir(join(root, "src-tauri", "binaries"), { recursive: true });
const destination = join(
  root,
  "src-tauri",
  "binaries",
  `ancleto-code-host-${triple}${triple.includes("windows") ? ".exe" : ""}`,
);
await copyFile(source, destination);
console.log(`host sidecar: ${destination}`);
