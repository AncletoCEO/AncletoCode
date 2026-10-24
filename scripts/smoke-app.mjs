import { existsSync } from "node:fs";
import { spawn } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const binary = join(root, "target/debug/ancleto-code-desktop");
const index = join(root, "dist/index.html");
const seconds = Number(process.env.SMOKE_SECONDS ?? 10);

if (!existsSync(index)) {
  console.error("smoke:app necesita el frontend compilado: npm run build");
  process.exit(1);
}
if (!existsSync(binary)) {
  console.error("smoke:app necesita el binario: cargo build --manifest-path src-tauri/Cargo.toml");
  process.exit(1);
}

const child = spawn(
  "timeout",
  [
    "--signal=TERM",
    String(seconds),
    "xvfb-run",
    "-a",
    "env",
    "WEBKIT_DISABLE_COMPOSITING_MODE=1",
    "NO_AT_BRIDGE=1",
    "GTK_A11Y=none",
    binary,
  ],
  { cwd: root },
);

let stdout = "";
let stderr = "";
child.stdout.on("data", (chunk) => {
  stdout += chunk;
});
child.stderr.on("data", (chunk) => {
  stderr += chunk;
});

child.on("error", (error) => {
  console.error(`smoke:app no pudo iniciar: ${error.message}`);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  const output = `${stdout}\n${stderr}`;
  const crashed =
    /panicked at|error while loading shared libraries|Segmentation fault/i.test(output) ||
    (code !== 124 && signal !== "SIGTERM");
  if (crashed) {
    console.error(`smoke:app falló (code=${code ?? "null"}, signal=${signal ?? "null"})`);
    console.error(output.trim() || "(sin salida)");
    process.exit(1);
  }
  const hostLine = output.split("\n").find((line) => line.includes("el host tardó"));
  console.log(`smoke:app OK (${seconds}s, sin panic, ventana iniciada)`);
  if (hostLine) {
    console.log(hostLine.trim());
  } else {
    console.log("host: < 150 ms en conectar (sin aviso de inicialización perezosa)");
  }
});
