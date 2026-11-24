use crate::draw::Decoded;

pub const PAIR_WIDTH: i32 = 956;
const BLUR_BAND_AT_1080: i32 = 520;
const BLUR_RADIUS: i32 = 3;
const NIGHT_KEEP: u32 = 141;
const TAP_ONE: u32 = 256;
const RB: u32 = 0x00FF_00FF;
const OPAQUE: u32 = 0xFF00_0000;

pub struct Placed<'a> {
    pub picture: &'a Decoded,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy)]
struct Area {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

struct Axis {
    start: Vec<usize>,
    count: Vec<usize>,
    weight: Vec<u32>,
    stride: usize,
    lo: usize,
    hi: usize,
}

fn axis(source: i32, scaled: i32, offset: i32, c0: i32, c1: i32) -> Axis {
    let s = source as f64 / scaled as f64;
    let stride = if s > 1.0 { s.ceil() as usize + 2 } else { 2 };
    let n = (c1 - c0) as usize;
    let mut a = Axis { start: vec![0; n], count: vec![0; n], weight: vec![0; n * stride], stride, lo: source as usize, hi: 0 };
    let mut exact = vec![0.0f64; stride];
    for i in 0..n {
        let pos = (c0 + i as i32 - offset) as f64;
        let (first, taps);
        if s > 1.0 {
            let from = pos * s;
            let to = (from + s).min(source as f64);
            first = from.floor().max(0.0) as i32;
            let last = ((to.ceil() as i32) - 1).min(source - 1);
            taps = (last - first + 1).max(1) as usize;
            for (k, e) in exact.iter_mut().enumerate().take(taps) {
                let lo = ((first + k as i32) as f64).max(from);
                let hi = ((first + k as i32 + 1) as f64).min(to);
                *e = (hi - lo).max(0.0) / (to - from);
            }
        } else {
            let centre = (pos + 0.5) * s - 0.5;
            let base = centre.floor() as i32;
            let f = centre - base as f64;
            if base < 0 {
                (first, taps, exact[0]) = (0, 1, 1.0);
            } else if base >= source - 1 {
                (first, taps, exact[0]) = (source - 1, 1, 1.0);
            } else {
                (first, taps) = (base, 2);
                exact[0] = 1.0 - f;
                exact[1] = f;
            }
        }
        let w = &mut a.weight[i * stride..i * stride + taps];
        let mut biggest = 0;
        for k in 0..taps {
            w[k] = (exact[k] * TAP_ONE as f64 + 0.5) as u32;
            if w[k] > w[biggest] {
                biggest = k;
            }
        }
        let sum: u32 = w.iter().sum();
        w[biggest] = (w[biggest] + TAP_ONE).saturating_sub(sum);
        a.start[i] = first as usize;
        a.count[i] = taps;
        a.lo = a.lo.min(first as usize);
        a.hi = a.hi.max(first as usize + taps - 1);
    }
    a
}

fn pixel(p: &Decoded, x: usize, y: usize) -> u32 {
    let at = (y * p.w as usize + x) * 4;
    u32::from_le_bytes([p.rgba[at], p.rgba[at + 1], p.rgba[at + 2], 255])
}

fn mixed(values: impl Iterator<Item = (u32, u32)>) -> u32 {
    let (mut rb, mut g) = (0u32, 0u32);
    for (v, w) in values {
        rb += (v & RB) * w;
        g += (v >> 8 & 255) * w;
    }
    ((rb + 0x0080_0080) >> 8 & RB) | ((g + 128) >> 8) << 8
}

fn resample_into(frame: &mut [u32], frame_w: i32, src: &Decoded, (dw, dh, dx, dy): (i32, i32, i32, i32), area: Area) {
    let (c0, c1) = (dx.max(area.x), (dx + dw).min(area.x + area.w));
    let (r0, r1) = (dy.max(area.y), (dy + dh).min(area.y + area.h));
    if c1 <= c0 || r1 <= r0 || src.w <= 0 || src.h <= 0 {
        return;
    }
    let cw = (c1 - c0) as usize;
    let (ax, ay) = (axis(src.w, dw, dx, c0, c1), axis(src.h, dh, dy, r0, r1));
    let rows = ay.hi - ay.lo + 1;
    let mut mid = vec![0u32; rows * cw];
    for r in 0..rows {
        let y = ay.lo + r;
        for x in 0..cw {
            let w = &ax.weight[x * ax.stride..];
            mid[r * cw + x] = mixed((0..ax.count[x]).map(|k| (pixel(src, ax.start[x] + k, y), w[k])));
        }
    }
    for y in 0..(r1 - r0) as usize {
        let w = &ay.weight[y * ay.stride..];
        let first = ay.start[y] - ay.lo;
        let out = &mut frame[(r0 as usize + y) * frame_w as usize + c0 as usize..][..cw];
        for (x, o) in out.iter_mut().enumerate() {
            *o = OPAQUE | mixed((0..ay.count[y]).map(|k| (mid[(first + k) * cw + x], w[k])));
        }
    }
}

fn box_blur_line(line: &mut [u32], spare: &mut [u32], r: i32) {
    let n = line.len() as i32;
    let recip = (65536 + r as u32) / (2 * r as u32 + 1);
    let at = |i: i32| line[i.clamp(0, n - 1) as usize];
    let (mut rb, mut g) = (0u32, 0u32);
    for k in -r..=r {
        rb += at(k) & RB;
        g += at(k) >> 8 & 255;
    }
    for i in 0..n {
        spare[i as usize] = ((rb >> 16) * recip + 32768) >> 16 << 16 | ((g * recip + 32768) >> 16) << 8 | (((rb & 0xFFFF) * recip + 32768) >> 16);
        let (add, sub) = (at(i + r + 1), at(i - r));
        rb = rb + (add & RB) - (sub & RB);
        g = g + (add >> 8 & 255) - (sub >> 8 & 255);
    }
    line.copy_from_slice(spare);
}

fn blur_mask(t: f64) -> u32 {
    let m = if t < 0.4 { t / 0.4 * 0.6 } else if t < 0.7 { 0.6 + (t - 0.4) / 0.3 * 0.4 } else { 1.0 };
    (m * 256.0 + 0.5) as u32
}

fn blur_bottom(frame: &mut [u32], frame_w: usize, frame_h: usize, x0: usize, w: usize) {
    let band = (BLUR_BAND_AT_1080 as usize * frame_h / 1080).min(frame_h);
    let y0 = frame_h - band;
    let mut soft: Vec<u32> = (0..band).flat_map(|y| frame[(y0 + y) * frame_w + x0..][..w].to_vec()).collect();
    let mut spare = vec![0u32; w.max(band)];
    for _ in 0..2 {
        for row in soft.chunks_exact_mut(w) {
            box_blur_line(row, &mut spare[..w], BLUR_RADIUS);
        }
        let mut column = vec![0u32; band];
        for x in 0..w {
            for y in 0..band {
                column[y] = soft[y * w + x];
            }
            box_blur_line(&mut column, &mut spare[..band], BLUR_RADIUS);
            for y in 0..band {
                soft[y * w + x] = column[y];
            }
        }
    }
    for y in 0..band {
        let m = blur_mask((y as f64 + 0.5) / band as f64) as i32;
        let sharp = &mut frame[(y0 + y) * frame_w + x0..][..w];
        for (s, b) in sharp.iter_mut().zip(&soft[y * w..(y + 1) * w]) {
            let channel = |shift: u32| {
                let (a, c) = ((*s >> shift & 255) as i32, (*b >> shift & 255) as i32);
                ((a + ((c - a) * m >> 8)) as u32) << shift
            };
            *s = OPAQUE | channel(16) | channel(8) | channel(0);
        }
    }
}

fn place_one(frame: &mut [u32], frame_w: i32, one: &Placed, area: Area, fit_width: bool) {
    let p = one.picture;
    let (across, down) = (area.w as f32 / p.w as f32, area.h as f32 / p.h as f32);
    let scale = if fit_width { across } else { across.max(down) };
    let (dw, dh) = ((p.w as f32 * scale + 0.5) as i32, (p.h as f32 * scale + 0.5) as i32);
    let at = (area.x + ((area.w - dw) as f32 * one.x) as i32, area.y + ((area.h - dh) as f32 * one.y) as i32);
    resample_into(frame, frame_w, p, (dw, dh, at.0, at.1), area);
}

pub fn frame(photos: &[Placed], w: i32, h: i32, night: bool) -> Option<Decoded> {
    if photos.is_empty() || photos.len() > 2 || w <= 0 || h <= 0 || photos.iter().any(|p| p.picture.w <= 0 || p.picture.h <= 0) {
        return None;
    }
    let (fw, fh) = (w as usize, h as usize);
    let mut frame = vec![OPAQUE; fw * fh];
    if let [one] = photos {
        place_one(&mut frame, w, one, Area { x: 0, y: 0, w, h }, false);
        blur_bottom(&mut frame, fw, fh, 0, fw);
    } else {
        let side = PAIR_WIDTH * w / 1920;
        for (i, one) in photos.iter().enumerate() {
            let x = if i == 0 { 0 } else { w - side };
            place_one(&mut frame, w, &Placed { picture: one.picture, x: 0.0, y: one.y }, Area { x, y: 0, w: side, h }, true);
            blur_bottom(&mut frame, fw, fh, x as usize, side as usize);
        }
    }
    if night {
        for v in frame.iter_mut() {
            let keep = |shift: u32| ((*v >> shift & 255) * NIGHT_KEEP >> 8) << shift;
            *v = OPAQUE | keep(16) | keep(8) | keep(0);
        }
    }
    Some(Decoded { rgba: frame.iter().flat_map(|v| v.to_le_bytes()).collect(), w, h })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(w: i32, h: i32, rgb: [u8; 3]) -> Decoded {
        Decoded { rgba: (0..w * h).flat_map(|_| [rgb[0], rgb[1], rgb[2], 255]).collect(), w, h }
    }

    fn at(d: &Decoded, x: i32, y: i32) -> [u8; 4] {
        let i = ((y * d.w + x) * 4) as usize;
        [d.rgba[i], d.rgba[i + 1], d.rgba[i + 2], d.rgba[i + 3]]
    }

    #[test]
    fn a_portrait_pair_fills_both_sides_with_black_between_and_keeps_its_colours() {
        let (red, blue) = (flat(30, 60, [200, 10, 10]), flat(30, 60, [10, 10, 200]));
        let frame = frame(&[Placed { picture: &red, x: 0.0, y: 0.5 }, Placed { picture: &blue, x: 1.0, y: 0.5 }], 192, 108, false).unwrap();
        assert_eq!(at(&frame, 5, 54), [200, 10, 10, 255]);
        assert_eq!(at(&frame, 186, 54), [10, 10, 200, 255]);
        assert_eq!(at(&frame, 96, 54), [0, 0, 0, 255]);
    }

    #[test]
    fn a_wide_photo_covers_the_screen_shifted_to_where_it_is_placed_and_night_dims_it() {
        let mut wide = flat(40, 10, [100, 100, 100]);
        for y in 0..10 {
            for x in 0..20 {
                wide.rgba[((y * 40 + x) * 4) as usize] = 250;
            }
        }
        let left = frame(&[Placed { picture: &wide, x: 0.0, y: 0.5 }], 16, 9, false).unwrap();
        assert_eq!(at(&left, 15, 0)[0], 250);
        let right = frame(&[Placed { picture: &wide, x: 1.0, y: 0.5 }], 16, 9, true).unwrap();
        assert_eq!(at(&right, 0, 0), [55, 55, 55, 255]);
    }
}
