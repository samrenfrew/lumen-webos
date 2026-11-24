use crate::anim::{Anim, EASE_OUT, EASE_SCROLL};
use crate::apps::{background_of, kept_icon_name, App, Icon};
use crate::draw::{decoded, Decoded, Draw, Style};
use crate::ffi::*;
use crate::gfx::{Tex, DESIGN_H, DESIGN_W};
use crate::home::Key;
use crate::livetv::DEFAULT_ROW_CHANNELS;
use crate::settings_options::{self as opts, cities_in, group_named, City, Facts, Group, Kind, Opt, GROUPS};
use crate::settings_tv::{self as tv, Shared};
use crate::text::Shadow;
use crate::thumbs::Thumbs;
use crate::{root, store};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::rc::Rc;

const NATIVE_DIR: &str = "/media/developer/apps/usr/palm/applications/com.samrenfrew.lumen.native";
const ROWS_EDGE: f32 = 125.0;
const HEADER_TOP: f32 = 84.0;
const VIEW_X: f32 = 810.0;
const VIEW_TOP: f32 = 170.0;
const VIEW_H: f32 = 840.0;
const TRACK_TOP: f32 = 12.0;
const ROW_X: f32 = VIEW_X + 50.0;
const ROW_W: f32 = DESIGN_W - ROWS_EDGE - 50.0 - ROW_X;
const ROW_H: f32 = 70.0;
const ROW_STEP: f32 = 84.0;
const ROW_PAD: f32 = 34.0;
const SHADOW_ROOM: f32 = 48.0;
const FADE: f32 = 40.0;
const VALUE_MAX: f32 = 380.0;
const SIDE_W: f32 = 640.0;
const CARD: f32 = 480.0;
const DESCRIPTION_W: f32 = 540.0;
const DESCRIPTION_LINE: f32 = 26.0 * 1.45;
const LIFT_PRESS: f32 = 1.03;
const LIFT_PILL: f32 = 1.08;
const FOCUS_SHADOW: (f32, f32, f32) = (14.0, 28.0, 0.30);
const SHAPE_MARGIN: f32 = 50.0;
const GRID_COLUMNS: usize = 6;
const TILE_W: f32 = 236.0;
const TILE_H: f32 = 142.0;
const TILE_GAP: f32 = 28.0;
const TILE_RADIUS: f32 = 34.0;
const TILE_MARGIN: f32 = 28.0;
const GRID_INSET: f32 = (DESIGN_W - 2.0 * ROWS_EDGE - (GRID_COLUMNS as f32 * TILE_W + (GRID_COLUMNS - 1) as f32 * TILE_GAP)) / 2.0;
const GRID_LEFT: f32 = ROWS_EDGE + GRID_INSET;
const GRID_RIGHT: f32 = DESIGN_W - ROWS_EDGE - GRID_INSET;
const HEAD_BOTTOM: f32 = HEADER_TOP + 72.0 + 18.0;
const CONTENT_TOP: f32 = HEAD_BOTTOM + 22.0 * 1.2 + 26.0;
const GRID_VIEW_H: f32 = 720.0;
const GRID_PAD: f32 = 16.0;
const GRID_GAP: f32 = 40.0;
const GRID_STEP: f32 = TILE_H + 22.0 + 18.0 * 2.4 + GRID_GAP;
const GRID_END_ROOM: f32 = 30.0;
const LIFT: f32 = 1.14;
const LIFT_RISE: f32 = 7.5;
const LIFTED: f32 = 1.2;
const SHADOW_REST: [f32; 3] = [8.0, 16.0, 0.30];
const SHADOW_LIFT: [f32; 3] = [26.0, 40.0, 0.46];
const TILE_WITHOUT_COLOUR: [f32; 3] = [0.16, 0.17, 0.22];
const INPUT_TILE: [f32; 3] = [0.19, 0.2, 0.24];
const CHIPS_W: f32 = 1640.0;
const CHIP_H: f32 = 12.0 + 22.0 * 1.2 + 12.0;
const CHIPS_H: f32 = CHIP_H + 12.0;
const WP_COLUMNS: usize = 4;
const WP_VIEW_H: f32 = 700.0;
const WP_THUMB_W: f32 = 360.0;
const WP_THUMB_H: f32 = 203.0;
const WP_COL_STEP: f32 = 360.0 + 66.67;
const WP_ROW_STEP: f32 = 203.0 + 70.0;
const WP_GRID_MS: u32 = 250;
const KEY_W: f32 = 72.0;
const KEY_H: f32 = 64.0;
const KEY_GAP: f32 = 10.0;
const KEY_ROWS: [&str; 4] = ["1234567890", "qwertyuiop", "asdfghjkl:", "zxcvbnm./-"];
const KEY_ACTIONS: [&str; 4] = ["ABC", "Delete", "Clear", "Save"];
const HINT_SELECT: &str = "OK adds or removes an item · Hold OK to re-order · Back returns";
const HINT_REORDER: &str = "OK picks up an item · Hold OK or choose Done to finish";
const HINT_HOLDING: &str = "Arrows move it · OK puts it down · Back cancels";
const HINT_WALLPAPER: &str = "Left and right change category · OK picks or unpicks · Back returns";
const WALLPAPER_CATEGORIES: [&str; 2] = ["Landscapes", "Art"];
const IMMICH_SOURCES: [(&str, &str); 5] = [("immich:recent", "Recent"), ("immich:recent-people", "Recent, with people"), ("immich:memories", "On this day"), ("immich:favorites", "Favourites"), ("immich:all", "All photos")];
const CHEVRON: &str = "M7.3 2.4L16.7 12 7.3 21.6";
const TICK: &str = "M5 12.5l4.5 4.5L19 7.5";
const SCHEDULE_KEYS: [&str; 9] = ["brightnessOn", "blDayAt", "blDay", "blEveningAt", "blEvening", "blNightAt", "blNight", "blRamp", "blEye"];

const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
const INK_1: [f32; 3] = [0.933, 0.933, 0.933];
const INK_2: [f32; 3] = [0.667, 0.667, 0.667];
const FOCUS_INK: [f32; 3] = [0.067, 0.067, 0.067];
const LABEL_SHADOW: [Shadow; 2] = [Shadow { drop: 2, blur: 14, strength: 0.55 }, Shadow { drop: 0, blur: 2, strength: 0.25 }];
const TITLE: Style = Style { px: 56, weight: "600", spacing_em: -0.02, feature: "", ink: WHITE, shadows: &[] };
const ROW_TEXT: Style = Style { px: 32, weight: "500", spacing_em: -0.012, feature: "", ink: WHITE, shadows: &[] };
const ROW_FOCUS: Style = Style { ink: FOCUS_INK, ..ROW_TEXT };
const DESCRIPTION: Style = Style { px: 26, weight: "400", spacing_em: 0.0, feature: "", ink: WHITE, shadows: &[] };
const HINT: Style = Style { px: 22, weight: "400", spacing_em: 0.0, feature: "", ink: WHITE, shadows: &[] };
const BUTTON: Style = Style { px: 26, weight: "500", spacing_em: 0.0, feature: "", ink: INK_1, shadows: &[] };
const BUTTON_ON: Style = Style { weight: "600", ..BUTTON };
const BUTTON_FOCUS: Style = Style { weight: "600", ink: FOCUS_INK, ..BUTTON };
const TILE_LABEL: Style = Style { px: 18, weight: "600", spacing_em: -0.015, feature: "", ink: INK_1, shadows: &LABEL_SHADOW };
const CHIP: Style = Style { px: 22, weight: "500", spacing_em: 0.0, feature: "", ink: INK_1, shadows: &[] };
const CHIP_ON: Style = Style { weight: "600", ..CHIP };
const CHIP_FOCUS: Style = Style { weight: "600", ink: FOCUS_INK, ..CHIP };
const WP_TITLE: Style = Style { px: 22, weight: "400", spacing_em: 0.0, feature: "", ink: INK_2, shadows: &[] };
const WP_TITLE_FOCUS: Style = Style { ink: INK_1, ..WP_TITLE };
const WP_PLAIN: Style = Style { px: 26, weight: "400", spacing_em: 0.0, feature: "", ink: INK_2, shadows: &[] };
const KEY_TEXT: Style = Style { px: 28, weight: "500", spacing_em: 0.0, feature: "", ink: WHITE, shadows: &[] };
const KEY_FOCUS: Style = Style { ink: FOCUS_INK, ..KEY_TEXT };

