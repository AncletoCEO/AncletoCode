import { useEffect, useState, useSyncExternalStore, type CSSProperties } from "react";
import { ArrowLeft, BookOpen, GitBranch, ListTodo, Sparkles, TerminalSquare } from "lucide-react";
import AboutDialog from "../components/AboutDialog";
import AgentTabs from "../components/AgentTabs";
import AppearancePanel from "../components/AppearancePanel";
import ConversationPanel from "../components/ConversationPanel";
import ProjectTabs from "../components/ProjectTabs";
import SpecPanel from "../components/SpecPanel";
import TerminalView from "../components/TerminalView";
import WriterPanel from "../components/WriterPanel";
import { defaultShell } from "../lib/shell";
import {
  addProject,
  getSnapshot as getProjects,
  subscribe as subscribeProjects,
} from "../stores/projectsStore";
import {
  addSession,
  getSnapshot as getSessions,
  restoreSessions,
  subscribe as subscribeSessions,
} from "../stores/sessionsStore";
import {
  getSnapshot as getDrivers,
  loadDrivers,
  subscribe as subscribeDrivers,
} from "../stores/driversStore";
import {
  getSnapshot as getAppearance,
  loadAppearance,
  subscribe as subscribeAppearance,
} from "../stores/appearanceStore";
import { cn } from "../ui/cn";
import CommandPalette from "./CommandPalette";
import Composer from "./Composer";
import Inspector from "./Inspector";
import Sidebar from "./Sidebar";
import SettingsDialog from "./SettingsDialog";
import UpdatesBanner from "./UpdatesBanner";
import TopBar from "./TopBar";
import { useShellPreferences } from "./preferences";
import { useUpdatesStore } from "../stores/updatesStore";
import type { InspectorTab, ShellView } from "./types";

