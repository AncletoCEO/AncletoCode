import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { parseHex, triplet } from "../../third_party/pandacode/src/color.ts";
import { resolveAppearance, parseTheme, serializeTheme } from "../appearance/resolve";
import { defaultOpacity, themeCatalog } from "../appearance/resolve";
import type {
  AppearanceData,
  AppearanceOverride,
  AppearanceScope,
  BackgroundSettings,
  OpacitySettings,
  ResolvedAppearance,
  ThemeManifest,
  ThemeTokens,
} from "../appearance/types";

const THEMES_KEY = "appearance.themes";
const GLOBAL_KEY = "appearance.global";
const WORKSPACE_KEY = "appearance.workspace.default";
const projectKey = (id: string) => `appearance.project.${id}`;

type AppearanceState = {
  data: AppearanceData;
  projectId: string | null;
  loaded: boolean;
  error: string | null;
  resolved: ResolvedAppearance;
};

const emptyData: AppearanceData = {
  global: {},
  workspace: {},
  project: {},
  customThemes: [],
};

let state: AppearanceState = {
  data: emptyData,
  projectId: null,
  loaded: false,
  error: null,
  resolved: resolveAppearance(emptyData, null),
};

const listeners = new Set<() => void>();

function emit() {
  for (const listener of listeners) {
    listener();
  }
}

function recompute(patch: Partial<AppearanceState>) {
  state = { ...state, ...patch };
  state = { ...state, resolved: resolveAppearance(state.data, state.projectId) };
  applyAppearance(state.resolved);
  emit();
}

export function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function getSnapshot(): AppearanceState {
  return state;
}

export function resetAppearanceStore() {
  state = {
    data: emptyData,
    projectId: null,
    loaded: false,
    error: null,
    resolved: resolveAppearance(emptyData, null),
  };
  emit();
}

function parseOverride(text: string | null): AppearanceOverride {
  if (!text) {
    return {};
  }
  try {
    const value: unknown = JSON.parse(text);
    return value && typeof value === "object" ? (value as AppearanceOverride) : {};
  } catch {
    return {};
  }
}

function parseThemes(text: string | null): ThemeManifest[] {
  if (!text) {
    return [];
  }
  try {
    const value: unknown = JSON.parse(text);
    return Array.isArray(value)
      ? value.filter(
          (entry): entry is ThemeManifest =>
            Boolean(entry) && typeof entry === "object" && "id" in entry && "tokens" in entry,
        )
      : [];
  } catch {
    return [];
  }
}

export async function loadAppearance(projectId: string | null): Promise<void> {
  try {
    const [global, workspace, project, themes] = await Promise.all([
      invoke<string | null>("settings_get", { key: GLOBAL_KEY }),
      invoke<string | null>("settings_get", { key: WORKSPACE_KEY }),
      projectId
        ? invoke<string | null>("settings_get", { key: projectKey(projectId) })
        : Promise.resolve(null),
      invoke<string | null>("settings_get", { key: THEMES_KEY }),
    ]);
    const data: AppearanceData = {
      global: parseOverride(global),
      workspace: parseOverride(workspace),
      project: parseOverride(project),
      customThemes: parseThemes(themes),
    };
    recompute({ data, projectId, loaded: true, error: null });
  } catch (error) {
    recompute({ error: messageOf(error), loaded: true });
  }
}

export function catalog(): ThemeManifest[] {
  return themeCatalog(state.data.customThemes);
}

export function activeTheme(): ThemeManifest {
  return state.resolved.theme;
}

function overrideFor(scope: AppearanceScope): AppearanceOverride {
  return scope === "project" ? state.data.project : state.data.global;
}

async function persistScope(scope: AppearanceScope): Promise<void> {
  const key = scope === "project" ? projectKey(state.projectId ?? "") : GLOBAL_KEY;
  if (scope === "project" && !state.projectId) {
    return;
  }
  try {
    await invoke("settings_put", { key, value: JSON.stringify(overrideFor(scope)) });
  } catch (error) {
    recompute({ error: messageOf(error) });
  }
}

