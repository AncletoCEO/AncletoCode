import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const temporary = mkdtempSync(join(tmpdir(), "acode-smoke-"));

function run(command, args, options = {}) {
  return execFileSync(command, args, { cwd: root, stdio: "inherit", ...options });
}

try {
  run("cargo", [
    "build",
    "--release",
    "-p",
    "ancleto-code-cli",
    "-p",
    "ancleto-code-host",
    "-p",
    "ancleto-code-cmd",
  ]);

  const platform = run(
    process.execPath,
    [
      "-e",
      `import("${join(root, "packages", "acode", "lib", "launcher.mjs").replaceAll("\\", "/")}").then(m => console.log(m.platformKey()))`,
    ],
    { stdio: ["ignore", "pipe", "inherit"] },
  )
    .toString()
    .trim();

  run(process.execPath, [
    join(root, "scripts", "package-npm.mjs"),
    "--binary-dir",
    join(root, "target", "release"),
    "--platform",
    platform,
    "--product",
    "all",
  ]);

  const packDestination = join(temporary, "packs");
  mkdirSync(packDestination, { recursive: true });
  for (const product of ["code", "cmd"]) {
    const platformDirectory = join(root, "packages", `${product}-${platform}`);
    run("npm", ["pack", platformDirectory, "--pack-destination", packDestination]);
  }
  run("npm", ["pack", join(root, "packages", "acode"), "--pack-destination", packDestination]);
  run("npm", ["pack", join(root, "packages", "acmd"), "--pack-destination", packDestination]);

  const tarballs = readdirSync(packDestination)
    .filter((name) => name.endsWith(".tgz"))
    .map((name) => join(packDestination, name));
  const prefix = join(temporary, "prefix");
  run("npm", ["install", "-g", "--prefix", prefix, "--ignore-scripts", ...tarballs]);

  const extension = process.platform === "win32" ? ".cmd" : "";
  const acode = join(prefix, "bin", `acode${extension}`);
  const version = execFileSync(acode, ["--version"], { encoding: "utf8" }).trim();
  console.log(`smoke-npm: ${version}`);
  JSON.parse(execFileSync(acode, ["--json", "agents"], { encoding: "utf8" }));
  JSON.parse(execFileSync(acode, ["--json", "doctor"], { encoding: "utf8" }));
  const acmd = join(prefix, "bin", `acmd${extension}`);
  const cmdVersion = execFileSync(acmd, ["--version"], { encoding: "utf8" }).trim();
  console.log(`smoke-npm: ${cmdVersion}`);
  console.log("smoke-npm: OK");
} finally {
  rmSync(temporary, { recursive: true, force: true });
}
