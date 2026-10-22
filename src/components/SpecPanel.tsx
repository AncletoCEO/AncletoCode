import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type ToolStatus = "available" | "missing" | "incompatible" | "error";

type SpecProjectState = {
  initialized: boolean;
  host: string | null;
  tier: string | null;
  language: string | null;
  discoveryExclude: string[];
  discoveryOutputDir: string | null;
  activeChanges: { name: string; artifacts: string[] }[];
  tool: {
    status: ToolStatus;
    version: string | null;
    binaryPath: string | null;
    installHint: string | null;
  };
  node: {
    available: boolean;
    version: string | null;
    binaryPath: string | null;
    compatible: boolean;
  };
};

type DiscoveryState = {
  schemaVersion: number;
  state: string;
  impact: string;
  recommendedAction: string | null;
  message: string | null;
  missingDocs: string[];
  changedAreas: string[];
};

type SpecsCheck = {
  ok: boolean;
  scanned: number;
  nonCanonical: unknown[];
};

type MemoryNode = {
  memory_key: string;
  type: string;
  scope: string;
  content: string;
  status?: string | null;
};

type CommandOutcome = {
  success: boolean;
  exitCode: number | null;
  stdout: string;
  stderr: string;
};

type InitOptions = {
  agent?: string;
  tier?: string;
  language?: string;
  exclude?: string[];
};

type SpecPanelProps = {
  projectId: string;
};

const hosts = [
  "opencode",
  "claude",
  "vscode",
  "antigravity",
  "cursor",
  "roo",
  "copilot",
  "commandcode",
];
const tiers = ["normal", "minimo", "gratis"];
const languages = ["auto", "es", "en", "pt"];

