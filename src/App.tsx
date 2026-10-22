import { useEffect, useState, useSyncExternalStore } from "react";
import AboutDialog from "./components/AboutDialog";
import AgentTabs from "./components/AgentTabs";
import AppearancePanel from "./components/AppearancePanel";
import ConversationPanel from "./components/ConversationPanel";
import ProjectActions from "./components/ProjectActions";
import ProjectTabs from "./components/ProjectTabs";
import ProjectTree from "./components/ProjectTree";
import Sidebar from "./components/Sidebar";
import SpecPanel from "./components/SpecPanel";
import TerminalView from "./components/TerminalView";
import WriterPanel from "./components/WriterPanel";
import { loadAppearance } from "./stores/appearanceStore";
import { getSnapshot as getProjects, subscribe as subscribeProjects } from "./stores/projectsStore";
import {
  getSnapshot as getSessions,
  restoreSessions,
  subscribe as subscribeSessions,
} from "./stores/sessionsStore";

export default function App() {
  const [aboutOpen, setAboutOpen] = useState(false);
  const [conversationOpen, setConversationOpen] = useState(false);
  const [specOpen, setSpecOpen] = useState(false);
  const [writerOpen, setWriterOpen] = useState(false);
  const [appearanceOpen, setAppearanceOpen] = useState(false);
  const projects = useSyncExternalStore(subscribeProjects, getProjects);
  const sessions = useSyncExternalStore(subscribeSessions, getSessions);
  const selected = projects.projects.find((project) => project.id === projects.selectedId);
  const active = sessions.sessions.find(
    (session) => session.id === sessions.activeId && session.projectId === projects.selectedId,
  );

  useEffect(() => {
    void loadAppearance(projects.selectedId ?? null);
  }, [projects.selectedId]);

  useEffect(() => {
    void restoreSessions();
  }, []);

  return (
    <main className="layout">
      <Sidebar />
      <section className="workspace">
        <header className="workspace-header">
          <span className="prompt">❯ AncletoCode</span>
          <span className="header-actions">
            {selected ? (
              <button
                type="button"
                onClick={() => setConversationOpen((open) => !open)}
                aria-pressed={conversationOpen}
              >
                conversación
              </button>
            ) : null}
            {selected ? (
              <button
                type="button"
                onClick={() => setSpecOpen((open) => !open)}
                aria-pressed={specOpen}
              >
                spec
              </button>
            ) : null}
            {selected ? (
              <button
                type="button"
                onClick={() => setWriterOpen((open) => !open)}
                aria-pressed={writerOpen}
              >
                writer
              </button>
            ) : null}
            <button
              type="button"
              onClick={() => setAppearanceOpen((open) => !open)}
              aria-pressed={appearanceOpen}
            >
              apariencia
            </button>
            <button type="button" className="about-trigger" onClick={() => setAboutOpen(true)}>
              About
            </button>
          </span>
        </header>
        <ProjectTabs />
        {selected ? (
          <>
            <p className="project-path">{selected.rootPath}</p>
            <ProjectActions projectId={selected.id} />
            {specOpen ? <SpecPanel projectId={selected.id} /> : null}
            {writerOpen ? <WriterPanel projectId={selected.id} /> : null}
            {appearanceOpen ? <AppearancePanel projectId={selected.id} /> : null}
            <AgentTabs project={selected} />
            {conversationOpen ? <ConversationPanel project={selected} /> : null}
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
