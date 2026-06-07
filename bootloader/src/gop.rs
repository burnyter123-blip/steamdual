//! Software framebuffer + drawing primitives.
//!
//! We render into an owned back-buffer of [`BltPixel`] and present the whole
//! frame with a single `Blt(BufferToVideo)`. That keeps us independent of the
//! GOP's stride / pixel-format details and gives us flicker-free updates.

use alloc::vec;
use alloc::vec::Vec;
use uefi::boot;
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};

use crate::theme::Bgra;

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    buf: Vec<BltPixel>,
    gop: uefi::boot::ScopedProtocol<GraphicsOutput>,
}

#[inline]
fn px(c: Bgra) -> BltPixel {
    BltPixel::new(c.r, c.g, c.b)
}

impl Canvas {
    /// Open the GOP, select the largest available mode, and allocate the back-buffer.
    pub fn new() -> uefi::Result<Canvas> {
        let handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
        let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(handle)?;

        // Prefer the Deck's native 1280x800; otherwise take the largest mode.
        let mut best: Option<(usize, (usize, usize))> = None;
        for (i, mode) in gop.modes().enumerate() {
            let (w, h) = mode.info().resolution();
            if w == 1280 && h == 800 {
                best = Some((i, (w, h)));
                break;
            }
            let area = w * h;
            if best.map_or(true, |(_, (bw, bh))| area > bw * bh) {
                best = Some((i, (w, h)));
            }
        }
        if let Some((i, _)) = best {
            if let Some(mode) = gop.modes().nth(i) {
                let _ = gop.set_mode(&mode);
            }
        }

        let (w, h) = gop.current_mode_info().resolution();
        let buf = vec![BltPixel::new(0, 0, 0); w * h];
        Ok(Canvas { w, h, buf, gop })
    }

    #[inline]
    pub fn put(&mut self, x: usize, y: usize, c: Bgra) {
        if x < self.w && y < self.h {
            self.buf[y * self.w + x] = px(c);
        }
    }

    /// Alpha blend `c` over the existing pixel. `a` is 0..=255.
    #[inline]
    pub fn blend(&mut self, x: usize, y: usize, c: Bgra, a: u8) {
        if x >= self.w || y >= self.h {
            return;
        }
        let dst = self.buf[y * self.w + x];
        let mix = |s: u8, d: u8| ((s as u16 * a as u16 + d as u16 * (255 - a as u16)) / 255) as u8;
        self.buf[y * self.w + x] = BltPixel::new(mix(c.r, dst.red), mix(c.g, dst.green), mix(c.b, dst.blue));
    }

    /// Composite a **premultiplied** BGRA source pixel: dst = src + dst*(1-a).
    #[inline]
    fn over_premul(&mut self, x: usize, y: usize, b: u8, g: u8, r: u8, a: u8) {
        if x >= self.w || y >= self.h {
            return;
        }
        let dst = self.buf[y * self.w + x];
        let inv = 255 - a as u16;
        let add = |s: u8, d: u8| (s as u16 + (d as u16 * inv) / 255).min(255) as u8;
        self.buf[y * self.w + x] = BltPixel::new(add(r, dst.red), add(g, dst.green), add(b, dst.blue));
    }

    pub fn clear(&mut self, c: Bgra) {
        let p = px(c);
        for v in self.buf.iter_mut() {
            *v = p;
        }
    }

