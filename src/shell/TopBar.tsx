import { Command, GitBranch, PanelRight, Plus, Settings2 } from "lucide-react";
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
};

export default function TopBar({
  project,
  session,
  inspectorOpen,
  onToggleInspector,
  onPalette,
  onNewSession,
  onSettings,
}: TopBarProps) {
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
          className="shell-branch-chip"
          title="La integración Git se está preparando"
          onClick={onPalette}
        >
          <GitBranch size={13} />
          <span>Git</span>
          <span className="shell-chip-muted">próximamente</span>
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
