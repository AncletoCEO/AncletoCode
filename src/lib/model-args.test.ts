import { describe, expect, it } from "vitest";
import { modelArguments } from "./model-args";

describe("modelArguments", () => {
  it("uses the driver-specific flag", () => {
    expect(modelArguments("agy", "gemini-3.8-flash-high")).toEqual([
      "--model",
      "gemini-3.8-flash-high",
    ]);
    expect(modelArguments("cmd", "deepseek/deepseek-v4-flash")).toEqual([
      "-m",
      "deepseek/deepseek-v4-flash",
    ]);
  });

  it("lets the CLI choose its default", () => {
    expect(modelArguments("opencode", "")).toEqual([]);
    expect(modelArguments("shell", "anything")).toEqual([]);
  });
});
