import { invoke } from "@tauri-apps/api/core";

export type ToolStatus = "available" | "missing" | "incompatible" | "error";

export type Capabilities = {
  interactivePty: boolean;
  oneShotPrompt: boolean;
  structuredOutput: boolean;
  resumeSession: boolean;
  headlessJson: boolean;
};

export type DriverStatus = {
  id: string;
  displayName: string;
  binary: string;
  status: ToolStatus;
  version: string | null;
  binaryPath: string | null;
  installHint: string | null;
  capabilities: Capabilities;
};

type DriversState = {
  drivers: DriverStatus[];
  loading: boolean;
  error: string | null;
};

const initialState: DriversState = {
  drivers: [],
  loading: false,
  error: null,
};

let state: DriversState = initialState;
const listeners = new Set<() => void>();

function emit() {
  for (const listener of listeners) {
    listener();
  }
}

function update(patch: Partial<DriversState>) {
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

export function getSnapshot(): DriversState {
  return state;
}

export function resetDriversStore() {
  state = initialState;
  emit();
}

let inflight: Promise<void> | null = null;

export async function loadDrivers(): Promise<void> {
  if (inflight) {
    return inflight;
  }
  inflight = (async () => {
    update({ loading: true, error: null });
    try {
      const drivers = await invoke<DriverStatus[]>("drivers_list");
      update({ drivers, loading: false });
    } catch (error) {
      update({ loading: false, error: messageOf(error) });
    } finally {
      inflight = null;
    }
  })();
  return inflight;
}
