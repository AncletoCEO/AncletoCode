const PANDACODE_URL = "https://github.com/aicodeking/pandacode";

type AboutDialogProps = {
  onClose: () => void;
};

export default function AboutDialog({ onClose }: AboutDialogProps) {
  return (
    <div className="about-backdrop" role="presentation" onClick={onClose}>
      <section
        className="about"
        role="dialog"
        aria-modal="true"
        aria-labelledby="about-title"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="about-header">
          <h1 id="about-title">AncletoCode</h1>
          <button type="button" className="about-close" onClick={onClose} aria-label="Cerrar">
            ×
          </button>
        </header>
        <p>Workspace de agentes de código para escritorio y terminal.</p>
        <p>Versión 0.0.7</p>
        <p>Licencia AGPL-3.0-only.</p>
        <p>
          This is a fork of PandaCode —{" "}
          <a href={PANDACODE_URL} target="_blank" rel="noreferrer">
            {PANDACODE_URL}
          </a>
        </p>
        <p className="about-muted">
          AncletoCode reutiliza código de PandaCode bajo la PandaCode License (MIT con requisito de
          atribución). Los avisos completos están en THIRD_PARTY_NOTICES.md.
        </p>
      </section>
    </div>
  );
}
