import { useEffect, useState } from "react";
import { Engine } from "../lib/engine.js";
import { gib } from "../lib/format.js";

// Shown when Windows is already installed: the app becomes a settings pane.
export default function ConfigPane() {
  const [cfg, setCfg] = useState(null);

  useEffect(() => {
    Engine.getConfig().then(setCfg);
  }, []);

  if (!cfg) return <p className="bpm-muted">Loading configuration…</p>;

  const setDefault = (os) => {
    setCfg({ ...cfg, defaultOs: os });
    Engine.setDefaultOs(os);
  };
  const setTimeout_ = (seconds) => {
    setCfg({ ...cfg, timeoutSeconds: seconds });
    Engine.setTimeout(seconds);
  };

  return (
    <section>
      <div className="sdb-eyebrow">Dual-boot installed</div>
      <h1 className="sdb-title">Boot configuration</h1>
      <p className="sdb-subtitle">
        Windows 11 is set up ({gib(cfg.windowsGib)}). Adjust how the boot picker behaves, or
        manage the Windows partition.
      </p>

      <div className="bpm-card" style={{ padding: 8 }}>
        <div className="bpm-field bpm-focusable" tabIndex={0}>
          <div>
            <div className="bpm-field__label">Default operating system</div>
            <div className="bpm-field__desc">Booted automatically when the timer runs out</div>
          </div>
          <div className="bpm-row">
            {["steamos", "windows"].map((os) => (
              <button
                key={os}
                className={
                  "bpm-button bpm-focusable" + (cfg.defaultOs === os ? " bpm-button--primary" : "")
                }
                onClick={() => setDefault(os)}
              >
                {os === "steamos" ? "SteamOS" : "Windows 11"}
              </button>
            ))}
          </div>
        </div>

        <div className="bpm-field bpm-focusable" tabIndex={0}>
          <div>
            <div className="bpm-field__label">Boot menu timeout</div>
            <div className="bpm-field__desc">{cfg.timeoutSeconds}s before the default boots</div>
          </div>
          <input
            className="sdb-range bpm-focusable"
            style={{ width: 220 }}
            type="range"
            min={0}
            max={30}
            value={cfg.timeoutSeconds}
            onChange={(e) => setTimeout_(Number(e.target.value))}
          />
        </div>
      </div>

      <h2 className="bpm-section__title" style={{ fontSize: 20, margin: "28px 0 12px" }}>
        Maintenance
      </h2>
      <div className="bpm-card" style={{ padding: 8 }}>
        <div className="bpm-field bpm-focusable" tabIndex={0}>
          <div>
            <div className="bpm-field__label">Repair boot picker</div>
            <div className="bpm-field__desc">Re-assert the EFI entry if a SteamOS update changed it</div>
          </div>
          <button className="bpm-button bpm-focusable" onClick={() => Engine.repairBootloader()}>
            Repair
          </button>
        </div>
        <div className="bpm-field bpm-focusable" tabIndex={0}>
          <div>
            <div className="bpm-field__label">Remove Windows</div>
            <div className="bpm-field__desc">Delete Windows partitions and give the space back to SteamOS</div>
          </div>
          <button className="bpm-button bpm-button--destructive bpm-focusable" onClick={() => Engine.uninstall()}>
            Uninstall
          </button>
        </div>
      </div>
    </section>
  );
}
