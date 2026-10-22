import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  addSession,
  closeSession,
  getSnapshot,
  killSession,
  renameSession,
  resetSessionsStore,
  restartSession,
  type SessionSpec,
} from "./sessionsStore";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);
const handlers = new Map<string, (event: { payload: unknown }) => void>();

const spec: SessionSpec = {
  projectId: "project-1",
  driverId: "claude",
  program: "claude",
  args: [],
  cwd: "/tmp/demo",
  env: [],
  cols: 80,
  rows: 24,
};

function info(id: string, status = "ready") {
  return {
    id,
    projectId: spec.projectId,
    driverId: spec.driverId,
    status,
    pid: 123,
  };
}

describe("sessionsStore", () => {
  beforeEach(() => {
    resetSessionsStore();
    invokeMock.mockReset();
    handlers.clear();
    listenMock.mockReset();
    listenMock.mockImplementation((async (
      event: string,
      handler: (event: { payload: unknown }) => void,
    ) => {
      handlers.set(event, handler);
      return () => {
        handlers.delete(event);
      };
    }) as never);
  });

  it("crea una sesión y la activa", async () => {
    invokeMock.mockResolvedValueOnce(info("session-1"));

    const id = await addSession(spec, "Claude Code");

    expect(id).toBe("session-1");
    expect(invokeMock).toHaveBeenCalledWith("session_create", { spec });
    expect(getSnapshot().sessions[0]?.title).toBe("Claude Code");
    expect(getSnapshot().activeId).toBe("session-1");
  });

  it("renombra una tab", async () => {
    invokeMock.mockResolvedValueOnce(info("session-1"));
    await addSession(spec, "Claude Code");

    renameSession("session-1", "  refactor  ");

    expect(getSnapshot().sessions[0]?.title).toBe("refactor");
  });

  it("mata la sesión y conserva la tab detenida", async () => {
    invokeMock.mockResolvedValueOnce(info("session-1"));
    await addSession(spec, "Claude Code");
    invokeMock.mockResolvedValueOnce(undefined);

    await killSession("session-1");

    expect(invokeMock).toHaveBeenLastCalledWith("session_kill", { id: "session-1" });
    expect(getSnapshot().sessions[0]?.status).toBe("stopped");
  });

  it("cierra la tab activa y libera la selección", async () => {
    invokeMock.mockResolvedValueOnce(info("session-1"));
    await addSession(spec, "Claude Code");
    invokeMock.mockResolvedValueOnce(undefined);

    await closeSession("session-1");

    expect(getSnapshot().sessions).toHaveLength(0);
    expect(getSnapshot().activeId).toBeNull();
  });

  it("reinicia una sesión con una tab nueva", async () => {
    invokeMock.mockResolvedValueOnce(info("session-1"));
    await addSession(spec, "Claude Code");
    invokeMock.mockResolvedValueOnce(undefined).mockResolvedValueOnce(info("session-2"));

    await restartSession("session-1");

    expect(getSnapshot().sessions).toHaveLength(1);
    expect(getSnapshot().sessions[0]?.id).toBe("session-2");
  });

  it("actualiza el estado con los eventos de sesión", async () => {
    invokeMock.mockResolvedValueOnce(info("session-1"));
    await addSession(spec, "Claude Code");

    handlers.get("session://state/session-1")?.({ payload: "busy" });

    expect(getSnapshot().sessions[0]?.status).toBe("busy");
  });
});
