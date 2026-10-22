import { useEffect, useMemo, useState, useSyncExternalStore } from "react";
import { defaultShell } from "../lib/shell";
import {
  getSnapshot as getDrivers,
  loadDrivers,
  subscribe as subscribeDrivers,
} from "../stores/driversStore";
import type { Project } from "../stores/projectsStore";
import {
  addSession,
  closeSession,
  getSnapshot as getSessions,
  killSession,
  renameSession,
  restartSession,
  setActiveSession,
  subscribe as subscribeSessions,
} from "../stores/sessionsStore";

type AgentTabsProps = {
  project: Project;
};

type DriverOption = {
  id: string;
  label: string;
  program: string;
};

export default function AgentTabs({ project }: AgentTabsProps) {
  const sessions = useSyncExternalStore(subscribeSessions, getSessions);
  const drivers = useSyncExternalStore(subscribeDrivers, getDrivers);
  const [driverId, setDriverId] = useState("shell");

  useEffect(() => {
    void loadDrivers();
  }, []);

  const options = useMemo<DriverOption[]>(() => {
    const shell = defaultShell();
    return [
      { id: "shell", label: "Shell", program: shell.program },
      ...drivers.drivers
        .filter((driver) => driver.status === "available")
        .map((driver) => ({ id: driver.id, label: driver.displayName, program: driver.binary })),
    ];
  }, [drivers.drivers]);

  const projectSessions = sessions.sessions.filter((session) => session.projectId === project.id);
  const active = projectSessions.find((session) => session.id === sessions.activeId);

  async function createSession() {
    const option = options.find((entry) => entry.id === driverId) ?? options[0];
    if (!option) {
      return;
    }
    await addSession(
      {
        projectId: project.id,
        driverId: option.id,
        program: option.program,
        args: option.id === "shell" ? defaultShell().args : [],
        cwd: project.rootPath,
        env: [],
        cols: 80,
        rows: 24,
      },
      option.label,
    );
  }

  function askRename(id: string, current: string) {
    const title = window.prompt("Nombre de la tab", current);
    if (title) {
      renameSession(id, title);
    }
  }

  async function askClose(id: string, title: string) {
    if (!window.confirm(`¿Cerrar "${title}"? Se terminará el proceso.`)) {
      return;
    }
    await closeSession(id);
  }

  return (
    <section className="agent-tabs">
      <div className="agent-tab-row">
        {projectSessions.map((session) => (
          <button
            key={session.id}
            type="button"
            className={session.id === sessions.activeId ? "agent-tab active" : "agent-tab"}
            onClick={() => setActiveSession(session.id)}
            onDoubleClick={() => askRename(session.id, session.title)}
            title={`${session.spec.program} · ${session.status}`}
          >
            <span className={`status-dot ${session.status}`} />
            {session.title}
          </button>
        ))}
        <span className="agent-create">
          <select
            value={driverId}
            onChange={(event) => setDriverId(event.target.value)}
            aria-label="Driver"
          >
            {options.map((option) => (
              <option key={option.id} value={option.id}>
                {option.label}
              </option>
            ))}
          </select>
          <button type="button" onClick={() => void createSession()}>
            + sesión
          </button>
        </span>
      </div>
      {active ? (
        <div className="tab-actions">
          <span className={`status-dot ${active.status}`} />
          <span className="muted">{active.status}</span>
          <button type="button" onClick={() => askRename(active.id, active.title)}>
            renombrar
          </button>
          <button type="button" onClick={() => void killSession(active.id)}>
            matar
          </button>
          <button type="button" onClick={() => void restartSession(active.id)}>
            reiniciar
          </button>
          <button type="button" onClick={() => void askClose(active.id, active.title)}>
            cerrar
          </button>
        </div>
      ) : null}
    </section>
  );
}
