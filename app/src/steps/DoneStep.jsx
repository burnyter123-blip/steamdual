import { Engine } from "../lib/engine.js";

export default function DoneStep() {
  return (
    <section>
      <div className="sdb-eyebrow" style={{ color: "#70d61d" }}>Success</div>
      <h1 className="sdb-title">Windows 11 is installed 🎉</h1>
      <p className="sdb-subtitle">
        Reboot to finish. You'll see the new <strong style={{ color: "#fff" }}>SteamDualBoot</strong>{" "}
        picker at power-on — flick the left stick or press the d-pad to choose SteamOS or Windows,
        and press <strong style={{ color: "#fff" }}>A</strong> to boot. Press{" "}
        <strong style={{ color: "#fff" }}>Y</strong> to set your default.
      </p>

      <div className="bpm-card">
        <div className="sdb-check">
          <span className="sdb-check__icon sdb-check__icon--ok">✓</span>
          <div style={{ flex: 1 }}>
            <div className="sdb-check__label">Boot picker installed</div>
            <div className="sdb-check__detail">Survives SteamOS updates via the heal service</div>
          </div>
        </div>
        <div className="sdb-check">
          <span className="sdb-check__icon sdb-check__icon--ok">✓</span>
          <div style={{ flex: 1 }}>
            <div className="sdb-check__label">Reopen this app any time</div>
            <div className="sdb-check__detail">
              Resize Windows, change the default OS, or remove Windows from the config pane
            </div>
          </div>
        </div>
      </div>

      <div className="sdb-actions">
        <button
          className="bpm-button bpm-button--primary bpm-focusable"
          onClick={() => Engine.reboot()}
        >
          Reboot now
        </button>
        <button className="bpm-button bpm-focusable" onClick={() => window.location.reload()}>
          Later
        </button>
      </div>
    </section>
  );
}
