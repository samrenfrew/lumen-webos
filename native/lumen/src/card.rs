use crate::draw::{decoded, Decoded};
use crate::livetv;
use crate::text::Fonts;

pub const CHANNEL_DARK: [f32; 3] = [0.133, 0.149, 0.227];

pub struct Card {
    pub w: usize,
    pub h: usize,
    pub radius: f32,
    pub logo_box: (f32, f32, f32, f32),
    pub name_px: i32,
    pub name_box: (f32, f32, f32, f32),
}

pub const TILE_CARD: Card = Card { w: 240, h: 144, radius: 0.0, logo_box: (24.0, 16.0, 192.0, 82.0), name_px: 32, name_box: (19.2, 0.0, 201.6, 132.0) };
pub const GUIDE_CARD: Card = Card { w: 132, h: 80, radius: 16.0, logo_box: (13.2, 14.4, 105.6, 51.2), name_px: 18, name_box: (8.0, 0.0, 116.0, 80.0) };
pub const GRID_CARD: Card = Card { w: 84, h: 52, radius: 12.0, logo_box: (8.4, 8.8, 67.2, 34.3), name_px: 13, name_box: (4.0, 0.0, 76.0, 52.0) };

pub fn colour_of(name: &str) -> [f32; 3] {
    livetv::channel_color(name).unwrap_or(CHANNEL_DARK)
}

fn swept(bg: [f32; 3], card: &Card, x: f32, y: f32) -> [f32; 3] {
    let (dx, dy) = (0.342f32, 0.940f32);
    let span = card.w as f32 * dx + card.h as f32 * dy;
    let t = (((x - card.w as f32 / 2.0) * dx + (y - card.h as f32 / 2.0) * dy) / span + 0.5).clamp(0.0, 1.0);
    let (light, dark) = (0.1 * (1.0 - t), 0.28 * t);
    let a = light + dark;
    bg.map(|c| c * (1.0 - a) + light)
}

fn corner_cover(card: &Card, x: f32, y: f32) -> f32 {
    if card.radius <= 0.0 {
        return 1.0;
    }
    let (hw, hh) = (card.w as f32 / 2.0, card.h as f32 / 2.0);
    let q = ((x - hw).abs() - (hw - card.radius), (y - hh).abs() - (hh - card.radius));
    let d = (q.0.max(0.0).powi(2) + q.1.max(0.0).powi(2)).sqrt() + q.0.max(q.1).min(0.0) - card.radius;
    (0.5 - d).clamp(0.0, 1.0)
}

const STRETCHED: f32 = 1.2;

fn crisp(ink: [f32; 4]) -> [f32; 4] {
    let t = ((ink[3] * 255.0 - 48.0) / 160.0).clamp(0.0, 1.0);
    let a = t * t * (3.0 - 2.0 * t);
    let k = if ink[3] > 0.0 { a / ink[3] } else { 0.0 };
    [ink[0] * k, ink[1] * k, ink[2] * k, a]
}

fn sample(src: &Decoded, u: f32, v: f32) -> [f32; 4] {
    let (x, y) = ((u - 0.5).clamp(0.0, (src.w - 1) as f32), (v - 0.5).clamp(0.0, (src.h - 1) as f32));
    let (x0, y0) = (x.floor() as i32, y.floor() as i32);
    let (x1, y1) = ((x0 + 1).min(src.w - 1), (y0 + 1).min(src.h - 1));
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let at = |px: i32, py: i32| {
        let p = &src.rgba[((py * src.w + px) * 4) as usize..][..4];
        let a = p[3] as f32 / 255.0;
        [p[0] as f32 / 255.0 * a, p[1] as f32 / 255.0 * a, p[2] as f32 / 255.0 * a, a]
    };
    let (a, b, c, d) = (at(x0, y0), at(x1, y0), at(x0, y1), at(x1, y1));
    [0, 1, 2, 3].map(|i| (a[i] * (1.0 - fx) + b[i] * fx) * (1.0 - fy) + (c[i] * (1.0 - fx) + d[i] * fx) * fy)
}

