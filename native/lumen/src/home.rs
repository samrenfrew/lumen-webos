use crate::anim::{Anim, EASE, EASE_DOWN, EASE_OUT, EASE_SCROLL, EASE_UP, EASE_VEIL};
use crate::apps::background_of;
use crate::draw::{decoded, Decoded, Draw};
use crate::ffi::*;
use crate::gfx::{Tex, DESIGN_H, DESIGN_W};
use crate::note;
use crate::text::{inked, Shadow};
use serde_json::{json, Value};

pub const COLS: usize = 6;
// ponytail: three rows and no scrolling, as the test scene had; phase 2 brings the grid's scrolling
const ROWS_SHOWN: usize = 3;
const PLATE_MARGIN: f32 = 70.0;
const TILE_MARGIN: f32 = 28.0;
const TILE_W: f32 = 240.0;
const TILE_H: f32 = 144.0;
const TILE_GAP: f32 = 46.0;
const TILE_RADIUS: f32 = 34.0;
const ROWS_EDGE: f32 = 125.0;
const ROW_Y: [f32; ROWS_SHOWN] = [846.0, 1060.0, 1317.0];
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
const TILE_WITHOUT_COLOUR: [f32; 3] = [0.16, 0.17, 0.22];
const INK_ON_PHOTO: [f32; 3] = [0.933, 0.933, 0.933];
const TIME_PX: i32 = 80;
const TIME_LEFT: f32 = ROWS_EDGE - 6.0;
const TIME_BASELINE: f32 = 132.0;
const TIME_SHADOW: [Shadow; 1] = [Shadow { drop: 4, blur: 24, strength: 0.28 }];
const PERIOD_PX: i32 = 22;
const PERIOD_GAP: f32 = 7.0;
const DATE_PX: i32 = 26;
const DATE_BASELINE: f32 = 175.0;
const DATE_SPACING_EM: f32 = -0.012;
const STATUS_SHADOW: [Shadow; 2] = [Shadow { drop: 1, blur: 16, strength: 0.5 }, Shadow { drop: 0, blur: 2, strength: 0.22 }];
const MONOGRAM_PX: i32 = 92;

#[derive(Clone, PartialEq, Debug)]
pub struct Shown {
    pub id: String,
    pub title: String,
    pub icon_file: Option<String>,
    pub color: Option<[f32; 3]>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    Ok,
    Back,
}

struct Tile {
    app: Shown,
    bg: [f32; 3],
    focus: Anim,
}

#[derive(Default)]
struct Move {
    active: bool,
    frames: u32,
    started: u32,
    last: u32,
    slowest: u32,
    gaps: Vec<u32>,
    draw_ms: f64,
    draw_worst: f64,
    name: &'static str,
}

#[derive(Clone, Copy)]
struct StatusLine {
    picture: Tex,
    left: f32,
    baseline: f32,
    advance: f32,
}

pub struct Home {
    tiles: Vec<Tile>,
    bar_count: usize,
    focus: usize,
    sharp: Tex,
    glass: Tex,
    veil: Tex,
    blurs_made_with: [i32; 4],
    bar_made_with: f32,
    slide: Anim,
    veil_fade: Anim,
    drop: Anim,
    shine_x: Anim,
    shine_alpha: Anim,
    status_alpha: Anim,
    clock: (String, String, String),
    status_made: Vec<(String, StatusLine)>,
    mv: Move,
    pub last_move: String,
    pub refresh_ms: f32,
    pub reduce_motion: bool,
}

pub fn place(index: usize, bar_count: usize) -> (usize, usize) {
    if index < bar_count {
        (0, index)
    } else {
        (1 + (index - bar_count) / COLS, (index - bar_count) % COLS)
    }
}

fn first_of(row: usize, bar_count: usize) -> usize {
    if row == 0 {
        0
    } else {
        bar_count + (row - 1) * COLS
    }
}

