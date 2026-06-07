//! Anti-aliased text drawing using pre-rasterized Noto Sans Mono glyphs.
//!
//! Each glyph is a coverage map (0..=255); we blend the text color over the
//! canvas using that coverage as alpha, matching the soft BP type look.

use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};

use crate::gop::Canvas;
use crate::theme::Bgra;

/// UI text sizes, mapped onto the bitmap raster heights the crate ships.
#[derive(Clone, Copy)]
pub enum Size {
    Body,   // 20px — meta / hints
    H3,     // 24px — sub labels
    H2,     // 32px — tile titles / headings (largest bitmap size available)
}

impl Size {
    fn raster(self) -> RasterHeight {
        match self {
            Size::Body => RasterHeight::Size20,
            Size::H3 => RasterHeight::Size24,
            Size::H2 => RasterHeight::Size32,
        }
    }
    fn height(self) -> usize {
        self.raster().val()
    }
}

#[derive(Clone, Copy)]
pub enum Weight {
    Regular,
    Bold,
}

impl Weight {
    fn fw(self) -> FontWeight {
        match self {
            Weight::Regular => FontWeight::Regular,
            Weight::Bold => FontWeight::Bold,
        }
    }
}

/// Advance width of a single char at a given size/weight (monospace, so constant).
fn char_width(size: Size, weight: Weight) -> usize {
    get_raster_width(weight.fw(), size.raster())
}

/// Total pixel width of `s`.
pub fn measure(s: &str, size: Size, weight: Weight) -> usize {
    s.chars().count() * char_width(size, weight)
}

pub fn line_height(size: Size) -> usize {
    size.height()
}

/// Draw `s` with its top-left at (x,y). Returns the x advance.
pub fn draw(canvas: &mut Canvas, x: i32, y: i32, s: &str, size: Size, weight: Weight, color: Bgra) -> i32 {
    let mut pen = x;
    let cw = char_width(size, weight) as i32;
    for ch in s.chars() {
        if let Some(raster) = get_raster(ch, weight.fw(), size.raster()) {
            for (row, line) in raster.raster().iter().enumerate() {
                for (col, &cov) in line.iter().enumerate() {
                    if cov == 0 {
                        continue;
                    }
                    let (px_, py_) = (pen + col as i32, y + row as i32);
                    if px_ >= 0 && py_ >= 0 {
                        canvas.blend(px_ as usize, py_ as usize, color, cov);
                    }
                }
            }
        }
        pen += cw;
    }
    pen
}

/// Draw `s` horizontally centered within [x, x+w), top at `y`.
pub fn draw_centered(canvas: &mut Canvas, x: i32, w: i32, y: i32, s: &str, size: Size, weight: Weight, color: Bgra) {
    let tw = measure(s, size, weight) as i32;
    draw(canvas, x + (w - tw) / 2, y, s, size, weight, color);
}
