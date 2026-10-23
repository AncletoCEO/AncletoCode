import { useEffect } from "react";
import { Activity, RefreshCw } from "lucide-react";
import { useUsageStore, type ProviderUsage } from "../stores/usageStore";
import { cn } from "../ui/cn";

const statusLabel: Record<ProviderUsage["status"], string> = {
  ready: "Con datos",
  unavailable: "No disponible",
  unsupported: "Sin cuota expuesta",
  signedOut: "Sin sesión",
  error: "Error",
};

function compact(value: number): string {
  return new Intl.NumberFormat("es", { notation: "compact", maximumFractionDigits: 1 }).format(
    value,
  );
}

export default function UsageView() {
  const usage = useUsageStore();
  const load = usage.load;

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <div className="shell-usage-view">
      <header className="shell-usage-head">
        <div>
          <h1>
            <Activity size={19} /> Uso de proveedores
          </h1>
          <p>
            Lectura local. No se envían prompts; las cuotas compartidas pueden incluir otras apps.
          </p>
        </div>
        <button
          type="button"
          className="shell-ghost-button"
          onClick={() => void load()}
          disabled={usage.loading}
        >
          <RefreshCw size={13} className={usage.loading ? "animate-spin" : ""} />
          Actualizar
        </button>
      </header>

      {usage.error ? <p className="shell-error">{usage.error}</p> : null}

      <div className="shell-usage-grid">
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
      </div>

      <p className="shell-usage-note">
        Orca inspira este panel: porcentajes de cuota requieren un adapter por proveedor. OpenCode
        hoy reporta tokens y costo local; para cuotas de suscripción se usará el endpoint o estado
        local de cada CLI.
      </p>
    </div>
  );
}
