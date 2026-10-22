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
import { invoke } from "@tauri-apps/api/core";
import { catalog, getSnapshot as getAppearance, setTheme } from "./stores/appearanceStore";
import { loadAppearance } from "./stores/appearanceStore";
import {
  decreaseUiScale,
  increaseUiScale,
  loadUiScale,
  resetUiScale,
  setUiScale,
} from "./lib/ui-size";
import {
  getSnapshot as getProjects,
  selectProject,
  subscribe as subscribeProjects,
} from "./stores/projectsStore";
import {
  addSession,
  getSnapshot as getSessions,
  killSession,
  restoreSessions,
  subscribe as subscribeSessions,
} from "./stores/sessionsStore";
import { defaultShell } from "./lib/shell";

type FileEntry = {
  name: string;
  path: string;
  isDir: boolean;
};

export default function App() {
  const [aboutOpen, setAboutOpen] = useState(false);
  const [conversationOpen, setConversationOpen] = useState(false);
  const [specOpen, setSpecOpen] = useState(false);
  const [writerOpen, setWriterOpen] = useState(false);
  const [appearanceOpen, setAppearanceOpen] = useState(false);
  const [uiScale, setUiScaleValue] = useState(1);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<FileEntry[]>([]);
  const [searchError, setSearchError] = useState<string | null>(null);
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

  useEffect(() => {
    void loadUiScale().then(setUiScaleValue);
  }, []);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (!event.ctrlKey && !event.metaKey) {
        return;
      }
      const key = event.key.toLowerCase();
      const selected = projects.projects.find((project) => project.id === projects.selectedId);
      if (event.shiftKey && event.altKey && key === "l") {
        event.preventDefault();
        const themes = catalog();
        const current = themes.findIndex((theme) => theme.id === appearanceThemeId());
        const next = themes[(current + 1) % themes.length];
        if (next) {
          void setTheme(next.id, "global");
        }
        return;
      }
      if (event.altKey && (event.key === "PageDown" || event.key === "PageUp")) {
        event.preventDefault();
        if (projects.projects.length > 0) {
          const index = projects.projects.findIndex(
            (project) => project.id === projects.selectedId,
          );
          const delta = event.key === "PageDown" ? 1 : -1;
          const next =
            projects.projects[
              (index + delta + projects.projects.length) % projects.projects.length
            ];
          if (next) {
            void selectProject(next.id);
          }
        }
        return;
      }
      if (event.shiftKey && key === "n" && selected) {
        event.preventDefault();
        const shell = defaultShell();
        void addSession(
          {
            projectId: selected.id,
            driverId: "shell",
            program: shell.program,
            args: shell.args,
            cwd: selected.rootPath,
            env: [],
            cols: 100,
            rows: 30,
          },
          "Shell",
        );
        return;
      }
      if (event.shiftKey && key === "w") {
        const active = sessions.sessions.find((session) => session.id === sessions.activeId);
        if (active && window.confirm(`¿Terminar la sesión "${active.title}"?`)) {
          event.preventDefault();
          void killSession(active.id);
        }
        return;
      }
      if (event.shiftKey && key === "y") {
        event.preventDefault();
        setConversationOpen((open) => !open);
        return;
      }
      if (event.shiftKey && key === "b") {
        event.preventDefault();
        setWriterOpen(true);
        if (selected) {
          void invoke("writer_build", { id: selected.id }).catch(() => undefined);
        }
        return;
      }
      if (event.key === "=" || event.key === "+") {
        event.preventDefault();
        void setUiScale(increaseUiScale(uiScale)).then(setUiScaleValue);
      } else if (event.key === "-") {
        event.preventDefault();
        void setUiScale(decreaseUiScale(uiScale)).then(setUiScaleValue);
      } else if (event.key === "0") {
        event.preventDefault();
        void setUiScale(resetUiScale()).then(setUiScaleValue);
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [uiScale, projects, sessions]);

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
            <form
              className="file-search"
              onSubmit={(event) => {
                event.preventDefault();
                setSearchError(null);
                if (!searchQuery.trim()) {
                  setSearchResults([]);
                  return;
                }
                void invoke<FileEntry[]>("project_search", {
                  id: selected.id,
                  query: searchQuery,
                  limit: 40,
                })
                  .then(setSearchResults)
                  .catch((error: unknown) =>
                    setSearchError(error instanceof Error ? error.message : String(error)),
                  );
              }}
            >
              <input
                value={searchQuery}
                onChange={(event) => setSearchQuery(event.target.value)}
                placeholder="buscar archivos…"
                aria-label="Buscar archivos en el proyecto"
              />
              <button type="submit">buscar</button>
            </form>
            {searchError ? <p className="error">{searchError}</p> : null}
            {searchResults.length > 0 ? (
              <ul className="search-results">
                {searchResults.map((entry) => (
                  <li key={entry.path}>
                    {entry.isDir ? "▸ " : "· "}
                    {entry.path}
                  </li>
                ))}
              </ul>
            ) : null}
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

function appearanceThemeId(): string {
  const id = getAppearance().resolved.theme.id;
  return id;
}
