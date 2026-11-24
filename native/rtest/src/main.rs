mod ffi;
mod gfx;
#[cfg(feature = "webos")]
mod probe;
mod shaders;
mod util;

use ffi::*;
use gfx::{Gfx, Program, Target, Tex, DESIGN_H, DESIGN_W};
use std::ffi::{c_int, CStr, CString};
use std::ptr;
use util::status_kb;

const TUNING_PATH: &str = "/tmp/ntest-tune";
const BENCH_PATH: &str = "/tmp/ntest-bench";
const WEBOS_BACK_SCANCODE: c_int = 482;
const STEPS_PATH: &str = "/tmp/rtest-steps";

const ROWS: usize = 3;
const COLS: usize = 6;
const TILES: usize = ROWS * COLS;
const IDLE_WAIT_MS: c_int = 250;
const PLATE_MARGIN: f32 = 70.0;
const TILE_MARGIN: f32 = 28.0;

const LIQUID: usize = 0;
const NO_BLUR: usize = 2;
const MODE_NAMES: [&str; 3] = ["Liquid glass", "Blur only", "No blur"];

const TILE_W: f32 = 240.0;
const TILE_H: f32 = 144.0;
const TILE_GAP: f32 = 46.0;
const TILE_RADIUS: f32 = 34.0;
const ROWS_EDGE: f32 = 125.0;
const ROW_Y: [f32; ROWS] = [846.0, 1060.0, 1317.0];
const BAR_X: f32 = 85.0;
const BAR_Y: f32 = 806.0;
const BAR_W: f32 = 1750.0;
const BAR_H: f32 = 224.0;
const BAR_RADIUS: f32 = 74.0;
const LOWER_SHIFT: f32 = 770.0;
const GRID_DROP: f32 = 40.0;
const LIFT_BAR: f32 = 1.12;
const LIFT_GRID: f32 = 1.14;
const LIFT_RISE: f32 = 7.5;
const SHADOW_REST: [f32; 3] = [8.0, 16.0, 0.30];
const SHADOW_BAR: [f32; 3] = [12.0, 22.0, 0.38];
const SHADOW_GRID: [f32; 3] = [26.0, 40.0, 0.46];
const VEIL_TINT: f32 = 0.26;

type Curve = [f32; 4];
static EASE_UP: Curve = [0.65, 0.0, 0.25, 1.0];
static EASE_DOWN: Curve = [0.25, 0.1, 0.25, 1.0];
static EASE_VEIL: Curve = [0.45, 0.0, 0.55, 1.0];
static EASE_OUT: Curve = [0.0, 0.0, 0.58, 1.0];
static EASE_SCROLL: Curve = [0.2, 0.7, 0.2, 1.0];
static EASE: Curve = [0.25, 0.1, 0.25, 1.0];

const TEXT_SAMPLES: [(&str, i32, f32); 11] = [
    ("The quick brown fox jumps over the lazy dog. Settings, Wallpaper, Live TV", 34, 0.0),
    ("Zwölf Boxkämpfer jagen Viktor quer über den großen Sylter Deich. Einstellungen", 34, 0.0),
    ("El veloz murciélago hindú comía feliz cardillo y kiwi. Configuración, ¿Qué ver?", 34, 0.0),
    ("Portez ce vieux whisky au juge blond qui fume. Paramètres, Fond d’écran, Télé", 34, 0.0),
    ("Pranzo d'acqua fa volti sghembi. Impostazioni, Sfondo, TV in diretta, Più", 34, 0.0),
    ("À noite, vovô Kowalsky vê o ímã cair no pé do pinguim. Configurações, Não", 34, 0.0),
    ("Pa's wijze lynx bezag vroom het fikse aquaduct. Instellingen, Achtergrond, IJ", 34, 0.0),
    ("Pijamalı hasta yağız şoföre çabucak güvendi. Ayarlar, Duvar kâğıdı, İĞŞ ığş", 34, 0.0),
    ("सेटिंग्स, वॉलपेपर, लाइव टीवी। ऋषियों को सताने वाले दुष्ट राक्षसों के राजा रावण का सर्वनाश", 34, 0.0),
    ("AV To Wa Ty LT P. 11° 21:17 fi fl", 64, 0.0),
    ("Letter-spacing of -0.02em: Recently watched, Continue watching", 34, -0.68),
];
const TEXT_LEFT: f32 = 80.0;
const TEXT_TOP: f32 = 50.0;
const TEXT_STEP: f32 = 90.0;

const NO_BACKDROP: u32 = 1;
const NO_GRID: u32 = 2;
const NO_BAR: u32 = 4;
const NO_BAR_TILES: u32 = 8;
const NO_HUD: u32 = 16;
const NO_CLEAR: u32 = 32;

struct Tune {
    down_ms: f32,
    up_ms: f32,
    veil_down_ms: f32,
    veil_up_ms: f32,
    focus_ms: f32,
    glass_halvings: f32,
    glass_passes: f32,
    veil_halvings: f32,
    veil_passes: f32,
    bezel: f32,
    bend: f32,
    fringe: f32,
    saturate: f32,
    brighten: f32,
    tint: f32,
    mode: f32,
    hud: f32,
    finish: f32,
    one_fbo: f32,
    no_depth: f32,
    text: f32,
    wallpaper: String,
}

impl Tune {
    fn defaults() -> Tune {
        Tune {
            down_ms: 675.0,
            up_ms: 600.0,
            veil_down_ms: 600.0,
            veil_up_ms: 800.0,
            focus_ms: 160.0,
            glass_halvings: 1.0,
            glass_passes: 2.0,
            veil_halvings: 2.0,
            veil_passes: 3.0,
            bezel: 28.0,
            bend: 44.0,
            fringe: 0.12,
            saturate: 1.5,
            brighten: 1.1,
            tint: 0.3,
            mode: LIQUID as f32,
            hud: 1.0,
            finish: 1.0,
            one_fbo: 0.0,
            no_depth: 0.0,
            text: 0.0,
            wallpaper: String::new(),
        }
    }

