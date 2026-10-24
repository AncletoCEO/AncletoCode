import { describe, expect, it } from "vitest";
import { freshnessLabel, isFresh, mergeProvider, USAGE_TTL_SECS } from "./usage";
import type { ProviderUsage } from "../stores/usageStore";

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

describe("usage helpers", () => {
  it("considera fresco solo lo que está dentro del TTL", () => {
    const nowMs = 1_700_000_000_000;
    const nowSecs = nowMs / 1000;
    expect(isFresh(0, nowMs)).toBe(false);
    expect(isFresh(nowSecs - USAGE_TTL_SECS + 1, nowMs)).toBe(true);
    expect(isFresh(nowSecs - USAGE_TTL_SECS, nowMs)).toBe(false);
  });

  it("reemplaza un proveedor existente sin moverlo de lugar", () => {
    const list = [provider("opencode"), provider("claude")];
    const next = mergeProvider(list, provider("opencode", 9));
    expect(next.map((entry) => entry.provider)).toEqual(["opencode", "claude"]);
    expect(next[0]?.cost).toBe(9);
    expect(list[0]?.cost).toBeNull();
  });

  it("agrega proveedores nuevos al final", () => {
    const next = mergeProvider([provider("opencode")], provider("claude"));
    expect(next.map((entry) => entry.provider)).toEqual(["opencode", "claude"]);
  });

  it("es idempotente ante eventos duplicados", () => {
    const once = mergeProvider([], provider("agy"));
    const twice = mergeProvider(once, provider("agy"));
    expect(twice).toHaveLength(1);
  });

  it("describe la frescura en lenguaje natural", () => {
    const nowMs = 1_700_000_000_000;
    const nowSecs = nowMs / 1000;
    expect(freshnessLabel(0, nowMs)).toBe("sin datos");
    expect(freshnessLabel(nowSecs - 5, nowMs)).toBe("actualizado hace instantes");
    expect(freshnessLabel(nowSecs - 120, nowMs)).toBe("actualizado hace 2 min");
    expect(freshnessLabel(nowSecs - 7200, nowMs)).toBe("actualizado hace 2 h");
  });
});
