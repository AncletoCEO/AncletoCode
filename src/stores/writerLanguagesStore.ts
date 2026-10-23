import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

export type WriterLanguagesStatus = {
  installed: boolean;
  version: string | null;
  path: string | null;
  message: string | null;
};

type WriterLanguagesState = {
  status: WriterLanguagesStatus | null;
  loading: boolean;
  installing: boolean;
  error: string | null;
  load: () => Promise<void>;
  install: () => Promise<void>;
};

export const useWriterLanguagesStore = create<WriterLanguagesState>()((set) => ({
  status: null,
  loading: false,
  installing: false,
  error: null,
  load: async () => {
    set({ loading: true, error: null });
    try {
      const status = await invoke<WriterLanguagesStatus>("writer_languages_status");
      set({ status, loading: false });
    } catch (error) {
      set({
        loading: false,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  },
  install: async () => {
    set({ installing: true, error: null });
    try {
      const status = await invoke<WriterLanguagesStatus>("writer_languages_install");
      set({ status, installing: false });
    } catch (error) {
      set({
        installing: false,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  },
}));
