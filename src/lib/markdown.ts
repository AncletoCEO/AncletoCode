import DOMPurify from "dompurify";
import { ensureMathStyles } from "./math-styles";
import {
  parseMarkdown,
  renderMarkdown,
  typesetMath,
} from "../../third_party/pandacode/src/enriched-markdown.ts";

const FORBID_TAGS = [
  "style",
  "img",
  "iframe",
  "form",
  "input",
  "button",
  "textarea",
  "select",
  "option",
  "video",
  "audio",
  "script",
];

export async function markdownHtml(value: string): Promise<string> {
  let html: string;
  try {
    html = renderMarkdown(await parseMarkdown(value));
  } catch {
    return `<pre class="md-fallback">${escapeHtml(value)}</pre>`;
  }
  const safe = DOMPurify.sanitize(html, {
    ALLOW_DATA_ATTR: false,
    FORBID_TAGS,
    FORBID_ATTR: ["style", "id", "name"],
  });
  try {
    await ensureMathStyles();
    return await typesetMath(safe);
  } catch {
    return safe;
  }
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
