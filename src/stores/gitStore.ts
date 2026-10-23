import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

export type GitFile = {
  path: string;
  status: string;
  staged: boolean;
};

export type GitStatus = {
  isRepo: boolean;
  branch: string | null;
  upstream: string | null;
  ahead: number;
  behind: number;
  dirty: boolean;
  files: GitFile[];
};

export type GitBranch = {
  name: string;
  current: boolean;
  upstream: string | null;
};

export type GitWorktree = {
  path: string;
  branch: string | null;
  head: string | null;
  isMain: boolean;
};

type GitState = {
  projectId: string | null;
  status: GitStatus | null;
  branches: GitBranch[];
  worktrees: GitWorktree[];
  loading: boolean;
  error: string | null;
  diff: { path: string; staged: boolean; text: string } | null;
  load: (projectId: string) => Promise<void>;
  createBranch: (name: string, checkout: boolean) => Promise<boolean>;
  checkout: (name: string) => Promise<boolean>;
  addWorktree: (branch: string) => Promise<boolean>;
  removeWorktree: (path: string) => Promise<boolean>;
  openDiff: (path: string, staged: boolean) => Promise<void>;
  closeDiff: () => void;
  stage: (path: string) => Promise<void>;
  unstage: (path: string) => Promise<void>;
  commit: (message: string) => Promise<boolean>;
  reset: () => void;
};

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export const useGitStore = create<GitState>()((set, get) => ({
  projectId: null,
  status: null,
  branches: [],
  worktrees: [],
  loading: false,
  error: null,
  diff: null,
  load: async (projectId) => {
    set({ projectId, loading: true, error: null });
    try {
      const [status, branches, worktrees] = await Promise.all([
        invoke<GitStatus>("git_status", { id: projectId }),
        invoke<GitBranch[]>("git_branches", { id: projectId }),
        invoke<GitWorktree[]>("git_worktrees", { id: projectId }),
      ]);
      set({ status, branches, worktrees, loading: false });
    } catch (error) {
      set({ status: null, branches: [], loading: false, error: messageOf(error) });
    }
  },
  createBranch: async (name, checkout) => {
    const { projectId } = get();
    if (!projectId) {
      return false;
    }
    set({ error: null });
    try {
      const status = await invoke<GitStatus>("git_create_branch", {
        id: projectId,
        name,
        checkout,
      });
      const branches = await invoke<GitBranch[]>("git_branches", { id: projectId });
      set({ status, branches });
      return true;
    } catch (error) {
      set({ error: messageOf(error) });
      return false;
    }
  },
  checkout: async (name) => {
    const { projectId } = get();
    if (!projectId) {
      return false;
    }
    set({ error: null });
    try {
      const status = await invoke<GitStatus>("git_checkout", { id: projectId, name });
      const branches = await invoke<GitBranch[]>("git_branches", { id: projectId });
      set({ status, branches });
      return true;
    } catch (error) {
      set({ error: messageOf(error) });
      return false;
    }
  },
  addWorktree: async (branch) => {
    const { projectId } = get();
    if (!projectId) {
      return false;
    }
    set({ error: null });
    try {
      await invoke<GitWorktree>("git_worktree_add", { id: projectId, branch });
      await get().load(projectId);
      return true;
    } catch (error) {
      set({ error: messageOf(error) });
      return false;
    }
  },
  removeWorktree: async (path) => {
    const { projectId } = get();
    if (!projectId) {
      return false;
    }
    set({ error: null });
    try {
      await invoke("git_worktree_remove", { id: projectId, path, force: false });
      await get().load(projectId);
      return true;
    } catch (error) {
      set({ error: messageOf(error) });
      return false;
    }
  },
  openDiff: async (path, staged) => {
    const { projectId } = get();
    if (!projectId) {
      return;
    }
    set({ error: null });
    try {
      const text = await invoke<string>("git_diff", { id: projectId, path, staged });
      set({ diff: { path, staged, text } });
    } catch (error) {
      set({ error: messageOf(error) });
    }
  },
  closeDiff: () => set({ diff: null }),
  stage: async (path) => {
    const { projectId } = get();
    if (!projectId) {
      return;
    }
    set({ error: null });
    try {
      await invoke("git_stage", { id: projectId, paths: [path] });
      await get().load(projectId);
    } catch (error) {
      set({ error: messageOf(error) });
    }
  },
  unstage: async (path) => {
    const { projectId } = get();
    if (!projectId) {
      return;
    }
    set({ error: null });
    try {
      await invoke("git_unstage", { id: projectId, paths: [path] });
      await get().load(projectId);
    } catch (error) {
      set({ error: messageOf(error) });
    }
  },
  commit: async (message) => {
    const { projectId } = get();
    if (!projectId) {
      return false;
    }
    set({ error: null });
    try {
      const status = await invoke<GitStatus>("git_commit", { id: projectId, message });
      const branches = await invoke<GitBranch[]>("git_branches", { id: projectId });
      set({ status, branches, diff: null });
      return true;
    } catch (error) {
      set({ error: messageOf(error) });
      return false;
    }
  },
  reset: () =>
    set({ projectId: null, status: null, branches: [], loading: false, error: null, diff: null }),
}));
