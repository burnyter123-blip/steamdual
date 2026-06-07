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

    // Best-effort POSITIVE confirmation only. Windows install media is UDF and
    // its `sources/install.wim` is usually >4 GiB, so it lives ONLY in the UDF
    // filesystem and is invisible to ISO9660-only readers (bsdtar/isoinfo). We
    // therefore never *reject* on a content listing — a "not found" may just be
    // an ISO9660-only read — and only upgrade the message when we positively see
    // an installer via a UDF-capable reader.
    if iso_has_installer(path) {
        return IsoInfo {
            ok: true,
            edition: "Windows 11".into(),
            arch: "x64".into(),
            build: "detected".into(),
            detail: "sources/install.wim present".into(),
        };
    }

    IsoInfo {
        ok: true,
        edition: "Windows 11 ISO".into(),
        arch: "x64".into(),
        build: "unverified".into(),
        detail: format!("{gib:.0} GiB — selected (contents verified during install)"),
    }
}

/// True only if a reader can positively see the Windows installer image. UDF
/// readers (`7z`/`7za`) come first since that's where `install.wim` actually is.
fn iso_has_installer(path: &str) -> bool {
    let readers: [(&str, &[&str]); 4] = [
        ("7z", &["l", "-ba", path]),
        ("7za", &["l", "-ba", path]),
        ("bsdtar", &["-tf", path]),
        ("isoinfo", &["-f", "-i", path]),
    ];
    for (prog, args) in readers {
        if let Ok(out) = exec::run_ro(prog, args) {
            let l = out.to_lowercase();
            if l.contains("install.wim") || l.contains("install.esd") || l.contains("install.swm") {
                return true;
            }
        }
    }
    false
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_obvious_non_iso() {
        assert!(!validate_iso("/etc/hostname").ok); // wrong extension / too small
        assert!(!validate_iso("/no/such/file.iso").ok); // missing
    }

    #[test]
    fn accepts_real_windows_iso_when_provided() {
        // Opt-in: SDB_TEST_ISO=/path/to/Win11.iso cargo test
        if let Ok(p) = std::env::var("SDB_TEST_ISO") {
            let info = validate_iso(&p);
            assert!(info.ok, "a valid Windows ISO must be accepted, got: {}", info.detail);
        }
    }
}
