import { useState, useSyncExternalStore } from "react";
import { invoke } from "@tauri-apps/api/core";
import AboutDialog from "./components/AboutDialog";
import ProjectTree from "./components/ProjectTree";
import Sidebar from "./components/Sidebar";
import TerminalView from "./components/TerminalView";
import { defaultShell } from "./lib/shell";
import { getSnapshot, subscribe } from "./stores/projectsStore";

type SessionInfo = {
  id: string;
};

export default function App() {
  const [aboutOpen, setAboutOpen] = useState(false);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const state = useSyncExternalStore(subscribe, getSnapshot);
  const selected = state.projects.find((project) => project.id === state.selectedId);

  async function openTerminal() {
    if (!selected || sessionId) {
      return;
    }
    const shell = defaultShell();
    const info = await invoke<SessionInfo>("session_create", {
      spec: {
        projectId: selected.id,
        driverId: "shell",
        program: shell.program,
        args: shell.args,
        cwd: selected.rootPath,
        env: [],
        cols: 80,
        rows: 24,
      },
    });
    setSessionId(info.id);
  }

  async function closeTerminal() {
    if (!sessionId) {
      return;
    }
    await invoke("session_kill", { id: sessionId });
    setSessionId(null);
  }

  return (
    <main className="layout">
      <Sidebar />
      <section className="workspace">
        <header className="workspace-header">
          <span className="prompt">❯ AncletoCode</span>
          <span className="actions">
            {selected ? (
              <button
                type="button"
                onClick={() => void openTerminal()}
                disabled={sessionId !== null}
              >
                terminal
              </button>
            ) : null}
            {sessionId ? (
              <button type="button" onClick={() => void closeTerminal()}>
                cerrar terminal
              </button>
            ) : null}
            <button type="button" className="about-trigger" onClick={() => setAboutOpen(true)}>
              About
            </button>
          </span>
        </header>
        {selected ? (
          <>
            <p className="project-path">{selected.rootPath}</p>
            {sessionId ? <TerminalView sessionId={sessionId} /> : null}
            {state.tree ? (
              <ul className="tree">
                <ProjectTree node={state.tree} />
              </ul>
            ) : (
              <p className="muted">cargando árbol…</p>
            )}
          </>
        ) : (
          <p className="muted">Agregá un proyecto para empezar.</p>
        )}
      </section>
      {aboutOpen ? <AboutDialog onClose={() => setAboutOpen(false)} /> : null}
    </main>
  );
}
