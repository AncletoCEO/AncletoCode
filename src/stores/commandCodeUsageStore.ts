import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { CommandCodeUsage } from "../lib/command-code-usage";

type State = {
  usage: CommandCodeUsage | null;
  raw: string | null;
  save: (raw: string, usage: CommandCodeUsage) => void;
  clear: () => void;
};

export const useCommandCodeUsageStore = create<State>()(
  persist(
    (set) => ({
      usage: null,
      raw: null,
      save: (raw, usage) => set({ raw, usage }),
      clear: () => set({ raw: null, usage: null }),
    }),
    { name: "ancleto.command-code-usage" },
  ),
);
