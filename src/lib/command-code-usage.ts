export type CommandCodeWindow = {
  label: string;
  percent: number;
  reset: string | null;
};

export type CommandCodeUsage = {
  plan: string | null;
  monthlyPercent: number | null;
  requests: number | null;
  renewDays: number | null;
  windows: CommandCodeWindow[];
};

const numberFrom = (value: string | undefined): number | null => {
  if (!value) return null;
  const parsed = Number.parseInt(value.replace(/[^\d]/g, ""), 10);
  return Number.isFinite(parsed) ? parsed : null;
};

export function parseCommandCodeUsage(raw: string): CommandCodeUsage | null {
  const text = raw.replace(/\r/g, "");
  if (!/usage/i.test(text) || !/%\s*used|%\s*·/i.test(text)) {
    return null;
  }

  const plan = text.match(/USAGE\s+(.+?)\s+Plan\b/i)?.[1]?.trim() ?? null;
  const monthlyPercent = numberFrom(text.match(/(\d{1,3})%\s*used/i)?.[1]);
  const requests = numberFrom(text.match(/([\d,]+)\s+requests\s+this month/i)?.[1]);
  const renewDays = numberFrom(text.match(/(\d+)\s+days?\s+to renew/i)?.[1]);

  const windows: CommandCodeWindow[] = [];
  for (const rawLine of text.split("\n")) {
    const line = rawLine.trim();
    const match = line.match(/(\d{1,3})%\s*·\s*resets?\s+in\s+(.+)$/i);
    if (!match) continue;
    const label = line
      .replace(/[█░▓▒]+/g, "")
      .replace(/\s+\d{1,3}%.*$/, "")
      .trim();
    if (!label) continue;
    windows.push({
      label,
      percent: Number.parseInt(match[1] ?? "0", 10),
      reset: match[2]?.trim() ?? null,
    });
  }

  if (windows.length === 0 && monthlyPercent == null) {
    return null;
  }

  return { plan, monthlyPercent, requests, renewDays, windows };
}
