import { useState, useSyncExternalStore } from "react";
import AboutDialog from "./components/AboutDialog";
import AgentTabs from "./components/AgentTabs";
import ProjectTabs from "./components/ProjectTabs";
import ProjectTree from "./components/ProjectTree";
import Sidebar from "./components/Sidebar";
import TerminalView from "./components/TerminalView";
import { getSnapshot as getProjects, subscribe as subscribeProjects } from "./stores/projectsStore";
import { getSnapshot as getSessions, subscribe as subscribeSessions } from "./stores/sessionsStore";

export default function App() {
  const [aboutOpen, setAboutOpen] = useState(false);
  const projects = useSyncExternalStore(subscribeProjects, getProjects);
  const sessions = useSyncExternalStore(subscribeSessions, getSessions);
  const selected = projects.projects.find((project) => project.id === projects.selectedId);
  const active = sessions.sessions.find(
    (session) => session.id === sessions.activeId && session.projectId === projects.selectedId,
  );

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
        <ProjectTabs />
        {selected ? (
          <>
            <p className="project-path">{selected.rootPath}</p>
            <AgentTabs project={selected} />
            {active ? (
              <TerminalView key={active.id} sessionId={active.id} />
            ) : projects.tree ? (
              <ul className="tree">
                <ProjectTree node={projects.tree} />
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
