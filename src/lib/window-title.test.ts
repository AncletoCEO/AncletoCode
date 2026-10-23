import { describe, expect, it } from "vitest";
import { APP_TITLE, THURSDAY_TITLE, titleForDate } from "./window-title";

describe("titleForDate", () => {
  it("uses the Thursday title on Thursday", () => {
    expect(titleForDate(new Date(2026, 9, 22))).toBe(THURSDAY_TITLE);
    expect(THURSDAY_TITLE).toBe("AncletoCode - Feliz Jueves 🏃");
  });

  it("uses the normal title on other days", () => {
    expect(titleForDate(new Date(2026, 9, 23))).toBe(APP_TITLE);
    expect(titleForDate(new Date(2026, 9, 19))).toBe(APP_TITLE);
  });
});
