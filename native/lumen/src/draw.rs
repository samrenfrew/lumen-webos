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

pub struct Decoded {
    pub rgba: Vec<u8>,
    pub w: i32,
    pub h: i32,
}

pub struct Draw {
    pub gfx: Gfx,
    pub p: Programs,
    pub pictures: Pictures<Tex>,
    pub look: Look,
    pub fonts: Fonts,
    pub app_dir: String,
    pub glass_backdrop: Tex,
    measures: HashMap<String, (f32, f32, f32)>,
}

pub fn decoded(path: &str) -> Option<Decoded> {
    let c_path = CString::new(path).ok()?;
    unsafe {
        let loaded = IMG_Load(c_path.as_ptr());
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
            },
            pictures: Pictures::new(PICTURE_BUDGET_BYTES),
            look,
            fonts: Fonts::new(app_dir),
            app_dir: app_dir.to_string(),
            glass_backdrop: Tex::default(),
            measures: HashMap::new(),
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

    pub fn glass(&self, g: &Glass) {
        let p = &self.p.glass;
        p.bind(0, c"u_glass", self.glass_backdrop.tex);
        p.set4(c"u_box", g.x, g.y, g.w, g.h);
        p.set4(c"u_look", g.radius, g.blur, g.sheen, g.rim);
        p.set3(c"u_tone", [self.look.saturate, self.look.brighten, self.look.tint]);
        p.set1(c"u_alpha", g.alpha);
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
        self.p.copy.bind(0, c"u_tex", from.tex);
        self.p.copy.set4(c"u_uvx", scale_x, scale_y, offset_x, offset_y);
        self.gfx.quad(&self.p.copy, 0.0, 0.0, DESIGN_W, DESIGN_H);
    }

    pub fn picture(&self, picture: Tex, x: f32, y: f32, w: f32, h: f32, alpha: f32) {
        self.p.picture.bind(0, c"u_tex", picture.tex);
        self.p.picture.set4(c"u_box", x, y, w, h);
        self.p.picture.set1(c"u_alpha", alpha);
        self.gfx.quad(&self.p.picture, x, y, w, h);
    }

    pub fn blurred(&self, from: Tex, halvings: i32, passes: i32, mul: f32) -> Tex {
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
            for (into, from, step, mul) in [(&other, &current, (1.0 / current.w as f32, 0.0), 1.0), (&current, &other, (0.0, 1.0 / current.h as f32), if pass == passes - 1 { mul } else { 1.0 })] {
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
