use crate::anim::{Anim, EASE, EASE_OUT, EASE_SCROLL, EASE_VEIL};
use crate::apps::background_of;
use crate::draw::{decoded, Decoded, Draw, Glass, Plate, Style};
use crate::ffi::*;
use crate::gfx::{Tex, DESIGN_H, DESIGN_W};
use crate::livetv::{self, Programme};
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
const SLEEP_MS: f32 = 800.0;
const WAKE_MS: f32 = 600.0;
const SLEEP_CURVE: [f32; 4] = [0.45, 0.0, 0.2, 1.0];
const SLEEP_DROP: f32 = DESIGN_H + 80.0 - (BAR_TILES_TOP - BAR_ABOVE_TILES);
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
pub const DOCK_DROP_MS: f32 = 350.0;
pub const HOLD_MS: u32 = 600;
pub const GUIDE_TILE: &str = "livetv:guide";
const CHANNEL_PREFIX: &str = "channel:";
const LIVE_TITLE_H: f32 = 65.0;
const LIVE_TITLE_ABOVE: f32 = 40.5;
const LIVE_BELOW: f32 = 150.0;
const LIVE_INFO_GAP: f32 = 36.0;
const LIVE_INFO_W: f32 = livetv::LIVE_INFO_W;
const GUIDE_GLYPH: &str = "M3.5 5.5h17v13h-17zM3.5 9.5h17M9 9.5v9";
const BAR_PLATE: Plate = Plate { h: 6.0, radius: 3.0, fill: [1.0, 1.0, 1.0, 0.22], rim: 0.0, shadow: [0.0; 3], ring: 0.0 };
const BAR_FILL: Plate = Plate { fill: [1.0, 1.0, 1.0, 1.0], ..BAR_PLATE };
const METER: Plate = Plate { h: 5.0, radius: 2.5, fill: [1.0, 1.0, 1.0, 0.3], rim: 0.0, shadow: [0.0; 3], ring: 0.0 };
const METER_FILL: Plate = Plate { fill: [1.0, 1.0, 1.0, 1.0], ..METER };

const TIME_SHADOW: [Shadow; 1] = [Shadow { drop: 4, blur: 24, strength: 0.28 }];
const STATUS_SHADOW: [Shadow; 2] = [Shadow { drop: 1, blur: 16, strength: 0.5 }, Shadow { drop: 0, blur: 2, strength: 0.22 }];
const LABEL_SHADOW: [Shadow; 2] = [Shadow { drop: 2, blur: 14, strength: 0.55 }, Shadow { drop: 0, blur: 2, strength: 0.25 }];
const TEXT_SHADOW: [Shadow; 1] = [Shadow { drop: 2, blur: 10, strength: 0.8 }];
const CAPTION_SHADOW: [Shadow; 1] = [Shadow { drop: 2, blur: 14, strength: 0.6 }];
const GLASS_INK_SHADOW: [Shadow; 1] = [Shadow { drop: 1, blur: 8, strength: 0.35 }];
const ROW_TITLE_SHADOW: [Shadow; 1] = [Shadow { drop: 2, blur: 12, strength: 0.45 }];
const INFO_SHADOW: [Shadow; 1] = [Shadow { drop: 2, blur: 12, strength: 0.5 }];

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
const ROW_TITLE: Style = Style { px: 26, weight: "600", spacing_em: -0.01, feature: "", ink: INK_ON_PHOTO, shadows: &ROW_TITLE_SHADOW };
const INFO_LINE: Style = Style { px: 22, weight: "500", spacing_em: 0.0, feature: "", ink: WHITE, shadows: &INFO_SHADOW };
const INFO_TITLE: Style = Style { px: 32, weight: "600", spacing_em: -0.015, feature: "", ink: WHITE, shadows: &INFO_SHADOW };

const SLIDERS: &str = "M4 7h8M16 7h4M4 17h4M12 17h8M12 4.5v5M8 14.5v5";
const GEAR: &str = "M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z";
const INPUTS: &str = "M3.5 7.5h17v7.2a2 2 0 0 1-2 2h-3.2l-1 2.3H9.7l-1-2.3H5.5a2 2 0 0 1-2-2zM8 10.5v2.4M12 10.5v2.4M16 10.5v2.4";
const TRASH: &str = "M4.5 7h15M9.5 7V4.5h5V7M6.5 7l1 12.5h9l1-12.5M10 10.5v6M14 10.5v6";
const MINUS: &str = "M12 3.5a8.5 8.5 0 1 1 0 17 8.5 8.5 0 0 1 0-17zM8 12h8";
const FOLDER: &str = "M3.5 7.5a2 2 0 0 1 2-2h4l2 2.5h7a2 2 0 0 1 2 2V17a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z";
const OPEN_DOT: f32 = 10.0;
pub const TOP_ITEMS: [(&str, &str, &str); 3] = [("controls", "Control Center", SLIDERS), ("settings", "Settings", GEAR), ("inputs", "Inputs", INPUTS)];

#[derive(Clone, PartialEq, Debug)]
pub struct Shown {
    pub id: String,
    pub title: String,
    pub icon_file: Option<String>,
    pub glyph: Option<&'static str>,
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
    Guide,
    Channel(String),
}

