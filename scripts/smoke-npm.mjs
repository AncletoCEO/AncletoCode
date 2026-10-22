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
  run("cargo", ["build", "--release", "-p", "ancleto-code-cli", "-p", "ancleto-code-host"]);

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
    "code",
  ]);

  const platformDirectory = join(root, "packages", `code-${platform}`);
  const packDestination = join(temporary, "packs");
  mkdirSync(packDestination, { recursive: true });
  run("npm", ["pack", platformDirectory, "--pack-destination", packDestination]);
  run("npm", ["pack", join(root, "packages", "acode"), "--pack-destination", packDestination]);

  const tarballs = readdirSync(packDestination)
    .filter((name) => name.endsWith(".tgz"))
    .map((name) => join(packDestination, name));
  const prefix = join(temporary, "prefix");
  run("npm", ["install", "-g", "--prefix", prefix, "--ignore-scripts", ...tarballs]);

  const bin = join(prefix, "bin", process.platform === "win32" ? "acode.cmd" : "acode");
  const version = execFileSync(bin, ["--version"], { encoding: "utf8" }).trim();
  console.log(`smoke-npm: ${version}`);
  const agents = execFileSync(bin, ["--json", "agents"], { encoding: "utf8" });
  JSON.parse(agents);
  console.log("smoke-npm: OK");
} finally {
  rmSync(temporary, { recursive: true, force: true });
}
