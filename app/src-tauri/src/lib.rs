//! SteamDualBoot system engine — Tauri command layer.
//!
//! Wires the React frontend (`app/src/lib/engine.js`) to the engine modules.
//! The heavy/destructive work lives in `ops.rs` and is dry-run-guarded; the
//! commands here are thin and mostly assemble typed results.

mod disk;
mod exec;
mod ops;
mod planning;
mod state;
mod types;
mod unattend;

use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::DialogExt;

use types::*;

#[tauri::command]
fn get_state() -> AppState {
    let installed = state::is_installed();
    AppState { mode: if installed { "config".into() } else { "setup".into() }, installed }
}

#[tauri::command]
fn preflight() -> Preflight {
    let d = disk::probe();
    let exists = |p: &str| std::path::Path::new(p).exists();
    let dmi = std::fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
        .unwrap_or_default();
    let is_deck = matches!(dmi.trim(), "Jupiter" | "Galileo");

    let mut checks = vec![
        Check {
            id: "model".into(),
            label: "Steam Deck detected".into(),
            ok: is_deck || !exec::live(),
            detail: if is_deck { dmi.trim().into() } else { format!("{} (dev: not a Deck)", d.model) },
            fatal: false,
        },
        Check {
            id: "nvme".into(),
            label: "Internal NVMe present".into(),
            ok: exists(disk::DEVICE) || !exec::live(),
            detail: d.device.clone(),
            fatal: true,
        },
        Check {
            id: "desktop".into(),
            label: "Running in Desktop Mode".into(),
            ok: true,
            detail: std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "unknown".into()),
            fatal: false,
        },
        Check {
            id: "kvm".into(),
            label: "Virtualization (KVM) available".into(),
            ok: exists("/dev/kvm") || !exec::live(),
            detail: if exists("/dev/kvm") { "/dev/kvm".into() } else { "no /dev/kvm (dry-run ok)".into() },
            fatal: true,
        },
        Check {
            id: "power".into(),
            label: "On AC power".into(),
            ok: on_ac_power().unwrap_or(true),
            detail: if on_ac_power().unwrap_or(false) { "Charger connected".into() } else { "Plug in before installing".into() },
            fatal: false,
        },
        Check {
            id: "space".into(),
            label: "Enough free space".into(),
            ok: d.shrinkable_gib >= planning::MIN_WINDOWS_GIB,
            detail: format!("{:.0} GiB shrinkable", d.shrinkable_gib),
            fatal: true,
        },
        Check {
            id: "secureboot".into(),
            label: "Secure Boot disabled".into(),
            ok: !secure_boot_enabled().unwrap_or(false),
            detail: if secure_boot_enabled().unwrap_or(false) {
                "Enabled — disable it so the unsigned picker can run".into()
            } else {
                "OK — unsigned loader can run".into()
            },
            fatal: false,
        },
    ];
    // Stable order for the UI.
    checks.sort_by_key(|c| c.id.clone());
    let can_proceed = checks.iter().all(|c| !c.fatal || c.ok);
    Preflight { can_proceed, checks }
}

fn on_ac_power() -> Option<bool> {
    for entry in std::fs::read_dir("/sys/class/power_supply").ok()? {
        let p = entry.ok()?.path();
        if let Ok(t) = std::fs::read_to_string(p.join("type")) {
            if t.trim() == "Mains" {
                let online = std::fs::read_to_string(p.join("online")).ok()?;
                return Some(online.trim() == "1");
            }
        }
    }
    None
}

fn secure_boot_enabled() -> Option<bool> {
    // SecureBoot efivar: last data byte == 1 means enabled.
    let dir = std::fs::read_dir("/sys/firmware/efi/efivars").ok()?;
    for e in dir.flatten() {
        let name = e.file_name();
        if name.to_string_lossy().starts_with("SecureBoot-") {
            let data = std::fs::read(e.path()).ok()?;
            return data.last().map(|b| *b == 1);
        }
    }
    None
}

#[tauri::command]
fn probe_disk() -> Disk {
    disk::probe()
}

#[tauri::command]
fn plan_partitions(windows_gib: f64) -> Plan {
    let d = disk::probe();
    let win = planning::clamp_windows(d.home_gib, windows_gib);
    planning::plan(d.home_part, d.home_gib, win)
}

#[tauri::command]
async fn pick_iso(app: AppHandle) -> Option<String> {
    // Must be async + non-blocking: a synchronous `blocking_pick_file()` runs the
    // GTK/portal chooser on the main thread and deadlocks the whole UI (the Deck
    // "freeze" on click). Post the dialog without blocking, then wait for the
    // user's choice off the event loop.
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("Windows ISO", &["iso"])
        .pick_file(move |f| {
            let _ = tx.send(f);
        });
    tauri::async_runtime::spawn_blocking(move || rx.recv().ok().flatten())
        .await
        .ok()
        .flatten()
        .map(|p| p.to_string())
}

