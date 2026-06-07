import { useEffect, useState } from "react";
import { Check, TriangleAlert, ArrowRight } from "lucide-react";
import { Engine } from "../lib/engine.js";

export default function Welcome({ onNext }) {
  const [pf, setPf] = useState(null);

  useEffect(() => {
    Engine.preflight().then(setPf);
  }, []);

  return (
    <section>
      <div className="sdb-eyebrow">Set up dual-boot</div>
      <h1 className="sdb-title">Install Windows 11 alongside SteamOS</h1>
      <p className="sdb-subtitle">
        This wizard shrinks your SteamOS storage, creates Windows partitions, installs
        Windows 11 unattended, and sets up a controller-friendly boot picker. Your games
        and SteamOS stay intact. You can change the split or remove Windows later.
      </p>

      <div className="bpm-card" style={{ padding: 8 }}>
        {!pf && <p className="bpm-muted" style={{ padding: 16 }}>Running pre-flight checks…</p>}
        {pf &&
          pf.checks.map((c) => (
            <div className="sdb-check" key={c.id}>
              <span className={`sdb-check__icon ${c.ok ? "sdb-check__icon--ok" : "sdb-check__icon--warn"}`}>
                {c.ok ? <Check size={16} strokeWidth={3} /> : <TriangleAlert size={15} strokeWidth={2.5} />}
              </span>
              <div style={{ flex: 1 }}>
                <div className="sdb-check__label">{c.label}</div>
                <div className="sdb-check__detail">{c.detail}</div>
              </div>
            </div>
          ))}
      </div>

      <div className="sdb-actions">
        <button
          className="bpm-button bpm-button--primary bpm-focusable"
          disabled={!pf || !pf.canProceed}
          onClick={onNext}
        >
          Begin <ArrowRight size={16} />
        </button>
      </div>
    </section>
  );
}
