//! The boot-picker screen, drawn in the Steam Big Picture design language:
//! dark gradient backdrop, two large rounded "capsule" tiles (SteamOS /
//! Windows), the focused tile gets the BP lift + accent glow, and a footer
//! A/B/Y controller-glyph hint bar.

use crate::assets;
use crate::config::Os;
use crate::gop::Canvas;
use crate::text::{self, Size, Weight};
use crate::theme::*;

pub struct Tile {
    pub os: Os,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub logo: &'static assets::Logo,
    pub present: bool,
}

pub struct State {
    pub sel: usize,
    pub default: Os,
    /// Remaining auto-boot seconds, or `None` once the user has interacted.
    pub countdown: Option<u8>,
}

const TILE_W: i32 = 340;
const TILE_H: i32 = 420;
const GAP: i32 = 64;
const TOP: i32 = 196;
const LIFT: i32 = 10; // focused-tile scale "lift" in px per side

pub fn render(canvas: &mut Canvas, tiles: &[Tile; 2], st: &State) {
    let w = canvas.w as i32;
    canvas.gradient_v(BG_TOP, BG_BOTTOM);

    // --- Header ------------------------------------------------------------
    text::draw_centered(canvas, 0, w, 64, "STEAM DUALBOOT", Size::Body, Weight::Bold, ACCENT);
    text::draw_centered(canvas, 0, w, 92, "Choose an operating system", Size::H2, Weight::Bold, TEXT_WHITE);

    // --- Tiles -------------------------------------------------------------
    let total = TILE_W * 2 + GAP;
    let x0 = (w - total) / 2;
    for (i, tile) in tiles.iter().enumerate() {
        let tx = x0 + i as i32 * (TILE_W + GAP);
        draw_tile(canvas, tx, TOP, tile, i == st.sel, tile.os == st.default);
    }

    // --- Countdown / hint --------------------------------------------------
    let hint_y = TOP + TILE_H + 48;
    if let Some(secs) = st.countdown {
        let name = match tiles.iter().find(|t| t.os == st.default) {
            Some(t) => t.title,
            None => "default",
        };
        // "Starting Windows 11 in 5s  ·  press any key to choose"
        let mut line = alloc::string::String::new();
        line.push_str("Starting ");
        line.push_str(name);
        line.push_str(" in ");
        push_u8(&mut line, secs);
        line.push_str("s   ·   press any input to choose");
        text::draw_centered(canvas, 0, w, hint_y, &line, Size::Body, Weight::Regular, TEXT_SECONDARY);
    } else {
        text::draw_centered(
            canvas, 0, w, hint_y,
            "Highlight an OS and press A / Enter to boot",
            Size::Body, Weight::Regular, TEXT_SECONDARY,
        );
    }

    // --- Footer glyph hint bar --------------------------------------------
    draw_footer(canvas);
    canvas.present();
}

fn draw_tile(canvas: &mut Canvas, x: i32, y: i32, tile: &Tile, focused: bool, is_default: bool) {
    let (x, y, tw, th) = if focused {
        (x - LIFT, y - LIFT, TILE_W + LIFT * 2, TILE_H + LIFT * 2)
    } else {
        (x, y, TILE_W, TILE_H)
    };

    // Surface — brighter when focused (the BP "lift").
    let surface = if focused { SURFACE_3.scale(135) } else { SURFACE_3 };
    canvas.round_rect(x, y, tw, th, 12, surface);

    // Border: accent glow when focused, hairline otherwise.
    if focused {
        canvas.round_border(x - 2, y - 2, tw + 4, th + 4, 14, 3, GLOW);
        canvas.round_border(x, y, tw, th, 12, 1, ACCENT);
    } else {
        canvas.round_border(x, y, tw, th, 12, 1, SURFACE_4);
    }

    // Logo, centered horizontally, upper third.
    let logo = tile.logo;
    let lx = x + (tw - logo.w as i32) / 2;
    let ly = y + 56;
    canvas.blit_bgra(lx, ly, logo.w, logo.h, logo.bgra);

    // Title + subtitle.
    let title_y = y + 280;
    let title_color = if focused { TEXT_WHITE } else { TEXT_BRIGHT };
    text::draw_centered(canvas, x, tw, title_y, tile.title, Size::H2, Weight::Bold, title_color);
    text::draw_centered(canvas, x, tw, title_y + 44, tile.subtitle, Size::Body, Weight::Regular, TEXT_SECONDARY);

    // "DEFAULT" badge / "NOT INSTALLED" warning.
    let badge_y = y + th - 44;
    if !tile.present {
        text::draw_centered(canvas, x, tw, badge_y, "NOT INSTALLED", Size::Body, Weight::Bold, DESTRUCTIVE);
    } else if is_default {
        // A real drawn dot + label, centered as a group (no font glyph needed).
        let label = "DEFAULT";
        let dot_r = 5;
        let gap = 10;
        let tw_label = text::measure(label, Size::Body, Weight::Bold) as i32;
        let group_w = dot_r * 2 + gap + tw_label;
        let gx = x + (tw - group_w) / 2;
        let cy = badge_y + text::line_height(Size::Body) as i32 / 2;
        canvas.fill_circle(gx + dot_r, cy, dot_r, ACCENT);
        text::draw(canvas, gx + dot_r * 2 + gap, badge_y, label, Size::Body, Weight::Bold, ACCENT);
    }
}

/// A/B/Y glyph hints, centered along the bottom.
fn draw_footer(canvas: &mut Canvas) {
    let w = canvas.w as i32;
    let y = canvas.h as i32 - 56;
    let items: [(&str, &str, Bgra); 3] = [
        ("A", "Boot", GLYPH_A),
        ("B", "Firmware Setup", GLYPH_B),
        ("Y", "Set Default", GLYPH_Y),
    ];

    // Measure total width to center the row.
    let glyph_d = 30;
    let pad = 10; // glyph -> label
    let gap = 36; // between items
    let mut total = 0;
    for (_, label, _) in items {
        total += glyph_d + pad + text::measure(label, Size::Body, Weight::Regular) as i32 + gap;
    }
    total -= gap;

    let mut cx = (w - total) / 2;
    for (glyph, label, color) in items {
        // Glyph chip (filled circle + centered letter).
        canvas.round_rect(cx, y, glyph_d, glyph_d, glyph_d / 2, color);
        text::draw_centered(canvas, cx, glyph_d, y + 4, glyph, Size::Body, Weight::Bold, TEXT_WHITE);
        cx += glyph_d + pad;
        text::draw(canvas, cx, y + 4, label, Size::Body, Weight::Regular, TEXT_BRIGHT);
        cx += text::measure(label, Size::Body, Weight::Regular) as i32 + gap;
    }
}

fn push_u8(s: &mut alloc::string::String, n: u8) {
    if n >= 100 {
        s.push((b'0' + n / 100) as char);
    }
    if n >= 10 {
        s.push((b'0' + (n / 10) % 10) as char);
    }
    s.push((b'0' + n % 10) as char);
}