    fn tunable(&mut self, name: &str) -> Option<(&mut f32, f32, f32)> {
        Some(match name {
            "down" => (&mut self.down_ms, 1.0, 5000.0),
            "up" => (&mut self.up_ms, 1.0, 5000.0),
            "veil_down" => (&mut self.veil_down_ms, 1.0, 5000.0),
            "veil_up" => (&mut self.veil_up_ms, 1.0, 5000.0),
            "focus" => (&mut self.focus_ms, 1.0, 2000.0),
            "glass_halvings" => (&mut self.glass_halvings, 0.0, 4.0),
            "glass_passes" => (&mut self.glass_passes, 1.0, 8.0),
            "veil_halvings" => (&mut self.veil_halvings, 0.0, 4.0),
            "veil_passes" => (&mut self.veil_passes, 1.0, 8.0),
            "bezel" => (&mut self.bezel, 1.0, 110.0),
            "bend" => (&mut self.bend, -200.0, 200.0),
            "fringe" => (&mut self.fringe, 0.0, 1.0),
            "saturate" => (&mut self.saturate, 0.0, 3.0),
            "brighten" => (&mut self.brighten, 0.0, 3.0),
            "tint" => (&mut self.tint, 0.0, 1.0),
            "mode" => (&mut self.mode, 0.0, (MODE_NAMES.len() - 1) as f32),
            "hud" => (&mut self.hud, 0.0, 1.0),
            "finish" => (&mut self.finish, 0.0, 1.0),
            "one_fbo" => (&mut self.one_fbo, 0.0, 1.0),
            "no_depth" => (&mut self.no_depth, 0.0, 1.0),
            "text" => (&mut self.text, 0.0, 1.0),
            _ => return None,
        })
    }

    fn apply(&mut self, text: &str) {
        for line in text.lines() {
            let Some((name, value)) = line.split_once('=') else { continue };
            if name == "wallpaper" {
                self.wallpaper = value.to_string();
                continue;
            }
            let v: f32 = value.trim().parse().unwrap_or(0.0);
            if let Some((slot, min, max)) = self.tunable(name) {
                if v >= min && v <= max {
                    *slot = v;
                }
            }
        }
    }

    fn read(&mut self) {
        if let Ok(text) = std::fs::read_to_string(TUNING_PATH) {
            self.apply(&text);
        }
    }

    fn mode(&self) -> usize {
        self.mode as usize
    }

    fn blur_strengths(&self) -> [i32; 4] {
        [self.glass_halvings as i32, self.glass_passes as i32, self.veil_halvings as i32, self.veil_passes as i32]
    }
}

#[derive(Clone, Copy)]
struct Anim {
    from: f32,
    to: f32,
    start: u32,
    dur: u32,
    ease: &'static Curve,
}

fn bezier(a: f32, b: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
}

fn eased(e: &Curve, x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let (mut lo, mut hi, mut t) = (0.0f32, 1.0f32, x);
    for _ in 0..20 {
        if bezier(e[0], e[2], t) < x {
            lo = t;
        } else {
            hi = t;
        }
        t = (lo + hi) / 2.0;
    }
    bezier(e[1], e[3], t)
}

impl Anim {
    fn still(value: f32) -> Anim {
        Anim { from: value, to: value, start: 0, dur: 0, ease: &EASE }
    }

    fn at(&self, now: u32) -> f32 {
        let since = now.wrapping_sub(self.start);
        if self.dur == 0 || since >= self.dur {
            return self.to;
        }
        self.from + (self.to - self.from) * eased(self.ease, since as f32 / self.dur as f32)
    }

    fn busy(&self, now: u32) -> bool {
        self.dur != 0 && now.wrapping_sub(self.start) < self.dur
    }

    fn go(&mut self, to: f32, ms: f32, ease: &'static Curve, now: u32) {
        if self.to == to {
            return;
        }
        self.from = self.at(now);
        self.to = to;
        self.start = now;
        self.dur = ms as u32;
        self.ease = ease;
    }

    fn set(&mut self, value: f32) {
        self.from = value;
        self.to = value;
        self.dur = 0;
    }
}

struct Tile {
    icon: Tex,
    bg: [f32; 3],
    focus: Anim,
    resting: Target,
}

struct Programs {
    copy: Program,
    blur: Program,
    backdrop: Program,
    lens: Program,
    gloss: Program,
    bar_middle: Program,
    bar_rim: Program,
    shine: Program,
    tile: Program,
    fill: Program,
    text: Program,
}

#[derive(Default)]
struct Move {
    active: bool,
    frames: u32,
    late: u32,
    started: u32,
    last: u32,
    slowest: u32,
    draw_ms: f64,
    draw_worst: f64,
    name: &'static str,
}

struct App {
    gfx: Gfx,
    p: Programs,
    tune: Tune,
    app_dir: String,
    screen_w: i32,
    screen_h: i32,
    sharp: Target,
    glass: Target,
    veil: Target,
    lens_plate: Target,
    gloss_plate: Target,
    baked_bezel: f32,
    refresh_ms: f32,
    tiles: Vec<Tile>,
    hud: Tex,
    font: *mut TTF_Font,
    slide: Anim,
    veil_fade: Anim,
    drop: Anim,
    shine_x: Anim,
    shine_alpha: Anim,
    focus_row: usize,
    focus_col: usize,
    last_move: String,
    leave_out: u32,
    mv: Move,
    text_sheet: Vec<(Tex, f32, f32)>,
}

fn ticks() -> u32 {
    unsafe { SDL_GetTicks() }
}

fn counter() -> u64 {
    unsafe { SDL_GetPerformanceCounter() }
}

fn ms_per_count() -> f64 {
    1000.0 / unsafe { SDL_GetPerformanceFrequency() } as f64
}

fn sdl_error() -> String {
    unsafe { CStr::from_ptr(SDL_GetError()) }.to_string_lossy().into_owned()
}

fn gl_string(name: GLenum) -> String {
    let s = unsafe { glGetString(name) };
    if s.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(s) }.to_string_lossy().into_owned()
}

fn c_path(path: &str) -> CString {
    CString::new(path).unwrap_or_default()
}

fn step(name: &str) {
    if std::path::Path::new(STEPS_PATH).exists() {
        unsafe { glFinish() }
        note!("step {}", name);
        std::thread::sleep(std::time::Duration::from_millis(2500));
    }
}

fn tile_centre(col: usize) -> f32 {
    ROWS_EDGE + col as f32 * (TILE_W + TILE_GAP) + TILE_W / 2.0
}

fn closes(key: i32, scancode: c_int) -> bool {
    key == SDLK_ESCAPE || key == SDLK_BACKSPACE || key == SDLK_AC_BACK || key == SDLK_Q || scancode == WEBOS_BACK_SCANCODE
}

impl App {
    fn copy(&self, from: Tex, into: Option<&Target>, sx: f32, sy: f32, ox: f32, oy: f32) {
        match into {
            Some(target) => self.gfx.into_target(target),
            None => self.gfx.into_screen(),
        }
        self.p.copy.bind(0, c"u_tex", from.tex);
        self.p.copy.set4(c"u_uvx", sx, sy, ox, oy);
        self.gfx.quad(&self.p.copy, 0.0, 0.0, DESIGN_W, DESIGN_H);
    }