fn row_len(row: usize, total: usize, bar_count: usize) -> usize {
    let first = first_of(row, bar_count);
    let width = if row == 0 { bar_count } else { COLS };
    total.saturating_sub(first).min(width)
}

pub fn moved(focus: usize, key: Key, total: usize, bar_count: usize) -> usize {
    if total == 0 {
        return 0;
    }
    let (row, col) = place(focus, bar_count);
    let to = |row: usize, col: usize| first_of(row, bar_count) + col.min(row_len(row, total, bar_count) - 1);
    match key {
        Key::Left if col > 0 => focus - 1,
        Key::Right if col + 1 < row_len(row, total, bar_count) => focus + 1,
        Key::Down if row + 1 < ROWS_SHOWN && row_len(row + 1, total, bar_count) > 0 => to(row + 1, col),
        Key::Up if row > 0 => to(row - 1, col),
        _ => focus,
    }
}

fn tile_left(col: usize) -> f32 {
    ROWS_EDGE + col as f32 * (TILE_W + TILE_GAP)
}

fn tile_centre(col: usize) -> f32 {
    tile_left(col) + TILE_W / 2.0
}

fn tile_key(id: &str) -> String {
    format!("tile:{id}")
}

fn icon_key(id: &str) -> String {
    format!("icon:{id}")
}

impl Home {
    pub fn new() -> Home {
        Home {
            tiles: Vec::new(),
            bar_count: 0,
            focus: 0,
            sharp: Tex::default(),
            glass: Tex::default(),
            veil: Tex::default(),
            blurs_made_with: [0; 4],
            bar_made_with: 0.0,
            slide: Anim::still(0.0),
            veil_fade: Anim::still(0.0),
            drop: Anim::still(0.0),
            shine_x: Anim::still(tile_centre(0)),
            shine_alpha: Anim::still(1.0),
            status_alpha: Anim::still(1.0),
            clock: Default::default(),
            status_made: Vec::new(),
            mv: Move::default(),
            last_move: String::new(),
            refresh_ms: 20.0,
            reduce_motion: false,
        }
    }

    fn ms(&self, ms: f32) -> f32 {
        if self.reduce_motion {
            1.0
        } else {
            ms
        }
    }

    fn blur_strengths(d: &Draw) -> [i32; 4] {
        [d.look.glass_halvings as i32, d.look.glass_passes as i32, d.look.veil_halvings as i32, d.look.veil_passes as i32]
    }

    pub fn set_wallpaper(&mut self, d: &mut Draw, wallpaper: &Decoded) {
        let mut raw = d.gfx.upload_rgba(&wallpaper.rgba, wallpaper.w, wallpaper.h);
        unsafe { glDisable(GL_BLEND) }
        d.gfx.free(&mut self.sharp);
        self.sharp = d.gfx.blank(d.gfx.screen_w, d.gfx.screen_h);
        d.gfx.stretched_over(&self.sharp);
        let shape = (raw.w as f32 / raw.h as f32) / (DESIGN_W / DESIGN_H);
        if shape > 1.0 {
            d.copy_whole(raw, 1.0 / shape, 1.0, (1.0 - 1.0 / shape) / 2.0, 0.0);
        } else {
            d.copy_whole(raw, 1.0, shape, 0.0, (1.0 - shape) / 2.0);
        }
        d.gfx.free(&mut raw);
        self.blur_wallpaper(d);
    }

    fn blur_wallpaper(&mut self, d: &mut Draw) {
        if self.sharp.tex == 0 {
            return;
        }
        unsafe { glDisable(GL_BLEND) }
        let mut half = d.gfx.blank(d.gfx.screen_w / 2, d.gfx.screen_h / 2);
        d.gfx.stretched_over(&half);
        d.copy_whole(self.sharp, 1.0, 1.0, 0.0, 0.0);
        let strengths = Self::blur_strengths(d);
        d.gfx.free(&mut self.glass);
        d.gfx.free(&mut self.veil);
        self.glass = d.blurred(half, strengths[0], strengths[1], 1.0);
        self.veil = d.blurred(half, strengths[2], strengths[3], 1.0 - VEIL_TINT);
        d.gfx.free(&mut half);
        d.gfx.into_screen();
        self.blurs_made_with = strengths;
    }

