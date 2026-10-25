import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const commandsPath = join(root, "src-tauri/src/commands/mod.rs");
const docsPath = join(root, "docs/performance.md");
const check = process.argv.includes("--check");

const blockingPrefixes = ["git_", "spec_", "writer_", "ide_", "ancleto_runtime_"];

const blockingCommands = new Set([
  "project_tree",
  "project_search",
  "task_list",
  "session_snapshot",
  "session_saved",
  "driver_models",
  "usage_report",
  "appearance_save_image",
  "writer_languages_status",
  "writer_languages_install",
]);

const blockingMarkers =
  /(\bCommand::new\b|\breqwest::|std::fs::|std::process::|\.spawn\(|\.output\(|read_to_string|write_all\(|\bstate\.(projects|orchestrator|sessions|host|drivers)\b|\bproject_root\b|\bStore::open\b|\.lock\(\)|\b(list_turns|list_projects|insert_|update_task|delete_task|add_turn|get_setting|set_setting)\b)/;

function extractBody(source, start) {
  let depth = 0;
  for (let index = start; index < source.length; index += 1) {
    const character = source[index];
    if (character === "{") {
      depth += 1;
    } else if (character === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(start + 1, index);
      }
    }
  }
  return "";
}

function commands(source) {
  const found = [];
  const pattern = /#\[tauri::command\][^\n]*\n\s*pub\s+(async\s+)?fn\s+(\w+)/g;
  for (const match of source.matchAll(pattern)) {
    const name = match[2];
    const signatureEnd = source.indexOf("{", match.index + match[0].length);
    found.push({
      name,
      isAsync: Boolean(match[1]),
      body: extractBody(source, signatureEnd),
      line: source.slice(0, match.index).split("\n").length,
    });
  }
  return found;
}

function allowlist() {
  const markdown = readFileSync(docsPath, "utf8");
  const entries = new Map();
  const row = /^\|\s*`([a-z0-9_]+)`\s*\|\s*(.+?)\s*\|\s*$/gm;
  for (const match of markdown.matchAll(row)) {
    entries.set(match[1], match[2]);
  }
  return entries;
}

const allowed = allowlist();
const parsed = commands(readFileSync(commandsPath, "utf8"));
const violations = [];

for (const command of parsed) {
  const looksBlocking =
    blockingPrefixes.some((prefix) => command.name.startsWith(prefix)) ||
    blockingCommands.has(command.name) ||
    blockingMarkers.test(command.body);
  if (!looksBlocking || command.isAsync) {
    continue;
  }
  if (allowed.has(command.name)) {
    continue;
  }
  violations.push(
    `src-tauri/src/commands/mod.rs:${command.line} ${command.name} es sincrónico y bloquea (spawn, red o IO); declaralo async + spawn_blocking o justificalo en la allowlist de docs/performance.md`,
  );
}

console.log(
  `check:blocking-commands ${parsed.length} comandos, ${parsed.filter((entry) => !entry.isAsync).length} sincrónicos, ${allowed.size} excepciones`,
);

if (violations.length > 0) {
  console.error(
    check
      ? "check:blocking-commands encontró comandos bloqueantes fuera de la allowlist:"
      : "check:blocking-commands (reporte) encontró comandos bloqueantes fuera de la allowlist:",
  );
  for (const violation of violations) {
    console.error(`- ${violation}`);
  }
  if (check) {
    process.exit(1);
  }
}

if (check) {
  console.log("check:blocking-commands OK");
}