    fn blurred(&self, from: &Target, halvings: i32, passes: i32, mul: f32) -> Target {
        let mut cur = Target::default();
        let mut src = from.t;
        let mut i = 0;
        while i < halvings || cur.fbo == 0 {
            let shrink = if halvings > 0 { 2 } else { 1 };
            let next = self.gfx.target(src.w / shrink, src.h / shrink);
            self.copy(src, Some(&next), 1.0, 1.0, 0.0, 0.0);
            if cur.fbo != 0 {
                self.gfx.target_free(&mut cur);
            }
            cur = next;
            src = cur.t;
            i += 1;
        }
        let mut tmp = self.gfx.target(cur.t.w, cur.t.h);
        for i in 0..passes {
            self.gfx.into_target(&tmp);
            self.p.blur.bind(0, c"u_tex", cur.t.tex);
            self.p.blur.set2(c"u_step", 1.0 / cur.t.w as f32, 0.0);
            self.p.blur.set1(c"u_mul", 1.0);
            self.gfx.quad(&self.p.blur, 0.0, 0.0, DESIGN_W, DESIGN_H);
            self.gfx.into_target(&cur);
            self.p.blur.bind(0, c"u_tex", tmp.t.tex);
            self.p.blur.set2(c"u_step", 0.0, 1.0 / cur.t.h as f32);
            self.p.blur.set1(c"u_mul", if i == passes - 1 { mul } else { 1.0 });
            self.gfx.quad(&self.p.blur, 0.0, 0.0, DESIGN_W, DESIGN_H);
        }
        self.gfx.target_free(&mut tmp);
        cur
    }

    fn build_backdrops(&mut self) -> bool {
        let t0 = ticks();
        let path = if self.tune.wallpaper.is_empty() { format!("{}/wallpaper.jpg", self.app_dir) } else { self.tune.wallpaper.clone() };
        let picture = unsafe { IMG_Load(c_path(&path).as_ptr()) };
        if picture.is_null() {
            note!("wallpaper {}: {}", path, sdl_error());
            return false;
        }
        let decoded = ticks();
        let mut raw = self.gfx.upload(picture, false);
        unsafe {
            SDL_FreeSurface(picture);
            glDisable(GL_BLEND);
        }
        if self.sharp.fbo != 0 {
            self.gfx.target_free(&mut self.sharp);
            self.gfx.target_free(&mut self.glass);
            self.gfx.target_free(&mut self.veil);
        }
        self.sharp = self.gfx.target(self.screen_w, self.screen_h);
        let shape = (raw.w as f32 / raw.h as f32) / (DESIGN_W / DESIGN_H);
        if shape > 1.0 {
            self.copy(raw, Some(&self.sharp), 1.0 / shape, 1.0, (1.0 - 1.0 / shape) / 2.0, 0.0);
        } else {
            self.copy(raw, Some(&self.sharp), 1.0, shape, 0.0, (1.0 - shape) / 2.0);
        }
        self.gfx.free(&mut raw);
        let mut half = self.gfx.target(self.screen_w / 2, self.screen_h / 2);
        self.copy(self.sharp.t, Some(&half), 1.0, 1.0, 0.0, 0.0);
        let [glass_halvings, glass_passes, veil_halvings, veil_passes] = self.tune.blur_strengths();
        self.glass = self.blurred(&half, glass_halvings, glass_passes, 1.0);
        self.veil = self.blurred(&half, veil_halvings, veil_passes, 1.0 - VEIL_TINT);
        self.gfx.target_free(&mut half);
        unsafe { glFinish() }
        self.gfx.into_screen();
        note!(
            "backdrops: decode {} ms, upload and blur {} ms, glass {}x{}, veil {}x{}",
            decoded.wrapping_sub(t0),
            ticks().wrapping_sub(decoded),
            self.glass.t.w,
            self.glass.t.h,
            self.veil.t.w,
            self.veil.t.h
        );
        true
    }

    fn load_tiles(&mut self) {
        let mut found: Vec<String> = std::fs::read_dir(format!("{}/icons", self.app_dir))
            .map(|dir| dir.flatten().filter_map(|entry| entry.path().to_str().map(String::from)).collect())
            .unwrap_or_default();
        found.retain(|path| path.ends_with(".png") && !path.rsplit('/').next().unwrap_or("").starts_with('.'));
        found.sort();
        for i in 0..TILES {
            let mut tile = Tile { icon: Tex::default(), bg: [0.16, 0.17, 0.22], focus: Anim::still(0.0), resting: Target::default() };
            if !found.is_empty() {
                unsafe {
                    let loaded = IMG_Load(c_path(&found[i % found.len()]).as_ptr());
                    let icon = if loaded.is_null() { ptr::null_mut() } else { SDL_ConvertSurfaceFormat(loaded, SDL_PIXELFORMAT_RGBA32, 0) };
                    if !loaded.is_null() {
                        SDL_FreeSurface(loaded);
                    }
                    if !icon.is_null() {
                        let edge = (*icon).pixels.cast::<u8>().add(((*icon).h / 2) as usize * (*icon).pitch as usize + 3 * 4);
                        if *edge.add(3) > 200 {
                            for c in 0..3 {
                                tile.bg[c] = *edge.add(c) as f32 / 255.0;
                            }
                        }
                        tile.icon = self.gfx.upload(icon, false);
                        SDL_FreeSurface(icon);
                    }
                }
            }
            self.tiles.push(tile);
        }
        note!("tiles: {} icons", found.len());
    }

    fn open_font(&mut self) {
        let places = [format!("{}/fonts/sf-500.ttf", self.app_dir), "/usr/share/fonts/LG_Smart_UI-Regular.ttf".into(), "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf".into()];
        for path in places {
            if self.font.is_null() {
                self.font = unsafe { TTF_OpenFont(c_path(&path).as_ptr(), 24) };
            }
        }
        if self.font.is_null() {
            note!("hud: no font");
        }
    }

    fn draw_hud_text(&mut self) {
        self.gfx.free(&mut self.hud);
        self.hud.w = 0;
        self.hud.h = 0;
        if self.font.is_null() {
            return;
        }
        let first = c_path(&format!("Glass: {}", MODE_NAMES[self.tune.mode()]));
        let second = c_path(&self.last_move);
        let white = SDL_Color { r: 255, g: 255, b: 255, a: 255 };
        unsafe {
            let lines = [TTF_RenderUTF8_Blended(self.font, first.as_ptr(), white), TTF_RenderUTF8_Blended(self.font, second.as_ptr(), white)];
            if !lines[0].is_null() && !lines[1].is_null() {
                let w = (*lines[0]).w.max((*lines[1]).w);
                let both = SDL_CreateRGBSurfaceWithFormat(0, w, (*lines[0]).h + (*lines[1]).h + 6, 32, SDL_PIXELFORMAT_RGBA32);
                if !both.is_null() {
                    SDL_SetSurfaceBlendMode(lines[0], SDL_BLENDMODE_NONE);
                    SDL_SetSurfaceBlendMode(lines[1], SDL_BLENDMODE_NONE);
                    let mut below = SDL_Rect { x: 0, y: (*lines[0]).h + 6, w: 0, h: 0 };
                    SDL_UpperBlit(lines[0], ptr::null(), both, ptr::null_mut());
                    SDL_UpperBlit(lines[1], ptr::null(), both, &mut below);
                    self.hud = self.gfx.upload(both, true);
                    SDL_FreeSurface(both);
                }
            }
            for line in lines {
                if !line.is_null() {
                    SDL_FreeSurface(line);
                }
            }
        }
    }

