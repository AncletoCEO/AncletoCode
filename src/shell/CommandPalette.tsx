import { Command } from "cmdk";
import {
  BookOpen,
  FolderPlus,
  GitBranch,
  ListTodo,
  Palette,
  Plus,
  Settings2,
  Sparkles,
  Terminal,
} from "lucide-react";
import {
  catalog,
  getSnapshot as getAppearance,
  setTheme,
  subscribe as subscribeAppearance,
} from "../stores/appearanceStore";
import { useSyncExternalStore } from "react";
import type { ShellView } from "./types";

type CommandPaletteProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onNewSession: (driverId: "shell" | "claude" | "opencode") => void;
  onNewProject: () => void;
  onSettings: () => void;
  onAbout: () => void;
  onView: (view: ShellView) => void;
};

export default function CommandPalette({
  open,
  onOpenChange,
  onNewSession,
  onNewProject,
  onSettings,
  onAbout,
  onView,
}: CommandPaletteProps) {
  const appearance = useSyncExternalStore(subscribeAppearance, getAppearance);
  const themes = catalog();
  const currentIndex = themes.findIndex((theme) => theme.id === appearance.resolved.theme.id);

  function run(action: () => void) {
    onOpenChange(false);
    action();
  }

  return (
    <Command.Dialog
      open={open}
      onOpenChange={onOpenChange}
      label="Comandos de AncletoCode"
      className="shell-command-dialog"
    >
      <Command.Input placeholder="Buscar proyectos, sesiones, acciones…" autoFocus />
      <Command.List>
        <Command.Empty>No hay resultados.</Command.Empty>
        <Command.Group heading="Sesiones">
          <Command.Item onSelect={() => run(() => onNewSession("shell"))}>
            <Terminal size={15} />
            Nueva sesión Shell
            <kbd>shell</kbd>
          </Command.Item>
          <Command.Item onSelect={() => run(() => onNewSession("claude"))}>
            <Sparkles size={15} />
            Nueva sesión Claude
            <kbd>claude</kbd>
          </Command.Item>
          <Command.Item onSelect={() => run(() => onNewSession("opencode"))}>
            <Sparkles size={15} />
            Nueva sesión OpenCode
            <kbd>opencode</kbd>
          </Command.Item>
        </Command.Group>
        <Command.Group heading="Workspace">
          <Command.Item onSelect={() => run(onNewProject)}>
            <FolderPlus size={15} />
            Agregar proyecto
          </Command.Item>
          <Command.Item onSelect={() => run(() => onView("tasks"))}>
            <ListTodo size={15} />
            Ir a Tareas
          </Command.Item>
          <Command.Item onSelect={() => run(() => onView("git"))}>
            <GitBranch size={15} />
            Ir a Git
          </Command.Item>
          <Command.Item onSelect={() => run(() => onView("knowledge"))}>
            <BookOpen size={15} />
            Ir a Conocimiento
          </Command.Item>
        </Command.Group>
        <Command.Group heading="Apariencia">
          <Command.Item
            onSelect={() =>
              run(() => {
                const next = themes[(currentIndex + 1) % themes.length];
                if (next) {
                  void setTheme(next.id, "global");
                }
              })
            }
          >
            <Palette size={15} />
            Cambiar tema
            <kbd>{appearance.resolved.theme.name}</kbd>
          </Command.Item>
          <Command.Item onSelect={() => run(onSettings)}>
            <Settings2 size={15} />
            Ajustes de apariencia
          </Command.Item>
        </Command.Group>
        <Command.Separator />
        <Command.Item onSelect={() => run(onAbout)}>
          <Plus size={15} />
          Acerca de AncletoCode
        </Command.Item>
      </Command.List>
    </Command.Dialog>
  );
}
