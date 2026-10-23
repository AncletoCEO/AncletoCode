import { Command, GitBranch, PanelRight, Plus, Settings2 } from "lucide-react";
import { useEffect } from "react";
import { useGitStore } from "../stores/gitStore";
import type { Project } from "../stores/projectsStore";
import type { Session } from "../stores/sessionsStore";
import { cn } from "../ui/cn";

type TopBarProps = {
  project: Project | null;
  session: Session | null;
  inspectorOpen: boolean;
  onToggleInspector: () => void;
  onPalette: () => void;
  onNewSession: () => void;
  onSettings: () => void;
  onGit: () => void;
};

export default function TopBar({
  project,
  session,
  inspectorOpen,
  onToggleInspector,
  onPalette,
  onNewSession,
  onSettings,
  onGit,
}: TopBarProps) {
  const git = useGitStore();
  const projectId = project?.id;
  const loadGit = git.load;

  useEffect(() => {
    if (projectId) {
      void loadGit(projectId);
    }
  }, [projectId, loadGit]);

  const gitLabel = git.loading
    ? "Git…"
    : git.status?.isRepo
      ? (git.status.branch ?? "detached")
      : "Sin Git";

  return (
    <header className="shell-topbar">
      <div className="shell-breadcrumb">
        <span className="shell-crumb-muted">Workspace</span>
        {project ? (
          <>
            <span className="shell-crumb-sep">/</span>
            <span className="shell-crumb-current" title={project.rootPath}>
              {project.name}
            </span>
          </>
        ) : null}
        {session ? (
          <>
            <span className="shell-crumb-sep">/</span>
            <span className="shell-crumb-session">{session.title}</span>
          </>
        ) : null}
      </div>

      <div className="shell-topbar-actions">
        <button
          type="button"
          className={git.status?.dirty ? "shell-branch-chip is-dirty" : "shell-branch-chip"}
          title={git.status?.isRepo ? `Rama ${gitLabel}` : "El proyecto no es un repositorio Git"}
          onClick={onGit}
        >
          <GitBranch size={13} />
          <span>{gitLabel}</span>
          {git.status?.dirty ? <span className="shell-branch-dirty" /> : null}
          {git.status && git.status.ahead > 0 ? (
            <span className="shell-chip-muted">↑{git.status.ahead}</span>
          ) : null}
          {git.status && git.status.behind > 0 ? (
            <span className="shell-chip-muted">↓{git.status.behind}</span>
          ) : null}
        </button>

        <button type="button" className="shell-search-trigger" onClick={onPalette}>
          <Command size={13} />
          <span>Buscar o ejecutar</span>
          <kbd>⌘K</kbd>
        </button>

        <button
          type="button"
          className="icon-button"
          onClick={onNewSession}
          aria-label="Nueva sesión"
          title="Nueva sesión"
        >
          <Plus size={16} />
        </button>
        <button
          type="button"
          className={cn("icon-button", inspectorOpen && "is-active")}
          onClick={onToggleInspector}
          aria-label="Alternar inspector"
          aria-pressed={inspectorOpen}
          title="Inspector"
        >
          <PanelRight size={16} />
        </button>
        <button
          type="button"
          className="icon-button"
          onClick={onSettings}
          aria-label="Ajustes"
          title="Ajustes"
        >
          <Settings2 size={16} />
        </button>
      </div>
    </header>
  );
}
