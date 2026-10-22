import { invoke } from "@tauri-apps/api/core";

const KEY = "ui.scale";
const MIN = 0.8;
const MAX = 1.6;
const STEP = 0.1;

function clamp(scale: number): number {
  return Math.round(Math.min(MAX, Math.max(MIN, scale)) * 10) / 10;
}

function apply(scale: number): number {
  if (typeof document !== "undefined") {
    document.documentElement.style.setProperty("--ui-scale", String(scale));
    document.documentElement.style.fontSize = `${16 * scale}px`;
  }
  return scale;
}

export async function loadUiScale(): Promise<number> {
  try {
    const stored = await invoke<string | null>("settings_get", { key: KEY });
    const scale = Number(stored);
    return apply(Number.isFinite(scale) && stored ? clamp(scale) : 1);
  } catch {
    return apply(1);
  }
}

export async function setUiScale(scale: number): Promise<number> {
  const next = apply(clamp(scale));
  try {
    await invoke("settings_put", { key: KEY, value: String(next) });
  } catch {
    // Sin Tauri (preview web) el zoom queda en memoria.
  }
  return next;
}

export function increaseUiScale(current: number): number {
  return clamp(current + STEP);
}

export function decreaseUiScale(current: number): number {
  return clamp(current - STEP);
}

export function resetUiScale(): number {
  return 1;
}
