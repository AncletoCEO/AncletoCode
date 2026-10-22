import { useSyncExternalStore } from "react";
import { getSnapshot as getDrivers, subscribe as subscribeDrivers } from "../stores/driversStore";
import {
  getSnapshot as getProjects,
  selectProject,
  subscribe as subscribeProjects,
} from "../stores/projectsStore";
import {
  getSnapshot as getSessions,
  subscribe as subscribeSessions,
} from "../stores/sessionsStore";

export default function ProjectTabs() {
  const projects = useSyncExternalStore(subscribeProjects, getProjects);
  const sessions = useSyncExternalStore(subscribeSessions, getSessions);
  const drivers = useSyncExternalStore(subscribeDrivers, getDrivers);
  const hasDrivers = drivers.drivers.some((driver) => driver.status === "available");

  return (
    <nav className="project-tabs" aria-label="Proyectos">
      {projects.projects.map((project) => {
        const active = sessions.sessions.filter(
          (session) =>
            session.projectId === project.id &&
            session.status !== "stopped" &&
            session.status !== "crashed",
        ).length;
        return (
          <button
            key={project.id}
            type="button"
            className={project.id === projects.selectedId ? "project-tab active" : "project-tab"}
            onClick={() => void selectProject(project.id)}
            title={project.rootPath}
          >
            {project.name}
            {active > 0 ? <span className="tab-activity"> ●{active}</span> : null}
            {!hasDrivers ? (
              <span className="tab-warning" title="Sin drivers disponibles">
                {" "}
                !
              </span>
            ) : null}
          </button>
        );
      })}
    </nav>
  );
}
