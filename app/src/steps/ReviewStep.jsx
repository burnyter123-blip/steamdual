import { useEffect, useState } from "react";
import { Engine } from "../lib/engine.js";
import { gib } from "../lib/format.js";

function PartList({ title, parts }) {
  return (
    <div>
      <div className="bpm-section__title" style={{ fontSize: 18, marginBottom: 10 }}>{title}</div>
      <ul className="sdb-parts">
        {parts.map((p) => (
          <li
            key={p.n}
            className={
              "sdb-part" +
              (p.added ? " sdb-part--added" : "") +
              (p.role?.includes("shrunk") ? " sdb-part--shrunk" : "")
            }
          >
            <span className="sdb-part__n">p{p.n}</span>
            <span className="sdb-part__role">
              {p.role} <span className="bpm-muted">· {p.fs}</span>
            </span>
            <span className="sdb-part__size">{gib(p.gib)}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

export default function ReviewStep({ windowsGib, iso, onConfirm, onBack }) {
  const [plan, setPlan] = useState(null);
  const [typed, setTyped] = useState("");

  useEffect(() => {
    Engine.planPartitions(windowsGib).then(setPlan);
  }, [windowsGib]);

  const armed = typed.trim().toUpperCase() === "CONFIRM";

  return (
    <section>
      <div className="sdb-eyebrow">Step 4 — Review</div>
      <h1 className="sdb-title">Confirm the plan</h1>
      <p className="sdb-subtitle">
        Review exactly what will change on <span className="bpm-mono">/dev/nvme0n1</span>. A full
        backup of the partition table and ESP is taken before any write.
      </p>

      {plan && (
        <div className="bpm-card">
          <div className="sdb-diff">
            <PartList title="Now" parts={plan.before} />
            <PartList title="After" parts={plan.after} />
          </div>
        </div>
      )}

      <div className="sdb-danger">
        <div className="sdb-row" style={{ gap: 10, marginBottom: 8 }}>
          <span className="bpm-badge" style={{ background: "var(--steam-destructive)" }}>
            IRREVERSIBLE
          </span>
          <strong>Repartitioning can cause data loss if interrupted.</strong>
        </div>
        <p className="bpm-muted" style={{ margin: "0 0 14px" }}>
          Make sure the Deck stays on AC power. Installing{" "}
          <strong style={{ color: "#fff" }}>{gib(windowsGib)}</strong> of Windows 11 from{" "}
          <span className="bpm-mono">{iso.path?.split("/").pop()}</span>. Type{" "}
          <strong style={{ color: "#fff" }}>CONFIRM</strong> to proceed.
        </p>
        <div className="sdb-row">
          <input
            className="sdb-confirm-input bpm-focusable"
            value={typed}
            placeholder="CONFIRM"
            onChange={(e) => setTyped(e.target.value)}
          />
        </div>
      </div>

      <div className="sdb-actions">
        <button className="bpm-button bpm-focusable" onClick={onBack}>← Back</button>
        <button
          className="bpm-button bpm-button--destructive bpm-focusable"
          disabled={!armed}
          onClick={onConfirm}
        >
          Start installation
        </button>
      </div>
    </section>
  );
}
