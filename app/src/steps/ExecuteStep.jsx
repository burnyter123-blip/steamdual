import { useEffect, useRef, useState } from "react";
import { Engine, on } from "../lib/engine.js";

const STEP_LABELS = [
  ["backup", "Back up GPT + ESP"],
  ["shrink", "Shrink SteamOS home"],
  ["partition", "Create Windows partitions"],
  ["stage", "Generate autounattend.xml"],
  ["install", "Install Windows 11 (VM)"],
  ["boot", "Register boot picker + heal hook"],
];

export default function ExecuteStep({ windowsGib, iso, onDone }) {
  const [progress, setProgress] = useState({ pct: 0, step: null, status: "running" });
  const [log, setLog] = useState([]);
  const logRef = useRef(null);
  const started = useRef(false);

  useEffect(() => {
    let unlisten;
    (async () => {
      unlisten = await on("install://progress", (e) => {
        const p = e.payload;
        setProgress(p);
        setLog((l) => [...l, p.log].slice(-200));
        if (p.status === "done") setTimeout(onDone, 700);
      });
      if (!started.current) {
        started.current = true;
        Engine.startInstall({ windowsGib, isoPath: iso.path });
      }
    })();
    return () => unlisten && unlisten();
  }, [windowsGib, iso, onDone]);

  useEffect(() => {
    if (logRef.current) logRef.current.scrollTop = logRef.current.scrollHeight;
  }, [log]);

  const curIndex = progress.stepIndex ?? 0;

  return (
    <section>
      <div className="sdb-eyebrow">Installing</div>
      <h1 className="sdb-title">Setting up Windows 11</h1>
      <p className="sdb-subtitle">
        Keep the Deck plugged in. This takes a while — the Windows installer runs in a VM
        against the new partition. You can watch progress below.
      </p>

      <div className="sdb-spread" style={{ marginBottom: 6 }}>
        <span className="bpm-muted">{progress.label || "Starting…"}</span>
        <span className="bpm-mono">{progress.pct ?? 0}%</span>
      </div>
      <div className="bpm-progress" style={{ height: 8 }}>
        <div className="bpm-progress__fill" style={{ width: `${progress.pct ?? 0}%` }} />
      </div>

      <ul className="sdb-exec-steps">
        {STEP_LABELS.map(([id, label], i) => {
          const cls =
            i < curIndex || progress.status === "done"
              ? "sdb-exec-step--done"
              : i === curIndex
              ? "sdb-exec-step--running"
              : "";
          const mark = i < curIndex || progress.status === "done" ? "✓" : i === curIndex ? "▶" : i + 1;
          return (
            <li className={`sdb-exec-step ${cls}`} key={id}>
              <span className="sdb-exec-step__mark">{mark}</span>
              {label}
            </li>
          );
        })}
      </ul>

      <div className="sdb-log" ref={logRef}>
        {log.join("\n")}
      </div>
    </section>
  );
}
