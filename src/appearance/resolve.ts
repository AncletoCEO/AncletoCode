import { presetThemes } from "./themes";
import type {
  AppearanceData,
  AppearanceOverride,
  BackgroundSettings,
  OpacitySettings,
  ResolvedAppearance,
  ThemeManifest,
} from "./types";

export const defaultBackground: BackgroundSettings = {
  kind: "solid",
  color: "",
  gradientFrom: "#0d1117",
  gradientTo: "#161b22",
  angle: 135,
  imagePath: null,
  fit: "cover",
  blur: 0,
  dim: 0.25,
  pattern: "none",
};

export const defaultOpacity: OpacitySettings = {
  window: 1,
  panels: 0.94,
  terminal: 0.98,
  opaque: false,
  effect: "auto",
};

export function themeCatalog(customThemes: ThemeManifest[]): ThemeManifest[] {
  return [...presetThemes, ...customThemes];
}

function merged(source: AppearanceOverride | undefined): AppearanceOverride {
  return source ?? {};
}

export function resolveAppearance(
  data: AppearanceData,
  projectId: string | null,
): ResolvedAppearance {
  void projectId;
  const chain = [merged(data.global), merged(data.workspace), merged(data.project)];
  const themeId = [...chain].reverse().find((entry) => entry.themeId)?.themeId;
  const catalog = themeCatalog(data.customThemes);
  const theme = catalog.find((entry) => entry.id === themeId) ?? presetThemes[0]!;

  const background = chain.reduce<BackgroundSettings>(
    (current, entry) => ({ ...current, ...(entry.background ?? {}) }),
    { ...defaultBackground, gradientFrom: theme.tokens.bgBase, gradientTo: theme.tokens.bgPanel },
  );

  const opacity = chain.reduce<OpacitySettings>(
    (current, entry) => ({ ...current, ...(entry.opacity ?? {}) }),
    { ...defaultOpacity },
  );

  return { theme, background, opacity };
}

export function serializeTheme(theme: ThemeManifest): string {
  return JSON.stringify(theme, null, 2);
}

export function parseTheme(text: string): ThemeManifest {
  const value: unknown = JSON.parse(text);
  if (!value || typeof value !== "object") {
    throw new Error("theme.json inválido");
  }
  const candidate = value as Partial<ThemeManifest>;
  if (
    typeof candidate.id !== "string" ||
    typeof candidate.name !== "string" ||
    (candidate.kind !== "light" && candidate.kind !== "dark") ||
    !candidate.tokens ||
    !Array.isArray(candidate.ansi) ||
    candidate.ansi.length < 16
  ) {
    throw new Error("theme.json no tiene el formato esperado");
  }
  return candidate as ThemeManifest;
}
