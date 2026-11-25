use crate::ffi::{lumen_text_free, lumen_text_line};
use std::ffi::CString;
use std::path::Path;
use std::ptr;

const CANVAS_PAD: i32 = 2;

pub struct Fonts {
    app_dir: String,
}

#[derive(Clone)]
pub struct Line {
    pub rgba: Vec<u8>,
    pub w: i32,
    pub h: i32,
    pub left: i32,
    pub baseline: i32,
    pub advance: i32,
}

#[derive(Clone, Copy)]
pub struct Shadow {
    pub drop: i32,
    pub blur: i32,
    pub strength: f32,
}

impl Fonts {
    pub fn new(app_dir: &str) -> Fonts {
        Fonts { app_dir: app_dir.to_string() }
    }

    fn file(&self, weight: &str) -> String {
        let heavy = weight.starts_with('6') || weight.starts_with('7');
        let places = [
            format!("{}/fonts/sf-{}.ttf", self.app_dir, weight),
            format!("{}/fonts/sf-{}.ttf", self.app_dir, if heavy { "600" } else { "500" }),
            format!("/usr/share/fonts/LG_Smart_UI-{}.ttf", if heavy { "SemiBold" } else { "Regular" }),
            "/usr/share/fonts/LG_Smart_UI-Regular.ttf".to_string(),
            format!("/usr/share/fonts/truetype/dejavu/DejaVuSans{}.ttf", if heavy { "-Bold" } else { "" }),
        ];
        places.into_iter().find(|path| Path::new(path).exists()).unwrap_or_default()
    }

    pub fn is_sf(&self, weight: &str) -> bool {
        self.file(weight).contains("/fonts/sf-")
    }

    pub fn line(&self, text: &str, px: i32, weight: &str, letter_spacing: f32, feature: &str) -> Option<Line> {
        let other_script = ["/usr/share/fonts/LG_Smart_UI_Devanagari.ttf", "/usr/share/fonts/truetype/lohit-devanagari/Lohit-Devanagari.ttf"].into_iter().find(|p| Path::new(p).exists()).unwrap_or("");
        let (font, other, feature, utf8) = (CString::new(self.file(weight)).ok()?, CString::new(other_script).ok()?, CString::new(feature).ok()?, CString::new(text).ok()?);
        let (mut pixels, mut w, mut h) = (ptr::null_mut(), 0, 0);
        if unsafe { lumen_text_line(font.as_ptr(), other.as_ptr(), px, letter_spacing, feature.as_ptr(), utf8.as_ptr(), &mut pixels, &mut w, &mut h) } == 0 || pixels.is_null() {
            return None;
        }
        let rgba = unsafe { std::slice::from_raw_parts(pixels, (w * h * 4) as usize) }.to_vec();
        unsafe { lumen_text_free(pixels) }
        Some(Line { rgba, w, h, left: CANVAS_PAD, baseline: px, advance: w - 2 * CANVAS_PAD - 2 })
    }
}

fn box_blurred(plane: &mut [f32], w: usize, h: usize, radius: usize) {
    if radius == 0 {
        return;
    }
    let span = (2 * radius + 1) as f32;
    let mut line = vec![0.0f32; w.max(h)];
    for (step, count, lines) in [(1, w, h), (w, h, w)] {
        for l in 0..lines {
            let start = if step == 1 { l * w } else { l };
            for i in 0..count {
                line[i] = plane[start + i * step];
            }
            let mut sum: f32 = line[..count.min(radius + 1)].iter().sum();
            for i in 0..count {
                plane[start + i * step] = sum / span;
                if i + radius + 1 < count {
                    sum += line[i + radius + 1];
                }
                if i >= radius {
                    sum -= line[i - radius];
                }
            }
        }
    }
}

