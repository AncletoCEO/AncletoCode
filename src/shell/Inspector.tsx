import { useEffect, useState, useSyncExternalStore } from "react";
import {
  BookOpen,
  CircleDot,
  FileDiff,
  GitBranch,
  GitPullRequest,
  PanelRightClose,
  Play,
  Users,
} from "lucide-react";
import type { Project } from "../stores/projectsStore";
import ProjectActions from "../components/ProjectActions";
import type { Session } from "../stores/sessionsStore";
import {
  getSnapshot as getDrivers,
  loadDrivers,
  subscribe as subscribeDrivers,
} from "../stores/driversStore";
import { useGitStore } from "../stores/gitStore";
import GitDiffDialog from "./GitDiffDialog";
import { cn } from "../ui/cn";
import type { InspectorTab, ShellView } from "./types";

type InspectorProps = {
  project: Project | null;
  session: Session | null;
  sessions: Session[];
  tab: InspectorTab;
  onTab: (tab: InspectorTab) => void;
  onClose: () => void;
  onAux: (view: ShellView) => void;
};

const tabs: { id: InspectorTab; label: string }[] = [
  { id: "context", label: "Contexto" },
  { id: "changes", label: "Cambios" },
  { id: "git", label: "Git" },
  { id: "team", label: "Equipo" },
];