    /// Vertical gradient fill across the whole canvas (BP body backdrop).
    pub fn gradient_v(&mut self, top: Bgra, bottom: Bgra) {
        for y in 0..self.h {
            let t = ((y * 255) / self.h.max(1)) as u8;
            let c = px(top.lerp(bottom, t));
            let row = &mut self.buf[y * self.w..y * self.w + self.w];
            for v in row.iter_mut() {
                *v = c;
            }
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Bgra) {
        for yy in y.max(0)..(y + h).min(self.h as i32) {
            for xx in x.max(0)..(x + w).min(self.w as i32) {
                self.put(xx as usize, yy as usize, c);
            }
        }
    }

    /// Rounded-rectangle fill with cheap 1px corner anti-aliasing.
    pub fn round_rect(&mut self, x: i32, y: i32, w: i32, h: i32, r: i32, c: Bgra) {
        let r = r.min(w / 2).min(h / 2).max(0);
        for yy in 0..h {
            for xx in 0..w {
                let cov = corner_coverage(xx, yy, w, h, r);
                if cov == 0 {
                    continue;
                }
                let (px_, py_) = (x + xx, y + yy);
                if px_ < 0 || py_ < 0 {
                    continue;
                }
                if cov >= 255 {
                    self.put(px_ as usize, py_ as usize, c);
                } else {
                    self.blend(px_ as usize, py_ as usize, c, cov);
                }
            }
        }
    }

    /// 1px rounded border (drawn as outer round_rect minus inner).
    pub fn round_border(&mut self, x: i32, y: i32, w: i32, h: i32, r: i32, t: i32, c: Bgra) {
        for yy in 0..h {
            for xx in 0..w {
                let outer = corner_coverage(xx, yy, w, h, r);
                if outer == 0 {
                    continue;
                }
                let inner = corner_coverage(xx - t, yy - t, w - 2 * t, h - 2 * t, (r - t).max(0));
                let cov = outer.saturating_sub(inner);
                if cov == 0 {
                    continue;
                }
                let (px_, py_) = (x + xx, y + yy);
                if px_ >= 0 && py_ >= 0 {
                    self.blend(px_ as usize, py_ as usize, c, cov);
                }
            }
        }
    }

    /// Filled, anti-aliased circle centered at (cx, cy) with radius r.
    pub fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, c: Bgra) {
        self.round_rect(cx - r, cy - r, r * 2, r * 2, r, c);
    }

    /// Blit a premultiplied BGRA buffer at (x,y).
    pub fn blit_bgra(&mut self, x: i32, y: i32, w: usize, h: usize, bgra: &[u8]) {
        for row in 0..h {
            for col in 0..w {
                let i = (row * w + col) * 4;
                let (b, g, r, a) = (bgra[i], bgra[i + 1], bgra[i + 2], bgra[i + 3]);
                if a == 0 {
                    continue;
                }
                let (px_, py_) = (x + col as i32, y + row as i32);
                if px_ >= 0 && py_ >= 0 {
                    self.over_premul(px_ as usize, py_ as usize, b, g, r, a);
                }
            }
        }
    }

    /// Push the back-buffer to the screen.
    pub fn present(&mut self) {
        let (w, h) = (self.w, self.h);
        let _ = self.gop.blt(BltOp::BufferToVideo {
            buffer: &self.buf,
            src: BltRegion::Full,
            dest: (0, 0),
            dims: (w, h),
        });
    }
}

/// Coverage (0..=255) of point (xx,yy) inside a rounded rect of size w×h, radius r.
/// 255 = solidly inside, 0 = outside, intermediate = on a corner's AA edge.
#[inline]
fn corner_coverage(xx: i32, yy: i32, w: i32, h: i32, r: i32) -> u8 {
    if xx < 0 || yy < 0 || xx >= w || yy >= h {
        return 0;
    }
    if r <= 0 {
        return 255;
    }
    // Distance from the nearest corner's circle center, only inside corner boxes.
    let cx = if xx < r {
        r
    } else if xx >= w - r {
        w - 1 - r
    } else {
        return 255;
    };
    let cy = if yy < r {
        r
    } else if yy >= h - r {
        h - 1 - r
    } else {
        return 255;
    };
    let (dx, dy) = ((xx - cx) as f32, (yy - cy) as f32);
    let d = libm::sqrtf(dx * dx + dy * dy);
    let edge = r as f32;
    if d <= edge - 1.0 {
        255
    } else if d >= edge {
        0
    } else {
        ((edge - d) * 255.0) as u8
    }
}