export default function DesktopShell() {
  const projects = useSyncExternalStore(subscribeProjects, getProjects);
  const sessions = useSyncExternalStore(subscribeSessions, getSessions);
  const drivers = useSyncExternalStore(subscribeDrivers, getDrivers);
  const appearance = useSyncExternalStore(subscribeAppearance, getAppearance);
  const preferences = useShellPreferences();
  const checkUpdates = useUpdatesStore((state) => state.check);

  const [view, setView] = useState<ShellView>("workspace");
  const [inspectorTab, setInspectorTab] = useState<InspectorTab>("context");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);
  const [conversationOpen, setConversationOpen] = useState(false);
  const [auxPanel, setAuxPanel] = useState<"spec" | "writer">("writer");

  const project = projects.projects.find((entry) => entry.id === projects.selectedId) ?? null;
  const projectSessions = sessions.sessions.filter((entry) => entry.projectId === project?.id);
  const session = projectSessions.find((entry) => entry.id === sessions.activeId) ?? null;
  const themeAccent = appearance.resolved.theme.tokens.accent;
  const accent = preferences.accent ?? themeAccent;

  useEffect(() => {
    void restoreSessions();
    void loadDrivers();
    void checkUpdates();
  }, [checkUpdates]);

  useEffect(() => {
    void loadAppearance(project?.id ?? null);
  }, [project?.id]);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      const mod = event.metaKey || event.ctrlKey;
      if (!mod) {
        return;
      }
      if (event.key.toLowerCase() === "k") {
        event.preventDefault();
        setPaletteOpen((open) => !open);
      } else if (event.key === ",") {
        event.preventDefault();
        setSettingsOpen(true);
      } else if (event.shiftKey && event.key.toLowerCase() === "n") {
        event.preventDefault();
        void createSession("shell");
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  });

  async function createSession(driverId: string) {
    if (!project) {
      return;
    }
    const driver = drivers.drivers.find((entry) => entry.id === driverId);
    const shell = defaultShell();
    const isShell = driverId === "shell" || !driver;
    if (!isShell && driver.status !== "available") {
      return;
    }
    await addSession(
      {
        projectId: project.id,
        driverId: isShell ? "shell" : driverId,
        program: isShell ? shell.program : (driver?.binary ?? shell.program),
        args: isShell ? shell.args : [],
        cwd: project.rootPath,
        env: [],
        cols: 120,
        rows: 32,
      },
      isShell ? "Shell" : (driver?.displayName ?? driverId),
    );
    setView("workspace");
  }

  async function addProjectFromPrompt() {
    const path = window.prompt("Ruta del proyecto");
    if (!path?.trim()) {
      return;
    }
    const added = await addProject(path.trim());
    if (added) {
      setView("workspace");
    }
  }

  function openAux(panel: "spec" | "writer") {
    setAuxPanel(panel);
    setView("knowledge");
  }

  const style = {
    "--shell-accent": accent,
    "--shell-radius": `${preferences.radius}px`,
    "--shell-font-scale": preferences.fontScale,
  } as CSSProperties;

  return (
    <main className="app-shell" data-density={preferences.density} style={style}>
      <Sidebar
        view={view}
        onView={setView}
        onSettings={() => setSettingsOpen(true)}
        onPalette={() => setPaletteOpen(true)}
      />

      <section className="app-main">
        <UpdatesBanner />
        <TopBar
          project={project}
          session={session}
          inspectorOpen={preferences.inspectorOpen}
          onToggleInspector={() => preferences.setInspectorOpen(!preferences.inspectorOpen)}
          onPalette={() => setPaletteOpen(true)}
          onNewSession={() => void createSession("shell")}
          onSettings={() => setSettingsOpen(true)}
          onGit={() => {
            preferences.setInspectorOpen(true);
            setInspectorTab("git");
            setView("workspace");
          }}
        />

        <div className="app-body">
          <section className="workspace-column">
            {view === "workspace" ? (
              project ? (
                <div className="shell-thread">
                  <div className="shell-tabs">
                    <ProjectTabs />
                    <AgentTabs project={project} />
                    <div className="shell-thread-tools">
                      <button
                        type="button"
                        className={cn("shell-tool-button", conversationOpen && "is-active")}
                        onClick={() => setConversationOpen((open) => !open)}
                      >
                        <Sparkles size={13} />
                        Conversación
                      </button>
                      <button
                        type="button"
                        className="shell-tool-button"
                        onClick={() => openAux("spec")}
                      >
                        <ListTodo size={13} />
                        Spec
                      </button>
                      <button
                        type="button"
                        className="shell-tool-button"
                        onClick={() => openAux("writer")}
                      >
                        <BookOpen size={13} />
                        Writer
                      </button>
                    </div>
                  </div>

                  <div className="shell-thread-scroll">
                    {conversationOpen ? (
                      <ConversationPanel project={project} />
                    ) : session ? (
                      <div className="shell-terminal-host">
                        <TerminalView key={session.id} sessionId={session.id} />
                      </div>
                    ) : (
                      <WelcomeEmpty
                        projectName={project.name}
                        onNewSession={() => void createSession("shell")}
                        onOpenSpec={() => openAux("spec")}
                        onOpenPalette={() => setPaletteOpen(true)}
                      />
                    )}
                  </div>

                  <Composer session={session} onNewSession={() => void createSession("shell")} />
                </div>
              ) : (
                <WelcomeProject
                  onAdd={() => void addProjectFromPrompt()}
                  onPalette={() => setPaletteOpen(true)}
                />
              )
            ) : view === "knowledge" ? (
              <AuxView
                title={auxPanel === "spec" ? "Spec" : "Conocimiento"}
                description={
                  auxPanel === "spec"
                    ? "Estado del SDD, discovery y memoria del proyecto."
                    : "Grafo, hotspots y documentos del proyecto."
                }
                icon={auxPanel === "spec" ? <ListTodo size={20} /> : <BookOpen size={20} />}
                onBack={() => setView("workspace")}
              >
                {project ? (
                  <>
                    <div className="shell-aux-switch">
                      <button
                        type="button"
                        className={cn(auxPanel === "spec" && "is-active")}
                        onClick={() => setAuxPanel("spec")}
                      >
                        Spec
                      </button>
                      <button
                        type="button"
                        className={cn(auxPanel === "writer" && "is-active")}
                        onClick={() => setAuxPanel("writer")}
                      >
                        Writer
                      </button>
                    </div>
                    {auxPanel === "spec" ? (
                      <SpecPanel projectId={project.id} />
                    ) : (
                      <WriterPanel projectId={project.id} />
                    )}
                  </>
                ) : (
                  <p className="shell-muted">Seleccioná un proyecto.</p>
                )}
              </AuxView>
            ) : view === "appearance" ? (
              <AuxView
                title="Apariencia"
                description="Temas, fondos, transparencia y contraste."
                icon={<Sparkles size={20} />}
                onBack={() => setView("workspace")}
              >
                {project ? (
                  <AppearancePanel projectId={project.id} />
                ) : (
                  <p className="shell-muted">
                    Seleccioná un proyecto para usar la apariencia avanzada.
                  </p>
                )}
              </AuxView>
            ) : view === "tasks" ? (
              <FeatureView
                icon={<ListTodo size={22} />}
                title="Tareas en preparación"
                description="El board de tareas, worktrees y agentes en paralelo llega con la fase Git."
                onBack={() => setView("workspace")}
              />
            ) : (
              <FeatureView
                icon={<GitBranch size={22} />}
                title="Git en preparación"
                description="Ramas, worktrees, diffs y PRs se están diseñando sobre el nuevo shell."
                onBack={() => setView("workspace")}
              />
            )}
          </section>

          {preferences.inspectorOpen ? (
            <Inspector
              project={project}
              session={session}
              sessions={projectSessions}
              tab={inspectorTab}
              onTab={setInspectorTab}
              onClose={() => preferences.setInspectorOpen(false)}
              onAux={(next) => {
                if (next === "knowledge") {
                  setAuxPanel("writer");
                }
                setView(next);
              }}
            />
          ) : null}
        </div>
      </section>

      <CommandPalette
        open={paletteOpen}
        onOpenChange={setPaletteOpen}
        onNewSession={(driverId) => void createSession(driverId)}
        onNewProject={() => void addProjectFromPrompt()}
        onSettings={() => setSettingsOpen(true)}
        onAbout={() => setAboutOpen(true)}
        onView={setView}
      />
      <SettingsDialog
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        onOpenAppearance={() => setView("appearance")}
      />
      {aboutOpen ? <AboutDialog onClose={() => setAboutOpen(false)} /> : null}
    </main>
  );
}

