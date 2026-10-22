import { invoke } from "@tauri-apps/api/core";

export type Project = {
  id: string;
  name: string;
  rootPath: string;
  createdAt: number;
  lastOpenedAt: number;
};

export type FileNode = {
  name: string;
  path: string;
  isDir: boolean;
  children: FileNode[];
};

type ProjectsState = {
  projects: Project[];
  tree: FileNode | null;
  selectedId: string | null;
  loading: boolean;
  error: string | null;
};

const initialState: ProjectsState = {
  projects: [],
  tree: null,
  selectedId: null,
  loading: false,
  error: null,
};

let state: ProjectsState = initialState;
const listeners = new Set<() => void>();

function emit() {
  for (const listener of listeners) {
    listener();
  }
}

function update(patch: Partial<ProjectsState>) {
  state = { ...state, ...patch };
  emit();
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function getSnapshot(): ProjectsState {
  return state;
}

export function resetProjectsStore() {
  state = initialState;
  emit();
}

export async function loadProjects(): Promise<void> {
  update({ loading: true, error: null });
  try {
    const projects = await invoke<Project[]>("project_list");
    update({ projects, loading: false });
  } catch (error) {
    update({ loading: false, error: messageOf(error) });
  }
}

export async function addProject(path: string): Promise<boolean> {
  update({ error: null });
  try {
    await invoke("project_add", { path });
    const projects = await invoke<Project[]>("project_list");
    update({ projects });
    return true;
  } catch (error) {
    update({ error: messageOf(error) });
    return false;
  }
}

export async function removeProject(id: string): Promise<void> {
  update({ error: null });
  try {
    await invoke("project_remove", { id });
    const projects = await invoke<Project[]>("project_list");
    update({
      projects,
      selectedId: state.selectedId === id ? null : state.selectedId,
      tree: state.selectedId === id ? null : state.tree,
    });
  } catch (error) {
    update({ error: messageOf(error) });
  }
}

export async function selectProject(id: string): Promise<void> {
  update({ selectedId: id, error: null });
  try {
    await invoke<Project>("project_open", { id });
    const projects = await invoke<Project[]>("project_list");
    const tree = await invoke<FileNode>("project_tree", { id, limit: 5000 });
    update({ projects, tree });
  } catch (error) {
    update({ error: messageOf(error), tree: null });
  }
}
