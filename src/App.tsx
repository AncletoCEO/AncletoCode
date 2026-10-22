import { useState } from "react";
import AboutDialog from "./components/AboutDialog";

export default function App() {
  const [aboutOpen, setAboutOpen] = useState(false);

  return (
    <main className="app">
      <p className="prompt">❯ AncletoCode</p>
      <p className="muted">Workspace de agentes de código.</p>
      <button type="button" className="about-trigger" onClick={() => setAboutOpen(true)}>
        About
      </button>
      {aboutOpen ? <AboutDialog onClose={() => setAboutOpen(false)} /> : null}
    </main>
  );
}
