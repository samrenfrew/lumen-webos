use crate::anim::{Anim, EASE, EASE_DOWN, EASE_OUT, EASE_SCROLL, EASE_UP, EASE_VEIL};
use crate::apps::background_of;
use crate::draw::{decoded, Decoded, Draw, Glass, Style};
use crate::ffi::*;
use crate::gfx::{Tex, DESIGN_H, DESIGN_W};
use crate::settings::Forecast;
use crate::text::Shadow;
use crate::weather::{self, Weather};
use serde_json::{json, Value};

pub const COLS: usize = 6;
const PLATE_MARGIN: f32 = 70.0;
const TILE_MARGIN: f32 = 28.0;
const TILE_W: f32 = 240.0;
const TILE_H: f32 = 144.0;
const TILE_GAP: f32 = 46.0;
const TILE_RADIUS: f32 = 34.0;
const ROWS_EDGE: f32 = 125.0;
const BAR_TILES_TOP: f32 = 846.0;
const FIRST_GRID_TOP: f32 = 1060.0;
const GRID_ROW_STEP: f32 = 257.0;
const GRID_TOP: f32 = 76.0;
const GRID_BOTTOM: f32 = 780.0;
const BAR_X: f32 = 85.0;
const BAR_ABOVE_TILES: f32 = 40.0;
const BAR_W: f32 = 1750.0;
const BAR_H: f32 = 224.0;
const BAR_RADIUS: f32 = 74.0;
const GRID_DROP: f32 = 40.0;
const LIFT_BAR: f32 = 1.12;
const LIFT_GRID: f32 = 1.14;
const LIFT_RISE: f32 = 7.5;
const LIFT_MOVING: f32 = 1.06;
const MOVING_RISE: f32 = 10.0;
const WOBBLE_MS: f32 = 260.0;
const SHADOW_REST: [f32; 3] = [8.0, 16.0, 0.30];
const SHADOW_BAR: [f32; 3] = [12.0, 22.0, 0.38];
const SHADOW_GRID: [f32; 3] = [26.0, 40.0, 0.46];
const VEIL_TINT: f32 = 0.26;
const NIGHT_DIM: f32 = 0.45;
const WALLPAPER_FADE_MS: f32 = 450.0;
const TILE_WITHOUT_COLOUR: [f32; 3] = [0.16, 0.17, 0.22];
const FOLDER_COLOUR: [f32; 3] = [0.231, 0.259, 0.337];
const INK_ON_PHOTO: [f32; 3] = [0.933, 0.933, 0.933];
const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
const TIME_LEFT: f32 = ROWS_EDGE - 6.0;
const TIME_BASELINE: f32 = 132.0;
const PERIOD_GAP: f32 = 7.0;
const DATE_BASELINE: f32 = 175.0;
const WEATHER_BASELINE: f32 = 211.0;
const WEATHER_ICON_TOP: f32 = 185.0;
const MONOGRAM_PX: i32 = 92;
const BAR_LABEL_BELOW: f32 = 169.0;
const GRID_LABEL_BELOW: f32 = 191.5;
const GRID_LABEL_RISE: f32 = 4.5;
const PILL: f32 = 58.0;
const PILL_GAP: f32 = 16.0;
const PILL_TOP: f32 = 64.0;
const PILL_LIFT: f32 = 1.14;
const PILL_LABEL_BASELINE: f32 = 166.0;
const PILLS_RIGHT: f32 = DESIGN_W - ROWS_EDGE - PILL_GAP;
const CAPTION_RIGHT: f32 = 1779.0;
const CAPTION_TITLE_BASELINE: f32 = 770.0;
const CAPTION_CREDIT_BASELINE: f32 = 745.0;
const CAST_BOTTOM: f32 = 775.0;
const CAST_COL: f32 = 72.0;
const CAST_GAP: f32 = 6.0;
const CAST_ICON: f32 = 30.0;
const CAST_HOURS: usize = 6;
const RAIN_LIKELY: i32 = 50;
const MENU_W: f32 = 400.0;
const MENU_ITEM_H: f32 = 72.0;
const MENU_ITEM_STEP: f32 = 84.0;
const MENU_GAP: f32 = 24.0;
const MENU_EDGE: f32 = 60.0;
const MENU_LIFT: f32 = 1.03;
const FOLDER_PANEL_W: f32 = 760.0;
const FOLDER_PANEL_TOP: f32 = 250.0;
const FOLDER_TILE_W: f32 = 200.0;
const FOLDER_TILE_H: f32 = 120.0;
const FOLDER_TILE_GAP: f32 = 24.0;
const FOLDER_COLUMNS: usize = 3;
const FOLDER_ROW_STEP: f32 = 176.0;
const FOLDER_PRESETS: [&str; 3] = ["Games", "Tools", "Media"];
const TOAST_MS: u32 = 3500;
pub const HOLD_MS: u32 = 600;

const TIME_SHADOW: [Shadow; 1] = [Shadow { drop: 4, blur: 24, strength: 0.28 }];
const STATUS_SHADOW: [Shadow; 2] = [Shadow { drop: 1, blur: 16, strength: 0.5 }, Shadow { drop: 0, blur: 2, strength: 0.22 }];
const LABEL_SHADOW: [Shadow; 2] = [Shadow { drop: 2, blur: 14, strength: 0.55 }, Shadow { drop: 0, blur: 2, strength: 0.25 }];
const TEXT_SHADOW: [Shadow; 1] = [Shadow { drop: 2, blur: 10, strength: 0.8 }];
const CAPTION_SHADOW: [Shadow; 1] = [Shadow { drop: 2, blur: 14, strength: 0.6 }];
const GLASS_INK_SHADOW: [Shadow; 1] = [Shadow { drop: 1, blur: 8, strength: 0.35 }];

const TIME: Style = Style { px: 80, weight: "600-clock", spacing_em: 0.0, feature: "case", ink: INK_ON_PHOTO, shadows: &TIME_SHADOW };
const TIME_TNUM: Style = Style { feature: "tnum", ..TIME };
const PERIOD: Style = Style { px: 22, weight: "400", spacing_em: -0.012, feature: "", ink: INK_ON_PHOTO, shadows: &TIME_SHADOW };
const DATE: Style = Style { px: 26, weight: "500", spacing_em: -0.012, feature: "", ink: INK_ON_PHOTO, shadows: &STATUS_SHADOW };
const WEATHER_LINE: Style = Style { px: 26, weight: "500", spacing_em: -0.005, feature: "", ink: INK_ON_PHOTO, shadows: &STATUS_SHADOW };
const TILE_LABEL: Style = Style { px: 18, weight: "600", spacing_em: -0.015, feature: "", ink: INK_ON_PHOTO, shadows: &LABEL_SHADOW };
const PILL_LABEL: Style = Style { px: 22, weight: "500", spacing_em: 0.0, feature: "", ink: INK_ON_PHOTO, shadows: &TEXT_SHADOW };
const CAST_TEMP: Style = Style { px: 26, weight: "600", spacing_em: -0.011, feature: "", ink: WHITE, shadows: &STATUS_SHADOW };
const CAST_TIME: Style = Style { px: 18, weight: "400", spacing_em: -0.011, feature: "", ink: WHITE, shadows: &STATUS_SHADOW };
const CAPTION_TITLE: Style = Style { px: 18, weight: "500", spacing_em: 0.01, feature: "", ink: WHITE, shadows: &CAPTION_SHADOW };
const CAPTION_CREDIT: Style = Style { px: 15, weight: "400", spacing_em: 0.0, feature: "", ink: [0.86, 0.86, 0.86], shadows: &CAPTION_SHADOW };
const MENU_NAME: Style = Style { px: 22, weight: "600", spacing_em: 0.0, feature: "", ink: WHITE, shadows: &GLASS_INK_SHADOW };
const MENU_WARN: Style = Style { ink: [1.0, 0.36, 0.32], ..MENU_NAME };
const FOLDER_TITLE: Style = Style { px: 32, weight: "600", spacing_em: -0.02, feature: "", ink: INK_ON_PHOTO, shadows: &[] };
const FOLDER_LABEL: Style = Style { px: 22, weight: "600", spacing_em: -0.015, feature: "", ink: INK_ON_PHOTO, shadows: &LABEL_SHADOW };
const TOAST: Style = Style { px: 26, weight: "400", spacing_em: 0.0, feature: "", ink: [0.667, 0.667, 0.667], shadows: &[] };
const TOAST_ERROR: Style = Style { ink: [0.878, 0.337, 0.31], ..TOAST };

const SLIDERS: &str = "M4 7h8M16 7h4M4 17h4M12 17h8M12 4.5v5M8 14.5v5";
const GEAR: &str = "M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z";
const INPUTS: &str = "M3.5 7.5h17v7.2a2 2 0 0 1-2 2h-3.2l-1 2.3H9.7l-1-2.3H5.5a2 2 0 0 1-2-2zM8 10.5v2.4M12 10.5v2.4M16 10.5v2.4";
const TRASH: &str = "M4.5 7h15M9.5 7V4.5h5V7M6.5 7l1 12.5h9l1-12.5M10 10.5v6M14 10.5v6";
const MINUS: &str = "M12 3.5a8.5 8.5 0 1 1 0 17 8.5 8.5 0 0 1 0-17zM8 12h8";
const FOLDER: &str = "M3.5 7.5a2 2 0 0 1 2-2h4l2 2.5h7a2 2 0 0 1 2 2V17a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z";
pub const TOP_ITEMS: [(&str, &str, &str); 3] = [("controls", "Control Center", SLIDERS), ("settings", "Settings", GEAR), ("inputs", "Inputs", INPUTS)];

