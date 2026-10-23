import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

export type AncletoRuntimeStatus = {
  activeVersion: string | null;
  activePath: string | null;
  bundledVersion: string | null;
  nodePath: string | null;
  latestVersion: string | null;
  updateAvailable: boolean;
  error: string | null;
};

type AncletoRuntimeState = {
  status: AncletoRuntimeStatus | null;
  loading: boolean;
  updating: boolean;
  error: string | null;
  load: (checkRemote?: boolean) => Promise<void>;
  update: () => Promise<boolean>;
};

export const useAncletoRuntimeStore = create<AncletoRuntimeState>()((set) => ({
  status: null,
  loading: false,
  updating: false,
  error: null,
  load: async (checkRemote = false) => {
    set({ loading: true, error: null });
    try {
      const status = await invoke<AncletoRuntimeStatus>("ancleto_runtime_status", {
        checkRemote,
      });
      set({ status, loading: false, error: status.error });
    } catch (error) {
      set({
        loading: false,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  },
  update: async () => {
    set({ updating: true, error: null });
    try {
      const status = await invoke<AncletoRuntimeStatus>("ancleto_runtime_update");
      set({ status, updating: false, error: status.error });
      return true;
    } catch (error) {
      set({
        updating: false,
        error: error instanceof Error ? error.message : String(error),
      });
      return false;
    }
  },
}));
