//! Thin wrapper around running external system tools (parted, resize2fs, qemu,
//! efibootmgr, …) with three properties:
//!
//!   1. **Dry-run by default.** Destructive commands only execute when the
//!      `live` cargo feature is on *or* `SDB_LIVE=1` is set in the environment.
//!      Otherwise the exact command line is logged and a synthetic success is
//!      returned, so the whole pipeline is exercisable on loopback images / a
//!      dev box without ever touching a real disk.
//!   2. **Privilege elevation** via `pkexec` for commands that need root.
//!   3. **Flatpak escape.** When running inside the Flatpak sandbox, every
//!      external tool is run on the host via `flatpak-spawn --host` — `lsblk`,
//!      `parted`, `qemu`, `efibootmgr` etc. live on the host, not in the
//!      sandbox, and the disk work must happen on the host anyway.

use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[allow(dead_code)] // constructed by future validation paths
    #[error("{0}")]
    Msg(String),
    #[error("command `{cmd}` failed ({code}): {stderr}")]
    Cmd { cmd: String, code: i32, stderr: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl serde::Serialize for EngineError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, EngineError>;

/// True when real, destructive execution is enabled.
pub fn live() -> bool {
    cfg!(feature = "live") || std::env::var("SDB_LIVE").as_deref() == Ok("1")
}

fn is_root() -> bool {
    // Avoid a libc dep: read the effective uid from /proc.
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .and_then(|l| l.split_whitespace().nth(2)) // effective uid
                .map(|u| u == "0")
        })
        .unwrap_or(false)
}

/// Inside the Flatpak sandbox? (`FLATPAK_ID`/`FLATPAK` env, or `/.flatpak-info`.)
fn in_flatpak() -> bool {
    std::env::var_os("FLATPAK_ID").is_some()
        || std::env::var_os("FLATPAK").is_some()
        || std::path::Path::new("/.flatpak-info").exists()
}

/// Run a read-only/safe command for real (always executed) and return stdout.
pub fn run_ro(program: &str, args: &[&str]) -> Result<String> {
    let mut argv = vec![program.to_string()];
    argv.extend(args.iter().map(|s| s.to_string()));
    exec(argv)
}

/// Run a destructive command. In non-live mode it is logged and skipped.
pub fn run_priv(program: &str, args: &[&str]) -> Result<String> {
    let line = format!("{program} {}", args.join(" "));
    if !live() {
        return Ok(format!("DRY-RUN: {line}"));
    }
    let mut argv = Vec::new();
    // Elevate unless we're already root (we never are inside Flatpak).
    if !is_root() {
        argv.push("pkexec".to_string());
    }
    argv.push(program.to_string());
    argv.extend(args.iter().map(|s| s.to_string()));
    exec(argv)
}

/// Execute an argv, transparently hopping out of the Flatpak sandbox to the host.
fn exec(argv: Vec<String>) -> Result<String> {
    let argv = if in_flatpak() {
        let mut wrapped = vec!["flatpak-spawn".to_string(), "--host".to_string()];
        wrapped.extend(argv);
        wrapped
    } else {
        argv
    };
    let out = Command::new(&argv[0]).args(&argv[1..]).output()?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(EngineError::Cmd {
            cmd: argv.join(" "),
            code: out.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }
}