    fn build_text_sheet(&mut self) {
        let first_there = |places: &[String]| places.iter().find(|p| std::path::Path::new(p).exists()).cloned().unwrap_or_default();
        let font = first_there(&[format!("{}/fonts/sf-500.ttf", self.app_dir), "/usr/share/fonts/LG_Smart_UI-Regular.ttf".into(), "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf".into()]);
        let other = first_there(&["/usr/share/fonts/LG_Smart_UI_Devanagari.ttf".into(), "/usr/share/fonts/truetype/lohit-devanagari/Lohit-Devanagari.ttf".into()]);
        note!("text: HarfBuzz {}, {}, {}", unsafe { CStr::from_ptr(lumen_text_shaper()) }.to_string_lossy(), font, other);
        let (font, other) = (c_path(&font), c_path(&other));
        let began = std::time::Instant::now();
        for (i, (text, px, spacing)) in TEXT_SAMPLES.iter().enumerate() {
            let (mut rgba, mut w, mut h) = (ptr::null_mut(), 0, 0);
            let line = c_path(text);
            if unsafe { lumen_text_line(font.as_ptr(), other.as_ptr(), *px, *spacing, line.as_ptr(), &mut rgba, &mut w, &mut h) } == 0 {
                note!("text line {}: not drawn", i);
                continue;
            }
            self.text_sheet.push((self.gfx.upload_rgba(rgba, w, h), TEXT_LEFT, TEXT_TOP + i as f32 * TEXT_STEP));
            unsafe { lumen_text_free(rgba) }
            note!("text line {}: {} px wide at {} px", i, w - 6, px);
        }
        note!("text: {} lines in {} ms", self.text_sheet.len(), began.elapsed().as_millis());
    }

    fn draw_text_sheet(&self) {
        unsafe {
            glClearColor(0.06, 0.06, 0.08, 1.0);
            glClear(GL_COLOR_BUFFER_BIT);
            glClearColor(0.0, 0.0, 0.0, 1.0);
            glEnable(GL_BLEND);
            glBlendFuncSeparate(GL_ONE, GL_ONE_MINUS_SRC_ALPHA, GL_ZERO, GL_ONE);
        }
        self.gfx.into_screen();
        for (tex, x, y) in &self.text_sheet {
            self.p.text.bind(0, c"u_tex", tex.tex);
            self.p.text.set4(c"u_box", *x, *y, tex.w as f32, tex.h as f32);
            self.gfx.quad(&self.p.text, *x, *y, tex.w as f32, tex.h as f32);
        }
    }

    fn tile_uniforms(&self, t: &Tile, x: f32, y: f32, scale: f32, shadow: [f32; 3], focus: f32) {
        let p = &self.p.tile;
        p.bind(0, c"u_icon", t.icon.tex);
        p.set4(c"u_box", x, y, TILE_W * scale, TILE_H * scale);
        p.set1(c"u_radius", TILE_RADIUS * scale);
        p.set3(c"u_bg", t.bg);
        p.set3(c"u_shadow", shadow);
        p.set1(c"u_focus", focus);
    }

    fn bake_tiles(&mut self) {
        unsafe { glDisable(GL_BLEND) }
        let (w, h) = (TILE_W + 2.0 * TILE_MARGIN, TILE_H + 2.0 * TILE_MARGIN);
        let shared = if self.tune.one_fbo != 0.0 { self.gfx.shared_fbo() } else { 0 };
        for i in 0..TILES {
            self.tiles[i].resting = if shared != 0 { self.gfx.target_on(shared, w as i32, h as i32) } else { self.gfx.target(w as i32, h as i32) };
            self.gfx.into_plate(&self.tiles[i].resting);
            self.tile_uniforms(&self.tiles[i], TILE_MARGIN, TILE_MARGIN, 1.0, SHADOW_REST, 0.0);
            self.gfx.quad(&self.p.tile, 0.0, 0.0, w, h);
        }
        self.gfx.into_screen();
    }

    fn bake_bar(&mut self) {
        let (w, h) = (BAR_W + 2.0 * PLATE_MARGIN, BAR_H + 2.0 * PLATE_MARGIN);
        if self.lens_plate.fbo == 0 {
            self.lens_plate = self.gfx.target(w as i32, h as i32);
            self.gloss_plate = self.gfx.target(w as i32, h as i32);
        }
        unsafe { glDisable(GL_BLEND) }
        self.gfx.into_plate(&self.lens_plate);
        self.p.lens.activate();
        self.p.lens.set4(c"u_bar", PLATE_MARGIN, PLATE_MARGIN, BAR_W, BAR_H);
        self.p.lens.set1(c"u_radius", BAR_RADIUS);
        self.p.lens.set1(c"u_bezel", self.tune.bezel);
        self.gfx.quad(&self.p.lens, 0.0, 0.0, w, h);
        self.gfx.into_plate(&self.gloss_plate);
        self.p.gloss.activate();
        self.p.gloss.set4(c"u_bar", PLATE_MARGIN, PLATE_MARGIN, BAR_W, BAR_H);
        self.p.gloss.set1(c"u_radius", BAR_RADIUS);
        self.gfx.quad(&self.p.gloss, 0.0, 0.0, w, h);
        self.gfx.into_screen();
        self.baked_bezel = self.tune.bezel;
    }

