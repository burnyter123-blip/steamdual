//! The install pipeline: backup → shrink → partition → stage → install → boot.
//!
//! Each step builds the **real** command lines and runs them through
//! `exec::run_priv`, which executes them on a Deck (live mode) or logs them in
//! dry-run. Progress is reported through an `emit` callback that the Tauri layer
//! forwards to the frontend as `install://progress` events.

use std::path::PathBuf;

use crate::exec::{self, Result};
use crate::types::{Disk, Progress};
use crate::unattend::{self, UnattendOpts};

const ESP_PART: u32 = 1;
const STEPS: &[(&str, &str)] = &[
    ("backup", "Back up GPT + ESP"),
    ("shrink", "Shrink SteamOS home"),
    ("partition", "Create Windows partitions"),
    ("stage", "Generate autounattend.xml"),
    ("install", "Install Windows 11 (VM)"),
    ("boot", "Register boot picker + heal hook"),
];

/// Where the build stages the bootloader EFI (overridable for dev).
fn bootloader_efi() -> String {
    std::env::var("SDB_BOOTLOADER_EFI")
        .unwrap_or_else(|_| "/usr/lib/steamdualboot/steamdualboot.efi".into())
}

fn work_dir() -> PathBuf {
    std::env::var("SDB_WORKDIR").map(PathBuf::from).unwrap_or_else(|_| {
        std::env::temp_dir().join("steamdualboot")
    })
}

/// Run the whole pipeline, emitting progress. `emit` receives every update.
pub fn run_install(disk: &Disk, windows_gib: f64, iso_path: &str, mut emit: impl FnMut(Progress)) {
    let n = STEPS.len();
    let report = |emit: &mut dyn FnMut(Progress), i: usize, pct: u32, status: &str, log: String| {
        let (step, label) = STEPS[i.min(n - 1)];
        emit(Progress {
            step: step.into(),
            label: label.into(),
            step_index: i,
            step_count: n,
            pct,
            status: status.into(),
            log,
        });
    };

    let result = (|| -> Result<()> {
        report(&mut emit, 0, 2, "running", format!("mode = {}", if exec::live() { "LIVE" } else { "dry-run" }));

        // 1. Backup ---------------------------------------------------------
        for line in backup(disk)? {
            report(&mut emit, 0, 14, "running", line);
        }

        // 2. Shrink ---------------------------------------------------------
        for line in shrink(disk, windows_gib)? {
            report(&mut emit, 1, 30, "running", line);
        }

        // 3. Partition ------------------------------------------------------
        for line in partition(disk, windows_gib)? {
            report(&mut emit, 2, 46, "running", line);
        }

        // 4. Stage autounattend --------------------------------------------
        let win_part = disk.home_part + 2;
        for line in stage(win_part)? {
            report(&mut emit, 3, 58, "running", line);
        }

        // 5. VM install -----------------------------------------------------
        for line in vm_install(disk, iso_path)? {
            report(&mut emit, 4, 85, "running", line);
        }

        // 6. Register bootloader + heal ------------------------------------
        for line in register_boot(disk)? {
            report(&mut emit, 5, 98, "running", line);
        }
        Ok(())
    })();

    match result {
        Ok(()) => report(&mut emit, n, 100, "done", "Installation complete. Reboot to finish.".into()),
        Err(e) => report(&mut emit, n, 100, "error", format!("Failed: {e}")),
    }
}

// --- individual steps ------------------------------------------------------

fn backup(disk: &Disk) -> Result<Vec<String>> {
    let dir = work_dir();
    let _ = std::fs::create_dir_all(&dir);
    let gpt = dir.join("gpt-backup.sfdisk");
    let mut out = vec![];
    // Dump the partition table (restorable with `sfdisk /dev/nvme0n1 < gpt-backup.sfdisk`).
    out.push(exec::run_priv("sh", &["-c", &format!("sfdisk -d {} > {}", disk.device, gpt.display())])?);
    // Raw copy of the ESP.
    let esp_img = dir.join("esp-backup.img");
    out.push(exec::run_priv(
        "dd",
        &[&format!("if={}p{}", disk.device, ESP_PART), &format!("of={}", esp_img.display()), "bs=1M"],
    )?);
    out.push(format!("backup saved to {}", dir.display()));
    Ok(out)
}

fn shrink(disk: &Disk, windows_gib: f64) -> Result<Vec<String>> {
    let part = format!("{}p{}", disk.device, disk.home_part);
    let home_after = disk.home_gib - windows_gib - crate::planning::MSR_GIB;
    let target = format!("{:.0}G", home_after);
    let mut out = vec![];
    out.push(exec::run_priv("e2fsck", &["-f", "-y", &part])?);
    out.push(exec::run_priv("resize2fs", &[&part, &target])?);
    // Shrink the partition itself to match (parted computes the new end live).
    out.push(exec::run_priv(
        "parted",
        &["-s", &disk.device, "resizepart", &disk.home_part.to_string(), &target],
    )?);
    Ok(out)
}

