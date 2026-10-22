import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type SessionStatus =
  "starting" | "ready" | "busy" | "waiting_input" | "detached" | "stopped" | "crashed";

export type SessionSpec = {
  projectId: string;
  driverId: string;
  program: string;
  args: string[];
  cwd: string;
  env: [string, string][];
  cols: number;
  rows: number;
};

export type Session = {
  id: string;
  projectId: string;
  driverId: string;
  title: string;
  status: SessionStatus;
  pid: number | null;
  spec: SessionSpec;
};

type SessionInfo = {
  id: string;
  projectId: string;
  driverId: string;
  status: SessionStatus;
  pid: number | null;
};

type SessionsState = {
  sessions: Session[];
  activeId: string | null;
  error: string | null;
};

const initialState: SessionsState = {
  sessions: [],
  activeId: null,
  error: null,
};

let state: SessionsState = initialState;
const listeners = new Set<() => void>();
const unlisteners = new Map<string, UnlistenFn[]>();

function emit() {
  for (const listener of listeners) {
    listener();
  }
}

function update(patch: Partial<SessionsState>) {
  state = { ...state, ...patch };
  emit();
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function setStatus(id: string, status: SessionStatus) {
  update({
    sessions: state.sessions.map((session) =>
      session.id === id ? { ...session, status } : session,
    ),
  });
}

async function attachListeners(id: string): Promise<void> {
  const stopState = await listen<SessionStatus>(`session://state/${id}`, (event) => {
    setStatus(id, event.payload);
  });
  const stopExit = await listen(`session://exit/${id}`, () => {
    setStatus(id, "stopped");
  });
  unlisteners.set(id, [stopState, stopExit]);
}

function detachListeners(id: string) {
  for (const stop of unlisteners.get(id) ?? []) {
    stop();
  }
  unlisteners.delete(id);
}

export function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function getSnapshot(): SessionsState {
  return state;
}

export function resetSessionsStore() {
  for (const id of [...unlisteners.keys()]) {
    detachListeners(id);
  }
  state = initialState;
  emit();
}

export function setActiveSession(id: string | null) {
  update({ activeId: id });
}

export async function addSession(spec: SessionSpec, title: string): Promise<string | null> {
  update({ error: null });
  try {
    const info = await invoke<SessionInfo>("session_create", { spec });
    const session: Session = {
      id: info.id,
      projectId: info.projectId,
      driverId: info.driverId,
      title,
      status: info.status,
      pid: info.pid,
      spec,
    };
    update({ sessions: [...state.sessions, session], activeId: info.id });
    await attachListeners(info.id);
    return info.id;
  } catch (error) {
    update({ error: messageOf(error) });
    return null;
  }
}

export function renameSession(id: string, title: string) {
  const trimmed = title.trim();
  if (!trimmed) {
    return;
  }
  update({
    sessions: state.sessions.map((session) =>
      session.id === id ? { ...session, title: trimmed } : session,
    ),
  });
}

export async function killSession(id: string): Promise<void> {
  try {
    await invoke("session_kill", { id });
    setStatus(id, "stopped");
  } catch (error) {
    update({ error: messageOf(error) });
  }
}

export async function closeSession(id: string): Promise<void> {
  const session = state.sessions.find((entry) => entry.id === id);
  if (!session) {
    return;
  }
  if (session.status !== "stopped" && session.status !== "crashed") {
    await killSession(id);
  }
  detachListeners(id);
  const sessions = state.sessions.filter((entry) => entry.id !== id);
  update({
    sessions,
    activeId: state.activeId === id ? (sessions.at(-1)?.id ?? null) : state.activeId,
  });
}

export async function restartSession(id: string): Promise<void> {
  const session = state.sessions.find((entry) => entry.id === id);
  if (!session) {
    return;
  }
  await closeSession(id);
  await addSession(session.spec, session.title);
}
