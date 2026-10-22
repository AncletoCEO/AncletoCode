import { useState, useSyncExternalStore } from "react";
import AboutDialog from "./components/AboutDialog";
import ProjectTree from "./components/ProjectTree";
import Sidebar from "./components/Sidebar";
import { getSnapshot, subscribe } from "./stores/projectsStore";

export default function App() {
  const [aboutOpen, setAboutOpen] = useState(false);
  const state = useSyncExternalStore(subscribe, getSnapshot);
  const selected = state.projects.find((project) => project.id === state.selectedId);

  return (
    <main className="layout">
      <Sidebar />
      <section className="workspace">
        <header className="workspace-header">
          <span className="prompt">❯ AncletoCode</span>
          <button type="button" className="about-trigger" onClick={() => setAboutOpen(true)}>
            About
          </button>
        </header>
        {selected ? (
          <>
            <p className="project-path">{selected.rootPath}</p>
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