#[derive(Clone, PartialEq, Debug)]
pub struct LiveShown {
    pub id: String,
    pub number: String,
    pub name: String,
    pub logo: Option<String>,
    pub now: Option<Programme>,
    pub next: Option<Programme>,
    pub loading: bool,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DockExit {
    Bar,
    Down,
    Up,
    Open,
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

struct LiveTile {
    shown: LiveShown,
    tile: Tile,
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
    pub print: bool,
    glass: Tex,
    veil: Tex,
    blurs_made_with: [i32; 5],
    bar_made_with: f32,
    plates_made_with: [f32; 13],
    shift: Anim,
    sleep: Anim,
    sleep_clock: (f32, f32),
    veil_fade: Anim,
    drop: Anim,
    rows_at: Vec<Anim>,
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
    pub open_apps: Vec<String>,
    pub unix_now: u64,
    toast: Option<(String, bool, u32)>,
    focus_changed: bool,
    mv: Move,
    pub last_move: String,
    pub refresh_ms: f32,
    pub reduce_motion: bool,
    pub panel_open: bool,
    pub lab: Option<bool>,
    live: Vec<LiveTile>,
    live_at: usize,
    in_live: bool,
    live_slot: usize,
    pub preview_showing: Option<String>,
    pub preview_loading: Option<String>,
    live_offset: Anim,
    pub dock: bool,
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

pub fn live_offset(selected: usize, count: usize, slot: usize) -> f32 {
    selected.min(count.saturating_sub(2)).saturating_sub(slot) as f32 * (TILE_W + TILE_GAP)
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

fn resting_key(id: &str, shade: f32) -> String {
    if shade == 1.0 { tile_key(id) } else { format!("tile:{id}:{shade}") }
}

fn shaded(shadow: [f32; 3], shade: f32) -> [f32; 3] {
    [shadow[0], shadow[1], shadow[2] * shade]
}

fn icon_key(id: &str) -> String {
    format!("icon:{id}")
}

fn live_in_view(at: usize, offset: f32) -> bool {
    let left = ROWS_EDGE + at as f32 * (TILE_W + TILE_GAP) - offset;
    left + TILE_W + 80.0 >= 0.0 && left - 80.0 <= DESIGN_W
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

fn glyph_picture(path: &str) -> Decoded {
    let size = TILE_H as usize;
    let drawn = (TILE_H * 0.58) as usize;
    let coverage = crate::vector::stroke(path, 24.0, drawn as u32, 1.6);
    let mut rgba = vec![0u8; size * size * 4];
    let at = (size - drawn) / 2;
    for y in 0..drawn {
        for x in 0..drawn {
            let a = coverage[y * drawn + x];
            rgba[((y + at) * size + x + at) * 4..][..4].copy_from_slice(&[238, 238, 238, a]);
        }
    }
    Decoded { rgba, w: size as i32, h: size as i32 }
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
            print: false,
            glass: Tex::default(),
            veil: Tex::default(),
            blurs_made_with: [0; 5],
            bar_made_with: 0.0,
            plates_made_with: [0.0; 13],
            shift: Anim::still(0.0),
            sleep: Anim::still(0.0),
            sleep_clock: (0.0, 0.0),
            veil_fade: Anim::still(0.0),
            drop: Anim::still(0.0),
            rows_at: Vec::new(),
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
            open_apps: Vec::new(),
            unix_now: 0,
            toast: None,
            focus_changed: false,
            mv: Move::default(),
            last_move: String::new(),
            refresh_ms: 20.0,
            reduce_motion: false,
            lab: None,
            panel_open: false,
            live: Vec::new(),
            live_at: 0,
            in_live: false,
            live_slot: 0,
            preview_showing: None,
            preview_loading: None,
            live_offset: Anim::still(0.0),
            dock: false,
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

    fn blur_strengths(d: &Draw) -> [i32; 5] {
        let glass_reach = if d.liquid { d.look.liquid_blur } else { 1.0 };
        [d.look.glass_halvings as i32, d.look.glass_passes as i32, d.look.veil_halvings as i32, d.look.veil_passes as i32, (glass_reach * 1000.0) as i32]
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
            d.copy_printed(raw, (1.0 / shape, 1.0, (1.0 - 1.0 / shape) / 2.0, 0.0), mul, self.print);
        } else {
            d.copy_printed(raw, (1.0, shape, 0.0, (1.0 - shape) / 2.0), mul, self.print);
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
        self.glass = d.blurred(half, strengths[0], strengths[1], strengths[4] as f32 / 1000.0, 1.0);
        self.veil = d.blurred(half, strengths[2], strengths[3], 1.0, 1.0 - VEIL_TINT);
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
        if Self::plate_look(d) != self.plates_made_with {
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
        self.engaged && !self.in_live && matches!(self.zone, Zone::Rows | Zone::Menu) && !(self.zone == Zone::Menu && self.menu.from_folder)
    }

    fn live_focused(&self) -> bool {
        self.engaged && self.in_live && self.zone == Zone::Rows
    }

    fn lower(&self) -> bool {
        self.engaged && matches!(self.zone, Zone::Rows | Zone::Menu) && (self.place_of(self.focus).0 > 0 || self.in_live) && !(self.zone == Zone::Menu && self.menu.from_folder)
    }

    pub fn live_preview_box(&self) -> Option<(String, [f32; 4])> {
        if !self.live_focused() {
            return None;
        }
        let shown = &self.live.get(self.live_at)?.shown;
        if shown.id == GUIDE_TILE {
            return None;
        }
        let offset = live_offset(self.live_at, self.live.len(), self.live_slot);
        let left = ROWS_EDGE + self.live_at as f32 * (TILE_W + TILE_GAP) - offset;
        let top = self.live_top() - self.shift.target() + self.drop.target() * GRID_DROP;
        let (w, h) = (TILE_W * LIFT_GRID, TILE_H * LIFT_GRID);
        let (x, y) = (left + TILE_W / 2.0 - w / 2.0, top + TILE_H / 2.0 - LIFT_RISE * LIFT_GRID - h / 2.0);
        Some((shown.id.clone(), [x, y, w, h]))
    }

    fn row_target(&self) -> f32 {
        -self.shift.target() + self.drop.target() * GRID_DROP
    }

    fn place_rows(&mut self) {
        let target = self.row_target();
        self.rows_at = vec![Anim::still(target); self.row_count() + 1];
    }

    fn peel_rows(&mut self, ms: f32, delay: f32, catch_up: f32, curve: &crate::anim::Curve, now: u32) {
        let target = self.row_target();
        self.rows_at.resize(self.row_count() + 1, Anim::still(target));
        for (row, at) in self.rows_at.iter_mut().enumerate().skip(1) {
            let wait = delay * row as f32;
            at.go_after(wait, target, (ms - wait * catch_up).max(ms * 0.3), curve, now);
        }
    }

    fn row_offset(&self, row: usize, now: u32) -> f32 {
        let shift = self.shift.at(now);
        let asleep = self.sleep.at(now) * SLEEP_DROP;
        if row == 0 {
            return asleep - shift;
        }
        asleep + self.rows_at.get(row).map_or(-shift + self.drop.at(now) * GRID_DROP, |at| at.at(now))
    }

    pub fn go_to_sleep(&mut self, clock_at: (f32, f32), now: u32) -> u32 {
        self.sleep_clock = clock_at;
        let ms = self.ms(SLEEP_MS);
        self.sleep.go(1.0, ms, &SLEEP_CURVE, now);
        ms as u32
    }

    pub fn wake(&mut self, now: u32) {
        if self.sleep.target() > 0.0 {
            self.sleep.go(0.0, self.ms(WAKE_MS), &SLEEP_CURVE, now);
        }
    }

    fn live_top(&self) -> f32 {
        row_top(self.row_count()) + LIVE_TITLE_H
    }

    fn shift_for(&self, row: usize) -> f32 {
        if self.in_live {
            (BAR_TILES_TOP - GRID_TOP).max(self.live_top() - GRID_BOTTOM + LIVE_BELOW)
        } else {
            lower_shift(row)
        }
    }

    fn settle(&mut self, _now: u32) {
        let lower = self.lower();
        let slid = if lower { 1.0 } else { 0.0 };
        let shown = self.focused_tile();
        for (at, tile) in self.tiles.iter_mut().enumerate() {
            tile.focus.set(if shown && at == self.focus { 1.0 } else { 0.0 });
        }
        let (row, col) = self.place_of(self.focus);
        self.shift.set(if lower { self.shift_for(row) } else { 0.0 });
        let live_shown = self.live_focused();
        for (at, live) in self.live.iter_mut().enumerate() {
            live.tile.focus.set(if live_shown && at == self.live_at { 1.0 } else { 0.0 });
        }
        self.live_offset.set(live_offset(self.live_at, self.live.len(), self.live_slot));
        self.veil_fade.set(slid);
        self.drop.set(slid);
        self.place_rows();
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
        let live_shown = self.live_focused();
        for (at, live) in self.live.iter_mut().enumerate() {
            live.tile.focus.go(if live_shown && at == self.live_at { 1.0 } else { 0.0 }, focus_ms, &EASE_OUT, now);
        }
        let offset = live_offset(self.live_at, self.live.len(), self.live_slot);
        self.live_offset.go(offset, self.ms(look.scroll_ms), &EASE_SCROLL, now);
        let (row, col) = self.place_of(self.focus);
        if row == 0 {
            self.shine_x.go(tile_centre(col), self.ms(look.scroll_ms), &EASE_SCROLL, now);
        }
        self.shine_alpha.go(if shown && row == 0 { 1.0 } else { 0.0 }, self.ms(200.0), &EASE, now);
        let shift = if lower { self.shift_for(row) } else { 0.0 };
        if lower != was_lower {
            self.shift.go(shift, self.ms(if lower { look.down_ms } else { look.up_ms }), &if lower { [look.down_x1, look.down_y1, look.down_x2, look.down_y2] } else { [look.up_x1, look.up_y1, look.up_x2, look.up_y2] }, now);
            self.veil_fade.go(slid, self.ms(if lower { look.veil_down_ms } else { look.veil_up_ms }), &EASE_VEIL, now);
            let dock_ms = self.ms(if lower { look.down_ms } else { look.up_ms });
            self.drop.go(slid, dock_ms, &if lower { [look.down_x1, look.down_y1, look.down_x2, look.down_y2] } else { [look.up_x1, look.up_y1, look.up_x2, look.up_y2] }, now);
            let delay = self.ms(if lower { look.peel_down_ms } else { look.peel_up_ms });
            let curve = if lower { [look.peel_down_x1, look.peel_down_y1, look.peel_down_x2, look.peel_down_y2] } else { [look.peel_up_x1, look.peel_up_y1, look.peel_up_x2, look.peel_up_y2] };
            self.peel_rows(dock_ms, delay, look.peel_catch, &curve, now);
            self.status_alpha.go_after(self.ms(if lower { look.hide_top_wait_ms } else { look.show_top_wait_ms }), 1.0 - slid, self.ms(look.fade_ms), &EASE, now);
        } else if shift != was_shift {
            self.shift.go(shift, self.ms(look.scroll_ms), &EASE_SCROLL, now);
            self.peel_rows(self.ms(look.scroll_ms), 0.0, 0.0, &EASE_SCROLL, now);
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
        if self.zone != Zone::Rows || self.in_live {
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
            if self.moving.is_some() || !matches!(self.zone, Zone::Rows | Zone::Top) || n == 0 || n > 6 || n > self.bar_count {
                return None;
            }
            self.engaged = true;
            self.in_live = false;
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
        if self.in_live {
            return self.live_key(key);
        }
        let row = self.place_of(self.focus).0;
        match key {
            Key::Down if row + 1 == self.row_count() && !self.live.is_empty() => {
                self.in_live = true;
                self.live_at = self.place_of(self.focus).1.min(self.live.len() - 1);
                self.focus_changed = true;
            }
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

    fn live_key(&mut self, key: Key) -> Option<Action> {
        let last_row = self.row_count() - 1;
        match key {
            Key::Left if self.live_at > 0 => self.live_at -= 1,
            Key::Right if self.live_at + 1 < self.live.len() => self.live_at += 1,
            Key::Up => {
                self.in_live = false;
                self.focus = first_of(last_row, self.bar_count, self.cols) + self.live_at.min(row_len(last_row, self.tiles.len(), self.bar_count, self.cols).max(1) - 1);
            }
            Key::Back => {
                self.in_live = false;
                self.focus = self.live_at.min(self.bar_count.saturating_sub(1));
            }
            Key::Ok => {
                let id = &self.live.get(self.live_at)?.shown.id;
                return Some(if id == GUIDE_TILE { Action::Guide } else { Action::Channel(id.clone()) });
            }
            _ => return None,
        }
        self.focus_changed = true;
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
        (0..self.tiles.len()).find_map(|at| {
            let (row, col) = self.place_of(at);
            let top = row_top(row) + self.row_offset(row, now);
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
                self.in_live = false;
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
        self.in_live = false;
        let col = self.place_of(self.focus).1;
        self.focus = col.min(self.bar_count.saturating_sub(1));
        self.animate_to_state(d, was_lower, was_shift, now);
    }

    pub fn go_to_grid(&mut self, d: &Draw, now: u32) {
        if self.tiles.len() <= self.bar_count {
            return;
        }
        let (was_lower, was_shift) = (self.lower(), self.shift.to_value());
        self.engaged = true;
        self.zone = Zone::Rows;
        self.moving = None;
        self.in_live = false;
        self.focus = self.bar_count;
        self.animate_to_state(d, was_lower, was_shift, now);
    }

    pub fn go_home(&mut self, d: &Draw, now: u32) {
        let (was_lower, was_shift) = (self.lower(), self.shift.to_value());
        self.zone = Zone::Rows;
        self.moving = None;
        self.folder = None;
        self.engaged = false;
        self.in_live = false;
        self.focus = 0;
        self.animate_to_state(d, was_lower, was_shift, now);
    }

    pub fn open_as_dock(&mut self, d: &mut Draw, tint: Tex, app: Option<&str>, now: u32) {
        self.dock = true;
        self.glass = tint;
        self.veil = tint;
        d.glass_backdrop = tint;
        d.veil_backdrop = tint;
        self.zone = Zone::Rows;
        self.moving = None;
        self.folder = None;
        self.in_live = false;
        self.engaged = true;
        self.focus = dock_start(&self.tiles.iter().take(self.bar_count).map(|t| &t.app).collect::<Vec<_>>(), app);
        self.settle(now);
        self.sleep = Anim::still(1.0);
        self.sleep.go(0.0, self.ms(WAKE_MS), &SLEEP_CURVE, now);
    }

    pub fn drop_dock(&mut self, now: u32) {
        self.sleep.go(1.0, self.ms(DOCK_DROP_MS), &SLEEP_CURVE, now);
    }

    pub fn dock_focus(&self) -> Option<(usize, &Shown)> {
        let tile = self.tiles.get(self.focus).filter(|_| self.focus < self.bar_count)?;
        Some((self.focus, &tile.app))
    }

    pub fn dock_select(&mut self, d: &Draw, at: usize, now: u32) -> bool {
        if at >= self.bar_count {
            return false;
        }
        let (was_lower, was_shift) = (self.lower(), self.shift.to_value());
        self.focus = at;
        self.animate_to_state(d, was_lower, was_shift, now);
        true
    }

    pub fn from_dock(&mut self, d: &Draw, col: usize, exit: DockExit, now: u32) -> Option<Action> {
        self.zone = Zone::Rows;
        self.moving = None;
        self.folder = None;
        self.in_live = false;
        self.engaged = true;
        self.focus = col.min(self.bar_count.saturating_sub(1));
        self.settle(now);
        match exit {
            DockExit::Bar => None,
            DockExit::Down => self.key(d, Key::Down, now),
            DockExit::Up => self.key(d, Key::Up, now),
            DockExit::Open => self.key(d, Key::Ok, now),
        }
    }

    pub fn bar_focus(&self) -> Option<(usize, usize)> {
        (self.zone == Zone::Rows && !self.engaged && self.place_of(self.focus).0 == 0).then_some((self.focus, self.bar_count))
    }

    pub fn moving(&self, now: u32) -> bool {
        let anims = [&self.wallpaper_fade, &self.shift, &self.sleep, &self.veil_fade, &self.drop, &self.shine_x, &self.shine_alpha, &self.status_alpha, &self.panel_alpha];
        anims.iter().any(|a| a.busy(now))
            || self.rows_at.iter().any(|a| a.busy(now))
            || self.tiles.iter().any(|t| t.focus.busy(now))
            || self.live.iter().any(|l| l.tile.focus.busy(now))
            || self.live_offset.busy(now)
            || self.preview_loading.is_some()
            || self.top_focus.iter().any(|a| a.busy(now))
            || (self.moving.is_some() && !self.reduce_motion)
            || self.toast.as_ref().is_some_and(|(_, _, until)| (until.wrapping_sub(now) as i32) > -300)
    }

    pub fn where_now(&self) -> Value {
        let mut focused = Vec::new();
        if self.engaged {
            match self.zone {
                Zone::Top => focused.push(json!({"part": "pill", "label": TOP_ITEMS[self.top_index].1, "index": self.top_index, "in": "top"})),
                Zone::Rows if self.in_live => {
                    if let Some(live) = self.live.get(self.live_at) {
                        focused.push(json!({"part": "tile", "label": live.tile.app.title, "id": live.shown.id, "index": self.live_at, "in": "Live TV"}));
                    }
                }
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
        } else if let Some(glyph) = tile.app.glyph {
            glyph_picture(glyph)
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
        p.set1(c"u_wide", if tile.app.inside.is_some() || tile.app.id.starts_with(CHANNEL_PREFIX) { 1.0 } else { 0.0 });
    }

    fn resting_tile(d: &mut Draw, tile: &mut Tile, icon: Tex, shade: f32) -> Tex {
        let key = resting_key(&tile.app.id, shade);
        if let Some(resting) = d.pictures.find(&key) {
            return resting;
        }
        let (w, h) = (TILE_W + 2.0 * TILE_MARGIN, TILE_H + 2.0 * TILE_MARGIN);
        let resting = d.gfx.blank(w as i32, h as i32);
        unsafe { glDisable(GL_BLEND) }
        d.gfx.at_design_scale_into(&resting);
        Self::tile_uniforms(d, tile, icon, (TILE_MARGIN, TILE_MARGIN, TILE_W, TILE_H), TILE_RADIUS, shaded(SHADOW_REST, shade), 0.0);
        d.gfx.quad(&d.p.tile, 0.0, 0.0, w, h);
        d.keep(&key, resting);
        resting
    }

    fn plate_look(d: &Draw) -> [f32; 13] {
        let l = &d.look;
        [l.bezel, l.edge_shape, l.specular, l.light_angle, l.colour_rims, l.rim_light, l.sheen, l.glow, l.side_width, l.side_bend, l.dock_colour_rims, l.dock_glow, l.dock_glow_reach]
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
            program.set1(c"u_edge_shape", d.look.edge_shape);
            program.set2(c"u_side", d.look.side_width, d.look.side_bend);
            let [lx, ly] = d.light();
            let (rims, glow, reach) = if self.dock { (d.look.dock_colour_rims, d.look.dock_glow, d.look.dock_glow_reach) } else { (d.look.colour_rims, d.look.glow, 14.0) };
            program.set4(c"u_light", lx, ly, d.look.specular, rims);
            program.set3(c"u_finish", [d.look.rim_light, d.look.sheen, glow]);
            program.set1(c"u_glow_reach", reach);
            d.gfx.quad(program, 0.0, 0.0, w, h);
        }
        self.bar_made_with = d.look.bezel * d.look.side_width.max(1.0);
        self.plates_made_with = Self::plate_look(d);
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

    fn tile_focus(&self, at: usize, now: u32) -> f32 {
        match self.lab {
            Some(focused) => if focused && at == self.focus { 1.0 } else { 0.0 },
            None => self.tiles[at].focus.at(now),
        }
    }

    fn tile_box(&self, at: usize, now: u32) -> (f32, f32, f32, f32, f32) {
        let (row, col) = self.place_of(at);
        let f = self.tile_focus(at, now);
        let left = tile_left(col);
        let top = row_top(row) + self.row_offset(row, now);
        let lifting = self.moving.as_ref().is_some_and(|id| *id == self.tiles[at].app.id);
        let scale = if lifting { LIFT_MOVING } else { 1.0 + ((if row > 0 { LIFT_GRID } else { LIFT_BAR }) - 1.0) * f };
        let rise = if lifting { MOVING_RISE } else { LIFT_RISE * f * scale };
        let (w, h) = (TILE_W * scale, TILE_H * scale);
        (left + TILE_W / 2.0 - w / 2.0, top + TILE_H / 2.0 - rise - h / 2.0, w, h, f)
    }

    fn draw_tile(&self, d: &Draw, at: usize, resting: Tex, icon: Tex, now: u32) {
        let tile = &self.tiles[at];
        let (row, col) = self.place_of(at);
        let lifting = self.moving.as_ref().is_some_and(|id| *id == tile.app.id);
        let (x, y, w, h, f) = self.tile_box(at, now);
        if f <= 0.0 && !lifting {
            let top = row_top(row) + self.row_offset(row, now);
            let (rw, rh) = (TILE_W + 2.0 * TILE_MARGIN, TILE_H + 2.0 * TILE_MARGIN);
            return d.picture(resting, tile_left(col) - TILE_MARGIN, top - TILE_MARGIN, rw, rh, 1.0);
        }
        let lifted = if row > 0 { SHADOW_GRID } else { SHADOW_BAR };
        let shadow = shaded([0, 1, 2].map(|i| SHADOW_REST[i] + (lifted[i] - SHADOW_REST[i]) * f.max(if lifting { 1.0 } else { 0.0 })), if row > 0 { d.look.grid_shadow } else { 1.0 });
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
        program.set1(c"u_frost", d.look.frost);
        program.set1(c"u_clear", if self.dock { 1.0 } else { 0.0 });
        program.set3(c"u_mag", [BAR_X + BAR_W / 2.0, plate.1 + plate.3 / 2.0, d.magnify()]);
    }

    fn draw_bar(&self, d: &Draw, lens: Tex, gloss: Tex, bar_y: f32, veiled: f32, now: u32) {
        let (px, py) = (BAR_X - PLATE_MARGIN, bar_y - PLATE_MARGIN);
        let (pw, ph) = (BAR_W + 2.0 * PLATE_MARGIN, BAR_H + 2.0 * PLATE_MARGIN);
        let inset = self.bar_made_with.max(22.0) + 1.0;
        let (ix, iy, iw, ih) = (BAR_X + inset, bar_y + inset, BAR_W - 2.0 * inset, BAR_H - 2.0 * inset);

        unsafe {
            if self.dock {
                glEnable(GL_BLEND);
            } else {
                glDisable(GL_BLEND);
            }
        }
        self.bar_uniforms(d, &d.p.bar_middle, gloss, (px, py, pw, ph), veiled);
        d.gfx.quad(&d.p.bar_middle, ix, iy, iw, ih);

        unsafe { glEnable(GL_BLEND) }
        let rim = &d.p.bar_rim;
        self.bar_uniforms(d, rim, gloss, (px, py, pw, ph), veiled);
        rim.bind(3, c"u_lens", lens.tex);
        rim.set2(c"u_bend", d.bend(), d.look.fringe);
        d.gfx.quad(rim, px, py, pw, iy - py);
        d.gfx.quad(rim, px, iy + ih, pw, py + ph - iy - ih);
        d.gfx.quad(rim, px, iy, ix - px, ih);
        d.gfx.quad(rim, ix + iw, iy, px + pw - ix - iw, ih);

        let glow = match self.lab {
            Some(focused) => if focused { 1.0 } else { 0.0 },
            None => self.shine_alpha.at(now),
        };
        if glow > 0.0 {
            let reach = BAR_W / 2.0 - BAR_RADIUS - 130.0;
            let x = (self.shine_x.at(now) - DESIGN_W / 2.0).clamp(-reach, reach) + DESIGN_W / 2.0 - 130.0;
            d.p.shine.activate();
            d.p.shine.set4(c"u_box", x, bar_y, 260.0, 28.0);
            d.p.shine.set1(c"u_alpha", glow);
            d.gfx.quad(&d.p.shine, x, bar_y, 260.0, 28.0);
        }
    }

    fn draw_open_dots(&self, d: &Draw, row: usize, now: u32) {
        for at in (0..self.tiles.len()).filter(|&at| self.place_of(at).0 == row) {
            let app = &self.tiles[at].app;
            let open = |id: &String| self.open_apps.contains(id);
            if !(open(&app.id) || app.inside.as_ref().is_some_and(|inside| inside.iter().any(|a| open(&a.id)))) {
                continue;
            }
            let shown = 1.0 - self.tile_focus(at, now);
            if shown <= 0.0 {
                continue;
            }
            let col = self.place_of(at).1;
            let (cx, cy) = (tile_centre(col), row_top(row) + self.row_offset(row, now) - BAR_ABOVE_TILES / 2.0);
            d.fill(cx - OPEN_DOT / 2.0 - 3.0, cy - OPEN_DOT / 2.0 - 3.0, OPEN_DOT + 6.0, OPEN_DOT + 6.0, OPEN_DOT / 2.0 + 3.0, [0.0, 0.0, 0.0, 0.2 * shown]);
            d.fill(cx - OPEN_DOT / 2.0, cy - OPEN_DOT / 2.0, OPEN_DOT, OPEN_DOT, OPEN_DOT / 2.0, [1.0, 1.0, 1.0, 0.92 * shown]);
        }
    }

    fn draw_labels(&mut self, d: &mut Draw, row: usize, now: u32) {
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
            let top = row_top(row) + self.row_offset(row, now);
            let baseline = top + if row == 0 { BAR_LABEL_BELOW } else { GRID_LABEL_BELOW - GRID_LABEL_RISE * f };
            let title = self.tiles[at].app.title.clone();
            if let Some(text) = d.text(&title, &TILE_LABEL) {
                text.draw(d, tile_centre(col) - text.advance / 2.0, baseline, f);
            }
        }
    }

    fn draw_sleeping_clock(&mut self, d: &mut Draw, alpha: f32, asleep: f32) {
        let (time, period, _) = self.clock.clone();
        let time_style = if d.fonts.is_sf("600-clock") { crate::saver::TIME } else { crate::saver::TIME_TNUM };
        let mix = |a: f32, b: f32| a + (b - a) * asleep;
        let scale = mix(TIME.px as f32 / time_style.px as f32, 1.0);
        let (x, baseline) = (mix(TIME_LEFT, self.sleep_clock.0), mix(TIME_BASELINE, self.sleep_clock.1));
        let Some(time) = d.text(&time, &time_style) else { return };
        time.draw_scaled(d, x, baseline, scale, alpha);
        if let Some(p) = d.text(&period, &crate::saver::PERIOD) {
            let period_scale = mix(PERIOD.px as f32 / crate::saver::PERIOD.px as f32, 1.0);
            p.draw_scaled(d, x + time.advance * scale + mix(PERIOD_GAP, crate::saver::PERIOD_GAP), baseline - mix(9.0, crate::saver::PERIOD_RISE), period_scale, alpha);
        }
    }

    fn draw_status(&mut self, d: &mut Draw, alpha: f32, clock_alpha: f32, asleep: f32) {
        if asleep > 0.0 {
            self.draw_sleeping_clock(d, clock_alpha, asleep);
        }
        if alpha <= 0.0 {
            return;
        }
        let (time, period, date) = self.clock.clone();
        let time_style = if d.fonts.is_sf("600-clock") { TIME } else { TIME_TNUM };
        if let Some(time) = d.text(&time, &time_style).filter(|_| asleep <= 0.0) {
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
        if self.forecast == Forecast::Large {
            let hours = weather::hours_ahead(&w, self.unix_now, crate::saver::LARGE_HOURS);
            if hours.is_empty() {
                return;
            }
            let large = crate::saver::Large::lay_out(d, &w, &hours, self.twenty_four_hour);
            return large.draw(d, &w, &hours, self.twenty_four_hour, ROWS_EDGE + large.width, CAST_BOTTOM, alpha);
        }
        let hours = weather::hours_ahead(&w, self.unix_now, CAST_HOURS);
        if hours.is_empty() {
            return;
        }
        let width = hours.len() as f32 * (CAST_COL + CAST_GAP) - CAST_GAP;
        let top = CAST_BOTTOM - 103.8;
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
            let f = match self.lab {
                Some(focused) => if focused && at == 0 { 1.0 } else { 0.0 },
                None => self.top_focus[at].at(now),
            };
            let scale = 1.0 + (PILL_LIFT - 1.0) * f;
            let size = PILL * scale;
            let (cx, cy) = (pill_left(at) + PILL / 2.0, PILL_TOP + PILL / 2.0);
            d.glass(&Glass { x: cx - size / 2.0, y: cy - size / 2.0, w: size, h: size, radius: size / 2.0, blur: f, sheen: 1.0 + 1.6 * f, rim: 1.0 + 0.6 * f, alpha });
            let icon = d.glyph(glyph, 28, 1.8);
            let g = 28.0 * scale;
            d.inked_picture(icon, cx - g / 2.0, cy - g / 2.0, g, g, [1.0, 1.0, 1.0, 0.62 * alpha]);
            if f > 0.0 && !self.panel_open && self.lab.is_none() {
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
        let asleep = self.sleep.at(now);
        let clock_alpha = self.status_alpha.at(now);
        let status_alpha = clock_alpha * (1.0 - (asleep * 2.0).min(1.0));
        let panel_alpha = self.panel_alpha.at(now);

        let mut pictures: Vec<Option<(Tex, Tex)>> = Vec::with_capacity(self.tiles.len());
        for at in 0..self.tiles.len() {
            let row = self.place_of(at).0;
            let top = row_top(row) + self.row_offset(row, now);
            let seen = top - 80.0 <= DESIGN_H && top + TILE_H + 80.0 >= 0.0;
            pictures.push(if seen {
                let tile = &mut self.tiles[at];
                let icon = Self::icon_of(d, tile);
                let shade = if row > 0 { d.look.grid_shadow } else { 1.0 };
                Some((Self::resting_tile(d, tile, icon, shade), icon))
            } else {
                None
            });
        }
        if self.live_top() + self.row_offset(self.row_count(), now) - LIVE_TITLE_H <= DESIGN_H {
            let offset = self.live_offset.at(now);
            for (_, live) in self.live.iter_mut().enumerate().filter(|(at, l)| l.shown.id != GUIDE_TILE && live_in_view(*at, offset)) {
                let icon = Self::channel_icon(d, &live.tile);
                let shade = d.look.grid_shadow;
                Self::resting_tile(d, &mut live.tile, icon, shade);
            }
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
        let in_lab = self.lab.is_some();
        if !in_lab {
            self.draw_forecast(d, status_alpha);
            self.draw_caption(d, status_alpha);
        }
        let focus_row = self.place_of(self.focus).0;
        let lifted = self.moving.as_ref().and_then(|id| self.tiles.iter().position(|t| &t.app.id == id));
        self.draw_live(d, now);
        let rows = self.row_count();
        for row in (0..rows).rev() {
            if row == 0 && self.sharp.tex != 0 {
                self.draw_bar(d, lens, gloss, BAR_TILES_TOP - BAR_ABOVE_TILES - shift + asleep * SLEEP_DROP, veiled, now);
                unsafe { glEnable(GL_BLEND) }
            }
            let in_row = |at: &usize| place(*at, self.bar_count, self.cols).0 == row;
            let last = if lifted.is_some_and(|at| self.place_of(at).0 == row) { lifted } else { (row == focus_row).then_some(self.focus) };
            for at in (0..self.tiles.len()).filter(in_row).filter(|&at| Some(at) != last).chain(last) {
                if let Some(Some((resting, icon))) = pictures.get(at) {
                    self.draw_tile(d, at, *resting, *icon, now);
                }
            }
            if !in_lab {
                self.draw_open_dots(d, row, now);
                self.draw_labels(d, row, now);
            }
        }

        if !in_lab {
            self.draw_status(d, status_alpha, clock_alpha + (1.0 - clock_alpha) * asleep, asleep);
        }
        self.draw_top(d, status_alpha, now);
        if in_lab {
            return self.draw_toast(d, now);
        }
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

    pub fn draw_dock(&mut self, d: &mut Draw, now: u32) {
        d.pictures.next_frame();
        let mut pictures = Vec::with_capacity(self.bar_count);
        for tile in self.tiles.iter_mut().take(self.bar_count) {
            let icon = Self::icon_of(d, tile);
            pictures.push((Self::resting_tile(d, tile, icon, 1.0), icon));
        }
        let (lens, gloss) = self.bar_plates(d);
        d.gfx.into_screen();
        unsafe {
            glClearColor(0.0, 0.0, 0.0, 0.0);
            glClear(GL_COLOR_BUFFER_BIT);
            glBlendFuncSeparate(GL_ONE, GL_ONE_MINUS_SRC_ALPHA, GL_ONE, GL_ONE_MINUS_SRC_ALPHA);
        }
        self.draw_bar(d, lens, gloss, BAR_TILES_TOP - BAR_ABOVE_TILES + self.sleep.at(now) * SLEEP_DROP, 0.0, now);
        unsafe { glEnable(GL_BLEND) }
        let last = (self.focus < self.bar_count).then_some(self.focus);
        for at in (0..self.bar_count).filter(|&at| Some(at) != last).chain(last) {
            let (resting, icon) = pictures[at];
            self.draw_tile(d, at, resting, icon, now);
        }
        self.draw_open_dots(d, 0, now);
        self.draw_labels(d, 0, now);
    }

    pub fn set_live(&mut self, d: &mut Draw, shown: Vec<LiveShown>, now: u32) {
        let slot = livetv::live_slot(self.cols, TILE_W, TILE_GAP);
        if shown.iter().eq(self.live.iter().map(|l| &l.shown)) && slot == self.live_slot {
            return;
        }
        let mut old: Vec<LiveTile> = std::mem::take(&mut self.live);
        for s in shown {
            let id = if s.id == GUIDE_TILE { GUIDE_TILE.to_string() } else { format!("{CHANNEL_PREFIX}{}", s.id) };
            let kept = old.iter().position(|l| l.tile.app.id == id && l.shown.name == s.name && l.shown.logo == s.logo).map(|at| old.remove(at));
            let tile = match kept {
                Some(l) => l.tile,
                None => {
                    d.let_go(|key| key == icon_key(&id) || key == tile_key(&id) || key.starts_with(&format!("{}:", tile_key(&id))));
                    let color = (id != GUIDE_TILE).then(|| crate::card::colour_of(&s.name));
                    let title = if id == GUIDE_TILE { "Guide".to_string() } else { s.name.clone() };
                    Tile { bg: color.unwrap_or(TILE_WITHOUT_COLOUR), app: Shown { id, title, icon_file: s.logo.clone(), glyph: None, color, inside: None }, focus: Anim::still(0.0) }
                }
            };
            self.live.push(LiveTile { shown: s, tile });
        }
        self.live_slot = slot;
        if self.live.is_empty() && self.in_live {
            self.in_live = false;
            self.focus = first_of(self.row_count() - 1, self.bar_count, self.cols);
        }
        self.live_at = self.live_at.min(self.live.len().saturating_sub(1));
        self.settle(now);
    }

    fn channel_icon(d: &mut Draw, tile: &Tile) -> Tex {
        let key = icon_key(&tile.app.id);
        if let Some(icon) = d.pictures.find(&key) {
            return icon;
        }
        let face = crate::card::channel_card(&d.fonts, &tile.app.title, tile.app.icon_file.as_deref(), &crate::card::TILE_CARD);
        let picture = d.gfx.upload_rgba(&face.rgba, face.w, face.h);
        d.keep(&key, picture);
        picture
    }

    fn draw_live(&mut self, d: &mut Draw, now: u32) {
        if self.live.is_empty() {
            return;
        }
        let top = self.live_top() + self.row_offset(self.row_count(), now);
        if top - LIVE_TITLE_H > DESIGN_H {
            return;
        }
        let lit = self.live_focused() || !self.engaged;
        if let Some(title) = d.text("Live TV", &ROW_TITLE) {
            title.draw(d, ROWS_EDGE, top - LIVE_TITLE_ABOVE, if lit { 1.0 } else { 0.72 });
        }
        let offset = self.live_offset.at(now);
        let step = TILE_W + TILE_GAP;
        let order: Vec<usize> = (0..self.live.len()).filter(|&at| at != self.live_at).chain(std::iter::once(self.live_at)).collect();
        for at in order {
            let left = ROWS_EDGE + at as f32 * step - offset;
            if !live_in_view(at, offset) {
                continue;
            }
            let f = self.live[at].tile.focus.at(now);
            let scale = 1.0 + (LIFT_GRID - 1.0) * f;
            let (w, h) = (TILE_W * scale, TILE_H * scale);
            let (x, y) = (left + TILE_W / 2.0 - w / 2.0, top + TILE_H / 2.0 - LIFT_RISE * f * scale - h / 2.0);
            if self.live[at].shown.id == GUIDE_TILE {
                d.glass(&Glass { x, y, w, h, radius: TILE_RADIUS * scale, blur: f, sheen: 1.0 + 1.6 * f, rim: 1.0 + 0.6 * f, alpha: 1.0 });
                let g = TILE_H * 0.58 * scale;
                let icon = d.glyph(GUIDE_GLYPH, (TILE_H * 0.58) as i32, 1.6);
                d.inked_picture(icon, x + w / 2.0 - g / 2.0, y + h / 2.0 - g / 2.0, g, g, [INK_ON_PHOTO[0], INK_ON_PHOTO[1], INK_ON_PHOTO[2], 1.0]);
                if f > 0.0 && !self.hide_names {
                    if let Some(text) = d.text("Guide", &TILE_LABEL) {
                        text.draw(d, left + TILE_W / 2.0 - text.advance / 2.0, top + GRID_LABEL_BELOW - GRID_LABEL_RISE * f, f);
                    }
                }
                continue;
            }
            let icon = Self::channel_icon(d, &self.live[at].tile);
            let resting = if f <= 0.0 { d.pictures.find(&resting_key(&self.live[at].tile.app.id, d.look.grid_shadow)) } else { None };
            if let Some(resting) = resting {
                d.picture(resting, left - TILE_MARGIN, top - TILE_MARGIN, TILE_W + 2.0 * TILE_MARGIN, TILE_H + 2.0 * TILE_MARGIN, 1.0);
            } else {
                let shadow = shaded([0, 1, 2].map(|i| SHADOW_REST[i] + (SHADOW_GRID[i] - SHADOW_REST[i]) * f), d.look.grid_shadow);
                let margin = shadow[0] + shadow[1] + 4.0;
                Self::tile_uniforms(d, &self.live[at].tile, icon, (x, y, w, h), TILE_RADIUS * scale, shadow, f);
                d.p.tile.set1(c"u_turn", 0.0);
                d.gfx.quad(&d.p.tile, x - margin, y - margin, w + 2.0 * margin, h + 2.0 * margin);
            }
            if self.preview_showing.as_deref() == Some(self.live[at].shown.id.as_str()) {
                d.hole(x, y, w, h, TILE_RADIUS * scale);
            }
            if self.preview_loading.as_deref() == Some(self.live[at].shown.id.as_str()) {
                d.spinner(x + 20.0 * scale, y + 20.0 * scale, 18.0 * scale, 2.5 * scale, now);
            }
            let shown = self.live[at].shown.clone();
            let done = shown.now.as_ref().map_or(0.0, |p| livetv::progress(p, self.unix_now));
            let (bx, by, bw) = (x + 22.0 * scale, y + h - (16.0 + 6.0) * scale, w - 44.0 * scale);
            d.plate(&BAR_PLATE, bx, by, bw, 6.0 * scale, 1.0);
            if done > 0.0 {
                d.plate(&BAR_FILL, bx, by, (bw * done).max(6.0 * scale), 6.0 * scale, 1.0);
            }
            if f > 0.0 {
                self.draw_live_info(d, &shown, left, top + TILE_H + LIVE_INFO_GAP, f);
            }
        }
    }

    fn draw_live_info(&self, d: &mut Draw, shown: &LiveShown, left: f32, top: f32, alpha: f32) {
        let clock = |t: u64| crate::clock::clock_text(t, self.twenty_four_hour);
        if let Some(line) = d.text_fit(&format!("{} · {}", shown.number, shown.name), &INFO_LINE, LIVE_INFO_W) {
            line.draw(d, left, top + 21.0, 0.78 * alpha);
        }
        let title = match (&shown.now, &shown.next) {
            (Some(now), _) => now.title.clone(),
            (None, Some(_)) => "Off air".to_string(),
            (None, None) if shown.loading => "Loading…".to_string(),
            (None, None) => "No guide data".to_string(),
        };
        if let Some(text) = d.text_fit(&title, &INFO_TITLE, LIVE_INFO_W) {
            text.draw(d, left, top + 61.0, alpha);
        }
        let times_baseline = top + 97.7;
        match (&shown.now, &shown.next) {
            (Some(now), next) => {
                let mut x = left;
                if let Some(span) = d.text(&format!("{} – {}", clock(now.start), clock(now.end)), &INFO_LINE) {
                    span.draw(d, x, times_baseline, 0.9 * alpha);
                    x += span.advance + 16.0;
                }
                let meter_y = times_baseline - 7.7 - 2.5;
                d.plate(&METER, x, meter_y, 160.0, 5.0, 0.9 * alpha);
                d.plate(&METER_FILL, x, meter_y, (160.0 * livetv::progress(now, self.unix_now)).max(5.0), 5.0, 0.9 * alpha);
                x += 160.0 + 16.0;
                if let Some(left_text) = d.text(&livetv::time_left(now, self.unix_now), &INFO_LINE) {
                    left_text.draw(d, x, times_baseline, 0.9 * alpha);
                }
                if let Some(next) = next {
                    if let Some(text) = d.text_fit(&format!("Next {} · {}", clock(next.start), next.title), &INFO_LINE, LIVE_INFO_W) {
                        text.draw(d, left, top + 134.0, 0.7 * alpha);
                    }
                }
            }
            (None, Some(next)) => {
                if let Some(text) = d.text(&format!("Starts at {}", clock(next.start)), &INFO_LINE) {
                    text.draw(d, left, times_baseline, 0.9 * alpha);
                }
            }
            (None, None) => {}
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
        crate::note!("move {}", self.last_move);
        true
    }
}

pub fn dock_start(bar: &[&Shown], app: Option<&str>) -> usize {
    let Some(app) = app else { return 0 };
    bar.iter().position(|shown| shown.id == app || shown.inside.as_ref().is_some_and(|inside| inside.iter().any(|a| a.id == app))).unwrap_or(0)
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
        Shown { id: id.into(), title: id.into(), icon_file: None, glyph: None, color: None, inside: None }
    }

    #[test]
    fn the_dock_starts_on_the_app_in_front_or_the_folder_holding_it_else_the_first() {
        let mut folder = app("folder:Games");
        folder.inside = Some(vec![app("stadia")]);
        let (plex, youtube) = (app("plex"), app("youtube"));
        let bar = [&plex, &folder, &youtube];
        assert_eq!(dock_start(&bar, Some("youtube")), 2);
        assert_eq!(dock_start(&bar, Some("stadia")), 1);
        assert_eq!(dock_start(&bar, Some("com.webos.app.livetv")), 0);
        assert_eq!(dock_start(&bar, None), 0);
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
    fn the_live_tv_row_keeps_its_focus_in_place_then_scrolls_and_stops_with_the_last_two_in_view() {
        let step = TILE_W + TILE_GAP;
        assert_eq!(live_offset(3, 12, 3), 0.0);
        assert_eq!(live_offset(5, 12, 3), 2.0 * step);
        assert_eq!(live_offset(11, 12, 3), 7.0 * step);
        assert_eq!(live_offset(1, 2, 0), 0.0);
    }

    #[test]
    fn only_the_live_tv_tiles_on_screen_are_drawn_however_many_channels_there_are() {
        let in_view: Vec<usize> = (0..209).filter(|&at| live_in_view(at, 0.0)).collect();
        assert_eq!(in_view, (0..7).collect::<Vec<_>>());
        let far = live_offset(200, 209, 3);
        assert!((0..209).filter(|&at| live_in_view(at, far)).all(|at| at > 195));
    }

    #[test]
    fn down_from_the_last_row_reaches_live_tv_where_ok_opens_the_guide_or_a_channel() {
        let mut home = Home::new();
        home.tiles = ["a", "b", "c"].iter().map(|id| Tile { app: app(id), bg: [0.0; 3], focus: Anim::still(0.0) }).collect();
        home.bar_count = 3;
        let shown = |id: &str| LiveShown { id: id.into(), number: String::new(), name: id.into(), logo: None, now: None, next: None, loading: false };
        home.live = [GUIDE_TILE, "1_41_1"].iter().map(|id| LiveTile { shown: shown(id), tile: Tile { app: app(id), bg: [0.0; 3], focus: Anim::still(0.0) } }).collect();
        home.engaged = true;
        home.react(Key::Right);
        home.react(Key::Down);
        assert_eq!(home.react(Key::Ok), Some(Action::Channel("1_41_1".into())));
        home.react(Key::Left);
        assert_eq!(home.react(Key::Ok), Some(Action::Guide));
        home.react(Key::Up);
        assert_eq!(home.react(Key::Ok), Some(Action::Open("a".into())));
    }

    #[test]
    fn hours_read_as_the_clock_does() {
        assert_eq!(hour_text("2026-11-24T15:00", true), "15:00");
        assert_eq!(hour_text("2026-11-24T15:00", false), "3 PM");
        assert_eq!(hour_text("2026-11-24T00:00", false), "12 AM");
    }
}
