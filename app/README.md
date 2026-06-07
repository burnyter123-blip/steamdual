# SteamDualBoot — App + System Engine

A **Tauri v2** desktop app: a React/Vite (JavaScript) frontend in the Steam Big
Picture design language, backed by a Rust **system engine**. Ships as an
**AppImage**. It guides the user through adding a Windows 11 dual-boot, then
becomes a configuration pane.

## Frontend (`src/`)

- Imports the `SteamBPClone` CSS tokens/components directly — the app *is* Big
  Picture (nav, `.bpm-*` controls, `.gpfocus` inversion, A/B/X/Y footer).
- `hooks/useGamepadFocus.js` — spatial focus nav (ported from the reference) +
  Gamepad API polling, so the whole wizard is controller- and keyboard-navigable.
- `steps/` — one screen each: Welcome/preflight → Disk size → Windows ISO →
  Review (typed-CONFIRM gate) → Install (live progress) → Done. `ConfigPane.jsx`
  is shown instead once Windows is installed.
- `lib/engine.js` — the command/event contract. Inside Tauri it calls the Rust
  backend; in a plain browser it falls back to a realistic **mock**, so the UI
  runs and demos standalone (a `DEMO` badge shows when mocked).

## System engine (`src-tauri/src/`)

| Module | Role |
|---|---|
| `lib.rs` | Tauri commands (the engine.js contract) + preflight checks |
| `disk.rs` | probe `/dev/nvme0n1` via `lsblk -J`; simulated fallback off-Deck |
| `planning.rs` | pure shrink/partition math (unit-tested) |
| `unattend.rs` | generate `autounattend.xml` — installs into the pre-made partition, bypasses Win11 TPM/SecureBoot checks (unit-tested) |
| `ops.rs` | the install pipeline: backup → shrink → partition → stage → VM install → register boot + heal hook |
| `exec.rs` | command runner: **dry-run by default**, `pkexec` elevation |
| `state.rs` | persisted config + Windows-ISO validation |

### Safety model

Destructive commands run **only** when the `live` cargo feature is set or
`SDB_LIVE=1` is in the environment. Otherwise every command line is logged and
skipped, so the full pipeline is exercisable on loopback images / a dev box
without touching a disk. A GPT + ESP backup is always taken first, and the UI
requires typing `CONFIRM` before the pipeline starts.

The bootloader is registered as `BootOrder[0]`; a systemd unit + pacman hook
(`ops::install_heal`) re-assert it after SteamOS updates wipe the boot order.

## Build & test

```bash
npm install
npm run build                 # frontend -> dist/
npm run tauri build           # -> src-tauri/target/release/bundle/appimage/*.AppImage
cd src-tauri && cargo test    # planning + unattend + dry-run pipeline tests

# Probe the real (or a loopback) disk read-only:
cargo run --example probe                       # simulated/real auto
sudo ../../scripts/test-engine-loopback.sh      # safe loopback geometry
```

`SDB_LIVE=1` enables real execution (Deck only). `SDB_DEV`, `SDB_WORKDIR`,
`SDB_BOOTLOADER_EFI` override targets for testing.
