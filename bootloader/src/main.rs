//! SteamDualBoot — a Steam Big Picture–styled UEFI boot picker.
//!
//! Installed to `\EFI\steamdualboot\steamdualboot.efi` on the Steam Deck ESP and
//! registered as the default boot entry. At power-on it presents SteamOS and
//! Windows as two BP "capsule" tiles, navigable by controller / keyboard, and
//! chainloads the chosen OS. See `../README` and the repo plan for the system.

#![no_std]
#![no_main]
// Some primitives (fill_rect, the HID stick decoder, extra theme tokens) are
// deliberate API surface for the controller/hardware work that lands next.
#![allow(dead_code)]

extern crate alloc;

mod assets;
mod chainload;
mod config;
mod gop;
mod input;
mod text;
mod theme;
mod ui;

use core::time::Duration;
use uefi::prelude::*;

use config::{Config, Os};
use input::Event;
use ui::{State, Tile};

#[entry]
fn main() -> Status {
    if uefi::helpers::init().is_err() {
        return Status::ABORTED;
    }

    let mut canvas = match gop::Canvas::new() {
        Ok(c) => c,
        // No GOP — fall back to the firmware text console rather than a black screen.
        Err(_) => return text_fallback(),
    };

    let mut cfg = Config::load();

    let tiles: [Tile; 2] = [
        Tile {
            os: Os::SteamOs,
            title: "SteamOS",
            subtitle: "Linux · Gaming",
            logo: &assets::STEAMOS,
            present: chainload::loader_present(chainload::STEAMOS_LOADER),
        },
        Tile {
            os: Os::Windows,
            title: "Windows 11",
            subtitle: "Microsoft",
            logo: &assets::WINDOWS,
            present: chainload::loader_present(chainload::WINDOWS_LOADER),
        },
    ];

    let mut st = State {
        sel: match cfg.default {
            Os::SteamOs => 0,
            Os::Windows => 1,
        },
        default: cfg.default,
        countdown: if cfg.timeout > 0 { Some(cfg.timeout) } else { None },
    };

    input::flush();

    loop {
        ui::render(&mut canvas, &tiles, &st);

        // Poll input for ~1s in 20ms slices; tick the countdown if it elapses.
        let mut acted = false;
        for _ in 0..50 {
            if let Some(ev) = input::poll() {
                handle(ev, &mut st, &mut cfg, &tiles, &mut canvas);
                acted = true;
                break;
            }
            boot::stall(Duration::from_millis(20));
        }

        if !acted {
            if let Some(secs) = st.countdown {
                if secs <= 1 {
                    // Time's up — boot the default OS.
                    boot_os(st.default, &tiles, &mut cfg, &mut canvas);
                    st.countdown = None; // boot failed; let the user choose
                } else {
                    st.countdown = Some(secs - 1);
                }
            }
        }
    }
}

fn handle(ev: Event, st: &mut State, cfg: &mut Config, tiles: &[Tile; 2], canvas: &mut gop::Canvas) {
    // Any interaction cancels the auto-boot countdown.
    st.countdown = None;
    match ev {
        Event::Prev => st.sel = st.sel.saturating_sub(1),
        Event::Next => st.sel = (st.sel + 1).min(tiles.len() - 1),
        Event::Select => {
            let os = tiles[st.sel].os;
            if tiles[st.sel].present {
                boot_os(os, tiles, cfg, canvas);
            }
        }
        Event::SetDefault => {
            cfg.default = tiles[st.sel].os;
            st.default = cfg.default;
            cfg.save();
        }
        Event::Back => enter_firmware(),
    }
}

/// Record the choice as `last`, persist, and chainload. Returns only on failure.
fn boot_os(os: Os, tiles: &[Tile; 2], cfg: &mut Config, canvas: &mut gop::Canvas) {
    let loader = match os {
        Os::SteamOs => chainload::STEAMOS_LOADER,
        Os::Windows => chainload::WINDOWS_LOADER,
    };
    if !tiles.iter().any(|t| t.os == os && t.present) {
        return;
    }
    cfg.last = Some(os);
    cfg.save();
    if chainload::boot_loader(loader).is_err() {
        // Chainload failed — paint a brief notice and return to the picker.
        canvas.gradient_v(theme::BG_TOP, theme::BG_BOTTOM);
        text::draw_centered(
            canvas, 0, canvas.w as i32, canvas.h as i32 / 2 - 16,
            "Failed to start that OS — its loader may be missing.",
            text::Size::H3, text::Weight::Bold, theme::DESTRUCTIVE,
        );
        canvas.present();
        boot::stall(Duration::from_secs(2));
    }
}

/// Best-effort: ask the firmware to reboot into its setup UI (B button).
fn enter_firmware() {
    use uefi::runtime::{VariableAttributes, VariableVendor};
    const BOOT_TO_FW_UI: u64 = 0x0000_0000_0000_0001;
    let name = cstr16!("OsIndications");
    let attrs = VariableAttributes::NON_VOLATILE
        | VariableAttributes::BOOTSERVICE_ACCESS
        | VariableAttributes::RUNTIME_ACCESS;
    let _ = uefi::runtime::set_variable(name, &VariableVendor::GLOBAL_VARIABLE, attrs, &BOOT_TO_FW_UI.to_le_bytes());
    uefi::runtime::reset(uefi::runtime::ResetType::COLD, Status::SUCCESS, None);
}

/// Last resort if GOP is unavailable: a plain text menu on the firmware console.
fn text_fallback() -> Status {
    let mut cfg = Config::load();
    log::info!("SteamDualBoot (text mode): default = {}", cfg.default.key());
    // Without graphics we simply boot the configured default after a short wait.
    boot::stall(Duration::from_secs(1));
    let loader = match cfg.default {
        Os::SteamOs => chainload::STEAMOS_LOADER,
        Os::Windows => chainload::WINDOWS_LOADER,
    };
    cfg.last = Some(cfg.default);
    cfg.save();
    let _ = chainload::boot_loader(loader);
    Status::ABORTED
}
