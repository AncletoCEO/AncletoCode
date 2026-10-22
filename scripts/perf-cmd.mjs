import { execFileSync, spawn } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, mkdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const binary = join(root, "target", "debug", "acmd");
if (!existsSync(binary)) {
  execFileSync("cargo", ["build", "-p", "ancleto-code-cmd"], { cwd: root, stdio: "inherit" });
}

const temporary = mkdtempSync(join(tmpdir(), "acmd-perf-"));
const perfOut = join(temporary, "perf.json");
const state = join(temporary, "state");
const home = join(temporary, "home");
mkdirSync(state, { recursive: true });
mkdirSync(home, { recursive: true });

const env = {
  ...process.env,
  ANCLETO_CODE_HOME: home,
  ANCLETO_CODE_STATE: state,
  ACMD_PERF_OUT: perfOut,
  WINIT_UNIX_BACKEND: "x11",
};
delete env.WAYLAND_DISPLAY;

const child = spawn(binary, [], {
  cwd: root,
  env,
  stdio: ["ignore", "ignore", "inherit"],
});

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

try {
  let window;
  for (let attempt = 0; attempt < 100; attempt += 1) {
    try {
      window = execFileSync("xdotool", ["search", "--name", "AncletoCMD"], { encoding: "utf8" })
        .trim()
        .split("\n")[0];
      if (window) break;
    } catch {
      // Ventana aún no mapeada.
    }
    await sleep(100);
  }
  if (!window) throw new Error("no apareció la ventana de AncletoCMD");

  execFileSync("xdotool", ["windowactivate", "--sync", window]);
  await sleep(600);

  const ticks = () => {
    const stat = readFileSync(`/proc/${child.pid}/stat`, "utf8").split(" ");
    return Number(stat[13]) + Number(stat[14]);
  };
  const cpuBefore = ticks();
  await sleep(2000);
  const idlePercent = ((ticks() - cpuBefore) / 100 / 2) * 100;

  for (let index = 0; index < 12; index += 1) {
    execFileSync("xdotool", ["key", "--window", window, "a"]);
    await sleep(40);
    execFileSync("xdotool", ["key", "--window", window, "BackSpace"]);
    await sleep(40);
  }
  await sleep(500);
  execFileSync("xdotool", ["key", "ctrl+shift+q"]);
  await new Promise((resolve) => child.once("exit", resolve));
  await sleep(300);

  const metrics = JSON.parse(readFileSync(perfOut, "utf8"));
  const startup = metrics.startupMs ?? Number.POSITIVE_INFINITY;
  const frames = metrics.keyFrames ?? {};
  const p99 = frames.p99Ms ?? Number.POSITIVE_INFINITY;
  const count = frames.count ?? 0;

  console.log(
    `perf-cmd: startup ${startup} ms · key-to-frame p99 ${p99} ms (${count}) · idle ${idlePercent.toFixed(2)}%`,
  );

  const failures = [];
  if (startup > 3000) failures.push(`startup ${startup} ms > 3000 ms`);
  if (count < 5) failures.push(`solo ${count} muestras de tecla`);
  if (p99 > 150) failures.push(`key-to-frame p99 ${p99} ms > 150 ms (umbral de CI)`);
  if (idlePercent > 10) failures.push(`idle ${idlePercent.toFixed(2)}% > 10% (umbral de CI)`);

  if (failures.length > 0) {
    console.error("perf-cmd: umbrales incumplidos:");
    for (const failure of failures) console.error(`- ${failure}`);
    process.exitCode = 1;
  }
} finally {
  if (!child.killed) child.kill();
  const acode = join(root, "target", "debug", "acode");
  if (existsSync(acode)) {
    try {
      execFileSync(acode, ["host", "stop"], { env, stdio: "ignore" });
    } catch {
      // El host ya estaba detenido.
    }
  }
  rmSync(temporary, { recursive: true, force: true });
}