    fn draw_tile(&self, index: usize, shift: f32, grid_drop: f32, now: u32) {
        let t = &self.tiles[index];
        let (row, col) = (index / COLS, index % COLS);
        let f = t.focus.at(now);
        let left = ROWS_EDGE + col as f32 * (TILE_W + TILE_GAP);
        let top = ROW_Y[row] - shift + if row > 0 { grid_drop } else { 0.0 };
        if top - 80.0 > DESIGN_H || top + TILE_H + 80.0 < 0.0 {
            return;
        }
        if f <= 0.0 && t.resting.fbo != 0 {
            let (w, h) = (TILE_W + 2.0 * TILE_MARGIN, TILE_H + 2.0 * TILE_MARGIN);
            self.p.text.bind(0, c"u_tex", t.resting.t.tex);
            self.p.text.set4(c"u_box", left - TILE_MARGIN, top - TILE_MARGIN, w, h);
            self.gfx.quad(&self.p.text, left - TILE_MARGIN, top - TILE_MARGIN, w, h);
            return;
        }
        let scale = 1.0 + ((if row > 0 { LIFT_GRID } else { LIFT_BAR }) - 1.0) * f;
        let (w, h) = (TILE_W * scale, TILE_H * scale);
        let x = left + TILE_W / 2.0 - w / 2.0;
        let y = top + TILE_H / 2.0 - LIFT_RISE * f * scale - h / 2.0;
        let lifted = if row > 0 { SHADOW_GRID } else { SHADOW_BAR };
        let mut shadow = [0.0f32; 3];
        for i in 0..3 {
            shadow[i] = SHADOW_REST[i] + (lifted[i] - SHADOW_REST[i]) * f;
        }
        let margin = shadow[0] + shadow[1] + 4.0;
        self.tile_uniforms(t, x, y, scale, shadow, f);
        self.gfx.quad(&self.p.tile, x - margin, y - margin, w + 2.0 * margin, h + 2.0 * margin);
    }

    fn draw_row(&self, row: usize, shift: f32, grid_drop: f32, now: u32) {
        for col in 0..COLS {
            if !(row == self.focus_row && col == self.focus_col) {
                self.draw_tile(row * COLS + col, shift, grid_drop, now);
            }
        }
        if row == self.focus_row {
            self.draw_tile(row * COLS + self.focus_col, shift, grid_drop, now);
        }
    }

    fn bar_uniforms(&self, program: &Program, plate_x: f32, plate_y: f32, veiled: f32) {
        let plain = self.tune.mode() == NO_BLUR;
        program.bind(0, c"u_glass", if plain { self.sharp.t.tex } else { self.glass.t.tex });
        program.bind(1, c"u_veil", self.veil.t.tex);
        program.bind(2, c"u_gloss", self.gloss_plate.t.tex);
        program.set1(c"u_t", veiled);
        program.set4(c"u_plate", plate_x, plate_y, self.lens_plate.t.w as f32, self.lens_plate.t.h as f32);
        program.set3(c"u_tone", [if plain { 1.0 } else { self.tune.saturate }, if plain { 1.0 } else { self.tune.brighten }, self.tune.tint]);
    }

    fn draw_bar(&self, bar_y: f32, veiled: f32, now: u32) {
        let (px, py) = (BAR_X - PLATE_MARGIN, bar_y - PLATE_MARGIN);
        let (pw, ph) = (BAR_W + 2.0 * PLATE_MARGIN, BAR_H + 2.0 * PLATE_MARGIN);
        let inset = self.baked_bezel.max(22.0) + 1.0;
        let (ix, iy, iw, ih) = (BAR_X + inset, bar_y + inset, BAR_W - 2.0 * inset, BAR_H - 2.0 * inset);

        unsafe { glDisable(GL_BLEND) }
        self.bar_uniforms(&self.p.bar_middle, px, py, veiled);
        self.gfx.quad(&self.p.bar_middle, ix, iy, iw, ih);

        unsafe { glEnable(GL_BLEND) }
        let rim = &self.p.bar_rim;
        self.bar_uniforms(rim, px, py, veiled);
        rim.bind(3, c"u_lens", self.lens_plate.t.tex);
        rim.set2(c"u_bend", if self.tune.mode() == LIQUID { self.tune.bend } else { 0.0 }, self.tune.fringe);
        self.gfx.quad(rim, px, py, pw, iy - py);
        self.gfx.quad(rim, px, iy + ih, pw, py + ph - iy - ih);
        self.gfx.quad(rim, px, iy, ix - px, ih);
        self.gfx.quad(rim, ix + iw, iy, px + pw - ix - iw, ih);

        let glow = self.shine_alpha.at(now);
        if glow > 0.0 {
            let reach = BAR_W / 2.0 - BAR_RADIUS - 130.0;
            let x = (self.shine_x.at(now) - DESIGN_W / 2.0).clamp(-reach, reach) + DESIGN_W / 2.0 - 130.0;
            self.p.shine.activate();
            self.p.shine.set4(c"u_box", x, bar_y, 260.0, 28.0);
            self.p.shine.set1(c"u_alpha", glow);
            self.gfx.quad(&self.p.shine, x, bar_y, 260.0, 28.0);
        }
    }

    fn draw_scene(&self, now: u32) {
        if !self.text_sheet.is_empty() {
            return self.draw_text_sheet();
        }
        let shift = self.slide.at(now) * LOWER_SHIFT;
        let veiled = self.veil_fade.at(now);
        let grid_drop = self.drop.at(now) * GRID_DROP;
        let left_out = |part: u32| self.leave_out & part != 0;

        unsafe {
            if !left_out(NO_CLEAR) {
                glClear(GL_COLOR_BUFFER_BIT);
            }
            glDisable(GL_BLEND);
        }
        if left_out(NO_BACKDROP) {
        } else if veiled <= 0.0 {
            self.copy(self.sharp.t, None, 1.0, 1.0, 0.0, 0.0);
        } else if veiled >= 1.0 {
            self.copy(self.veil.t, None, 1.0, 1.0, 0.0, 0.0);
        } else {
            self.p.backdrop.bind(0, c"u_wall", self.sharp.t.tex);
            self.p.backdrop.bind(1, c"u_veil", self.veil.t.tex);
            self.p.backdrop.set1(c"u_t", veiled);
            self.gfx.quad(&self.p.backdrop, 0.0, 0.0, DESIGN_W, DESIGN_H);
        }

        unsafe {
            glEnable(GL_BLEND);
            glBlendFuncSeparate(GL_ONE, GL_ONE_MINUS_SRC_ALPHA, GL_ZERO, GL_ONE);
        }
        if !left_out(NO_GRID) {
            for row in (1..ROWS).rev() {
                self.draw_row(row, shift, grid_drop, now);
            }
        }
        if !left_out(NO_BAR) {
            self.draw_bar(BAR_Y - shift, veiled, now);
        }
        unsafe { glEnable(GL_BLEND) }
        if !left_out(NO_BAR_TILES) {
            self.draw_row(0, shift, grid_drop, now);
        }

        if self.tune.hud != 0.0 && self.hud.tex != 0 && !left_out(NO_HUD) {
            let (hud_w, hud_h) = (self.hud.w as f32, self.hud.h as f32);
            let hud_y = if self.focus_row > 0 { DESIGN_H - 68.0 - hud_h } else { 40.0 };
            self.p.fill.activate();
            self.p.fill.set4(c"u_box", 40.0, hud_y, hud_w + 40.0, hud_h + 28.0);
            self.p.fill.set1(c"u_radius", 18.0);
            self.p.fill.set4(c"u_color", 0.0, 0.0, 0.0, 0.55);
            self.gfx.quad(&self.p.fill, 40.0, hud_y, hud_w + 40.0, hud_h + 28.0);
            self.p.text.bind(0, c"u_tex", self.hud.tex);
            self.p.text.set4(c"u_box", 60.0, hud_y + 14.0, hud_w, hud_h);
            self.gfx.quad(&self.p.text, 60.0, hud_y + 14.0, hud_w, hud_h);
        }
    }