export default function Inspector({
  project,
  session,
  sessions,
  tab,
  onTab,
  onClose,
  onAux,
}: InspectorProps) {
  const drivers = useSyncExternalStore(subscribeDrivers, getDrivers);
  const git = useGitStore();
  const [branchName, setBranchName] = useState("");
  const [commitMessage, setCommitMessage] = useState("");

  useEffect(() => {
    void loadDrivers();
  }, []);

  const projectId = project?.id;
  const loadGit = git.load;

  useEffect(() => {
    if (projectId && tab === "git") {
      void loadGit(projectId);
    }
  }, [projectId, tab, loadGit]);

  async function createBranch(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!branchName.trim()) {
      return;
    }
    const created = await git.createBranch(branchName.trim(), true);
    if (created) {
      setBranchName("");
    }
  }

  async function commit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!commitMessage.trim()) {
      return;
    }
    const committed = await git.commit(commitMessage.trim());
    if (committed) {
      setCommitMessage("");
    }
  }

  return (
    <aside className="shell-inspector">
      <div className="shell-inspector-head">
        <div className="shell-inspector-tabs" role="tablist" aria-label="Inspector">
          {tabs.map((entry) => (
            <button
              key={entry.id}
              type="button"
              role="tab"
              aria-selected={tab === entry.id}
              className={cn("shell-inspector-tab", tab === entry.id && "is-active")}
              onClick={() => onTab(entry.id)}
            >
              {entry.label}
            </button>
          ))}
        </div>
        <button
          type="button"
          className="icon-button icon-button-small"
          onClick={onClose}
          aria-label="Cerrar inspector"
        >
          <PanelRightClose size={14} />
        </button>
      </div>

      <div className="shell-inspector-body">
        {tab === "context" ? (
          <div className="shell-inspector-stack">
            <section className="shell-card">
              <div className="shell-card-title">Proyecto</div>
              {project ? (
                <>
                  <div className="shell-card-value">{project.name}</div>
                  <code className="shell-path">{project.rootPath}</code>
                  <ProjectActions projectId={project.id} />
                </>
              ) : (
                <p className="shell-muted">Sin proyecto seleccionado.</p>
              )}
            </section>

            <section className="shell-card">
              <div className="shell-card-title">Sesión</div>
              {session ? (
                <div className="shell-status-row">
                  <span className={cn("shell-status-dot", session.status)} />
                  <span>{session.status}</span>
                  <span className="shell-muted">{session.spec.driverId}</span>
                </div>
              ) : (
                <p className="shell-muted">No hay una sesión activa.</p>
              )}
            </section>

            <section className="shell-card shell-card-actions">
              <button type="button" onClick={() => onAux("knowledge")}>
                <BookOpen size={14} />
                Abrir Writer
              </button>
              <button type="button" onClick={() => onAux("appearance")}>
                <CircleDot size={14} />
                Apariencia avanzada
              </button>
            </section>
          </div>
        ) : null}

        {tab === "changes" ? (
          <div className="shell-empty">
            <FileDiff size={22} />
            <strong>Sin cambios todavía</strong>
            <p>Los diffs del turno y el staging van a aparecer acá cuando conectemos Git.</p>
          </div>
        ) : null}

        {tab === "git" ? (
          <div className="shell-inspector-stack">
            {git.loading ? <p className="shell-muted">Leyendo repositorio…</p> : null}
            {git.error ? <p className="shell-error">{git.error}</p> : null}
            {!git.loading && git.status && !git.status.isRepo ? (
              <div className="shell-empty">
                <GitBranch size={22} />
                <strong>Este proyecto no es un repo Git</strong>
                <p>
                  Inicializá el repositorio desde la terminal del proyecto para ver ramas y cambios.
                </p>
              </div>
            ) : null}
            {git.status?.isRepo ? (
              <>
                <section className="shell-card">
                  <div className="shell-card-title">
                    <GitBranch size={13} /> Rama actual
                  </div>
                  <div className="shell-branch-current">
                    <strong>{git.status.branch ?? "detached"}</strong>
                    {git.status.upstream ? <span>{git.status.upstream}</span> : null}
                    {git.status.dirty ? (
                      <span className="shell-branch-dirty-label">cambios</span>
                    ) : null}
                  </div>
                  <div className="shell-git-meta">
                    <span>↑ {git.status.ahead}</span>
                    <span>↓ {git.status.behind}</span>
                    <span>{git.status.files.length} archivos</span>
                  </div>
                </section>

                <section className="shell-card">
                  <div className="shell-card-title">Cambios</div>
                  {git.status.files.length === 0 ? (
                    <p className="shell-muted">Árbol limpio.</p>
                  ) : (
                    <div className="shell-git-files">
                      {git.status.files.slice(0, 12).map((file) => (
                        <div key={`${file.path}-${file.staged}`} className="shell-git-file">
                          <span className={`shell-git-status ${file.status}`} />
                          <span>{file.path}</span>
                          <span className="shell-muted">
                            {file.staged ? "staged" : file.status}
                          </span>
                        </div>
                      ))}
                    </div>
                  )}
                </section>

                <section className="shell-card">
                  <div className="shell-card-title">Ramas</div>
                  <div className="shell-git-branches">
                    {git.branches.map((branch) => (
                      <button
                        key={branch.name}
                        type="button"
                        className={cn("shell-git-branch", branch.current && "is-current")}
                        onClick={() => {
                          if (!branch.current) {
                            void git.checkout(branch.name);
                          }
                        }}
                        disabled={branch.current}
                      >
                        <GitBranch size={12} />
                        <span>{branch.name}</span>
                        {branch.current ? <span className="shell-muted">actual</span> : null}
                      </button>
                    ))}
                  </div>
                  <form
                    className="shell-branch-form"
                    onSubmit={(event) => void createBranch(event)}
                  >
                    <input
                      value={branchName}
                      onChange={(event) => setBranchName(event.target.value)}
                      placeholder="nueva-rama"
                      aria-label="Nombre de la nueva rama"
                      spellCheck={false}
                    />
                    <button type="submit" disabled={!branchName.trim()}>
                      Crear
                    </button>
                  </form>
                </section>

                <form className="shell-commit-form" onSubmit={(event) => void commit(event)}>
                  <input
                    value={commitMessage}
                    onChange={(event) => setCommitMessage(event.target.value)}
                    placeholder="Mensaje de commit"
                    aria-label="Mensaje de commit"
                  />
                  <button type="submit" disabled={!commitMessage.trim()}>
                    Commit
                  </button>
                </form>

                <button type="button" className="shell-ghost-button" disabled>
                  <GitPullRequest size={14} />
                  Conectar PR / MR
                </button>
              </>
            ) : null}
          </div>
        ) : null}

        {tab === "team" ? (
          <div className="shell-inspector-stack">
            <section className="shell-card">
              <div className="shell-card-title">
                <Users size={13} /> Drivers
              </div>
              <div className="shell-driver-list">
                {drivers.drivers.map((driver) => (
                  <div key={driver.id} className="shell-driver-row">
                    <span
                      className={cn(
                        "shell-status-dot",
                        driver.status === "available" ? "ready" : "crashed",
                      )}
                    />
                    <span>{driver.displayName}</span>
                    <span className="shell-muted">{driver.version ?? driver.status}</span>
                  </div>
                ))}
              </div>
            </section>
            <section className="shell-card">
              <div className="shell-card-title">
                <Play size={13} /> Sesiones
              </div>
              {sessions.length === 0 ? (
                <p className="shell-muted">Todavía no hay sesiones.</p>
              ) : (
                sessions.map((entry) => (
                  <div key={entry.id} className="shell-session-row">
                    <span className={cn("shell-status-dot", entry.status)} />
                    <span>{entry.title}</span>
                    <span className="shell-muted">{entry.spec.driverId}</span>
                  </div>
                ))
              )}
            </section>
          </div>
        ) : null}
      </div>
      <GitDiffDialog />
    </aside>
  );
}
