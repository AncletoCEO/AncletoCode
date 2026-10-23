import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

export type TaskStatus = "backlog" | "inProgress" | "review" | "done" | "canceled";

export type Task = {
  id: string;
  projectId: string;
  title: string;
  status: TaskStatus;
  branch: string | null;
  worktreePath: string | null;
  sessionId: string | null;
  prUrl: string | null;
  createdAt: number;
  updatedAt: number;
};

type TasksState = {
  projectId: string | null;
  tasks: Task[];
  loading: boolean;
  error: string | null;
  load: (projectId: string) => Promise<void>;
  create: (title: string) => Promise<boolean>;
  update: (task: Task) => Promise<boolean>;
  setStatus: (task: Task, status: TaskStatus) => Promise<boolean>;
  start: (task: Task) => Promise<boolean>;
  remove: (id: string) => Promise<void>;
  reset: () => void;
};

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function replaceTask(tasks: Task[], next: Task): Task[] {
  return tasks.map((task) => (task.id === next.id ? next : task));
}

export const useTasksStore = create<TasksState>()((set, get) => ({
  projectId: null,
  tasks: [],
  loading: false,
  error: null,
  load: async (projectId) => {
    set({ projectId, loading: true, error: null });
    try {
      const tasks = await invoke<Task[]>("task_list", { id: projectId });
      set({ tasks, loading: false });
    } catch (error) {
      set({ tasks: [], loading: false, error: messageOf(error) });
    }
  },
  create: async (title) => {
    const { projectId } = get();
    if (!projectId || !title.trim()) {
      return false;
    }
    set({ error: null });
    try {
      const task = await invoke<Task>("task_create", { id: projectId, title: title.trim() });
      set({ tasks: [task, ...get().tasks] });
      return true;
    } catch (error) {
      set({ error: messageOf(error) });
      return false;
    }
  },
  update: async (task) => {
    set({ error: null });
    try {
      const next = await invoke<Task>("task_update", { task });
      set({ tasks: replaceTask(get().tasks, next) });
      return true;
    } catch (error) {
      set({ error: messageOf(error) });
      return false;
    }
  },
  setStatus: async (task, status) => {
    return get().update({ ...task, status });
  },
  start: async (task) => {
    set({ error: null });
    try {
      const next = await invoke<Task>("task_start", { id: task.id });
      set({ tasks: replaceTask(get().tasks, next) });
      return true;
    } catch (error) {
      set({ error: messageOf(error) });
      return false;
    }
  },
  remove: async (id) => {
    set({ error: null });
    try {
      await invoke("task_delete", { id });
      set({ tasks: get().tasks.filter((task) => task.id !== id) });
    } catch (error) {
      set({ error: messageOf(error) });
    }
  },
  reset: () => set({ projectId: null, tasks: [], loading: false, error: null }),
}));