    pub fn backdrop_bytes(&self) -> usize {
        self.sharp.bytes() + self.glass.bytes() + self.veil.bytes()
    }

    pub fn look_changed(&mut self, d: &mut Draw) {
        if Self::blur_strengths(d) != self.blurs_made_with {
            self.blur_wallpaper(d);
        }
        if d.look.bezel != self.bar_made_with {
            d.let_go(|key| key.starts_with("bar:"));
        }
    }

    pub fn set_apps(&mut self, d: &mut Draw, apps: Vec<Shown>, first_row: usize, now: u32) {
        if apps.iter().eq(self.tiles.iter().map(|tile| &tile.app)) && self.bar_count == first_row.min(COLS).min(apps.len()) {
            return;
        }
        let focused_id = self.tiles.get(self.focus).map(|tile| tile.app.id.clone());
        d.let_go(|key| key.starts_with("tile:") || key.starts_with("icon:"));
        self.bar_count = first_row.min(COLS).min(apps.len());
        self.tiles = apps.into_iter().map(|app| Tile { bg: app.color.unwrap_or(TILE_WITHOUT_COLOUR), app, focus: Anim::still(0.0) }).collect();
        let keep = focused_id.and_then(|id| self.tiles.iter().position(|tile| tile.app.id == id)).filter(|&at| place(at, self.bar_count).0 < ROWS_SHOWN);
        self.focus = keep.unwrap_or(0);
        self.settle(now);
    }

    fn settle(&mut self, now: u32) {
        let lower = place(self.focus, self.bar_count).0 > 0;
        let slid = if lower { 1.0 } else { 0.0 };
        for (at, tile) in self.tiles.iter_mut().enumerate() {
            tile.focus.set(if at == self.focus { 1.0 } else { 0.0 });
        }
        self.slide.set(slid);
        self.veil_fade.set(slid);
        self.drop.set(slid);
        self.shine_x.set(tile_centre(place(self.focus, self.bar_count).1));
        self.shine_alpha.set(1.0 - slid);
        self.status_alpha.set(1.0 - slid);
        let _ = now;
    }

    pub fn set_clock(&mut self, d: &mut Draw, time: String, period: String, date: String) -> bool {
        let clock = (time, period, date);
        if clock == self.clock {
            return false;
        }
        self.clock = clock;
        d.let_go(|key| key.starts_with("status:"));
        true
    }

    fn focus_on(&mut self, d: &Draw, next: usize, now: u32) {
        let old = self.focus;
        let (was_lower, lower) = (place(old, self.bar_count).0 > 0, place(next, self.bar_count).0 > 0);
        let slid = if lower { 1.0 } else { 0.0 };
        let look = &d.look;
        self.focus = next;
        let focus_ms = self.ms(look.focus_ms);
        self.tiles[old].focus.go(0.0, focus_ms, &EASE_OUT, now);
        self.tiles[next].focus.go(1.0, focus_ms, &EASE_OUT, now);
        if !lower {
            self.shine_x.go(tile_centre(place(next, self.bar_count).1), self.ms(look.scroll_ms), &EASE_SCROLL, now);
        }
        self.shine_alpha.go(1.0 - slid, self.ms(200.0), &EASE, now);
        if lower != was_lower {
            self.slide.go(slid, self.ms(if lower { look.down_ms } else { look.up_ms }), if lower { &EASE_DOWN } else { &EASE_UP }, now);
            self.veil_fade.go(slid, self.ms(if lower { look.veil_down_ms } else { look.veil_up_ms }), &EASE_VEIL, now);
            self.drop.go(slid, self.ms(200.0), &EASE_OUT, now);
            self.status_alpha.go_after(self.ms(if lower { look.hide_top_wait_ms } else { look.show_top_wait_ms }), 1.0 - slid, self.ms(look.fade_ms), &EASE, now);
        }
    }

