import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { worstContrast } from "./contrast";
import { defaultOpacity, parseTheme, resolveAppearance, serializeTheme } from "./resolve";
import { presetThemes } from "./themes";
import type { AppearanceData } from "./types";
import {
  getSnapshot,
  loadAppearance,
  resetAppearanceStore,
  setTheme,
} from "../stores/appearanceStore";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => path),
}));

const invokeMock = vi.mocked(invoke);

const empty: AppearanceData = {
  global: {},
  workspace: {},
  project: {},
  customThemes: [],
};

describe("appearance", () => {
  beforeEach(() => {
    resetAppearanceStore();
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(null);
  });

  it("resuelve tema y opacidad con precedencia global → workspace → proyecto", () => {
    const data: AppearanceData = {
      global: { themeId: "carbon", opacity: { panels: 0.8 } },
      workspace: { opacity: { panels: 0.85 } },
      project: { themeId: "dracula", opacity: { terminal: 0.7 } },
      customThemes: [],
    };

    const resolved = resolveAppearance(data, "project-1");

    expect(resolved.theme.id).toBe("dracula");
    expect(resolved.opacity.panels).toBe(0.85);
    expect(resolved.opacity.terminal).toBe(0.7);
    expect(resolved.opacity.window).toBe(defaultOpacity.window);
  });

  it("usa el tema por defecto y el fondo del tema cuando no hay overrides", () => {
    const resolved = resolveAppearance(empty, null);

    expect(resolved.theme.id).toBe("ancleto-dark");
    expect(resolved.background.gradientFrom).toBe(resolved.theme.tokens.bgBase);
    expect(resolved.background.gradientTo).toBe(resolved.theme.tokens.bgPanel);
  });

  it("todos los presets cumplen contraste AA para texto y acento", () => {
    for (const theme of presetThemes) {
      const worst = worstContrast(theme);
      expect(worst.value, `${theme.name}: ${worst.pair}`).toBeGreaterThanOrEqual(4.5);
    }
  });

  it("exporta e importa un tema sin perder tokens", () => {
    const theme = presetThemes[0]!;
    const parsed = parseTheme(serializeTheme(theme));

    expect(parsed).toEqual(theme);
    expect(() => parseTheme("{}")).toThrow();
  });

  it("persiste el tema por proyecto y lo aplica en vivo", async () => {
    await loadAppearance("project-1");
    invokeMock.mockClear();

    await setTheme("carbon", "project");

    const put = invokeMock.mock.calls.find(([command]) => command === "settings_put");
    const args = put?.[1] as { key: string; value: string } | undefined;
    expect(args).toMatchObject({ key: "appearance.project.project-1" });
    expect(args?.value).toContain("carbon");
    expect(getSnapshot().resolved.theme.id).toBe("carbon");
    expect(document.documentElement.style.getPropertyValue("--bg-base")).toBe("#161616");
  });
});
