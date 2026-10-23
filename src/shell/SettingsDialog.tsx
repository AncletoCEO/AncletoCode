import * as Dialog from "@radix-ui/react-dialog";
import { Check, Palette, RotateCcw, SlidersHorizontal, X } from "lucide-react";
import { useSyncExternalStore } from "react";
import {
  catalog,
  getSnapshot as getAppearance,
  setOpacity,
  setTheme,
  subscribe as subscribeAppearance,
} from "../stores/appearanceStore";
import { cn } from "../ui/cn";
import { useShellPreferences } from "./preferences";

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
  const themes = catalog();
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
