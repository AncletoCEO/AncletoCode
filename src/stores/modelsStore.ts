import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

export type DriverModel = {
  id: string;
  label: string;
};

type ModelsState = {
  byDriver: Record<string, DriverModel[]>;
  loading: string | null;
  error: string | null;
  load: (driverId: string, force?: boolean) => Promise<void>;
  reset: () => void;
};

export const useModelsStore = create<ModelsState>()((set, get) => ({
  byDriver: {},
  loading: null,
  error: null,
  load: async (driverId, force = false) => {
    if (driverId === "shell" || (!force && get().byDriver[driverId])) {
      return;
    }
    set({ loading: driverId, error: null });
    try {
      const models = await invoke<DriverModel[]>("driver_models", { id: driverId });
      set({
        byDriver: { ...get().byDriver, [driverId]: models },
        loading: null,
      });
    } catch (error) {
      set({
        loading: null,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  },
  reset: () => set({ byDriver: {}, loading: null, error: null }),
}));
