import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";

type OutputPayload = {
  data: number[];
};

type TerminalViewProps = {
  sessionId: string;
};

export default function TerminalView({ sessionId }: TerminalViewProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) {
      return;
    }

    const terminal = new Terminal({
      fontFamily: '"JetBrains Mono", "IBM Plex Mono", ui-monospace, monospace',
      fontSize: 13,
      cursorBlink: true,
      scrollback: 10_000,
      theme: {
        background: "#0d1117",
        foreground: "#e6edf3",
        cursor: "#58a6ff",
        selectionBackground: "#264f78",
      },
    });
    const fit = new FitAddon();
    terminal.loadAddon(fit);
    terminal.open(container);
    fit.fit();

    let disposed = false;
    let unlisten: (() => void) | undefined;

    terminal.onData((data) => {
      void invoke("session_write", {
        id: sessionId,
        data: Array.from(new TextEncoder().encode(data)),
      }).catch((error: unknown) => console.error(error));
    });

    terminal.onResize(({ cols, rows }) => {
      void invoke("session_resize", { id: sessionId, cols, rows }).catch((error: unknown) =>
        console.error(error),
      );
    });

    void invoke<number[]>("session_snapshot", { id: sessionId })
      .then((data) => terminal.write(new Uint8Array(data)))
      .catch((error: unknown) => console.error(error))
      .then(() =>
        listen<OutputPayload>(`session://output/${sessionId}`, (event) => {
          terminal.write(new Uint8Array(event.payload.data));
        }),
      )
      .then((stop) => {
        if (disposed) {
          stop();
        } else {
          unlisten = stop;
        }
      })
      .catch((error: unknown) => console.error(error));

    const observer = new ResizeObserver(() => {
      fit.fit();
    });
    observer.observe(container);

    return () => {
      disposed = true;
      unlisten?.();
      observer.disconnect();
      terminal.dispose();
    };
  }, [sessionId]);

  return <div className="terminal" ref={containerRef} />;
}
