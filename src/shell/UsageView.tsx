import { useEffect, useState } from "react";
import { Activity, ClipboardPaste, RefreshCw } from "lucide-react";
import { useUsageStore, type ProviderUsage } from "../stores/usageStore";
import { cn } from "../ui/cn";
import CommandCodeUsageDialog from "./CommandCodeUsageDialog";
import { useCommandCodeUsageStore } from "../stores/commandCodeUsageStore";

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
  const commandCode = useCommandCodeUsageStore();
  const [pasteOpen, setPasteOpen] = useState(false);

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
                <span>
                  {provider.provider === "commandcode" && commandCode.usage
                    ? "Pegado desde /usage"
                    : (provider.source ?? "Fuente local")}
                </span>
              </div>
              {provider.provider === "commandcode" ? (
                <button
                  type="button"
                  className="shell-git-refresh shell-usage-paste-button"
                  onClick={() => setPasteOpen(true)}
                >
                  <ClipboardPaste size={10} /> {commandCode.usage ? "Editar" : "Pegar /usage"}
                </button>
              ) : (
                <span className={cn("shell-usage-status", provider.status)}>
                  {statusLabel[provider.status]}
                </span>
              )}
            </div>

            {provider.provider === "commandcode" && commandCode.usage ? (
              <div className="shell-usage-manual">
                <div className="shell-usage-manual-head">
                  <strong>{commandCode.usage.plan ?? "Command Code"} Plan</strong>
                  <span>{commandCode.usage.plan ? "activo" : "datos pegados"}</span>
                </div>
                {commandCode.usage.monthlyPercent != null ? (
                  <div className="shell-usage-window">
                    <div>
                      <span>Monthly</span>
                      <strong>{commandCode.usage.monthlyPercent}%</strong>
                    </div>
                    <span className="shell-usage-track">
                      <span
                        className={cn(
                          "shell-usage-fill",
                          commandCode.usage.monthlyPercent >= 80
                            ? "is-danger"
                            : commandCode.usage.monthlyPercent >= 60
                              ? "is-warning"
                              : "is-ok",
                        )}
                        style={{
                          width: `${Math.min(100, Math.max(0, commandCode.usage.monthlyPercent))}%`,
                        }}
                      />
                    </span>
                  </div>
                ) : null}
                {commandCode.usage.windows.map((window) => (
                  <div key={window.label} className="shell-usage-window">
                    <div>
                      <span>{window.label}</span>
                      <strong>{window.percent}%</strong>
                    </div>
                    <span className="shell-usage-track">
                      <span
                        className={cn(
                          "shell-usage-fill",
                          window.percent >= 80
                            ? "is-danger"
                            : window.percent >= 60
                              ? "is-warning"
                              : "is-ok",
                        )}
                        style={{ width: `${Math.min(100, Math.max(0, window.percent))}%` }}
                      />
                    </span>
                    {window.reset ? <small>reset {window.reset}</small> : null}
                  </div>
                ))}
                <div className="shell-usage-details">
                  {commandCode.usage.requests != null ? (
                    <span>
                      requests <strong>{commandCode.usage.requests}</strong>
                    </span>
                  ) : null}
                  {commandCode.usage.renewDays != null ? (
                    <span>
                      renueva en <strong>{commandCode.usage.renewDays} días</strong>
                    </span>
                  ) : null}
                </div>
              </div>
            ) : null}

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
                    {window.resetsAt ? (
                      <small>reset {new Date(window.resetsAt).toLocaleString()}</small>
                    ) : null}
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
      </div>

      <p className="shell-usage-note">
        Antigravity reporta cuotas reales vía `agy -p /usage`; OpenCode reporta tokens y costo
        local. Command Code no expone cuota headless: podés pegar la salida de `/usage` para verla
        acá.
      </p>
      <CommandCodeUsageDialog open={pasteOpen} onOpenChange={setPasteOpen} />
    </div>
  );
}
