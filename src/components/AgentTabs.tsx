import { useEffect, useMemo, useState, useSyncExternalStore } from "react";
import { defaultShell } from "../lib/shell";
import { modelArguments } from "../lib/model-args";
import { sessionLabel } from "../lib/session-model";
import { useModelsStore } from "../stores/modelsStore";
import {
  getSnapshot as getDrivers,
  loadDrivers,
  subscribe as subscribeDrivers,
} from "../stores/driversStore";
import type { Project } from "../stores/projectsStore";
import { invoke } from "@tauri-apps/api/core";
import {
  addSession,
  closeSession,
  getSnapshot as getSessions,
  killSession,
  renameSession,
  restartSession,
  setActiveSession,
  subscribe as subscribeSessions,
  type SavedSession,
} from "../stores/sessionsStore";
import { RefreshCw } from "lucide-react";

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
  const models = useModelsStore();
  const loadModels = models.load;
  const [driverId, setDriverId] = useState("shell");
  const [modelId, setModelId] = useState("");
  const [sessionName, setSessionName] = useState("");
  const [saved, setSaved] = useState<SavedSession[]>([]);
  const [recovered, setRecovered] = useState<string[]>([]);

  useEffect(() => {
    void loadDrivers();
  }, []);

  useEffect(() => {
    if (driverId !== "shell") {
      void loadModels(driverId);
    }
  }, [driverId, loadModels]);

  useEffect(() => {
    void invoke<SavedSession[]>("session_saved")
      .then(setSaved)
      .catch(() => undefined);
  }, [project.id, sessions.sessions.length]);

  const options = useMemo<DriverOption[]>(() => {
    const shell = defaultShell();
    return [
      { id: "shell", label: "Shell", program: shell.program },
      ...drivers.drivers
        .filter((driver) => driver.status === "available")
        .map((driver) => ({
          id: driver.id,
          label: driver.displayName,
          program: driver.binaryPath ?? driver.binary,
        })),
    ];
  }, [drivers.drivers]);

  const projectSessions = sessions.sessions.filter((session) => session.projectId === project.id);
  const active = projectSessions.find((session) => session.id === sessions.activeId);
  const recoverable = saved.filter(
    (entry) =>
      entry.spec.projectId === project.id &&
      entry.status === "crashed" &&
      !recovered.includes(entry.id),
  );

  async function recover(entry: SavedSession) {
    await addSession(entry.spec, entry.spec.driverId === "shell" ? "Shell" : entry.spec.driverId);
    setRecovered((current) => [...current, entry.id]);
  }

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
        args: option.id === "shell" ? defaultShell().args : modelArguments(option.id, modelId),
        cwd: project.rootPath,
        env: [],
        cols: 80,
        rows: 24,
      },
      sessionName.trim() || (modelId ? `${option.label} (${modelId})` : option.label),
    );
    setSessionName("");
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
            {sessionLabel(session.title, session.spec.args, session.spec.driverId)}
          </button>
        ))}
        <span className="agent-create">
          <select
            value={driverId}
            onChange={(event) => {
              setDriverId(event.target.value);
              setModelId("");
            }}
            aria-label="Driver"
          >
            {options.map((option) => (
              <option key={option.id} value={option.id}>
                {option.label}
              </option>
            ))}
          </select>
          <input
            className="session-name-input"
            value={sessionName}
            onChange={(event) => setSessionName(event.target.value)}
            placeholder="nombre de sesión"
            aria-label="Nombre de la sesión"
          />
          {driverId !== "shell" ? (
            <button
              type="button"
              className="agent-model-refresh"
              onClick={() => void loadModels(driverId, true)}
              title={models.errors[driverId] ?? "Recargar modelos"}
              aria-label="Recargar modelos"
            >
              <RefreshCw size={12} />
            </button>
          ) : null}
          {driverId !== "shell" ? (
            <select
              value={modelId}
              onChange={(event) => setModelId(event.target.value)}
              aria-label="Modelo"
              title="Modelo de la nueva sesión"
            >
              <option value="">
                {models.loading === driverId
                  ? "cargando modelos…"
                  : models.errors[driverId]
                    ? "error al cargar modelos"
                    : `modelo por defecto (${(models.byDriver[driverId] ?? []).length})`}
              </option>
              {(models.byDriver[driverId] ?? []).map((model) => (
                <option key={model.id} value={model.id}>
                  {model.label || model.id}
                </option>
              ))}
            </select>
          ) : null}
          <button type="button" onClick={() => void createSession()}>
            + sesión
          </button>
        </span>
      </div>
      {recoverable.length > 0 ? (
        <div className="recoverable">
          <span className="muted">sesiones recuperables</span>
          {recoverable.map((entry) => (
            <button key={entry.id} type="button" onClick={() => void recover(entry)}>
              recrear {entry.spec.driverId}
            </button>
          ))}
        </div>
      ) : null}
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
