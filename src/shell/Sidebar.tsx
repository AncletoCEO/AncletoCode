import { useEffect, useState, useSyncExternalStore } from "react";
import {
  Activity,
  BookOpen,
  ChevronRight,
  FileCode2,
  FolderGit2,
  GitBranch,
  ListTodo,
  Plus,
  Search,
  Settings2,
  Sparkles,
  X,
} from "lucide-react";
import ProjectTree from "../components/ProjectTree";
import {
  addProject,
  getSnapshot as getProjects,
  loadProjects,
  removeProject,
  selectProject,
  subscribe as subscribeProjects,
} from "../stores/projectsStore";
import { cn } from "../ui/cn";
import type { ShellView } from "./types";

type SidebarProps = {
  view: ShellView;
  onView: (view: ShellView) => void;
  onSettings: () => void;
  onPalette: () => void;
};

const nav = [
  { id: "workspace", label: "Workspace", icon: Sparkles },
  { id: "tasks", label: "Tareas", icon: ListTodo },
  { id: "git", label: "Git", icon: GitBranch },
  { id: "usage", label: "Uso", icon: Activity },
  { id: "knowledge", label: "Conocimiento", icon: BookOpen },
] as const;

export default function Sidebar({ view, onView, onSettings, onPalette }: SidebarProps) {
  const projects = useSyncExternalStore(subscribeProjects, getProjects);
  const [adding, setAdding] = useState(false);
  const [path, setPath] = useState("");
  const [busy, setBusy] = useState(false);
  const [filesOpen, setFilesOpen] = useState(true);

  useEffect(() => {
    void loadProjects();
  }, []);

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const value = path.trim();
    if (!value || busy) {
      return;
    }
    setBusy(true);
    const added = await addProject(value);
    setBusy(false);
    if (added) {
      setPath("");
      setAdding(false);
    }
  }

  const selected = projects.projects.find((project) => project.id === projects.selectedId);

  return (
    <aside className="shell-sidebar">
      <div className="shell-brand">
        <span className="shell-brand-mark">A</span>
        <span className="shell-brand-text">
          <strong>AncletoCode</strong>
          <small>agent workspace</small>
        </span>
        <button type="button" className="icon-button" onClick={onPalette} aria-label="Buscar">
          <Search size={15} />
        </button>
      </div>

      <nav className="shell-nav" aria-label="Navegación principal">
        {nav.map((entry) => {
          const Icon = entry.icon;
          return (
            <button
              key={entry.id}
              type="button"
              className={cn("shell-nav-item", view === entry.id && "is-active")}
              onClick={() => onView(entry.id)}
            >
              <Icon size={15} />
              <span>{entry.label}</span>
              {entry.id === "git" ? <span className="shell-badge">soon</span> : null}
            </button>
          );
        })}
      </nav>

      <div className="shell-section">
        <div className="shell-section-title">
          <span>Proyectos</span>
          <button
            type="button"
            className="icon-button icon-button-small"
            onClick={() => setAdding((open) => !open)}
            aria-label="Agregar proyecto"
          >
            <Plus size={14} />
          </button>
        </div>

        {adding ? (
          <form className="shell-project-form" onSubmit={(event) => void submit(event)}>
            <input
              autoFocus
              value={path}
              onChange={(event) => setPath(event.target.value)}
              placeholder="/ruta/al/proyecto"
              aria-label="Ruta del proyecto"
              spellCheck={false}
            />
            <button type="submit" disabled={busy} aria-label="Confirmar proyecto">
              <ChevronRight size={14} />
            </button>
            <button type="button" onClick={() => setAdding(false)} aria-label="Cancelar">
              <X size={14} />
            </button>
          </form>
        ) : null}

        <div className="shell-projects">
          {projects.projects.length === 0 ? (
            <button type="button" className="shell-empty-link" onClick={() => setAdding(true)}>
              <FolderGit2 size={15} />
              Agregar el primer proyecto
            </button>
          ) : (
            projects.projects.map((project) => (
              <div
                key={project.id}
                className={cn("shell-project", project.id === projects.selectedId && "is-active")}
              >
                <button
                  type="button"
                  className="shell-project-main"
                  onClick={() => void selectProject(project.id)}
                  title={project.rootPath}
                >
                  <span className="shell-project-dot" />
                  <span className="shell-project-name">{project.name}</span>
                </button>
                <button
                  type="button"
                  className="shell-project-remove"
                  onClick={() => void removeProject(project.id)}
                  aria-label={`Quitar ${project.name}`}
                >
                  <X size={13} />
                </button>
              </div>
            ))
          )}
        </div>
      </div>

      {selected ? (
        <div className="shell-section shell-files">
          <button
            type="button"
            className="shell-section-title shell-section-toggle"
            onClick={() => setFilesOpen((open) => !open)}
          >
            <span>Archivos</span>
            <ChevronRight size={14} className={filesOpen ? "rotate-90" : ""} />
          </button>
          {filesOpen && projects.tree ? (
            <ul className="tree shell-tree">
              <ProjectTree node={projects.tree} />
            </ul>
          ) : null}
          {filesOpen && !projects.tree ? <p className="shell-muted">Abriendo árbol…</p> : null}
        </div>
      ) : null}

      <div className="shell-sidebar-footer">
        <button type="button" className="shell-nav-item" onClick={onSettings}>
          <Settings2 size={15} />
          <span>Ajustes</span>
        </button>
        <div className="shell-version">
          <FileCode2 size={13} />
          0.0.1
        </div>
      </div>
    </aside>
  );
}