#[derive(Clone, PartialEq, Debug)]
pub struct Shown {
    pub id: String,
    pub title: String,
    pub icon_file: Option<String>,
    pub color: Option<[f32; 3]>,
    pub inside: Option<Vec<Shown>>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    Ok,
    Back,
    Digit(usize),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Action {
    Open(String),
    Top(&'static str),
    Move(String, isize),
    Remove(String),
    Uninstall(String),
    FolderAdd(String, String),
    FolderOut(String),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Zone {
    Rows,
    Top,
    Menu,
    Folder,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum MenuItem {
    Uninstall,
    Remove,
    AddToFolder,
    Into(usize),
    TakeOut,
}

struct Menu {
    from_folder: bool,
    folders: bool,
    confirm: bool,
    index: usize,
    anchor: Option<(f32, f32, f32)>,
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

pub struct Home {
    tiles: Vec<Tile>,
    cols: usize,
    bar_count: usize,
    focus: usize,
    engaged: bool,
    zone: Zone,
    top_index: usize,
    top_focus: [Anim; 3],
    moving: Option<String>,
    menu: Menu,
    folder: Option<(String, Vec<Tile>)>,
    folder_index: usize,
    sharp: Tex,
    fading_from: Tex,
    wallpaper_fade: Anim,
    pub night: bool,
    glass: Tex,
    veil: Tex,
    blurs_made_with: [i32; 4],
    bar_made_with: f32,
    shift: Anim,
    veil_fade: Anim,
    drop: Anim,
    shine_x: Anim,
    shine_alpha: Anim,
    status_alpha: Anim,
    panel_alpha: Anim,
    clock: (String, String, String),
    pub weather: Option<Weather>,
    pub weather_line: bool,
    pub forecast: Forecast,
    pub twenty_four_hour: bool,
    pub caption: Option<(String, String)>,
    pub hide_names: bool,
    pub unix_now: u64,
    toast: Option<(String, bool, u32)>,
    focus_changed: bool,
    mv: Move,
    pub last_move: String,
    pub refresh_ms: f32,
    pub reduce_motion: bool,
}

pub fn place(index: usize, bar_count: usize, cols: usize) -> (usize, usize) {
    if index < bar_count {
        (0, index)
    } else {
        (1 + (index - bar_count) / cols, (index - bar_count) % cols)
    }
}

fn first_of(row: usize, bar_count: usize, cols: usize) -> usize {
    if row == 0 {
        0
    } else {
        bar_count + (row - 1) * cols
    }
}

fn row_len(row: usize, total: usize, bar_count: usize, cols: usize) -> usize {
    let first = first_of(row, bar_count, cols);
    let width = if row == 0 { bar_count } else { cols };
    total.saturating_sub(first).min(width)
}

pub fn moved(focus: usize, key: Key, total: usize, bar_count: usize, cols: usize) -> usize {
    if total == 0 {
        return 0;
    }
    let (row, col) = place(focus, bar_count, cols);
    let to = |row: usize, col: usize| first_of(row, bar_count, cols) + col.min(row_len(row, total, bar_count, cols) - 1);
    match key {
        Key::Left if col > 0 => focus - 1,
        Key::Right if col + 1 < row_len(row, total, bar_count, cols) => focus + 1,
        Key::Down if row_len(row + 1, total, bar_count, cols) > 0 => to(row + 1, col),
        Key::Up if row > 0 => to(row - 1, col),
        _ => focus,
    }
}

pub fn row_top(row: usize) -> f32 {
    if row == 0 {
        BAR_TILES_TOP
    } else {
        FIRST_GRID_TOP + (row - 1) as f32 * GRID_ROW_STEP
    }
}

pub fn lower_shift(row: usize) -> f32 {
    if row == 0 {
        0.0
    } else {
        (BAR_TILES_TOP - GRID_TOP).max(row_top(row) - GRID_BOTTOM)
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

pub fn pill_left(at: usize) -> f32 {
    PILLS_RIGHT - (TOP_ITEMS.len() - at) as f32 * (PILL + PILL_GAP) + PILL_GAP
}

fn menu_items(from_folder: bool, folders: bool, can_uninstall: bool) -> Vec<MenuItem> {
    if from_folder {
        return vec![MenuItem::TakeOut];
    }
    if folders {
        return (0..FOLDER_PRESETS.len()).map(MenuItem::Into).collect();
    }
    let mut items = Vec::new();
    if can_uninstall {
        items.push(MenuItem::Uninstall);
    }
    items.extend([MenuItem::Remove, MenuItem::AddToFolder]);
    items
}

pub fn menu_place(anchor: Option<(f32, f32, f32)>, count: usize) -> (f32, f32) {
    let h = count as f32 * MENU_ITEM_STEP;
    let Some((x, top, bottom)) = anchor else { return ((DESIGN_W - MENU_W) / 2.0, 300.0) };
    let left = (x - MENU_W / 2.0).clamp(MENU_EDGE, DESIGN_W - MENU_EDGE - MENU_W);
    let above = top - MENU_GAP - h >= MENU_EDGE;
    (left, if above { top - MENU_GAP - h } else { bottom + MENU_GAP })
}

fn hour_text(t: &str, twenty_four_hour: bool) -> String {
    let hour: u32 = t.get(11..13).and_then(|h| h.parse().ok()).unwrap_or(0);
    if twenty_four_hour {
        format!("{hour:02}:00")
    } else {
        format!("{} {}", if hour % 12 == 0 { 12 } else { hour % 12 }, if hour < 12 { "AM" } else { "PM" })
    }
}

fn scaled_into(dst: &mut [u8], dst_w: usize, (x0, y0, w, h): (usize, usize, usize, usize), src: &Decoded, radius: f32) {
    let (sw, sh) = (src.w as f32, src.h as f32);
    let cover = (w as f32 / sw).max(h as f32 / sh);
    let (ox, oy) = ((sw * cover - w as f32) / 2.0, (sh * cover - h as f32) / 2.0);
    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 + 0.5 - w as f32 / 2.0).abs() - (w as f32 / 2.0 - radius);
            let dy = (y as f32 + 0.5 - h as f32 / 2.0).abs() - (h as f32 / 2.0 - radius);
            let outside = (dx.max(0.0).powi(2) + dy.max(0.0).powi(2)).sqrt() - radius;
            let cover_a = (0.5 - outside).clamp(0.0, 1.0);
            if cover_a <= 0.0 {
                continue;
            }
            let sx = (((x as f32 + ox) / cover) as i32).clamp(0, src.w - 1) as usize;
            let sy = (((y as f32 + oy) / cover) as i32).clamp(0, src.h - 1) as usize;
            let s = &src.rgba[(sy * src.w as usize + sx) * 4..][..4];
            let d = &mut dst[((y0 + y) * dst_w + x0 + x) * 4..][..4];
            let a = s[3] as f32 / 255.0 * cover_a;
            for c in 0..3 {
                d[c] = (s[c] as f32 * a + d[c] as f32 * (1.0 - a)) as u8;
            }
        }
    }
}

fn folder_picture(inside: &[Shown]) -> Decoded {
    let (w, h) = (TILE_W as usize, TILE_H as usize);
    let mut rgba: Vec<u8> = std::iter::repeat([59u8, 66, 86, 255]).take(w * h).flatten().collect();
    let (left, top, area_w, area_h) = (0.12 * TILE_W, 0.14 * TILE_H, 0.76 * TILE_W, 0.72 * TILE_H);
    let (mini_w, mini_h) = (0.47 * area_w, 0.47 * area_h);
    for (at, app) in inside.iter().take(4).enumerate() {
        let x = left + if at % 2 == 1 { area_w - mini_w } else { 0.0 };
        let y = top + if at >= 2 { area_h - mini_h } else { 0.0 };
        let mini = (x as usize, y as usize, mini_w as usize, mini_h as usize);
        let plate = Decoded { rgba: vec![255, 255, 255, 36], w: 1, h: 1 };
        scaled_into(&mut rgba, w, mini, &plate, 10.0);
        if let Some(icon) = app.icon_file.as_deref().and_then(decoded) {
            scaled_into(&mut rgba, w, mini, &icon, 10.0);
        }
    }
    Decoded { rgba, w: w as i32, h: h as i32 }
}

impl Home {
    pub fn new() -> Home {
        Home {
            tiles: Vec::new(),
            cols: COLS,
            bar_count: 0,
            focus: 0,
            engaged: false,
            zone: Zone::Rows,
            top_index: 0,
            top_focus: [Anim::still(0.0); 3],
            moving: None,
            menu: Menu { from_folder: false, folders: false, confirm: false, index: 0, anchor: None },
            folder: None,
            folder_index: 0,
            sharp: Tex::default(),
            fading_from: Tex::default(),
            wallpaper_fade: Anim::still(1.0),
            night: false,
            glass: Tex::default(),
            veil: Tex::default(),
            blurs_made_with: [0; 4],
            bar_made_with: 0.0,
            shift: Anim::still(0.0),
            veil_fade: Anim::still(0.0),
            drop: Anim::still(0.0),
            shine_x: Anim::still(tile_centre(0)),
            shine_alpha: Anim::still(0.0),
            status_alpha: Anim::still(1.0),
            panel_alpha: Anim::still(0.0),
            clock: Default::default(),
            weather: None,
            weather_line: true,
            forecast: Forecast::Off,
            twenty_four_hour: true,
            caption: None,
            hide_names: false,
            unix_now: 0,
            toast: None,
            focus_changed: false,
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

    fn place_of(&self, at: usize) -> (usize, usize) {
        place(at, self.bar_count, self.cols)
    }

    fn row_count(&self) -> usize {
        if self.tiles.len() <= self.bar_count {
            1
        } else {
            1 + (self.tiles.len() - self.bar_count).div_ceil(self.cols)
        }
    }

    fn blur_strengths(d: &Draw) -> [i32; 4] {
        [d.look.glass_halvings as i32, d.look.glass_passes as i32, d.look.veil_halvings as i32, d.look.veil_passes as i32]
    }

    pub fn set_wallpaper(&mut self, d: &mut Draw, wallpaper: &Decoded, fade: bool, now: u32) {
        let mut raw = d.gfx.upload_rgba(&wallpaper.rgba, wallpaper.w, wallpaper.h);
        unsafe { glDisable(GL_BLEND) }
        d.gfx.free(&mut self.fading_from);
        if fade && self.sharp.tex != 0 && !self.reduce_motion {
            self.fading_from = self.sharp;
            self.wallpaper_fade = Anim::still(0.0);
            self.wallpaper_fade.go(1.0, WALLPAPER_FADE_MS, &EASE, now);
        } else {
            d.gfx.free(&mut self.sharp);
            self.wallpaper_fade = Anim::still(1.0);
        }
        self.sharp = d.gfx.blank(d.gfx.screen_w, d.gfx.screen_h);
        d.gfx.stretched_over(&self.sharp);
        let shape = (raw.w as f32 / raw.h as f32) / (DESIGN_W / DESIGN_H);
        let mul = if self.night { 1.0 - NIGHT_DIM } else { 1.0 };
        if shape > 1.0 {
            d.copy_dimmed(raw, (1.0 / shape, 1.0, (1.0 - 1.0 / shape) / 2.0, 0.0), mul);
        } else {
            d.copy_dimmed(raw, (1.0, shape, 0.0, (1.0 - shape) / 2.0), mul);
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
        d.glass_backdrop = self.glass;
        d.veil_backdrop = self.veil;
        self.blurs_made_with = strengths;
    }

    pub fn backdrop_bytes(&self) -> usize {
        self.sharp.bytes() + self.glass.bytes() + self.veil.bytes() + self.fading_from.bytes()
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
        let cols = first_row.max(1);
        let bar_count = cols.min(apps.len());
        if apps.iter().eq(self.tiles.iter().map(|tile| &tile.app)) && self.bar_count == bar_count && self.cols == cols {
            return;
        }
        let focused_id = self.moving.clone().or_else(|| self.tiles.get(self.focus).map(|tile| tile.app.id.clone()));
        let old_place = if self.tiles.is_empty() { (0, 0) } else { self.place_of(self.focus) };
        d.let_go(|key| key.starts_with("tile:") || key.starts_with("icon:"));
        self.cols = cols;
        self.bar_count = bar_count;
        self.tiles = apps.into_iter().map(|app| Tile { bg: app.color.unwrap_or(if app.inside.is_some() { FOLDER_COLOUR } else { TILE_WITHOUT_COLOUR }), app, focus: Anim::still(0.0) }).collect();
        let kept = focused_id.and_then(|id| self.tiles.iter().position(|tile| tile.app.id == id));
        let near = || {
            let (row, col) = old_place;
            let row = row.min(self.row_count() - 1);
            first_of(row, self.bar_count, self.cols) + col.min(row_len(row, self.tiles.len(), self.bar_count, self.cols).max(1) - 1)
        };
        self.focus = kept.unwrap_or_else(near).min(self.tiles.len().saturating_sub(1));
        if self.moving.as_ref().is_some_and(|id| !self.tiles.iter().any(|t| &t.app.id == id)) {
            self.moving = None;
        }
        if let Some((name, _)) = &self.folder {
            let name = name.clone();
            self.folder = self.tiles.iter().find(|t| t.app.inside.is_some() && t.app.title == name).map(|t| (name, Self::folder_tiles(t)));
            match &self.folder {
                Some((_, inside)) => self.folder_index = self.folder_index.min(inside.len().saturating_sub(1)),
                None if self.zone == Zone::Folder => self.zone = Zone::Rows,
                None => {}
            }
        }
        self.settle(now);
    }

    fn folder_tiles(tile: &Tile) -> Vec<Tile> {
        tile.app.inside.clone().unwrap_or_default().into_iter().map(|app| Tile { bg: app.color.unwrap_or(TILE_WITHOUT_COLOUR), app, focus: Anim::still(0.0) }).collect()
    }

    fn focused_tile(&self) -> bool {
        self.engaged && matches!(self.zone, Zone::Rows | Zone::Menu) && !(self.zone == Zone::Menu && self.menu.from_folder)
    }

    fn lower(&self) -> bool {
        self.engaged && matches!(self.zone, Zone::Rows | Zone::Menu) && self.place_of(self.focus).0 > 0 && !(self.zone == Zone::Menu && self.menu.from_folder)
    }

    fn settle(&mut self, _now: u32) {
        let lower = self.lower();
        let slid = if lower { 1.0 } else { 0.0 };
        let shown = self.focused_tile();
        for (at, tile) in self.tiles.iter_mut().enumerate() {
            tile.focus.set(if shown && at == self.focus { 1.0 } else { 0.0 });
        }
        let (row, col) = self.place_of(self.focus);
        self.shift.set(if lower { lower_shift(row) } else { 0.0 });
        self.veil_fade.set(slid);
        self.drop.set(slid);
        self.shine_x.set(tile_centre(col));
        self.shine_alpha.set(if shown && row == 0 { 1.0 } else { 0.0 });
        self.status_alpha.set(1.0 - slid);
        for (at, pill) in self.top_focus.iter_mut().enumerate() {
            pill.set(if self.engaged && self.zone == Zone::Top && at == self.top_index { 1.0 } else { 0.0 });
        }
        self.panel_alpha.set(if matches!(self.zone, Zone::Menu | Zone::Folder) { 1.0 } else { 0.0 });
    }

    pub fn set_clock(&mut self, time: String, period: String, date: String) -> bool {
        let clock = (time, period, date);
        if clock == self.clock {
            return false;
        }
        self.clock = clock;
        true
    }

    pub fn toast(&mut self, text: &str, error: bool, now: u32) {
        self.toast = Some((text.to_string(), error, now.wrapping_add(TOAST_MS)));
    }

    fn animate_to_state(&mut self, d: &Draw, was_lower: bool, was_shift: f32, now: u32) {
        let look = &d.look;
        let lower = self.lower();
        let slid = if lower { 1.0 } else { 0.0 };
        let shown = self.focused_tile();
        let focus_ms = self.ms(look.focus_ms);
        for (at, tile) in self.tiles.iter_mut().enumerate() {
            tile.focus.go(if shown && at == self.focus { 1.0 } else { 0.0 }, focus_ms, &EASE_OUT, now);
        }
        for (at, pill) in self.top_focus.iter_mut().enumerate() {
            pill.go(if self.engaged && self.zone == Zone::Top && at == self.top_index { 1.0 } else { 0.0 }, focus_ms, &EASE_OUT, now);
        }
        let (row, col) = self.place_of(self.focus);
        if row == 0 {
            self.shine_x.go(tile_centre(col), self.ms(look.scroll_ms), &EASE_SCROLL, now);
        }
        self.shine_alpha.go(if shown && row == 0 { 1.0 } else { 0.0 }, self.ms(200.0), &EASE, now);
        let shift = if lower { lower_shift(row) } else { 0.0 };
        if lower != was_lower {
            self.shift.go(shift, self.ms(if lower { look.down_ms } else { look.up_ms }), if lower { &EASE_DOWN } else { &EASE_UP }, now);
            self.veil_fade.go(slid, self.ms(if lower { look.veil_down_ms } else { look.veil_up_ms }), &EASE_VEIL, now);
            self.drop.go(slid, self.ms(200.0), &EASE_OUT, now);
            self.status_alpha.go_after(self.ms(if lower { look.hide_top_wait_ms } else { look.show_top_wait_ms }), 1.0 - slid, self.ms(look.fade_ms), &EASE, now);
        } else if shift != was_shift {
            self.shift.go(shift, self.ms(look.scroll_ms), &EASE_SCROLL, now);
        }
        self.panel_alpha.go(if matches!(self.zone, Zone::Menu | Zone::Folder) { 1.0 } else { 0.0 }, self.ms(200.0), &EASE_OUT, now);
    }

    fn start_move(&mut self, key: Key, now: u32) {
        if self.mv.active {
            return;
        }
        let name = match key {
            Key::Left => "Left",
            Key::Right => "Right",
            Key::Up => "Up",
            Key::Down => "Down",
            Key::Ok => "OK",
            Key::Back => "Back",
            Key::Digit(_) => "Digit",
        };
        self.mv = Move { active: true, name, started: now, last: now, ..Move::default() };
    }

    pub fn can_hold(&self) -> bool {
        if !self.engaged || self.zone == Zone::Folder {
            return true;
        }
        if self.zone != Zone::Rows {
            return false;
        }
        self.moving.is_some() || self.tiles.get(self.focus).is_some_and(|t| t.app.inside.is_none())
    }

    pub fn hold(&mut self, d: &Draw, now: u32) {
        let (was_lower, was_shift) = (self.lower(), self.shift.to_value());
        match self.zone {
            Zone::Folder if self.engaged => self.open_menu(true),
            Zone::Rows if self.engaged => {
                if self.moving.is_some() {
                    self.open_menu(false);
                } else if let Some(tile) = self.tiles.get(self.focus).filter(|t| t.app.inside.is_none()) {
                    self.moving = Some(tile.app.id.clone());
                }
            }
            _ => return,
        }
        self.animate_to_state(d, was_lower, was_shift, now);
    }

    fn open_menu(&mut self, from_folder: bool) {
        let anchor = (!from_folder).then(|| {
            let (row, col) = self.place_of(self.focus);
            let shift = if row == 0 { 0.0 } else { lower_shift(row) };
            let top = row_top(row) - shift + if row > 0 { GRID_DROP } else { 0.0 } - MOVING_RISE;
            (tile_centre(col), top - TILE_H * (LIFT_MOVING - 1.0) / 2.0, top + TILE_H * (1.0 + (LIFT_MOVING - 1.0) / 2.0))
        });
        self.menu = Menu { from_folder, folders: false, confirm: false, index: 0, anchor };
        self.zone = Zone::Menu;
    }

    fn current_menu(&self) -> Vec<MenuItem> {
        let can_uninstall = self.moving.as_deref().is_some_and(crate::root::can_uninstall);
        menu_items(self.menu.from_folder, self.menu.folders, can_uninstall)
    }

    pub fn key(&mut self, d: &Draw, key: Key, now: u32) -> Option<Action> {
        let (was_lower, was_shift) = (self.lower(), self.shift.to_value());
        let action = self.react(key);
        if self.lower() != was_lower || self.shift.to_value() != was_shift || self.focus_changed {
            self.start_move(key, now);
        }
        self.focus_changed = false;
        self.animate_to_state(d, was_lower, was_shift, now);
        action
    }

    fn react(&mut self, key: Key) -> Option<Action> {
        if let Key::Digit(n) = key {
            if self.moving.is_some() || !matches!(self.zone, Zone::Rows | Zone::Top) || n == 0 || n > self.bar_count {
                return None;
            }
            self.engaged = true;
            self.zone = Zone::Rows;
            self.set_focus(n - 1);
            return self.open_focused();
        }
        if !self.engaged {
            self.engaged = true;
            self.focus_changed = true;
            return None;
        }
        match self.zone {
            Zone::Top => {
                match key {
                    Key::Left => self.top_index = self.top_index.saturating_sub(1),
                    Key::Right => self.top_index = (self.top_index + 1).min(TOP_ITEMS.len() - 1),
                    Key::Down | Key::Back => self.zone = Zone::Rows,
                    Key::Ok => return Some(Action::Top(TOP_ITEMS[self.top_index].0)),
                    _ => {}
                }
                self.focus_changed = true;
                None
            }
            Zone::Menu => self.menu_key(key),
            Zone::Folder => {
                let count = self.folder.as_ref().map_or(0, |(_, inside)| inside.len());
                let last = count.saturating_sub(1);
                match key {
                    Key::Left => self.folder_index = self.folder_index.saturating_sub(1),
                    Key::Right => self.folder_index = (self.folder_index + 1).min(last),
                    Key::Up if self.folder_index >= FOLDER_COLUMNS => self.folder_index -= FOLDER_COLUMNS,
                    Key::Down if self.folder_index + FOLDER_COLUMNS <= last => self.folder_index += FOLDER_COLUMNS,
                    Key::Back => self.zone = Zone::Rows,
                    Key::Ok => {
                        let id = self.folder.as_ref().and_then(|(_, inside)| inside.get(self.folder_index)).map(|t| t.app.id.clone());
                        self.zone = Zone::Rows;
                        return id.map(Action::Open);
                    }
                    _ => {}
                }
                None
            }
            Zone::Rows => self.rows_key(key),
        }
    }

    fn set_focus(&mut self, at: usize) {
        if at != self.focus {
            self.focus = at;
            self.focus_changed = true;
        }
    }

    fn open_focused(&mut self) -> Option<Action> {
        let tile = self.tiles.get(self.focus)?;
        if tile.app.inside.is_some() {
            self.folder = Some((tile.app.title.clone(), Self::folder_tiles(tile)));
            self.folder_index = 0;
            self.zone = Zone::Folder;
            return None;
        }
        Some(Action::Open(tile.app.id.clone()))
    }

    fn rows_key(&mut self, key: Key) -> Option<Action> {
        if self.tiles.is_empty() {
            if key == Key::Up {
                self.zone = Zone::Top;
            }
            return None;
        }
        if let Some(id) = self.moving.clone() {
            return match key {
                Key::Left => Some(Action::Move(id, -1)),
                Key::Right => Some(Action::Move(id, 1)),
                Key::Up => Some(Action::Move(id, -(self.cols as isize))),
                Key::Down => Some(Action::Move(id, self.cols as isize)),
                Key::Ok | Key::Back => {
                    self.moving = None;
                    self.focus_changed = true;
                    None
                }
                Key::Digit(_) => None,
            };
        }
        let row = self.place_of(self.focus).0;
        match key {
            Key::Ok => return self.open_focused(),
            Key::Up if row == 0 => {
                self.zone = Zone::Top;
                self.focus_changed = true;
            }
            Key::Back => {
                if row > 0 {
                    let col = self.place_of(self.focus).1;
                    self.set_focus(col.min(self.bar_count.saturating_sub(1)));
                }
            }
            _ => self.set_focus(moved(self.focus, key, self.tiles.len(), self.bar_count, self.cols)),
        }
        None
    }

    fn menu_key(&mut self, key: Key) -> Option<Action> {
        let items = self.current_menu();
        let last = items.len().saturating_sub(1);
        match key {
            Key::Up => {
                self.menu.index = self.menu.index.saturating_sub(1);
                self.menu.confirm = false;
            }
            Key::Down => {
                self.menu.index = (self.menu.index + 1).min(last);
                self.menu.confirm = false;
            }
            Key::Back => {
                if self.menu.folders {
                    self.menu.folders = false;
                    self.menu.index = self.current_menu().len().saturating_sub(1);
                } else {
                    self.zone = if self.menu.from_folder { Zone::Folder } else { Zone::Rows };
                }
            }
            Key::Ok => {
                let pick = *items.get(self.menu.index)?;
                if pick == MenuItem::TakeOut {
                    self.zone = Zone::Folder;
                    let id = self.folder.as_ref().and_then(|(_, inside)| inside.get(self.folder_index)).map(|t| t.app.id.clone());
                    return id.map(Action::FolderOut);
                }
                if pick == MenuItem::AddToFolder {
                    self.menu.folders = true;
                    self.menu.index = 0;
                    return None;
                }
                if pick == MenuItem::Uninstall && !self.menu.confirm {
                    self.menu.confirm = true;
                    return None;
                }
                let id = self.moving.take()?;
                self.zone = Zone::Rows;
                self.focus_changed = true;
                return Some(match pick {
                    MenuItem::Uninstall => Action::Uninstall(id),
                    MenuItem::Remove => Action::Remove(id),
                    MenuItem::Into(at) => Action::FolderAdd(id, FOLDER_PRESETS[at].to_string()),
                    _ => return None,
                });
            }
            _ => {}
        }
        None
    }

    pub fn hit(&self, x: f32, y: f32, now: u32) -> Option<(Zone, usize)> {
        if !matches!(self.zone, Zone::Rows | Zone::Top) || self.moving.is_some() {
            return None;
        }
        if self.status_alpha.at(now) > 0.5 {
            for at in 0..TOP_ITEMS.len() {
                let (cx, cy) = (pill_left(at) + PILL / 2.0, PILL_TOP + PILL / 2.0);
                if (x - cx).powi(2) + (y - cy).powi(2) <= (PILL / 2.0 + 6.0).powi(2) {
                    return Some((Zone::Top, at));
                }
            }
        }
        let shift = self.shift.at(now);
        let grid_drop = self.drop.at(now) * GRID_DROP;
        (0..self.tiles.len()).find_map(|at| {
            let (row, col) = self.place_of(at);
            let top = row_top(row) - shift + if row > 0 { grid_drop } else { 0.0 };
            let left = tile_left(col) - TILE_GAP / 2.0;
            (x >= left && x < left + TILE_W + TILE_GAP && y >= top - 20.0 && y < top + TILE_H + 20.0).then_some((Zone::Rows, at))
        })
    }

    pub fn point(&mut self, d: &Draw, x: f32, y: f32, now: u32) -> bool {
        let target = self.hit(x, y, now);
        let (was_lower, was_shift) = (self.lower(), self.shift.to_value());
        let before = (self.engaged, self.zone, self.focus, self.top_index);
        match target {
            Some((Zone::Top, at)) => {
                self.engaged = true;
                self.zone = Zone::Top;
                self.top_index = at;
            }
            Some((_, at)) => {
                self.engaged = true;
                self.zone = Zone::Rows;
                self.focus = at;
            }
            None if matches!(self.zone, Zone::Rows | Zone::Top) && self.moving.is_none() && !self.lower() => {
                self.engaged = false;
                self.zone = Zone::Rows;
            }
            None => {}
        }
        let changed = before != (self.engaged, self.zone, self.focus, self.top_index);
        if changed {
            self.animate_to_state(d, was_lower, was_shift, now);
        }
        changed
    }

    pub fn resting_zone(&self) -> bool {
        matches!(self.zone, Zone::Rows | Zone::Top) && self.moving.is_none()
    }

    pub fn go_to_bar(&mut self, d: &Draw, now: u32) {
        let (was_lower, was_shift) = (self.lower(), self.shift.to_value());
        self.engaged = true;
        self.zone = Zone::Rows;
        let col = self.place_of(self.focus).1;
        self.focus = col.min(self.bar_count.saturating_sub(1));
        self.animate_to_state(d, was_lower, was_shift, now);
    }

    pub fn go_home(&mut self, d: &Draw, now: u32) {
        let (was_lower, was_shift) = (self.lower(), self.shift.to_value());
        self.zone = Zone::Rows;
        self.moving = None;
        self.folder = None;
        self.engaged = false;
        self.focus = 0;
        self.animate_to_state(d, was_lower, was_shift, now);
    }

    pub fn moving(&self, now: u32) -> bool {
        let anims = [&self.wallpaper_fade, &self.shift, &self.veil_fade, &self.drop, &self.shine_x, &self.shine_alpha, &self.status_alpha, &self.panel_alpha];
        anims.iter().any(|a| a.busy(now))
            || self.tiles.iter().any(|t| t.focus.busy(now))
            || self.top_focus.iter().any(|a| a.busy(now))
            || (self.moving.is_some() && !self.reduce_motion)
            || self.toast.as_ref().is_some_and(|(_, _, until)| (until.wrapping_sub(now) as i32) > -300)
    }

    pub fn where_now(&self) -> Value {
        let mut focused = Vec::new();
        if self.engaged {
            match self.zone {
                Zone::Top => focused.push(json!({"part": "pill", "label": TOP_ITEMS[self.top_index].1, "index": self.top_index, "in": "top"})),
                Zone::Rows | Zone::Menu => {
                    if let Some(tile) = self.tiles.get(self.focus) {
                        let (row, col) = self.place_of(self.focus);
                        let part = if tile.app.inside.is_some() { "folder" } else { "tile" };
                        focused.push(json!({"part": part, "label": tile.app.title, "id": tile.app.id, "index": col, "in": if row == 0 { "bar".to_string() } else { format!("row {row}") }, "moving": self.moving.is_some()}));
                    }
                }
                Zone::Folder => {}
            }
            if self.zone == Zone::Folder || (self.zone == Zone::Menu && self.menu.from_folder) {
                if let Some((name, inside)) = &self.folder {
                    if let Some(tile) = inside.get(self.folder_index) {
                        focused.push(json!({"part": "tile", "label": tile.app.title, "id": tile.app.id, "index": self.folder_index, "in": format!("folder {name}")}));
                    }
                }
            }
            if self.zone == Zone::Menu {
                let items = self.current_menu();
                if let Some(item) = items.get(self.menu.index) {
                    focused.push(json!({"part": "menu", "label": self.menu_label(*item), "index": self.menu.index}));
                }
            }
        }
        json!({"screen": "home", "controls": false, "resting": !self.engaged, "focused": focused})
    }

    fn menu_label(&self, item: MenuItem) -> &'static str {
        match item {
            MenuItem::Uninstall if self.menu.confirm => "Press OK again to uninstall",
            MenuItem::Uninstall => "Uninstall App",
            MenuItem::Remove => "Remove from Home Screen",
            MenuItem::AddToFolder => "Add to Folder",
            MenuItem::Into(at) => FOLDER_PRESETS[at],
            MenuItem::TakeOut => "Take out of Folder",
        }
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

    fn icon_of(d: &mut Draw, tile: &mut Tile) -> Tex {
        let key = icon_key(&tile.app.id);
        if let Some(icon) = d.pictures.find(&key) {
            return icon;
        }
        let icon = if let Some(inside) = &tile.app.inside {
            folder_picture(inside)
        } else {
            let from_file = tile.app.icon_file.as_deref().and_then(decoded);
            if let Some(icon) = &from_file {
                if let Some(background) = background_of(&icon.rgba, icon.w as usize, icon.h as usize) {
                    tile.bg = background;
                }
            }
            from_file.unwrap_or_else(|| Self::monogram(d, &tile.app.title))
        };
        let picture = d.gfx.upload_rgba(&icon.rgba, icon.w, icon.h);
        d.keep(&key, picture);
        picture
    }

    fn tile_uniforms(d: &Draw, tile: &Tile, icon: Tex, (x, y, w, h): (f32, f32, f32, f32), radius: f32, shadow: [f32; 3], focus: f32) {
        let p = &d.p.tile;
        p.bind(0, c"u_icon", icon.tex);
        p.set4(c"u_box", x, y, w, h);
        p.set1(c"u_radius", radius);
        p.set3(c"u_bg", tile.bg);
        p.set3(c"u_shadow", shadow);
        p.set1(c"u_focus", focus);
        p.set1(c"u_wide", if tile.app.inside.is_some() { 1.0 } else { 0.0 });
    }

    fn resting_tile(d: &mut Draw, tile: &mut Tile, icon: Tex) -> Tex {
        let key = tile_key(&tile.app.id);
        if let Some(resting) = d.pictures.find(&key) {
            return resting;
        }
        let (w, h) = (TILE_W + 2.0 * TILE_MARGIN, TILE_H + 2.0 * TILE_MARGIN);
        let resting = d.gfx.blank(w as i32, h as i32);
        unsafe { glDisable(GL_BLEND) }
        d.gfx.at_design_scale_into(&resting);
        Self::tile_uniforms(d, tile, icon, (TILE_MARGIN, TILE_MARGIN, TILE_W, TILE_H), TILE_RADIUS, SHADOW_REST, 0.0);
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

    fn wobble(&self, now: u32) -> f32 {
        if self.reduce_motion {
            return 0.0;
        }
        let phase = (now as f32 % (2.0 * WOBBLE_MS)) / WOBBLE_MS;
        let swing = if phase < 1.0 { phase } else { 2.0 - phase };
        let eased = crate::anim::eased(&EASE, swing);
        (-1.5 + 3.0 * eased).to_radians()
    }

    fn tile_box(&self, at: usize, shift: f32, grid_drop: f32, now: u32) -> (f32, f32, f32, f32, f32) {
        let (row, col) = self.place_of(at);
        let f = self.tiles[at].focus.at(now);
        let left = tile_left(col);
        let top = row_top(row) - shift + if row > 0 { grid_drop } else { 0.0 };
        let lifting = self.moving.as_ref().is_some_and(|id| *id == self.tiles[at].app.id);
        let scale = if lifting { LIFT_MOVING } else { 1.0 + ((if row > 0 { LIFT_GRID } else { LIFT_BAR }) - 1.0) * f };
        let rise = if lifting { MOVING_RISE } else { LIFT_RISE * f * scale };
        let (w, h) = (TILE_W * scale, TILE_H * scale);
        (left + TILE_W / 2.0 - w / 2.0, top + TILE_H / 2.0 - rise - h / 2.0, w, h, f)
    }

    fn draw_tile(&self, d: &Draw, at: usize, resting: Tex, icon: Tex, shift: f32, grid_drop: f32, now: u32) {
        let tile = &self.tiles[at];
        let (row, col) = self.place_of(at);
        let lifting = self.moving.as_ref().is_some_and(|id| *id == tile.app.id);
        let (x, y, w, h, f) = self.tile_box(at, shift, grid_drop, now);
        if f <= 0.0 && !lifting {
            let top = row_top(row) - shift + if row > 0 { grid_drop } else { 0.0 };
            let (rw, rh) = (TILE_W + 2.0 * TILE_MARGIN, TILE_H + 2.0 * TILE_MARGIN);
            return d.picture(resting, tile_left(col) - TILE_MARGIN, top - TILE_MARGIN, rw, rh, 1.0);
        }
        let lifted = if row > 0 { SHADOW_GRID } else { SHADOW_BAR };
        let shadow = [0, 1, 2].map(|i| SHADOW_REST[i] + (lifted[i] - SHADOW_REST[i]) * f.max(if lifting { 1.0 } else { 0.0 }));
        let margin = shadow[0] + shadow[1] + 4.0;
        Self::tile_uniforms(d, tile, icon, (x, y, w, h), TILE_RADIUS * w / TILE_W, shadow, f);
        let angle = if lifting { self.wobble(now) } else { 0.0 };
        d.p.tile.set1(c"u_turn", angle);
        d.gfx.quad(&d.p.tile, x - margin - 8.0, y - margin - 8.0, w + 2.0 * margin + 16.0, h + 2.0 * margin + 16.0);
        d.p.tile.set1(c"u_turn", 0.0);
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

    fn draw_labels(&mut self, d: &mut Draw, row: usize, shift: f32, grid_drop: f32, now: u32) {
        if self.hide_names {
            return;
        }
        for at in 0..self.tiles.len() {
            if self.place_of(at).0 != row {
                continue;
            }
            let f = self.tiles[at].focus.at(now);
            if f <= 0.0 {
                continue;
            }
            let (row, col) = self.place_of(at);
            let top = row_top(row) - shift + if row > 0 { grid_drop } else { 0.0 };
            let baseline = top + if row == 0 { BAR_LABEL_BELOW } else { GRID_LABEL_BELOW - GRID_LABEL_RISE * f };
            let title = self.tiles[at].app.title.clone();
            if let Some(text) = d.text(&title, &TILE_LABEL) {
                text.draw(d, tile_centre(col) - text.advance / 2.0, baseline, f);
            }
        }
    }

    fn draw_status(&mut self, d: &mut Draw, alpha: f32) {
        if alpha <= 0.0 {
            return;
        }
        let (time, period, date) = self.clock.clone();
        let time_style = if d.fonts.is_sf("600-clock") { TIME } else { TIME_TNUM };
        if let Some(time) = d.text(&time, &time_style) {
            time.draw(d, TIME_LEFT, TIME_BASELINE, alpha);
            if let Some(period) = d.text(&period, &PERIOD) {
                period.draw(d, TIME_LEFT + time.advance + PERIOD_GAP, TIME_BASELINE - 9.0, alpha);
            }
        }
        if let Some(date) = d.text(&date, &DATE) {
            date.draw(d, ROWS_EDGE, DATE_BASELINE, alpha);
        }
        let weather = self.weather.clone().filter(|_| self.weather_line && self.forecast != Forecast::Large);
        if let Some(w) = weather {
            let icon = d.glyph(weather::icon(w.kind, w.day), 32, 1.8);
            let x = ROWS_EDGE - 2.0;
            d.inked_picture(icon, x, WEATHER_ICON_TOP + 1.0, 32.0, 32.0, [0.0, 0.0, 0.0, 0.35 * alpha]);
            d.inked_picture(icon, x, WEATHER_ICON_TOP, 32.0, 32.0, [INK_ON_PHOTO[0], INK_ON_PHOTO[1], INK_ON_PHOTO[2], alpha]);
            let mut x = x + 32.0 + 10.0;
            if let Some(temp) = d.text(&format!("{}°", w.temp), &WEATHER_LINE) {
                temp.draw(d, x, WEATHER_BASELINE, alpha);
                x += temp.advance + 12.0;
            }
            if let Some(cond) = d.text(&w.label, &WEATHER_LINE) {
                cond.draw(d, x, WEATHER_BASELINE, alpha);
            }
        }
    }

    fn draw_forecast(&mut self, d: &mut Draw, alpha: f32) {
        let Some(w) = self.weather.clone().filter(|_| self.forecast != Forecast::Off) else { return };
        if alpha <= 0.0 {
            return;
        }
        // ponytail: the large widget draws as the small one; Sam uses the small one on home
        let hours = weather::hours_ahead(&w, self.unix_now, CAST_HOURS);
        if hours.is_empty() {
            return;
        }
        let width = hours.len() as f32 * (CAST_COL + CAST_GAP) - CAST_GAP;
        let top = CAST_BOTTOM - 103.8;
        let shade = self.forecast_shade(d, width);
        d.picture(shade, ROWS_EDGE - 90.0, top - 60.0, shade.w as f32, shade.h as f32, alpha);
        for (i, hour) in hours.iter().enumerate() {
            let centre = ROWS_EDGE + i as f32 * (CAST_COL + CAST_GAP) + CAST_COL / 2.0;
            let icon = d.glyph(weather::icon(hour.kind, hour.day), CAST_ICON as i32, 1.8);
            d.inked_picture(icon, centre - CAST_ICON / 2.0, top + 1.0, CAST_ICON, CAST_ICON, [0.0, 0.0, 0.0, 0.35 * alpha]);
            d.inked_picture(icon, centre - CAST_ICON / 2.0, top, CAST_ICON, CAST_ICON, [1.0, 1.0, 1.0, alpha]);
            if let Some(temp) = d.text(&format!("{}°", hour.temp), &CAST_TEMP) {
                temp.draw(d, centre - temp.advance / 2.0, top + 57.4, alpha);
            }
            if let Some(time) = d.text(&hour_text(&hour.t, self.twenty_four_hour), &CAST_TIME) {
                time.draw(d, centre - time.advance / 2.0, top + 85.8, alpha * 0.6);
            }
            let pill_y = CAST_BOTTOM - 5.0;
            let fill = if hour.rain >= RAIN_LIKELY { [0.486, 0.769, 1.0, 1.0] } else { [0.486, 0.769, 1.0, 0.6] };
            self.round_bar(d, centre - 22.0, pill_y, 44.0, [1.0, 1.0, 1.0, 0.24 * alpha]);
            if hour.rain > 0 {
                self.round_bar(d, centre - 22.0, pill_y, 44.0 * hour.rain.min(100) as f32 / 100.0, [fill[0], fill[1], fill[2], fill[3] * alpha]);
            }
        }
    }

    fn round_bar(&self, d: &mut Draw, x: f32, y: f32, w: f32, ink: [f32; 4]) {
        let key = "shape:bar5";
        let picture = d.pictures.find(key).unwrap_or_else(|| {
            let (pw, ph) = (64usize, 5usize);
            let mut rgba = vec![0u8; pw * ph * 4];
            for py in 0..ph {
                for px in 0..pw {
                    let dx = (px as f32 + 0.5 - pw as f32 / 2.0).abs() - (pw as f32 / 2.0 - 2.5);
                    let dy = (py as f32 + 0.5 - 2.5).abs();
                    let dist = (dx.max(0.0).powi(2) + dy.powi(2)).sqrt() - 2.5;
                    let a = ((0.5 - dist).clamp(0.0, 1.0) * 255.0) as u8;
                    rgba[(py * pw + px) * 4..][..4].copy_from_slice(&[a, a, a, a]);
                }
            }
            let picture = d.gfx.upload_rgba(&rgba, pw as i32, ph as i32);
            d.keep(key, picture);
            picture
        });
        let w = w.max(5.0);
        let ends = 2.5;
        let piece = |d: &Draw, from_u: f32, to_u: f32, at: f32, width: f32| {
            d.p.ink.bind(0, c"u_tex", picture.tex);
            d.p.ink.set4(c"u_box", at - from_u * width / (to_u - from_u), y, width / (to_u - from_u), 5.0);
            d.p.ink.set4(c"u_ink", ink[0], ink[1], ink[2], ink[3]);
            d.gfx.quad(&d.p.ink, at, y, width, 5.0);
        };
        piece(d, 0.0, ends / 64.0, x, ends);
        piece(d, ends / 64.0, 1.0 - ends / 64.0, x + ends, w - 2.0 * ends);
        piece(d, 1.0 - ends / 64.0, 1.0, x + w - ends, ends);
    }

    fn forecast_shade(&mut self, d: &mut Draw, width: f32) -> Tex {
        let key = format!("shape:castshade:{}", width as i32);
        if let Some(p) = d.pictures.find(&key) {
            return p;
        }
        let (w, h) = ((width + 180.0) as usize, (103.8f32 + 110.0) as usize);
        let mut rgba = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let (nx, ny) = ((x as f32 + 0.5) / w as f32 * 2.0 - 1.0, (y as f32 + 0.5) / h as f32 * 2.0 - 1.0);
                let r = (nx * nx + ny * ny).sqrt();
                let a = if r < 0.55 { 0.34 - (0.14 * r / 0.55) } else { (0.2 * (1.0 - (r - 0.55) / 0.45)).max(0.0) };
                rgba[(y * w + x) * 4 + 3] = (a * 255.0) as u8;
            }
        }
        let picture = d.gfx.upload_rgba(&rgba, w as i32, h as i32);
        d.keep(&key, picture);
        picture
    }

    fn draw_caption(&mut self, d: &mut Draw, alpha: f32) {
        let Some((title, credit)) = self.caption.clone() else { return };
        if alpha <= 0.0 {
            return;
        }
        if let Some(t) = d.text(&title, &CAPTION_TITLE) {
            t.draw(d, CAPTION_RIGHT - t.advance, CAPTION_TITLE_BASELINE, 0.7 * alpha);
        }
        if let Some(c) = d.text(&credit, &CAPTION_CREDIT) {
            c.draw(d, CAPTION_RIGHT - c.advance, CAPTION_CREDIT_BASELINE, 0.7 * alpha);
        }
    }

    fn draw_top(&mut self, d: &mut Draw, alpha: f32, now: u32) {
        if alpha <= 0.0 {
            return;
        }
        for (at, (_, label, glyph)) in TOP_ITEMS.iter().enumerate() {
            let f = self.top_focus[at].at(now);
            let scale = 1.0 + (PILL_LIFT - 1.0) * f;
            let size = PILL * scale;
            let (cx, cy) = (pill_left(at) + PILL / 2.0, PILL_TOP + PILL / 2.0);
            d.glass(&Glass { x: cx - size / 2.0, y: cy - size / 2.0, w: size, h: size, radius: size / 2.0, blur: f, sheen: 1.0 + 1.6 * f, rim: 1.0 + 0.6 * f, alpha });
            let icon = d.glyph(glyph, 28, 1.8);
            let g = 28.0 * scale;
            d.inked_picture(icon, cx - g / 2.0, cy - g / 2.0, g, g, [1.0, 1.0, 1.0, 0.62 * alpha]);
            if f > 0.0 {
                if let Some(text) = d.text(label, &PILL_LABEL) {
                    let x = if at == TOP_ITEMS.len() - 1 { pill_left(at) + PILL - text.advance } else { cx - text.advance / 2.0 };
                    text.draw(d, x, PILL_LABEL_BASELINE, f * alpha);
                }
            }
        }
    }

    fn draw_menu(&mut self, d: &mut Draw, alpha: f32) {
        if self.zone != Zone::Menu || alpha <= 0.0 {
            return;
        }
        let items = self.current_menu();
        let (left, top) = menu_place(self.menu.anchor, items.len());
        for (i, item) in items.iter().enumerate() {
            let focused = i == self.menu.index;
            let scale = if focused { MENU_LIFT } else { 1.0 };
            let (w, h) = (MENU_W * scale, MENU_ITEM_H * scale);
            let (cx, cy) = (left + MENU_W / 2.0, top + i as f32 * MENU_ITEM_STEP + MENU_ITEM_H / 2.0);
            let f = if focused { 1.0 } else { 0.0 };
            d.glass(&Glass { x: cx - w / 2.0, y: cy - h / 2.0, w, h, radius: h / 2.0, blur: f, sheen: 1.0 + 1.6 * f, rim: 1.0 + 0.6 * f, alpha });
            let glyph = match item {
                MenuItem::Uninstall => TRASH,
                MenuItem::Remove => MINUS,
                _ => FOLDER,
            };
            let warn = *item == MenuItem::Uninstall && self.menu.confirm;
            let icon = d.glyph(glyph, 34, 1.8);
            let ink = if warn { [1.0, 0.36, 0.32] } else { [0.9, 0.9, 0.9] };
            let x = cx - w / 2.0 + 20.0 * scale;
            d.inked_picture(icon, x, cy - 17.0 * scale, 34.0 * scale, 34.0 * scale, [ink[0], ink[1], ink[2], alpha]);
            let label = self.menu_label(*item);
            if let Some(text) = d.text(label, if warn { &MENU_WARN } else { &MENU_NAME }) {
                text.draw_scaled(d, x + (34.0 + 14.0) * scale, cy + 8.0 * scale, scale, alpha);
            }
        }
    }

    fn draw_folder(&mut self, d: &mut Draw, alpha: f32, now: u32) {
        let open = self.zone == Zone::Folder || (self.zone == Zone::Menu && self.menu.from_folder);
        let Some((name, inside)) = self.folder.take() else { return };
        if !open || alpha <= 0.0 {
            self.folder = Some((name, inside));
            return;
        }
        let mut inside = inside;
        self.dim(d, 0.38 * alpha);
        let rows = inside.len().div_ceil(FOLDER_COLUMNS).max(1);
        let left = DESIGN_W / 2.0 - FOLDER_PANEL_W / 2.0;
        let height = 30.0 + 38.0 + 22.0 + 6.0 + rows as f32 * FOLDER_ROW_STEP - 4.0;
        d.glass(&Glass { x: left, y: FOLDER_PANEL_TOP, w: FOLDER_PANEL_W, h: height, radius: 44.0, blur: 1.0, sheen: 1.0, rim: 1.0, alpha });
        if let Some(title) = d.text(&name, &FOLDER_TITLE) {
            title.draw(d, left + 36.0 + 6.0, FOLDER_PANEL_TOP + 30.0 + 32.0, alpha);
        }
        let grid_top = FOLDER_PANEL_TOP + 30.0 + 38.0 + 22.0 + 6.0;
        let focus_ms = self.ms(d.look.focus_ms);
        let selected = (self.zone == Zone::Folder || self.zone == Zone::Menu).then_some(self.folder_index);
        for (i, tile) in inside.iter_mut().enumerate() {
            tile.focus.go(if Some(i) == selected { 1.0 } else { 0.0 }, focus_ms, &EASE_OUT, now);
            let f = tile.focus.at(now);
            let (col, row) = (i % FOLDER_COLUMNS, i / FOLDER_COLUMNS);
            let x = left + 36.0 + 6.0 + col as f32 * (FOLDER_TILE_W + FOLDER_TILE_GAP);
            let y = grid_top + row as f32 * FOLDER_ROW_STEP;
            let scale = 1.0 + (LIFT_GRID - 1.0) * f;
            let (w, h) = (FOLDER_TILE_W * scale, FOLDER_TILE_H * scale);
            let (bx, by) = (x + FOLDER_TILE_W / 2.0 - w / 2.0, y + FOLDER_TILE_H / 2.0 - LIFT_RISE * f - h / 2.0);
            let icon = Self::icon_of(d, tile);
            let shadow = [0, 1, 2].map(|k| SHADOW_REST[k] + (SHADOW_GRID[k] - SHADOW_REST[k]) * f);
            let margin = shadow[0] + shadow[1] + 4.0;
            Self::tile_uniforms(d, tile, icon, (bx, by, w, h), 24.0 * scale, shadow, f);
            d.p.tile.set1(c"u_turn", 0.0);
            d.gfx.quad(&d.p.tile, bx - margin, by - margin, w + 2.0 * margin, h + 2.0 * margin);
            if !self.hide_names {
                if let Some(text) = d.text(&tile.app.title, &FOLDER_LABEL) {
                    text.draw(d, x + FOLDER_TILE_W / 2.0 - text.advance / 2.0, y + FOLDER_TILE_H + 12.0 + 21.0, alpha);
                }
            }
        }
        self.folder = Some((name, inside));
    }

    fn dim(&self, d: &Draw, alpha: f32) {
        d.p.dim.activate();
        d.p.dim.set1(c"u_alpha", alpha);
        d.gfx.quad(&d.p.dim, 0.0, 0.0, DESIGN_W, DESIGN_H);
    }

    pub fn toast_showing(&self, now: u32) -> bool {
        self.toast.as_ref().is_some_and(|(_, _, until)| (until.wrapping_sub(now) as i32) > -300)
    }

    pub fn toast_over(&mut self, d: &mut Draw, now: u32) {
        unsafe {
            glEnable(GL_BLEND);
            glBlendFuncSeparate(GL_ONE, GL_ONE_MINUS_SRC_ALPHA, GL_ZERO, GL_ONE);
        }
        self.draw_toast(d, now);
    }

    fn draw_toast(&mut self, d: &mut Draw, now: u32) {
        let Some((text, error, until)) = self.toast.clone() else { return };
        let left = until.wrapping_sub(now) as i32;
        if left <= -300 {
            self.toast = None;
            return;
        }
        let alpha = if left < 0 { 1.0 + left as f32 / 300.0 } else { 1.0 };
        let Some(t) = d.text(&text, if error { &TOAST_ERROR } else { &TOAST }) else { return };
        let (w, h) = (t.advance + 56.0, 26.0 * 1.2 + 28.0);
        let (x, y) = (DESIGN_W / 2.0 - w / 2.0, DESIGN_H - 56.0 - h);
        d.p.fill.activate();
        d.p.fill.set4(c"u_box", x, y, w, h);
        d.p.fill.set1(c"u_radius", h / 2.0);
        d.p.fill.set4(c"u_ink", 1.0, 1.0, 1.0, 0.18 * alpha);
        d.gfx.quad(&d.p.fill, x, y, w, h);
        t.draw(d, x + 28.0, y + 14.0 + 26.0 * 0.95, alpha);
    }

    pub fn draw(&mut self, d: &mut Draw, now: u32) {
        d.pictures.next_frame();
        let shift = self.shift.at(now);
        let veiled = self.veil_fade.at(now);
        let grid_drop = self.drop.at(now) * GRID_DROP;
        let status_alpha = self.status_alpha.at(now);
        let panel_alpha = self.panel_alpha.at(now);

        let mut pictures: Vec<Option<(Tex, Tex)>> = Vec::with_capacity(self.tiles.len());
        for at in 0..self.tiles.len() {
            let row = self.place_of(at).0;
            let top = row_top(row) - shift + if row > 0 { grid_drop } else { 0.0 };
            let seen = top - 80.0 <= DESIGN_H && top + TILE_H + 80.0 >= 0.0;
            pictures.push(if seen {
                let tile = &mut self.tiles[at];
                let icon = Self::icon_of(d, tile);
                Some((Self::resting_tile(d, tile, icon), icon))
            } else {
                None
            });
        }
        let (lens, gloss) = self.bar_plates(d);

        d.gfx.into_screen();
        unsafe {
            glClearColor(0.0, 0.0, 0.0, 1.0);
            glClear(GL_COLOR_BUFFER_BIT);
            glDisable(GL_BLEND);
        }
        let faded_in = self.wallpaper_fade.at(now);
        if faded_in >= 1.0 && self.fading_from.tex != 0 {
            d.gfx.free(&mut self.fading_from);
        }
        if self.sharp.tex == 0 {
        } else if veiled <= 0.0 && self.fading_from.tex != 0 {
            d.p.backdrop.bind(0, c"u_wall", self.fading_from.tex);
            d.p.backdrop.bind(1, c"u_veil", self.sharp.tex);
            d.p.backdrop.set1(c"u_t", faded_in);
            d.gfx.quad(&d.p.backdrop, 0.0, 0.0, DESIGN_W, DESIGN_H);
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
        self.draw_forecast(d, status_alpha);
        self.draw_caption(d, status_alpha);
        let focus_row = self.place_of(self.focus).0;
        let lifted = self.moving.as_ref().and_then(|id| self.tiles.iter().position(|t| &t.app.id == id));
        let rows = self.row_count();
        for row in (0..rows).rev() {
            if row == 0 && self.sharp.tex != 0 {
                self.draw_bar(d, lens, gloss, BAR_TILES_TOP - BAR_ABOVE_TILES - shift, veiled, now);
                unsafe { glEnable(GL_BLEND) }
            }
            let in_row = |at: &usize| place(*at, self.bar_count, self.cols).0 == row;
            let last = if lifted.is_some_and(|at| self.place_of(at).0 == row) { lifted } else { (row == focus_row).then_some(self.focus) };
            for at in (0..self.tiles.len()).filter(in_row).filter(|&at| Some(at) != last).chain(last) {
                if let Some(Some((resting, icon))) = pictures.get(at) {
                    self.draw_tile(d, at, *resting, *icon, shift, grid_drop, now);
                }
            }
            self.draw_labels(d, row, shift, grid_drop, now);
        }

        self.draw_status(d, status_alpha);
        self.draw_top(d, status_alpha, now);
        if self.moving.is_some() && self.zone == Zone::Rows {
            let hint = Style { px: 26, weight: "500", spacing_em: 0.0, feature: "", ink: INK_ON_PHOTO, shadows: &TEXT_SHADOW };
            if let Some(text) = d.text("Move with the arrows · OK to drop · Hold OK for more", &hint) {
                text.draw(d, DESIGN_W / 2.0 - text.advance / 2.0, 120.0 + 26.0, 1.0);
            }
        }
        self.draw_folder(d, panel_alpha, now);
        self.draw_menu(d, panel_alpha);
        self.draw_toast(d, now);
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
        crate::note!("move {}", self.last_move);
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

pub fn moved_in_order(ids: &[String], id: &str, by: isize) -> Option<Vec<String>> {
    let from = ids.iter().position(|x| x == id)?;
    let to = (from as isize + by).clamp(0, ids.len() as isize - 1) as usize;
    if to == from {
        return None;
    }
    let mut out = ids.to_vec();
    let moving = out.remove(from);
    out.insert(to, moving);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(id: &str) -> Shown {
        Shown { id: id.into(), title: id.into(), icon_file: None, color: None, inside: None }
    }

    #[test]
    fn the_bar_holds_the_first_apps_and_the_rest_fill_rows_as_wide() {
        assert_eq!(place(0, 6, 6), (0, 0));
        assert_eq!(place(5, 6, 6), (0, 5));
        assert_eq!(place(6, 6, 6), (1, 0));
        assert_eq!(place(13, 6, 6), (2, 1));
        assert_eq!(place(5, 5, 5), (1, 0));
    }

    #[test]
    fn left_and_right_stop_at_the_ends_of_a_row() {
        assert_eq!(moved(0, Key::Left, 14, 6, 6), 0);
        assert_eq!(moved(5, Key::Right, 14, 6, 6), 5);
        assert_eq!(moved(13, Key::Right, 14, 6, 6), 13);
        assert_eq!(moved(6, Key::Left, 14, 6, 6), 6);
        assert_eq!(moved(7, Key::Right, 14, 6, 6), 8);
    }

    #[test]
    fn down_into_a_short_row_lands_on_its_last_tile_and_up_comes_back_above_it() {
        assert_eq!(moved(4, Key::Down, 14, 6, 6), 10);
        assert_eq!(moved(10, Key::Down, 14, 6, 6), 13);
        assert_eq!(moved(13, Key::Up, 14, 6, 6), 7);
        assert_eq!(moved(13, Key::Down, 14, 6, 6), 13);
        assert_eq!(moved(2, Key::Up, 14, 6, 6), 2);
    }

    #[test]
    fn down_reaches_every_row_however_many_there_are() {
        assert_eq!(moved(12, Key::Down, 30, 6, 6), 18);
        assert_eq!(moved(24, Key::Down, 30, 6, 6), 24);
        assert_eq!(moved(3, Key::Down, 6, 6, 6), 3);
        assert_eq!(moved(0, Key::Down, 0, 0, 6), 0);
    }

    #[test]
    fn the_grid_slides_the_bar_near_the_top_and_scrolls_on_only_past_the_bottom() {
        assert_eq!(lower_shift(0), 0.0);
        assert_eq!(lower_shift(1), 770.0);
        assert_eq!(lower_shift(2), 770.0);
        assert_eq!(lower_shift(3), row_top(3) - GRID_BOTTOM);
        assert!(lower_shift(4) > lower_shift(3));
    }

    #[test]
    fn the_screens_beat_is_the_usual_gap_between_frames_of_a_move_long_enough_to_tell() {
        assert_eq!(usual_gap(&mut [20, 21, 19, 20, 41, 20, 20]), Some(20.0));
        assert_eq!(usual_gap(&mut [16, 17, 17]), None);
        assert_eq!(usual_gap(&mut [200, 210, 190, 220, 205]), None);
    }

    #[test]
    fn a_moved_app_changes_places_and_stops_at_either_end() {
        let ids: Vec<String> = ["a", "b", "c", "d"].iter().map(|s| s.to_string()).collect();
        assert_eq!(moved_in_order(&ids, "b", 1).unwrap(), ["a", "c", "b", "d"]);
        assert_eq!(moved_in_order(&ids, "b", -6).unwrap(), ["b", "a", "c", "d"]);
        assert_eq!(moved_in_order(&ids, "d", 1), None);
        assert_eq!(moved_in_order(&ids, "x", 1), None);
    }

    #[test]
    fn the_menu_sits_over_the_app_inside_the_screen_or_under_it_near_the_top() {
        assert_eq!(menu_place(Some((960.0, 700.0, 850.0)), 3), (760.0, 700.0 - 24.0 - 252.0));
        assert_eq!(menu_place(Some((100.0, 700.0, 850.0)), 3).0, 60.0);
        assert_eq!(menu_place(Some((960.0, 100.0, 250.0)), 3).1, 250.0 + 24.0);
    }

    #[test]
    fn the_menu_offers_uninstall_only_where_allowed_and_one_choice_inside_a_folder() {
        assert_eq!(menu_items(false, false, true), [MenuItem::Uninstall, MenuItem::Remove, MenuItem::AddToFolder]);
        assert_eq!(menu_items(false, false, false), [MenuItem::Remove, MenuItem::AddToFolder]);
        assert_eq!(menu_items(true, false, true), [MenuItem::TakeOut]);
        assert_eq!(menu_items(false, true, true).len(), FOLDER_PRESETS.len());
    }

    #[test]
    fn the_first_key_only_wakes_the_focus_and_up_from_the_bar_reaches_the_top_buttons() {
        let mut home = Home::new();
        home.tiles = ["a", "b", "c"].iter().map(|id| Tile { app: app(id), bg: [0.0; 3], focus: Anim::still(0.0) }).collect();
        home.bar_count = 3;
        assert_eq!(home.react(Key::Ok), None);
        assert!(home.engaged);
        assert_eq!(home.react(Key::Right), None);
        assert_eq!(home.focus, 1);
        assert_eq!(home.react(Key::Up), None);
        assert_eq!(home.zone, Zone::Top);
        assert_eq!(home.react(Key::Ok), Some(Action::Top("controls")));
        home.react(Key::Down);
        assert_eq!(home.react(Key::Ok), Some(Action::Open("b".into())));
    }

    #[test]
    fn hold_ok_lifts_an_app_and_the_arrows_ask_to_move_it() {
        let mut home = Home::new();
        home.tiles = ["a", "b", "c"].iter().map(|id| Tile { app: app(id), bg: [0.0; 3], focus: Anim::still(0.0) }).collect();
        home.bar_count = 3;
        home.engaged = true;
        home.moving = Some("b".into());
        assert_eq!(home.react(Key::Right), Some(Action::Move("b".into(), 1)));
        assert_eq!(home.react(Key::Down), Some(Action::Move("b".into(), 6)));
        assert_eq!(home.react(Key::Ok), None);
        assert_eq!(home.moving, None);
    }

    #[test]
    fn uninstall_takes_a_second_ok() {
        let mut home = Home::new();
        home.tiles = vec![Tile { app: app("bbc.iplayer.3.0"), bg: [0.0; 3], focus: Anim::still(0.0) }];
        home.bar_count = 1;
        home.engaged = true;
        home.moving = Some("bbc.iplayer.3.0".into());
        home.open_menu(false);
        assert_eq!(home.react(Key::Ok), None);
        assert_eq!(home.react(Key::Ok), Some(Action::Uninstall("bbc.iplayer.3.0".into())));
    }

    #[test]
    fn the_pointer_finds_the_tile_or_top_button_under_it_and_nothing_between() {
        let mut home = Home::new();
        home.tiles = ["a", "b", "c", "d", "e", "f", "g"].iter().map(|id| Tile { app: app(id), bg: [0.0; 3], focus: Anim::still(0.0) }).collect();
        home.bar_count = 6;
        assert_eq!(home.hit(tile_centre(1), BAR_TILES_TOP + 70.0, 0), Some((Zone::Rows, 1)));
        assert_eq!(home.hit(tile_centre(0), FIRST_GRID_TOP + 70.0, 0), Some((Zone::Rows, 6)));
        assert_eq!(home.hit(pill_left(2) + PILL / 2.0, PILL_TOP + PILL / 2.0, 0), Some((Zone::Top, 2)));
        assert_eq!(home.hit(960.0, 400.0, 0), None);
    }

    #[test]
    fn hours_read_as_the_clock_does() {
        assert_eq!(hour_text("2026-11-24T15:00", true), "15:00");
        assert_eq!(hour_text("2026-11-24T15:00", false), "3 PM");
        assert_eq!(hour_text("2026-11-24T00:00", false), "12 AM");
    }
}