    fn anything_moving(&self, now: u32) -> bool {
        [&self.slide, &self.veil_fade, &self.drop, &self.shine_x, &self.shine_alpha].iter().any(|a| a.busy(now)) || self.tiles.iter().any(|t| t.focus.busy(now))
    }

    fn focus_on(&mut self, row: usize, col: usize, now: u32, at_once: bool) {
        let (was_lower, lower) = (self.focus_row > 0, row > 0);
        let (old, next) = (self.focus_row * COLS + self.focus_col, row * COLS + col);
        let slid = if lower { 1.0 } else { 0.0 };
        let glow = if row > 0 { 0.0 } else { 1.0 };
        self.focus_row = row;
        self.focus_col = col;
        self.tiles[old].focus.go(0.0, self.tune.focus_ms, &EASE_OUT, now);
        self.tiles[next].focus.go(1.0, self.tune.focus_ms, &EASE_OUT, now);
        if row == 0 {
            self.shine_x.go(tile_centre(col), 260.0, &EASE_SCROLL, now);
        }
        self.shine_alpha.go(glow, 200.0, &EASE, now);
        if lower != was_lower {
            self.slide.go(slid, if lower { self.tune.down_ms } else { self.tune.up_ms }, if lower { &EASE_DOWN } else { &EASE_UP }, now);
            self.veil_fade.go(slid, if lower { self.tune.veil_down_ms } else { self.tune.veil_up_ms }, &EASE_VEIL, now);
            self.drop.go(slid, 200.0, &EASE_OUT, now);
        }
        if at_once {
            self.tiles[old].focus.set(0.0);
            self.tiles[next].focus.set(1.0);
            self.shine_x.set(tile_centre(col));
            self.shine_alpha.set(glow);
            self.slide.set(slid);
            self.veil_fade.set(slid);
            self.drop.set(slid);
        }
    }

    fn move_begins(&mut self, name: &'static str, now: u32) {
        if self.mv.active {
            return;
        }
        self.mv = Move { active: true, name, started: now, last: now, ..Move::default() };
    }

    fn move_ends(&mut self, now: u32) {
        if self.mv.frames == 0 {
            self.mv.active = false;
            return;
        }
        let m = &self.mv;
        self.last_move = format!(
            "{}: {} frames of {}, {} late, slowest {} ms, draw {:.1} ms (worst {:.1})",
            m.name,
            m.frames,
            (now.wrapping_sub(m.started) as f32 / self.refresh_ms + 0.5) as i32,
            m.late,
            m.slowest,
            m.draw_ms / m.frames as f64,
            m.draw_worst
        );
        note!("{}, {}", MODE_NAMES[self.tune.mode()], self.last_move);
        self.mv.active = false;
        self.draw_hud_text();
    }

    fn bench_case(&mut self, window: *mut SDL_Window, name: &str, without: u32, slid: f32, interval: c_int, finish: bool) {
        const FRAMES: u32 = 45;
        self.leave_out = without;
        self.slide.set(slid);
        self.veil_fade.set(slid);
        self.drop.set(slid);
        unsafe { SDL_GL_SetSwapInterval(interval) };
        let per_ms = ms_per_count();
        let (mut draw, mut worst) = (0.0f64, 0.0f64);
        for _ in 0..3 {
            self.draw_scene(0);
            unsafe { SDL_GL_SwapWindow(window) }
        }
        let began = counter();
        for _ in 0..FRAMES {
            let from = counter();
            self.draw_scene(0);
            if finish {
                unsafe { glFinish() }
            }
            let ms = (counter() - from) as f64 * per_ms;
            draw += ms;
            worst = worst.max(ms);
            unsafe { SDL_GL_SwapWindow(window) }
        }
        unsafe { glFinish() }
        let frame = (counter() - began) as f64 * per_ms / FRAMES as f64;
        note!("bench {:<34} swap wait {}, finish {}: {:.1} ms a frame, draw {:.1} ms (worst {:.1})", name, interval, finish as i32, frame, draw / FRAMES as f64, worst);
    }

    fn bench(&mut self, window: *mut SDL_Window) {
        const BACKDROP_ONLY: u32 = NO_GRID | NO_BAR | NO_BAR_TILES | NO_HUD;
        let cases: [(&str, u32, f32); 9] = [
            ("nothing but the clear", NO_BACKDROP | BACKDROP_ONLY, 0.0),
            ("wallpaper only", BACKDROP_ONLY, 0.0),
            ("wallpaper only, no clear", NO_CLEAR | BACKDROP_ONLY, 0.0),
            ("wallpaper and bar", NO_GRID | NO_BAR_TILES | NO_HUD, 0.0),
            ("wallpaper, bar and its tiles", NO_GRID | NO_HUD, 0.0),
            ("all, at rest", 0, 0.0),
            ("mid slide: two pictures mixed, only", BACKDROP_ONLY, 0.5),
            ("all, mid slide", 0, 0.5),
            ("all, down in the grid", 0, 1.0),
        ];
        for (name, without, slid) in cases {
            self.bench_case(window, name, without, slid, 0, true);
        }
        self.bench_case(window, "all, at rest", 0, 0.0, 0, false);
        self.bench_case(window, "all, at rest", 0, 0.0, 1, false);
        self.bench_case(window, "all, at rest", 0, 0.0, 1, true);
        self.bench_case(window, "wallpaper only", BACKDROP_ONLY, 0.0, 1, false);
        self.bench_case(window, "all, mid slide", 0, 0.5, 1, false);
        self.leave_out = 0;
        unsafe { SDL_GL_SetSwapInterval(1) };
        self.focus_on(0, 0, ticks(), true);
    }