pub fn inked(line: &Line, ink: [f32; 3], shadows: &[Shadow]) -> Line {
    let margin = shadows.iter().map(|s| s.blur * 2 + s.drop.abs()).max().unwrap_or(0);
    let (w, h) = ((line.w + 2 * margin) as usize, (line.h + 2 * margin) as usize);
    let (lw, lh, m) = (line.w as usize, line.h as usize, margin as usize);
    let mut shade = vec![0.0f32; w * h];
    for shadow in shadows {
        let mut plane = vec![0.0f32; w * h];
        for y in 0..lh {
            let to_y = y as i32 + margin + shadow.drop;
            if to_y < 0 || to_y >= h as i32 {
                continue;
            }
            for x in 0..lw {
                plane[to_y as usize * w + x + m] = line.rgba[(y * lw + x) * 4 + 3] as f32 / 255.0;
            }
        }
        for _ in 0..3 {
            box_blurred(&mut plane, w, h, (shadow.blur / 2) as usize);
        }
        for (total, cover) in shade.iter_mut().zip(&plane) {
            *total = *total + cover * shadow.strength * (1.0 - *total);
        }
    }
    let mut rgba = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let inside = x >= m && x < m + lw && y >= m && y < m + lh;
            let cover = if inside { line.rgba[((y - m) * lw + x - m) * 4 + 3] as f32 / 255.0 } else { 0.0 };
            let alpha = cover + shade[y * w + x] * (1.0 - cover);
            let out = &mut rgba[(y * w + x) * 4..(y * w + x) * 4 + 4];
            for c in 0..3 {
                out[c] = (ink[c] * cover * 255.0 + 0.5) as u8;
            }
            out[3] = (alpha * 255.0 + 0.5) as u8;
        }
    }
    Line { rgba, w: w as i32, h: h as i32, left: line.left + margin, baseline: line.baseline + margin, advance: line.advance }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dot() -> Line {
        let mut rgba = vec![0u8; 5 * 5 * 4];
        rgba[(2 * 5 + 2) * 4..(2 * 5 + 2) * 4 + 4].copy_from_slice(&[255; 4]);
        Line { rgba, w: 5, h: 5, left: 2, baseline: 4, advance: 1 }
    }

    #[test]
    fn ink_colours_the_letters_and_leaves_the_rest_clear() {
        let line = inked(&dot(), [1.0, 0.5, 0.0], &[]);
        assert_eq!((line.w, line.h, line.left, line.baseline), (5, 5, 2, 4));
        assert_eq!(&line.rgba[(2 * 5 + 2) * 4..(2 * 5 + 2) * 4 + 4], &[255, 128, 0, 255]);
        assert_eq!(&line.rgba[0..4], &[0, 0, 0, 0]);
    }

    #[test]
    fn a_shadow_grows_the_picture_keeps_the_baseline_in_place_and_falls_below_the_letters() {
        let line = inked(&dot(), [1.0; 3], &[Shadow { drop: 2, blur: 2, strength: 0.5 }]);
        assert_eq!((line.w, line.h, line.left, line.baseline), (17, 17, 8, 10));
        let alpha = |x: usize, y: usize| line.rgba[(y * 17 + x) * 4 + 3];
        assert_eq!(alpha(8, 8), 255);
        assert!(alpha(8, 10) > 0 && alpha(8, 10) < 128);
        assert!(alpha(8, 11) > alpha(8, 5));
        assert_eq!(line.rgba[(10 * 17 + 8) * 4], 0);
    }

    #[test]
    fn blurring_spreads_a_dot_without_losing_or_gaining_ink() {
        let mut plane = vec![0.0f32; 21 * 21];
        plane[10 * 21 + 10] = 1.0;
        box_blurred(&mut plane, 21, 21, 2);
        assert!((plane.iter().sum::<f32>() - 1.0).abs() < 1e-4);
        assert!((plane[10 * 21 + 10] - 0.04).abs() < 1e-4);
        assert_eq!(plane[10 * 21 + 13], 0.0);
    }
}
