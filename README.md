# SteamDualBoot

Add a **Windows 11 dual-boot** to a Steam Deck — guided end-to-end from a
friendly, Steam **Big Picture**–styled GUI, with a matching boot picker.

> ⚠️ **This software repartitions the internal drive and rewrites boot entries.**
> Those operations are irreversible. Every destructive step takes a GPT + ESP
> backup first and is gated behind an explicit typed confirmation.

## Components

| Dir | What | Status |
|---|---|---|
| [`bootloader/`](bootloader/) | UEFI **GOP** boot picker (Rust, `no_std`) in the BP design language, controller-navigable, chainloads SteamOS / Windows | ✅ builds + verified in QEMU+OVMF; HID controller decode is the remaining hardware task |
| [`app/`](app/) | **Tauri + React/Vite (JS)** app: the setup wizard, then a config pane. Ships as a **Flatpak** (Steam Deck) and an AppImage (desktop Linux) | ✅ full BP wizard, verified end-to-end |
| [`flatpak/`](flatpak/) | Flatpak manifest + desktop/metainfo. **The build that runs on the Steam Deck** — SteamOS has no system WebKit, so the AppImage's bundled WebKit can't initialize EGL (blank window); the GNOME-runtime WebKit renders fine | ✅ manifest + CI job; verify by installing on a Deck |
| `app/src-tauri/` | The **system engine** (Rust): disk probe/shrink, partitioning + backups, unattended Windows install via QEMU/KVM, `efibootmgr` registration, update self-healing | ✅ compiles, 8 tests pass, real probe verified; live disk run is the remaining hardware step |
| [`SteamBPClone/`](SteamBPClone/) | The Big Picture **design language** reference — the visual source of truth | 📐 reference |
| [`scripts/`](scripts/) | Dev harnesses (run the bootloader in QEMU, build loopback test disks) | |

## Design language

Everything visual derives from [`SteamBPClone/`](SteamBPClone/DESIGN_LANGUAGE.md):
the app imports its CSS tokens/components directly; the bootloader ports the same
color tokens into its framebuffer renderer. The signature **`.gpfocus`** focus
model (focused control lifts + inverts/glows) is reproduced in both.

## Decisions (v1)

- Bootloader = pure **UEFI GOP** app.
- Windows install = **VM install to the raw partition** (QEMU/KVM + autounattend).
- Target = **internal NVMe only** (`/dev/nvme0n1`).
- Live operations, with mandatory backups + a typed confirmation gate.

## Quick start (bootloader)

```bash
cd bootloader && cargo build --release
../scripts/run-bootloader.sh                 # QEMU + OVMF, BP picker in a window
```

See the implementation plan for the full architecture and the edge-case matrix.
