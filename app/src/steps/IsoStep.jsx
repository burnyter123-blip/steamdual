import { useState } from "react";
import { Check, TriangleAlert, FolderOpen, ArrowLeft, ArrowRight } from "lucide-react";
import { Engine } from "../lib/engine.js";

export default function IsoStep({ iso, setIso, onNext, onBack }) {
  const [busy, setBusy] = useState(false);

  async function pick() {
    setBusy(true);
    try {
      const path = await Engine.pickIso();
      if (!path) return;
      const info = await Engine.validateIso(path);
      setIso({ path, info });
    } finally {
      setBusy(false);
    }
  }

  const valid = iso.info?.ok;

  return (
    <section>
      <div className="sdb-eyebrow">Step 3 — Windows ISO</div>
      <h1 className="sdb-title">Select your Windows 11 ISO</h1>
      <p className="sdb-subtitle">
        Point to a Windows 11 (x64) installation ISO you downloaded from Microsoft. We verify
        the edition and that it contains a valid installer before continuing.
      </p>

      <div className="bpm-card">
        <div className="sdb-spread">
          <div style={{ flex: 1, minWidth: 0 }}>
            <div className="bpm-field__label">ISO file</div>
            <div className="bpm-field__desc" style={{ wordBreak: "break-all" }}>
              {iso.path || "No file selected"}
            </div>
          </div>
          <button className="bpm-button bpm-focusable" onClick={pick} disabled={busy}>
            <FolderOpen size={16} /> {busy ? "Checking…" : "Browse…"}
          </button>
        </div>

        {iso.info && (
          <div style={{ marginTop: 16 }}>
            <div className={`sdb-check`}>
              <span className={`sdb-check__icon ${valid ? "sdb-check__icon--ok" : "sdb-check__icon--warn"}`}>
                {valid ? <Check size={16} strokeWidth={3} /> : <TriangleAlert size={15} strokeWidth={2.5} />}
              </span>
              <div style={{ flex: 1 }}>
                <div className="sdb-check__label">
                  {valid ? iso.info.edition : "Not a usable Windows 11 ISO"}
                </div>
                <div className="sdb-check__detail">
                  {valid
                    ? `${iso.info.arch} · build ${iso.info.build} · ${iso.info.detail}`
                    : iso.info.detail}
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      <div className="sdb-actions">
        <button className="bpm-button bpm-focusable" onClick={onBack}>
          <ArrowLeft size={16} /> Back
        </button>
        <button
          className="bpm-button bpm-button--primary bpm-focusable"
          disabled={!valid}
          onClick={onNext}
        >
          Review <ArrowRight size={16} />
        </button>
      </div>
    </section>
  );
}
