import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ProviderUsage, UsageSnapshot } from "../stores/usageStore";

const invoke = vi.hoisted(() => vi.fn());
const listeners = vi.hoisted(() => new Map<string, (event: { payload: unknown }) => void>());

vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (name: string, handler: (event: { payload: unknown }) => void) => {
    listeners.set(name, handler);
    return () => listeners.delete(name);
  }),
}));

import UsageView from "./UsageView";
import { useUsageStore } from "../stores/usageStore";

function provider(id: string, label: string, cost: number | null = null): ProviderUsage {
  return {
    provider: id,
    label,
    status: "ready",
    message: null,
    source: null,
    cost,
    tokens: null,
    sessions: null,
    details: [],
    windows: [],
    checkedAt: Math.floor(Date.now() / 1000),
  };
}

function snapshot(overrides: Partial<UsageSnapshot> = {}): UsageSnapshot {
  return {
    providers: [],
    checkedAt: 0,
    stale: true,
    refreshing: true,
    durationMs: null,
    ...overrides,
  };
}

function emit(name: string, payload: unknown) {
  const handler = listeners.get(name);
  if (!handler) {
    throw new Error(`sin listener para ${name}`);
  }
  act(() => handler({ payload }));
}

afterEach(() => {
  cleanup();
});

beforeEach(() => {
  listeners.clear();
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

describe("UsageView", () => {
  it("muestra skeletons y deshabilita Actualizar durante el primer refresco", async () => {
    invoke.mockResolvedValue(snapshot());
    render(<UsageView />);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("usage_report", { force: false }));
    expect(await screen.findAllByText("consultando…")).toHaveLength(4);
    expect(screen.getByRole("button", { name: /actualizando/i })).toBeDisabled();
  });

  it("pinta cada proveedor al llegar su evento y cierra el refresco con done", async () => {
    invoke.mockResolvedValue(snapshot());
    render(<UsageView />);
    await screen.findAllByText("consultando…");

    emit("usage://provider", {
      kind: "provider",
      index: 0,
      usage: provider("opencode", "OpenCode", 12.5),
    });
    expect(screen.getByText("OpenCode")).toBeInTheDocument();
    expect(screen.getByText("$12.50")).toBeInTheDocument();
    expect(screen.getAllByText("consultando…")).toHaveLength(3);

    emit("usage://done", {
      kind: "done",
      snapshot: snapshot({
        providers: [provider("opencode", "OpenCode", 12.5)],
        checkedAt: Math.floor(Date.now() / 1000),
        stale: false,
        refreshing: false,
        durationMs: 900,
      }),
    });
    expect(screen.queryByText("consultando…")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /actualizar/i })).toBeEnabled();
  });

  it("sigue mostrando datos vencidos y completa los faltantes con skeletons", async () => {
    invoke.mockResolvedValue(
      snapshot({
        providers: [provider("opencode", "OpenCode", 3)],
        checkedAt: Math.floor(Date.now() / 1000) - 3600,
        stale: true,
        refreshing: true,
      }),
    );
    render(<UsageView />);
    expect(await screen.findByText("OpenCode")).toBeInTheDocument();
    expect(await screen.findByText(/datos vencidos/i)).toBeInTheDocument();
    expect(screen.getAllByText("consultando…")).toHaveLength(3);
  });

  it("no consulta de nuevo cuando el snapshot está fresco", async () => {
    useUsageStore.setState({
      providers: [provider("opencode", "OpenCode")],
      checkedAt: Math.floor(Date.now() / 1000),
      stale: false,
      refreshing: false,
    });
    render(<UsageView />);
    await waitFor(() => expect(screen.getByText("OpenCode")).toBeInTheDocument());
    expect(invoke).not.toHaveBeenCalled();
  });

  it("fuerza el refresco desde el botón Actualizar", async () => {
    invoke.mockResolvedValue(
      snapshot({
        providers: [provider("opencode", "OpenCode")],
        checkedAt: Math.floor(Date.now() / 1000),
        stale: false,
        refreshing: false,
      }),
    );
    render(<UsageView />);
    const button = await screen.findByRole("button", { name: /actualizar/i });
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(1));
    fireEvent.click(button);
    await waitFor(() => expect(invoke).toHaveBeenLastCalledWith("usage_report", { force: true }));
  });

  it("muestra el error sin descartar los proveedores ya pintados", async () => {
    invoke.mockRejectedValue(new Error("sin red"));
    render(<UsageView />);
    expect(await screen.findByText("sin red")).toBeInTheDocument();
  });
});
