export function modelFromArguments(args: string[]): string | null {
  for (let index = 0; index < args.length; index += 1) {
    if ((args[index] === "--model" || args[index] === "-m") && args[index + 1]) {
      return args[index + 1] ?? null;
    }
  }
  return null;
}

export function sessionLabel(title: string, args: string[], driverId?: string): string {
  const model = modelFromArguments(args);
  const detail = [driverId, model].filter(Boolean).join(" · ");
  return detail ? `${title} (${detail})` : title;
}