    pub fn key(&mut self, d: &Draw, key: Key, now: u32) -> Option<String> {
        if self.tiles.is_empty() {
            return None;
        }
        let next = match key {
            Key::Ok => return Some(self.tiles[self.focus].app.id.clone()),
            Key::Back => 0,
            _ => moved(self.focus, key, self.tiles.len(), self.bar_count),
        };
        if next != self.focus {
            if !self.mv.active {
                let name = match key {
                    Key::Left => "Left",
                    Key::Right => "Right",
                    Key::Up => "Up",
                    Key::Down => "Down",
                    _ => "Back",
                };
                self.mv = Move { active: true, name, started: now, last: now, ..Move::default() };
            }
            self.focus_on(d, next, now);
        }
        None
    }

    pub fn go_home(&mut self, d: &Draw, now: u32) {
        if !self.tiles.is_empty() && self.focus != 0 {
            self.focus_on(d, 0, now);
        }
    }

    pub fn moving(&self, now: u32) -> bool {
        [&self.slide, &self.veil_fade, &self.drop, &self.shine_x, &self.shine_alpha, &self.status_alpha].iter().any(|a| a.busy(now)) || self.tiles.iter().any(|t| t.focus.busy(now))
    }

    pub fn where_now(&self) -> Value {
        let focused: Vec<Value> = self
            .tiles
            .get(self.focus)
            .map(|tile| {
                let (row, col) = place(self.focus, self.bar_count);
                json!({"part": "tile", "label": tile.app.title, "id": tile.app.id, "index": col, "in": if row == 0 { "bar".to_string() } else { format!("row {row}") }})
            })
            .into_iter()
            .collect();
        json!({"screen": "home", "controls": false, "focused": focused})
    }

    pub fn hidden(&mut self, d: &mut Draw) {
        d.let_go(|key| !key.starts_with("icon:"));
    }

    fn monogram(d: &Draw, title: &str) -> Decoded {
        let size = TILE_H as usize;
        let mut rgba = vec![0u8; size * size * 4];
        let letter: String = title.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
        if let Some(line) = d.fonts.line(&letter, MONOGRAM_PX, "600", 0.0, "") {
            let (left, top) = ((size as i32 - line.advance) / 2 - line.left, (size as i32 + MONOGRAM_PX * 7 / 10) / 2 - line.baseline);
            for y in 0..line.h {
                for x in 0..line.w {
                    let (to_x, to_y) = (x + left, y + top);
                    if to_x < 0 || to_y < 0 || to_x >= size as i32 || to_y >= size as i32 {
                        continue;
                    }
                    let cover = line.rgba[((y * line.w + x) * 4 + 3) as usize];
                    rgba[(to_y as usize * size + to_x as usize) * 4..][..4].copy_from_slice(&[238, 238, 238, cover]);
                }
            }
        }
        Decoded { rgba, w: size as i32, h: size as i32 }
    }

    fn icon(&mut self, d: &mut Draw, at: usize) -> Tex {
        let key = icon_key(&self.tiles[at].app.id);
        if let Some(icon) = d.pictures.find(&key) {
            return icon;
        }
        let from_file = self.tiles[at].app.icon_file.as_deref().and_then(decoded);
        if let Some(icon) = &from_file {
            if let Some(background) = background_of(&icon.rgba, icon.w as usize, icon.h as usize) {
                self.tiles[at].bg = background;
            }
        }
        let icon = from_file.unwrap_or_else(|| Self::monogram(d, &self.tiles[at].app.title));
        let picture = d.gfx.upload_rgba(&icon.rgba, icon.w, icon.h);
        d.keep(&key, picture);
        picture
    }

    fn tile_uniforms(d: &Draw, tile: &Tile, icon: Tex, x: f32, y: f32, scale: f32, shadow: [f32; 3], focus: f32) {
        let p = &d.p.tile;
        p.bind(0, c"u_icon", icon.tex);
        p.set4(c"u_box", x, y, TILE_W * scale, TILE_H * scale);
        p.set1(c"u_radius", TILE_RADIUS * scale);
        p.set3(c"u_bg", tile.bg);
        p.set3(c"u_shadow", shadow);
        p.set1(c"u_focus", focus);
    }