    fn save_shot(&mut self, path: &str, args: &[String]) -> bool {
        let number = |at: usize| args.get(at).and_then(|a| a.parse::<i64>().ok()).unwrap_or(0);
        self.focus_on(number(3).rem_euclid(ROWS as i64) as usize, number(4).rem_euclid(COLS as i64) as usize, 0, true);
        if args.len() > 5 {
            self.tune.mode = number(5).rem_euclid(MODE_NAMES.len() as i64) as f32;
        }
        if let Some(slid) = args.get(6) {
            let slid: f32 = slid.parse().unwrap_or(0.0);
            self.slide.set(slid);
            self.veil_fade.set(slid);
        }
        self.draw_hud_text();
        self.draw_scene(0);
        unsafe { glFinish() }
        let mut shot = self.gfx.read_screen();
        let zoom: Vec<c_int> = std::env::var("LUMEN_SHOT_ZOOM").map(|z| z.split(',').filter_map(|n| n.trim().parse().ok()).collect()).unwrap_or_default();
        if !shot.is_null() && zoom.len() == 4 {
            let crop = SDL_Rect { x: zoom[0], y: zoom[1], w: zoom[2], h: zoom[3] };
            unsafe {
                let close_up = SDL_CreateRGBSurfaceWithFormat(0, crop.w * 3, crop.h * 3, 32, SDL_PIXELFORMAT_RGBA32);
                SDL_SetSurfaceBlendMode(shot, SDL_BLENDMODE_NONE);
                SDL_UpperBlitScaled(shot, &crop, close_up, ptr::null_mut());
                shot = close_up;
            }
        }
        let saved = !shot.is_null() && unsafe { IMG_SavePNG(shot, c_path(path).as_ptr()) } == 0;
        println!("{} {}", if saved { "saved" } else { "could not save" }, path);
        saved
    }
}

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    let started_at = ticks();
    if std::env::var_os("EGL_PLATFORM").is_none() {
        std::env::set_var("EGL_PLATFORM", "wayland");
    }
    if std::env::var_os("XDG_RUNTIME_DIR").is_none() {
        std::env::set_var("XDG_RUNTIME_DIR", "/tmp/xdg");
    }
    util::init();
    let args: Vec<String> = std::env::args().collect();
    let shot_path = if args.len() > 2 && args[1] == "--shot" { Some(args[2].clone()) } else { None };

    let app_dir;
    unsafe {
        lumen_abi_checked();
        SDL_SetHint(c"SDL_WEBOS_ACCESS_POLICY_KEYS_BACK".as_ptr(), c"true".as_ptr());
        if SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS) != 0 {
            note!("SDL_Init: {}", sdl_error());
            return 1;
        }
        IMG_Init(IMG_INIT_JPG | IMG_INIT_PNG);
        TTF_Init();
        let base = SDL_GetBasePath();
        app_dir = if base.is_null() {
            ".".to_string()
        } else {
            let dir = CStr::from_ptr(base).to_string_lossy().into_owned();
            SDL_free(base.cast());
            if dir.len() > 1 { dir.trim_end_matches('/').to_string() } else { dir }
        };
    }
    let mut tune = Tune::defaults();
    tune.read();

    let window;
    let context;
    let (mut screen_w, mut screen_h) = (0, 0);
    unsafe {
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_ES);
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 2);
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 0);
        SDL_GL_SetAttribute(SDL_GL_RED_SIZE, 8);
        SDL_GL_SetAttribute(SDL_GL_GREEN_SIZE, 8);
        SDL_GL_SetAttribute(SDL_GL_BLUE_SIZE, 8);
        SDL_GL_SetAttribute(SDL_GL_ALPHA_SIZE, 8);
        SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
        if tune.no_depth != 0.0 {
            SDL_GL_SetAttribute(SDL_GL_DEPTH_SIZE, 0);
            SDL_GL_SetAttribute(SDL_GL_STENCIL_SIZE, 0);
        }
        #[cfg(feature = "webos")]
        {
            let mut display = SDL_DisplayMode { format: 0, w: 0, h: 0, refresh_rate: 0, driverdata: ptr::null_mut() };
            if SDL_GetCurrentDisplayMode(0, &mut display) != 0 || display.w == 0 {
                display.w = DESIGN_W as c_int;
                display.h = DESIGN_H as c_int;
            }
            window = SDL_CreateWindow(c"Lumen Rust Test".as_ptr(), 0, 0, display.w, display.h, SDL_WINDOW_OPENGL | SDL_WINDOW_FULLSCREEN | SDL_WINDOW_SHOWN);
        }
        #[cfg(not(feature = "webos"))]
        {
            window = SDL_CreateWindow(c"Lumen Rust Test".as_ptr(), 0, 0, DESIGN_W as c_int, DESIGN_H as c_int, SDL_WINDOW_OPENGL | SDL_WINDOW_SHOWN);
        }
        if window.is_null() {
            note!("window: {}", sdl_error());
            return 1;
        }
        #[cfg(feature = "webos")]
        match lumen_claim_window(window) {
            1 => note!("window: no wayland info"),
            2 => note!("window: no wl_webos_shell"),
            _ => {}
        }
        context = SDL_GL_CreateContext(window);
        if context.is_null() {
            note!("context: {}", sdl_error());
            return 1;
        }
        SDL_GL_SetSwapInterval(1);
        SDL_GL_GetDrawableSize(window, &mut screen_w, &mut screen_h);
    }
    step("window and context");
    note!("open: {}x{}, {}, {}, {} ms after start", screen_w, screen_h, gl_string(GL_RENDERER), gl_string(GL_VERSION), ticks().wrapping_sub(started_at));

    let mut app = App {
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
            fill: Program::new("fill", shaders::FILL_FRAGMENT),
            text: Program::new("text", shaders::TEXT_FRAGMENT),
        },
        tune,
        app_dir,
        screen_w,
        screen_h,
        sharp: Target::default(),
        glass: Target::default(),
        veil: Target::default(),
        lens_plate: Target::default(),
        gloss_plate: Target::default(),
        baked_bezel: 0.0,
        refresh_ms: 16.7,
        tiles: Vec::with_capacity(TILES),
        hud: Tex::default(),
        font: ptr::null_mut(),
        slide: Anim::still(0.0),
        veil_fade: Anim::still(0.0),
        drop: Anim::still(0.0),
        shine_x: Anim::still(0.0),
        shine_alpha: Anim::still(0.0),
        focus_row: 0,
        focus_col: 0,
        last_move: "Down and Up slide, Left and Right move, OK changes the glass, Back closes".to_string(),
        leave_out: 0,
        mv: Move::default(),
        text_sheet: Vec::new(),
    };
    step("eleven programs");
    if !app.build_backdrops() {
        return 1;
    }
    step("wallpaper at screen size and its two blurred copies");
    app.load_tiles();
    step("18 icons");
    app.bake_tiles();
    step("18 resting tiles");
    app.bake_bar();
    step("the bar's two pictures");
    app.open_font();
    if app.tune.text != 0.0 || std::env::var_os("LUMEN_TEXT").is_some() {
        app.build_text_sheet();
    }
    app.shine_x.set(tile_centre(0));
    app.focus_on(0, 0, ticks(), true);

    if let Some(path) = shot_path {
        return if app.save_shot(&path, &args) { 0 } else { 1 };
    }

    app.draw_hud_text();
    app.refresh_ms = measured_refresh(window);
    step("24 frames swapped");
    app.draw_scene(0);
    unsafe { SDL_GL_SwapWindow(window) }
    step("the scene drawn once");
    if std::path::Path::new(BENCH_PATH).exists() {
        app.bench(window);
    }
    note!("first frame ready {} ms after start, RSS {} kB", ticks().wrapping_sub(started_at), status_kb("VmRSS"));
    let (mut redraw, mut running) = (true, true);
    let per_ms = ms_per_count();
    #[cfg(feature = "webos")]
    let mut probe = probe::Probe::open("com.samrenfrew.lumen.rtest");

    while running {
        let mut e = SDL_Event { padding: [0; 7] };
        while unsafe { SDL_PollEvent(&mut e) } != 0 {
            let kind = unsafe { e.kind };
            if kind == SDL_QUIT {
                note!("close: quit");
                running = false;
            }
            if kind == SDL_WINDOWEVENT {
                note!("window event {}", unsafe { e.window.event });
                redraw = true;
            }
            if kind != SDL_KEYDOWN || unsafe { e.key.repeat } != 0 {
                continue;
            }
            let (key, scancode) = unsafe { (e.key.keysym.sym, e.key.keysym.scancode) };
            let now = ticks();
            let blur_before = app.tune.blur_strengths();
            let wallpaper_before = app.tune.wallpaper.clone();
            let mode_before = app.tune.mode();
            app.tune.read();
            if blur_before != app.tune.blur_strengths() || wallpaper_before != app.tune.wallpaper {
                app.build_backdrops();
            }
            if app.tune.bezel != app.baked_bezel {
                app.bake_bar();
            }
            redraw = true;
            let (row, col) = (app.focus_row, app.focus_col);
            if closes(key, scancode) {
                note!("close: key {}, scancode {}", key, scancode);
                running = false;
            } else if key == SDLK_LEFT && col > 0 {
                app.move_begins("Left", now);
                app.focus_on(row, col - 1, now, false);
            } else if key == SDLK_RIGHT && col < COLS - 1 {
                app.move_begins("Right", now);
                app.focus_on(row, col + 1, now, false);
            } else if key == SDLK_DOWN && row < ROWS - 1 {
                app.move_begins("Down", now);
                app.focus_on(row + 1, col, now, false);
            } else if key == SDLK_UP && row > 0 {
                app.move_begins("Up", now);
                app.focus_on(row - 1, col, now, false);
            } else if key == SDLK_RETURN || key == SDLK_KP_ENTER {
                app.tune.mode = ((app.tune.mode() + 1) % MODE_NAMES.len()) as f32;
            } else {
                note!("key {}, scancode {}", key, scancode);
            }
            if app.tune.mode() != mode_before {
                app.draw_hud_text();
            }
        }
        if !running {
            break;
        }
        #[cfg(feature = "webos")]
        probe.tick();

        let now = ticks();
        let moving = app.anything_moving(now);
        if !moving && app.mv.active {
            app.move_ends(now);
            redraw = true;
        }
        if !moving && !redraw {
            unsafe { SDL_WaitEventTimeout(ptr::null_mut(), IDLE_WAIT_MS) };
            continue;
        }
        redraw = false;

        let draw_from = counter();
        app.draw_scene(now);
        if app.tune.finish != 0.0 {
            unsafe { glFinish() }
        }
        let draw_ms = (counter() - draw_from) as f64 * per_ms;
        unsafe { SDL_GL_SwapWindow(window) }

        let after = ticks();
        if app.mv.active {
            let gap = after.wrapping_sub(app.mv.last);
            app.mv.frames += 1;
            if gap as f32 > app.refresh_ms * 1.5 {
                app.mv.late += 1;
            }
            app.mv.slowest = app.mv.slowest.max(gap);
            app.mv.draw_ms += draw_ms;
            app.mv.draw_worst = app.mv.draw_worst.max(draw_ms);
            app.mv.last = after;
        }
    }

    note!("RSS {} kB, peak {} kB", status_kb("VmRSS"), status_kb("VmHWM"));
    unsafe {
        SDL_GL_DeleteContext(context);
        SDL_DestroyWindow(window);
        TTF_Quit();
        IMG_Quit();
        SDL_Quit();
    }
    0
}

