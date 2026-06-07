import { useEffect, useState } from "react";
import { ArrowRightLeft, Check } from "lucide-react";
import { Engine, isMock } from "./lib/engine.js";
import { useGamepadFocus } from "./hooks/useGamepadFocus.js";
import Footer from "./components/Footer.jsx";

import Welcome from "./steps/Welcome.jsx";
import SizeStep from "./steps/SizeStep.jsx";
import IsoStep from "./steps/IsoStep.jsx";
import ReviewStep from "./steps/ReviewStep.jsx";
import ExecuteStep from "./steps/ExecuteStep.jsx";
import DoneStep from "./steps/DoneStep.jsx";
import ConfigPane from "./steps/ConfigPane.jsx";

const STEPS = ["Welcome", "Disk size", "Windows ISO", "Review", "Install", "Done"];

function Stepper({ index }) {
  return (
    <div className="sdb-stepper">
      {STEPS.map((label, i) => (
        <div key={label} style={{ display: "flex", alignItems: "center" }}>
          {i > 0 && <span className="sdb-step-sep" />}
          <div
            className={
              "sdb-step-dot" +
              (i === index ? " sdb-step-dot--active" : "") +
              (i < index ? " sdb-step-dot--done" : "")
            }
          >
            <span className="sdb-step-dot__num">
              {i < index ? <Check size={14} strokeWidth={3} /> : i + 1}
            </span>
            {i === index && <span>{label}</span>}
          </div>
        </div>
      ))}
    </div>
  );
}

export default function App() {
  useGamepadFocus();
  const [mode, setMode] = useState("loading"); // loading | setup | config
  const [step, setStep] = useState(0);

  // Shared wizard data.
  const [disk, setDisk] = useState(null);
  const [windowsGib, setWindowsGib] = useState(128);
  const [iso, setIso] = useState({ path: null, info: null });

  useEffect(() => {
    Engine.getState().then((s) => setMode(s.installed ? "config" : "setup"));
  }, []);

  const next = () => setStep((s) => Math.min(s + 1, STEPS.length - 1));
  const back = () => setStep((s) => Math.max(s - 1, 0));

  function renderStep() {
    switch (step) {
      case 0:
        return <Welcome onNext={next} />;
      case 1:
        return (
          <SizeStep
            disk={disk}
            setDisk={setDisk}
            windowsGib={windowsGib}
            setWindowsGib={setWindowsGib}
            onNext={next}
            onBack={back}
          />
        );
      case 2:
        return <IsoStep iso={iso} setIso={setIso} onNext={next} onBack={back} />;
      case 3:
        return (
          <ReviewStep
            windowsGib={windowsGib}
            iso={iso}
            onConfirm={next}
            onBack={back}
          />
        );
      case 4:
        return <ExecuteStep windowsGib={windowsGib} iso={iso} onDone={next} />;
      case 5:
        return <DoneStep />;
      default:
        return null;
    }
  }

  return (
    <div className="sdb-app">
      <nav className="sdb-nav">
        <div className="sdb-nav__brand">
          <span className="sdb-nav__logo"><ArrowRightLeft size={18} strokeWidth={2.5} /></span>
          STEAM&nbsp;DUALBOOT
        </div>
        <span className="sdb-nav__spacer" />
        {mode === "setup" && <Stepper index={step} />}
        {isMock && (
          <span className="bpm-badge" style={{ marginLeft: 16 }} title="Running outside Tauri">
            DEMO
          </span>
        )}
      </nav>

      <main className="sdb-main">
        {mode === "loading" && <p className="bpm-muted">Loading…</p>}
        {mode === "config" && <ConfigPane />}
        {mode === "setup" && renderStep()}
      </main>

      {mode === "setup" && step < 4 && (
        <Footer
          hints={[
            { k: "A", label: "Select" },
            ...(step > 0 ? [{ k: "B", label: "Back" }] : []),
          ]}
        />
      )}
    </div>
  );
}
