export function modelArguments(driverId: string, modelId: string): string[] {
  if (!modelId || driverId === "shell") {
    return [];
  }
  return driverId === "cmd" ? ["-m", modelId] : ["--model", modelId];
}