async function persistThemes(): Promise<void> {
  try {
    await invoke("settings_put", {
      key: THEMES_KEY,
      value: JSON.stringify(state.data.customThemes),
    });
  } catch (error) {
    recompute({ error: messageOf(error) });
  }
}

export async function setTheme(themeId: string, scope: AppearanceScope): Promise<void> {
  const data: AppearanceData =
    scope === "project"
      ? { ...state.data, project: { ...state.data.project, themeId } }
      : { ...state.data, global: { ...state.data.global, themeId } };
  recompute({ data });
  await persistScope(scope);
}

export async function setBackground(
  patch: Partial<BackgroundSettings>,
  scope: AppearanceScope,
): Promise<void> {
  const current = overrideFor(scope);
  const data: AppearanceData =
    scope === "project"
      ? { ...state.data, project: { ...current, background: { ...current.background, ...patch } } }
      : { ...state.data, global: { ...current, background: { ...current.background, ...patch } } };
  recompute({ data });
  await persistScope(scope);
}

export async function setOpacity(
  patch: Partial<OpacitySettings>,
  scope: AppearanceScope,
): Promise<void> {
  const current = overrideFor(scope);
  const data: AppearanceData =
    scope === "project"
      ? { ...state.data, project: { ...current, opacity: { ...current.opacity, ...patch } } }
      : { ...state.data, global: { ...current, opacity: { ...current.opacity, ...patch } } };
  recompute({ data });
  await persistScope(scope);
}

export async function addCustomTheme(theme: ThemeManifest): Promise<void> {
  const data = { ...state.data, customThemes: [...state.data.customThemes, theme] };
  recompute({ data });
  await persistThemes();
}

export async function duplicateTheme(sourceId: string): Promise<string | null> {
  const source = catalog().find((theme) => theme.id === sourceId);
  if (!source) {
    return null;
  }
  const copy: ThemeManifest = {
    ...source,
    id: `${source.id}-copy-${Date.now()}`,
    name: `${source.name} copia`,
    tokens: { ...source.tokens },
    ansi: [...source.ansi],
  };
  await addCustomTheme(copy);
  await setTheme(copy.id, "global");
  return copy.id;
}

export async function removeCustomTheme(id: string): Promise<void> {
  const data = {
    ...state.data,
    customThemes: state.data.customThemes.filter((theme) => theme.id !== id),
  };
  recompute({ data });
  if (state.data.global.themeId === id) {
    await setTheme("ancleto-dark", "global");
  }
  await persistThemes();
}

export async function updateThemeToken(
  id: string,
  token: keyof ThemeTokens,
  value: string,
): Promise<void> {
  let themeId = id;
  let custom = state.data.customThemes.find((theme) => theme.id === id);
  if (!custom) {
    const created = await duplicateTheme(id);
    if (!created) {
      return;
    }
    themeId = created;
    custom = state.data.customThemes.find((theme) => theme.id === created);
  }
  if (!custom) {
    return;
  }
  const data = {
    ...state.data,
    customThemes: state.data.customThemes.map((theme) =>
      theme.id === themeId ? { ...theme, tokens: { ...theme.tokens, [token]: value } } : theme,
    ),
  };
  recompute({ data });
  await persistThemes();
}

export async function importTheme(text: string): Promise<void> {
  const theme = parseTheme(text);
  const exists = catalog().some((entry) => entry.id === theme.id);
  const imported: ThemeManifest = exists ? { ...theme, id: `${theme.id}-${Date.now()}` } : theme;
  await addCustomTheme(imported);
  await setTheme(imported.id, "global");
}

export function exportTheme(id: string): string | null {
  const theme = catalog().find((entry) => entry.id === id);
  return theme ? serializeTheme(theme) : null;
}

export function terminalTheme(resolved: ResolvedAppearance) {
  const { tokens, ansi } = resolved.theme;
  const names = [
    "black",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "cyan",
    "white",
    "brightBlack",
    "brightRed",
    "brightGreen",
    "brightYellow",
    "brightBlue",
    "brightMagenta",
    "brightCyan",
    "brightWhite",
  ] as const;
  const palette: Record<string, string> = {};
  names.forEach((name, index) => {
    const color = ansi[index];
    if (color) {
      palette[name] = color;
    }
  });
  return {
    ...palette,
    background: "transparent",
    foreground: tokens.fg,
    cursor: tokens.cursor,
    selectionBackground: tokens.selection,
  };
}

