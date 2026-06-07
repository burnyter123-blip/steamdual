import { useEffect } from "react";
import { ArrowLeft, ArrowRight } from "lucide-react";
import { Engine } from "../lib/engine.js";
import { gib } from "../lib/format.js";

export default function SizeStep({ disk, setDisk, windowsGib, setWindowsGib, onNext, onBack }) {
  useEffect(() => {
    if (!disk) {
      Engine.probeDisk().then((d) => {
        setDisk(d);
        // Sensible default: half of shrinkable, clamped to [min, max].
        const def = Math.min(Math.max(d.minWindowsGib, Math.round(d.shrinkableGib / 2)), d.maxWindowsGib);
        setWindowsGib(def);
      });
    }
  }, [disk, setDisk, setWindowsGib]);

  if (!disk) return <p className="bpm-muted">Probing {`/dev/nvme0n1`}…</p>;

  const steamosAfter = disk.totalGib - disk.steamosUsedGib - windowsGib;
  const winPct = Math.round((windowsGib / disk.totalGib) * 100);

  return (
    <section>
      <div className="sdb-eyebrow">Step 2 — Disk size</div>
      <h1 className="sdb-title">How much space for Windows?</h1>
      <p className="sdb-subtitle">
        {disk.model} · {gib(disk.totalGib)} total. Drag to choose how much to carve out for
        Windows 11. SteamOS keeps the rest.
      </p>

      <div className="sdb-spread" style={{ alignItems: "flex-end" }}>
        <div className="sdb-stat">
          <span className="sdb-stat__k">Windows 11</span>
          <span className="sdb-bigval">{gib(windowsGib)}</span>
        </div>
        <div className="sdb-stat" style={{ textAlign: "right" }}>
          <span className="sdb-stat__k">SteamOS remaining</span>
          <span className="sdb-stat__v">{gib(steamosAfter)}</span>
        </div>
      </div>

      <div className="sdb-split">
        <div className="sdb-split__seg sdb-split__seg--steam" style={{ width: `${100 - winPct}%` }}>
          SteamOS
        </div>
        <div className="sdb-split__seg sdb-split__seg--win" style={{ width: `${winPct}%` }}>
          {winPct > 8 ? "Windows" : ""}
        </div>
      </div>

      <input
        className="sdb-range bpm-focusable"
        type="range"
        min={disk.minWindowsGib}
        max={disk.maxWindowsGib}
        step={1}
        value={windowsGib}
        onChange={(e) => setWindowsGib(Number(e.target.value))}
      />
      <div className="sdb-spread bpm-muted" style={{ fontSize: 13 }}>
        <span>min {gib(disk.minWindowsGib)} (Windows 11)</span>
        <span>max {gib(disk.maxWindowsGib)}</span>
      </div>

      <div className="sdb-actions">
        <button className="bpm-button bpm-focusable" onClick={onBack}>
          <ArrowLeft size={16} /> Back
        </button>
        <button className="bpm-button bpm-button--primary bpm-focusable" onClick={onNext}>
          Choose ISO <ArrowRight size={16} />
        </button>
      </div>
    </section>
  );
}