    fn resting_tile(&mut self, d: &mut Draw, at: usize) -> Tex {
        let key = tile_key(&self.tiles[at].app.id);
        if let Some(resting) = d.pictures.find(&key) {
            return resting;
        }
        let icon = self.icon(d, at);
        let (w, h) = (TILE_W + 2.0 * TILE_MARGIN, TILE_H + 2.0 * TILE_MARGIN);
        let resting = d.gfx.blank(w as i32, h as i32);
        unsafe { glDisable(GL_BLEND) }
        d.gfx.at_design_scale_into(&resting);
        Self::tile_uniforms(d, &self.tiles[at], icon, TILE_MARGIN, TILE_MARGIN, 1.0, SHADOW_REST, 0.0);
        d.gfx.quad(&d.p.tile, 0.0, 0.0, w, h);
        d.keep(&key, resting);
        resting
    }

    fn bar_plates(&mut self, d: &mut Draw) -> (Tex, Tex) {
        if let (Some(lens), Some(gloss)) = (d.pictures.find("bar:lens"), d.pictures.find("bar:gloss")) {
            return (lens, gloss);
        }
        let (w, h) = (BAR_W + 2.0 * PLATE_MARGIN, BAR_H + 2.0 * PLATE_MARGIN);
        let (lens, gloss) = (d.gfx.blank(w as i32, h as i32), d.gfx.blank(w as i32, h as i32));
        unsafe { glDisable(GL_BLEND) }
        for (plate, program) in [(&lens, &d.p.lens), (&gloss, &d.p.gloss)] {
            d.gfx.at_design_scale_into(plate);
            program.activate();
            program.set4(c"u_bar", PLATE_MARGIN, PLATE_MARGIN, BAR_W, BAR_H);
            program.set1(c"u_radius", BAR_RADIUS);
            program.set1(c"u_bezel", d.look.bezel);
            d.gfx.quad(program, 0.0, 0.0, w, h);
        }
        self.bar_made_with = d.look.bezel;
        d.keep("bar:lens", lens);
        d.keep("bar:gloss", gloss);
        (lens, gloss)
    }

    fn status_line(&mut self, d: &mut Draw, part: &str, text: &str, px: i32, weight: &str, spacing_em: f32, feature: &str, shadows: &[Shadow]) -> Option<StatusLine> {
        if text.is_empty() {
            return None;
        }
        let key = format!("status:{part}:{text}");
        if let (Some(picture), Some((_, made))) = (d.pictures.find(&key), self.status_made.iter().find(|(made_key, _)| *made_key == key)) {
            return Some(StatusLine { picture, ..*made });
        }
        let line = d.fonts.line(text, px, weight, spacing_em * px as f32, feature)?;
        let shadowed = inked(&line, INK_ON_PHOTO, shadows);
        let made = StatusLine { picture: d.keep_line(&key, &shadowed), left: shadowed.left as f32, baseline: shadowed.baseline as f32, advance: shadowed.advance as f32 };
        self.status_made.retain(|(made_key, _)| !made_key.starts_with(&format!("status:{part}:")));
        self.status_made.push((key, made));
        Some(made)
    }

    fn draw_status_line(d: &Draw, line: &StatusLine, x: f32, baseline: f32, alpha: f32) {
        d.picture(line.picture, x - line.left, baseline - line.baseline, line.picture.w as f32, line.picture.h as f32, alpha);
    }

