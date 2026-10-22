import { useEffect, useRef } from "react";
import { streamMarkdown } from "../../third_party/pandacode/src/streaming-markdown.ts";

type MarkdownProps = {
  value: string;
};

export default function Markdown({ value }: MarkdownProps) {
  const targetRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const target = targetRef.current;
    if (!target) {
      return;
    }
    streamMarkdown(target, value);
  }, [value]);

  return <div ref={targetRef} className="markdown" />;
}
