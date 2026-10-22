export type ThemeKind = "light" | "dark";

export type ThemeTokens = {
  bgBase: string;
  bgPanel: string;
  bgElevated: string;
  fg: string;
  fgMuted: string;
  accent: string;
  accentAlt: string;
  success: string;
  warning: string;
  danger: string;
  border: string;
  selection: string;
  cursor: string;
};

export type ThemeManifest = {
  id: string;
  name: string;
  kind: ThemeKind;
  tokens: ThemeTokens;
  ansi: string[];
};

export type BackgroundKind = "solid" | "gradient" | "image" | "pattern";
export type BackgroundFit = "cover" | "contain" | "tile";
export type BackgroundPattern = "none" | "grid" | "scanlines";

export type BackgroundSettings = {
  kind: BackgroundKind;
  color: string;
  gradientFrom: string;
  gradientTo: string;
  angle: number;
  imagePath: string | null;
  fit: BackgroundFit;
  blur: number;
  dim: number;
  pattern: BackgroundPattern;
};

export type WindowEffect = "auto" | "none" | "mica" | "acrylic" | "blur" | "vibrancy";

export type OpacitySettings = {
  window: number;
  panels: number;
  terminal: number;
  opaque: boolean;
  effect: WindowEffect;
};

export type AppearanceOverride = {
  themeId?: string;
  background?: Partial<BackgroundSettings>;
  opacity?: Partial<OpacitySettings>;
};

export type AppearanceScope = "global" | "project";

export type AppearanceData = {
  global: AppearanceOverride;
  workspace: AppearanceOverride;
  project: AppearanceOverride;
  customThemes: ThemeManifest[];
};

export type ResolvedAppearance = {
  theme: ThemeManifest;
  background: BackgroundSettings;
  opacity: OpacitySettings;
};