#[tauri::command]
fn validate_iso(path: String) -> IsoInfo {
    state::validate_iso(&path)
}

#[tauri::command]
fn start_install(app: AppHandle, windows_gib: f64, iso_path: String) -> serde_json::Value {
    let d = disk::probe();
    let win = planning::clamp_windows(d.home_gib, windows_gib);
    std::thread::spawn(move || {
        ops::run_install(&d, win, &iso_path, |p| {
            let done = p.status == "done";
            let _ = app.emit("install://progress", p);
            if done {
                // Record the install so the next launch opens the config pane.
                let mut cfg = state::load_config();
                cfg.windows_gib = win;
                state::save_config(&cfg);
            }
        });
    });
    serde_json::json!({ "started": true })
}

#[tauri::command]
fn get_config() -> Config {
    let mut cfg = state::load_config();
    if cfg.windows_gib <= 0.0 {
        cfg.windows_gib = 128.0; // sensible display default
    }
    cfg
}

#[tauri::command]
fn set_default_os(os: String) -> exec::Result<String> {
    let mut cfg = state::load_config();
    cfg.default_os = os.clone();
    state::save_config(&cfg);
    exec::run_priv("sh", &["-c", &format!(
        "printf 'default={}\\ntimeout={}\\n' > /run/sdb-esp/EFI/steamdualboot/config", os, cfg.timeout_seconds
    )])
}

#[tauri::command]
fn set_timeout(seconds: u32) -> exec::Result<String> {
    let mut cfg = state::load_config();
    cfg.timeout_seconds = seconds;
    state::save_config(&cfg);
    exec::run_priv("sh", &["-c", &format!(
        "printf 'default={}\\ntimeout={}\\n' > /run/sdb-esp/EFI/steamdualboot/config", cfg.default_os, seconds
    )])
}

#[tauri::command]
fn repair_bootloader() -> exec::Result<String> {
    exec::run_priv("/usr/local/bin/steamdualboot-heal", &[])
}

#[tauri::command]
fn uninstall() -> exec::Result<String> {
    let d = disk::probe();
    let msr = d.home_part + 1;
    let win = d.home_part + 2;
    let mut log = String::new();
    // Remove Windows partitions, give the space back to home, drop our boot entry.
    log += &exec::run_priv("parted", &["-s", &d.device, "rm", &win.to_string()])?;
    log += &exec::run_priv("parted", &["-s", &d.device, "rm", &msr.to_string()])?;
    log += &exec::run_priv("parted", &["-s", &d.device, "resizepart", &d.home_part.to_string(), "100%"])?;
    log += &exec::run_priv("resize2fs", &[&format!("{}p{}", d.device, d.home_part)])?;
    log += &exec::run_priv("sh", &["-c",
        "for n in $(efibootmgr | sed -n 's/^Boot\\([0-9A-F]\\{4\\}\\)\\* SteamDualBoot$/\\1/p'); do efibootmgr -b $n -B; done"])?;
    let mut cfg = state::load_config();
    cfg.windows_gib = 0.0;
    state::save_config(&cfg);
    Ok(log)
}

#[tauri::command]
fn reboot() -> exec::Result<String> {
    exec::run_priv("systemctl", &["reboot"])
}

/// Diagnostics helper for the loopback test harness: probe + plan as JSON.
pub fn diagnostics_probe() -> String {
    let d = disk::probe();
    let plan = planning::plan(d.home_part, d.home_gib, planning::clamp_windows(d.home_gib, 128.0));
    serde_json::to_string_pretty(&serde_json::json!({
        "disk": d,
        "home_gib": d.home_gib,
        "home_part": d.home_part,
        "plan": plan,
    }))
    .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // WebKitGTK's DMABUF renderer tries to create an EGL display for GPU
    // compositing. On several Linux GPU/driver stacks — notably the Steam Deck's
    // AMD + Mesa — that fails ("Could not create default EGL display:
    // EGL_BAD_PARAMETER. Aborting...") and the window renders blank white.
    // Force WebKit's compatible (non-DMABUF) path unless the user overrode it.
    // Must be set before GTK/WebKit initialize (i.e. before the builder runs).
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_state,
            preflight,
            probe_disk,
            plan_partitions,
            pick_iso,
            validate_iso,
            start_install,
            get_config,
            set_default_os,
            set_timeout,
            repair_bootloader,
            uninstall,
            reboot,
        ])
        .run(tauri::generate_context!())
        .expect("error while running SteamDualBoot");
}
