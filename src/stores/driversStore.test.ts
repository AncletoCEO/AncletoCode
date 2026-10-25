import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { getSnapshot, loadDrivers, resetDriversStore } from "./driversStore";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

const driver = {
  id: "claude",
  displayName: "Claude Code",
  binary: "claude",
  status: "available" as const,
  version: "1.0.0",
  binaryPath: "/usr/bin/claude",
  installHint: null,
  capabilities: {
    interactivePty: true,
    oneShotPrompt: true,
    structuredOutput: true,
    resumeSession: true,
    headlessJson: true,
  },
};

describe("driversStore", () => {
  beforeEach(() => {
    resetDriversStore();
    invokeMock.mockReset();
  });

  it("carga el estado de los drivers", async () => {
    invokeMock.mockResolvedValueOnce([driver]);

    await loadDrivers();

    expect(invokeMock).toHaveBeenCalledWith("drivers_list");
    expect(getSnapshot().drivers).toEqual([driver]);
  });

  it("expone el error cuando la detección falla", async () => {
    invokeMock.mockRejectedValueOnce(new Error("sin acceso"));

    await loadDrivers();

    expect(getSnapshot().error).toContain("sin acceso");
  });

  it("deduplica cargas concurrentes", async () => {
    let resolve: (drivers: (typeof driver)[]) => void = () => undefined;
    invokeMock.mockImplementation(
      () =>
        new Promise<(typeof driver)[]>((done) => {
          resolve = done;
        }),
    );

    const first = loadDrivers();
    const second = loadDrivers();
    resolve([driver]);
    await Promise.all([first, second]);

    expect(invokeMock).toHaveBeenCalledTimes(1);
    expect(getSnapshot().drivers).toEqual([driver]);
  });
});
