# SteamDualBoot — Bootloader

A Steam **Big Picture**–styled UEFI **GOP** boot picker for the Steam Deck. At
power-on it shows SteamOS and Windows as two BP "capsule" tiles, navigable by
controller / keyboard, and chainloads the chosen OS. Installed to the ESP at
`\EFI\steamdualboot\steamdualboot.efi` and registered as the default boot entry
by the Tauri engine.

## What it does

- **Renders** the BP design language directly to the GOP framebuffer (software
  renderer): gradient backdrop, rounded tiles, the focus **lift + accent glow**,
  alpha-composited logos, and the A/B/Y controller-glyph footer.
- **Inputs** (`src/input.rs`): keyboard / firmware navigation is the guaranteed
  baseline (the Deck surfaces volume + d-pad through Simple Text Input). The
  **joystick** path decodes the built-in controller's HID reports via
  `EFI_USB_IO_PROTOCOL` — hardware-only, behind `input::controller`, degrading
  silently to the keyboard baseline when no gamepad enumerates (e.g. in QEMU).
- **Chainloads** SteamOS (`\EFI\steamos\steamcl.efi`) or Windows
  (`\EFI\Microsoft\Boot\bootmgfw.efi`) by building a device path on the boot
  volume — correct device-path loading matters so `bootmgfw.efi` finds its BCD.
- **Config** (`\EFI\steamdualboot\config`): `default` OS, `timeout`, `last`
  booted. Y sets the default; the engine writes the initial file.
- **Fails safe**: missing loader → an in-tile "NOT INSTALLED" notice; no GOP →
  text-mode fallback; chainload error → notice + return to the picker.

## Layout

| File | Role |
|---|---|
| `src/main.rs` | EFI entry, event loop, auto-boot countdown |
| `src/gop.rs` | framebuffer canvas + drawing primitives |
| `src/text.rs` | anti-aliased Noto Sans Mono text |
| `src/ui.rs` | the BP picker screen |
| `src/input.rs` | keyboard baseline + HID controller layer |
| `src/chainload.rs` | device-path load + start of the OS loader |
| `src/config.rs` | ESP config read/write |
| `src/theme.rs` | BP color tokens (ported from `SteamBPClone/tokens.css`) |
| `src/assets.rs` + `build.rs` | build-time PNG→premultiplied-BGRA logo blobs |

## Build & test

```bash
cargo build --release                      # -> target/x86_64-unknown-uefi/release/steamdualboot.efi
../scripts/run-bootloader.sh               # interactive QEMU + OVMF
../scripts/run-bootloader.sh --screenshot  # headless, dumps one frame to target/bootloader.ppm
```

In QEMU there is no gamepad, so navigate with the **arrow keys** (move),
**Enter/Space** (boot), **Esc** (firmware), **Y** (set default).

## Remaining hardware work

- Validate `EFI_USB_IO` HID decode of the Deck's built-in controller on real
  hardware and wire `input::controller::poll` (stick deadzone + edge-trigger are
  already implemented in `stick_x_to_event`).
- Confirm Windows `bootmgfw.efi` chainload end-to-end on a real dual-boot ESP.
