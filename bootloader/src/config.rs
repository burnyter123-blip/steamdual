//! Persisted picker configuration, stored on the ESP at
//! `\EFI\steamdualboot\config` as a tiny `key=value` text file:
//!
//! ```text
//! default=steamos
//! timeout=5
//! last=windows
//! ```
//!
//! The Tauri engine writes the initial file at install time; the picker updates
//! `default` / `last` when the user presses Y / boots an OS.

use uefi::boot;
use uefi::cstr16;
use uefi::fs::{FileSystem, Path};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Os {
    SteamOs,
    Windows,
}

impl Os {
    pub fn key(self) -> &'static str {
        match self {
            Os::SteamOs => "steamos",
            Os::Windows => "windows",
        }
    }
    fn parse(s: &str) -> Option<Os> {
        match s.trim() {
            "steamos" => Some(Os::SteamOs),
            "windows" => Some(Os::Windows),
            _ => None,
        }
    }
}

pub struct Config {
    pub default: Os,
    pub timeout: u8,
    pub last: Option<Os>,
}

impl Default for Config {
    fn default() -> Self {
        Config { default: Os::SteamOs, timeout: 5, last: None }
    }
}

const DIR: &uefi::CStr16 = cstr16!("\\EFI\\steamdualboot");
const PATH: &uefi::CStr16 = cstr16!("\\EFI\\steamdualboot\\config");

fn fs() -> uefi::Result<FileSystem> {
    let img = boot::image_handle();
    Ok(boot::get_image_file_system(img)?.into())
}

impl Config {
    /// Load config from the ESP, falling back to defaults on any error.
    pub fn load() -> Config {
        let mut cfg = Config::default();
        let Ok(mut fs) = fs() else { return cfg };
        let Ok(bytes) = fs.read(Path::new(PATH)) else { return cfg };
        let text = core::str::from_utf8(&bytes).unwrap_or("");
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            match k.trim() {
                "default" => {
                    if let Some(o) = Os::parse(v) {
                        cfg.default = o;
                    }
                }
                "timeout" => {
                    if let Ok(t) = v.trim().parse::<u8>() {
                        cfg.timeout = t.min(60);
                    }
                }
                "last" => cfg.last = Os::parse(v),
                _ => {}
            }
        }
        cfg
    }

    /// Persist config back to the ESP. Best-effort: errors are swallowed so a
    /// read-only / full ESP never blocks booting.
    pub fn save(&self) {
        let Ok(mut fs) = fs() else { return };
        let _ = fs.create_dir_all(Path::new(DIR));
        let mut s = alloc::string::String::new();
        s.push_str("default=");
        s.push_str(self.default.key());
        s.push_str("\ntimeout=");
        // u8 -> decimal without std fmt allocations beyond String.
        let mut n = self.timeout;
        let mut digits = [0u8; 3];
        let mut i = 3;
        loop {
            i -= 1;
            digits[i] = b'0' + (n % 10);
            n /= 10;
            if n == 0 {
                break;
            }
        }
        s.push_str(core::str::from_utf8(&digits[i..]).unwrap_or("5"));
        if let Some(last) = self.last {
            s.push_str("\nlast=");
            s.push_str(last.key());
        }
        s.push('\n');
        let _ = fs.write(Path::new(PATH), s.as_bytes());
    }
}
