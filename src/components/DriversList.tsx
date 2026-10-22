import { useEffect, useSyncExternalStore } from "react";
import { getSnapshot, loadDrivers, subscribe, type ToolStatus } from "../stores/driversStore";

const glyphs: Record<ToolStatus, string> = {
  available: "●",
  missing: "○",
  incompatible: "!",
  error: "×",
};

export default function DriversList() {
  const state = useSyncExternalStore(subscribe, getSnapshot);

  useEffect(() => {
    void loadDrivers();
  }, []);

  return (
    <section className="drivers">
      <div className="sidebar-title">❯ drivers</div>
      {state.error ? <p className="error">{state.error}</p> : null}
      <ul className="driver-list">
        {state.drivers.map((driver) => (
          <li
            key={driver.id}
            className={`driver ${driver.status}`}
            title={driver.installHint ?? ""}
          >
            <span className="driver-glyph">{glyphs[driver.status]}</span>
            <span className="driver-name">{driver.displayName}</span>
            <span className="driver-version">{driver.version ?? driver.status}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}
