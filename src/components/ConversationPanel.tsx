import { useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Project } from "../stores/projectsStore";
import {
  getSnapshot as getSessions,
  subscribe as subscribeSessions,
} from "../stores/sessionsStore";

type ConversationState = "running" | "paused" | "completed" | "aborted" | "failed";

type Conversation = {
  id: string;
  sessionA: string;
  sessionB: string;
  objective: string;
  maxTurns: number;
  turnBudgetMs: number;
  state: ConversationState;
  reason: string | null;
  createdAt: number;
};

type Turn = {
  conversationId: string;
  index: number;
  fromSession: string;
  toSession: string;
  text: string;
  mode: "oneShot" | "pty";
  durationMs: number;
  createdAt: number;
};

type StatePayload = {
  state: ConversationState;
  reason: string | null;
};

type ConversationPanelProps = {
  project: Project;
};

export default function ConversationPanel({ project }: ConversationPanelProps) {
  const sessions = useSyncExternalStore(subscribeSessions, getSessions);
  const projectSessions = useMemo(
    () => sessions.sessions.filter((session) => session.projectId === project.id),
    [sessions.sessions, project.id],
  );
  const [sessionA, setSessionA] = useState("");
  const [sessionB, setSessionB] = useState("");
  const [objective, setObjective] = useState("");
  const [maxTurns, setMaxTurns] = useState(6);
  const [conversation, setConversation] = useState<Conversation | null>(null);
  const [turns, setTurns] = useState<Turn[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const unlisteners = useRef<UnlistenFn[]>([]);

  const effectiveA = sessionA || (projectSessions[0]?.id ?? "");
  const effectiveB = sessionB || (projectSessions[1]?.id ?? "");

  const conversationId = conversation?.id;

  useEffect(() => {
    if (!conversationId) {
      return;
    }
    const id = conversationId;
    let disposed = false;

    void listen<Turn>(`conversation://turn/${id}`, (event) => {
      setTurns((current) => [...current, event.payload]);
    }).then((stop) => {
      if (disposed) {
        stop();
      } else {
        unlisteners.current.push(stop);
      }
    });

    void listen<StatePayload>(`conversation://state/${id}`, (event) => {
      setConversation((current) =>
        current
          ? { ...current, state: event.payload.state, reason: event.payload.reason }
          : current,
      );
    }).then((stop) => {
      if (disposed) {
        stop();
      } else {
        unlisteners.current.push(stop);
      }
    });

    return () => {
      disposed = true;
      for (const stop of unlisteners.current) {
        stop();
      }
      unlisteners.current = [];
    };
  }, [conversationId]);

  async function start() {
    if (!effectiveA || !effectiveB || effectiveA === effectiveB || !objective.trim()) {
      setError("Elegí dos sesiones distintas y un objetivo.");
      return;
    }
    setBusy(true);
    setError(null);
    setTurns([]);
    try {
      const created = await invoke<Conversation>("conversation_start", {
        sessionA: effectiveA,
        sessionB: effectiveB,
        objective,
        maxTurns,
      });
      setConversation(created);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setBusy(false);
    }
  }

  async function control(action: "pause" | "resume" | "abort") {
    if (!conversation) {
      return;
    }
    try {
      await invoke(`conversation_${action}`, { id: conversation.id });
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    }
  }

  const running = conversation?.state === "running";
  const paused = conversation?.state === "paused";

  return (
    <section className="conversation">
      <div className="conversation-form">
        <select
          value={effectiveA}
          onChange={(event) => setSessionA(event.target.value)}
          aria-label="Sesión A"
        >
          <option value="">sesión A</option>
          {projectSessions.map((session) => (
            <option key={session.id} value={session.id}>
              {session.title}
            </option>
          ))}
        </select>
        <select
          value={effectiveB}
          onChange={(event) => setSessionB(event.target.value)}
          aria-label="Sesión B"
        >
          <option value="">sesión B</option>
          {projectSessions.map((session) => (
            <option key={session.id} value={session.id}>
              {session.title}
            </option>
          ))}
        </select>
        <input
          type="number"
          min={1}
          max={50}
          value={maxTurns}
          onChange={(event) => setMaxTurns(Number(event.target.value))}
          aria-label="Máximo de turnos"
        />
        <button type="button" onClick={() => void start()} disabled={busy}>
          iniciar
        </button>
      </div>
      <textarea
        className="conversation-objective"
        value={objective}
        onChange={(event) => setObjective(event.target.value)}
        placeholder="Objetivo de la conversación"
        aria-label="Objetivo"
        rows={2}
      />
      {error ? <p className="error">{error}</p> : null}
      {conversation ? (
        <>
          <div className="conversation-state">
            <span className={`status-dot ${running ? "busy" : conversation.state}`} />
            <span>{conversation.state}</span>
            {conversation.reason ? <span className="muted">({conversation.reason})</span> : null}
            <span className="muted">{turns.length} turnos</span>
            <button type="button" onClick={() => void control("pause")} disabled={!running}>
              pausar
            </button>
            <button type="button" onClick={() => void control("resume")} disabled={!paused}>
              reanudar
            </button>
            <button
              type="button"
              onClick={() => void control("abort")}
              disabled={!running && !paused}
            >
              abortar
            </button>
          </div>
          <ol className="turn-list">
            {turns.map((turn) => (
              <li key={`${turn.conversationId}-${turn.index}`} className="turn">
                <div className="turn-meta">
                  #{turn.index} {labelOf(projectSessions, turn.fromSession)} →{" "}
                  {labelOf(projectSessions, turn.toSession)} · {turn.mode} · {turn.durationMs} ms
                </div>
                <pre className="turn-text">{turn.text}</pre>
              </li>
            ))}
          </ol>
        </>
      ) : null}
    </section>
  );
}

function labelOf(sessions: { id: string; title: string }[], id: string): string {
  return sessions.find((session) => session.id === id)?.title ?? id.slice(0, 8);
}
