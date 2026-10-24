import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { create } from "zustand";
import { isFresh, mergeProvider } from "../lib/usage";

export type UsageMetric = {
  label: string;
  value: string;
};

export type UsageWindow = {
  label: string;
  usedPercent: number;
  resetsAt: string | null;
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
  windows: UsageWindow[];
  checkedAt: number;
  durationMs?: number | null;
};

export type UsageSnapshot = {
  providers: ProviderUsage[];
  checkedAt: number;
  stale: boolean;
  refreshing: boolean;
  durationMs: number | null;
};

type UsageProviderEvent = {
  kind: "provider";
  index: number;
  usage: ProviderUsage;
};

type UsageDoneEvent = {
  kind: "done";
  snapshot: UsageSnapshot;
};

type UsageState = {
  providers: ProviderUsage[];
  checkedAt: number;
  stale: boolean;
  refreshing: boolean;
  durationMs: number | null;
  error: string | null;
  load: (force?: boolean) => Promise<void>;
  applyProvider: (usage: ProviderUsage) => void;
  applyDone: (snapshot: UsageSnapshot) => void;
  reset: () => void;
};

const emptyState = {
  providers: [] as ProviderUsage[],
  checkedAt: 0,
  stale: true,
  refreshing: false,
  durationMs: null as number | null,
  error: null as string | null,
};

let inflight: Promise<void> | null = null;
let listeners: UnlistenFn[] | null = null;
let starting: Promise<void> | null = null;
let references = 0;

export const useUsageStore = create<UsageState>()((set, get) => ({
  ...emptyState,
  load: async (force = false) => {
    const state = get();
    if (
      !force &&
      state.providers.length > 0 &&
      !state.stale &&
      isFresh(state.checkedAt, Date.now())
    ) {
      return;
    }
    if (inflight) {
      await inflight;
      return;
    }
    inflight = (async () => {
      set({ error: null });
      try {
        const snapshot = await invoke<UsageSnapshot>("usage_report", { force });
        set({
          providers: snapshot.providers,
          checkedAt: snapshot.checkedAt,
          stale: snapshot.stale,
          refreshing: snapshot.refreshing,
          durationMs: snapshot.durationMs,
          error: null,
        });
      } catch (error) {
        set({ error: error instanceof Error ? error.message : String(error) });
      } finally {
        inflight = null;
      }
    })();
    await inflight;
  },
  applyProvider: (usage) => {
    set((state) => ({ providers: mergeProvider(state.providers, usage) }));
  },
  applyDone: (snapshot) => {
    set({
      providers: snapshot.providers,
      checkedAt: snapshot.checkedAt,
      stale: snapshot.stale,
      refreshing: snapshot.refreshing,
      durationMs: snapshot.durationMs,
    });
  },
  reset: () => set({ ...emptyState }),
}));

export async function acquireUsageEvents(): Promise<() => void> {
  references += 1;
  if (!starting) {
    starting = Promise.all([
      listen<UsageProviderEvent>("usage://provider", (event) => {
        useUsageStore.getState().applyProvider(event.payload.usage);
      }),
      listen<UsageDoneEvent>("usage://done", (event) => {
        useUsageStore.getState().applyDone(event.payload.snapshot);
      }),
    ])
      .then((unlisten) => {
        listeners = unlisten;
      })
      .catch(() => {
        starting = null;
      });
  }
  await starting;

  let released = false;
  return () => {
    if (released) {
      return;
    }
    released = true;
    references = Math.max(0, references - 1);
    if (references === 0 && listeners) {
      listeners.forEach((unlisten) => unlisten());
      listeners = null;
      starting = null;
    }
  };
}