function applyAppearance(resolved: ResolvedAppearance) {
  if (typeof document === "undefined") {
    return;
  }
  const root = document.documentElement;
  const { tokens } = resolved.theme;
  const variables: Record<string, string> = {
    "--bg-base": tokens.bgBase,
    "--bg-panel": tokens.bgPanel,
    "--bg-elevated": tokens.bgElevated,
    "--fg": tokens.fg,
    "--fg-muted": tokens.fgMuted,
    "--accent": tokens.accent,
    "--accent-alt": tokens.accentAlt,
    "--success": tokens.success,
    "--warning": tokens.warning,
    "--danger": tokens.danger,
    "--border": tokens.border,
    "--selection": tokens.selection,
    "--cursor": tokens.cursor,
    "--bg-base-rgb": rgb(tokens.bgBase),
    "--bg-panel-rgb": rgb(tokens.bgPanel),
    "--bg-elevated-rgb": rgb(tokens.bgElevated),
  };
  for (const [name, value] of Object.entries(variables)) {
    root.style.setProperty(name, value);
  }

  const opacity = resolved.opacity.opaque
    ? { ...defaultOpacity, window: 1, panels: 1, terminal: 1 }
    : resolved.opacity;
  root.style.setProperty("--window-opacity", String(opacity.window));
  root.style.setProperty("--panel-opacity", String(opacity.panels));
  root.style.setProperty("--terminal-opacity", String(opacity.terminal));
  root.style.setProperty("--panel-blur", opacity.opaque ? "0px" : "12px");
  root.dataset.opaque = opacity.opaque ? "true" : "false";

  const background = resolved.background;
  const dim = background.kind === "image" ? Math.max(background.dim, 0.25) : background.dim;
  const dimLayer = `linear-gradient(rgb(0 0 0 / ${dim}), rgb(0 0 0 / ${dim}))`;
  let image = dimLayer;
  let size = "auto";
  if (background.kind === "gradient") {
    image = `${dimLayer}, linear-gradient(${background.angle}deg, ${background.gradientFrom}, ${background.gradientTo})`;
  } else if (background.kind === "image" && background.imagePath) {
    image = `${dimLayer}, url("${convertFileSrc(background.imagePath)}")`;
    size = background.fit === "tile" ? "auto" : background.fit;
  } else if (background.kind === "pattern" && background.pattern === "grid") {
    image = `${dimLayer}, repeating-linear-gradient(0deg, ${tokens.border} 0 1px, transparent 1px 24px), repeating-linear-gradient(90deg, ${tokens.border} 0 1px, transparent 1px 24px)`;
  } else if (background.kind === "pattern" && background.pattern === "scanlines") {
    image = `${dimLayer}, repeating-linear-gradient(0deg, rgb(0 0 0 / 40%) 0 1px, transparent 1px 4px)`;
  }
  root.style.setProperty("--app-background", image);
  root.style.setProperty("--app-background-size", size);
  root.style.setProperty("--app-background-color", background.color || tokens.bgBase);
  root.style.setProperty("--app-background-blur", `${background.blur}px`);

  void applyNativeEffect(resolved);
}

async function applyNativeEffect(resolved: ResolvedAppearance) {
  const requested = resolved.opacity.effect;
  let effect = requested;
  if (requested === "auto") {
    const platform = navigator.userAgent;
    effect = platform.includes("Windows") ? "mica" : platform.includes("Mac") ? "vibrancy" : "none";
  }
  if (effect === "none") {
    await invoke("appearance_native_effect", { effect: "none" }).catch(() => undefined);
    return;
  }
  await invoke("appearance_native_effect", {
    effect,
    dark: resolved.theme.kind === "dark",
  }).catch(() => undefined);
}

function rgb(hex: string): string {
  const parsed = parseHex(hex);
  if (!parsed) {
    return "0 0 0";
  }
  return triplet(parsed.rgb);
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
