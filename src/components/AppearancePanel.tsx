import { useState, useSyncExternalStore } from "react";
import { invoke } from "@tauri-apps/api/core";
import { worstContrast } from "../appearance/contrast";
import type { ThemeTokens } from "../appearance/types";
import {
  catalog,
  duplicateTheme,
  exportTheme,
  getSnapshot,
  importTheme,
  removeCustomTheme,
  setBackground,
  setOpacity,
  setTheme,
  subscribe,
  updateThemeToken,
} from "../stores/appearanceStore";

type AppearancePanelProps = {
  projectId: string;
};

export default function AppearancePanel({ projectId }: AppearancePanelProps) {
  const state = useSyncExternalStore(subscribe, getSnapshot);
  const [scope, setScope] = useState<"global" | "project">("global");
  const [error, setError] = useState<string | null>(null);

  const theme = state.resolved.theme;
  const background = state.resolved.background;
  const opacity = state.resolved.opacity;
  const themes = catalog();
  const contrast = worstContrast(theme);

  async function action(run: () => Promise<unknown>) {
    setError(null);
    try {
      await run();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    }
  }

  function download() {
    const text = exportTheme(theme.id);
    if (!text) {
      return;
    }
    const url = URL.createObjectURL(new Blob([text], { type: "application/json" }));
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `${theme.id}.json`;
    anchor.click();
    URL.revokeObjectURL(url);
  }

  async function loadFile(file: File | undefined) {
    if (!file) {
      return;
    }
    const text = await file.text();
    await action(() => importTheme(text));
  }

  async function loadImage(file: File | undefined) {
    if (!file) {
      return;
    }
    const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
    const extension = file.name.split(".").pop() ?? "png";
    await action(async () => {
      const path = await invoke<string>("appearance_save_image", { extension, bytes });
      await setBackground({ kind: "image", imagePath: path }, scope);
    });
  }

  return (
    <section className="spec-panel appearance-panel">
      <div className="spec-status">
        <span className="spec-label">apariencia</span>
        <label>
          scope
          <select
            value={scope}
            onChange={(event) => setScope(event.target.value as "global" | "project")}
          >
            <option value="global">global</option>
            <option value="project">proyecto</option>
          </select>
        </label>
        <span className="muted">{projectId.slice(0, 8)}</span>
      </div>

      <div className="spec-settings">
        <label>
          tema
          <select value={theme.id} onChange={(event) => void setTheme(event.target.value, scope)}>
            {themes.map((entry) => (
              <option key={entry.id} value={entry.id}>
                {entry.name}
              </option>
            ))}
          </select>
        </label>
        <span className="theme-preview">
          {[
            theme.tokens.bgBase,
            theme.tokens.bgPanel,
            theme.tokens.fg,
            theme.tokens.accent,
            theme.tokens.success,
            theme.tokens.warning,
            theme.tokens.danger,
          ].map((color) => (
            <span key={color} className="swatch" style={{ background: color }} title={color} />
          ))}
        </span>
        <button type="button" onClick={() => void action(() => duplicateTheme(theme.id))}>
          duplicar
        </button>
        <button type="button" onClick={download}>
          exportar
        </button>
        <label className="file-button">
          importar
          <input
            type="file"
            accept="application/json,.json"
            onChange={(event) => void loadFile(event.target.files?.[0])}
          />
        </label>
        {state.data.customThemes.some((entry) => entry.id === theme.id) ? (
          <button type="button" onClick={() => void action(() => removeCustomTheme(theme.id))}>
            eliminar
          </button>
        ) : null}
        {contrast.value < 4.5 ? (
          <span className="error">
            contraste {contrast.value.toFixed(1)} ({contrast.pair})
            <button
              type="button"
              onClick={() => void setBackground({ dim: Math.max(background.dim, 0.35) }, scope)}
            >
              aplicar scrim
            </button>
          </span>
        ) : (
          <span className="muted">contraste {contrast.value.toFixed(1)}</span>
        )}
      </div>

      <details className="appearance-tokens">
        <summary>tokens</summary>
        <div className="token-grid">
          {Object.entries(theme.tokens).map(([name, value]) => (
            <label key={name}>
              {name}
              <input
                type="color"
                value={value}
                onChange={(event) =>
                  void updateThemeToken(theme.id, name as keyof ThemeTokens, event.target.value)
                }
              />
            </label>
          ))}
        </div>
      </details>

      <div className="spec-settings">
        <label>
          fondo
          <select
            value={background.kind}
            onChange={(event) =>
              void setBackground({ kind: event.target.value as typeof background.kind }, scope)
            }
          >
            <option value="solid">sólido</option>
            <option value="gradient">degradado</option>
            <option value="image">imagen</option>
            <option value="pattern">patrón</option>
          </select>
        </label>
        {background.kind === "solid" ? (
          <label>
            color
            <input
              type="color"
              value={background.color || theme.tokens.bgBase}
              onChange={(event) => void setBackground({ color: event.target.value }, scope)}
            />
          </label>
        ) : null}
        {background.kind === "gradient" ? (
          <>
            <label>
              desde
              <input
                type="color"
                value={background.gradientFrom}
                onChange={(event) =>
                  void setBackground({ gradientFrom: event.target.value }, scope)
                }
              />
            </label>
            <label>
              hasta
              <input
                type="color"
                value={background.gradientTo}
                onChange={(event) => void setBackground({ gradientTo: event.target.value }, scope)}
              />
            </label>
            <label>
              ángulo
              <input
                type="number"
                min={0}
                max={360}
                value={background.angle}
                onChange={(event) =>
                  void setBackground({ angle: Number(event.target.value) }, scope)
                }
              />
            </label>
          </>
        ) : null}
        {background.kind === "image" ? (
          <>
            <label className="file-button">
              imagen
              <input
                type="file"
                accept="image/png,image/jpeg,image/webp,image/gif"
                onChange={(event) => void loadImage(event.target.files?.[0])}
              />
            </label>
            <label>
              ajuste
              <select
                value={background.fit}
                onChange={(event) =>
                  void setBackground({ fit: event.target.value as typeof background.fit }, scope)
                }
              >
                <option value="cover">cover</option>
                <option value="contain">contain</option>
                <option value="tile">tile</option>
              </select>
            </label>
            <label>
              blur
              <input
                type="range"
                min={0}
                max={24}
                value={background.blur}
                onChange={(event) =>
                  void setBackground({ blur: Number(event.target.value) }, scope)
                }
              />
            </label>
            <label>
              dim
              <input
                type="range"
                min={0}
                max={0.8}
                step={0.05}
                value={background.dim}
                onChange={(event) => void setBackground({ dim: Number(event.target.value) }, scope)}
              />
            </label>
          </>
        ) : null}
        {background.kind === "pattern" ? (
          <label>
            patrón
            <select
              value={background.pattern}
              onChange={(event) =>
                void setBackground(
                  { pattern: event.target.value as typeof background.pattern },
                  scope,
                )
              }
            >
              <option value="none">ninguno</option>
              <option value="grid">grid</option>
              <option value="scanlines">scanlines</option>
            </select>
          </label>
        ) : null}
      </div>

      <div className="spec-settings">
        <label>
          ventana
          <input
            type="range"
            min={0.5}
            max={1}
            step={0.01}
            value={opacity.window}
            onChange={(event) => void setOpacity({ window: Number(event.target.value) }, scope)}
          />
        </label>
        <label>
          paneles
          <input
            type="range"
            min={0.5}
            max={1}
            step={0.01}
            value={opacity.panels}
            onChange={(event) => void setOpacity({ panels: Number(event.target.value) }, scope)}
          />
        </label>
        <label>
          terminal
          <input
            type="range"
            min={0.5}
            max={1}
            step={0.01}
            value={opacity.terminal}
            onChange={(event) => void setOpacity({ terminal: Number(event.target.value) }, scope)}
          />
        </label>
        <label>
          <input
            type="checkbox"
            checked={opacity.opaque}
            onChange={(event) => void setOpacity({ opaque: event.target.checked }, scope)}
          />
          modo opaco
        </label>
        <label>
          efecto
          <select
            value={opacity.effect}
            onChange={(event) =>
              void setOpacity({ effect: event.target.value as typeof opacity.effect }, scope)
            }
          >
            <option value="auto">auto</option>
            <option value="none">ninguno</option>
            <option value="mica">mica</option>
            <option value="acrylic">acrylic</option>
            <option value="blur">blur</option>
            <option value="vibrancy">vibrancy</option>
          </select>
        </label>
      </div>

      {error ? <p className="error">{error}</p> : null}
    </section>
  );
}