fn wordmark_lines(fonts: &Fonts, name: &str, px: i32, max_w: f32) -> Vec<crate::text::Line> {
    let line = |text: &str| fonts.line(text, px, "700", -0.015 * px as f32, "");
    let mut out = Vec::new();
    let mut current = String::new();
    for word in name.split_whitespace() {
        let tried = if current.is_empty() { word.to_string() } else { format!("{current} {word}") };
        if !current.is_empty() && line(&tried).is_some_and(|l| l.advance as f32 > max_w) {
            out.extend(line(&current));
            current = word.to_string();
        } else {
            current = tried;
        }
    }
    out.extend(line(&current));
    out
}

pub fn channel_card(fonts: &Fonts, name: &str, logo: Option<&str>, card: &Card) -> Decoded {
    let bg = colour_of(name);
    let mut rgba = vec![0u8; card.w * card.h * 4];
    let mut put = |x: usize, y: usize, rgb: [f32; 3]| {
        let a = corner_cover(card, x as f32 + 0.5, y as f32 + 0.5);
        let at = (y * card.w + x) * 4;
        for c in 0..3 {
            rgba[at + c] = (rgb[c].clamp(0.0, 1.0) * a * 255.0) as u8;
        }
        rgba[at + 3] = (a * 255.0) as u8;
    };
    let picture = logo.and_then(decoded);
    let mut ink = vec![[0.0f32; 4]; card.w * card.h];
    match &picture {
        Some(src) => {
            let (bx, by, bw, bh) = card.logo_box;
            let scale = (bw / src.w as f32).min(bh / src.h as f32);
            let (w, h) = (src.w as f32 * scale, src.h as f32 * scale);
            let (left, top) = (bx + (bw - w) / 2.0, by + (bh - h) / 2.0);
            let stretched = scale > STRETCHED;
            for y in (top.floor() as usize)..((top + h).ceil() as usize).min(card.h) {
                for x in (left.floor() as usize)..((left + w).ceil() as usize).min(card.w) {
                    let soft = sample(src, (x as f32 + 0.5 - left) / scale, (y as f32 + 0.5 - top) / scale);
                    ink[y * card.w + x] = if stretched { crisp(soft) } else { soft };
                }
            }
        }
        None => {
            let (bx, by, bw, bh) = card.name_box;
            let lines = wordmark_lines(fonts, name, card.name_px, bw);
            let step = card.name_px as f32 * 1.1;
            let first = by + bh / 2.0 - step * (lines.len() as f32 - 1.0) / 2.0 + 0.35 * card.name_px as f32;
            for (k, line) in lines.iter().enumerate() {
                let x0 = (bx + bw / 2.0 - line.advance as f32 / 2.0) as i32 - line.left;
                let y0 = (first + k as f32 * step) as i32 - line.baseline;
                for y in 0..line.h {
                    for x in 0..line.w {
                        let (tx, ty) = (x + x0, y + y0);
                        if (tx as f32) < bx || (tx as f32) >= bx + bw || ty < 0 || ty as usize >= card.h {
                            continue;
                        }
                        let a = line.rgba[((y * line.w + x) * 4 + 3) as usize] as f32 / 255.0;
                        let cell = &mut ink[ty as usize * card.w + tx as usize];
                        *cell = [a.max(cell[0]), a.max(cell[1]), a.max(cell[2]), a.max(cell[3])];
                    }
                }
            }
        }
    }
    for y in 0..card.h {
        for x in 0..card.w {
            let under = swept(bg, card, x as f32 + 0.5, y as f32 + 0.5);
            let over = ink[y * card.w + x];
            put(x, y, [0, 1, 2].map(|c| under[c] * (1.0 - over[3]) + over[c]));
        }
    }
    Decoded { rgba, w: card.w as i32, h: card.h as i32 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stretched_logos_soft_edge_is_pushed_towards_solid_or_clear() {
        let faint = crisp([0.15, 0.15, 0.15, 0.15]);
        let strong = crisp([0.85, 0.85, 0.85, 0.85]);
        assert_eq!(faint[3], 0.0);
        assert_eq!(strong[3], 1.0);
        assert!((strong[0] - strong[3]).abs() < 1e-5, "white stays white");
        let middle = crisp([0.5, 0.5, 0.5, 0.5]);
        assert!((middle[3] - 0.5).abs() < 0.02, "the very middle of the edge stays soft so it does not jag");
    }
}
