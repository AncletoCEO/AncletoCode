import { useEffect, useState } from "react";
import {
  ArrowRight,
  Check,
  CircleDashed,
  GitBranch,
  GitPullRequest,
  ListTodo,
  Plus,
  Trash2,
} from "lucide-react";
import type { Project } from "../stores/projectsStore";
import { useTasksStore, type Task, type TaskStatus } from "../stores/tasksStore";
import { cn } from "../ui/cn";

type TasksViewProps = {
  project: Project | null;
};

const columns: { id: TaskStatus; label: string; icon: React.ReactNode }[] = [
  { id: "backlog", label: "Pendientes", icon: <CircleDashed size={13} /> },
  { id: "inProgress", label: "En progreso", icon: <ListTodo size={13} /> },
  { id: "review", label: "Review", icon: <GitPullRequest size={13} /> },
  { id: "done", label: "Hechas", icon: <Check size={13} /> },
  { id: "canceled", label: "Canceladas", icon: <Trash2 size={13} /> },
];

const nextStatus: Record<TaskStatus, TaskStatus> = {
  backlog: "inProgress",
  inProgress: "review",
  review: "done",
  done: "done",
  canceled: "canceled",
};

export default function TasksView({ project }: TasksViewProps) {
  const tasks = useTasksStore();
  const [title, setTitle] = useState("");
  const loadTasks = tasks.load;
  const projectId = project?.id;

  useEffect(() => {
    if (projectId) {
      void loadTasks(projectId);
    }
  }, [projectId, loadTasks]);

  if (!project) {
    return (
      <div className="shell-feature-view">
        <div className="shell-empty shell-empty-thread">
          <div className="shell-welcome-icon">
            <ListTodo size={22} />
          </div>
          <strong>Elegí un proyecto</strong>
          <p>Las tareas se organizan por proyecto.</p>
        </div>
      </div>
    );
  }

  async function create(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!title.trim()) {
      return;
    }
    const created = await tasks.create(title);
    if (created) {
      setTitle("");
    }
  }

  return (
    <div className="shell-tasks-view">
      <header className="shell-tasks-head">
        <div>
          <h1>
            <ListTodo size={19} /> Tareas
          </h1>
          <p>
            {project.name} · {tasks.tasks.length} tareas
          </p>
        </div>
        <form className="shell-task-create" onSubmit={(event) => void create(event)}>
          <input
            value={title}
            onChange={(event) => setTitle(event.target.value)}
            placeholder="Nueva tarea…"
            aria-label="Título de la nueva tarea"
          />
          <button type="submit" disabled={!title.trim()}>
            <Plus size={14} /> Crear
          </button>
        </form>
      </header>

      {tasks.error ? <p className="shell-error">{tasks.error}</p> : null}

      <div className="shell-task-board">
        {columns.map((column) => {
          const items = tasks.tasks.filter((task) => task.status === column.id);
          return (
            <section key={column.id} className="shell-task-column">
              <div className="shell-task-column-head">
                <span>
                  {column.icon} {column.label}
                </span>
                <span className="shell-task-count">{items.length}</span>
              </div>
              <div className="shell-task-column-body">
                {items.map((task) => (
                  <TaskCard key={task.id} task={task} />
                ))}
                {items.length === 0 ? <p className="shell-task-empty">Sin tareas</p> : null}
              </div>
            </section>
          );
        })}
      </div>
    </div>
  );
}

function TaskCard({ task }: { task: Task }) {
  const tasks = useTasksStore();
  const canAdvance = task.status !== "done" && task.status !== "canceled";

  return (
    <article className="shell-task-card">
      <div className="shell-task-card-title">{task.title}</div>
      <div className="shell-task-meta">
        {task.branch ? (
          <span>
            <GitBranch size={11} /> {task.branch}
          </span>
        ) : null}
        {task.worktreePath ? <span title={task.worktreePath}>worktree</span> : null}
        {task.prUrl ? (
          <a href={task.prUrl} target="_blank" rel="noreferrer">
            <GitPullRequest size={11} /> PR
          </a>
        ) : null}
      </div>
      <div className="shell-task-actions">
        <select
          value={task.status}
          onChange={(event) => void tasks.setStatus(task, event.target.value as TaskStatus)}
          aria-label={`Estado de ${task.title}`}
        >
          {columns.map((column) => (
            <option key={column.id} value={column.id}>
              {column.label}
            </option>
          ))}
        </select>
        {canAdvance ? (
          <button
            type="button"
            className="shell-task-advance"
            title="Avanzar estado"
            onClick={() => void tasks.setStatus(task, nextStatus[task.status])}
          >
            <ArrowRight size={12} />
          </button>
        ) : null}
        <button
          type="button"
          className={cn("shell-task-remove", task.status === "canceled" && "is-visible")}
          title="Eliminar tarea"
          onClick={() => {
            if (window.confirm(`¿Eliminar la tarea "${task.title}"?`)) {
              void tasks.remove(task.id);
            }
          }}
        >
          <Trash2 size={12} />
        </button>
      </div>
    </article>
  );
}
