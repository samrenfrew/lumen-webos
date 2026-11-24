use crate::anim::{Anim, EASE_GROW, EASE_OUT};
use crate::draw::{decoded, Draw, Glass, Style};
use crate::gfx::{Tex, DESIGN_W};
use crate::home::{pill_left, Key};
use crate::inputs::{self, Input};
use crate::root::{PICTURE_MODES, SOUND_MODES};
use crate::text::Shadow;
use crate::tvcontrols::{self, Controls};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, PartialEq, Debug)]
pub enum Kind {
    Controls,
    Inputs(Vec<String>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Outcome {
    Stay,
    Close,
    CloseDown,
    Open(String),
    Action(&'static str),
    Setting(&'static str, Value),
    Toast(String, bool),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Cell {
    Sound,
    Picture,
    Clock,
    Sleep,
    Art,
    Home,
    ScreenOff,
    Settings,
    Backlight,
    Eye,
}

// ponytail: Restart, which only Quick Settings' layout shows
const GRID: [&[(Cell, usize)]; 4] = [
    &[(Cell::Sound, 2), (Cell::Picture, 2)],
    &[(Cell::Clock, 2), (Cell::Sleep, 2)],
    &[(Cell::Art, 1), (Cell::Home, 1), (Cell::ScreenOff, 1), (Cell::Settings, 1)],
    &[(Cell::Backlight, 3), (Cell::Eye, 1)],
];
const COLUMNS: usize = 4;
const SLEEP_CHOICES: [&str; 7] = ["off", "10", "30", "60", "90", "120", "180"];

const UNIT: f32 = 90.0;
const GAP: f32 = 18.0;
const RIGHT_EDGE: f32 = 125.0;
const PANEL_TOP: f32 = 150.0;
const CONTROLS_W: f32 = 4.0 * UNIT + 3.0 * GAP;
const INPUTS_W: f32 = 3.0 * UNIT + 2.0 * GAP;
const CAPTION_H: f32 = 30.0;
const OPT_H: f32 = 72.0;
const OPT_GAP: f32 = 12.0;
const PILL_PAD: f32 = 18.0;
const FOCUS_LIFT: f32 = 1.1;
const PRESS_LIFT: f32 = 1.03;
const BUTTON: f32 = 58.0;
const BUTTON_TOP: f32 = 64.0;
const CONTROLS_BUTTON: usize = 0;
const INPUTS_BUTTON: usize = 2;
const CONTROLS_GROW_MS: f32 = 460.0;
const INPUTS_GROW_MS: f32 = 260.0;
const GROW_FROM: f32 = 0.6;
const DOUBLE_DOWN_MS: u32 = 1000;
const BACKLIGHT_STEP: i32 = 5;
const BACKLIGHT_DELAY_MS: u64 = 300;
const OPEN_MS: f32 = 330.0;
const CLOSE_MS: f32 = 300.0;
const LONG_LIST: usize = 4;
const LONG_STRETCH: f32 = 0.85;
const GRID_SHRINK: f32 = 0.15;
const SHAPE_BY: f32 = 0.35;
const GROW: [f32; 11] = [0.0, 0.12, 0.35, 0.6, 0.8, 0.9, 0.94, 0.97, 0.985, 0.995, 1.0];
// ponytail: the backlight fill is one flat frost, without the web gradient and inner rim
const FILL_TINT: f32 = 0.14;
const FILL_TINT_FOCUSED: f32 = 0.24;

const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
const GLYPH_INK: [f32; 3] = [1.0, 1.0, 1.0];
const GLYPH_ALPHA: f32 = 0.9;
const ACCENT_ON: [f32; 3] = [64.0 / 255.0, 196.0 / 255.0, 1.0];
const GLASS_INK_SHADOW: [Shadow; 1] = [Shadow { drop: 1, blur: 8, strength: 0.35 }];
const CAPTION_SHADOW: [Shadow; 1] = [Shadow { drop: 2, blur: 10, strength: 0.6 }];
const NAME: Style = Style { px: 22, weight: "600", spacing_em: 0.0, feature: "", ink: WHITE, shadows: &GLASS_INK_SHADOW };
const SUB: Style = Style { weight: "400", ..NAME };
const NUMBER: Style = Style { weight: "500", ..NAME };
const CAPTION: Style = Style { shadows: &CAPTION_SHADOW, ..NAME };

const SPEAKER: &str = "M4 9.5h3.5L12 6v12l-4.5-3.5H4zM15.5 9a4 4 0 0 1 0 6M18 6.5a8 8 0 0 1 0 11";
const PICTURE: &str = "M3.5 5.5h17v11h-17zM8 20h8M12 16.5V20";
const MOON: &str = "M20 14.5A8.5 8.5 0 1 1 9.5 4a6.8 6.8 0 0 0 10.5 10.5z";
const TIMER: &str = "M12 8v4.5l3 1.8M12 4.5a8 8 0 1 0 0 16 8 8 0 0 0 0-16zM9.5 2.5h5";
const HOUSE: &str = "M3 11.5 12 4l9 7.5M5.5 9.8V20h13V9.8M10 20v-6h4v6";
const GEAR: &str = "M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z";
const IMAGE: &str = "M3.5 5h17v14h-17zM3.5 15.5l5-5 4 4 3-3 5 5M15.5 9.3a.8.8 0 1 0 0-.1";
const POWER: &str = "M12 3.5v8M7 6.5a7.5 7.5 0 1 0 10 0";
const SUN: &str = "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.3 5.3l1.4 1.4M17.3 17.3l1.4 1.4M5.3 18.7l1.4-1.4M17.3 6.7l1.4-1.4";
const EYE: &str = "M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12zM12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6z";
const STAND_IN_ICON: &str = "hdmigeneric.png";

static BACKLIGHT_WAITING: Mutex<Option<(u64, i32)>> = Mutex::new(None);
static BACKLIGHT_TURN: AtomicU64 = AtomicU64::new(0);

fn toast_later(text: &'static str) {
    crate::post(move |app| {
        app.home.toast(text, true, crate::ticks());
        app.redraw = true;
    });
}

fn backlight_soon(level: i32) {
    let turn = BACKLIGHT_TURN.fetch_add(1, Ordering::SeqCst) + 1;
    if let Ok(mut waiting) = BACKLIGHT_WAITING.lock() {
        *waiting = Some((turn, level));
    }
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(BACKLIGHT_DELAY_MS));
        send_waiting_backlight(Some(turn));
    });
}

fn send_waiting_backlight(only: Option<u64>) {
    let level = BACKLIGHT_WAITING.lock().ok().and_then(|mut waiting| {
        let taken = match *waiting {
            Some((turn, level)) if only.is_none_or(|o| o == turn) => Some(level),
            _ => None,
        };
        if taken.is_some() {
            *waiting = None;
        }
        taken
    });
    if let Some(level) = level {
        tvcontrols::set_backlight(level, |done| {
            if done.is_err() {
                toast_later("Couldn't change the backlight");
            }
        });
    }
}

fn cell_at(row: usize, x: usize) -> (Cell, usize) {
    let mut start = 0;
    for &(cell, span) in GRID[row] {
        if x < start + span {
            return (cell, start);
        }
        start += span;
    }
    let (last, span) = GRID[row][GRID[row].len() - 1];
    (last, COLUMNS - span)
}

fn start_of(row: usize, i: usize) -> usize {
    GRID[row][..i].iter().map(|c| c.1).sum()
}

fn cell_box(cell: Cell) -> (f32, f32, f32, f32) {
    for (row, cells) in GRID.iter().enumerate() {
        if let Some(i) = cells.iter().position(|c| c.0 == cell) {
            let span = cells[i].1 as f32;
            return (start_of(row, i) as f32 * (UNIT + GAP), row as f32 * (UNIT + GAP), span * UNIT + (span - 1.0) * GAP, UNIT);
        }
    }
    (0.0, 0.0, UNIT, UNIT)
}

fn cell_index(cell: Cell) -> usize {
    GRID.iter().flat_map(|cells| cells.iter()).position(|c| c.0 == cell).unwrap_or(0)
}

fn cell_label(cell: Cell) -> &'static str {
    match cell {
        Cell::Sound => "Sound",
        Cell::Picture => "Picture",
        Cell::Clock => "Clock",
        Cell::Sleep => "Sleep",
        Cell::Art => "Screensaver",
        Cell::Home => "LG Home",
        Cell::ScreenOff => "Screen off",
        Cell::Settings => "All settings",
        Cell::Backlight => "Backlight",
        Cell::Eye => "Eye Comfort",
    }
}

fn circle_glyph(cell: Cell) -> &'static str {
    match cell {
        Cell::Home => HOUSE,
        Cell::Settings => GEAR,
        Cell::Art => IMAGE,
        Cell::ScreenOff => POWER,
        _ => EYE,
    }
}

