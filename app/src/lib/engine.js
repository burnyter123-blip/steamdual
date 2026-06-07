// Bridge to the Rust "system engine" (Tauri commands + events).
//
// When running inside Tauri we call the real backend via `invoke`/`listen`.
// When running in a plain browser (e.g. `vite dev` for UI work, or a CI
// screenshot) we fall back to a realistic in-memory MOCK so the whole wizard is
// navigable and demoable without touching a disk. The command surface here is
// the contract the Rust side in `src-tauri/src/lib.rs` implements 1:1.

const inTauri = typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;

let _invoke, _listen;
async function bridge() {
  if (_invoke) return { invoke: _invoke, listen: _listen };
  if (inTauri) {
    const core = await import("@tauri-apps/api/core");
    const event = await import("@tauri-apps/api/event");
    _invoke = core.invoke;
    _listen = event.listen;
  } else {
    ({ invoke: _invoke, listen: _listen } = mock());
  }
  return { invoke: _invoke, listen: _listen };
}

export const isMock = !inTauri;

export async function call(cmd, args) {
  const { invoke } = await bridge();
  return invoke(cmd, args);
}

export async function on(eventName, handler) {
  const { listen } = await bridge();
  return listen(eventName, handler);
}

// --- High-level helpers used by the UI ------------------------------------
export const Engine = {
  getState: () => call("get_state"),
  preflight: () => call("preflight"),
  probeDisk: () => call("probe_disk"),
  planPartitions: (windowsGib) => call("plan_partitions", { windowsGib }),
  validateIso: (path) => call("validate_iso", { path }),
  pickIso: () => call("pick_iso"),
  startInstall: (opts) => call("start_install", opts),
  // config pane
  getConfig: () => call("get_config"),
  setDefaultOs: (os) => call("set_default_os", { os }),
  setTimeout: (seconds) => call("set_timeout", { seconds }),
  repairBootloader: () => call("repair_bootloader"),
  uninstall: () => call("uninstall"),
  reboot: () => call("reboot"),
};

// ===========================================================================
// MOCK backend — only used outside Tauri.
// ===========================================================================
function mock() {
  const GIB = 1;
  const disk = {
    device: "/dev/nvme0n1",
    model: "Valve Deck NVMe 512GB",
    totalGib: 476,
    steamosUsedGib: 92,
    freeGib: 300,
    minWindowsGib: 64,
    maxWindowsGib: 280,
    shrinkableGib: 280,
  };
  // Pretend Windows is not yet installed in mock-setup mode.
  const state = { mode: "setup", installed: false };

  const listeners = {};
  const emit = (name, payload) =>
    (listeners[name] || []).forEach((h) => h({ payload }));

  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

  const invoke = async (cmd, args = {}) => {
    switch (cmd) {
      case "get_state":
        return state;
      case "preflight":
        return {
          canProceed: true,
          checks: [
            { id: "model", label: "Steam Deck detected", ok: true, detail: disk.model },
            { id: "nvme", label: "Internal NVMe present", ok: true, detail: disk.device },
            { id: "desktop", label: "Running in Desktop Mode", ok: true, detail: "KDE Plasma" },
            { id: "kvm", label: "Virtualization (KVM) available", ok: true, detail: "/dev/kvm" },
            { id: "power", label: "On AC power", ok: true, detail: "Charger connected, 87%" },
            { id: "space", label: "Enough free space", ok: true, detail: `${disk.freeGib} GiB shrinkable` },
            { id: "secureboot", label: "Secure Boot disabled", ok: true, detail: "OK — unsigned loader can run" },
          ],
        };
      case "probe_disk":
        return disk;
      case "plan_partitions": {
        const win = args.windowsGib ?? disk.minWindowsGib;
        return {
          before: [
            { n: 1, name: "esp", fs: "vfat", gib: 0.06, role: "ESP (shared)" },
            { n: 8, name: "home", fs: "ext4", gib: 380, role: "SteamOS data" },
          ],
          after: [
            { n: 1, name: "esp", fs: "vfat", gib: 0.06, role: "ESP (shared)" },
            { n: 8, name: "home", fs: "ext4", gib: 380 - win, role: "SteamOS data (shrunk)" },
            { n: 9, name: "msr", fs: "—", gib: 0.016, role: "Windows MSR", added: true },
            { n: 10, name: "windows", fs: "ntfs", gib: win, role: "Windows 11", added: true },
          ],
        };
      }
      case "pick_iso":
        return "/home/deck/Downloads/Win11_24H2_English_x64.iso";
      case "validate_iso":
        return {
          ok: true,
          edition: "Windows 11 Pro",
          arch: "x64",
          build: "26100 (24H2)",
          detail: "sources/install.wim present",
        };
      case "start_install": {
        const steps = [
          ["backup", "Backing up GPT + ESP"],
          ["shrink", "Shrinking SteamOS home filesystem"],
          ["partition", "Creating Windows MSR + NTFS partitions"],
          ["stage", "Generating autounattend.xml"],
          ["install", "Installing Windows 11 (QEMU/KVM)"],
          ["boot", "Registering boot picker + heal hook"],
        ];
        (async () => {
          for (let i = 0; i < steps.length; i++) {
            const [id, label] = steps[i];
            const sub = id === "install" ? 8 : 3;
            for (let s = 1; s <= sub; s++) {
              await sleep(220);
              emit("install://progress", {
                step: id,
                label,
                stepIndex: i,
                stepCount: steps.length,
                pct: Math.round(((i + s / sub) / steps.length) * 100),
                status: "running",
                log: `${label} — ${Math.round((s / sub) * 100)}%`,
              });
            }
          }
          emit("install://progress", {
            step: "done", label: "Done", stepIndex: steps.length, stepCount: steps.length,
            pct: 100, status: "done", log: "Installation complete. Reboot to finish.",
          });
        })();
        return { started: true };
      }
      case "set_default_os":
      case "set_timeout":
      case "repair_bootloader":
      case "uninstall":
      case "reboot":
        return { ok: true };
      case "get_config":
        return { defaultOs: "steamos", timeoutSeconds: 5, windowsGib: 128 };
      default:
        throw new Error(`mock: unknown command ${cmd}`);
    }
  };

  const listen = async (name, handler) => {
    (listeners[name] ||= []).push(handler);
    return () => {
      listeners[name] = (listeners[name] || []).filter((h) => h !== handler);
    };
  };

  return { invoke, listen };
}
