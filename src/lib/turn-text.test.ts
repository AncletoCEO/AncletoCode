import { describe, expect, it } from "vitest";
import { turnText } from "./turn-text";

describe("turnText", () => {
  it("unwraps structured agent responses", () => {
    expect(
      turnText('{"status":"SUCCESS","response":"Hola\\n\\nMundo","usage":{"input_tokens":1}}'),
    ).toBe("Hola\n\nMundo");
  });

  it("keeps plain text", () => {
    expect(turnText("respuesta normal")).toBe("respuesta normal");
  });
});
