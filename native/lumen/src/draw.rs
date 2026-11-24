use crate::ffi::*;
use crate::gfx::{Gfx, Program, Tex, DESIGN_H, DESIGN_W};
use crate::look::Look;
use crate::pictures::Pictures;
use crate::shaders;
use crate::text::{inked, Fonts, Line, Shadow};
use std::collections::HashMap;
use std::ffi::CString;

const PICTURE_BUDGET_BYTES: usize = 28 * 1024 * 1024;

pub struct Programs {
    pub copy: Program,
    pub blur: Program,
    pub backdrop: Program,
    pub lens: Program,
    pub gloss: Program,
    pub bar_middle: Program,
    pub bar_rim: Program,
    pub shine: Program,
    pub tile: Program,
    pub picture: Program,
    pub glass: Program,
    pub ink: Program,
    pub dim: Program,
    pub fill: Program,
    pub spin: Program,
}

#[derive(Clone, Copy)]
pub struct Style {
    pub px: i32,
    pub weight: &'static str,
    pub spacing_em: f32,
    pub feature: &'static str,
    pub ink: [f32; 3],
    pub shadows: &'static [Shadow],
}

#[derive(Clone, Copy)]
pub struct Text {
    pub picture: Tex,
    pub left: f32,
    pub baseline: f32,
    pub advance: f32,
}

impl Text {
    pub fn draw(&self, d: &Draw, x: f32, baseline: f32, alpha: f32) {
        d.picture(self.picture, x - self.left, baseline - self.baseline, self.picture.w as f32, self.picture.h as f32, alpha);
    }

    pub fn draw_scaled(&self, d: &Draw, x: f32, baseline: f32, scale: f32, alpha: f32) {
        d.picture(self.picture, x - self.left * scale, baseline - self.baseline * scale, self.picture.w as f32 * scale, self.picture.h as f32 * scale, alpha);
    }
}

#[derive(Clone, Copy)]
pub struct Glass {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub radius: f32,
    pub blur: f32,
    pub sheen: f32,
    pub rim: f32,
    pub alpha: f32,
}

#[derive(Clone, Copy)]
pub struct Plate {
    pub h: f32,
    pub radius: f32,
    pub fill: [f32; 4],
    pub rim: f32,
    pub shadow: [f32; 3],
    pub ring: f32,
}

pub struct Decoded {
    pub rgba: Vec<u8>,
    pub w: i32,
    pub h: i32,
}

const SPIN_MS: u32 = 900;

pub struct Draw {
    pub gfx: Gfx,
    pub p: Programs,
    pub pictures: Pictures<Tex>,
    pub look: Look,
    pub liquid: bool,
    pub fonts: Fonts,
    pub app_dir: String,
    pub glass_backdrop: Tex,
    pub veil_backdrop: Tex,
    measures: HashMap<String, (f32, f32, f32)>,
    fits: HashMap<String, String>,
}

pub fn decoded(path: &str) -> Option<Decoded> {
    let c_path = CString::new(path).ok()?;
    let loaded = unsafe { IMG_Load(c_path.as_ptr()) };
    rgba_of(loaded)
}

pub fn decoded_bytes(bytes: &[u8]) -> Option<Decoded> {
    let reader = unsafe { SDL_RWFromConstMem(bytes.as_ptr().cast(), bytes.len() as i32) };
    if reader.is_null() {
        return None;
    }
    rgba_of(unsafe { IMG_Load_RW(reader, 1) })
}

fn rgba_of(loaded: *mut SDL_Surface) -> Option<Decoded> {
    unsafe {
        if loaded.is_null() {
            return None;
        }
        let surface = SDL_ConvertSurfaceFormat(loaded, SDL_PIXELFORMAT_RGBA32, 0);
        SDL_FreeSurface(loaded);
        if surface.is_null() {
            return None;
        }
        let (w, h, pitch) = ((*surface).w as usize, (*surface).h as usize, (*surface).pitch as usize);
        let mut rgba = vec![0u8; w * h * 4];
        for (y, row) in rgba.chunks_exact_mut(w * 4).enumerate() {
            std::ptr::copy_nonoverlapping((*surface).pixels.cast::<u8>().add(y * pitch), row.as_mut_ptr(), w * 4);
        }
        SDL_FreeSurface(surface);
        Some(Decoded { rgba, w: w as i32, h: h as i32 })
    }
}

