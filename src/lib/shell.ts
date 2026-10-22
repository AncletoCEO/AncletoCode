export type ShellCommand = {
  program: string;
  args: string[];
};

export function defaultShell(): ShellCommand {
  const isWindows = navigator.userAgent.includes("Windows");
  return isWindows ? { program: "powershell.exe", args: [] } : { program: "/bin/sh", args: [] };
}
