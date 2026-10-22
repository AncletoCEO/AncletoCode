import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type ToolStatus = "available" | "missing" | "incompatible" | "error";

type WriterState = {
  tool: {
    status: ToolStatus;
    version: string | null;
    binaryPath: string | null;
    installHint: string | null;
  };
  check: { status: string; exitCode: number } | null;
  manifest: {
    schemaVersion: number;
    writerVersion: string | null;
    generatedAt: string | null;
    gitCommit: string | null;
    stats: Record<string, unknown>;
    outputs: unknown[];
  } | null;
  compatible: boolean;
  graph: { nodes: number; edges: number } | null;
  facts: Record<string, unknown>;
  languagePack: { installed: boolean; enabled: boolean; module: string };
  outputDir: string;
  humanDir: string;
};

type CommandOutcome = {
  success: boolean;
  exitCode: number | null;
  stdout: string;
  stderr: string;
};

type WriterPanelProps = {
  projectId: string;
};

const agentDocs = ["INDEX.md", "API.md", "SYMBOLS.md", "GOTCHAS.md", "ARCHITECTURE.md"];

export default function WriterPanel({ projectId }: WriterPanelProps) {
  const [state, setState] = useState<WriterState | null>(null);
  const [output, setOutput] = useState<string | null>(null);
  const [document, setDocument] = useState<string | null>(null);
  const [docName, setDocName] = useState(agentDocs[0] ?? "INDEX.md");
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    setBusy(true);
    try {
      setState(await invoke<WriterState>("writer_status", { id: projectId }));
    } catch (error) {
      setOutput(messageOf(error));
    } finally {
      setBusy(false);
    }
  }, [projectId]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    void refresh();
  }, [refresh]);

  async function command(label: string, run: () => Promise<CommandOutcome>) {
    setBusy(true);
    setOutput(null);
    try {
      const outcome = await run();
      const text = (outcome.stdout || outcome.stderr).trim();
      setOutput(
        `${label}: ${outcome.success ? "ok" : `falló (${outcome.exitCode ?? "?"})`}\n${text}`,
      );
      await refresh();
    } catch (error) {
      setOutput(`${label}: ${messageOf(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function toggleLanguagePack() {
    if (!state) {
      return;
    }
    setBusy(true);
    try {
      await invoke("writer_set_language_pack", {
        id: projectId,
        enabled: !state.languagePack.enabled,
      });
      await refresh();
    } catch (error) {
      setOutput(messageOf(error));
    } finally {
      setBusy(false);
    }
  }

  async function openHuman() {
    try {
      await invoke("writer_open_human", { id: projectId });
    } catch (error) {
      setOutput(messageOf(error));
    }
  }

  async function loadDocument() {
    setBusy(true);
    try {
      setDocument(await invoke<string>("writer_agent_doc", { id: projectId, name: docName }));
    } catch (error) {
      setOutput(messageOf(error));
    } finally {
      setBusy(false);
    }
  }

  async function copyDocument() {
    if (!document) {
      return;
    }
    try {
      await navigator.clipboard.writeText(document);
      setOutput(`${docName}: copiado al portapapeles`);
    } catch (error) {
      setOutput(messageOf(error));
    }
  }

  return (
    <section className="spec-panel writer-panel">
      <div className="spec-status">
        <span
          className={`status-dot ${state?.tool.status === "available" ? "ready" : "crashed"}`}
        />
        <span>ancleto-writer {state?.tool.version ?? state?.tool.status ?? "—"}</span>
        {state?.check ? (
          <span>
            frescura: {state.check.status} <span className="muted">({state.check.exitCode})</span>
          </span>
        ) : null}
        {state?.tool.status === "missing" && state.tool.installHint ? (
          <code>{state.tool.installHint}</code>
        ) : null}
        <button type="button" onClick={() => void refresh()} disabled={busy}>
          refrescar
        </button>
      </div>

      {state?.manifest ? (
        <div className="spec-check">
          <span className="spec-label">manifest</span>
          <span>schema {state.manifest.schemaVersion}</span>
          {!state.compatible ? <span className="error">incompatible</span> : null}
          {Object.entries(state.manifest.stats).map(([key, value]) => (
            <span key={key} className="muted">
              {key} {String(value)}
            </span>
          ))}
          {state.graph ? (
            <span className="muted">
              {state.graph.nodes} nodos · {state.graph.edges} aristas
            </span>
          ) : null}
          {state.manifest.gitCommit ? (
            <span className="muted">git {state.manifest.gitCommit.slice(0, 7)}</span>
          ) : null}
        </div>
      ) : (
        <p className="muted">
          Sin conocimiento publicado en {state?.outputDir ?? ".ancleto/writer"}.
        </p>
      )}

      <div className="spec-actions">
        <button
          type="button"
          onClick={() =>
            void command("init", () => invoke<CommandOutcome>("writer_init", { id: projectId }))
          }
        >
          init
        </button>
        <button
          type="button"
          onClick={() =>
            void command("build", () => invoke<CommandOutcome>("writer_build", { id: projectId }))
          }
        >
          build
        </button>
        <button
          type="button"
          onClick={() =>
            void command("check", () => invoke<CommandOutcome>("writer_check", { id: projectId }))
          }
        >
          check
        </button>
        <button
          type="button"
          onClick={() =>
            void command("lint", () => invoke<CommandOutcome>("writer_lint", { id: projectId }))
          }
        >
          lint
        </button>
        <button
          type="button"
          onClick={() =>
            void command("doctor", () => invoke<CommandOutcome>("writer_doctor", { id: projectId }))
          }
        >
          doctor
        </button>
        <button
          type="button"
          onClick={() =>
            void command("human", () => invoke<CommandOutcome>("writer_human", { id: projectId }))
          }
        >
          vista humana
        </button>
        <button type="button" onClick={() => void openHuman()}>
          abrir vista
        </button>
      </div>

      <div className="spec-discovery">
        <span className="spec-label">language pack</span>
        <span>{state?.languagePack.module ?? "—"}</span>
        <span className={state?.languagePack.installed ? "muted" : "error"}>
          {state?.languagePack.installed ? "instalado" : "ausente"}
        </span>
        <span className="muted">
          {state?.languagePack.enabled ? "habilitado" : "deshabilitado"}
        </span>
        <button type="button" onClick={() => void toggleLanguagePack()} disabled={busy}>
          {state?.languagePack.enabled ? "deshabilitar" : "habilitar"}
        </button>
        {!state?.languagePack.installed ? (
          <code>npm install @ancleto/writer @ancleto/writer-languages --legacy-peer-deps</code>
        ) : null}
      </div>

      {state ? (
        <div className="spec-memory">
          <span className="spec-label">hechos</span>
          {Object.entries(state.facts).map(([name, value]) => (
            <span key={name} className="muted">
              {name} {summarize(value)}
            </span>
          ))}
          {godNodes(state.facts).length > 0 ? (
            <ul>
              {godNodes(state.facts).map((node) => (
                <li key={node}>{node}</li>
              ))}
            </ul>
          ) : null}
        </div>
      ) : null}

      <div className="spec-actions">
        <select value={docName} onChange={(event) => setDocName(event.target.value)}>
          {agentDocs.map((name) => (
            <option key={name} value={name}>
              {name}
            </option>
          ))}
        </select>
        <button type="button" onClick={() => void loadDocument()} disabled={busy}>
          ver documento
        </button>
        <button type="button" onClick={() => void copyDocument()} disabled={!document}>
          adjuntar al contexto
        </button>
      </div>

      {document ? <pre className="spec-context">{document}</pre> : null}
      {output ? <pre className="spec-output">{output}</pre> : null}
    </section>
  );
}

function summarize(value: unknown): string {
  if (Array.isArray(value)) {
    return String(value.length);
  }
  if (value && typeof value === "object") {
    const array = Object.values(value).find((entry) => Array.isArray(entry));
    if (Array.isArray(array)) {
      return String(array.length);
    }
    return "·";
  }
  return "";
}

function godNodes(facts: Record<string, unknown>): string[] {
  const nodes = facts["god-nodes"];
  if (!Array.isArray(nodes)) {
    return [];
  }
  return nodes
    .slice(0, 5)
    .map((node) =>
      node && typeof node === "object" && "path" in node ? String(node.path) : String(node),
    );
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
