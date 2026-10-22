import { describe, expect, it } from "vitest";
import { decreaseUiScale, increaseUiScale, resetUiScale } from "./ui-size";

describe("ui-size", () => {
  it("sube y baja el zoom en pasos de 0.1 dentro del rango", () => {
    expect(increaseUiScale(1)).toBe(1.1);
    expect(decreaseUiScale(1)).toBe(0.9);
    expect(increaseUiScale(1.6)).toBe(1.6);
    expect(decreaseUiScale(0.8)).toBe(0.8);
  });

  it("restablece el zoom al 100%", () => {
    expect(resetUiScale()).toBe(1);
  });
});