#[derive(Clone, PartialEq, Debug)]
pub enum Outcome {
    Stay,
    Close,
    Setting(String, Value),
    Settings(Vec<(String, Value)>),
    Toast(String, bool),
    Restore,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Tab {
    Apps,
    Sources,
    Channels,
}

pub struct ChannelTile {
    pub id: String,
    pub number: String,
    pub name: String,
    pub logo: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Page {
    Root,
    Group(Group),
    Grid(Tab),
    Wallpaper,
    Choice(&'static str),
    Text(&'static str),
}

struct Entry {
    page: Page,
    focus: usize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Zone {
    Button,
    Content,
    Chips,
}

struct Held {
    index: usize,
    original: Vec<String>,
}

#[derive(Clone)]
enum Look {
    App(Icon, Option<[f32; 3]>, String),
    Input(String),
    Channel(String, Option<String>),
}

#[derive(Clone)]
struct Item {
    key: String,
    label: String,
    look: Look,
}

#[derive(Clone)]
struct WpEntry {
    key: String,
    title: String,
    category: String,
    on: bool,
    thumb: Option<(String, String)>,
    plain: Option<String>,
}

struct Row {
    label: String,
    value: Option<String>,
    chevron: bool,
    ticked: Option<bool>,
}

enum Activate {
    Push(Page, usize),
    Option(&'static str),
}

struct ListPage {
    title: String,
    glyph: &'static str,
    rows: Vec<Row>,
    descriptions: Vec<String>,
    acts: Vec<Activate>,
}

struct Lift {
    index: usize,
    anim: Anim,
}

struct Picture {
    id: String,
    url: String,
    title: String,
    category: String,
}

pub struct SettingsScreen {
    config: Value,
    apps: Vec<App>,
    pub channels: Vec<ChannelTile>,
    stack: Vec<Entry>,
    zone: Zone,
    grid_index: usize,
    reordering: bool,
    held: Option<Held>,
    wp_cat: usize,
    wp_shown: usize,
    wp_changed_at: u32,
    wp_index: usize,
    typed: String,
    caps: bool,
    key_at: (usize, usize),
    live: Shared,
    started: bool,
    app_dir: String,
    pub online: bool,
    dev: bool,
    cities: Vec<City>,
    pictures: Vec<Picture>,
    cached: Option<(u32, u32, Rc<Vec<Opt>>)>,
    generation: u32,
    enter: Anim,
    scroll: Anim,
    lifts: Vec<Lift>,
    grid_scroll: Anim,
    pill_x: Anim,
    pill_w: Anim,
    pill_alpha: Anim,
    button_lift: Anim,
    measures: HashMap<String, f32>,
    wraps: HashMap<String, Vec<String>>,
    tile_colours: HashMap<String, [f32; 3]>,
    thumbs: Thumbs,
    drop_tiles: bool,
    reduce_motion: bool,
}

fn baseline(top: f32, line: f32, px: f32) -> f32 {
    top + (line - 1.2 * px) / 2.0 + 0.95 * px
}

fn rr(px: f32, py: f32, hw: f32, hh: f32, r: f32) -> f32 {
    let (qx, qy) = (px.abs() - hw + r, py.abs() - hh + r);
    (qx.max(0.0).hypot(qy.max(0.0))) + qx.max(qy).min(0.0) - r
}

fn rr_normal(px: f32, py: f32, hw: f32, hh: f32, r: f32) -> (f32, f32) {
    let (qx, qy) = (px.abs() - hw + r, py.abs() - hh + r);
    let (nx, ny) = if qx > 0.0 && qy > 0.0 {
        let l = qx.hypot(qy);
        (qx / l, qy / l)
    } else if qx > qy {
        (1.0, 0.0)
    } else {
        (0.0, 1.0)
    };
    (nx * px.signum(), ny * py.signum())
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

struct ShapeLook {
    fill: [f32; 4],
    rim: f32,
    shadow: Option<(f32, f32, f32)>,
}

fn shape_rgba(w: f32, h: f32, radius: f32, margin: f32, look: &ShapeLook) -> Decoded {
    let (pw, ph) = ((w + 2.0 * margin).ceil() as usize, (h + 2.0 * margin).ceil() as usize);
    let mut rgba = vec![0u8; pw * ph * 4];
    let (hw, hh, r) = (w / 2.0, h / 2.0, radius.min(w / 2.0).min(h / 2.0));
    for y in 0..ph {
        for x in 0..pw {
            let (px, py) = (x as f32 + 0.5 - margin - hw, y as f32 + 0.5 - margin - hh);
            let d = rr(px, py, hw, hh, r);
            let cover = (0.5 - d).clamp(0.0, 1.0);
            let mut out = [0.0f32; 4];
            if let Some((dy, blur, strength)) = look.shadow {
                let ds = rr(px, py - dy, hw, hh, r);
                out[3] = strength * (1.0 - smoothstep(-blur * 0.5, blur * 0.5, ds));
            }
            let fa = look.fill[3] * cover;
            for c in 0..3 {
                out[c] = look.fill[c] * fa + out[c] * (1.0 - fa);
            }
            out[3] = fa + out[3] * (1.0 - fa);
            if look.rim > 0.0 && cover > 0.0 {
                let (nx, ny) = rr_normal(px, py, hw, hh, r);
                let edge = 1.0 - smoothstep(0.0, 2.0, -d);
                let ra = (look.rim * cover * edge * (0.45 * (-ny).max(0.0) + 0.2 * (-nx).max(0.0))).clamp(0.0, 1.0);
                for c in 0..3 {
                    out[c] = ra + out[c] * (1.0 - ra);
                }
                out[3] = ra + out[3] * (1.0 - ra);
            }
            let at = (y * pw + x) * 4;
            for c in 0..4 {
                rgba[at + c] = (out[c].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            }
        }
    }
    Decoded { rgba, w: pw as i32, h: ph as i32 }
}

fn rounded_corners(picture: &mut Decoded, radius: f32) {
    let (w, h) = (picture.w as f32, picture.h as f32);
    for y in 0..picture.h as usize {
        for x in 0..picture.w as usize {
            let cover = (0.5 - rr(x as f32 + 0.5 - w / 2.0, y as f32 + 0.5 - h / 2.0, w / 2.0, h / 2.0, radius)).clamp(0.0, 1.0);
            if cover < 1.0 {
                let at = (y * picture.w as usize + x) * 4;
                for c in 0..4 {
                    picture.rgba[at + c] = (picture.rgba[at + c] as f32 * cover) as u8;
                }
            }
        }
    }
}

fn greyed(picture: &Decoded) -> Decoded {
    let mut rgba = picture.rgba.clone();
    for px in rgba.chunks_exact_mut(4) {
        let grey = (0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32) as u8;
        px[0] = grey;
        px[1] = grey;
        px[2] = grey;
    }
    Decoded { rgba, w: picture.w, h: picture.h }
}

fn grey_of(colour: [f32; 3]) -> [f32; 3] {
    let g = 0.299 * colour[0] + 0.587 * colour[1] + 0.114 * colour[2];
    [g, g, g]
}

fn padded_glyph(icon: &Decoded, size: usize, share: f32, brighten: f32) -> Decoded {
    let mut rgba = vec![0u8; size * size * 4];
    let inner = (size as f32 * share) as usize;
    let scale = inner as f32 / icon.w.max(icon.h) as f32;
    let (dw, dh) = ((icon.w as f32 * scale) as usize, (icon.h as f32 * scale) as usize);
    let (left, top) = ((size - dw) / 2, (size - dh) / 2);
    for y in 0..dh {
        for x in 0..dw {
            let (sx, sy) = (((x as f32 + 0.5) / scale) as usize, ((y as f32 + 0.5) / scale) as usize);
            let from = (sy.min(icon.h as usize - 1) * icon.w as usize + sx.min(icon.w as usize - 1)) * 4;
            let to = ((top + y) * size + left + x) * 4;
            let a = icon.rgba[from + 3] as f32 / 255.0;
            for c in 0..3 {
                rgba[to + c] = ((icon.rgba[from + c] as f32 * brighten).min(255.0) * a) as u8;
            }
            rgba[to + 3] = icon.rgba[from + 3];
        }
    }
    Decoded { rgba, w: size as i32, h: size as i32 }
}

fn clip(d: &Draw, x: f32, y: f32, w: f32, h: f32) {
    let (sx, sy) = (d.gfx.screen_w as f32 / DESIGN_W, d.gfx.screen_h as f32 / DESIGN_H);
    unsafe {
        glEnable(GL_SCISSOR_TEST);
        glScissor((x * sx) as i32, ((DESIGN_H - y - h) * sy) as i32, (w * sx).ceil() as i32, (h * sy).ceil() as i32);
    }
}

fn unclip() {
    unsafe { glDisable(GL_SCISSOR_TEST) }
}

fn row_window(count: usize, focus: usize, in_view: usize, focus_at: usize) -> usize {
    focus.saturating_sub(focus_at).min(count.saturating_sub(in_view))
}

fn grid_offset(count: usize, columns: usize, selected: usize, step: f32, gap: f32, viewport: f32, end_room: f32) -> f32 {
    let total = count.div_ceil(columns) as f32 * step - gap;
    let row = (selected / columns.max(1)) as f32;
    if total <= viewport {
        return 0.0;
    }
    (row * step - (viewport - step) / 2.0).clamp(0.0, total - viewport + end_room)
}

fn faded_share(top: f32, bottom: f32, view_bottom: f32) -> f32 {
    let mask = |y: f32| ((view_bottom - y) / FADE).clamp(0.0, 1.0);
    (0..8).map(|i| mask(top + (bottom - top) * (i as f32 + 0.5) / 8.0)).sum::<f32>() / 8.0
}

impl SettingsScreen {
    pub fn new(config: Value, apps: Vec<App>, now: u32) -> SettingsScreen {
        let reduce_motion = config["reduceMotion"].as_bool() == Some(true);
        let mut enter = Anim::still(0.0);
        enter.go(1.0, if reduce_motion { 1.0 } else { 220.0 }, &EASE_OUT, now);
        SettingsScreen {
            config: if config.is_object() { config } else { json!({}) },
            apps,
            channels: Vec::new(),
            stack: vec![Entry { page: Page::Root, focus: 0 }],
            zone: Zone::Content,
            grid_index: 0,
            reordering: false,
            held: None,
            wp_cat: 0,
            wp_shown: 0,
            wp_changed_at: now,
            wp_index: 0,
            typed: String::new(),
            caps: false,
            key_at: (1, 0),
            live: Shared::default(),
            started: false,
            app_dir: String::new(),
            online: true,
            dev: cfg!(feature = "dev"),
            cities: Vec::new(),
            pictures: Vec::new(),
            cached: None,
            generation: 0,
            enter,
            scroll: Anim::still(0.0),
            lifts: vec![Lift { index: 0, anim: Anim::still(1.0) }],
            grid_scroll: Anim::still(0.0),
            pill_x: Anim::still(-1.0),
            pill_w: Anim::still(0.0),
            pill_alpha: Anim::still(0.22),
            button_lift: Anim::still(0.0),
            measures: HashMap::new(),
            wraps: HashMap::new(),
            tile_colours: HashMap::new(),
            thumbs: Thumbs::default(),
            drop_tiles: false,
            reduce_motion,
        }
    }

    fn start(&mut self, d: &Draw) {
        if self.started {
            return;
        }
        self.started = true;
        self.app_dir = d.app_dir.clone();
        self.cities = cities_in(&std::fs::read_to_string(format!("{}/assets/cities.tsv", d.app_dir)).unwrap_or_default());
        self.pictures = std::fs::read_to_string(format!("{}/assets/wallpapers.tsv", d.app_dir))
            .unwrap_or_default()
            .lines()
            .filter_map(|line| {
                let f: Vec<&str> = line.split('\t').collect();
                (f.len() >= 5 && WALLPAPER_CATEGORIES.contains(&f[4])).then(|| Picture { id: f[0].into(), url: f[1].into(), title: f[2].into(), category: f[4].into() })
            })
            .collect();
        let tv_dir = if cfg!(feature = "webos") { NATIVE_DIR.to_string() } else { d.app_dir.clone() };
        tv::read_tv(&self.live, &tv_dir);
        tv::read_side(&self.live, self.online);
        tv::read_inputs(&self.live);
        tv::read_immich(&self.live);
    }

    fn tv_dir(&self) -> String {
        if cfg!(feature = "webos") {
            NATIVE_DIR.to_string()
        } else {
            self.app_dir.clone()
        }
    }

    fn ms(&self, ms: f32) -> f32 {
        if self.reduce_motion {
            1.0
        } else {
            ms
        }
    }

    fn options(&mut self) -> Rc<Vec<Opt>> {
        let changed = self.live.lock().map_or(0, |l| l.changed);
        if let Some((generation, seen, all)) = &self.cached {
            if *generation == self.generation && *seen == changed {
                return all.clone();
            }
        }
        let all = {
            let Ok(live) = self.live.lock() else { return Rc::new(Vec::new()) };
            let facts = Facts { tv: live.tv.as_ref(), tv_failed: live.tv_failed, busy: &live.busy, side: live.side.as_ref(), immich: &live.immich, cities: &self.cities, dev: self.dev };
            Rc::new(opts::options(&self.config, &facts))
        };
        self.cached = Some((self.generation, changed, all.clone()));
        all
    }

    fn apply(&mut self, changes: &[(String, Value)]) {
        if let Some(config) = self.config.as_object_mut() {
            for (key, value) in changes {
                config.insert(key.clone(), value.clone());
            }
        }
        self.generation = self.generation.wrapping_add(1);
        self.reduce_motion = self.config["reduceMotion"].as_bool() == Some(true);
        if changes.iter().any(|(key, _)| SCHEDULE_KEYS.contains(&key.as_str())) {
            root::write_shared_file("backlight-schedule", &opts::brightness_schedule(&self.config), |written| {
                if let Err(why) = written {
                    crate::note!("brightness schedule: {}", why);
                }
            });
        }
    }

    fn changed(&mut self, changes: Vec<(String, Value)>) -> Outcome {
        if changes.is_empty() {
            return Outcome::Stay;
        }
        self.apply(&changes);
        if changes.len() == 1 {
            let (key, value) = changes.into_iter().next().expect("one change");
            Outcome::Setting(key, value)
        } else {
            Outcome::Settings(changes)
        }
    }

    pub fn set_config(&mut self, config: Value) {
        if config.is_object() {
            self.config = config;
            self.generation = self.generation.wrapping_add(1);
        }
    }

    fn top(&self) -> &Entry {
        self.stack.last().expect("the top page")
    }

    fn page(&self) -> Page {
        self.top().page
    }

    fn focus(&self) -> usize {
        self.top().focus
    }

    fn strings(&self, key: &str) -> Option<Vec<String>> {
        Some(self.config[key].as_array()?.iter().filter_map(|v| v.as_str().map(String::from)).collect())
    }

    fn inputs(&self) -> Vec<tv::Input> {
        self.live.lock().ok().and_then(|l| l.inputs.clone()).unwrap_or_default()
    }

    fn tab_items(&self, tab: Tab) -> (Vec<Item>, Vec<Item>) {
        let items: Vec<Item> = match tab {
            Tab::Apps => self.apps.iter().map(|a| Item { key: a.id.clone(), label: a.title.clone(), look: Look::App(a.icon.clone(), a.color, a.title.clone()) }).collect(),
            Tab::Sources => self.inputs().into_iter().map(|i| Item { key: i.app_id, label: i.label, look: Look::Input(i.icon) }).collect(),
            Tab::Channels => self.channels.iter().map(|c| Item { key: c.id.clone(), label: format!("{} · {}", c.number, c.name), look: Look::Channel(c.name.clone(), c.logo.clone()) }).collect(),
        };
        let ids = match tab {
            Tab::Apps => self.strings("appIds").unwrap_or_default(),
            Tab::Sources => self.strings("sourceIds").unwrap_or_else(|| items.iter().map(|i| i.key.clone()).collect()),
            Tab::Channels => self.strings("liveChannels").unwrap_or_else(|| items.iter().take(DEFAULT_ROW_CHANNELS).map(|i| i.key.clone()).collect()),
        };
        let shown = ids.iter().filter_map(|id| items.iter().find(|i| &i.key == id).cloned()).collect();
        (items, shown)
    }

    fn grid_list(&self, tab: Tab) -> (Vec<Item>, Vec<Item>) {
        let (items, shown) = self.tab_items(tab);
        if self.reordering {
            (shown.clone(), shown)
        } else {
            (items, shown)
        }
    }

    fn wallpaper_tab(&self) -> (Vec<String>, Vec<WpEntry>, String) {
        let kind = opts::text(&self.config, "wallpaperType", "pictures").to_string();
        let mut entries = Vec::new();
        if kind == "immich" {
            let live = self.live.lock().map(|l| l.immich.clone()).unwrap_or_default();
            let picks = self.strings("immichPicks").unwrap_or_else(|| vec!["immich:recent".into()]);
            if live.ready {
                for (key, title) in IMMICH_SOURCES.iter().map(|(k, t)| (k.to_string(), t.to_string())).chain(live.albums.iter().map(|(id, t)| (format!("immich:album:{id}"), t.clone()))) {
                    entries.push(WpEntry { on: picks.contains(&key), key, title, category: "Immich".into(), thumb: None, plain: None });
                }
            } else {
                entries.push(WpEntry { key: "immich:setup".into(), title: "Set up Immich".into(), category: "Immich".into(), on: false, thumb: None, plain: Some("Set up".into()) });
            }
            return (vec!["Immich".into()], entries, "Pick which of your Immich photos the home screen shows.".into());
        }
        if kind == "plain" {
            return (Vec::new(), entries, String::new());
        }
        let picked = self.strings("wallpaperPicks").unwrap_or_default();
        for category in WALLPAPER_CATEGORIES {
            let keys: Vec<String> = self.pictures.iter().filter(|p| p.category == category).map(|p| format!("cloud:{}", p.id)).collect();
            let all = !keys.is_empty() && keys.iter().all(|k| picked.contains(k));
            entries.push(WpEntry { key: format!("all:{category}"), title: (if all { "Deselect all" } else { "Select all" }).into(), category: category.into(), on: all, thumb: None, plain: Some("All".into()) });
            for p in self.pictures.iter().filter(|p| p.category == category) {
                let key = format!("cloud:{}", p.id);
                entries.push(WpEntry { on: picked.contains(&key), key, title: p.title.clone(), category: category.into(), thumb: Some((p.id.clone(), p.url.clone())), plain: None });
            }
        }
        let summary = match picked.len() {
            0 => "Nothing picked: the default picture shows.".to_string(),
            1 => "1 picture picked.".to_string(),
            n => format!("{n} pictures picked."),
        };
        (WALLPAPER_CATEGORIES.iter().map(|c| c.to_string()).collect(), entries, summary)
    }

    fn wp_list(&self, category: usize) -> Vec<WpEntry> {
        let (categories, entries, _) = self.wallpaper_tab();
        let Some(name) = categories.get(category) else { return Vec::new() };
        entries.into_iter().filter(|e| &e.category == name).collect()
    }

    fn pick_wallpaper(&mut self, key: &str) -> Outcome {
        if key == "immich:setup" {
            return Outcome::Toast("Set up Immich first, in Settings > Immich".into(), false);
        }
        if key.starts_with("immich:") {
            let picks = self.strings("immichPicks").unwrap_or_else(|| vec!["immich:recent".into()]);
            if picks.len() == 1 && picks[0] == key {
                return Outcome::Toast("Keep at least one source".into(), false);
            }
            return self.changed(vec![("immichPicks".into(), json!(opts::toggled(&picks, key)))]);
        }
        let picked = self.strings("wallpaperPicks").unwrap_or_default();
        if let Some(category) = key.strip_prefix("all:") {
            let keys: Vec<String> = self.pictures.iter().filter(|p| p.category == category).map(|p| format!("cloud:{}", p.id)).collect();
            let all = keys.iter().all(|k| picked.contains(k));
            let next: Vec<String> = if all { picked.into_iter().filter(|k| !keys.contains(k)).collect() } else { picked.iter().cloned().chain(keys.into_iter().filter(|k| !picked.contains(k))).collect() };
            return self.changed(vec![("wallpaperPicks".into(), json!(next))]);
        }
        self.changed(vec![("wallpaperPicks".into(), json!(opts::toggled(&picked, key)))])
    }

    fn list_page(&mut self) -> Option<ListPage> {
        let all = self.options();
        match self.page() {
            Page::Root => {
                let mut tabs = vec![(Tab::Apps, "Apps", opts::GRID), (Tab::Sources, "Sources", opts::INPUTS)];
                if self.config["liveTvRow"].as_bool() != Some(false) && !self.channels.is_empty() {
                    tabs.push((Tab::Channels, "Channels", opts::GUIDE));
                }
                let groups: Vec<_> = GROUPS.iter().filter(|g| g.0 == Group::Wallpaper || all.iter().any(|o| o.group == g.0)).collect();
                let focus = self.focus();
                let glyph = if focus < tabs.len() { tabs[focus].2 } else { groups.get(focus - tabs.len()).map_or(opts::GEAR, |g| g.2) };
                let mut rows = Vec::new();
                let mut acts = Vec::new();
                for (tab, label, _) in tabs {
                    rows.push(Row { label: label.into(), value: None, chevron: true, ticked: None });
                    acts.push(Activate::Push(Page::Grid(tab), 0));
                }
                for g in groups {
                    rows.push(Row { label: g.1.into(), value: None, chevron: true, ticked: None });
                    acts.push(Activate::Push(Page::Group(g.0), 0));
                }
                Some(ListPage { title: "Settings".into(), glyph, descriptions: vec![String::new(); rows.len()], rows, acts })
            }
            Page::Group(group) => {
                let (title, glyph) = group_named(group);
                let list: Vec<&Opt> = all.iter().filter(|o| o.group == group).collect();
                let mut rows: Vec<Row> = list.iter().map(|o| Row { label: o.label.clone(), value: Some(o.value.clone()), chevron: o.opens_a_page(), ticked: None }).collect();
                let mut descriptions: Vec<String> = list.iter().map(|o| o.description.clone()).collect();
                let mut acts: Vec<Activate> = list.iter().map(|o| Activate::Option(o.id)).collect();
                let (categories, _, summary) = self.wallpaper_tab();
                if group == Group::Wallpaper && !categories.is_empty() {
                    let at = list.iter().position(|o| o.id == "wp-type").map_or(0, |i| i + 1);
                    rows.insert(at, Row { label: "Choose wallpaper".into(), value: None, chevron: true, ticked: None });
                    descriptions.insert(at, summary);
                    acts.insert(at, Activate::Push(Page::Wallpaper, 0));
                }
                Some(ListPage { title: title.into(), glyph, rows, descriptions, acts })
            }
            Page::Choice(id) => {
                let o = all.iter().find(|o| o.id == id)?;
                let Kind::List { labels, at, picks, .. } = &o.kind else { return None };
                let rows = labels.iter().enumerate().map(|(i, l)| Row { label: l.clone(), value: None, chevron: false, ticked: Some(picks.as_ref().map_or(i == *at, |p| p.contains(&i))) }).collect();
                Some(ListPage { title: o.label.clone(), glyph: group_named(o.group).1, descriptions: vec![o.description.clone(); labels.len()], rows, acts: Vec::new() })
            }
            _ => None,
        }
    }

    fn push(&mut self, page: Page, focus: usize, now: u32) {
        self.stack.push(Entry { page, focus });
        self.entered(now);
        match page {
            Page::Grid(_) => {
                self.grid_index = 0;
                self.zone = Zone::Content;
                self.grid_scroll.set(0.0);
            }
            Page::Wallpaper => {
                self.wp_index = 0;
                self.zone = Zone::Chips;
                self.grid_scroll.set(0.0);
                self.pill_x.set(-1.0);
            }
            _ => {}
        }
        self.settle_scroll(now, true);
    }

    fn pop(&mut self, now: u32) -> Outcome {
        if self.stack.len() == 1 {
            return Outcome::Close;
        }
        if matches!(self.page(), Page::Grid(_)) {
            self.drop_tiles = true;
        }
        self.stack.pop();
        self.reordering = false;
        self.held = None;
        self.entered(now);
        self.settle_scroll(now, true);
        Outcome::Stay
    }

    fn entered(&mut self, now: u32) {
        self.enter = Anim::still(0.0);
        self.enter.go(1.0, self.ms(220.0), &EASE_OUT, now);
        self.lifts.clear();
    }

    fn set_focus(&mut self, focus: usize, now: u32) {
        if let Some(top) = self.stack.last_mut() {
            top.focus = focus;
        }
        self.settle_scroll(now, false);
    }

    fn settle_scroll(&mut self, now: u32, at_once: bool) {
        let Some(count) = self.list_page().map(|l| l.rows.len()) else { return };
        let in_view = ((VIEW_H - SHADOW_ROOM) / ROW_STEP) as usize;
        let top = row_window(count, self.focus(), in_view, (in_view - 1) / 2) as f32 * ROW_STEP;
        if at_once {
            self.scroll.set(top);
        } else {
            self.scroll.go(top, self.ms(260.0), &EASE_SCROLL, now);
        }
    }

    fn lift_to(&mut self, index: usize, now: u32) {
        if self.lifts.is_empty() {
            self.lifts.push(Lift { index, anim: Anim::still(1.0) });
            return;
        }
        let ms = self.ms(160.0);
        for lift in &mut self.lifts {
            if lift.index != index {
                lift.anim.go(0.0, ms, &EASE_OUT, now);
            }
        }
        if !self.lifts.iter().any(|l| l.index == index) {
            let mut anim = Anim::still(0.0);
            anim.go(1.0, ms, &EASE_OUT, now);
            self.lifts.push(Lift { index, anim });
        } else if let Some(l) = self.lifts.iter_mut().find(|l| l.index == index) {
            l.anim.go(1.0, ms, &EASE_OUT, now);
        }
        self.lifts.retain(|l| l.index == index || l.anim.busy(now) || l.anim.to_value() > 0.0);
    }

    fn focus_code(&self) -> usize {
        match self.page() {
            Page::Grid(_) if self.zone == Zone::Content => self.grid_index,
            Page::Wallpaper if self.zone == Zone::Content => self.wp_index,
            Page::Grid(_) | Page::Wallpaper => usize::MAX,
            Page::Text(_) => self.key_at.0 * 100 + self.key_at.1,
            _ => self.focus(),
        }
    }

    fn lift_of(&self, index: usize, now: u32) -> f32 {
        self.lifts.iter().find(|l| l.index == index).map_or(0.0, |l| l.anim.at(now))
    }

    fn activate_option(&mut self, id: &'static str, now: u32) -> Outcome {
        let all = self.options();
        let Some(o) = all.iter().find(|o| o.id == id).cloned() else { return Outcome::Stay };
        match &o.kind {
            Kind::Text { value, .. } => {
                self.typed = value.clone();
                self.key_at = (1, 0);
                self.caps = false;
                self.push(Page::Text(id), 0, now);
                Outcome::Stay
            }
            Kind::List { at, .. } if o.flips() => {
                let next = if *at == 1 { 0 } else { 1 };
                self.choose(&o, next)
            }
            Kind::List { at, labels, .. } if !labels.is_empty() => {
                self.push(Page::Choice(id), *at, now);
                Outcome::Stay
            }
            Kind::Switch => {
                let changes = opts::switched(id, &self.config).unwrap_or_default();
                self.changed(changes)
            }
            _ => self.action(id),
        }
    }

    fn choose(&mut self, o: &Opt, index: usize) -> Outcome {
        match opts::chosen(o, index, &self.config, &self.cities) {
            Ok(changes) => self.changed(changes),
            Err(why) => Outcome::Toast(why, false),
        }
    }

    fn action(&mut self, id: &'static str) -> Outcome {
        let (state, side) = match self.live.lock() {
            Ok(l) => (l.tv.clone(), l.side.clone()),
            Err(_) => return Outcome::Stay,
        };
        let dir = self.tv_dir();
        let live = self.live.clone();
        match id {
            "immich-test" => {
                tv::test_immich(&live);
                Outcome::Stay
            }
            "restore-backup" => Outcome::Restore,
            "tv-scripts" => match state {
                None => Outcome::Toast((if cfg!(feature = "webos") { "Still checking the TV" } else { "Only on the TV" }).into(), false),
                Some(s) if s.scripts == "unshipped" => Outcome::Toast("This build has no TV scripts in it".into(), true),
                Some(_) => {
                    let at = dir.clone();
                    tv::run_job(&live, id, &dir, move |done| root::install_tv_scripts(&at, move |ok| done(ok.map(|ok| (if ok { "TV scripts installed" } else { "Could not install the TV scripts" }).to_string()))));
                    Outcome::Stay
                }
            },
            "side-guide" | "side-quick" | "side-saver" | "side-update" => {
                let Some(side) = side else { return Outcome::Toast("Still checking the side apps".into(), false) };
                let online = self.online;
                let after = live.clone();
                let reread = move |line: Result<String, String>| {
                    tv::read_side(&after, online);
                    line
                };
                if id == "side-update" {
                    let behind = side.behind();
                    if behind.is_empty() {
                        return Outcome::Toast("The side apps are up to date".into(), false);
                    }
                    tv::run_job(&live, id, &dir, move |done| install_each(side, behind, Vec::new(), Box::new(move |line| done(reread(line)))));
                    return Outcome::Stay;
                }
                let app = root::SIDE_APPS[opts::SIDE_OPTIONS.iter().position(|o| *o == id).unwrap_or(0)];
                let title = tv::side_title(app);
                if side.installed(app).is_some() {
                    tv::run_job(&live, id, &dir, move |done| root::uninstall_side_app(app, move |gone| done(reread(gone.map(|gone| if gone { format!("{title} removed") } else { format!("Could not remove {title}") })))));
                } else {
                    let Some(next) = side.latest(app).cloned() else {
                        return Outcome::Toast(if side.offline { "Could not reach GitHub for the side apps".into() } else { format!("{title} is not in a release yet") }, false);
                    };
                    tv::run_job(&live, id, &dir, move |done| root::install_side_app(app, &next.url, &next.sha256, move |out| done(reread(out.map(|r| if r == "installed" { format!("{title} {} installed", next.version) } else { format!("{title}: {r}") })))));
                }
                Outcome::Stay
            }
            _ => self.tv_action(id, state, &live, &dir),
        }
    }

    fn tv_action(&mut self, id: &'static str, state: Option<tv::TvState>, live: &Shared, dir: &str) -> Outcome {
        let Some(s) = state else { return Outcome::Toast((if cfg!(feature = "webos") { "Still checking the TV" } else { "Only on the TV" }).into(), false) };
        if !s.scripts_ready() {
            return Outcome::Toast("Install the TV scripts first, in Settings > System > TV scripts.".into(), false);
        }
        let lumen = |app: &str| ["com.samrenfrew.lumen", "com.samrenfrew.lumen.dev", "com.samrenfrew.lumen.native"].contains(&app);
        let action = match id {
            "tv-restart" => {
                tv::run_job(live, id, dir, |done| root::reboot(move |r| done(r.map(|_| "Restarting the TV".to_string()))));
                return Outcome::Stay;
            }
            "tv-home" if lumen(&s.home) => "nativeOff",
            "tv-power-on" if lumen(&s.power_on) => "powerOnOff",
            // ponytail: home-setup.sh points Home and power-on only at the web Lumen's ids, so Lumen Native can turn them off but not on
            "tv-home" | "tv-power-on" => return Outcome::Toast("Lumen Native cannot be the Home or power-on app yet".into(), true),
            "privacy-agreements" if s.agreements == Some(0) && !s.backup => return Outcome::Toast("Already withdrawn. LG's own menus can give them again.".into(), false),
            "privacy-agreements" if s.agreements == Some(0) => "agreementsRestore",
            "privacy-agreements" => "agreementsOff",
            "privacy-lmt" | "privacy-liveplus" if (if id == "privacy-lmt" { &s.lmt } else { &s.live_plus }).is_empty() => return Outcome::Toast("This TV has no such setting".into(), false),
            "privacy-lmt" if s.lmt == "on" => "lmtOff",
            "privacy-lmt" => "lmtOn",
            "privacy-liveplus" if s.live_plus == "on" => "livePlusOff",
            "privacy-liveplus" => "livePlusOn",
            "privacy-hosts" if s.hosts => "hostsOff",
            "privacy-hosts" => "hostsOn",
            "privacy-services" if s.services => "servicesOff",
            "privacy-services" => "servicesOn",
            "privacy-updates" if s.updates => "updatesOff",
            "privacy-updates" => "updatesOn",
            _ => return Outcome::Stay,
        };
        if id == "tv-home" || id == "tv-power-on" {
            tv::run_job(live, id, dir, move |done| root::home_setup(action, done));
        } else {
            tv::run_job(live, id, dir, move |done| root::privacy(action, done));
        }
        Outcome::Stay
    }

    fn set_mode(&mut self, tab: Tab, next: bool) {
        if next == self.reordering {
            return;
        }
        let (items, _) = self.grid_list(tab);
        let focused = items.get(self.grid_index).map(|i| i.key.clone());
        self.reordering = next;
        self.held = None;
        let (target, _) = self.grid_list(tab);
        self.grid_index = focused.and_then(|k| target.iter().position(|i| i.key == k)).unwrap_or(0);
        if next && target.is_empty() {
            self.zone = Zone::Button;
        }
        self.lifts.clear();
    }

    fn order_key(tab: Tab) -> &'static str {
        match tab {
            Tab::Apps => "appIds",
            Tab::Sources => "sourceIds",
            Tab::Channels => "liveChannels",
        }
    }

    fn move_held(&mut self, tab: Tab, to: isize) -> Outcome {
        let Some(held) = &self.held else { return Outcome::Stay };
        let (_, shown) = self.tab_items(tab);
        if shown.is_empty() {
            return Outcome::Stay;
        }
        let dest = to.clamp(0, shown.len() as isize - 1) as usize;
        if dest == held.index {
            return Outcome::Stay;
        }
        let keys: Vec<String> = shown.iter().map(|i| i.key.clone()).collect();
        let from = held.index;
        if let Some(h) = self.held.as_mut() {
            h.index = dest;
        }
        self.grid_index = dest;
        self.changed(vec![(Self::order_key(tab).into(), json!(opts::moved(&keys, from, dest)))])
    }

    fn activate_grid(&mut self, tab: Tab) -> Outcome {
        let (items, shown) = self.grid_list(tab);
        let Some(item) = items.get(self.grid_index) else { return Outcome::Stay };
        if !self.reordering {
            let ids: Vec<String> = match tab {
                Tab::Apps => self.strings("appIds").unwrap_or_default(),
                Tab::Sources => self.strings("sourceIds").unwrap_or_else(|| self.inputs().into_iter().map(|i| i.app_id).collect()),
                Tab::Channels => shown.iter().map(|i| i.key.clone()).collect(),
            };
            return self.changed(vec![(Self::order_key(tab).into(), json!(opts::toggled(&ids, &item.key)))]);
        }
        if self.held.take().is_some() {
            return Outcome::Stay;
        }
        self.held = Some(Held { index: self.grid_index, original: shown.iter().map(|i| i.key.clone()).collect() });
        Outcome::Stay
    }

    pub fn can_hold(&self) -> bool {
        matches!(self.page(), Page::Grid(_))
    }

    pub fn hold(&mut self, now: u32) {
        if let Page::Grid(tab) = self.page() {
            let next = !self.reordering;
            self.set_mode(tab, next);
            self.settle_grid(now, true);
        }
    }

    fn settle_grid(&mut self, now: u32, at_once: bool) {
        let target = match self.page() {
            Page::Grid(tab) => grid_offset(self.grid_list(tab).0.len(), GRID_COLUMNS, self.grid_index, GRID_STEP, GRID_GAP, GRID_VIEW_H, GRID_END_ROOM),
            Page::Wallpaper => grid_offset(self.wp_list(self.wp_shown).len(), WP_COLUMNS, self.wp_index, WP_ROW_STEP, 0.0, WP_VIEW_H, 0.0),
            _ => return,
        };
        if at_once {
            self.grid_scroll.set(target);
        } else {
            self.grid_scroll.go(target, self.ms(260.0), &EASE_SCROLL, now);
        }
    }

    pub fn key(&mut self, d: &Draw, key: Key, now: u32) -> Outcome {
        self.start(d);
        let out = self.react(key, now);
        if matches!(self.page(), Page::Grid(_) | Page::Wallpaper) {
            self.settle_grid(now, false);
        }
        self.lift_to(self.focus_code(), now);
        out
    }

    fn react(&mut self, key: Key, now: u32) -> Outcome {
        let page = self.page();
        if key == Key::Back {
            if let (Some(held), Page::Grid(tab)) = (self.held.take(), page) {
                self.grid_index = held.index;
                return self.changed(vec![(Self::order_key(tab).into(), json!(held.original))]);
            }
            if let (true, Page::Grid(tab)) = (self.reordering, page) {
                self.set_mode(tab, false);
                return Outcome::Stay;
            }
            return self.pop(now);
        }
        match page {
            Page::Text(id) => self.text_key(id, key, now),
            Page::Grid(tab) => self.grid_key(tab, key, now),
            Page::Wallpaper => self.wallpaper_key(key, now),
            _ => self.list_key(key, now),
        }
    }

    fn list_key(&mut self, key: Key, now: u32) -> Outcome {
        let Some(list) = self.list_page() else { return Outcome::Stay };
        let (focus, last) = (self.focus(), list.rows.len().saturating_sub(1));
        match key {
            Key::Up if focus > 0 => self.set_focus(focus - 1, now),
            Key::Down if focus < last => self.set_focus(focus + 1, now),
            Key::Ok => {
                if let Page::Choice(id) = self.page() {
                    let all = self.options();
                    let Some(o) = all.iter().find(|o| o.id == id).cloned() else { return Outcome::Stay };
                    let picks = matches!(o.kind, Kind::List { picks: Some(_), .. });
                    let out = self.choose(&o, focus);
                    if !picks && !matches!(out, Outcome::Toast(..)) {
                        self.pop(now);
                    }
                    return out;
                }
                return match list.acts.into_iter().nth(focus) {
                    Some(Activate::Push(page, at)) => {
                        self.push(page, at, now);
                        Outcome::Stay
                    }
                    Some(Activate::Option(id)) => self.activate_option(id, now),
                    None => Outcome::Stay,
                };
            }
            _ => {}
        }
        Outcome::Stay
    }

    fn grid_key(&mut self, tab: Tab, key: Key, now: u32) -> Outcome {
        let count = self.grid_list(tab).0.len();
        let cols = GRID_COLUMNS;
        if key == Key::Ok {
            if self.zone == Zone::Button {
                self.set_mode(tab, !self.reordering);
                self.button_lift.go(1.0, self.ms(160.0), &EASE_OUT, now);
                return Outcome::Stay;
            }
            return self.activate_grid(tab);
        }
        if let Some(held) = self.held.as_ref().map(|h| h.index as isize) {
            let to = match key {
                Key::Left => held - 1,
                Key::Right => held + 1,
                Key::Up => held - cols as isize,
                Key::Down => held + cols as isize,
                _ => held,
            };
            return self.move_held(tab, to);
        }
        if self.zone == Zone::Button {
            if key == Key::Down && count > 0 {
                self.zone = Zone::Content;
                self.button_lift.go(0.0, self.ms(160.0), &EASE_OUT, now);
            }
            return Outcome::Stay;
        }
        let i = self.grid_index;
        match key {
            Key::Left if i > 0 => self.grid_index = i - 1,
            Key::Right if i + 1 < count => self.grid_index = i + 1,
            Key::Up if i >= cols => self.grid_index = i - cols,
            Key::Up => {
                self.zone = Zone::Button;
                self.button_lift.go(1.0, self.ms(160.0), &EASE_OUT, now);
            }
            Key::Down if i + cols < count => self.grid_index = i + cols,
            _ => {}
        }
        Outcome::Stay
    }

    fn wallpaper_key(&mut self, key: Key, now: u32) -> Outcome {
        let categories = self.wallpaper_tab().0.len();
        if self.zone == Zone::Chips {
            match key {
                Key::Left | Key::Right => {
                    let next = if key == Key::Left { self.wp_cat.saturating_sub(1) } else { (self.wp_cat + 1).min(categories.saturating_sub(1)) };
                    if next != self.wp_cat {
                        self.wp_cat = next;
                        self.wp_index = 0;
                        self.wp_changed_at = now;
                    }
                }
                Key::Down | Key::Ok => {
                    if !self.wp_list(self.wp_cat).is_empty() {
                        self.wp_shown = self.wp_cat;
                        self.zone = Zone::Content;
                        self.pill_alpha.go(0.22, self.ms(160.0), &EASE_OUT, now);
                        self.settle_grid(now, true);
                    }
                }
                _ => {}
            }
            return Outcome::Stay;
        }
        let list = self.wp_list(self.wp_shown);
        let (i, last) = (self.wp_index, list.len().saturating_sub(1));
        match key {
            Key::Ok => {
                if let Some(entry) = list.get(i) {
                    let key = entry.key.clone();
                    return self.pick_wallpaper(&key);
                }
            }
            Key::Left if i > 0 => self.wp_index = i - 1,
            Key::Right if i < last => self.wp_index = i + 1,
            Key::Up if i < WP_COLUMNS => {
                self.zone = Zone::Chips;
                self.pill_alpha.go(1.0, self.ms(160.0), &EASE_OUT, now);
            }
            Key::Up => self.wp_index = i - WP_COLUMNS,
            Key::Down if i + WP_COLUMNS <= last => self.wp_index = i + WP_COLUMNS,
            _ => {}
        }
        Outcome::Stay
    }

    fn key_label(&self, row: usize, col: usize) -> String {
        if row < KEY_ROWS.len() {
            let c = KEY_ROWS[row].chars().nth(col).unwrap_or(' ');
            if self.caps {
                c.to_ascii_uppercase().to_string()
            } else {
                c.to_string()
            }
        } else {
            (if col == 0 && self.caps { "abc" } else { KEY_ACTIONS[col] }).to_string()
        }
    }

    fn text_key(&mut self, id: &'static str, key: Key, now: u32) -> Outcome {
        let (row, col) = self.key_at;
        let width = |row: usize| if row < KEY_ROWS.len() { KEY_ROWS[row].len() } else { KEY_ACTIONS.len() };
        let across = |col: usize, from: usize, to: usize| ((col as f32 + 0.5) * width(to) as f32 / width(from) as f32) as usize;
        match key {
            Key::Left if col > 0 => self.key_at.1 -= 1,
            Key::Right if col + 1 < width(row) => self.key_at.1 += 1,
            Key::Up if row > 0 => self.key_at = (row - 1, across(col, row, row - 1).min(width(row - 1) - 1)),
            Key::Down if row < KEY_ROWS.len() => self.key_at = (row + 1, across(col, row, row + 1).min(width(row + 1) - 1)),
            Key::Digit(n) => self.typed.push_str(&n.to_string()),
            Key::Ok if row < KEY_ROWS.len() => {
                let typed = self.key_label(row, col);
                self.typed.push_str(&typed);
            }
            Key::Ok => match col {
                0 => self.caps = !self.caps,
                1 => {
                    self.typed.pop();
                }
                2 => self.typed.clear(),
                _ => {
                    let value = self.typed.trim().to_string();
                    if let Ok(mut live) = self.live.lock() {
                        if id == "immich-url" {
                            live.immich.url = value;
                        } else {
                            live.immich.key = value;
                        }
                        live.immich.test = tv::Test::Idle;
                        live.changed = live.changed.wrapping_add(1);
                    }
                    return self.pop(now);
                }
            },
            _ => {}
        }
        Outcome::Stay
    }

    pub fn moving(&self, now: u32) -> bool {
        let anims = [&self.enter, &self.scroll, &self.grid_scroll, &self.pill_x, &self.pill_w, &self.pill_alpha, &self.button_lift];
        anims.iter().any(|a| a.busy(now)) || self.lifts.iter().any(|l| l.anim.busy(now)) || (self.page() == Page::Wallpaper && self.wp_shown != self.wp_cat)
    }

    pub fn where_now(&mut self) -> Value {
        let page = match self.page() {
            Page::Root => "settings".to_string(),
            Page::Group(g) => group_named(g).0.to_string(),
            Page::Grid(Tab::Apps) => "apps".into(),
            Page::Grid(Tab::Sources) => "sources".into(),
            Page::Grid(Tab::Channels) => "channels".into(),
            Page::Wallpaper => "choose wallpaper".into(),
            Page::Choice(id) | Page::Text(id) => id.to_string(),
        };
        let focused = match self.page() {
            Page::Grid(_) if self.zone == Zone::Button => json!([{"part": "settings__button--focused", "label": if self.reordering { "Done" } else { "Re-order" }}]),
            Page::Grid(tab) => {
                let label = self.grid_list(tab).0.get(self.grid_index).map(|i| i.label.clone()).unwrap_or_default();
                json!([{"part": "tile--focused", "label": label, "index": self.grid_index, "reordering": self.reordering, "holding": self.held.is_some()}])
            }
            Page::Wallpaper if self.zone == Zone::Chips => {
                let label = self.wallpaper_tab().0.get(self.wp_cat).cloned().unwrap_or_default();
                json!([{"part": "wp__chip--focused", "label": label, "index": self.wp_cat}])
            }
            Page::Wallpaper => {
                let label = self.wp_list(self.wp_cat).get(self.wp_index).map(|e| e.title.clone()).unwrap_or_default();
                json!([{"part": "wp__item--focused", "label": label, "index": self.wp_index}])
            }
            Page::Text(_) => json!([{"part": "key--focused", "label": self.key_label(self.key_at.0, self.key_at.1), "row": self.key_at.0, "index": self.key_at.1, "typed": self.typed.len()}]),
            _ => {
                let focus = self.focus();
                let label = self.list_page().and_then(|l| l.rows.get(focus).map(|r| r.label.clone())).unwrap_or_default();
                json!([{"part": "slist__row--focused", "label": label, "index": focus}])
            }
        };
        json!({"screen": "settings", "page": page, "depth": self.stack.len(), "focused": focused})
    }

    fn measure(&mut self, d: &Draw, text: &str, style: &Style) -> f32 {
        let key = format!("{}:{}:{}", style.px, style.weight, text);
        if let Some(w) = self.measures.get(&key) {
            return *w;
        }
        let w = d.fonts.line(text, style.px, style.weight, style.spacing_em * style.px as f32, style.feature).map_or(0.0, |l| l.advance as f32);
        if self.measures.len() > 4000 {
            self.measures.clear();
        }
        self.measures.insert(key, w);
        w
    }

    fn fit(&mut self, d: &Draw, text: &str, style: &Style, max: f32) -> String {
        if self.measure(d, text, style) <= max {
            return text.to_string();
        }
        let chars: Vec<char> = text.chars().collect();
        let (mut lo, mut hi) = (0usize, chars.len());
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            let candidate: String = chars[..mid].iter().collect::<String>().trim_end().to_string() + "…";
            if self.measure(d, &candidate, style) <= max {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        chars[..lo].iter().collect::<String>().trim_end().to_string() + "…"
    }

    fn wrap(&mut self, d: &Draw, text: &str, style: &Style, width: f32, most: usize) -> Vec<String> {
        let key = format!("{}:{}:{}:{}", style.px, width, most, text);
        if let Some(lines) = self.wraps.get(&key) {
            return lines.clone();
        }
        let mut lines: Vec<String> = Vec::new();
        let mut line = String::new();
        for word in text.split_whitespace() {
            let next = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if !line.is_empty() && self.measure(d, &next, style) > width {
                lines.push(std::mem::replace(&mut line, word.to_string()));
            } else {
                line = next;
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
        if lines.len() > most {
            let rest = lines[most - 1..].join(" ");
            lines.truncate(most - 1);
            lines.push(self.fit(d, &rest, style, width));
        }
        if self.wraps.len() > 400 {
            self.wraps.clear();
        }
        self.wraps.insert(key, lines.clone());
        lines
    }

    fn write(&self, d: &mut Draw, text: &str, style: &Style, x: f32, base: f32, alpha: f32) -> f32 {
        let Some(t) = d.text(text, style) else { return 0.0 };
        t.draw(d, x, base, alpha);
        t.advance
    }

    fn write_scaled(d: &mut Draw, text: &str, style: &Style, x: f32, base: f32, scale: f32, alpha: f32) {
        if let Some(t) = d.text(text, style) {
            t.draw_scaled(d, x, base, scale, alpha);
        }
    }

    fn shape(d: &mut Draw, key: &str, w: f32, h: f32, radius: f32, margin: f32, look: ShapeLook) -> Tex {
        if let Some(t) = d.pictures.find(key) {
            return t;
        }
        let made = shape_rgba(w, h, radius, margin, &look);
        let t = d.gfx.upload_rgba(&made.rgba, made.w, made.h);
        d.keep(key, t);
        t
    }

    fn row_plate(d: &mut Draw, focused: bool) -> (Tex, f32) {
        if focused {
            (Self::shape(d, "set:row-focus", ROW_W, ROW_H, ROW_H / 2.0, SHAPE_MARGIN, ShapeLook { fill: [1.0, 1.0, 1.0, 1.0], rim: 0.0, shadow: Some(FOCUS_SHADOW) }), SHAPE_MARGIN)
        } else {
            (Self::shape(d, "set:row", ROW_W, ROW_H, ROW_H / 2.0, 0.0, ShapeLook { fill: [1.0, 1.0, 1.0, 0.09], rim: 1.0, shadow: None }), 0.0)
        }
    }

    fn plate(d: &Draw, (tex, margin): (Tex, f32), x: f32, y: f32, w: f32, h: f32, scale: f32, alpha: f32) {
        let (cx, cy) = (x + w / 2.0, y + h / 2.0);
        let (sw, sh) = ((w + 2.0 * margin) * scale, (h + 2.0 * margin) * scale);
        d.picture(tex, cx - sw / 2.0, cy - sh / 2.0, sw, sh, alpha);
    }

    pub fn draw(&mut self, d: &mut Draw, now: u32) {
        self.start(d);
        if self.drop_tiles {
            d.let_go(|key| key.starts_with("stile:") || key.starts_with("sicon:"));
            self.drop_tiles = false;
        }
        for (id, mut picture) in self.thumbs.take_ready() {
            let radius = 22.0 * picture.w as f32 / WP_THUMB_W;
            rounded_corners(&mut picture, radius);
            let t = d.gfx.upload_rgba(&picture.rgba, picture.w, picture.h);
            d.keep(&format!("thumb:{id}"), t);
        }
        if self.page() == Page::Wallpaper && self.wp_shown != self.wp_cat && now.wrapping_sub(self.wp_changed_at) as i32 >= WP_GRID_MS as i32 {
            self.wp_shown = self.wp_cat;
            self.settle_grid(now, true);
        }
        let prepared = match self.page() {
            Page::Grid(tab) => Some(self.prepare_tiles(d, tab)),
            _ => None,
        };
        d.gfx.into_screen();
        unsafe {
            glDisable(GL_BLEND);
        }
        d.copy_whole(d.veil_backdrop, 1.0, 1.0, 0.0, 0.0);
        unsafe {
            glEnable(GL_BLEND);
            glBlendFuncSeparate(GL_ONE, GL_ONE_MINUS_SRC_ALPHA, GL_ZERO, GL_ONE);
        }
        let t = self.enter.at(now);
        let (dx, alpha) = (36.0 * (1.0 - t), t);
        match self.page() {
            Page::Grid(tab) => self.draw_grid(d, tab, prepared.unwrap_or_default(), dx, alpha, now),
            Page::Wallpaper => self.draw_wallpaper(d, dx, alpha, now),
            Page::Text(id) => self.draw_text_page(d, id, dx, alpha, now),
            _ => {
                if let Some(list) = self.list_page() {
                    self.draw_list(d, &list, dx, alpha, now);
                }
            }
        }
    }

    fn draw_side(&mut self, d: &mut Draw, title: &str, glyph: &str, description: &str, dx: f32, alpha: f32) {
        if let Some(t) = d.text(title, &TITLE) {
            t.draw(d, DESIGN_W / 2.0 - t.advance / 2.0 + dx, baseline(56.0, 72.0, 56.0), 0.62 * alpha);
        }
        let lines = if description.is_empty() { Vec::new() } else { self.wrap(d, description, &DESCRIPTION, DESCRIPTION_W, 12) };
        let text_h = (lines.len() as f32 * DESCRIPTION_LINE).max(140.0);
        let top = (DESIGN_H - (CARD + 40.0 + text_h)) / 2.0;
        let left = ROWS_EDGE + (SIDE_W - CARD) / 2.0 + dx;
        let card = Self::shape(d, "set:card", CARD, CARD, 110.0, 0.0, ShapeLook { fill: [1.0, 1.0, 1.0, 0.22], rim: 0.0, shadow: None });
        d.picture(card, left, top, CARD, CARD, alpha);
        let size = (CARD * 0.4) as i32;
        let icon = d.glyph(glyph, size, 1.4);
        d.inked_picture(icon, left + CARD * 0.3, top + CARD * 0.3, size as f32, size as f32, [15.0 / 255.0, 18.0 / 255.0, 28.0 / 255.0, 0.75 * alpha]);
        let centre = ROWS_EDGE + SIDE_W / 2.0 + dx;
        for (i, line) in lines.iter().enumerate() {
            if let Some(t) = d.text(line, &DESCRIPTION) {
                t.draw(d, centre - t.advance / 2.0, baseline(top + CARD + 40.0 + i as f32 * DESCRIPTION_LINE, DESCRIPTION_LINE, 26.0), 0.62 * alpha);
            }
        }
    }

    fn draw_list(&mut self, d: &mut Draw, list: &ListPage, dx: f32, alpha: f32, now: u32) {
        let focus = self.focus();
        let description = list.descriptions.get(focus).cloned().unwrap_or_default();
        self.draw_side(d, &list.title, list.glyph, &description, dx, alpha);
        let n = list.rows.len();
        let in_view = ((VIEW_H - SHADOW_ROOM) / ROW_STEP) as usize;
        let offset = self.scroll.at(now);
        let more = n > in_view && offset < (n - in_view) as f32 * ROW_STEP - 0.5;
        let first = ((offset / ROW_STEP).floor() as usize).saturating_sub(1);
        let last = (((offset + VIEW_H) / ROW_STEP).ceil() as usize + 1).min(n);
        clip(d, 0.0, VIEW_TOP, DESIGN_W, VIEW_H);
        let ticks = list.rows.iter().any(|r| r.ticked.is_some());
        let view_bottom = VIEW_TOP + VIEW_H;
        let order: Vec<usize> = (first..last).filter(|i| *i != focus).chain((first..last).filter(|i| *i == focus)).collect();
        for i in order {
            let row = &list.rows[i];
            let y = VIEW_TOP + TRACK_TOP + i as f32 * ROW_STEP - offset;
            let fade = if more { faded_share(y, y + ROW_H, view_bottom) } else { 1.0 };
            let a = alpha * fade;
            if a <= 0.0 {
                continue;
            }
            let focused = i == focus;
            let scale = 1.0 + (LIFT_PRESS - 1.0) * self.lift_of(i, now);
            let x = ROW_X + dx;
            let plate = Self::row_plate(d, focused);
            Self::plate(d, plate, x, y, ROW_W, ROW_H, scale, a);
            let (cx, cy) = (x + ROW_W / 2.0, y + ROW_H / 2.0);
            let at = |px: f32| cx + (px - cx) * scale;
            let base = cy + (baseline(y, ROW_H, 32.0) - cy) * scale;
            let ink = if focused { FOCUS_INK } else { WHITE };
            let mut left = x + ROW_PAD;
            if ticks {
                if row.ticked == Some(true) {
                    let tick = d.glyph(TICK, 30, 2.6);
                    let (tx, ty) = (at(left), cy + (cy - 15.0 - cy) * scale);
                    d.inked_picture(tick, tx, ty, 30.0 * scale, 30.0 * scale, [ink[0], ink[1], ink[2], a * if focused { 1.0 } else { 0.94 }]);
                }
                left += 48.0;
            }
            let mut right = x + ROW_W - ROW_PAD;
            if row.chevron {
                let chevron = d.glyph(CHEVRON, 24, 3.1);
                let colour = if focused { [FOCUS_INK[0], FOCUS_INK[1], FOCUS_INK[2], 0.45 * a] } else { [1.0, 1.0, 1.0, 0.5 * a] };
                d.inked_picture(chevron, at(right - 14.0 - 5.0), cy - 12.0 * scale, 24.0 * scale, 24.0 * scale, colour);
                right -= 14.0 + 20.0;
            }
            let row_style = if focused { &ROW_FOCUS } else { &ROW_TEXT };
            let value_room = if let Some(value) = &row.value {
                let shown = self.fit(d, value, row_style, VALUE_MAX);
                let w = self.measure(d, &shown, row_style);
                Self::write_scaled(d, &shown, row_style, at(right - w), base, scale, a * if focused { 0.55 } else { 0.55 });
                w + 24.0
            } else {
                0.0
            };
            let label = self.fit(d, &row.label, row_style, right - value_room - left);
            Self::write_scaled(d, &label, row_style, at(left), base, scale, a * if focused { 1.0 } else { 0.94 });
        }
        unclip();
    }

    fn draw_text_page(&mut self, d: &mut Draw, id: &'static str, dx: f32, alpha: f32, now: u32) {
        let all = self.options();
        let Some(o) = all.iter().find(|o| o.id == id).cloned() else { return };
        let placeholder = match &o.kind {
            Kind::Text { placeholder, .. } => *placeholder,
            _ => "",
        };
        self.draw_side(d, &o.label, group_named(o.group).1, &o.description, dx, alpha);
        let (x, y) = (ROW_X + dx, VIEW_TOP + TRACK_TOP);
        let plate = Self::row_plate(d, true);
        Self::plate(d, plate, x, y, ROW_W, ROW_H, 1.0, alpha);
        let base = baseline(y, ROW_H, 32.0);
        let shown_text = if self.typed.is_empty() { placeholder.to_string() } else { self.typed.clone() };
        let room = ROW_W - 2.0 * ROW_PAD - 8.0;
        let mut shown = shown_text.clone();
        while !shown.is_empty() && self.measure(d, &shown, &ROW_FOCUS) > room {
            shown.remove(0);
        }
        let w = self.write(d, &shown, &ROW_FOCUS, x + ROW_PAD, base, alpha * if self.typed.is_empty() { 0.4 } else { 1.0 });
        if !self.typed.is_empty() || shown.is_empty() {
            let caret_x = x + ROW_PAD + if self.typed.is_empty() { 0.0 } else { w } + 3.0;
            let caret = Self::shape(d, "set:caret", 3.0, 38.0, 1.5, 0.0, ShapeLook { fill: [0.067, 0.067, 0.067, 1.0], rim: 0.0, shadow: None });
            d.picture(caret, caret_x, y + (ROW_H - 38.0) / 2.0, 3.0, 38.0, alpha);
        }
        self.write(d, "Arrows and OK type · Save keeps it · Back cancels", &DESCRIPTION, VIEW_X + 84.0 + dx, baseline(VIEW_TOP + 110.0, 26.0 * 1.2, 26.0), 0.55 * alpha);
        let top = VIEW_TOP + 170.0;
        let width = KEY_ROWS[0].len() as f32 * (KEY_W + KEY_GAP) - KEY_GAP;
        let left = ROW_X + (ROW_W - width) / 2.0 + dx;
        let wide = (width - 3.0 * KEY_GAP) / 4.0;
        let mut keys: Vec<(usize, usize, f32, f32, f32)> = Vec::new();
        for (r, row) in KEY_ROWS.iter().enumerate() {
            for c in 0..row.len() {
                keys.push((r, c, left + c as f32 * (KEY_W + KEY_GAP), top + r as f32 * (KEY_H + KEY_GAP), KEY_W));
            }
        }
        for c in 0..KEY_ACTIONS.len() {
            keys.push((KEY_ROWS.len(), c, left + c as f32 * (wide + KEY_GAP), top + KEY_ROWS.len() as f32 * (KEY_H + KEY_GAP), wide));
        }
        keys.sort_by_key(|k| (k.0, k.1) == self.key_at);
        for (r, c, kx, ky, kw) in keys {
            let focused = (r, c) == self.key_at;
            let lift = self.lift_of(r * 100 + c, now);
            let scale = 1.0 + (LIFT_PILL - 1.0) * lift;
            let plate = if focused {
                (Self::shape(d, &format!("set:key-focus:{kw}"), kw, KEY_H, 20.0, SHAPE_MARGIN, ShapeLook { fill: [1.0, 1.0, 1.0, 1.0], rim: 0.0, shadow: Some(FOCUS_SHADOW) }), SHAPE_MARGIN)
            } else {
                (Self::shape(d, &format!("set:key:{kw}"), kw, KEY_H, 20.0, 0.0, ShapeLook { fill: [1.0, 1.0, 1.0, 0.09], rim: 1.0, shadow: None }), 0.0)
            };
            Self::plate(d, plate, kx, ky, kw, KEY_H, scale, alpha);
            let label = self.key_label(r, c);
            let style = if focused { &KEY_FOCUS } else { &KEY_TEXT };
            let w = self.measure(d, &label, style);
            let (cx, cy) = (kx + kw / 2.0, ky + KEY_H / 2.0);
            Self::write_scaled(d, &label, style, cx - w / 2.0 * scale, cy + (baseline(ky, KEY_H, 28.0) - cy) * scale, scale, alpha * if focused { 1.0 } else { 0.94 });
        }
    }

    fn head(&mut self, d: &mut Draw, title: &str, left: f32, hint: &str, dx: f32, alpha: f32) {
        self.write(d, title, &TITLE, left + dx, baseline(HEADER_TOP, 72.0, 56.0), 0.62 * alpha);
        self.write(d, hint, &HINT, left + dx, baseline(HEAD_BOTTOM, 22.0 * 1.2, 22.0), 0.55 * alpha);
    }

    fn icon_picture(&mut self, d: &mut Draw, item: &Item, grey: bool) -> (Tex, [f32; 3]) {
        let key = format!("sicon:{}:{}", item.key, grey);
        let colour = |this: &Self| this.tile_colours.get(&item.key).copied();
        if let (Some(t), Some(bg)) = (d.pictures.find(&key), colour(self)) {
            return (t, if grey { grey_of(bg) } else { bg });
        }
        let (picture, bg) = match &item.look {
            Look::Input(file) => {
                let path = format!("{}/icons/inputs/{}", d.app_dir, file);
                let icon = decoded(&path).or_else(|| decoded(&format!("{}/icons/inputs/hdmigeneric.png", d.app_dir)));
                (icon.map(|i| padded_glyph(&i, TILE_H as usize, 0.64, 2.4)), INPUT_TILE)
            }
            Look::Channel(name, logo) => (Some(crate::card::channel_card(&d.fonts, name, logo.as_deref(), &crate::card::TILE_CARD)), crate::card::colour_of(name)),
            Look::App(icon, colour, title) => {
                let file = match icon {
                    Icon::Bundled(file) => Some(format!("{}/icons/apps/{}", d.app_dir, file)),
                    Icon::OnTv(path) => {
                        let kept = store::path(&format!("icons/{}", kept_icon_name(path)));
                        Some(if std::path::Path::new(&kept).exists() { kept } else { path.clone() })
                    }
                    Icon::Glyph(_) | Icon::Missing => None,
                };
                let picture = file.as_deref().and_then(decoded);
                let bg = picture.as_ref().and_then(|p| background_of(&p.rgba, p.w as usize, p.h as usize)).or(*colour).unwrap_or(TILE_WITHOUT_COLOUR);
                (picture.or_else(|| Some(Self::monogram(d, title))), bg)
            }
        };
        self.tile_colours.insert(item.key.clone(), bg);
        let picture = picture.unwrap_or(Decoded { rgba: vec![0; 4], w: 1, h: 1 });
        let picture = if grey { greyed(&picture) } else { picture };
        let t = d.gfx.upload_rgba(&picture.rgba, picture.w, picture.h);
        d.keep(&key, t);
        (t, if grey { grey_of(bg) } else { bg })
    }

    fn monogram(d: &Draw, title: &str) -> Decoded {
        let size = TILE_H as usize;
        let mut rgba = vec![0u8; size * size * 4];
        let letter: String = title.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
        if let Some(line) = d.fonts.line(&letter, 64, "600", 0.0, "") {
            let (left, top) = ((size as i32 - line.advance) / 2 - line.left, (size as i32 + 64 * 7 / 10) / 2 - line.baseline);
            for y in 0..line.h {
                for x in 0..line.w {
                    let (tx, ty) = (x + left, y + top);
                    if tx >= 0 && ty >= 0 && (tx as usize) < size && (ty as usize) < size {
                        let cover = line.rgba[((y * line.w + x) * 4 + 3) as usize];
                        rgba[(ty as usize * size + tx as usize) * 4..][..4].copy_from_slice(&[(238 * cover as u32 / 255) as u8, (238 * cover as u32 / 255) as u8, (238 * cover as u32 / 255) as u8, cover]);
                    }
                }
            }
        }
        Decoded { rgba, w: size as i32, h: size as i32 }
    }

    fn tile_uniforms(d: &Draw, icon: Tex, bg: [f32; 3], (x, y, w, h): (f32, f32, f32, f32), shadow: [f32; 3], focus: f32, wide: bool) {
        let p = &d.p.tile;
        p.bind(0, c"u_icon", icon.tex);
        p.set4(c"u_box", x, y, w, h);
        p.set1(c"u_radius", TILE_RADIUS * w / TILE_W);
        p.set3(c"u_bg", bg);
        p.set3(c"u_shadow", shadow);
        p.set1(c"u_focus", focus);
        p.set1(c"u_wide", if wide { 1.0 } else { 0.0 });
        p.set1(c"u_turn", 0.0);
    }

    fn prepare_tiles(&mut self, d: &mut Draw, tab: Tab) -> Vec<Option<(Tex, Tex, [f32; 3])>> {
        let (items, shown) = self.grid_list(tab);
        let offset = self.grid_scroll.to_value();
        let mut out = Vec::with_capacity(items.len());
        for (i, item) in items.iter().enumerate() {
            let y = CONTENT_TOP + GRID_PAD + (i / GRID_COLUMNS) as f32 * GRID_STEP - offset;
            let near = y + TILE_H + 300.0 >= CONTENT_TOP && y - 300.0 <= CONTENT_TOP + GRID_VIEW_H + 2.0 * GRID_PAD;
            if !near && (i as isize - self.grid_index as isize).unsigned_abs() > GRID_COLUMNS * 4 {
                out.push(None);
                continue;
            }
            let grey = !self.reordering && !shown.iter().any(|s| s.key == item.key);
            let (icon, bg) = self.icon_picture(d, item, grey);
            let key = format!("stile:{}:{}", item.key, grey);
            let resting = match d.pictures.find(&key) {
                Some(t) => t,
                None => {
                    let (w, h) = (TILE_W + 2.0 * TILE_MARGIN, TILE_H + 2.0 * TILE_MARGIN);
                    let resting = d.gfx.blank(w as i32, h as i32);
                    unsafe { glDisable(GL_BLEND) }
                    d.gfx.at_design_scale_into(&resting);
                    Self::tile_uniforms(d, icon, bg, (TILE_MARGIN, TILE_MARGIN, TILE_W, TILE_H), SHADOW_REST, 0.0, matches!(item.look, Look::Channel(..)));
                    d.gfx.quad(&d.p.tile, 0.0, 0.0, w, h);
                    d.keep(&key, resting);
                    resting
                }
            };
            out.push(Some((resting, icon, bg)));
        }
        out
    }

    fn draw_grid(&mut self, d: &mut Draw, tab: Tab, pictures: Vec<Option<(Tex, Tex, [f32; 3])>>, dx: f32, alpha: f32, now: u32) {
        let (items, shown) = self.grid_list(tab);
        let (title, noun) = match tab {
            Tab::Apps => ("Apps", "apps"),
            Tab::Sources => ("Sources", "sources"),
            Tab::Channels => ("Channels", "channels"),
        };
        let hint = if self.held.is_some() {
            HINT_HOLDING
        } else if self.reordering {
            HINT_REORDER
        } else {
            HINT_SELECT
        };
        self.head(d, title, GRID_LEFT, hint, dx, alpha);
        self.draw_button(d, &if self.reordering { "Done".to_string() } else { format!("Re-order {noun}") }, dx, alpha, now);
        if self.reordering && items.is_empty() {
            self.write(d, "Nothing on the home screen yet.", &HINT, GRID_LEFT + dx, baseline(CONTENT_TOP, 22.0 * 1.2, 22.0), 0.55 * alpha);
        }
        let offset = self.grid_scroll.at(now);
        let focused = (self.zone == Zone::Content).then_some(self.grid_index);
        clip(d, 0.0, CONTENT_TOP, DESIGN_W, GRID_VIEW_H + 2.0 * GRID_PAD);
        let lifted = self.held.as_ref().map(|h| h.index);
        let order: Vec<usize> = (0..items.len()).filter(|i| Some(*i) != focused && Some(*i) != lifted).chain(focused).chain(lifted.filter(|l| Some(*l) != focused)).collect();
        for i in order {
            let Some(Some((resting, icon, bg))) = pictures.get(i).copied() else { continue };
            let item = &items[i];
            let (col, row) = (i % GRID_COLUMNS, i / GRID_COLUMNS);
            let x = GRID_LEFT + col as f32 * (TILE_W + TILE_GAP) + dx;
            let y = CONTENT_TOP + GRID_PAD + row as f32 * GRID_STEP - offset;
            if y + TILE_H + 80.0 < CONTENT_TOP || y - 80.0 > CONTENT_TOP + GRID_VIEW_H + 2.0 * GRID_PAD {
                continue;
            }
            let checked = !self.reordering && shown.iter().any(|s| s.key == item.key);
            let f = self.lift_of(i, now);
            let tile_alpha = alpha * if !self.reordering && !checked { 0.75 } else { 1.0 };
            let is_lifted = lifted == Some(i);
            let (scale, rise) = if is_lifted { (LIFTED, 6.0) } else { (1.0 + (LIFT - 1.0) * f, LIFT_RISE * f) };
            let (w, h) = (TILE_W * scale, TILE_H * scale);
            let (bx, by) = (x + TILE_W / 2.0 - w / 2.0, y + TILE_H / 2.0 - rise - h / 2.0);
            if f <= 0.0 && !is_lifted {
                d.picture(resting, x - TILE_MARGIN, y - TILE_MARGIN, TILE_W + 2.0 * TILE_MARGIN, TILE_H + 2.0 * TILE_MARGIN, tile_alpha);
            } else {
                if is_lifted {
                    let ring = Self::shape(d, "set:ring", TILE_W * LIFTED + 10.0, TILE_H * LIFTED + 10.0, TILE_RADIUS * LIFTED + 5.0, SHAPE_MARGIN, ShapeLook { fill: [0.933, 0.933, 0.933, 1.0], rim: 0.0, shadow: Some((18.0, 40.0, 0.7)) });
                    d.picture(ring, bx - 5.0 - SHAPE_MARGIN, by - 5.0 - SHAPE_MARGIN, w + 10.0 + 2.0 * SHAPE_MARGIN, h + 10.0 + 2.0 * SHAPE_MARGIN, alpha);
                }
                let lift = if is_lifted { 1.0 } else { f };
                let shadow = [0, 1, 2].map(|k| SHADOW_REST[k] + (SHADOW_LIFT[k] - SHADOW_REST[k]) * lift);
                let margin = shadow[0] + shadow[1] + 4.0;
                Self::tile_uniforms(d, icon, bg, (bx, by, w, h), shadow, lift, matches!(item.look, Look::Channel(..)));
                d.gfx.quad(&d.p.tile, bx - margin, by - margin, w + 2.0 * margin, h + 2.0 * margin);
            }
            if checked {
                let badge = Self::shape(d, "set:badge", 34.0, 34.0, 17.0, 0.0, ShapeLook { fill: [0.933, 0.933, 0.933, 1.0], rim: 0.0, shadow: None });
                let s = w / TILE_W;
                let (kx, ky) = (bx + w - (12.0 + 34.0) * s, by + 12.0 * s);
                d.picture(badge, kx, ky, 34.0 * s, 34.0 * s, alpha);
                let tick = d.glyph(TICK, 20, 3.0);
                d.inked_picture(tick, kx + 7.0 * s, ky + 7.0 * s, 20.0 * s, 20.0 * s, [0.0, 0.0, 0.0, alpha]);
            }
            if f > 0.0 {
                let lines = self.wrap(d, &item.label, &TILE_LABEL, TILE_W + TILE_GAP - 8.0, 2);
                for (n, line) in lines.iter().enumerate() {
                    let lw = self.measure(d, line, &TILE_LABEL);
                    self.write(d, line, &TILE_LABEL, x + TILE_W / 2.0 - lw / 2.0, baseline(y + TILE_H + 22.0 + n as f32 * 18.0 * 1.15, 18.0 * 1.15, 18.0), alpha * f);
                }
            }
        }
        unclip();
    }

    fn draw_button(&mut self, d: &mut Draw, label: &str, dx: f32, alpha: f32, now: u32) {
        let focused = self.zone == Zone::Button;
        let f = self.button_lift.at(now);
        let style = if focused { &BUTTON_FOCUS } else if self.reordering { &BUTTON_ON } else { &BUTTON };
        let tw = self.measure(d, label, style);
        let (w, h) = (tw + 60.0, 26.0 * 1.2 + 28.0);
        let scale = 1.0 + (LIFT_PILL - 1.0) * f;
        let right = GRID_RIGHT + dx;
        let cy = HEADER_TOP + 36.0;
        let (sw, sh) = (w * scale, h * scale);
        let (x, y) = (right - sw, cy - sh / 2.0);
        let key = format!("set:button:{}:{}:{}", w as i32, focused, self.reordering);
        let look = if focused {
            ShapeLook { fill: [1.0, 1.0, 1.0, 1.0], rim: 0.0, shadow: Some((14.0, 30.0, 0.3)) }
        } else {
            ShapeLook { fill: [1.0, 1.0, 1.0, if self.reordering { 0.22 } else { 0.1 }], rim: 1.0, shadow: None }
        };
        let margin = if focused { SHAPE_MARGIN } else { 0.0 };
        let plate = Self::shape(d, &key, w, h, h / 2.0, margin, look);
        if focused {
            let ring = Self::shape(d, &format!("set:button-ring:{}", w as i32), w + 12.0, h + 12.0, h / 2.0 + 6.0, 0.0, ShapeLook { fill: [1.0, 1.0, 1.0, 0.1], rim: 0.0, shadow: None });
            d.picture(ring, x - 6.0 * scale, y - 6.0 * scale, sw + 12.0 * scale, sh + 12.0 * scale, alpha);
        }
        d.picture(plate, x - margin * scale, y - margin * scale, sw + 2.0 * margin * scale, sh + 2.0 * margin * scale, alpha);
        Self::write_scaled(d, label, style, x + 30.0 * scale, cy + (baseline(cy - h / 2.0, h, 26.0) - cy) * scale, scale, alpha);
    }

    fn chips(&mut self, d: &Draw, categories: &[String]) -> Vec<(f32, f32)> {
        let inner = CHIPS_W - 12.0;
        let widths: Vec<f32> = categories.iter().map(|c| self.measure(d, c, &CHIP_ON)).collect();
        let spare = ((inner - widths.iter().sum::<f32>()) / categories.len().max(1) as f32).max(0.0);
        let mut x = ROWS_EDGE + 6.0;
        widths
            .iter()
            .map(|w| {
                let chip = (x, w + spare);
                x += w + spare;
                chip
            })
            .collect()
    }

    fn draw_wallpaper(&mut self, d: &mut Draw, dx: f32, alpha: f32, now: u32) {
        self.head(d, "Wallpaper", ROWS_EDGE, HINT_WALLPAPER, dx, alpha);
        let (categories, _, _) = self.wallpaper_tab();
        let chips = self.chips(d, &categories);
        let bar = Self::shape(d, "set:chips", CHIPS_W, CHIPS_H, CHIPS_H / 2.0, SHAPE_MARGIN, ShapeLook { fill: [1.0, 1.0, 1.0, 0.1], rim: 1.0, shadow: None });
        let bar_focus = Self::shape(d, "set:chips-focus", CHIPS_W, CHIPS_H, CHIPS_H / 2.0, SHAPE_MARGIN, ShapeLook { fill: [1.0, 1.0, 1.0, 0.1], rim: 1.0, shadow: Some((14.0, 30.0, 0.3)) });
        let chips_focused = self.zone == Zone::Chips;
        let plate = if chips_focused { bar_focus } else { bar };
        d.picture(plate, ROWS_EDGE - SHAPE_MARGIN + dx, CONTENT_TOP - SHAPE_MARGIN, CHIPS_W + 2.0 * SHAPE_MARGIN, CHIPS_H + 2.0 * SHAPE_MARGIN, alpha);
        if let Some(&(cx, cw)) = chips.get(self.wp_cat) {
            if self.pill_x.to_value() < 0.0 {
                self.pill_x.set(cx);
                self.pill_w.set(cw);
                self.pill_alpha.set(if chips_focused { 1.0 } else { 0.22 });
            } else {
                self.pill_x.go(cx, self.ms(160.0), &EASE_SCROLL, now);
                self.pill_w.go(cw, self.ms(160.0), &EASE_SCROLL, now);
            }
            let (px, pw) = (self.pill_x.at(now), self.pill_w.at(now));
            let pill = Self::shape(d, &format!("set:pill:{}", cw as i32), cw, CHIP_H, CHIP_H / 2.0, 0.0, ShapeLook { fill: [1.0, 1.0, 1.0, 1.0], rim: 0.0, shadow: None });
            d.picture(pill, px + dx, CONTENT_TOP + 6.0, pw, CHIP_H, alpha * self.pill_alpha.at(now));
        }
        for (i, name) in categories.iter().enumerate() {
            let Some(&(cx, cw)) = chips.get(i) else { continue };
            let on = i == self.wp_cat;
            let style = if on && chips_focused { &CHIP_FOCUS } else if on { &CHIP_ON } else { &CHIP };
            let w = self.measure(d, name, style);
            self.write(d, name, style, cx + cw / 2.0 - w / 2.0 + dx, baseline(CONTENT_TOP + 6.0, CHIP_H, 22.0), alpha * if on { 1.0 } else { 0.75 });
        }
        let top = CONTENT_TOP + CHIPS_H + 26.0;
        let list = self.wp_list(self.wp_shown);
        let offset = self.grid_scroll.at(now);
        let focused = (self.zone == Zone::Content).then_some(self.wp_index);
        let row = self.wp_index / WP_COLUMNS;
        let (from, to) = (row.saturating_sub(1) * WP_COLUMNS, ((row + 4) * WP_COLUMNS).min(list.len()));
        clip(d, 0.0, top - 20.0, DESIGN_W, WP_VIEW_H + 20.0);
        let order: Vec<usize> = (from..to).filter(|i| Some(*i) != focused).chain(focused.filter(|f| (from..to).contains(f))).collect();
        for i in order {
            let entry = &list[i];
            let x = ROWS_EDGE + (i % WP_COLUMNS) as f32 * WP_COL_STEP + dx;
            let y = top + (i / WP_COLUMNS) as f32 * WP_ROW_STEP - offset;
            if y + WP_ROW_STEP < top - 20.0 || y > top + WP_VIEW_H {
                continue;
            }
            let f = self.lift_of(i, now);
            let scale = 1.0 + 0.1 * f;
            let (w, h) = (WP_THUMB_W * scale, WP_THUMB_H * scale);
            let (tx, ty) = (x + WP_THUMB_W / 2.0 - w / 2.0, y + WP_THUMB_H / 2.0 - 6.0 * f - h / 2.0);
            if f > 0.0 {
                let shadow = Self::shape(d, "set:thumb-shadow", WP_THUMB_W, WP_THUMB_H, 22.0, SHAPE_MARGIN, ShapeLook { fill: [0.0, 0.0, 0.0, 0.0], rim: 0.0, shadow: Some((26.0, 40.0, 0.42)) });
                d.picture(shadow, tx - SHAPE_MARGIN * scale, ty - SHAPE_MARGIN * scale, w + 2.0 * SHAPE_MARGIN * scale, h + 2.0 * SHAPE_MARGIN * scale, alpha * f);
            }
            let thumb = entry.thumb.as_ref().and_then(|(id, url)| {
                let found = d.pictures.find(&format!("thumb:{id}"));
                if found.is_none() {
                    self.thumbs.forget(id);
                    self.thumbs.want(id, url, self.online);
                }
                found
            });
            match thumb {
                Some(t) => d.picture(t, tx, ty, w, h, alpha),
                None => {
                    let blank = Self::shape(d, "set:thumb", WP_THUMB_W, WP_THUMB_H, 22.0, 0.0, ShapeLook { fill: [1.0, 1.0, 1.0, 0.1], rim: 0.0, shadow: None });
                    d.picture(blank, tx, ty, w, h, alpha);
                    if let Some(plain) = &entry.plain {
                        let pw = self.measure(d, plain, &WP_PLAIN);
                        Self::write_scaled(d, plain, &WP_PLAIN, tx + w / 2.0 - pw * scale / 2.0, ty + h / 2.0 + 9.0 * scale, scale, alpha);
                    }
                }
            }
            if entry.on {
                let tick = Self::shape(d, "set:tick", 38.0, 38.0, 19.0, 0.0, ShapeLook { fill: [1.0, 1.0, 1.0, 1.0], rim: 0.0, shadow: None });
                let (kx, ky) = (tx + w - (12.0 + 38.0) * scale, ty + 12.0 * scale);
                d.picture(tick, kx, ky, 38.0 * scale, 38.0 * scale, alpha);
                let mark = d.glyph(TICK, 26, 2.8);
                d.inked_picture(mark, kx + 6.0 * scale, ky + 6.0 * scale, 26.0 * scale, 26.0 * scale, [0.0, 0.0, 0.0, alpha]);
            }
            let style = if Some(i) == focused { &WP_TITLE_FOCUS } else { &WP_TITLE };
            let title = self.fit(d, &entry.title, style, WP_THUMB_W);
            self.write(d, &title, style, x, baseline(y + WP_THUMB_H + 14.0, 22.0 * 1.2, 22.0), alpha);
        }
        unclip();
    }
}

fn install_each(side: tv::SideApps, mut left: Vec<&'static str>, mut lines: Vec<String>, done: Box<dyn FnOnce(Result<String, String>) + Send>) {
    if left.is_empty() {
        let failed: Vec<&String> = lines.iter().filter(|l| !l.ends_with(" installed")).collect();
        let summary = if !failed.is_empty() { failed.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(". ") } else if lines.len() == 1 { lines[0].clone() } else { format!("{} side apps updated", lines.len()) };
        return done(Ok(summary));
    }
    let app = left.remove(0);
    let title = tv::side_title(app);
    let Some(next) = side.latest(app).cloned() else {
        lines.push(format!("{title} is not in a release yet"));
        return install_each(side, left, lines, done);
    };
    root::install_side_app(app, &next.url, &next.sha256, move |out| {
        lines.push(match out {
            Ok(r) if r == "installed" => format!("{title} {} installed", next.version),
            Ok(r) => format!("{title}: {r}"),
            Err(why) => format!("{title}: {why}"),
        });
        install_each(side, left, lines, done);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_list_keeps_the_focused_row_in_the_middle_until_its_end() {
        assert_eq!(row_window(30, 0, 9, 4), 0);
        assert_eq!(row_window(30, 10, 9, 4), 6);
        assert_eq!(row_window(30, 29, 9, 4), 21);
        assert_eq!(row_window(5, 4, 9, 4), 0);
    }

    #[test]
    fn a_grid_that_fits_stays_still_and_a_long_one_centres_the_focused_row() {
        assert_eq!(grid_offset(12, 6, 11, 100.0, 40.0, 720.0, 0.0), 0.0);
        let total = 10.0 * 100.0 - 40.0;
        assert_eq!(grid_offset(60, 6, 0, 100.0, 40.0, 720.0, 30.0), 0.0);
        assert_eq!(grid_offset(60, 6, 30, 100.0, 40.0, 720.0, 30.0), 500.0 - 310.0);
        assert_eq!(grid_offset(60, 6, 59, 100.0, 40.0, 720.0, 30.0), total - 720.0 + 30.0);
    }

    #[test]
    fn where_names_the_focused_row_by_its_part_and_label_on_each_page() {
        let mut screen = SettingsScreen::new(json!({}), Vec::new(), 0);
        let at = |screen: &mut SettingsScreen| {
            let w = screen.where_now();
            (w["page"].as_str().unwrap_or("").to_string(), w["focused"][0]["part"].as_str().unwrap_or("").to_string(), w["focused"][0]["label"].as_str().unwrap_or("").to_string())
        };
        assert_eq!(at(&mut screen), ("settings".into(), "slist__row--focused".into(), "Apps".into()));
        for key in [Key::Down, Key::Down, Key::Down, Key::Ok] {
            screen.react(key, 0);
        }
        assert_eq!(at(&mut screen), ("Wallpaper".into(), "slist__row--focused".into(), "Wallpaper type".into()));
        screen.react(Key::Back, 0);
        screen.react(Key::Up, 0);
        screen.react(Key::Up, 0);
        screen.react(Key::Up, 0);
        screen.react(Key::Ok, 0);
        assert_eq!(at(&mut screen).0, "apps");
        assert_eq!(at(&mut screen).1, "tile--focused");
    }

}
