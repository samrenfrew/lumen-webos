use crate::ffi::*;
use crate::gfx::{Gfx, Program, Tex, DESIGN_H, DESIGN_W};
use crate::look::Look;
use crate::pictures::Pictures;
use crate::shaders;
use crate::text::{Fonts, Line};
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
            },
            pictures: Pictures::new(PICTURE_BUDGET_BYTES),
            look,
            fonts: Fonts::new(app_dir),
            app_dir: app_dir.to_string(),
        }
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
