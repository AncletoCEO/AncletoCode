export function turnText(raw: string): string {
  try {
    const value: unknown = JSON.parse(raw);
    if (value && typeof value === "object") {
      const record = value as Record<string, unknown>;
      for (const key of ["response", "result", "text"]) {
        if (typeof record[key] === "string") {
          return record[key];
        }
      }
    }
  } catch {
    // El turno ya viene como texto plano.
  }
  return raw;
}