fn list_glyph(cell: Cell) -> &'static str {
    match cell {
        Cell::Sound => SPEAKER,
        Cell::Picture => PICTURE,
        Cell::Sleep => MOON,
        _ => TIMER,
    }
}

fn clock_label(format: &str) -> &'static str {
    match format {
        "12h" => "12-hour",
        "24h" => "24-hour",
        _ => "Auto",
    }
}

fn options(cell: Cell) -> Vec<(String, String)> {
    match cell {
        Cell::Sound => SOUND_MODES.iter().map(|m| (m.to_string(), tvcontrols::sound_label(m).to_string())).collect(),
        Cell::Picture => PICTURE_MODES.iter().map(|m| (m.to_string(), tvcontrols::picture_label(m).to_string())).collect(),
        Cell::Sleep => SLEEP_CHOICES.iter().map(|m| (m.to_string(), tvcontrols::sleep_label(m))).collect(),
        _ => ["auto", "12h", "24h"].iter().map(|f| (f.to_string(), clock_label(f).to_string())).collect(),
    }
}

fn ramp(t: f32, a: f32, b: f32) -> f32 {
    ((t - a) / (b - a)).clamp(0.0, 1.0)
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn track(points: &[f32], t: f32) -> f32 {
    let at = t.clamp(0.0, 1.0) * (points.len() - 1) as f32;
    let i = (at.floor() as usize).min(points.len() - 2);
    points[i] + (points[i + 1] - points[i]) * (at - i as f32)
}

fn list_height(count: usize) -> f32 {
    count as f32 * OPT_H + count.saturating_sub(1) as f32 * OPT_GAP
}

#[derive(Clone, Copy, Debug)]
struct End {
    tx: f32,
    ty: f32,
    sx: f32,
    sy: f32,
    r: f32,
}

#[derive(Clone, Copy, Debug)]
struct Geo {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    fw: f32,
    fh: f32,
    tile: End,
    list: End,
}

fn geometry(cell: Cell, count: usize) -> Geo {
    let (x, y, w, h) = cell_box(cell);
    let fh = list_height(count);
    let s = FOCUS_LIFT;
    Geo {
        x,
        y,
        w,
        h,
        fw: CONTROLS_W,
        fh,
        tile: End { tx: (1.0 - s) * w / 2.0, ty: (1.0 - s) * h / 2.0, sx: s, sy: s, r: h / 2.0 * s },
        list: End { tx: -x - PILL_PAD, ty: -y - PILL_PAD, sx: (CONTROLS_W + 2.0 * PILL_PAD) / w, sy: (fh + 2.0 * PILL_PAD) / h, r: OPT_H / 2.0 },
    }
}

fn opening(g: &Geo, t: f32) -> End {
    let k = track(&GROW, t);
    let (l0, t0, w0, h0) = (g.x + g.tile.tx, g.y + g.tile.ty, g.w * g.tile.sx, g.h * g.tile.sy);
    let (l1, t1, w1, h1) = (g.x + g.list.tx, g.y + g.list.ty, g.w * g.list.sx, g.h * g.list.sy);
    let h = h0 + (h1 - h0) * k;
    let shape = w0 / h0 + (w1 / h1 - w0 / h0) * ease_out(ramp(t, 0.0, SHAPE_BY));
    let w = (h * shape).min(w0.max(w1)).max(w0.min(w1));
    let px = if w1 == w0 { k } else { (w - w0) / (w1 - w0) };
    let left = l0 + (l1 - l0) * px;
    let top = t0 + (t1 - t0) * k;
    End { tx: left - g.x, ty: top - g.y, sx: w / g.w, sy: h / g.h, r: g.tile.r + (g.list.r - g.tile.r) * k }
}

#[derive(Clone, Copy)]
struct Scale {
    ox: f32,
    oy: f32,
    s: f32,
}

impl Scale {
    fn none() -> Scale {
        Scale { ox: 0.0, oy: 0.0, s: 1.0 }
    }

    fn at(&self, x: f32, y: f32) -> (f32, f32) {
        (self.ox + (x - self.ox) * self.s, self.oy + (y - self.oy) * self.s)
    }
}

enum Mark {
    Glyph(&'static str, i32),
    Picture(Tex),
}

struct Row<'a> {
    width: f32,
    mark: Mark,
    mark_box: f32,
    name: &'a str,
    sub: Option<String>,
    ink: [f32; 3],
}

struct Menu {
    cell: Cell,
    index: usize,
    lifts: Vec<Anim>,
}

#[derive(Clone, Copy)]
struct Morph {
    open: bool,
    start: u32,
    ms: f32,
}

pub struct ControlCenter {
    kind: Kind,
    clock_format: String,
    row: usize,
    x: usize,
    menu: Option<Menu>,
    morph: Option<Morph>,
    lifts: Vec<Anim>,
    grow: Anim,
    bottom_down: Option<u32>,
    inputs: Arc<Mutex<Option<Vec<Input>>>>,
    input_index: usize,
    input_lifts: Vec<Anim>,
    drawn_at_rest: bool,
}

impl Drop for ControlCenter {
    fn drop(&mut self) {
        send_waiting_backlight(None);
    }
}

impl ControlCenter {
    pub fn new(kind: Kind, clock_format: &str, now: u32) -> ControlCenter {
        let panel = ControlCenter::fresh(kind, clock_format, now);
        match &panel.kind {
            Kind::Controls => tvcontrols::read_all(),
            Kind::Inputs(ids) => {
                let (ids, shown) = (ids.clone(), panel.inputs.clone());
                inputs::read(move |read| match read {
                    Ok(all) => {
                        if let Ok(mut shown) = shown.lock() {
                            *shown = Some(inputs::chosen(&all, &ids));
                        }
                        crate::wake();
                    }
                    Err(_) => toast_later("Could not read the inputs"),
                });
            }
        }
        panel
    }

    fn fresh(kind: Kind, clock_format: &str, now: u32) -> ControlCenter {
        let cells = GRID.iter().map(|r| r.len()).sum();
        let mut lifts = vec![Anim::still(0.0); cells];
        lifts[0] = Anim::still(1.0);
        let shown = match &kind {
            Kind::Inputs(ids) => inputs::last_read().map(|all| inputs::chosen(&all, ids)),
            Kind::Controls => None,
        };
        let mut grow = Anim::still(0.0);
        let grow_ms = if kind == Kind::Controls { CONTROLS_GROW_MS } else { INPUTS_GROW_MS };
        grow.go(1.0, grow_ms, &EASE_GROW, now);
        ControlCenter {
            kind,
            clock_format: clock_format.to_string(),
            row: 0,
            x: 0,
            menu: None,
            morph: None,
            lifts,
            grow,
            bottom_down: None,
            inputs: Arc::new(Mutex::new(shown)),
            input_index: 0,
            input_lifts: Vec::new(),
            drawn_at_rest: false,
        }
    }

    pub fn key(&mut self, d: &Draw, key: Key, now: u32) -> Outcome {
        self.react(key, now, d.look.focus_ms)
    }

    fn react(&mut self, key: Key, now: u32, focus_ms: f32) -> Outcome {
        self.settle(now);
        let outcome = match self.kind {
            Kind::Controls => self.controls_key(key, now),
            Kind::Inputs(_) => self.inputs_key(key, now),
        };
        self.lift_focused(now, focus_ms);
        outcome
    }

    fn lift_focused(&mut self, now: u32, focus_ms: f32) {
        let focused = cell_index(cell_at(self.row, self.x).0);
        for (i, lift) in self.lifts.iter_mut().enumerate() {
            lift.go(if i == focused { 1.0 } else { 0.0 }, focus_ms, &EASE_OUT, now);
        }
        if let Some(menu) = self.menu.as_mut() {
            for (i, lift) in menu.lifts.iter_mut().enumerate() {
                lift.go(if i == menu.index { 1.0 } else { 0.0 }, focus_ms, &EASE_OUT, now);
            }
        }
        let count = self.inputs.lock().ok().and_then(|l| l.as_ref().map(Vec::len)).unwrap_or(0);
        if self.input_lifts.len() != count {
            self.input_lifts = (0..count).map(|i| Anim::still(if i == self.input_index { 1.0 } else { 0.0 })).collect();
        }
        for (i, lift) in self.input_lifts.iter_mut().enumerate() {
            lift.go(if i == self.input_index { 1.0 } else { 0.0 }, focus_ms, &EASE_OUT, now);
        }
    }

    fn settle(&mut self, now: u32) {
        let Some(morph) = self.morph else { return };
        if (now.wrapping_sub(morph.start) as i32) as f32 >= morph.ms {
            self.morph = None;
            if !morph.open {
                self.menu = None;
            }
        }
    }

    fn morph_t(&self, now: u32) -> Option<(bool, f32)> {
        self.morph.map(|m| (m.open, ((now.wrapping_sub(m.start) as i32) as f32 / m.ms).clamp(0.0, 1.0)))
    }

    fn double_down(&mut self, now: u32) -> Outcome {
        if self.bottom_down.is_some_and(|at| now.wrapping_sub(at) < DOUBLE_DOWN_MS) {
            self.bottom_down = None;
            return Outcome::CloseDown;
        }
        self.bottom_down = Some(now);
        Outcome::Stay
    }

    fn go_to(&mut self, row: usize, x: usize) {
        self.row = row;
        self.x = x;
        self.bottom_down = None;
    }

    fn controls_key(&mut self, key: Key, now: u32) -> Outcome {
        if self.morph.is_some() {
            return Outcome::Stay;
        }
        if let Some(menu) = self.menu.as_mut() {
            let count = options(menu.cell).len();
            return match key {
                Key::Back => self.close_menu(false, now),
                Key::Ok => self.close_menu(true, now),
                Key::Up => {
                    menu.index = menu.index.saturating_sub(1);
                    Outcome::Stay
                }
                Key::Down => {
                    menu.index = (menu.index + 1).min(count - 1);
                    Outcome::Stay
                }
                _ => Outcome::Stay,
            };
        }
        let (cell, start) = cell_at(self.row, self.x);
        match key {
            Key::Back => Outcome::Close,
            Key::Ok => self.press(cell, now),
            Key::Left | Key::Right if cell == Cell::Backlight => {
                let by = if key == Key::Left { -BACKLIGHT_STEP } else { BACKLIGHT_STEP };
                let level = (tvcontrols::values().backlight + by).clamp(0, 100);
                tvcontrols::patch(|c| c.backlight = level);
                backlight_soon(level);
                Outcome::Stay
            }
            Key::Left | Key::Right => {
                let cells = GRID[self.row];
                let i = cells.iter().position(|c| c.0 == cell).unwrap_or(0);
                let j = if key == Key::Left { i.saturating_sub(1) } else { (i + 1).min(cells.len() - 1) };
                if j != i {
                    self.go_to(self.row, start_of(self.row, j));
                }
                Outcome::Stay
            }
            Key::Up if self.row == 0 => Outcome::Close,
            Key::Down if self.row == GRID.len() - 1 => self.double_down(now),
            Key::Up => {
                self.go_to(self.row - 1, start);
                Outcome::Stay
            }
            Key::Down => {
                self.go_to(self.row + 1, start);
                Outcome::Stay
            }
            Key::Digit(_) => Outcome::Stay,
        }
    }

    fn press(&mut self, cell: Cell, now: u32) -> Outcome {
        match cell {
            Cell::Eye => {
                let on = !tvcontrols::values().eye_comfort;
                tvcontrols::patch(|c| c.eye_comfort = on);
                tvcontrols::set_eye_comfort(on, move |done| {
                    if done.is_err() {
                        tvcontrols::patch(|c| c.eye_comfort = !on);
                        toast_later("Couldn't change Eye Comfort");
                    }
                });
                Outcome::Stay
            }
            Cell::Sound | Cell::Picture | Cell::Clock | Cell::Sleep => {
                self.open_menu(cell, now);
                Outcome::Stay
            }
            Cell::Art => Outcome::Action("art"),
            Cell::Home => Outcome::Action("lg-home"),
            Cell::Settings => Outcome::Action("settings"),
            Cell::ScreenOff => {
                tvcontrols::screen_off(|done| {
                    if done.is_err() {
                        toast_later("Couldn't turn the screen off (it needs the Homebrew Channel)");
                    }
                });
                Outcome::Close
            }
            Cell::Backlight => Outcome::Stay,
        }
    }

    fn chosen(&self, cell: Cell, values: &Controls) -> String {
        match cell {
            Cell::Sound => values.sound.clone(),
            Cell::Picture => values.picture.clone(),
            Cell::Sleep => values.sleep.clone(),
            _ => self.clock_format.clone(),
        }
    }

    fn open_menu(&mut self, cell: Cell, now: u32) {
        let list = options(cell);
        let chosen = self.chosen(cell, &tvcontrols::values());
        let index = list.iter().position(|o| o.0 == chosen).unwrap_or(0);
        let lifts = (0..list.len()).map(|i| Anim::still(if i == index { 1.0 } else { 0.0 })).collect();
        self.menu = Some(Menu { cell, index, lifts });
        self.morph = Some(Morph { open: true, start: now, ms: Self::morph_ms(OPEN_MS, list.len()) });
    }

    fn morph_ms(ms: f32, count: usize) -> f32 {
        ms * if count > LONG_LIST { LONG_STRETCH } else { 1.0 }
    }

    fn close_menu(&mut self, pick: bool, now: u32) -> Outcome {
        let Some((cell, index)) = self.menu.as_ref().map(|m| (m.cell, m.index)) else { return Outcome::Stay };
        let list = options(cell);
        let outcome = match list.get(index) {
            Some((value, _)) if pick => self.apply(cell, value),
            _ => Outcome::Stay,
        };
        self.morph = Some(Morph { open: false, start: now, ms: Self::morph_ms(CLOSE_MS, list.len()) });
        outcome
    }

    fn apply(&mut self, cell: Cell, value: &str) -> Outcome {
        let values = tvcontrols::values();
        if value == self.chosen(cell, &values) {
            return Outcome::Stay;
        }
        let value = value.to_string();
        match cell {
            Cell::Sound => {
                let before = values.sound;
                tvcontrols::patch(|c| c.sound = value.clone());
                tvcontrols::set_sound_mode(&value, move |done| {
                    if done.is_err() {
                        tvcontrols::patch(|c| c.sound = before);
                        toast_later("This TV didn't accept that sound mode");
                    }
                });
                Outcome::Stay
            }
            Cell::Picture => {
                tvcontrols::patch(|c| c.picture = value.clone());
                tvcontrols::set_picture_mode(&value, |done| match done {
                    Ok(()) => tvcontrols::read_backlight(),
                    Err(_) => toast_later("Couldn't change the picture mode"),
                });
                Outcome::Stay
            }
            Cell::Sleep => {
                let before = values.sleep;
                tvcontrols::patch(|c| c.sleep = value.clone());
                tvcontrols::set_sleep_timer(&value, move |done| {
                    if done.is_err() {
                        tvcontrols::patch(|c| c.sleep = before);
                        toast_later("Couldn't set the sleep timer");
                    }
                });
                Outcome::Stay
            }
            _ => {
                self.clock_format = value.clone();
                Outcome::Setting("clockFormat", json!(value))
            }
        }
    }

    fn inputs_key(&mut self, key: Key, now: u32) -> Outcome {
        let shown = self.inputs.lock().ok().and_then(|l| l.clone()).unwrap_or_default();
        match key {
            Key::Up if self.input_index == 0 => Outcome::Close,
            Key::Up => {
                self.input_index -= 1;
                self.bottom_down = None;
                Outcome::Stay
            }
            Key::Down if self.input_index + 1 < shown.len() => {
                self.input_index += 1;
                self.bottom_down = None;
                Outcome::Stay
            }
            Key::Down => self.double_down(now),
            Key::Back => Outcome::Close,
            Key::Ok => shown.get(self.input_index).map_or(Outcome::Stay, |input| Outcome::Open(input.app_id.clone())),
            _ => Outcome::Stay,
        }
    }

    fn grown(&self, button: usize, now: u32) -> (Scale, f32) {
        let g = self.grow.at(now);
        (Scale { ox: pill_left(button) + BUTTON / 2.0, oy: BUTTON_TOP + BUTTON / 2.0, s: GROW_FROM + (1.0 - GROW_FROM) * g }, g)
    }

    fn glass(d: &Draw, cx: f32, cy: f32, w: f32, h: f32, f: f32, alpha: f32) {
        d.glass(&Glass { x: cx - w / 2.0, y: cy - h / 2.0, w, h, radius: h / 2.0, blur: f, sheen: 1.0 + 1.6 * f, rim: 1.0 + 0.6 * f, alpha });
    }

    fn mark(d: &mut Draw, path: &'static str, size: i32, cx: f32, cy: f32, k: f32, ink: [f32; 3], alpha: f32) {
        let picture = d.glyph(path, size, 1.8);
        let s = size as f32 * k;
        d.inked_picture(picture, cx - s / 2.0, cy - s / 2.0, s, s, [ink[0], ink[1], ink[2], alpha]);
    }

    fn row_inside(d: &mut Draw, row: &Row, cx: f32, cy: f32, k: f32, alpha: f32) {
        let left = -row.width / 2.0 + 20.0;
        let (mx, glyph_alpha) = (cx + (left + row.mark_box / 2.0) * k, if row.ink == GLYPH_INK { GLYPH_ALPHA } else { 1.0 });
        match row.mark {
            Mark::Glyph(path, size) => Self::mark(d, path, size, mx, cy, k, row.ink, glyph_alpha * alpha),
            Mark::Picture(picture) => {
                let s = 50.0 * k;
                d.inked_picture(picture, mx - s / 2.0, cy - s / 2.0, s, s, [1.0, 1.0, 1.0, GLYPH_ALPHA * alpha]);
            }
        }
        let tx = cx + (left + row.mark_box + 14.0) * k;
        let name_baseline = if row.sub.is_some() { -5.2 } else { 8.0 };
        if let Some(text) = d.text(row.name, &NAME) {
            text.draw_scaled(d, tx, cy + name_baseline * k, k, alpha);
        }
        if let Some(text) = row.sub.as_deref().and_then(|sub| d.text(sub, &SUB)) {
            text.draw_scaled(d, tx, cy + 21.2 * k, k, 0.7 * alpha);
        }
    }

    fn tile_row<'a>(&'a self, cell: Cell, values: &'a Controls) -> Row<'a> {
        let (sub, on) = match cell {
            Cell::Sound => (tvcontrols::sound_label(&values.sound).to_string(), false),
            Cell::Picture => (tvcontrols::picture_label(&values.picture).to_string(), false),
            Cell::Sleep => (tvcontrols::sleep_label(&values.sleep), values.sleep != "off"),
            _ => (clock_label(&self.clock_format).to_string(), self.clock_format != "auto"),
        };
        Row {
            width: 2.0 * UNIT + GAP,
            mark: Mark::Glyph(list_glyph(cell), 40),
            mark_box: 40.0,
            name: cell_label(cell),
            sub: Some(sub),
            ink: if on { ACCENT_ON } else { GLYPH_INK },
        }
    }

    fn draw_cell(&self, d: &mut Draw, cell: Cell, cx: f32, cy: f32, scale: f32, f: f32, alpha: f32, values: &Controls) {
        let (_, _, w, h) = cell_box(cell);
        let k = scale * (1.0 + (FOCUS_LIFT - 1.0) * f);
        Self::glass(d, cx, cy, w * k, h * k, f, alpha);
        match cell {
            Cell::Sound | Cell::Picture | Cell::Clock | Cell::Sleep => Self::row_inside(d, &self.tile_row(cell, values), cx, cy, k, alpha),
            Cell::Backlight => {
                let filled = w * values.backlight as f32 / 100.0;
                let (x, y) = (cx - w * k / 2.0, cy - h * k / 2.0);
                d.p.fill.activate();
                d.p.fill.set4(c"u_box", x, y, w * k, h * k);
                d.p.fill.set1(c"u_radius", h * k / 2.0);
                d.p.fill.set4(c"u_ink", 1.0, 1.0, 1.0, (FILL_TINT + (FILL_TINT_FOCUSED - FILL_TINT) * f) * alpha);
                d.gfx.quad(&d.p.fill, x, y, filled * k, h * k);
                let left = -w / 2.0 + 22.0;
                Self::mark(d, SUN, 34, cx + (left + 17.0) * k, cy, k, GLYPH_INK, GLYPH_ALPHA * alpha);
                if let Some(text) = d.text(cell_label(cell), &NAME) {
                    text.draw_scaled(d, cx + (left + 34.0 + 12.0) * k, cy + 8.0 * k, k, alpha);
                }
                if let Some(text) = d.text(&values.backlight.to_string(), &NUMBER) {
                    text.draw_scaled(d, cx + (w / 2.0 - 26.0) * k - text.advance * k, cy + 8.0 * k, k, alpha);
                }
            }
            _ => {
                let on = cell == Cell::Eye && values.eye_comfort;
                Self::mark(d, circle_glyph(cell), 44, cx, cy, k, if on { ACCENT_ON } else { GLYPH_INK }, if on { alpha } else { GLYPH_ALPHA * alpha });
            }
        }
    }

    fn caption(&self, values: &Controls) -> Option<String> {
        if self.menu.is_some() {
            return None;
        }
        match cell_at(self.row, self.x).0 {
            Cell::Eye => Some(format!("Eye Comfort: {}", if values.eye_comfort { "On" } else { "Off" })),
            cell @ (Cell::Home | Cell::Settings | Cell::Art | Cell::ScreenOff) => Some(cell_label(cell).to_string()),
            _ => None,
        }
    }

    fn draw_controls(&mut self, d: &mut Draw, now: u32) {
        let values = tvcontrols::values();
        let (grown, grow_alpha) = self.grown(CONTROLS_BUTTON, now);
        let (left, top) = (DESIGN_W - RIGHT_EDGE - CONTROLS_W, PANEL_TOP);
        let menu = self.menu.as_ref().map(|m| (m.cell, m.index, m.lifts.iter().map(|l| l.at(now)).collect::<Vec<f32>>()));
        let morph = self.morph_t(now);
        let geo = menu.as_ref().map(|(cell, _, _)| geometry(*cell, options(*cell).len()));
        let (grid_scale, grid_alpha) = match (morph, menu.is_some()) {
            (Some((true, t)), _) => (1.0 - GRID_SHRINK * ease_out(ramp(t, 0.0, 0.4)), 1.0 - ramp(t, 0.08, 0.4)),
            (Some((false, t)), _) => (1.0 - GRID_SHRINK + GRID_SHRINK * ease_out(ramp(t, 0.2, 1.0)), ramp(t, 0.25, 0.85)),
            (None, true) => (1.0, 0.0),
            (None, false) => (1.0, 1.0),
        };
        let shrink = match geo {
            Some(g) => Scale { ox: left + g.x + g.w / 2.0, oy: top + g.y + g.h / 2.0, s: grid_scale },
            None => Scale::none(),
        };
        let place = |x: f32, y: f32| {
            let (x, y) = shrink.at(x, y);
            grown.at(x, y)
        };
        let outer = grid_scale * grown.s;
        if grid_alpha > 0.0 {
            let hidden = match (morph, &menu) {
                (Some((true, _)), Some((cell, _, _))) => Some(*cell),
                _ => None,
            };
            for cell in GRID.iter().flat_map(|r| r.iter().map(|c| c.0)) {
                if Some(cell) == hidden {
                    continue;
                }
                let (x, y, w, h) = cell_box(cell);
                let (cx, cy) = place(left + x + w / 2.0, top + y + h / 2.0);
                let f = self.lifts[cell_index(cell)].at(now);
                self.draw_cell(d, cell, cx, cy, outer, f, grow_alpha * grid_alpha, &values);
            }
            if let Some(caption) = self.caption(&values) {
                if let Some(text) = d.text(&caption, &CAPTION) {
                    let (cx, cy) = place(left + CONTROLS_W / 2.0, top + GRID.len() as f32 * (UNIT + GAP) + CAPTION_H / 2.0);
                    text.draw_scaled(d, cx - text.advance * outer / 2.0, cy + 8.0 * outer, outer, grow_alpha * grid_alpha);
                }
            }
        }
        let (Some((cell, index, lifts)), Some(g)) = (menu, geo) else { return };
        let list = options(cell);
        let chosen = self.chosen(cell, &values);
        let (lx, ly, fit, list_alpha) = match morph {
            Some((open, t)) => {
                let at = if open { opening(&g, t) } else { opening(&g, 1.0 - ease_out(ramp(t, 0.0, 0.7))) };
                let (pw, ph) = (g.w * at.sx, g.h * at.sy);
                if open {
                    let (px, py) = grown.at(left + g.x + at.tx + pw / 2.0, top + g.y + at.ty + ph / 2.0);
                    let (w, h) = (pw * grown.s, ph * grown.s);
                    let rim = 1.0 - ramp(t, 0.0, 0.1);
                    d.glass(&Glass { x: px - w / 2.0, y: py - h / 2.0, w, h, radius: at.r * grown.s, blur: 1.0, sheen: 2.6, rim: 1.6 * rim, alpha: grow_alpha * (1.0 - ramp(t, 0.2, 0.5)) });
                    Self::row_inside(d, &self.tile_row(cell, &values), px, py, FOCUS_LIFT * grown.s, grow_alpha * (1.0 - ramp(t, 0.06, 0.27)));
                }
                let fit = ((pw - 2.0 * PILL_PAD) / g.fw).min((ph - 2.0 * PILL_PAD) / g.fh).clamp(0.0, 1.0);
                let lx = g.x + at.tx + (pw - fit * g.fw) / 2.0;
                let ly = g.y + at.ty + (ph - fit * g.fh) / 2.0;
                (lx, ly, fit, if open { ease_out(ramp(t, 0.08, 0.8)) } else { 1.0 - ramp(t, 0.1, 0.7) })
            }
            None => (0.0, 0.0, 1.0, 1.0),
        };
        if list_alpha <= 0.0 {
            return;
        }
        for (i, (value, label)) in list.iter().enumerate() {
            let f = lifts.get(i).copied().unwrap_or(if i == index { 1.0 } else { 0.0 });
            let (cx, cy) = grown.at(left + lx + fit * CONTROLS_W / 2.0, top + ly + fit * (i as f32 * (OPT_H + OPT_GAP) + OPT_H / 2.0));
            let k = fit * grown.s * (1.0 + (PRESS_LIFT - 1.0) * f);
            Self::glass(d, cx, cy, CONTROLS_W * k, OPT_H * k, f, grow_alpha * list_alpha);
            let row = Row { width: CONTROLS_W, mark: Mark::Glyph(list_glyph(cell), 34), mark_box: 34.0, name: label, sub: None, ink: if *value == chosen { ACCENT_ON } else { GLYPH_INK } };
            Self::row_inside(d, &row, cx, cy, k, grow_alpha * list_alpha);
        }
    }

    fn input_icon(d: &mut Draw, name: &str) -> Tex {
        let key = format!("input:{name}");
        if let Some(picture) = d.pictures.find(&key) {
            return picture;
        }
        let loaded = decoded(&format!("{}/icons/inputs/{name}", d.app_dir)).or_else(|| decoded(&format!("{}/icons/inputs/{STAND_IN_ICON}", d.app_dir)));
        let Some(image) = loaded else { return Tex::default() };
        let picture = d.gfx.upload_rgba(&image.rgba, image.w, image.h);
        d.keep(&key, picture);
        picture
    }

    fn draw_inputs(&mut self, d: &mut Draw, now: u32) {
        let Some(shown) = self.inputs.lock().ok().and_then(|l| l.clone()) else { return };
        let (grown, grow_alpha) = self.grown(INPUTS_BUTTON, now);
        let (left, top) = (DESIGN_W - RIGHT_EDGE - INPUTS_W, PANEL_TOP);
        if shown.is_empty() {
            if let Some(text) = d.text("No inputs selected: add some in Settings > Sources", &CAPTION) {
                let (cx, cy) = grown.at(left + INPUTS_W / 2.0, top + CAPTION_H / 2.0);
                text.draw_scaled(d, cx - text.advance * grown.s / 2.0, cy + 8.0 * grown.s, grown.s, grow_alpha);
            }
            return;
        }
        for (i, input) in shown.iter().enumerate() {
            let f = self.input_lifts.get(i).map_or(if i == self.input_index { 1.0 } else { 0.0 }, |l| l.at(now));
            let (cx, cy) = grown.at(left + INPUTS_W / 2.0, top + i as f32 * (OPT_H + OPT_GAP) + OPT_H / 2.0);
            let k = grown.s * (1.0 + (PRESS_LIFT - 1.0) * f);
            Self::glass(d, cx, cy, INPUTS_W * k, OPT_H * k, f, grow_alpha);
            let icon = Self::input_icon(d, &input.icon);
            let row = Row { width: INPUTS_W, mark: Mark::Picture(icon), mark_box: 40.0, name: &input.label, sub: None, ink: GLYPH_INK };
            Self::row_inside(d, &row, cx, cy, k, grow_alpha);
        }
    }

    pub fn draw(&mut self, d: &mut Draw, now: u32) {
        self.settle(now);
        self.lift_focused(now, d.look.focus_ms);
        match self.kind {
            Kind::Controls => self.draw_controls(d, now),
            Kind::Inputs(_) => self.draw_inputs(d, now),
        }
        self.drawn_at_rest = !self.busy(now);
    }

    fn busy(&self, now: u32) -> bool {
        self.grow.busy(now)
            || self.morph.is_some_and(|m| ((now.wrapping_sub(m.start) as i32) as f32) < m.ms)
            || self.lifts.iter().any(|l| l.busy(now))
            || self.input_lifts.iter().any(|l| l.busy(now))
            || self.menu.as_ref().is_some_and(|m| m.lifts.iter().any(|l| l.busy(now)))
    }

    pub fn moving(&self, now: u32) -> bool {
        self.busy(now) || !self.drawn_at_rest
    }

    pub fn where_now(&self) -> Value {
        match &self.kind {
            Kind::Controls => {
                let focused = match &self.menu {
                    Some(menu) => {
                        let label = options(menu.cell).get(menu.index).map(|o| o.1.clone()).unwrap_or_default();
                        json!({"part": "option", "label": label, "index": menu.index, "in": cell_label(menu.cell)})
                    }
                    None => json!({"part": "control", "label": cell_label(cell_at(self.row, self.x).0), "row": self.row}),
                };
                json!({"panel": "controls", "focused": [focused]})
            }
            Kind::Inputs(_) => {
                let shown = self.inputs.lock().ok().and_then(|l| l.clone()).unwrap_or_default();
                let focused: Vec<Value> = shown.get(self.input_index).map(|i| json!({"part": "input", "label": i.label, "id": i.app_id, "index": self.input_index})).into_iter().collect();
                json!({"panel": "inputs", "focused": focused})
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel() -> ControlCenter {
        ControlCenter::fresh(Kind::Controls, "auto", 0)
    }

    fn press(panel: &mut ControlCenter, keys: &[Key], now: u32) -> Outcome {
        keys.iter().fold(Outcome::Stay, |_, key| panel.react(*key, now, 160.0))
    }

    #[test]
    fn every_sleep_time_offered_is_one_the_tv_takes_and_the_list_is_no_longer_than_picture_modes() {
        assert!(SLEEP_CHOICES.iter().all(|m| crate::root::SLEEP_MINUTES.contains(m)));
        assert!(options(Cell::Sleep).len() <= options(Cell::Picture).len());
    }

    #[test]
    fn down_keeps_the_column_so_all_settings_leads_to_eye_comfort_and_screen_off_to_the_backlight() {
        let mut p = panel();
        press(&mut p, &[Key::Down, Key::Down, Key::Right, Key::Right, Key::Right, Key::Right], 0);
        assert_eq!(cell_at(p.row, p.x).0, Cell::Settings);
        press(&mut p, &[Key::Down], 0);
        assert_eq!(cell_at(p.row, p.x).0, Cell::Eye);
        press(&mut p, &[Key::Up, Key::Left], 0);
        assert_eq!(cell_at(p.row, p.x).0, Cell::ScreenOff);
        press(&mut p, &[Key::Down], 0);
        assert_eq!(cell_at(p.row, p.x).0, Cell::Backlight);
    }

    #[test]
    fn up_past_the_top_closes_and_down_twice_at_the_bottom_goes_to_the_glass_bar() {
        assert_eq!(press(&mut panel(), &[Key::Up], 0), Outcome::Close);
        let mut p = panel();
        assert_eq!(press(&mut p, &[Key::Down, Key::Down, Key::Down, Key::Down], 5000), Outcome::Stay);
        assert_eq!(press(&mut p, &[Key::Down], 7000), Outcome::Stay);
        assert_eq!(press(&mut p, &[Key::Down], 7400), Outcome::CloseDown);
    }

    #[test]
    fn a_clock_format_picked_from_its_list_comes_back_as_the_setting_once_the_list_has_opened() {
        let mut p = panel();
        press(&mut p, &[Key::Down, Key::Ok], 0);
        assert_eq!(press(&mut p, &[Key::Down], 10), Outcome::Stay);
        assert_eq!(p.menu.as_ref().map(|m| m.index), Some(0));
        assert_eq!(press(&mut p, &[Key::Down, Key::Ok], 1000), Outcome::Setting("clockFormat", json!("12h")));
        assert_eq!(p.clock_format, "12h");
        press(&mut p, &[Key::Back], 2000);
        assert!(p.menu.is_none());
    }

    #[test]
    fn inputs_open_the_one_chosen_and_up_past_the_first_closes() {
        let mut p = ControlCenter::fresh(Kind::Inputs(Vec::new()), "auto", 0);
        *p.inputs.lock().unwrap() = Some(vec![
            Input { app_id: "com.webos.app.livetv".into(), label: "Live TV".into(), icon: "tv.png".into() },
            Input { app_id: "com.webos.app.hdmi1".into(), label: "Chromecast".into(), icon: "HDMI_1.png".into() },
        ]);
        assert_eq!(press(&mut p, &[Key::Down, Key::Ok], 0), Outcome::Open("com.webos.app.hdmi1".into()));
        assert_eq!(press(&mut p, &[Key::Down], 5000), Outcome::Stay);
        assert_eq!(press(&mut p, &[Key::Down], 5100), Outcome::CloseDown);
        assert_eq!(press(&mut p, &[Key::Up, Key::Up], 9000), Outcome::Close);
    }
}
