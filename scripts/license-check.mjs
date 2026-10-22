import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const failures = [];

function fail(message) {
  failures.push(message);
}

function requireFile(relativePath) {
  const absolutePath = join(root, relativePath);
  if (!existsSync(absolutePath)) {
    fail(`Falta el archivo obligatorio: ${relativePath}`);
    return "";
  }
  return readFileSync(absolutePath, "utf8");
}

function requireText(relativePath, text, needles) {
  for (const needle of needles) {
    if (!text.includes(needle)) {
      fail(`Falta ${JSON.stringify(needle)} en ${relativePath}`);
    }
  }
}

const requiredFiles = [
  "LICENSE",
  "THIRD_PARTY_NOTICES.md",
  "docs/provenance.md",
  "docs/assets-inventory.md",
  "third_party/pandacode/LICENSE",
  "third_party/pandacode/UPSTREAM.md",
];

for (const file of requiredFiles) {
  requireFile(file);
}

const forkDeclaration = "This is a fork of PandaCode";
const pandacodeUrl = "https://github.com/aicodeking/pandacode";

requireText("README.md", requireFile("README.md"), [
  "## Licencias y atribuciones",
  forkDeclaration,
  pandacodeUrl,
]);

requireText("src/components/AboutDialog.tsx", requireFile("src/components/AboutDialog.tsx"), [
  forkDeclaration,
  pandacodeUrl,
]);

requireText("THIRD_PARTY_NOTICES.md", requireFile("THIRD_PARTY_NOTICES.md"), [
  "PandaCode",
  "Software Mansion",
]);

const provenance = requireFile("docs/provenance.md");
const upstream = requireFile("third_party/pandacode/UPSTREAM.md");
const pandacodeRoot = join(root, "third_party", "pandacode");

function walk(directory) {
  const entries = [];
  for (const name of readdirSync(directory)) {
    const absolutePath = join(directory, name);
    if (statSync(absolutePath).isDirectory()) {
      entries.push(...walk(absolutePath));
    } else {
      entries.push(absolutePath);
    }
  }
  return entries;
}

const headerFields = [
  "SPDX-License-Identifier: MIT",
  "SPDX-FileCopyrightText: 2026 PandaCode contributors",
  "Source: https://github.com/aicodeking/pandacode/blob/",
  "Modified by AncletoCode:",
];

if (existsSync(pandacodeRoot)) {
  const metadataFiles = new Set(["LICENSE", "UPSTREAM.md"]);

  for (const absolutePath of walk(pandacodeRoot)) {
    const relativeToPandacode = relative(pandacodeRoot, absolutePath);
    if (metadataFiles.has(relativeToPandacode)) {
      continue;
    }

    const localPath = `third_party/pandacode/${relativeToPandacode.split(sep).join("/")}`;
    const contents = readFileSync(absolutePath, "utf8");
    const header = contents.split(/\r?\n/).slice(0, 8).join("\n");

    for (const field of headerFields) {
      if (!header.includes(field)) {
        fail(`Falta la cabecera SPDX ${JSON.stringify(field)} en ${localPath}`);
      }
    }

    if (!provenance.includes(localPath)) {
      fail(`Falta la entrada de ledger para ${localPath} en docs/provenance.md`);
    }

    if (!upstream.includes(localPath)) {
      fail(`Falta ${localPath} en third_party/pandacode/UPSTREAM.md`);
    }
  }
}

if (failures.length > 0) {
  console.error("license:check encontró problemas:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log("license:check OK");
