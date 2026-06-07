#!/usr/bin/env bash
# Exercise the engine's disk probing against a SAFE loopback image that mimics a
# Steam Deck layout (GPT: ESP + big ext4 "home"). This never touches a real disk.
#
# Needs root for losetup/mkfs. Run:  sudo scripts/test-engine-loopback.sh
#
# It sets SDB_DEV to the loop device and runs a tiny probe harness, so you can
# validate `disk::probe()` parsing on real (but disposable) geometry. The
# destructive pipeline stays in dry-run unless you also export SDB_LIVE=1
# (which, with SDB_DEV pointing at the loop device, is still safe).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IMG="$ROOT/target/loop-deck.img"
SIZE_GIB="${SIZE_GIB:-16}"   # small stand-in for the 512GB Deck

if [[ $EUID -ne 0 ]]; then echo "run with sudo"; exit 1; fi

echo ">> creating ${SIZE_GIB}GiB sparse image"
rm -f "$IMG"
truncate -s "${SIZE_GIB}G" "$IMG"

echo ">> GPT: p1 ESP (64MiB, vfat), p8 home (rest, ext4) — mirroring the Deck"
parted -s "$IMG" mklabel gpt
parted -s "$IMG" mkpart esp fat32 1MiB 65MiB
parted -s "$IMG" set 1 esp on
# Deck numbers home as p8; parted won't skip numbers, so we emulate with a single
# big data partition and rely on the probe picking the largest ext4.
parted -s "$IMG" mkpart home ext4 65MiB 100%

LOOP="$(losetup --show -fP "$IMG")"
trap 'losetup -d "$LOOP" 2>/dev/null || true' EXIT
echo ">> attached $LOOP"

mkfs.vfat "${LOOP}p1" >/dev/null
mkfs.ext4 -q -L home "${LOOP}p2"

echo ">> lsblk view the engine will parse:"
lsblk -b -o NAME,SIZE,FSTYPE,PARTLABEL "$LOOP"

echo ">> running probe harness (SDB_DEV=$LOOP)"
cd "$ROOT/app/src-tauri"
SDB_DEV="$LOOP" cargo run --quiet --example probe 2>/dev/null || {
  echo "   (add app/src-tauri/examples/probe.rs to print probe() output)"
}

echo ">> done. Detach handled on exit."