impl Draw {
    pub fn new(screen_w: i32, screen_h: i32, app_dir: &str, look: Look) -> Draw {
        Draw {
            gfx: Gfx::new(screen_w, screen_h),
            p: Programs {
                copy: Program::new("copy", shaders::COPY_FRAGMENT),
                blur: Program::new("blur", shaders::BLUR_FRAGMENT),
                backdrop: Program::new("backdrop", shaders::BACKDROP_FRAGMENT),
                lens: Program::new("lens", shaders::LENS_FRAGMENT),
                gloss: Program::new("gloss", shaders::GLOSS_FRAGMENT),
                bar_middle: Program::new("bar middle", shaders::BAR_MIDDLE_FRAGMENT),
                bar_rim: Program::new("bar rim", shaders::BAR_RIM_FRAGMENT),
                shine: Program::new("shine", shaders::SHINE_FRAGMENT),
                tile: Program::new("tile", shaders::TILE_FRAGMENT),
                picture: Program::new("picture", shaders::PICTURE_FRAGMENT),
                glass: Program::new("glass", shaders::GLASS_FRAGMENT),
                ink: Program::new("ink", shaders::INK_FRAGMENT),
                dim: Program::new("dim", shaders::DIM_FRAGMENT),
                fill: Program::new("fill", shaders::FILL_FRAGMENT),
                spin: Program::new("spin", shaders::SPIN_FRAGMENT),
            },
            pictures: Pictures::new(PICTURE_BUDGET_BYTES),
            look,
            liquid: false,
            fonts: Fonts::new(app_dir),
            app_dir: app_dir.to_string(),
            glass_backdrop: Tex::default(),
            veil_backdrop: Tex::default(),
            measures: HashMap::new(),
            fits: HashMap::new(),
        }
    }

    pub fn text(&mut self, text: &str, style: &Style) -> Option<Text> {
        if text.is_empty() {
            return None;
        }
        let key = format!("text:{}:{}:{}:{}:{:?}:{}", style.px, style.weight, style.spacing_em, style.feature, style.ink, text);
        if let (Some(picture), Some(&(left, baseline, advance))) = (self.pictures.find(&key), self.measures.get(&key)) {
            return Some(Text { picture, left, baseline, advance });
        }
        let line = self.fonts.line(text, style.px, style.weight, style.spacing_em * style.px as f32, style.feature)?;
        let shadowed = inked(&line, style.ink, style.shadows);
        let picture = self.keep_line(&key, &shadowed);
        let made = (shadowed.left as f32, shadowed.baseline as f32, shadowed.advance as f32);
        if self.measures.len() > 2000 {
            self.measures.clear();
        }
        self.measures.insert(key, made);
        Some(Text { picture, left: made.0, baseline: made.1, advance: made.2 })
    }

    pub fn text_fit(&mut self, text: &str, style: &Style, max_w: f32) -> Option<Text> {
        let key = format!("{}:{}:{}:{}", style.px, style.weight, max_w as i32, text);
        if let Some(fitted) = self.fits.get(&key).cloned() {
            return self.text(&fitted, style);
        }
        let full = self.text(text, style)?;
        let chars: Vec<char> = text.chars().collect();
        let mut keep = if full.advance <= max_w { chars.len() } else { ((chars.len() as f32 * max_w / full.advance) as usize).min(chars.len() - 1) };
        let fitted = loop {
            let shown: String = if keep == chars.len() { text.to_string() } else { format!("{}…", chars[..keep].iter().collect::<String>().trim_end()) };
            if keep == 0 || self.text(&shown, style).is_some_and(|t| t.advance <= max_w) {
                break shown;
            }
            keep -= 1;
        };
        if self.fits.len() > 500 {
            self.fits.clear();
        }
        self.fits.insert(key, fitted.clone());
        self.text(&fitted, style)
    }

