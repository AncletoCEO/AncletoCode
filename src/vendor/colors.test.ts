import { describe, expect, it } from "vitest";
import { contrast, onColor, parseHex, type Rgb } from "../../third_party/pandacode/src/color.ts";
import { accentTokens } from "../../third_party/pandacode/src/palette-core.ts";

const black: Rgb = [0, 0, 0];
const white: Rgb = [255, 255, 255];

function tokenRgb(tokens: Record<string, string>, name: string): Rgb {
  const parsed = parseHex(tokens[name] ?? "");
  if (!parsed) {
    throw new Error(`token sin color: ${name}`);
  }
  return parsed.rgb;
}

describe("color", () => {
  it("parsea hex de 3, 6 y 8 dígitos", () => {
    expect(parseHex("#369")?.rgb).toEqual([51, 102, 153]);
    expect(parseHex("#336699")?.rgb).toEqual([51, 102, 153]);
    expect(parseHex("#33669980")?.alpha).toBeCloseTo(128 / 255, 5);
    expect(parseHex("nope")).toBeUndefined();
  });

  it("calcula el contraste WCAG entre negro y blanco", () => {
    expect(contrast(black, white)).toBeCloseTo(21, 0);
    expect(contrast(white, white)).toBeCloseTo(1, 5);
  });

  it("elige texto legible sobre cualquier relleno", () => {
    const fills: Rgb[] = [black, white, [51, 102, 153], [250, 204, 21], [220, 38, 38]];
    for (const fill of fills) {
      expect(contrast(onColor(fill), fill)).toBeGreaterThanOrEqual(4.5);
    }
  });
});

describe("accentTokens", () => {
  it("deriva acento y texto con contraste AA en modo oscuro y claro", () => {
    for (const mode of ["dark", "light"] as const) {
      const tokens = accentTokens([51, 102, 153], mode);
      const accent = tokenRgb(tokens, "--accent");
      const onAccent = tokenRgb(tokens, "--on-accent");
      expect(tokens["--accent-rgb"]).toMatch(/^\d+ \d+ \d+$/);
      expect(contrast(accent, onAccent)).toBeGreaterThanOrEqual(4.5);
    }
  });
});
