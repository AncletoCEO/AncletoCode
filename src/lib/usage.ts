import type { ProviderUsage } from "../stores/usageStore";

export const USAGE_TTL_SECS = 300;

export function isFresh(checkedAt: number, nowMs: number): boolean {
  if (checkedAt <= 0) {
    return false;
  }
  return nowMs / 1000 - checkedAt < USAGE_TTL_SECS;
}

export function mergeProvider(list: ProviderUsage[], usage: ProviderUsage): ProviderUsage[] {
  const index = list.findIndex((entry) => entry.provider === usage.provider);
  if (index === -1) {
    return [...list, usage];
  }
  const next = [...list];
  next[index] = usage;
  return next;
}

export function freshnessLabel(checkedAt: number, nowMs: number): string {
  if (checkedAt <= 0) {
    return "sin datos";
  }
  const seconds = Math.max(0, Math.round(nowMs / 1000 - checkedAt));
  if (seconds < 60) {
    return "actualizado hace instantes";
  }
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) {
    return `actualizado hace ${minutes} min`;
  }
  const hours = Math.round(minutes / 60);
  return `actualizado hace ${hours} h`;
}