    pub fn clip(&self, area: Option<(f32, f32, f32, f32)>) {
        let Some((x, y, w, h)) = area else { return unsafe { glDisable(GL_SCISSOR_TEST) } };
        let (sx, sy) = (self.gfx.screen_w as f32 / DESIGN_W, self.gfx.screen_h as f32 / DESIGN_H);
        unsafe {
            glEnable(GL_SCISSOR_TEST);
            glScissor((x * sx) as i32, (self.gfx.screen_h as f32 - (y + h) * sy) as i32, (w * sx).ceil() as i32, (h * sy).ceil() as i32);
        }
    }

    pub fn plate(&mut self, p: &Plate, x: f32, y: f32, w: f32, h: f32, alpha: f32) {
        let margin = (p.shadow[0].abs() + p.shadow[1]).ceil();
        let key = format!("plate:{}:{}:{:?}:{}:{:?}:{}", p.h, p.radius, p.fill, p.rim, p.shadow, p.ring);
        let picture = match self.pictures.find(&key) {
            Some(picture) => picture,
            None => {
                let made = plate_picture(p, margin);
                let picture = self.gfx.upload_rgba(&made.rgba, made.w, made.h);
                self.keep(&key, picture);
                picture
            }
        };
        let s = h / p.h;
        let (m, cap) = (margin * s, (p.radius + margin) * s);
        let pw = picture.w as f32;
        let cap_u = (p.radius + margin) / pw;
        let (top, tall) = (y - m, h + 2.0 * m);
        let piece = |d: &Draw, from_u: f32, to_u: f32, at: f32, width: f32| {
            if width <= 0.0 {
                return;
            }
            let full = width / (to_u - from_u);
            d.p.picture.bind(0, c"u_tex", picture.tex);
            d.p.picture.set4(c"u_box", at - from_u * full, top, full, tall);
            d.p.picture.set1(c"u_alpha", alpha);
            d.gfx.quad(&d.p.picture, at, top, width, tall);
        };
        let left = x - m;
        let middle = (w + 2.0 * m - 2.0 * cap).max(0.0);
        piece(self, 0.0, cap_u, left, cap);
        piece(self, cap_u, 1.0 - cap_u, left + cap, middle);
        piece(self, 1.0 - cap_u, 1.0, left + cap + middle, cap);
    }

    pub fn glyph(&mut self, path: &str, size: i32, width: f32) -> Tex {
        let key = format!("glyph:{size}:{width}:{path}");
        if let Some(picture) = self.pictures.find(&key) {
            return picture;
        }
        let coverage = crate::vector::stroke(path, 24.0, size as u32, width);
        let rgba: Vec<u8> = coverage.iter().flat_map(|&a| [a, a, a, a]).collect();
        let picture = self.gfx.upload_rgba(&rgba, size, size);
        self.keep(&key, picture);
        picture
    }

    pub fn inked_picture(&self, picture: Tex, x: f32, y: f32, w: f32, h: f32, ink: [f32; 4]) {
        self.p.ink.bind(0, c"u_tex", picture.tex);
        self.p.ink.set4(c"u_box", x, y, w, h);
        self.p.ink.set4(c"u_ink", ink[0], ink[1], ink[2], ink[3]);
        self.gfx.quad(&self.p.ink, x, y, w, h);
    }

    pub fn hole(&self, x: f32, y: f32, w: f32, h: f32, radius: f32) {
        let p = &self.p.fill;
        p.activate();
        p.set4(c"u_box", x, y, w, h);
        p.set1(c"u_radius", radius);
        p.set4(c"u_ink", 0.0, 0.0, 0.0, 1.0);
        unsafe { glBlendFuncSeparate(GL_ZERO, GL_ONE_MINUS_SRC_ALPHA, GL_ZERO, GL_ONE_MINUS_SRC_ALPHA) }
        self.gfx.quad(p, x, y, w, h);
        unsafe { glBlendFuncSeparate(GL_ONE, GL_ONE_MINUS_SRC_ALPHA, GL_ZERO, GL_ONE) }
    }

