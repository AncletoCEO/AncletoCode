import { useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
import { ArrowLeftRight, MessageSquare, Pause, Play, Square } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { captureTranscriptAnchor } from "../../third_party/pandacode/src/transcript-scroll.ts";
import Markdown from "./Markdown";
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
  const [turnBudgetSeconds, setTurnBudgetSeconds] = useState(60);
  const [conversation, setConversation] = useState<Conversation | null>(null);
  const [turns, setTurns] = useState<Turn[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const unlisteners = useRef<UnlistenFn[]>([]);
  const listRef = useRef<HTMLOListElement>(null);

  const effectiveA = sessionA || (projectSessions[0]?.id ?? "");
  const effectiveB = sessionB || (projectSessions[1]?.id ?? "");

  useEffect(() => {
    void (async () => {
      try {
        const [turns, budget] = await Promise.all([
          invoke<string | null>("settings_get", { key: "conversation.maxTurns" }),
          invoke<string | null>("settings_get", { key: "conversation.turnBudgetMs" }),
        ]);
        if (turns && Number(turns) > 0) {
          setMaxTurns(Number(turns));
        }
        if (budget && Number(budget) >= 1000) {
          setTurnBudgetSeconds(Math.round(Number(budget) / 1000));
        }
      } catch {
        // Sin Tauri se usan los valores por defecto.
      }
    })();
  }, []);

  const conversationId = conversation?.id;

  useEffect(() => {
    if (!conversationId) {
      return;
    }
    const id = conversationId;
    let disposed = false;

    void listen<Turn>(`conversation://turn/${id}`, (event) => {
      const restore = listRef.current ? captureTranscriptAnchor(listRef.current) : null;
      setTurns((current) => [...current, event.payload]);
      requestAnimationFrame(() => restore?.());
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
        turnBudgetMs: turnBudgetSeconds * 1000,
      });
      await Promise.all([
        invoke("settings_put", { key: "conversation.maxTurns", value: String(maxTurns) }),
        invoke("settings_put", {
          key: "conversation.turnBudgetMs",
          value: String(turnBudgetSeconds * 1000),
        }),
      ]).catch(() => undefined);
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

  function reset() {
    if (running || paused) {
      return;
    }
    setConversation(null);
    setTurns([]);
    setError(null);
  }

  const stateLabel: Record<ConversationState, string> = {
    running: "En curso",
    paused: "Pausada",
    completed: "Completada",
    aborted: "Abortada",
    failed: "Falló",
  };

  return (
    <section className="conversation-screen">
      <header className="conversation-screen-head">
        <div>
          <span className="conversation-eyebrow">Orquestación</span>
          <h1>Conversación</h1>
          <p>Dos agentes trabajan sobre un objetivo común, con turnos y límites explícitos.</p>
        </div>
        <span
          className={
            conversation
              ? `conversation-state-badge ${conversation.state}`
              : "conversation-state-badge"
          }
        >
          {conversation ? stateLabel[conversation.state] : "Sin iniciar"}
        </span>
      </header>

      {error ? <p className="shell-error conversation-error">{error}</p> : null}

      {!conversation ? (
        <div className="conversation-setup-layout">
          <div className="conversation-setup-card">
            <div className="conversation-card-title">Configuración</div>
            <label className="conversation-field">
              <span>Agente A</span>
              <select
                value={effectiveA}
                onChange={(event) => setSessionA(event.target.value)}
                aria-label="Sesión A"
              >
                <option value="">Elegir sesión</option>
                {projectSessions.map((session) => (
                  <option key={session.id} value={session.id}>
                    {session.title}
                  </option>
                ))}
              </select>
            </label>
            <label className="conversation-field">
              <span>Agente B</span>
              <select
                value={effectiveB}
                onChange={(event) => setSessionB(event.target.value)}
                aria-label="Sesión B"
              >
                <option value="">Elegir sesión</option>
                {projectSessions.map((session) => (
                  <option key={session.id} value={session.id}>
                    {session.title}
                  </option>
                ))}
              </select>
            </label>
            <label className="conversation-field">
              <span>Objetivo</span>
              <textarea
                value={objective}
                onChange={(event) => setObjective(event.target.value)}
                placeholder="¿Qué tienen que resolver entre los dos?"
                aria-label="Objetivo"
                rows={4}
              />
            </label>
            <div className="conversation-limits">
              <label>
                <span>Turnos</span>
                <input
                  type="number"
                  min={1}
                  max={50}
                  value={maxTurns}
                  onChange={(event) => setMaxTurns(Number(event.target.value))}
                  aria-label="Máximo de turnos"
                />
              </label>
              <label>
                <span>Timeout / turno</span>
                <input
                  type="number"
                  min={5}
                  max={600}
                  value={turnBudgetSeconds}
                  onChange={(event) => setTurnBudgetSeconds(Number(event.target.value))}
                  aria-label="Timeout por turno (segundos)"
                />
                <small>segundos</small>
              </label>
            </div>
            <button
              type="button"
              className="shell-primary-button conversation-start-button"
              onClick={() => void start()}
              disabled={busy || projectSessions.length < 2}
            >
              {busy ? "Iniciando…" : "Iniciar conversación"}
            </button>
            {projectSessions.length < 2 ? (
              <p className="conversation-hint">
                Necesitás al menos dos sesiones de agente en este proyecto.
              </p>
            ) : null}
          </div>

          <div className="conversation-empty-panel">
            <MessageSquare size={26} />
            <strong>Dos agentes, un objetivo</strong>
            <p>
              Elegí quién empieza y quién responde. Cada turno queda registrado con su modo y
              duración; podés pausar o abortar en cualquier momento.
            </p>
            <div className="conversation-flow">
              <span>{effectiveA ? labelOf(projectSessions, effectiveA) : "Agente A"}</span>
              <ArrowLeftRight size={14} />
              <span>{effectiveB ? labelOf(projectSessions, effectiveB) : "Agente B"}</span>
            </div>
          </div>
        </div>
      ) : (
        <div className="conversation-live">
          <div className="conversation-toolbar">
            <div className="conversation-live-meta">
              <span className={`status-dot ${running ? "busy" : conversation.state}`} />
              <strong>{stateLabel[conversation.state]}</strong>
              <span>{turns.length} turnos</span>
              {conversation.reason ? <span>({conversation.reason})</span> : null}
            </div>
            <div className="conversation-controls">
              <button type="button" onClick={() => void control("pause")} disabled={!running}>
                <Pause size={12} /> Pausar
              </button>
              <button type="button" onClick={() => void control("resume")} disabled={!paused}>
                <Play size={12} /> Reanudar
              </button>
              <button
                type="button"
                onClick={() => void control("abort")}
                disabled={!running && !paused}
              >
                <Square size={11} /> Abortar
              </button>
              <button type="button" onClick={reset} disabled={running || paused}>
                Nueva
              </button>
            </div>
          </div>
          <div className="conversation-objective-bar">
            <span>Objetivo</span>
            <p>{conversation.objective}</p>
          </div>
          <ol className="conversation-turns" ref={listRef}>
            {turns.map((turn) => (
              <li key={`${turn.conversationId}-${turn.index}`} className="conversation-turn">
                <div className="conversation-turn-head">
                  <span className="conversation-turn-index">#{turn.index}</span>
                  <strong>{labelOf(projectSessions, turn.fromSession)}</strong>
                  <span>→</span>
                  <span>{labelOf(projectSessions, turn.toSession)}</span>
                  <span className="conversation-turn-meta">
                    {turn.mode} · {turn.durationMs} ms
                  </span>
                </div>
                <Markdown value={turn.text} />
              </li>
            ))}
            {turns.length === 0 ? (
              <li className="conversation-waiting">Esperando el primer turno…</li>
            ) : null}
          </ol>
        </div>
      )}
    </section>
  );
}

function labelOf(sessions: { id: string; title: string }[], id: string): string {
  return sessions.find((session) => session.id === id)?.title ?? id.slice(0, 8);
}
