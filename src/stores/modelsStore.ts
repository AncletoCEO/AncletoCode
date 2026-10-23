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
  errors: Record<string, string>;
  load: (driverId: string, force?: boolean) => Promise<void>;
  reset: () => void;
};

export const useModelsStore = create<ModelsState>()((set, get) => ({
  byDriver: {},
  loading: null,
  error: null,
  errors: {},
  load: async (driverId, force = false) => {
    if (driverId === "shell" || (!force && (get().byDriver[driverId]?.length ?? 0) > 0)) {
      return;
    }
    set({ loading: driverId, error: null });
    try {
      const models = await invoke<DriverModel[]>("driver_models", { id: driverId });
      const errors = { ...get().errors };
      delete errors[driverId];
      set({
        byDriver: { ...get().byDriver, [driverId]: models },
        errors,
        loading: null,
        error: null,
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({
        loading: null,
        error: message,
        errors: { ...get().errors, [driverId]: message },
      });
    }
  },
  reset: () => set({ byDriver: {}, loading: null, error: null, errors: {} }),
}));
