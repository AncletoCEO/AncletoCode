import * as Dialog from "@radix-ui/react-dialog";
import { FileDiff, X } from "lucide-react";
import { useGitStore } from "../stores/gitStore";

export default function GitDiffDialog() {
  const diff = useGitStore((state) => state.diff);
  const close = useGitStore((state) => state.closeDiff);

  if (!diff) {
    return null;
  }

  const lines = diff.text.split("\n");

  return (
    <Dialog.Root open onOpenChange={(open) => !open && close()}>
      <Dialog.Portal>
        <Dialog.Overlay className="shell-dialog-overlay" />
        <Dialog.Content className="shell-diff-dialog">
          <div className="shell-diff-head">
            <div>
              <Dialog.Title className="shell-diff-title">
                <FileDiff size={15} />
                {diff.path}
              </Dialog.Title>
              <Dialog.Description className="shell-diff-description">
                {diff.staged ? "Diff staged" : "Working tree"}
              </Dialog.Description>
            </div>
            <Dialog.Close className="icon-button" aria-label="Cerrar diff">
              <X size={16} />
            </Dialog.Close>
          </div>
          <div className="shell-diff-body">
            {diff.text ? (
              <pre>
                {lines.map((line, index) => (
                  <div
                    key={`${index}-${line.slice(0, 1)}`}
                    className={
                      line.startsWith("+") && !line.startsWith("+++")
                        ? "is-add"
                        : line.startsWith("-") && !line.startsWith("---")
                          ? "is-del"
                          : line.startsWith("@@")
                            ? "is-hunk"
                            : undefined
                    }
                  >
                    {line || " "}
                  </div>
                ))}
              </pre>
            ) : (
              <p className="shell-muted">Sin diff textual. Puede ser un archivo nuevo o binario.</p>
            )}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
