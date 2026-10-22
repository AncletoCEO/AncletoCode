import { describe, expect, it } from "vitest";
import { markdownHtml } from "./markdown";

describe("markdown", () => {
  it("renderiza markdown básico", async () => {
    const html = await markdownHtml("**hola** y `codigo`");
    expect(html).toContain("<strong>hola</strong>");
    expect(html).toContain("<code>codigo</code>");
  });

  it("sanea HTML y scripts", async () => {
    const html = await markdownHtml(
      '<script>alert("x")</script><img src=x onerror="alert(1)">\n\n**ok**',
    );
    expect(html).not.toContain("<script");
    expect(html).not.toContain("<img");
    expect(html).toContain("<strong>ok</strong>");
  });

  it("degrada a texto escapado ante fallas del parser", async () => {
    const html = await markdownHtml("<b>texto</b>");
    expect(html).toContain("texto");
    expect(html).not.toContain("<b>texto</b>");
  });
});
