import { describe, expect, it } from "vitest";
import { modelFromArguments, sessionLabel } from "./session-model";

describe("session model label", () => {
  it("reads the model from session args", () => {
    expect(modelFromArguments(["--model", "gemini-3.8-flash-high"])).toBe("gemini-3.8-flash-high");
    expect(modelFromArguments(["-m", "deepseek/v4"])).toBe("deepseek/v4");
    expect(modelFromArguments([])).toBeNull();
  });

  it("adds the model to the session label", () => {
    expect(sessionLabel("Investigador", ["--model", "gemini"])).toBe("Investigador (gemini)");
    expect(sessionLabel("Investigador", ["--model", "gemini"], "agy")).toBe(
      "Investigador (agy · gemini)",
    );
    expect(sessionLabel("Shell", [])).toBe("Shell");
  });
});
