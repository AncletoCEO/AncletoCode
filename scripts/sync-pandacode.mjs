import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const SHA = "e9f89891fd23518848765aef8219dbf39feac6e0";
const BASE = `https://raw.githubusercontent.com/aicodeking/pandacode/${SHA}`;

const files = [
  "src-tauri/src/providers/windows_job.rs",
  "src-tauri/src/providers/text_stream.rs",
  "src/color.ts",
  "src/palette-core.ts",
  "src/transcript-scroll.ts",
  "src/vendor/enriched-markdown/md4c.js",
  "src/vendor/enriched-markdown/md4c.d.ts",
];

function stripHeader(text) {
  const lines = text
    .replace(/\r\n/g, "\n")
    .split("\n")
    .filter((line) => line !== "// @ts-nocheck");
  if (lines[0]?.startsWith("// SPDX-License-Identifier")) {
    let index = 0;
    while (index < lines.length && lines[index].startsWith("//")) {
      index += 1;
    }
    while (lines[index] === "") {
      index += 1;
    }
    return lines.slice(index).join("\n");
  }
  if (lines[0]?.startsWith("/* SPDX-License-Identifier")) {
    let index = 0;
    while (index < lines.length && !lines[index].includes("*/")) {
      index += 1;
    }
    index += 1;
    if (lines[index]?.startsWith("// @ts-")) {
      index += 1;
    }
    while (lines[index] === "") {
      index += 1;
    }
    return lines.slice(index).join("\n");
  }
  return text;
}

function reportDiff(path, upstream, local) {
  const expected = stripHeader(upstream).split("\n");
  const actual = stripHeader(local).split("\n");
  const max = Math.max(expected.length, actual.length);
  let shown = 0;

  console.log(`\nDIFIEREN: ${path}`);
  for (let index = 0; index < max && shown < 20; index += 1) {
    const left = expected[index];
    const right = actual[index];
    if (left !== right) {
      console.log(`  línea ${index + 1}`);
      console.log(`  - ${left ?? "<sin línea>"}`);
      console.log(`  + ${right ?? "<sin línea>"}`);
      shown += 1;
    }
  }
}

let differences = 0;

for (const path of files) {
  const localPath = join(root, "third_party", "pandacode", path);
  const local = readFileSync(localPath, "utf8");
  const response = await fetch(`${BASE}/${path}`);

  if (!response.ok) {
    console.error(`No se pudo descargar ${path}: HTTP ${response.status}`);
    differences += 1;
    continue;
  }

  const upstream = await response.text();
  if (stripHeader(upstream).trimEnd() !== stripHeader(local).trimEnd()) {
    reportDiff(path, upstream, local);
    differences += 1;
  } else {
    console.log(`OK: ${path}`);
  }
}

if (differences > 0) {
  console.error(`\n${differences} archivo(s) difieren del commit ${SHA}`);
  process.exit(1);
}

console.log(`\nTodos los archivos coinciden con el commit ${SHA}`);
