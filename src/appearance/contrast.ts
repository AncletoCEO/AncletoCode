import { contrast, parseHex, toHex } from "../../third_party/pandacode/src/color.ts";
import type { ThemeManifest } from "./types";

export function ratio(foreground: string, background: string): number {
  const fg = parseHex(foreground);
  const bg = parseHex(background);
  if (!fg || !bg) {
    return 0;
  }
  return contrast(fg.rgb, bg.rgb);
}

export function blend(foreground: string, background: string, amount: number): string {
  const fg = parseHex(foreground);
  const bg = parseHex(background);
  if (!fg || !bg) {
    return background;
  }
  return toHex([
    fg.rgb[0] + (bg.rgb[0] - fg.rgb[0]) * amount,
    fg.rgb[1] + (bg.rgb[1] - fg.rgb[1]) * amount,
    fg.rgb[2] + (bg.rgb[2] - fg.rgb[2]) * amount,
  ]);
}

export function worstContrast(theme: ThemeManifest): { pair: string; value: number } {
  const checks: [string, number][] = [
    ["texto/fondo", ratio(theme.tokens.fg, theme.tokens.bgBase)],
    ["texto/panel", ratio(theme.tokens.fg, theme.tokens.bgPanel)],
    ["acento/fondo", ratio(theme.tokens.accent, theme.tokens.bgBase)],
  ];
  return checks.reduce<{ pair: string; value: number }>(
    (worst, [pair, value]) => (value < worst.value ? { pair, value } : worst),
    { pair: "—", value: Number.POSITIVE_INFINITY },
  );
}
