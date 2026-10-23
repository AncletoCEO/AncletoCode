import { describe, expect, it } from "vitest";
import { isNewer } from "./updatesStore";

describe("isNewer", () => {
  it("compares semantic versions", () => {
    expect(isNewer("0.2.0", "0.0.1")).toBe(true);
    expect(isNewer("v0.2.1", "0.2.0")).toBe(true);
    expect(isNewer("0.0.1", "0.0.1")).toBe(false);
    expect(isNewer("0.0.9", "0.1.0")).toBe(false);
  });
});