fn measured_refresh(window: *mut SDL_Window) -> f32 {
    const SWAPS: u32 = 20;
    let swap = || unsafe {
        glClear(GL_COLOR_BUFFER_BIT);
        SDL_GL_SwapWindow(window);
    };
    for _ in 0..4 {
        swap();
    }
    let began = counter();
    for _ in 0..SWAPS {
        swap();
    }
    let ms = ((counter() - began) as f64 * ms_per_count() / SWAPS as f64) as f32;
    note!("refresh: a frame every {:.2} ms", ms);
    if ms > 5.0 && ms < 50.0 { ms } else { 16.7 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_curve_starts_at_zero_ends_at_one_and_never_goes_back() {
        for curve in [&EASE_UP, &EASE_DOWN, &EASE_VEIL, &EASE_OUT, &EASE_SCROLL, &EASE] {
            assert_eq!(eased(curve, 0.0), 0.0);
            assert_eq!(eased(curve, 1.0), 1.0);
            let mut last = 0.0;
            for step in 1..=100 {
                let at = eased(curve, step as f32 / 100.0);
                assert!(at >= last - 1e-4, "{curve:?} goes back at {step}");
                last = at;
            }
        }
    }

    #[test]
    fn an_animation_turned_round_mid_way_carries_on_from_where_it_was() {
        let mut slide = Anim::still(0.0);
        slide.go(1.0, 600.0, &EASE_DOWN, 1000);
        let mid_way = slide.at(1300);
        assert!(mid_way > 0.0 && mid_way < 1.0);
        slide.go(0.0, 600.0, &EASE_UP, 1300);
        assert_eq!(slide.at(1300), mid_way);
        assert!(slide.busy(1899));
        assert!(!slide.busy(1900));
        assert_eq!(slide.at(1900), 0.0);
    }

    #[test]
    fn tuning_takes_values_in_range_and_ignores_the_rest() {
        let mut tune = Tune::defaults();
        tune.apply("down=450\nup=99999\nbend=-20\nwallpaper=/tmp/a=b.jpg\nnonsense=3\nmode=2\r\n");
        assert_eq!(tune.down_ms, 450.0);
        assert_eq!(tune.up_ms, 600.0);
        assert_eq!(tune.bend, -20.0);
        assert_eq!(tune.wallpaper, "/tmp/a=b.jpg");
        assert_eq!(tune.mode(), NO_BLUR);
    }
}
