//! Input abstraction for the boot picker.
//!
//! Per the plan, the **guaranteed baseline** is keyboard / firmware navigation
//! via the UEFI Simple Text Input protocol — on the Steam Deck the firmware
//! surfaces the volume buttons + d-pad through this path (the same way its own
//! BIOS boot menu is navigable), and an attached USB keyboard works too.
//!
//! The richer **joystick** path (decoding the built-in controller's HID input
//! reports via `EFI_USB_IO_PROTOCOL`) is hardware-only and cannot be exercised
//! in QEMU; it is implemented behind [`controller`] as a best-effort layer that
//! degrades silently to the keyboard baseline when no HID gamepad enumerates.

use uefi::proto::console::text::{Key, ScanCode};

/// A normalized navigation event, independent of which device produced it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Event {
    Prev,       // left / up  / stick-left  — move highlight
    Next,       // right/ down / stick-right — move highlight
    Select,     // A / Enter / Space — boot the highlighted OS
    Back,       // B / Esc — firmware setup
    SetDefault, // Y — make highlight the default OS
}

/// Poll all input sources once (non-blocking). Returns the first event seen.
pub fn poll() -> Option<Event> {
    if let Some(ev) = poll_keyboard() {
        return Some(ev);
    }
    controller::poll()
}

/// Drain any buffered keystrokes so a held key from the firmware menu doesn't
/// immediately trigger an action on the first frame.
pub fn flush() {
    let _ = uefi::system::with_stdin(|stdin| {
        while let Ok(Some(_)) = stdin.read_key() {}
    });
}

fn poll_keyboard() -> Option<Event> {
    let key = uefi::system::with_stdin(|stdin| stdin.read_key()).ok()??;
    match key {
        Key::Special(ScanCode::LEFT) | Key::Special(ScanCode::UP) => Some(Event::Prev),
        Key::Special(ScanCode::RIGHT) | Key::Special(ScanCode::DOWN) => Some(Event::Next),
        Key::Special(ScanCode::ESCAPE) => Some(Event::Back),
        Key::Printable(c) => {
            let ch = char::from(c);
            match ch {
                '\r' | '\n' | ' ' => Some(Event::Select),
                'y' | 'Y' => Some(Event::SetDefault),
                'b' | 'B' => Some(Event::Back),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Built-in controller (HID over USB) support. Hardware-only; on dev/QEMU this
/// enumerates nothing and `poll` returns `None`, so the keyboard baseline drives.
pub mod controller {
    use super::Event;

    /// Left-stick deadzone as a fraction of full deflection.
    const DEADZONE: i16 = 8192;

    /// Best-effort poll of the Deck's built-in gamepad.
    ///
    /// TODO(hardware): locate the controller's `EFI_USB_IO_PROTOCOL` interface,
    /// issue an interrupt/`UsbControlTransfer` read of its HID input report, and
    /// decode left-stick X (with [`DEADZONE`] + edge-trigger so one flick moves
    /// one tile) plus the A/B/Y face buttons into [`Event`]s. Until validated on
    /// a real Deck this returns `None` and the keyboard path is authoritative.
    pub fn poll() -> Option<Event> {
        let _ = DEADZONE;
        None
    }

    /// Map a decoded left-stick X sample to a navigation event (edge-triggered
    /// by the caller). Extracted so it can be unit-tested without hardware.
    pub fn stick_x_to_event(x: i16) -> Option<Event> {
        if x <= -DEADZONE {
            Some(Event::Prev)
        } else if x >= DEADZONE {
            Some(Event::Next)
        } else {
            None
        }
    }
}
