//! Thin wrapper around running external system tools (parted, resize2fs, qemu,
//! efibootmgr, …) with two safety properties:
//!
//!   1. **Dry-run by default.** Destructive commands only execute when the
//!      `live` cargo feature is on *or* `SDB_LIVE=1` is set in the environment.
//!      Otherwise the exact command line is logged and a synthetic success is
//!      returned, so the whole pipeline is exercisable on loopback images / a
//!      dev box without ever touching a real disk.
//!   2. **Privilege elevation** via `pkexec` for commands that need root.

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

/// Run a read-only/safe command for real (always executed) and return stdout.
pub fn run_ro(program: &str, args: &[&str]) -> Result<String> {
    exec(program, args)
}

/// Run a destructive command. In non-live mode it is logged and skipped.
pub fn run_priv(program: &str, args: &[&str]) -> Result<String> {
    let line = format!("{program} {}", args.join(" "));
    if !live() {
        return Ok(format!("DRY-RUN: {line}"));
    }
    if is_root() {
        exec(program, args)
    } else {
        let mut full = vec![program];
        full.extend_from_slice(args);
        exec("pkexec", &full)
    }
}

fn exec(program: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(program).args(args).output()?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(EngineError::Cmd {
            cmd: format!("{program} {}", args.join(" ")),
            code: out.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }
}
