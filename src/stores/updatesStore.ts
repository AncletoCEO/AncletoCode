import { getVersion } from "@tauri-apps/api/app";
import { create } from "zustand";

export type ReleaseInfo = {
  version: string;
  name: string;
  url: string;
  publishedAt: string;
  body: string;
};

type UpdatesState = {
  currentVersion: string;
  latest: ReleaseInfo | null;
  checking: boolean;
  checkedAt: number | null;
  error: string | null;
  dismissedVersion: string | null;
  check: (force?: boolean) => Promise<void>;
  dismiss: () => void;
  reset: () => void;
};

const LAST_CHECK_KEY = "ancleto.updates.lastCheck";
const DISMISSED_KEY = "ancleto.updates.dismissed";
const CHECK_INTERVAL_MS = 12 * 60 * 60 * 1000;

function parseVersion(value: string): number[] {
  return value
    .replace(/^v/i, "")
    .split(/[.+-]/)
    .slice(0, 3)
    .map((part) => Number.parseInt(part, 10) || 0);
}

export function isNewer(candidate: string, current: string): boolean {
  const left = parseVersion(candidate);
  const right = parseVersion(current);
  for (let index = 0; index < 3; index += 1) {
    const a = left[index] ?? 0;
    const b = right[index] ?? 0;
    if (a !== b) {
      return a > b;
    }
  }
  return false;
}

function storedNumber(key: string): number | null {
  const value = window.localStorage.getItem(key);
  return value ? Number(value) || null : null;
}

export const useUpdatesStore = create<UpdatesState>()((set, get) => ({
  currentVersion: "0.0.1",
  latest: null,
  checking: false,
  checkedAt: storedNumber(LAST_CHECK_KEY),
  error: null,
  dismissedVersion: window.localStorage.getItem(DISMISSED_KEY),
  check: async (force = false) => {
    const { checkedAt, checking } = get();
    if (checking) {
      return;
    }
    if (!force && checkedAt && Date.now() - checkedAt < CHECK_INTERVAL_MS) {
      return;
    }
    set({ checking: true, error: null });
    try {
      const currentVersion = await getVersion().catch(() => get().currentVersion);
      const response = await fetch(
        "https://api.github.com/repos/AncletoCEO/AncletoCode/releases?per_page=10",
        { headers: { Accept: "application/vnd.github+json" } },
      );
      if (!response.ok) {
        throw new Error(`GitHub respondió ${response.status}`);
      }
      const releases = (await response.json()) as Array<{
        tag_name: string;
        name: string;
        html_url: string;
        published_at: string;
        body: string;
        draft: boolean;
      }>;
      const latest =
        releases
          .filter((release) => !release.draft)
          .map<ReleaseInfo>((release) => ({
            version: release.tag_name.replace(/^v/i, ""),
            name: release.name,
            url: release.html_url,
            publishedAt: release.published_at,
            body: release.body,
          }))
          .sort((left, right) => (isNewer(left.version, right.version) ? -1 : 1))[0] ?? null;
      const checked = Date.now();
      window.localStorage.setItem(LAST_CHECK_KEY, String(checked));
      set({ currentVersion, latest, checking: false, checkedAt: checked });
    } catch (error) {
      set({
        checking: false,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  },
  dismiss: () => {
    const version = get().latest?.version;
    if (!version) {
      return;
    }
    window.localStorage.setItem(DISMISSED_KEY, version);
    set({ dismissedVersion: version });
  },
  reset: () => set({ latest: null, error: null, checkedAt: null, dismissedVersion: null }),
}));
