//! Big Picture design tokens, ported verbatim from
//! `SteamBPClone/styles/tokens.css` into framebuffer colors.
//!
//! Colors are stored as `Bgra` because the Steam Deck GOP framebuffer is BGRA.

/// A single framebuffer pixel (matches GOP `BltPixel` channel order: B, G, R, _).
#[derive(Clone, Copy)]
pub struct Bgra {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

impl Bgra {
    /// Build from a `0xRRGGBB` literal so values read like the CSS tokens.
    pub const fn hex(rgb: u32) -> Self {
        Bgra { r: (rgb >> 16) as u8, g: (rgb >> 8) as u8, b: rgb as u8 }
    }

    /// Linear blend toward `other` by `t` in 0..=255 (255 == fully `other`).
    pub fn lerp(self, other: Bgra, t: u8) -> Bgra {
        let mix = |a: u8, b: u8| (((a as u16) * (255 - t as u16) + (b as u16) * (t as u16)) / 255) as u8;
        Bgra { b: mix(self.b, other.b), g: mix(self.g, other.g), r: mix(self.r, other.r) }
    }

    /// Multiply brightness by `pct/100` (clamped) — used for the focus "lift".
    pub fn scale(self, pct: u16) -> Bgra {
        let m = |c: u8| ((c as u16 * pct) / 100).min(255) as u8;
        Bgra { b: m(self.b), g: m(self.g), r: m(self.r) }
    }
}

// --- Surfaces (the dark blue-black ramp) -----------------------------------
pub const BG_TOP: Bgra = Bgra::hex(0x212328); // body gradient top
pub const BG_BOTTOM: Bgra = Bgra::hex(0x191A1E); // body gradient bottom
pub const SURFACE_3: Bgra = Bgra::hex(0x23262E); // standard card / tile
pub const SURFACE_4: Bgra = Bgra::hex(0x3D4450); // raised edge / hairline

// --- Text ------------------------------------------------------------------
pub const TEXT_SECONDARY: Bgra = Bgra::hex(0x8B929A);
pub const TEXT_BRIGHT: Bgra = Bgra::hex(0xDCDEDF);
pub const TEXT_WHITE: Bgra = Bgra::hex(0xFFFFFF);
pub const TEXT_DARK: Bgra = Bgra::hex(0x23262E); // text on the inverted (white) focus surface

// --- Accents ---------------------------------------------------------------
pub const ACCENT: Bgra = Bgra::hex(0x1A9FFF); // THE Steam blue
pub const GLOW: Bgra = Bgra::hex(0x199FFF); // focus glow (active-tab underline)
pub const DESTRUCTIVE: Bgra = Bgra::hex(0xDE3618);

// --- Controller glyph chip colors (A green / B red / X blue / Y yellow) ----
pub const GLYPH_A: Bgra = Bgra::hex(0x5BA32B);
pub const GLYPH_B: Bgra = Bgra::hex(0xC0392B);
pub const GLYPH_X: Bgra = Bgra::hex(0x2D7FC1);
pub const GLYPH_Y: Bgra = Bgra::hex(0xD4A017);
