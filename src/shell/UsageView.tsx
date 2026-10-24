import { useEffect, useState } from "react";
import { Activity, RefreshCw } from "lucide-react";
import { freshnessLabel } from "../lib/usage";
import { acquireUsageEvents, useUsageStore, type ProviderUsage } from "../stores/usageStore";
import { cn } from "../ui/cn";

const statusLabel: Record<ProviderUsage["status"], string> = {
  ready: "Con datos",
  unavailable: "No disponible",
  unsupported: "Sin cuota expuesta",
  signedOut: "Sin sesión",
  error: "Error",
};

const placeholders = [
  { id: "opencode", label: "OpenCode" },
  { id: "claude", label: "Claude Code" },
  { id: "antigravity", label: "Antigravity" },
  { id: "commandcode", label: "Command Code" },
];

function compact(value: number): string {
  return new Intl.NumberFormat("es", { notation: "compact", maximumFractionDigits: 1 }).format(
    value,
  );
}

function resetLabel(value: string): string {
  const timestamp = /^\d+$/.test(value) ? Number(value) : Date.parse(value);
  return Number.isFinite(timestamp) ? new Date(timestamp).toLocaleString() : value;
}

export default function UsageView() {
  const usage = useUsageStore();
  const load = usage.load;
  const [freshness, setFreshness] = useState("");

  useEffect(() => {
    let release: (() => void) | undefined;
    let disposed = false;
    void acquireUsageEvents().then((dispose) => {
      if (disposed) {
        dispose();
      } else {
        release = dispose;
      }
    });
    void load(false);
    const updateFreshness = () => setFreshness(freshnessLabel(usage.checkedAt, Date.now()));
    const initial = window.setTimeout(updateFreshness, 0);
    const ticker = window.setInterval(updateFreshness, 30_000);
    return () => {
      disposed = true;
      release?.();
      window.clearTimeout(initial);
      window.clearInterval(ticker);
    };
  }, [load, usage.checkedAt]);

  const pending = usage.refreshing
    ? placeholders.filter(
        (entry) => !usage.providers.some((provider) => provider.provider === entry.id),
      )
    : [];

  return (
    <div className="shell-usage-view">
      <header className="shell-usage-head">
        <div>
          <h1>
            <Activity size={19} /> Uso de proveedores
          </h1>
          <p>
            Lectura local. No se envían prompts; las cuotas compartidas pueden incluir otras apps.
            <span className="shell-usage-freshness">
              {usage.stale && usage.checkedAt > 0 ? "datos vencidos · " : ""}
              {freshness || "sin datos"}
            </span>
          </p>
        </div>
        <button
          type="button"
          className="shell-ghost-button"
          onClick={() => void load(true)}
          disabled={usage.refreshing}
        >
          <RefreshCw size={13} className={usage.refreshing ? "animate-spin" : ""} />
          {usage.refreshing ? "Actualizando" : "Actualizar"}
        </button>
      </header>

      {usage.error ? <p className="shell-error">{usage.error}</p> : null}

      <div className="shell-usage-grid" aria-busy={usage.refreshing}>
        {usage.providers.map((provider) => (
          <article key={provider.provider} className="shell-usage-card">
            <div className="shell-usage-card-head">
              <div>
                <strong>{provider.label}</strong>
                <span>{provider.source ?? "Fuente local"}</span>
              </div>
              <span className={cn("shell-usage-status", provider.status)}>
                {statusLabel[provider.status]}
              </span>
            </div>

            {provider.cost != null ? (
              <div className="shell-usage-cost">
                <strong>${provider.cost.toFixed(2)}</strong>
                <span>estimado últimos 30 días</span>
              </div>
            ) : null}

            {provider.tokens != null ? (
              <div className="shell-usage-meta">
                <span>{compact(provider.tokens)} tokens</span>
                {provider.sessions != null ? <span>{provider.sessions} sesiones</span> : null}
              </div>
            ) : null}

            {provider.windows.length > 0 ? (
              <div className="shell-usage-windows">
                {provider.windows.map((window) => (
                  <div key={window.label} className="shell-usage-window">
                    <div>
                      <span>{window.label}</span>
                      <strong>{Math.round(window.usedPercent)}%</strong>
                    </div>
                    <span className="shell-usage-track">
                      <span
                        className={cn(
                          "shell-usage-fill",
                          window.usedPercent >= 80
                            ? "is-danger"
                            : window.usedPercent >= 60
                              ? "is-warning"
                              : "is-ok",
                        )}
                        style={{ width: `${Math.min(100, Math.max(0, window.usedPercent))}%` }}
                      />
                    </span>
                    {window.resetsAt ? <small>reset {resetLabel(window.resetsAt)}</small> : null}
                  </div>
                ))}
              </div>
            ) : null}

            {provider.details.length > 0 ? (
              <div className="shell-usage-details">
                {provider.details.map((metric) => (
                  <span key={metric.label}>
                    {metric.label} <strong>{metric.value}</strong>
                  </span>
                ))}
              </div>
            ) : null}

            {provider.message ? <p className="shell-usage-message">{provider.message}</p> : null}
          </article>
        ))}
        {pending.map((entry) => (
          <article key={entry.id} className="shell-usage-card shell-usage-skeleton">
            <div className="shell-usage-card-head">
              <div>
                <strong>{entry.label}</strong>
                <span>consultando…</span>
              </div>
            </div>
            <span className="shell-usage-skeleton-line" />
            <span className="shell-usage-skeleton-line is-short" />
          </article>
        ))}
      </div>

      {usage.providers.length === 0 && !usage.refreshing && !usage.error ? (
        <p className="shell-muted">Todavía no hay datos de uso. Probá con "Actualizar".</p>
      ) : null}

      <p className="shell-usage-note">
        Antigravity reporta cuotas reales vía `agy -p /usage`; OpenCode suma costo local y cuotas de
        OpenCode Go. Command Code lee sus límites desde su API con la sesión local, sin abrir una
        terminal.
      </p>
    </div>
  );
}
