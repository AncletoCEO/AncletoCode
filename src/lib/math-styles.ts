let styles: Promise<unknown> | null = null;

export function ensureMathStyles(): Promise<void> {
  styles ??= import("katex/dist/katex.min.css").catch((error) => {
    styles = null;
    throw error;
  });
  return styles.then(() => undefined);
}
