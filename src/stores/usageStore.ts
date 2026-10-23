import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

export type UsageMetric = {
  label: string;
  value: string;
};

export type ProviderUsage = {
  provider: string;
  label: string;
  status: "ready" | "unavailable" | "unsupported" | "signedOut" | "error";
  message: string | null;
  source: string | null;
  cost: number | null;
  tokens: number | null;
  sessions: number | null;
  details: UsageMetric[];
  checkedAt: number;
};

type UsageState = {
  providers: ProviderUsage[];
  loading: boolean;
  error: string | null;
  load: () => Promise<void>;
  reset: () => void;
};

export const useUsageStore = create<UsageState>()((set) => ({
  providers: [],
  loading: false,
  error: null,
  load: async () => {
    set({ loading: true, error: null });
    try {
      const providers = await invoke<ProviderUsage[]>("usage_report");
      set({ providers, loading: false });
    } catch (error) {
      set({
        loading: false,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  },
  reset: () => set({ providers: [], loading: false, error: null }),
}));