export default function SpecPanel({ projectId }: SpecPanelProps) {
  const [status, setStatus] = useState<SpecProjectState | null>(null);
  const [discovery, setDiscovery] = useState<DiscoveryState | null>(null);
  const [specsCheck, setSpecsCheck] = useState<SpecsCheck | null>(null);
  const [memory, setMemory] = useState<MemoryNode[] | null>(null);
  const [context, setContext] = useState<string | null>(null);
  const [output, setOutput] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const [agent, setAgent] = useState("");
  const [tier, setTier] = useState("");
  const [language, setLanguage] = useState("");
  const [exclude, setExclude] = useState("");

  const effectiveAgent = agent || status?.host || "";
  const effectiveTier = tier || status?.tier || "";
  const effectiveLanguage = language || status?.language || "";
  const effectiveExclude = exclude || (status?.discoveryExclude ?? []).join(", ");

  function options(): InitOptions {
    const result: InitOptions = {
      exclude: effectiveExclude
        .split(",")
        .map((entry) => entry.trim())
        .filter(Boolean),
    };
    if (effectiveAgent) {
      result.agent = effectiveAgent;
    }
    if (effectiveTier) {
      result.tier = effectiveTier;
    }
    if (effectiveLanguage) {
      result.language = effectiveLanguage;
    }
    return result;
  }

  const refresh = useCallback(async () => {
    setBusy(true);
    try {
      const next = await invoke<SpecProjectState>("spec_status", { id: projectId });
      setStatus(next);
      if (next.initialized) {
        const checked = await invoke<DiscoveryState>("spec_discovery_check", { id: projectId });
        setDiscovery(checked);
      }
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
      await refreshAfterChange();
    } catch (error) {
      setOutput(`${label}: ${messageOf(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function refreshAfterChange() {
    const next = await invoke<SpecProjectState>("spec_status", { id: projectId });
    setStatus(next);
  }

  async function initialize() {
    if (!window.confirm("¿Inicializar @ancleto/spec en este proyecto?")) {
      return;
    }
    await command("init", () =>
      invoke<CommandOutcome>("spec_init", { id: projectId, options: options() }),
    );
  }

  async function applySettings() {
    await command("install", () =>
      invoke<CommandOutcome>("spec_install", { id: projectId, options: options() }),
    );
  }

  async function checkSpecs() {
    setBusy(true);
    try {
      setSpecsCheck(await invoke<SpecsCheck>("spec_specs_check", { id: projectId }));
    } catch (error) {
      setOutput(`specs check: ${messageOf(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function regenerateDiscovery() {
    if (!window.confirm("¿Regenerar el technical seed con @ancleto/spec?")) {
      return;
    }
    await command("discovery", () =>
      invoke<CommandOutcome>("spec_discovery_generate", { id: projectId }),
    );
    try {
      setDiscovery(await invoke<DiscoveryState>("spec_discovery_check", { id: projectId }));
    } catch (error) {
      setOutput(`discovery: ${messageOf(error)}`);
    }
  }

  async function loadMemory() {
    setBusy(true);
    try {
      setMemory(await invoke<MemoryNode[]>("spec_memory_list", { id: projectId }));
      setContext(await invoke<string>("spec_memory_context", { id: projectId }));
    } catch (error) {
      setOutput(`memoria: ${messageOf(error)}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="spec-panel">
      <div className="spec-status">
        <span
          className={`status-dot ${status?.tool.status === "available" ? "ready" : "crashed"}`}
        />
        <span>ancleto {status?.tool.version ?? status?.tool.status ?? "—"}</span>
        <span className="muted">node {status?.node.version ?? "no disponible"}</span>
        {status?.tool.status === "missing" && status.tool.installHint ? (
          <code>{status.tool.installHint}</code>
        ) : null}
        {status?.node.available && !status.node.compatible ? (
          <span className="error">Node ≥ 24 requerido</span>
        ) : null}
        <button type="button" onClick={() => void refresh()} disabled={busy}>
          refrescar
        </button>
      </div>

      {status && !status.initialized ? <p className="muted">Proyecto sin `.ancletorc`.</p> : null}

      <div className="spec-settings">
        <label>
          host
          <select value={effectiveAgent} onChange={(event) => setAgent(event.target.value)}>
            <option value="">—</option>
            {hosts.map((host) => (
              <option key={host} value={host}>
                {host}
              </option>
            ))}
          </select>
        </label>
        <label>
          tier
          <select value={effectiveTier} onChange={(event) => setTier(event.target.value)}>
            <option value="">—</option>
            {tiers.map((entry) => (
              <option key={entry} value={entry}>
                {entry}
              </option>
            ))}
          </select>
        </label>
        <label>
          idioma
          <select value={effectiveLanguage} onChange={(event) => setLanguage(event.target.value)}>
            <option value="">—</option>
            {languages.map((entry) => (
              <option key={entry} value={entry}>
                {entry}
              </option>
            ))}
          </select>
        </label>
        <label className="spec-exclude">
          exclude
          <input value={effectiveExclude} onChange={(event) => setExclude(event.target.value)} />
        </label>
        <button
          type="button"
          onClick={() => void (status?.initialized ? applySettings() : initialize())}
          disabled={busy}
        >
          {status?.initialized ? "aplicar" : "inicializar"}
        </button>
      </div>

      {status?.initialized ? (
        <div className="spec-actions">
          <button
            type="button"
            onClick={() =>
              void command("check", () => invoke<CommandOutcome>("spec_check", { id: projectId }))
            }
          >
            check
          </button>
          <button
            type="button"
            onClick={() =>
              void command("update", () => invoke<CommandOutcome>("spec_update", { id: projectId }))
            }
          >
            update
          </button>
          <button
            type="button"
            onClick={() =>
              void command("upgrade", () =>
                invoke<CommandOutcome>("spec_upgrade", { id: projectId }),
              )
            }
          >
            upgrade
          </button>
          <button
            type="button"
            onClick={() =>
              void command("doctor", () => invoke<CommandOutcome>("spec_doctor", { id: projectId }))
            }
          >
            doctor
          </button>
          <button type="button" onClick={() => void checkSpecs()}>
            specs check
          </button>
          <button type="button" onClick={() => void loadMemory()}>
            memoria
          </button>
        </div>
      ) : null}

      {status?.activeChanges.length ? (
        <div className="spec-changes">
          <div className="spec-label">changes activos</div>
          <ul>
            {status.activeChanges.map((change) => (
              <li key={change.name}>
                {change.name} <span className="muted">{change.artifacts.join(" · ")}</span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}

      {discovery ? (
        <div className="spec-discovery">
          <span className="spec-label">discovery</span>
          <span>{discovery.state}</span>
          <span className="muted">impact {discovery.impact}</span>
          {discovery.message ? <span className="muted">{discovery.message}</span> : null}
          {discovery.state !== "READY" ? (
            <button type="button" onClick={() => void regenerateDiscovery()} disabled={busy}>
              regenerar
            </button>
          ) : null}
        </div>
      ) : null}

      {specsCheck ? (
        <div className="spec-check">
          <span className="spec-label">specs check</span>
          <span>{specsCheck.ok ? "ok" : "con errores"}</span>
          <span className="muted">{specsCheck.scanned} escaneados</span>
          {specsCheck.nonCanonical.length > 0 ? (
            <span className="error">{specsCheck.nonCanonical.length} no canónicos</span>
          ) : null}
        </div>
      ) : null}

      {memory ? (
        <div className="spec-memory">
          <span className="spec-label">memoria</span>
          <span className="muted">{memory.length} nodos</span>
          <ul>
            {memory.map((node) => (
              <li key={node.memory_key}>
                <strong>{node.memory_key}</strong> <span className="muted">{node.type}</span>{" "}
                {node.content.slice(0, 120)}
              </li>
            ))}
          </ul>
          {context ? <pre className="spec-context">{context}</pre> : null}
        </div>
      ) : null}

      {output ? <pre className="spec-output">{output}</pre> : null}
    </section>
  );
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