function WelcomeProject({ onAdd, onPalette }: { onAdd: () => void; onPalette: () => void }) {
  return (
    <div className="shell-welcome">
      <div className="shell-welcome-icon">
        <TerminalSquare size={24} />
      </div>
      <h1>Tu workspace, sin ruido.</h1>
      <p>
        Agregá un proyecto para abrir sus archivos, lanzar agentes y mantener las sesiones vivas
        aunque cierres la ventana.
      </p>
      <div className="shell-welcome-actions">
        <button type="button" className="shell-primary-button" onClick={onAdd}>
          Agregar proyecto
        </button>
        <button type="button" className="shell-ghost-button" onClick={onPalette}>
          Abrir comandos <kbd>⌘K</kbd>
        </button>
      </div>
    </div>
  );
}

function WelcomeEmpty({
  projectName,
  onNewSession,
  onOpenSpec,
  onOpenPalette,
}: {
  projectName: string;
  onNewSession: () => void;
  onOpenSpec: () => void;
  onOpenPalette: () => void;
}) {
  return (
    <div className="shell-empty shell-empty-thread">
      <div className="shell-welcome-icon">
        <Sparkles size={22} />
      </div>
      <strong>{projectName} está listo</strong>
      <p>Creá una sesión para hablar con un agente o abrí el estado SDD del proyecto.</p>
      <div className="shell-welcome-actions">
        <button type="button" className="shell-primary-button" onClick={onNewSession}>
          Nueva sesión
        </button>
        <button type="button" className="shell-ghost-button" onClick={onOpenSpec}>
          Ver spec
        </button>
        <button type="button" className="shell-ghost-button" onClick={onOpenPalette}>
          Comandos
        </button>
      </div>
    </div>
  );
}

function FeatureView({
  icon,
  title,
  description,
  onBack,
}: {
  icon: React.ReactNode;
  title: string;
  description: string;
  onBack: () => void;
}) {
  return (
    <div className="shell-feature-view">
      <button type="button" className="shell-back" onClick={onBack}>
        <ArrowLeft size={14} /> Workspace
      </button>
      <div className="shell-empty shell-empty-thread">
        <div className="shell-welcome-icon">{icon}</div>
        <strong>{title}</strong>
        <p>{description}</p>
      </div>
    </div>
  );
}

function AuxView({
  title,
  description,
  icon,
  onBack,
  children,
}: {
  title: string;
  description: string;
  icon: React.ReactNode;
  onBack: () => void;
  children: React.ReactNode;
}) {
  return (
    <div className="shell-feature-view">
      <button type="button" className="shell-back" onClick={onBack}>
        <ArrowLeft size={14} /> Workspace
      </button>
      <header className="shell-aux-head">
        <span className="shell-aux-icon">{icon}</span>
        <div>
          <h1>{title}</h1>
          <p>{description}</p>
        </div>
      </header>
      <div className="shell-aux-body">{children}</div>
    </div>
  );
}
