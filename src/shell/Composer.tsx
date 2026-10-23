import { useState } from "react";
import { ArrowUp, AtSign, Paperclip, Sparkles } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { Session } from "../stores/sessionsStore";

type ComposerProps = {
  session: Session | null;
  onNewSession: () => void;
};

export default function Composer({ session, onNewSession }: ComposerProps) {
  const [value, setValue] = useState("");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function send() {
    const text = value.trim();
    if (!session || !text || sending) {
      return;
    }
    setSending(true);
    setError(null);
    try {
      const bytes = Array.from(new TextEncoder().encode(`${text}\n`));
      await invoke("session_write", { id: session.id, data: bytes });
      setValue("");
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setSending(false);
    }
  }

  return (
    <div className="shell-composer-wrap">
      <div className="shell-composer">
        <div className="shell-composer-top">
          <span className="shell-composer-context">
            <Sparkles size={13} />
            {session ? session.title : "Sin sesión activa"}
          </span>
          <span className="shell-composer-hint">
            {session ? "Escribí para el agente" : "Creá una sesión para empezar"}
          </span>
        </div>
        <textarea
          value={value}
          onChange={(event) => setValue(event.target.value)}
          onKeyDown={(event) => {
            if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
              event.preventDefault();
              void send();
            }
          }}
          placeholder={session ? "Pedile algo al agente…" : "Nueva sesión…"}
          rows={2}
          spellCheck={false}
          disabled={!session}
        />
        <div className="shell-composer-actions">
          <div className="shell-composer-tools">
            <button
              type="button"
              className="icon-button icon-button-small"
              disabled
              title="Adjuntos próximamente"
            >
              <Paperclip size={14} />
            </button>
            <button
              type="button"
              className="icon-button icon-button-small"
              disabled
              title="Menciones próximamente"
            >
              <AtSign size={14} />
            </button>
            <span className="shell-composer-driver">
              {session ? session.spec.driverId : "sin driver"}
            </span>
          </div>
          {session ? (
            <button
              type="button"
              className="shell-send"
              onClick={() => void send()}
              disabled={!value.trim() || sending}
              aria-label="Enviar"
            >
              <ArrowUp size={15} />
            </button>
          ) : (
            <button type="button" className="shell-send-text" onClick={onNewSession}>
              Nueva sesión
            </button>
          )}
        </div>
        {error ? <p className="shell-error">{error}</p> : null}
      </div>
    </div>
  );
}