    pub fn spinner(&self, x: f32, y: f32, size: f32, width: f32, now: u32) {
        let p = &self.p.spin;
        p.activate();
        p.set4(c"u_box", x, y, size, size);
        p.set1(c"u_width", width);
        p.set1(c"u_turn", (now % SPIN_MS) as f32 / SPIN_MS as f32 * std::f32::consts::TAU);
        self.gfx.quad(p, x, y, size, size);
    }

    pub fn bend(&self) -> f32 {
        if self.liquid { self.look.bend } else { 0.0 }
    }

    pub fn magnify(&self) -> f32 {
        if self.liquid { self.look.magnify } else { 0.0 }
    }

    pub fn light(&self) -> [f32; 2] {
        let a = self.look.light_angle.to_radians();
        [a.sin(), -a.cos()]
    }

    pub fn fill(&self, x: f32, y: f32, w: f32, h: f32, radius: f32, ink: [f32; 4]) {
        let p = &self.p.fill;
        p.activate();
        p.set4(c"u_box", x, y, w, h);
        p.set1(c"u_radius", radius);
        p.set4(c"u_ink", ink[0], ink[1], ink[2], ink[3]);
        self.gfx.quad(p, x, y, w, h);
    }

    pub fn glass(&self, g: &Glass) {
        let p = &self.p.glass;
        p.bind(0, c"u_glass", self.glass_backdrop.tex);
        p.set4(c"u_box", g.x, g.y, g.w, g.h);
        p.set4(c"u_look", g.radius, if self.liquid { 1.0 } else { g.blur }, g.sheen * self.look.sheen, g.rim * self.look.rim_light);
        p.set3(c"u_tone", [self.look.saturate, self.look.brighten, self.look.tint]);
        p.set1(c"u_alpha", g.alpha);
        p.set1(c"u_shadow_strength", self.look.glass_shadow);
        p.set3(c"u_bend", [self.bend(), self.look.fringe, self.look.bezel]);
        p.set4(c"u_shape", self.look.edge_shape, self.magnify(), self.look.frost, self.look.glow);
        let [lx, ly] = self.light();
        p.set4(c"u_light", lx, ly, self.look.specular, self.look.button_colour_rims);
        p.set2(c"u_side", self.look.side_width, self.look.side_bend);
        let margin = 40.0;
        self.gfx.quad(p, g.x - margin, g.y - margin, g.w + 2.0 * margin, g.h + 2.0 * margin);
    }

    pub fn keep(&mut self, key: &str, picture: Tex) {
        let dropped = self.pictures.keep(key, picture, picture.bytes());
        self.gfx.free_all(dropped);
    }

    pub fn let_go(&mut self, which: impl Fn(&str) -> bool) {
        let dropped = self.pictures.let_go(which);
        self.gfx.free_all(dropped);
    }

    pub fn keep_line(&mut self, key: &str, line: &Line) -> Tex {
        let picture = self.gfx.upload_rgba(&line.rgba, line.w, line.h);
        self.keep(key, picture);
        picture
    }

    pub fn copy_whole(&self, from: Tex, scale_x: f32, scale_y: f32, offset_x: f32, offset_y: f32) {
        self.copy_dimmed(from, (scale_x, scale_y, offset_x, offset_y), 1.0);
    }

    pub fn copy_dimmed(&self, from: Tex, (scale_x, scale_y, offset_x, offset_y): (f32, f32, f32, f32), mul: f32) {
        self.p.copy.bind(0, c"u_tex", from.tex);
        self.p.copy.set1(c"u_mul", mul);
        self.p.copy.set4(c"u_uvx", scale_x, scale_y, offset_x, offset_y);
        self.gfx.quad(&self.p.copy, 0.0, 0.0, DESIGN_W, DESIGN_H);
    }

    pub fn picture(&self, picture: Tex, x: f32, y: f32, w: f32, h: f32, alpha: f32) {
        self.p.picture.bind(0, c"u_tex", picture.tex);
        self.p.picture.set4(c"u_box", x, y, w, h);
        self.p.picture.set1(c"u_alpha", alpha);
        self.gfx.quad(&self.p.picture, x, y, w, h);
    }

