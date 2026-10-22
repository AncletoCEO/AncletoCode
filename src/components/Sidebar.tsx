import { useEffect, useState, useSyncExternalStore } from "react";
import DriversList from "./DriversList";
import {
  addProject,
  getSnapshot,
  loadProjects,
  removeProject,
  selectProject,
  subscribe,
} from "../stores/projectsStore";

export default function Sidebar() {
  const state = useSyncExternalStore(subscribe, getSnapshot);
  const [path, setPath] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void loadProjects();
  }, []);

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
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
    }
  }

  return (
    <aside className="sidebar">
      <div className="sidebar-title">❯ proyectos</div>
      <form className="project-form" onSubmit={handleSubmit}>
        <input
          className="project-input"
          value={path}
          onChange={(event) => setPath(event.target.value)}
          placeholder="/ruta/al/proyecto"
          aria-label="Ruta del proyecto"
          spellCheck={false}
        />
        <button type="submit" disabled={busy}>
          +
        </button>
      </form>
      {state.error ? <p className="error">{state.error}</p> : null}
      {state.loading ? <p className="muted">cargando…</p> : null}
      <ul className="project-list">
        {state.projects.map((project) => (
          <li
            key={project.id}
            className={project.id === state.selectedId ? "project active" : "project"}
          >
            <button
              type="button"
              className="project-name"
              onClick={() => void selectProject(project.id)}
              title={project.rootPath}
            >
              {project.name}
            </button>
            <button
              type="button"
              className="project-remove"
              onClick={() => void removeProject(project.id)}
              aria-label={`Quitar ${project.name}`}
            >
              ×
            </button>
          </li>
        ))}
      </ul>
      <DriversList />
    </aside>
  );
}
