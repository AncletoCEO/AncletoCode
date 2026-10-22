import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type SyncResult = {
  success: boolean;
  exitCode: number | null;
  stdout: string;
  stderr: string;
};

type ProjectActionsProps = {
  projectId: string;
};

export default function ProjectActions({ projectId }: ProjectActionsProps) {
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function openInAntigravity() {
    setMessage(null);
    try {
      await invoke("ide_open_antigravity", { id: projectId });
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  async function syncAssets() {
    setBusy(true);
    setMessage(null);
    try {
      const result = await invoke<SyncResult>("ide_sync_antigravity", { id: projectId });
      const output = (result.stdout || result.stderr).trim();
      setMessage(
        result.success
          ? output || "Assets sincronizados."
          : `Falló (código ${result.exitCode ?? "?"}): ${output}`,
      );
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="project-actions">
      <button type="button" onClick={() => void openInAntigravity()}>
        abrir Antigravity
      </button>
      <button type="button" onClick={() => void syncAssets()} disabled={busy}>
        sincronizar assets
      </button>
      {message ? <span className="muted project-action-message">{message}</span> : null}
    </div>
  );
}
