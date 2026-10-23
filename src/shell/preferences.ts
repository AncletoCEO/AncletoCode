import { create } from "zustand";
import { persist } from "zustand/middleware";

export type Density = "comfortable" | "compact";

type ShellPreferences = {
  accent: string | null;
  density: Density;
  radius: number;
  fontScale: number;
  inspectorOpen: boolean;
  setAccent: (accent: string | null) => void;
  setDensity: (density: Density) => void;
  setRadius: (radius: number) => void;
  setFontScale: (fontScale: number) => void;
  setInspectorOpen: (open: boolean) => void;
  reset: () => void;
};

const defaults = {
  accent: null,
  density: "comfortable" as Density,
  radius: 10,
  fontScale: 1,
  inspectorOpen: true,
};

export const useShellPreferences = create<ShellPreferences>()(
  persist(
    (set) => ({
      ...defaults,
      setAccent: (accent) => set({ accent }),
      setDensity: (density) => set({ density }),
      setRadius: (radius) => set({ radius }),
      setFontScale: (fontScale) => set({ fontScale }),
      setInspectorOpen: (inspectorOpen) => set({ inspectorOpen }),
      reset: () => set(defaults),
    }),
    { name: "ancleto.shell.preferences" },
  ),
);
