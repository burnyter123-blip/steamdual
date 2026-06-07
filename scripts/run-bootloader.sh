#!/usr/bin/env bash
# Build the bootloader and run it in QEMU + OVMF against a synthetic ESP.
#
# The fake ESP mimics a Steam Deck layout so chainload paths resolve:
#   \EFI\BOOT\BOOTX64.EFI               <- our picker (firmware fallback path)
#   \EFI\steamdualboot\steamdualboot.efi
#   \EFI\steamdualboot\config           <- default=windows timeout=5
#   \EFI\steamos\steamcl.efi            <- dummy loader (so the tile shows "present")
#   \EFI\Microsoft\Boot\bootmgfw.efi    <- dummy loader
#
# Usage:
#   scripts/run-bootloader.sh            # interactive GUI window
#   scripts/run-bootloader.sh --screenshot out.ppm   # headless, dump one frame
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BL="$ROOT/bootloader"
ESP="$ROOT/target/esp"
EFI="$BL/target/x86_64-unknown-uefi/release/steamdualboot.efi"

echo ">> building bootloader"
( cd "$BL" && cargo build --release )

echo ">> staging synthetic ESP at $ESP"
rm -rf "$ESP"
mkdir -p "$ESP/EFI/BOOT" "$ESP/EFI/steamdualboot" "$ESP/EFI/steamos" "$ESP/EFI/Microsoft/Boot"
cp "$EFI" "$ESP/EFI/BOOT/BOOTX64.EFI"
cp "$EFI" "$ESP/EFI/steamdualboot/steamdualboot.efi"
printf 'default=windows\ntimeout=5\n' > "$ESP/EFI/steamdualboot/config"
# Dummy loaders so both tiles render as "installed" (they won't actually boot).
printf 'dummy' > "$ESP/EFI/steamos/steamcl.efi"
printf 'dummy' > "$ESP/EFI/Microsoft/Boot/bootmgfw.efi"
# The OVMF shell auto-runs startup.nsh; use it to launch our picker.
printf '@echo -off\nFS0:\n\\EFI\\BOOT\\BOOTX64.EFI\n' > "$ESP/startup.nsh"

# Locate OVMF firmware (Debian/Ubuntu split CODE/VARS), pipefail-safe.
pick() { for f in "$@"; do [[ -f "$f" ]] && { echo "$f"; return; }; done; }
OVMF_CODE="$(pick /usr/share/OVMF/OVMF_CODE_4M.fd /usr/share/OVMF/OVMF_CODE.fd)"
OVMF_VARS_SRC="$(pick /usr/share/OVMF/OVMF_VARS_4M.fd /usr/share/OVMF/OVMF_VARS.fd)"
[[ -n "$OVMF_CODE" && -n "$OVMF_VARS_SRC" ]] || { echo "OVMF firmware not found"; exit 1; }
VARS="$ROOT/target/OVMF_VARS.fd"
cp "$OVMF_VARS_SRC" "$VARS"

QEMU_ARGS=(
  -machine q35 -m 512 -nodefaults
  -drive if=pflash,format=raw,unit=0,readonly=on,file="$OVMF_CODE"
  -drive if=pflash,format=raw,unit=1,file="$VARS"
  -drive format=raw,file=fat:rw:"$ESP"
  -device virtio-vga
)

if [[ "${1:-}" == "--screenshot" ]]; then
  OUT="${2:-$ROOT/target/bootloader.ppm}"
  echo ">> headless run, screenshot -> $OUT"
  # Drive QEMU via its monitor: wait for the shell countdown + UI paint, then dump.
  ( sleep 10; printf 'screendump %s\n' "$OUT"; sleep 1; printf 'quit\n' ) \
    | qemu-system-x86_64 "${QEMU_ARGS[@]}" -display none -monitor stdio -serial null
  echo ">> wrote $OUT"
else
  echo ">> interactive run (close the window to exit)"
  qemu-system-x86_64 "${QEMU_ARGS[@]}" -serial stdio
fi