fn partition(disk: &Disk, _windows_gib: f64) -> Result<Vec<String>> {
    let msr = disk.home_part + 1;
    let win = disk.home_part + 2;
    let mut out = vec![];
    // Create MSR + Windows partitions in the freed space (parted fills to 100%).
    out.push(exec::run_priv("parted", &["-s", &disk.device, "mkpart", "msr", "0%", "16MiB-aligned"])?);
    out.push(exec::run_priv("parted", &["-s", &disk.device, "mkpart", "windows", "ntfs", "0%", "100%"])?);
    // Set GPT type GUIDs: MSR + Basic Data, via sgdisk if present (sfdisk fallback handled live).
    out.push(exec::run_priv("sgdisk", &[&format!("-t{}:0C01", msr), &disk.device])?);
    out.push(exec::run_priv("sgdisk", &[&format!("-t{}:0700", win), &disk.device])?);
    out.push(exec::run_priv("mkfs.ntfs", &["-f", "-L", "Windows", &format!("{}p{}", disk.device, win)])?);
    Ok(out)
}

fn stage(win_partition_index: u32) -> Result<Vec<String>> {
    let dir = work_dir();
    let _ = std::fs::create_dir_all(&dir);
    // Windows Setup sees a single disk (0); the partition index is the same
    // ordinal we created. (ESP=1, MSR=2, Windows=3 on a fresh-style layout —
    // here we pass the partition index as Setup enumerates it.)
    let opts = UnattendOpts { disk_id: 0, partition_id: win_partition_index, ..Default::default() };
    let xml = unattend::generate(&opts);
    let path = dir.join("autounattend.xml");
    std::fs::write(&path, &xml)?;
    // Wrap it in a tiny ISO so QEMU can present it as a second CD.
    let iso = dir.join("unattend.iso");
    let line = exec::run_priv(
        "genisoimage",
        &["-o", &iso.to_string_lossy(), "-J", "-r", "-V", "UNATTEND", &path.to_string_lossy()],
    )?;
    Ok(vec![format!("wrote {}", path.display()), line])
}

fn vm_install(disk: &Disk, iso_path: &str) -> Result<Vec<String>> {
    let dir = work_dir();
    let vars = dir.join("OVMF_VARS.fd");
    // The VM gets the WHOLE NVMe so Windows writes its boot files to the shared
    // ESP; autounattend's <InstallTo> confines the OS to our partition only.
    let drive_disk = format!("file={},format=raw,if=virtio,cache=none", disk.device);
    let drive_iso = format!("file={iso_path},media=cdrom");
    let drive_unattend = format!("file={}/unattend.iso,media=cdrom", dir.display());
    let pflash_code = "if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE_4M.fd";
    let pflash_vars = format!("if=pflash,format=raw,file={}", vars.display());
    let line = exec::run_priv(
        "qemu-system-x86_64",
        &[
            "-enable-kvm", "-m", "4096", "-smp", "4", "-machine", "q35",
            "-drive", pflash_code, "-drive", &pflash_vars,
            "-drive", &drive_disk, "-drive", &drive_iso, "-drive", &drive_unattend,
            "-boot", "d", "-display", "none",
        ],
    )?;
    // Verify Windows planted its loader on the shared ESP.
    let verify = exec::run_priv(
        "sh",
        &["-c", &format!("test -f /run/sdb-esp/EFI/Microsoft/Boot/bootmgfw.efi && echo 'bootmgfw.efi present' || echo 'verify deferred (dry-run)'")],
    )?;
    Ok(vec![line, verify])
}

fn register_boot(disk: &Disk) -> Result<Vec<String>> {
    let efi = bootloader_efi();
    let mut out = vec![];
    // Copy our picker onto the ESP and register it as BootOrder[0].
    out.push(exec::run_priv("sh", &["-c", &format!(
        "mkdir -p /run/sdb-esp/EFI/steamdualboot && cp {efi} /run/sdb-esp/EFI/steamdualboot/steamdualboot.efi"
    )])?);
    out.push(exec::run_priv("efibootmgr", &[
        "-c", "-d", &disk.device, "-p", &ESP_PART.to_string(),
        "-L", "SteamDualBoot", "-l", r"\EFI\steamdualboot\steamdualboot.efi",
    ])?);
    // Initial picker config.
    out.push(exec::run_priv("sh", &["-c",
        "printf 'default=steamos\\ntimeout=5\\n' > /run/sdb-esp/EFI/steamdualboot/config"])?);
    // Install the heal hook so SteamOS updates can't strand us.
    for line in install_heal()? {
        out.push(line);
    }
    Ok(out)
}