    fn draw_tile(&self, d: &Draw, at: usize, resting: Tex, icon: Tex, shift: f32, grid_drop: f32, now: u32) {
        let tile = &self.tiles[at];
        let (row, col) = place(at, self.bar_count);
        let f = tile.focus.at(now);
        let left = tile_left(col);
        let top = ROW_Y[row] - shift + if row > 0 { grid_drop } else { 0.0 };
        if f <= 0.0 {
            let (w, h) = (TILE_W + 2.0 * TILE_MARGIN, TILE_H + 2.0 * TILE_MARGIN);
            return d.picture(resting, left - TILE_MARGIN, top - TILE_MARGIN, w, h, 1.0);
        }
        let scale = 1.0 + ((if row > 0 { LIFT_GRID } else { LIFT_BAR }) - 1.0) * f;
        let (w, h) = (TILE_W * scale, TILE_H * scale);
        let x = left + TILE_W / 2.0 - w / 2.0;
        let y = top + TILE_H / 2.0 - LIFT_RISE * f * scale - h / 2.0;
        let lifted = if row > 0 { SHADOW_GRID } else { SHADOW_BAR };
        let shadow = [0, 1, 2].map(|i| SHADOW_REST[i] + (lifted[i] - SHADOW_REST[i]) * f);
        let margin = shadow[0] + shadow[1] + 4.0;
        Self::tile_uniforms(d, tile, icon, x, y, scale, shadow, f);
        d.gfx.quad(&d.p.tile, x - margin, y - margin, w + 2.0 * margin, h + 2.0 * margin);
    }

    fn bar_uniforms(&self, d: &Draw, program: &crate::gfx::Program, gloss: Tex, plate: (f32, f32, f32, f32), veiled: f32) {
        program.bind(0, c"u_glass", self.glass.tex);
        program.bind(1, c"u_veil", self.veil.tex);
        program.bind(2, c"u_gloss", gloss.tex);
        program.set1(c"u_t", veiled);
        program.set4(c"u_plate", plate.0, plate.1, plate.2, plate.3);
        program.set3(c"u_tone", [d.look.saturate, d.look.brighten, d.look.tint]);
    }

    fn draw_bar(&self, d: &Draw, lens: Tex, gloss: Tex, bar_y: f32, veiled: f32, now: u32) {
        let (px, py) = (BAR_X - PLATE_MARGIN, bar_y - PLATE_MARGIN);
        let (pw, ph) = (BAR_W + 2.0 * PLATE_MARGIN, BAR_H + 2.0 * PLATE_MARGIN);
        let inset = self.bar_made_with.max(22.0) + 1.0;
        let (ix, iy, iw, ih) = (BAR_X + inset, bar_y + inset, BAR_W - 2.0 * inset, BAR_H - 2.0 * inset);

        unsafe { glDisable(GL_BLEND) }
        self.bar_uniforms(d, &d.p.bar_middle, gloss, (px, py, pw, ph), veiled);
        d.gfx.quad(&d.p.bar_middle, ix, iy, iw, ih);

        unsafe { glEnable(GL_BLEND) }
        let rim = &d.p.bar_rim;
        self.bar_uniforms(d, rim, gloss, (px, py, pw, ph), veiled);
        rim.bind(3, c"u_lens", lens.tex);
        rim.set2(c"u_bend", d.look.bend, d.look.fringe);
        d.gfx.quad(rim, px, py, pw, iy - py);
        d.gfx.quad(rim, px, iy + ih, pw, py + ph - iy - ih);
        d.gfx.quad(rim, px, iy, ix - px, ih);
        d.gfx.quad(rim, ix + iw, iy, px + pw - ix - iw, ih);

        let glow = self.shine_alpha.at(now);
        if glow > 0.0 {
            let reach = BAR_W / 2.0 - BAR_RADIUS - 130.0;
            let x = (self.shine_x.at(now) - DESIGN_W / 2.0).clamp(-reach, reach) + DESIGN_W / 2.0 - 130.0;
            d.p.shine.activate();
            d.p.shine.set4(c"u_box", x, bar_y, 260.0, 28.0);
            d.p.shine.set1(c"u_alpha", glow);
            d.gfx.quad(&d.p.shine, x, bar_y, 260.0, 28.0);
        }
    }

