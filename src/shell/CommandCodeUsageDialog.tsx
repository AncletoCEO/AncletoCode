import * as Dialog from "@radix-ui/react-dialog";
import { ClipboardPaste, X } from "lucide-react";
import { useState } from "react";
import { parseCommandCodeUsage } from "../lib/command-code-usage";
import { useCommandCodeUsageStore } from "../stores/commandCodeUsageStore";

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

export default function CommandCodeUsageDialog({ open, onOpenChange }: Props) {
  const save = useCommandCodeUsageStore((state) => state.save);
  const [raw, setRaw] = useState("");
  const [error, setError] = useState<string | null>(null);

  function submit() {
    const parsed = parseCommandCodeUsage(raw);
    if (!parsed) {
      setError("No pude reconocer el formato de /usage. Pegá la salida completa de Command Code.");
      return;
    }
    save(raw, parsed);
    setError(null);
    onOpenChange(false);
  }

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="shell-dialog-overlay" />
        <Dialog.Content className="shell-usage-paste-dialog">
          <div className="shell-settings-head">
            <div>
              <Dialog.Title className="shell-settings-title">
                <ClipboardPaste size={16} />
                Pegar usage de Command Code
              </Dialog.Title>
              <Dialog.Description className="shell-settings-description">
                Ejecutá <code>/usage</code> en Command Code y pegá la salida. Se parsea localmente;
                no se envían credenciales.
              </Dialog.Description>
            </div>
            <Dialog.Close className="icon-button" aria-label="Cerrar">
              <X size={16} />
            </Dialog.Close>
          </div>
          <div className="shell-usage-paste-body">
            <textarea
              value={raw}
              onChange={(event) => setRaw(event.target.value)}
              placeholder={`USAGE  GOAT Plan · active\n\n████ 68% used\n...`}
              rows={14}
              spellCheck={false}
              aria-label="Salida de /usage"
            />
            {error ? <p className="shell-error">{error}</p> : null}
          </div>
          <div className="shell-settings-footer">
            <button type="button" className="shell-ghost-button" onClick={() => setRaw("")}>
              Limpiar
            </button>
            <button
              type="button"
              className="shell-primary-button"
              onClick={submit}
              disabled={!raw.trim()}
            >
              Guardar usage
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
