import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

describe("paquetes npm", () => {
  for (const name of ["acode", "acmd"]) {
    it(`${name} publica licencias y la atribución de fork`, () => {
      const directory = join(root, "packages", name);
      const manifest = JSON.parse(readFileSync(join(directory, "package.json"), "utf8"));

      expect(manifest.license).toBe("AGPL-3.0-only");
      expect(manifest.files).toContain("LICENSE");
      expect(manifest.files).toContain("THIRD_PARTY_NOTICES.md");
      expect(manifest.files).toContain("README.md");
      expect(existsSync(join(directory, "LICENSE"))).toBe(true);
      expect(existsSync(join(directory, "THIRD_PARTY_NOTICES.md"))).toBe(true);
      expect(readFileSync(join(directory, "THIRD_PARTY_NOTICES.md"), "utf8")).toContain(
        "PandaCode",
      );
      expect(readFileSync(join(directory, "README.md"), "utf8")).toContain(
        "This is a fork of PandaCode",
      );
    });
  }
});