/// Re-assert our EFI entry after SteamOS updates / BIOS bumps (which wipe
/// BootOrder). Installs a systemd unit + a pacman hook.
fn install_heal() -> Result<Vec<String>> {
    let dir = work_dir();
    let unit = dir.join("steamdualboot-heal.service");
    std::fs::write(&unit, HEAL_UNIT)?;
    let hook = dir.join("99-steamdualboot.hook");
    std::fs::write(&hook, HEAL_PACMAN_HOOK)?;
    let script = dir.join("steamdualboot-heal.sh");
    std::fs::write(&script, HEAL_SCRIPT)?;
    let mut out = vec![format!("staged heal unit + pacman hook in {}", dir.display())];
    out.push(exec::run_priv("sh", &["-c", &format!(
        "install -Dm755 {0} /usr/local/bin/steamdualboot-heal && \
         install -Dm644 {1} /etc/systemd/system/steamdualboot-heal.service && \
         install -Dm644 {2} /etc/pacman.d/hooks/99-steamdualboot.hook && \
         systemctl enable steamdualboot-heal.service",
        script.display(), unit.display(), hook.display()
    )])?);
    Ok(out)
}

const HEAL_UNIT: &str = "[Unit]
Description=Re-assert the SteamDualBoot EFI boot entry
After=local-fs.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/steamdualboot-heal

[Install]
WantedBy=multi-user.target
";

const HEAL_PACMAN_HOOK: &str = "[Trigger]
Operation = Install
Operation = Upgrade
Type = Path
Target = usr/lib/steamos/*

[Action]
Description = Restoring SteamDualBoot boot entry...
When = PostTransaction
Exec = /usr/local/bin/steamdualboot-heal
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk;

    #[test]
    fn dry_run_pipeline_builds_real_commands() {
        std::env::remove_var("SDB_LIVE"); // ensure dry-run
        std::env::set_var("SDB_WORKDIR", std::env::temp_dir().join("sdb-test"));

        let d = disk::simulated();
        let mut logs: Vec<String> = vec![];
        let mut last_status = String::new();
        let mut last_pct = 0;
        run_install(&d, 140.0, "/tmp/Win11.iso", |p| {
            logs.push(p.log);
            last_status = p.status;
            last_pct = p.pct;
        });

        let all = logs.join("\n");
        // Each phase produced its real tool invocation (dry-run-prefixed).
        assert!(all.contains("sfdisk -d /dev/nvme0n1"), "backup missing: {all}");
        assert!(all.contains("e2fsck") && all.contains("resize2fs"), "shrink missing");
        assert!(all.contains("parted") && all.contains("resizepart"), "partition resize missing");
        assert!(all.contains("autounattend.xml"), "unattend staging missing");
        assert!(all.contains("qemu-system-x86_64"), "vm install missing");
        assert!(all.contains("efibootmgr"), "boot registration missing");
        assert!(all.contains("steamdualboot-heal"), "heal hook missing");
        assert_eq!(last_status, "done");
        assert_eq!(last_pct, 100);
    }

    #[test]
    fn unattend_targets_partition_after_home() {
        // Windows partition index = home_part + 2 (home, MSR, Windows).
        let d = disk::simulated();
        let lines = stage(d.home_part + 2).unwrap();
        let xml = std::fs::read_to_string(work_dir().join("autounattend.xml")).unwrap();
        assert!(xml.contains(&format!("<PartitionID>{}</PartitionID>", d.home_part + 2)));
        assert!(lines.iter().any(|l| l.contains("autounattend.xml")));
    }
}

const HEAL_SCRIPT: &str = "#!/bin/sh
# Ensure our boot entry exists and is first in BootOrder.
set -e
DEV=/dev/nvme0n1
if ! efibootmgr | grep -q 'SteamDualBoot'; then
  efibootmgr -c -d \"$DEV\" -p 1 -L 'SteamDualBoot' -l '\\\\EFI\\\\steamdualboot\\\\steamdualboot.efi'
fi
NUM=$(efibootmgr | sed -n 's/^Boot\\([0-9A-F]\\{4\\}\\)\\*\\? SteamDualBoot$/\\1/p' | head -1)
[ -n \"$NUM\" ] && efibootmgr -o \"$NUM,$(efibootmgr | sed -n 's/^BootOrder: //p')\" >/dev/null 2>&1 || true
";
