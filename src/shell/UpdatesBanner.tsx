import { Download, X } from "lucide-react";
import { isNewer, useUpdatesStore } from "../stores/updatesStore";

export default function UpdatesBanner() {
  const updates = useUpdatesStore();
  const available =
    updates.latest &&
    isNewer(updates.latest.version, updates.currentVersion) &&
    updates.dismissedVersion !== updates.latest.version;

  if (!available || !updates.latest) {
    return null;
  }

  return (
    <div className="shell-update-banner" role="status">
      <Download size={14} />
      <span>
        AncletoCode <strong>{updates.latest.version}</strong> está disponible.
      </span>
      <a href={updates.latest.url} target="_blank" rel="noreferrer">
        Ver release
      </a>
      <button
        type="button"
        className="icon-button icon-button-small"
        onClick={updates.dismiss}
        aria-label="Descartar aviso"
      >
        <X size={13} />
      </button>
    </div>
  );
}