    pub fn draw(&mut self, d: &mut Draw, now: u32) {
        d.pictures.next_frame();
        let shift = self.slide.at(now) * LOWER_SHIFT;
        let veiled = self.veil_fade.at(now);
        let grid_drop = self.drop.at(now) * GRID_DROP;
        let status_alpha = self.status_alpha.at(now);

        let on_screen = |at: usize, bar_count: usize| {
            let row = place(at, bar_count).0;
            let top = ROW_Y[row.min(ROWS_SHOWN - 1)] - shift;
            row < ROWS_SHOWN && top - 80.0 <= DESIGN_H && top + TILE_H + 80.0 >= 0.0
        };
        let mut pictures: Vec<Option<(Tex, Tex)>> = Vec::with_capacity(self.tiles.len());
        for at in 0..self.tiles.len() {
            pictures.push(if on_screen(at, self.bar_count) { Some((self.resting_tile(d, at), self.icon(d, at))) } else { None });
        }
        let (lens, gloss) = self.bar_plates(d);
        let sf = d.fonts.is_sf("600-clock");
        let (time, period, date) = self.clock.clone();
        let status = if status_alpha > 0.0 {
            [
                self.status_line(d, "time", &time, TIME_PX, "600-clock", 0.0, if sf { "case" } else { "tnum" }, &TIME_SHADOW),
                self.status_line(d, "period", &period, PERIOD_PX, "400", DATE_SPACING_EM, "", &TIME_SHADOW),
                self.status_line(d, "date", &date, DATE_PX, "500", DATE_SPACING_EM, "", &STATUS_SHADOW),
            ]
        } else {
            [None, None, None]
        };

        d.gfx.into_screen();
        unsafe {
            glClearColor(0.0, 0.0, 0.0, 1.0);
            glClear(GL_COLOR_BUFFER_BIT);
            glDisable(GL_BLEND);
        }
        if self.sharp.tex == 0 {
        } else if veiled <= 0.0 {
            d.copy_whole(self.sharp, 1.0, 1.0, 0.0, 0.0);
        } else if veiled >= 1.0 {
            d.copy_whole(self.veil, 1.0, 1.0, 0.0, 0.0);
        } else {
            d.p.backdrop.bind(0, c"u_wall", self.sharp.tex);
            d.p.backdrop.bind(1, c"u_veil", self.veil.tex);
            d.p.backdrop.set1(c"u_t", veiled);
            d.gfx.quad(&d.p.backdrop, 0.0, 0.0, DESIGN_W, DESIGN_H);
        }

        unsafe {
            glEnable(GL_BLEND);
            glBlendFuncSeparate(GL_ONE, GL_ONE_MINUS_SRC_ALPHA, GL_ZERO, GL_ONE);
        }
        let focus_row = place(self.focus, self.bar_count).0;
        let draw_row = |row: usize| {
            let in_row = |at: &usize| place(*at, self.bar_count).0 == row;
            for at in (0..self.tiles.len()).filter(in_row).filter(|&at| at != self.focus).chain((row == focus_row).then_some(self.focus)) {
                if let Some(Some((resting, icon))) = pictures.get(at) {
                    self.draw_tile(d, at, *resting, *icon, shift, grid_drop, now);
                }
            }
        };
        for row in (1..ROWS_SHOWN).rev() {
            draw_row(row);
        }
        if self.sharp.tex != 0 {
            self.draw_bar(d, lens, gloss, BAR_Y - shift, veiled, now);
        }
        unsafe { glEnable(GL_BLEND) }
        draw_row(0);

        let [time, period, date] = &status;
        if let Some(time) = time {
            Self::draw_status_line(d, time, TIME_LEFT, TIME_BASELINE, status_alpha);
            if let Some(period) = period {
                Self::draw_status_line(d, period, TIME_LEFT + time.advance + PERIOD_GAP, TIME_BASELINE, status_alpha);
            }
        }
        if let Some(date) = date {
            Self::draw_status_line(d, date, ROWS_EDGE, DATE_BASELINE, status_alpha);
        }
    }

