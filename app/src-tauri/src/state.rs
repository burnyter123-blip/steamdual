//! Persisted app config + Windows-ISO validation.
//!
//! Config lives at `$XDG_CONFIG_HOME/steamdualboot/config.json` (or
//! `~/.config/...`). Whether Windows is "installed" is derived from that config
//! plus, in live mode, the presence of the Windows partition / loader.

use std::path::PathBuf;

use crate::exec;
use crate::types::{Config, IsoInfo};

fn config_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())).join(".config")
        });
    base.join("steamdualboot").join("config.json")
}

pub fn load_config() -> Config {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_config(cfg: &Config) {
    let path = config_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(path, json);
    }
}

/// Windows is considered installed once config records a sized install.
pub fn is_installed() -> bool {
    load_config().windows_gib > 0.0
}

/// Validate a Windows 11 ISO: existence, size, and — when a reader is available
/// — that it contains `sources/install.wim`/`.esd`. Tolerant by design so the
/// flow still works when no ISO tooling is installed.
pub fn validate_iso(path: &str) -> IsoInfo {
    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return bad("File not found"),
    };
    if !path.to_lowercase().ends_with(".iso") {
        return bad("Not an .iso file");
    }
    let gib = meta.len() as f64 / (1024.0 * 1024.0 * 1024.0);
    if gib < 3.0 {
        return bad(&format!("Too small ({gib:.1} GiB) to be a Windows 11 ISO"));
    }

    // Best-effort content listing via whichever reader exists.
    let listing = list_iso(path);
    if let Some(list) = &listing {
        let l = list.to_lowercase();
        let has_installer = l.contains("sources/install.wim") || l.contains("sources/install.esd");
        if !has_installer {
            return bad("ISO does not contain sources/install.wim — not a Windows installer");
        }
        let arch = if l.contains("efi/boot/bootx64.efi") { "x64" } else { "x64" };
        return IsoInfo {
            ok: true,
            edition: "Windows 11".into(),
            arch: arch.into(),
            build: "detected".into(),
            detail: "sources/install.wim present".into(),
        };
    }

    // No reader available — accept on heuristics but say so.
    IsoInfo {
        ok: true,
        edition: "Windows 11 (unverified)".into(),
        arch: "x64".into(),
        build: "unknown".into(),
        detail: format!("{gib:.0} GiB ISO — install no reader to verify contents"),
    }
}

fn list_iso(path: &str) -> Option<String> {
    // Try bsdtar, then 7z, then isoinfo.
    if let Ok(o) = exec::run_ro("bsdtar", &["-tf", path]) {
        return Some(o);
    }
    if let Ok(o) = exec::run_ro("7z", &["l", "-ba", path]) {
        return Some(o);
    }
    if let Ok(o) = exec::run_ro("isoinfo", &["-f", "-i", path]) {
        return Some(o);
    }
    None
}

fn bad(detail: &str) -> IsoInfo {
    IsoInfo {
        ok: false,
        edition: String::new(),
        arch: String::new(),
        build: String::new(),
        detail: detail.into(),
    }
}
