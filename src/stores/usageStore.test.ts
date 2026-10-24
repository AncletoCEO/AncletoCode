import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ProviderUsage, UsageSnapshot } from "./usageStore";

const invoke = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => undefined),
}));

import { useUsageStore } from "./usageStore";

function provider(id: string, cost: number | null = null): ProviderUsage {
  return {
    provider: id,
    label: id,
    status: "ready",
    message: null,
    source: null,
    cost,
    tokens: null,
    sessions: null,
    details: [],
    windows: [],
    checkedAt: 1,
  };
}

function snapshot(overrides: Partial<UsageSnapshot> = {}): UsageSnapshot {
  return {
    providers: [provider("opencode", 3)],
    checkedAt: Math.floor(Date.now() / 1000),
    stale: false,
    refreshing: false,
    durationMs: 120,
    ...overrides,
  };
}

beforeEach(() => {
  invoke.mockReset();
  useUsageStore.setState({
    providers: [],
    checkedAt: 0,
    stale: true,
    refreshing: false,
    durationMs: null,
    error: null,
  });
});

describe("usageStore", () => {
  it("no vuelve a consultar si el snapshot está fresco", async () => {
    useUsageStore.setState({
      providers: [provider("opencode")],
      checkedAt: Math.floor(Date.now() / 1000),
      stale: false,
    });
    await useUsageStore.getState().load(false);
    expect(invoke).not.toHaveBeenCalled();
  });

  it("consulta y aplica el snapshot cuando no hay datos", async () => {
    invoke.mockResolvedValue(snapshot());
    await useUsageStore.getState().load(false);
    expect(invoke).toHaveBeenCalledWith("usage_report", { force: false });
    const state = useUsageStore.getState();
    expect(state.providers).toHaveLength(1);
    expect(state.checkedAt).toBeGreaterThan(0);
    expect(state.durationMs).toBe(120);
  });

  it("fuerza el refresco con load(true)", async () => {
    useUsageStore.setState({
      providers: [provider("opencode")],
      checkedAt: Math.floor(Date.now() / 1000),
      stale: false,
    });
    invoke.mockResolvedValue(snapshot({ refreshing: true }));
    await useUsageStore.getState().load(true);
    expect(invoke).toHaveBeenCalledWith("usage_report", { force: true });
    expect(useUsageStore.getState().refreshing).toBe(true);
  });

  it("deduplica cargas concurrentes", async () => {
    let resolve: (value: UsageSnapshot) => void = () => undefined;
    invoke.mockReturnValue(
      new Promise<UsageSnapshot>((done) => {
        resolve = done;
      }),
    );
    const first = useUsageStore.getState().load(false);
    const second = useUsageStore.getState().load(false);
    resolve(snapshot());
    await Promise.all([first, second]);
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("hace merge por proveedor e ignora duplicados", () => {
    useUsageStore.getState().applyProvider(provider("opencode", 1));
    useUsageStore.getState().applyProvider(provider("opencode", 7));
    useUsageStore.getState().applyProvider(provider("claude", 2));
    const providers = useUsageStore.getState().providers;
    expect(providers.map((entry) => entry.provider)).toEqual(["opencode", "claude"]);
    expect(providers[0]?.cost).toBe(7);
  });

  it("aplica el snapshot final del evento done", () => {
    useUsageStore.setState({ refreshing: true });
    useUsageStore.getState().applyDone(snapshot({ refreshing: false }));
    const state = useUsageStore.getState();
    expect(state.refreshing).toBe(false);
    expect(state.providers).toHaveLength(1);
  });

  it("expone el error sin borrar los datos previos", async () => {
    useUsageStore.setState({ providers: [provider("opencode")], stale: true });
    invoke.mockRejectedValue(new Error("sin red"));
    await useUsageStore.getState().load(true);
    const state = useUsageStore.getState();
    expect(state.error).toBe("sin red");
    expect(state.providers).toHaveLength(1);
  });
});