    pub fn frame_shown(&mut self, after: u32, draw_ms: f64) {
        if !self.mv.active {
            return;
        }
        let gap = after.wrapping_sub(self.mv.last);
        if self.mv.frames > 0 {
            self.mv.gaps.push(gap);
        }
        self.mv.frames += 1;
        self.mv.slowest = self.mv.slowest.max(gap);
        self.mv.draw_ms += draw_ms;
        self.mv.draw_worst = self.mv.draw_worst.max(draw_ms);
        self.mv.last = after;
    }

    pub fn move_ended(&mut self, now: u32) -> bool {
        if !self.mv.active {
            return false;
        }
        let mut m = std::mem::take(&mut self.mv);
        if m.frames == 0 {
            return false;
        }
        if let Some(usual) = usual_gap(&mut m.gaps) {
            self.refresh_ms = usual;
        }
        let late = m.gaps.iter().filter(|&&gap| gap as f32 > self.refresh_ms * 1.5).count();
        self.last_move = format!(
            "{}: {} frames of {}, {} late, slowest {} ms, draw {:.1} ms (worst {:.1})",
            m.name,
            m.frames,
            (now.wrapping_sub(m.started) as f32 / self.refresh_ms + 0.5) as i32,
            late,
            m.slowest,
            m.draw_ms / m.frames as f64,
            m.draw_worst
        );
        note!("move {}", self.last_move);
        true
    }
}

pub fn usual_gap(gaps: &mut [u32]) -> Option<f32> {
    if gaps.len() < 5 {
        return None;
    }
    gaps.sort_unstable();
    let middle = gaps[gaps.len() / 2] as f32;
    (middle > 5.0 && middle < 50.0).then_some(middle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bar_holds_the_first_apps_and_the_rest_fill_rows_of_six() {
        assert_eq!(place(0, 6), (0, 0));
        assert_eq!(place(5, 6), (0, 5));
        assert_eq!(place(6, 6), (1, 0));
        assert_eq!(place(13, 6), (2, 1));
        assert_eq!(place(5, 5), (1, 0));
    }

    #[test]
    fn left_and_right_stop_at_the_ends_of_a_row() {
        assert_eq!(moved(0, Key::Left, 14, 6), 0);
        assert_eq!(moved(5, Key::Right, 14, 6), 5);
        assert_eq!(moved(13, Key::Right, 14, 6), 13);
        assert_eq!(moved(6, Key::Left, 14, 6), 6);
        assert_eq!(moved(7, Key::Right, 14, 6), 8);
    }

    #[test]
    fn down_into_a_short_row_lands_on_its_last_tile_and_up_comes_back_above_it() {
        assert_eq!(moved(4, Key::Down, 14, 6), 10);
        assert_eq!(moved(10, Key::Down, 14, 6), 13);
        assert_eq!(moved(13, Key::Up, 14, 6), 7);
        assert_eq!(moved(13, Key::Down, 14, 6), 13);
        assert_eq!(moved(2, Key::Up, 14, 6), 2);
    }

    #[test]
    fn down_does_nothing_when_every_app_is_on_the_bar_or_there_are_none() {
        assert_eq!(moved(3, Key::Down, 6, 6), 3);
        assert_eq!(moved(0, Key::Down, 0, 0), 0);
        assert_eq!(moved(0, Key::Right, 0, 0), 0);
    }

    #[test]
    fn the_screens_beat_is_the_usual_gap_between_frames_of_a_move_long_enough_to_tell() {
        assert_eq!(usual_gap(&mut [20, 21, 19, 20, 41, 20, 20]), Some(20.0));
        assert_eq!(usual_gap(&mut [16, 17, 17]), None);
        assert_eq!(usual_gap(&mut [200, 210, 190, 220, 205]), None);
    }

    #[test]
    fn rows_past_the_third_are_not_reached_until_the_grid_scrolls() {
        assert_eq!(moved(12, Key::Down, 30, 6), 12);
    }
}
