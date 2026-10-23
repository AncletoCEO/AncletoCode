import * as Dialog from "@radix-ui/react-dialog";
import {
  Check,
  Download,
  Palette,
  RefreshCw,
  RotateCcw,
  SlidersHorizontal,
  Terminal,
  X,
} from "lucide-react";
import { useEffect, useSyncExternalStore } from "react";
import {
  catalog,
  getSnapshot as getAppearance,
  setOpacity,
  setTheme,
  subscribe as subscribeAppearance,
} from "../stores/appearanceStore";
import { cn } from "../ui/cn";
import { useShellPreferences } from "./preferences";
import { useUpdatesStore } from "../stores/updatesStore";
import { useAncletoRuntimeStore } from "../stores/ancletoRuntimeStore";

type SettingsDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onOpenAppearance: () => void;
};

const accents = ["#58a6ff", "#8b5cf6", "#f97316", "#22c55e", "#ec4899", "#eab308"];

export default function SettingsDialog({
  open,
  onOpenChange,
  onOpenAppearance,
}: SettingsDialogProps) {
  const appearance = useSyncExternalStore(subscribeAppearance, getAppearance);
  const preferences = useShellPreferences();
  const updates = useUpdatesStore();
  const ancleto = useAncletoRuntimeStore();
  const loadAncleto = ancleto.load;
  const themes = catalog();

  useEffect(() => {
    if (open && !ancleto.status) {
      void loadAncleto(false);
    }
  }, [open, ancleto.status, loadAncleto]);
  const theme = appearance.resolved.theme;
  const opacity = appearance.resolved.opacity;

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="shell-dialog-overlay" />
        <Dialog.Content className="shell-settings-dialog">
          <div className="shell-settings-head">
            <div>
              <Dialog.Title className="shell-settings-title">
                <SlidersHorizontal size={16} />
                Ajustes
              </Dialog.Title>
              <Dialog.Description className="shell-settings-description">
                Apariencia y densidad del workspace. Se guarda en esta máquina.
              </Dialog.Description>
            </div>
            <Dialog.Close className="icon-button" aria-label="Cerrar ajustes">
              <X size={16} />
            </Dialog.Close>
          </div>

          <div className="shell-settings-body">
            <section className="shell-settings-section">
              <h3>
                <Palette size={14} /> Tema
              </h3>
              <div className="shell-theme-grid">
                {themes.map((entry) => (
                  <button
                    key={entry.id}
                    type="button"
                    className={cn("shell-theme-card", entry.id === theme.id && "is-active")}
                    onClick={() => void setTheme(entry.id, "global")}
                  >
                    <span
                      className="shell-theme-preview"
                      style={{ background: entry.tokens.bgBase, borderColor: entry.tokens.border }}
                    >
                      <span style={{ background: entry.tokens.bgPanel }} />
                      <span style={{ background: entry.tokens.accent }} />
                    </span>
                    <span>{entry.name}</span>
                    {entry.id === theme.id ? <Check size={13} /> : null}
                  </button>
                ))}
              </div>
            </section>

            <section className="shell-settings-section">
              <h3>Acento</h3>
              <div className="shell-swatches">
                <button
                  type="button"
                  className={cn(
                    "shell-swatch shell-swatch-auto",
                    !preferences.accent && "is-active",
                  )}
                  onClick={() => preferences.setAccent(null)}
                  title="Usar acento del tema"
                >
                  auto
                </button>
                {accents.map((accent) => (
                  <button
                    key={accent}
                    type="button"
                    className={cn("shell-swatch", preferences.accent === accent && "is-active")}
                    style={{ background: accent }}
                    onClick={() => preferences.setAccent(accent)}
                    aria-label={`Acento ${accent}`}
                  />
                ))}
              </div>
            </section>

            <section className="shell-settings-section">
              <h3>Interfaz</h3>
              <div className="shell-setting-row">
                <span>Densidad</span>
                <div className="shell-segmented">
                  {(["comfortable", "compact"] as const).map((density) => (
                    <button
                      key={density}
                      type="button"
                      className={cn(preferences.density === density && "is-active")}
                      onClick={() => preferences.setDensity(density)}
                    >
                      {density === "comfortable" ? "Cómoda" : "Compacta"}
                    </button>
                  ))}
                </div>
              </div>
              <div className="shell-setting-row">
                <span>Inspector</span>
                <button
                  type="button"
                  className={cn("shell-toggle", preferences.inspectorOpen && "is-on")}
                  aria-pressed={preferences.inspectorOpen}
                  onClick={() => preferences.setInspectorOpen(!preferences.inspectorOpen)}
                >
                  <span />
                </button>
              </div>
              <label className="shell-setting-row">
                <span>Radio</span>
                <input
                  type="range"
                  min={4}
                  max={18}
                  value={preferences.radius}
                  onChange={(event) => preferences.setRadius(Number(event.target.value))}
                />
                <output>{preferences.radius}px</output>
              </label>
              <label className="shell-setting-row">
                <span>Tipografía</span>
                <input
                  type="range"
                  min={0.9}
                  max={1.15}
                  step={0.01}
                  value={preferences.fontScale}
                  onChange={(event) => preferences.setFontScale(Number(event.target.value))}
                />
                <output>{Math.round(preferences.fontScale * 100)}%</output>
              </label>
              <label className="shell-setting-row">
                <span>Opacidad de paneles</span>
                <input
                  type="range"
                  min={0.65}
                  max={1}
                  step={0.01}
                  value={opacity.panels}
                  onChange={(event) =>
                    void setOpacity({ panels: Number(event.target.value) }, "global")
                  }
                />
                <output>{Math.round(opacity.panels * 100)}%</output>
              </label>
            </section>

            <section className="shell-settings-section">
              <h3>
                <Download size={14} /> Actualizaciones
              </h3>
              <div className="shell-update-row">
                <div>
                  <strong>AncletoCode {updates.currentVersion}</strong>
                  <span>
                    {updates.latest
                      ? `Última detectada: ${updates.latest.version}`
                      : updates.checkedAt
                        ? "Sin actualizaciones detectadas."
                        : "Todavía no se chequeó."}
                  </span>
                </div>
                <button
                  type="button"
                  className="shell-ghost-button"
                  onClick={() => void updates.check(true)}
                  disabled={updates.checking}
                >
                  <RefreshCw size={13} className={updates.checking ? "animate-spin" : ""} />
                  Buscar ahora
                </button>
              </div>
              {updates.error ? <p className="shell-error">{updates.error}</p> : null}
              {updates.latest && updates.latest.version !== updates.currentVersion ? (
                <a
                  className="shell-update-link"
                  href={updates.latest.url}
                  target="_blank"
                  rel="noreferrer"
                >
                  Ver AncletoCode {updates.latest.version}
                </a>
              ) : null}
            </section>

            <section className="shell-settings-section">
              <h3>
                <Terminal size={14} /> Ancleto SDD
              </h3>
              <div className="shell-update-row">
                <div>
                  <strong>
                    {ancleto.status?.activeVersion
                      ? `Ancleto ${ancleto.status.activeVersion}`
                      : "Ancleto no detectado"}
                  </strong>
                  <span>
                    {ancleto.status?.nodePath
                      ? "Runtime y paquete incluidos en la app."
                      : "Se está usando un runtime del sistema."}
                    {ancleto.status?.bundledVersion
                      ? ` Incluido: ${ancleto.status.bundledVersion}.`
                      : ""}
                  </span>
                </div>
                <button
                  type="button"
                  className="shell-ghost-button"
                  onClick={() => void ancleto.update()}
                  disabled={ancleto.updating || !ancleto.status?.nodePath}
                >
                  <RefreshCw size={13} className={ancleto.updating ? "animate-spin" : ""} />
                  Actualizar
                </button>
              </div>
              {ancleto.error ? <p className="shell-error">{ancleto.error}</p> : null}
              {ancleto.status?.latestVersion ? (
                <p className="shell-muted">
                  Última en npm: {ancleto.status.latestVersion}
                  {ancleto.status.updateAvailable ? " · actualización disponible" : ""}
                </p>
              ) : null}
            </section>

            <section className="shell-settings-section shell-settings-advanced">
              <div>
                <h3>Apariencia avanzada</h3>
                <p>Fondos, transparencia de ventana, temas custom e importación/exportación.</p>
              </div>
              <button
                type="button"
                className="shell-ghost-button"
                onClick={() => {
                  onOpenAppearance();
                  onOpenChange(false);
                }}
              >
                Abrir apariencia
              </button>
            </section>
          </div>

          <div className="shell-settings-footer">
            <button type="button" className="shell-ghost-button" onClick={preferences.reset}>
              <RotateCcw size={14} />
              Restablecer
            </button>
            <Dialog.Close className="shell-primary-button">Listo</Dialog.Close>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
