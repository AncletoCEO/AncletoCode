import { describe, expect, it } from "vitest";
import { parseCommandCodeUsage } from "./command-code-usage";

const sample = `USAGE  GOAT Plan · active

████████████████████░░░░░░░░░░ 68% used
7,656 requests this month · 15 days to renew

Usage limits
5-hour  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 0% · resets in 4h 58m

Weekly  ██████████████░░░░░░░░░░░░░░░░ 48% · resets in 4d 19h

Full breakdown at commandcode.ai/Lubonch/settings/usage

Press Esc to close`;

describe("parseCommandCodeUsage", () => {
  it("parses the Command Code /usage text", () => {
    const usage = parseCommandCodeUsage(sample);
    expect(usage).not.toBeNull();
    expect(usage?.plan).toBe("GOAT");
    expect(usage?.monthlyPercent).toBe(68);
    expect(usage?.requests).toBe(7656);
    expect(usage?.renewDays).toBe(15);
    expect(usage?.windows).toEqual([
      { label: "5-hour", percent: 0, reset: "4h 58m" },
      { label: "Weekly", percent: 48, reset: "4d 19h" },
    ]);
  });

  it("rejects unrelated text", () => {
    expect(parseCommandCodeUsage("hola")).toBeNull();
  });
});
