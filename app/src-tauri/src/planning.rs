//! Pure partition-planning math — the "what will change" diff shown on the
//! Review screen and used to drive the real partition operations. Kept free of
//! I/O so it is unit-testable.

use crate::types::{Part, Plan};

/// Windows reserves a 16 MiB Microsoft Reserved (MSR) partition.
pub const MSR_GIB: f64 = 16.0 / 1024.0;
/// ESP size on the Deck (~64 MiB), shown for context; we share it, not resize it.
pub const ESP_GIB: f64 = 64.0 / 1024.0;
/// Microsoft's hard floor for Windows 11 is 64 GB.
pub const MIN_WINDOWS_GIB: f64 = 64.0;
/// Always leave at least this much for SteamOS data after shrinking.
pub const MIN_STEAMOS_KEEP_GIB: f64 = 24.0;

/// Largest Windows size that still leaves `MIN_STEAMOS_KEEP_GIB` of home plus
/// the MSR partition.
pub fn max_windows_gib(home_gib: f64) -> f64 {
    (home_gib - MIN_STEAMOS_KEEP_GIB - MSR_GIB).max(0.0)
}

/// Build the before/after partition diff for carving `windows_gib` out of the
/// SteamOS home partition (`home_part`, currently `home_gib`).
pub fn plan(home_part: u32, home_gib: f64, windows_gib: f64) -> Plan {
    let home_after = home_gib - windows_gib - MSR_GIB;
    let before = vec![
        Part { n: 1, name: "esp".into(), fs: "vfat".into(), gib: ESP_GIB, role: "ESP (shared)".into(), added: false },
        Part { n: home_part, name: "home".into(), fs: "ext4".into(), gib: home_gib, role: "SteamOS data".into(), added: false },
    ];
    let after = vec![
        Part { n: 1, name: "esp".into(), fs: "vfat".into(), gib: ESP_GIB, role: "ESP (shared)".into(), added: false },
        Part { n: home_part, name: "home".into(), fs: "ext4".into(), gib: home_after, role: "SteamOS data (shrunk)".into(), added: false },
        Part { n: home_part + 1, name: "msr".into(), fs: "—".into(), gib: MSR_GIB, role: "Windows MSR".into(), added: true },
        Part { n: home_part + 2, name: "windows".into(), fs: "ntfs".into(), gib: windows_gib, role: "Windows 11".into(), added: true },
    ];
    Plan { before, after }
}

/// Clamp a requested Windows size into the feasible range for this disk.
pub fn clamp_windows(home_gib: f64, requested: f64) -> f64 {
    requested.clamp(MIN_WINDOWS_GIB, max_windows_gib(home_gib))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_conserves_space() {
        let p = plan(8, 380.0, 140.0);
        let before: f64 = p.before.iter().map(|x| x.gib).sum();
        let after: f64 = p.after.iter().map(|x| x.gib).sum();
        // After repartition, totals match (within float noise): we only moved space.
        assert!((before - after).abs() < 1e-9, "before={before} after={after}");
    }

    #[test]
    fn plan_adds_msr_and_windows() {
        let p = plan(8, 380.0, 140.0);
        assert_eq!(p.after.len(), 4);
        assert!(p.after.iter().any(|x| x.n == 9 && x.added && x.fs == "—"));
        assert!(p.after.iter().any(|x| x.n == 10 && x.added && x.fs == "ntfs" && (x.gib - 140.0).abs() < 1e-9));
        // Home shrank by exactly windows + MSR.
        let home_after = p.after.iter().find(|x| x.n == 8).unwrap().gib;
        assert!((home_after - (380.0 - 140.0 - MSR_GIB)).abs() < 1e-9);
    }

    #[test]
    fn max_windows_leaves_steamos_and_msr() {
        let m = max_windows_gib(380.0);
        assert!((m - (380.0 - MIN_STEAMOS_KEEP_GIB - MSR_GIB)).abs() < 1e-9);
    }

    #[test]
    fn clamp_respects_bounds() {
        assert_eq!(clamp_windows(380.0, 10.0), MIN_WINDOWS_GIB); // below floor -> floor
        assert_eq!(clamp_windows(380.0, 9999.0), max_windows_gib(380.0)); // above ceiling
        assert_eq!(clamp_windows(380.0, 128.0), 128.0); // in range -> unchanged
    }
}