    pub fn blurred(&self, from: Tex, halvings: i32, passes: i32, reach: f32, mul: f32) -> Tex {
        let mut current = Tex::default();
        let mut source = from;
        let mut made = 0;
        while made < halvings || current.tex == 0 {
            let shrink = if halvings > 0 { 2 } else { 1 };
            let next = self.gfx.blank(source.w / shrink, source.h / shrink);
            self.gfx.stretched_over(&next);
            self.copy_whole(source, 1.0, 1.0, 0.0, 0.0);
            self.gfx.free(&mut current);
            current = next;
            source = current;
            made += 1;
        }
        let mut other = self.gfx.blank(current.w, current.h);
        for pass in 0..passes {
            for (into, from, step, mul) in [(&other, &current, (reach / current.w as f32, 0.0), 1.0), (&current, &other, (0.0, reach / current.h as f32), if pass == passes - 1 { mul } else { 1.0 })] {
                self.gfx.stretched_over(into);
                self.p.blur.bind(0, c"u_tex", from.tex);
                self.p.blur.set2(c"u_step", step.0, step.1);
                self.p.blur.set1(c"u_mul", mul);
                self.gfx.quad(&self.p.blur, 0.0, 0.0, DESIGN_W, DESIGN_H);
            }
        }
        self.gfx.free(&mut other);
        current
    }
}

fn plate_picture(p: &Plate, margin: f32) -> Decoded {
    let (w, h) = ((2.0 * (p.radius + margin) + 2.0) as usize, (p.h + 2.0 * margin) as usize);
    let half = [(w as f32 - 2.0 * margin) / 2.0, p.h / 2.0];
    let r = p.radius.min(half[0]).min(half[1]);
    let sdf = |x: f32, y: f32| {
        let q = [x.abs() - (half[0] - r), y.abs() - (half[1] - r)];
        let outside = (q[0].max(0.0).powi(2) + q[1].max(0.0).powi(2)).sqrt();
        let normal = if q[0] > 0.0 && q[1] > 0.0 { [q[0].signum() * x.signum() * q[0] / outside, y.signum() * q[1] / outside] } else if q[0] > q[1] { [x.signum(), 0.0] } else { [0.0, y.signum()] };
        (outside + q[0].max(q[1]).min(0.0) - r, normal)
    };
    let mut rgba = vec![0u8; w * h * 4];
    for py in 0..h {
        for px in 0..w {
            let (cx, cy) = (px as f32 + 0.5 - w as f32 / 2.0, py as f32 + 0.5 - h as f32 / 2.0);
            let (d, n) = sdf(cx, cy);
            let cover = (0.5 - d).clamp(0.0, 1.0) - if p.ring > 0.0 { (0.5 - d - p.ring).clamp(0.0, 1.0) } else { 0.0 };
            let (sd, _) = sdf(cx, cy - p.shadow[0]);
            let blur = p.shadow[1].max(1.0);
            let shadow = p.shadow[2] * (1.0 - ((sd + 0.6 * blur) / (1.6 * blur)).clamp(0.0, 1.0)).powi(2);
            let edge = 1.0 - ((-d - 0.5) / 1.5).clamp(0.0, 1.0);
            let rim = p.rim * edge * (0.45 * (-n[1]).max(0.0) + 0.2 * (-n[0]).max(0.0));
            let fill_a = p.fill[3] + (1.0 - p.fill[3]) * rim;
            let fill_rgb: Vec<f32> = (0..3).map(|c| (p.fill[c] * p.fill[3] * (1.0 - rim) + rim) / fill_a.max(0.0001)).collect();
            let a_shape = fill_a * cover;
            let out_a = a_shape + (1.0 - a_shape) * shadow;
            let at = (py * w + px) * 4;
            for c in 0..3 {
                rgba[at + c] = ((fill_rgb[c] * a_shape).clamp(0.0, 1.0) * 255.0) as u8;
            }
            rgba[at + 3] = (out_a.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    Decoded { rgba, w: w as i32, h: h as i32 }
}
