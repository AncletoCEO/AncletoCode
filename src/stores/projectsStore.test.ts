import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  addProject,
  getSnapshot,
  loadProjects,
  resetProjectsStore,
  selectProject,
} from "./projectsStore";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

const project = {
  id: "project-1",
  name: "demo",
  rootPath: "/tmp/demo",
  createdAt: 1,
  lastOpenedAt: 1,
};

const tree = {
  name: "demo",
  path: "/tmp/demo",
  isDir: true,
  children: [],
};

describe("projectsStore", () => {
  beforeEach(() => {
    resetProjectsStore();
    invokeMock.mockReset();
  });

  it("carga la lista de proyectos", async () => {
    invokeMock.mockResolvedValueOnce([project]);

    await loadProjects();

    expect(invokeMock).toHaveBeenCalledWith("project_list");
    expect(getSnapshot().projects).toEqual([project]);
    expect(getSnapshot().error).toBeNull();
  });

  it("agrega un proyecto y recarga la lista", async () => {
    invokeMock.mockResolvedValueOnce(null).mockResolvedValueOnce([project]);

    const added = await addProject("/tmp/demo");

    expect(added).toBe(true);
    expect(invokeMock).toHaveBeenNthCalledWith(1, "project_add", { path: "/tmp/demo" });
    expect(getSnapshot().projects).toEqual([project]);
  });

  it("expone el error cuando la operación falla", async () => {
    invokeMock.mockRejectedValueOnce(new Error("sin permisos"));

    await loadProjects();

    expect(getSnapshot().error).toContain("sin permisos");
  });

  it("abre un proyecto y carga su árbol", async () => {
    invokeMock
      .mockResolvedValueOnce(project)
      .mockResolvedValueOnce([project])
      .mockResolvedValueOnce(tree);

    await selectProject("project-1");

    expect(invokeMock).toHaveBeenNthCalledWith(1, "project_open", { id: "project-1" });
    expect(invokeMock).toHaveBeenNthCalledWith(3, "project_tree", {
      id: "project-1",
      limit: 5000,
    });
    expect(getSnapshot().selectedId).toBe("project-1");
    expect(getSnapshot().tree).toEqual(tree);
  });
});
