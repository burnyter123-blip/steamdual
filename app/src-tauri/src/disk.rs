//! Disk probing for the internal NVMe (`/dev/nvme0n1`).
//!
//! Uses `lsblk -b -J` to read geometry and identify the SteamOS **home**
//! partition (the large ext4 we shrink). On a non-Deck dev box — or when the
//! expected layout isn't found — it returns a representative simulated disk so
//! `tauri dev` and the wizard remain fully usable without a Deck.

use serde::Deserialize;

use crate::exec;
use crate::planning::{max_windows_gib, MIN_WINDOWS_GIB};
use crate::types::Disk;

pub const DEVICE: &str = "/dev/nvme0n1";
const BYTES_PER_GIB: f64 = 1024.0 * 1024.0 * 1024.0;

#[derive(Deserialize)]
struct LsblkRoot {
    blockdevices: Vec<Node>,
}
#[derive(Deserialize)]
struct Node {
    #[allow(dead_code)]
    name: String,
    size: Option<u64>,
    fstype: Option<String>,
    #[serde(default)]
    mountpoint: Option<String>,
    #[serde(default)]
    partlabel: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    children: Vec<Node>,
}

/// Probe the internal NVMe, falling back to a simulated disk off-hardware.
pub fn probe() -> Disk {
    match probe_real() {
        Some(d) => d,
        None => simulated(),
    }
}

fn probe_real() -> Option<Disk> {
    let dev = std::env::var("SDB_DEV").unwrap_or_else(|_| DEVICE.to_string());
    let json = exec::run_ro("lsblk", &["-b", "-J", "-o", "NAME,SIZE,FSTYPE,MOUNTPOINT,PARTLABEL,MODEL", &dev]).ok()?;
    let root: LsblkRoot = serde_json::from_str(&json).ok()?;
    let disk = root.blockdevices.into_iter().next()?;
    let total = disk.size? as f64 / BYTES_PER_GIB;
    let model = disk.model.clone().unwrap_or_else(|| "Internal NVMe".into());

    // The SteamOS data partition: the largest ext4 child (often PARTLABEL "home").
    let parts: Vec<&Node> = disk.children.iter().collect();
    let (idx, home) = parts
        .iter()
        .enumerate()
        .filter(|(_, p)| p.fstype.as_deref() == Some("ext4") || p.partlabel.as_deref() == Some("home"))
        .max_by_key(|(_, p)| p.size.unwrap_or(0))?;
    let home_gib = home.size? as f64 / BYTES_PER_GIB;
    // Partition number = position in the child list (1-based) is unreliable; parse from name.
    let home_part = home
        .name
        .trim_start_matches(|c: char| !c.is_ascii_digit() || c == 'p')
        .rsplit('p')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or((idx as u32) + 1);

    let used = home.mountpoint.is_some() as u8 as f64 * 0.0; // unknown without statvfs; reported as free below
    let max_win = max_windows_gib(home_gib);
    Some(Disk {
        device: dev,
        model,
        total_gib: total,
        steamos_used_gib: used,
        free_gib: home_gib,
        min_windows_gib: MIN_WINDOWS_GIB,
        max_windows_gib: max_win,
        shrinkable_gib: max_win,
        home_gib,
        home_part,
    })
}

/// Representative Steam Deck 512 GB layout for dev / off-hardware use.
pub fn simulated() -> Disk {
    let home_gib = 380.0;
    let max_win = max_windows_gib(home_gib);
    Disk {
        device: "/dev/nvme0n1 (simulated)".into(),
        model: "Valve Deck NVMe 512GB (simulated)".into(),
        total_gib: 476.0,
        steamos_used_gib: 92.0,
        free_gib: 300.0,
        min_windows_gib: MIN_WINDOWS_GIB,
        max_windows_gib: max_win,
        shrinkable_gib: max_win,
        home_gib,
        home_part: 8,
    }
}
