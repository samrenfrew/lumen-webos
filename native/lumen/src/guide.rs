use crate::anim::{Anim, EASE_OUT, EASE_SCROLL};
use crate::card::{Card, GRID_CARD, GUIDE_CARD};
use crate::clock::clock_text;
use crate::draw::{Draw, Plate, Style};
use crate::ffi::*;
use crate::gfx::{DESIGN_H, DESIGN_W};
use crate::home::Key;
use crate::live::{Live, ScheduleState};
use crate::livetv::{self, Channel, Programme};
use crate::look::Look;
use std::collections::HashMap;

pub const GUIDE_APP_ID: &str = "com.samrenfrew.lumen.native.guide";
const EDGE: f32 = 125.0;
const ROWS_PREVIEW: [f32; 4] = [DESIGN_W - 64.0 - 448.0, DESIGN_H - 56.0 - 252.0, 448.0, 252.0];
const FULL_PREVIEW: [f32; 4] = [DESIGN_W - EDGE - 400.0, 150.0, 400.0, 225.0];
const ROWS_PREVIEW_CARD: Card = Card { w: 448, h: 252, radius: 24.0, logo_box: (112.0, 63.0, 224.0, 126.0), name_px: 36, name_box: (22.0, 0.0, 404.0, 252.0) };
const FULL_PREVIEW_CARD: Card = Card { w: 400, h: 225, radius: 24.0, logo_box: (100.0, 56.0, 200.0, 113.0), name_px: 34, name_box: (20.0, 0.0, 360.0, 225.0) };
const ROW_STEP: f32 = 132.0;
const ROW_H: f32 = 120.0;
const IN_VIEW: usize = 6;
const FOCUS_AT: usize = 2;
const OVER_IN_VIEW: usize = 7;
const OVER_FOCUS_AT: usize = 3;
const TYPED_MS: u32 = 2000;
const TYPED_MAX: usize = 4;
const FG_STEP: f32 = 80.0;
const FG_ROW_H: f32 = 70.0;
const FG_IN_VIEW: usize = 7;
const FG_FOCUS_AT: usize = 3;
const FG_TOP: f32 = 450.0;
const FG_LINE: f32 = EDGE + 372.0;
const HALF_HOUR: u64 = 1800;
const SPAN: u64 = 4 * HALF_HOUR;
const GRID_W: f32 = 1298.0;
const RAIL_W: f32 = 340.0;
const RAIL_ITEM: f32 = 64.0;
const RAIL_STEP: f32 = 76.0;
const SLIDE_MS: f32 = 320.0;
const LIFT_PRESS: f32 = 1.03;
const OVER_PANEL_W: f32 = 880.0;
const OVER_FADE_W: f32 = 280.0;
const OVER_TINT: f32 = 0.65;
const OVER_FULL_TINT: f32 = 0.75;
const OVER_TINT_RGB: [f32; 3] = [8.0 / 255.0, 10.0 / 255.0, 18.0 / 255.0];

const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
const FOCUS_INK: [f32; 3] = [0.067, 0.067, 0.067];
const LIST: &str = "M8.5 6.5h12M8.5 12h12M8.5 17.5h12M4 6.5h.01M4 12h.01M4 17.5h.01";
const NEXT: &str = "M4 12h15M13.5 6.5 19 12l-5.5 5.5";
const STARTED_EARLIER: &str = "M15 5.5 8.5 12l6.5 6.5";
const STARTED_EARLIER_W: f32 = 20.0;
const GRID: &str = "M4 4h6.5v6.5H4zM13.5 4H20v6.5h-6.5zM4 13.5h6.5V20H4zM13.5 13.5H20V20h-6.5z";

const ROW: Plate = Plate { h: ROW_H, radius: 30.0, fill: [1.0, 1.0, 1.0, 0.09], rim: 1.0, shadow: [0.0; 3], ring: 0.0 };
const ROW_ON: Plate = Plate { fill: [1.0, 1.0, 1.0, 0.2], ..ROW };
const ROW_FOCUS: Plate = Plate { fill: [1.0, 1.0, 1.0, 1.0], rim: 0.0, shadow: [18.0, 36.0, 0.32], ..ROW };
const CELL: Plate = Plate { h: FG_ROW_H, radius: 18.0, fill: [1.0, 1.0, 1.0, 0.09], rim: 1.0, shadow: [0.0; 3], ring: 0.0 };
const CELL_ON: Plate = Plate { fill: [1.0, 1.0, 1.0, 0.2], ..CELL };
const CELL_FOCUS: Plate = Plate { fill: [1.0, 1.0, 1.0, 1.0], rim: 0.0, shadow: [14.0, 30.0, 0.32], ..CELL };
const METER: Plate = Plate { h: 5.0, radius: 2.5, fill: [1.0, 1.0, 1.0, 0.3], rim: 0.0, shadow: [0.0; 3], ring: 0.0 };
const METER_FILL: Plate = Plate { fill: [1.0, 1.0, 1.0, 1.0], ..METER };
const METER_FOCUS: Plate = Plate { fill: [0.0, 0.0, 0.0, 0.14], ..METER };
const METER_FOCUS_FILL: Plate = Plate { fill: [FOCUS_INK[0], FOCUS_INK[1], FOCUS_INK[2], 1.0], ..METER };
const PH: Plate = Plate { h: 28.0, radius: 8.0, fill: [1.0, 1.0, 1.0, 0.07], rim: 0.0, shadow: [0.0; 3], ring: 0.0 };
const PH_FOCUS: Plate = Plate { fill: [0.0, 0.0, 0.0, 0.12], ..PH };
const RAIL_ON: Plate = Plate { h: RAIL_ITEM, radius: 32.0, fill: [1.0, 1.0, 1.0, 0.09], rim: 1.0, shadow: [0.0; 3], ring: 0.0 };
const RAIL_FOCUS: Plate = Plate { fill: [1.0, 1.0, 1.0, 1.0], rim: 0.0, ..RAIL_ON };
const TYPED: Plate = Plate { h: 84.0, radius: 42.0, fill: [1.0, 1.0, 1.0, 0.16], rim: 1.3, shadow: [12.0, 28.0, 0.3], ring: 0.0 };
const FLAG: Plate = Plate { h: 26.0, radius: 8.0, fill: [1.0, 1.0, 1.0, 0.85], rim: 0.0, shadow: [0.0; 3], ring: 2.0 };
const NOW_LINE: Plate = Plate { h: 12.0, radius: 6.0, fill: [1.0, 1.0, 1.0, 1.0], rim: 0.0, shadow: [0.0; 3], ring: 0.0 };

#[derive(Clone, Copy, PartialEq, Debug)]
enum View {
    NowNext,
    Now,
    Next,
    Full,
}

const MAIN_RAIL: [(View, &str, &str); 2] = [(View::NowNext, "Now and Next", LIST), (View::Full, "Full guide", GRID)];
const OVER_RAIL: [(View, &str, &str); 3] = [(View::Now, "Now", LIST), (View::Next, "Next", NEXT), (View::Full, "Full guide", GRID)];

#[derive(Clone, PartialEq, Debug)]
pub enum Outcome {
    Stay,
    Close,
    Open(String),
    Mode(bool),
}

pub struct Guide {
    over_live_tv: bool,
    view: View,
    started_full: bool,
    at: usize,
    moved: bool,
    on_rail: bool,
    rail_at: usize,
    rail: Anim,
    typed: String,
    typed_until: u32,
    scroll: Anim,
    lift: Anim,
    lifted: usize,
    dropping: Anim,
    dropped: usize,
    win: u64,
    anchor: u64,
    wrapped: HashMap<String, (String, String)>,
    pub preview: Option<bool>,
    pub preview_loading: bool,
}

fn style(px: i32, weight: &'static str, spacing_em: f32, ink: [f32; 3]) -> Style {
    Style { px, weight, spacing_em, feature: "", ink, shadows: &[] }
}

fn half_hour(t: u64) -> u64 {
    t - t % HALF_HOUR
}

pub fn typed_next(typed: &str, digit: usize) -> String {
    let kept = if typed.chars().count() >= TYPED_MAX { "" } else { typed };
    format!("{kept}{digit}")
}

fn window_top(count: usize, at: usize, in_view: usize, focus_at: usize) -> usize {
    at.saturating_sub(focus_at).min(count.saturating_sub(in_view))
}

fn rows_in_view(scroll: f32, in_view: usize, count: usize) -> std::ops::Range<usize> {
    let first = (scroll.floor().max(0.0) as usize).min(count);
    first..(first + in_view + 1).min(count)
}

fn in_window(list: &[Programme], win: u64, win_end: u64) -> Vec<&Programme> {
    list.iter().filter(|p| p.end > win && p.start < win_end).collect()
}

fn starts_in(p: &Programme, t: u64) -> String {
    let min = ((p.start as i64 - t as i64) as f64 / 60.0).ceil().max(1.0) as i64;
    match (min / 60, min % 60) {
        (0, m) => format!("in {m} min"),
        (h, 0) => format!("in {h} hr"),
        (h, m) => format!("in {h} hr {m} min"),
    }
}

fn pick(list: &[Programme], at: u64, win_end: u64) -> Option<usize> {
    list.iter().position(|p| p.start <= at && at < p.end).or_else(|| list.iter().position(|p| p.start > at && p.start < win_end))
}

impl Guide {
    pub fn new(full: bool, over_live_tv: bool, t: u64) -> Guide {
        let view = match (full, over_live_tv) {
            (true, _) => View::Full,
            (false, true) => View::Now,
            (false, false) => View::NowNext,
        };
        Guide {
            over_live_tv,
            view,
            started_full: full,
            at: 0,
            moved: false,
            on_rail: false,
            rail_at: 0,
            rail: Anim::still(0.0),
            typed: String::new(),
            typed_until: 0,
            scroll: Anim::still(0.0),
            lift: Anim::still(1.0),
            lifted: 0,
            dropping: Anim::still(0.0),
            dropped: 0,
            win: half_hour(t),
            anchor: t,
            wrapped: HashMap::new(),
            preview: None,
            preview_loading: false,
        }
    }

    pub fn over_live_tv(&self) -> bool {
        self.over_live_tv
    }

    pub fn preview_box(&self, live: &Live) -> Option<(String, [f32; 4])> {
        if self.over_live_tv {
            return None;
        }
        let channel = live.channels.get(self.at)?;
        Some((channel.id.clone(), if self.view == View::Full { FULL_PREVIEW } else { ROWS_PREVIEW }))
    }

    fn draw_preview(&self, d: &mut Draw, live: &Live, now: u32) {
        let (Some(showing), Some((_, [x, y, w, h]))) = (self.preview, self.preview_box(live)) else { return };
        let Some(c) = live.channels.get(self.at) else { return };
        let card = if self.view == View::Full { &FULL_PREVIEW_CARD } else { &ROWS_PREVIEW_CARD };
        let picture = Self::card(d, c, &live.logos, card);
        d.picture(picture, x, y, w, h, 1.0);
        if showing {
            d.hole(x, y, w, h, card.radius);
        } else if self.preview_loading {
            d.spinner(x + 18.0, y + 18.0, 22.0, 3.0, now);
        }
    }

    pub fn at(&self) -> usize {
        self.at
    }

    fn rail_items(&self) -> &'static [(View, &'static str, &'static str)] {
        if self.over_live_tv {
            &OVER_RAIL
        } else {
            &MAIN_RAIL
        }
    }

    fn list_window(&self) -> (usize, usize) {
        match (self.view, self.over_live_tv) {
            (View::Full, _) => (FG_IN_VIEW, FG_FOCUS_AT),
            (_, true) => (OVER_IN_VIEW, OVER_FOCUS_AT),
            (_, false) => (IN_VIEW, FOCUS_AT),
        }
    }

    fn top_row(&self, count: usize) -> usize {
        let (in_view, focus_at) = self.list_window();
        window_top(count, self.at, in_view, focus_at)
    }

    pub fn tv_channel(&mut self, index: usize, count: usize) {
        if self.moved || index >= count {
            return;
        }
        self.at = index;
        self.lifted = index;
        self.scroll.set(self.top_row(count) as f32);
    }

    fn set_at(&mut self, at: usize, count: usize, look: &Look, now: u32) {
        let at = at.min(count.saturating_sub(1));
        self.moved = true;
        if at != self.at {
            self.dropping = Anim::still(self.lift.at(now));
            self.dropping.go(0.0, look.focus_ms, &EASE_OUT, now);
            self.dropped = self.lifted;
            self.lift = Anim::still(0.0);
            self.lift.go(1.0, look.focus_ms, &EASE_OUT, now);
            self.lifted = at;
            self.at = at;
        }
        self.scroll.go(self.top_row(count) as f32, look.scroll_ms, &EASE_SCROLL, now);
    }

    fn open_rail(&mut self, now: u32) {
        self.rail_at = self.rail_items().iter().position(|r| r.0 == self.view).unwrap_or(0);
        self.on_rail = true;
        self.rail.go(1.0, SLIDE_MS, &EASE_SCROLL, now);
    }

    fn close_rail(&mut self, now: u32) {
        self.on_rail = false;
        self.rail.go(0.0, SLIDE_MS, &EASE_SCROLL, now);
    }

    fn show_view(&mut self, view: View, count: usize, t: u64) {
        self.view = view;
        self.win = half_hour(t);
        self.anchor = t;
        self.scroll.set(self.top_row(count) as f32);
    }

    pub fn key(&mut self, look: &Look, key: Key, now: u32, live: &Live, t: u64) -> Outcome {
        let count = live.channels.len();
        if let Key::Digit(digit) = key {
            if self.over_live_tv {
                return Outcome::Stay;
            }
            self.typed = typed_next(&self.typed, digit);
            self.typed_until = now.wrapping_add(TYPED_MS);
            if let Some(found) = livetv::find_channel(&live.channels, &self.typed) {
                self.set_at(found, count, look, now);
            }
            return Outcome::Stay;
        }
        if self.on_rail {
            let items = self.rail_items();
            match key {
                Key::Back => return Outcome::Close,
                Key::Up => self.rail_at = self.rail_at.saturating_sub(1),
                Key::Down => self.rail_at = (self.rail_at + 1).min(items.len() - 1),
                Key::Right => self.close_rail(now),
                Key::Ok => {
                    let view = items[self.rail_at].0;
                    self.close_rail(now);
                    if view != self.view {
                        self.show_view(view, count, t);
                        if !self.over_live_tv {
                            return Outcome::Mode(view == View::Full);
                        }
                    }
                }
                _ => {}
            }
            return Outcome::Stay;
        }
        if self.view == View::Full {
            return self.full_key(look, key, now, live, t);
        }
        match key {
            Key::Back => return Outcome::Close,
            Key::Left => self.open_rail(now),
            Key::Up => self.set_at(self.at.saturating_sub(1), count, look, now),
            Key::Down => self.set_at(self.at + 1, count, look, now),
            Key::Ok => {
                if let Some(c) = live.channels.get(self.at) {
                    return Outcome::Open(c.id.clone());
                }
            }
            _ => {}
        }
        Outcome::Stay
    }

    fn full_key(&mut self, look: &Look, key: Key, now: u32, live: &Live, t: u64) -> Outcome {
        let count = live.channels.len();
        let base = half_hour(t);
        let last_win = base.max(base + livetv::SCHEDULE_HOURS * 3600 - SPAN);
        let win = self.win.clamp(base, last_win);
        let win_end = win + SPAN;
        let anchor = self.anchor.max(t);
        let channel = live.channels.get(self.at);
        let list: &[Programme] = channel.and_then(|c| live.listings.get(&c.signal)).map_or(&[], Vec::as_slice);
        let focused = pick(list, anchor, win_end);
        match key {
            Key::Back if self.over_live_tv && !self.started_full => self.show_view(View::Now, count, t),
            Key::Back => return Outcome::Close,
            Key::Up => self.set_at(self.at.saturating_sub(1), count, look, now),
            Key::Down => self.set_at(self.at + 1, count, look, now),
            Key::Ok => {
                let on_now = focused.is_some_and(|i| list[i].start <= t && t < list[i].end);
                if let Some(c) = channel.filter(|_| !self.over_live_tv || on_now) {
                    return Outcome::Open(c.id.clone());
                }
            }
            Key::Left | Key::Right => {
                let right = key == Key::Right;
                let p = match (right, focused) {
                    (true, Some(i)) => list.get(i + 1),
                    (true, None) => list.iter().find(|p| p.start > anchor),
                    (false, Some(i)) => i.checked_sub(1).and_then(|i| list.get(i)),
                    (false, None) => list.iter().filter(|p| p.end <= anchor).last(),
                };
                match p.filter(|p| p.end > t && p.start < last_win + SPAN) {
                    Some(p) => {
                        self.anchor = p.start.max(t);
                        if p.start >= win_end - HALF_HOUR || p.start < win {
                            let back = if p.start >= win_end - HALF_HOUR { HALF_HOUR } else { 0 };
                            self.win = (half_hour(p.start).saturating_sub(back)).clamp(base, last_win);
                        }
                    }
                    None if right && win < last_win => {
                        self.win = win + HALF_HOUR;
                        self.anchor = anchor.max(win + 2 * HALF_HOUR);
                    }
                    None if !right && win > base => {
                        self.win = win - HALF_HOUR;
                        self.anchor = t.max(win - HALF_HOUR);
                    }
                    None if !right && !self.over_live_tv => self.open_rail(now),
                    None => {}
                }
            }
            Key::Digit(_) => {}
        }
        Outcome::Stay
    }

    pub fn wake_in(&mut self, now: u32) -> Option<u32> {
        if self.typed.is_empty() {
            return None;
        }
        let left = self.typed_until.wrapping_sub(now) as i32;
        if left <= 0 {
            self.typed.clear();
            return Some(0);
        }
        Some(left as u32)
    }

    pub fn moving(&self, now: u32) -> bool {
        self.scroll.busy(now) || self.lift.busy(now) || self.dropping.busy(now) || self.rail.busy(now) || self.preview_loading
    }

    #[cfg(feature = "dev")]
    pub fn where_now(&self, live: &Live) -> serde_json::Value {
        let screen = if self.over_live_tv { "lumen guide" } else { "guide" };
        let row = match (self.view, self.over_live_tv) {
            (View::Full, _) => "fg__prog",
            (_, true) => "lguide__row",
            (_, false) => "guide__row",
        };
        let view = match self.view {
            View::NowNext => "now and next",
            View::Now => "now",
            View::Next => "next",
            View::Full => "full guide",
        };
        let mut focused = Vec::new();
        if self.on_rail {
            focused.push(serde_json::json!({"part": "rail__item", "label": self.rail_items()[self.rail_at].1, "index": self.rail_at}));
        } else if let Some(c) = live.channels.get(self.at) {
            focused.push(serde_json::json!({"part": row, "label": format!("{} · {}", c.number, c.name), "id": c.id, "index": self.at}));
        }
        serde_json::json!({"screen": screen, "view": view, "typed": self.typed, "focused": focused})
    }

    fn lift_of(&self, i: usize, now: u32) -> f32 {
        if i == self.lifted {
            self.lift.at(now)
        } else if i == self.dropped {
            self.dropping.at(now)
        } else {
            0.0
        }
    }

    fn card(d: &mut Draw, c: &Channel, logos: &HashMap<String, String>, card: &Card) -> crate::gfx::Tex {
        let logo = c.logo_path.as_ref().and_then(|p| logos.get(p)).cloned();
        let key = format!("card:{}:{}:{}", card.w, c.id, logo.as_deref().unwrap_or(""));
        if let Some(picture) = d.pictures.find(&key) {
            return picture;
        }
        let made = crate::card::channel_card(&d.fonts, &c.name, logo.as_deref(), card);
        let picture = d.gfx.upload_rgba(&made.rgba, made.w, made.h);
        d.keep(&key, picture);
        picture
    }

    pub fn draw(&mut self, d: &mut Draw, now: u32, live: &Live, t: u64, twenty_four_hour: bool) {
        if !self.over_live_tv {
            d.copy_whole(d.veil_backdrop, 1.0, 1.0, 0.0, 0.0);
        }
        let (alpha_from, alpha_kept) = if self.over_live_tv { (GL_ONE, GL_ONE_MINUS_SRC_ALPHA) } else { (GL_ZERO, GL_ONE) };
        unsafe {
            glEnable(GL_BLEND);
            glBlendFuncSeparate(GL_ONE, GL_ONE_MINUS_SRC_ALPHA, alpha_from, alpha_kept);
        }
        let clock = |at: u64| clock_text(at, twenty_four_hour);
        match (self.view, self.over_live_tv) {
            (View::Full, over) => {
                if over {
                    let tint = Self::tint(d, "full", DESIGN_W as usize, 0.0, OVER_FULL_TINT);
                    d.picture(tint, 0.0, 0.0, DESIGN_W, DESIGN_H, 1.0);
                }
                if let Some(head) = d.text(if over { "Full guide" } else { "Guide" }, &style(56, "700", -0.02, WHITE)) {
                    head.draw(d, EDGE, 112.0, 1.0);
                }
                self.draw_full(d, now, live, t, &clock);
            }
            (_, false) => {
                if let Some(head) = d.text("Guide", &style(56, "700", -0.02, WHITE)) {
                    head.draw(d, EDGE, 112.0, 1.0);
                }
                let cols = style(22, "500", 0.0, WHITE);
                if let Some(text) = d.text(&format!("Now · {}", clock(t)), &cols) {
                    text.draw(d, EDGE + 330.0, 181.0, 0.75);
                }
                if let Some(text) = d.text("Next", &cols) {
                    text.draw(d, EDGE + 1160.0, 181.0, 0.75);
                }
                self.draw_rows(d, now, live, t, &clock);
            }
            (_, true) => {
                let tint = Self::tint(d, "panel", OVER_PANEL_W as usize, OVER_FADE_W, OVER_TINT);
                d.picture(tint, 0.0, 0.0, OVER_PANEL_W + OVER_FADE_W, DESIGN_H, 1.0);
                let head = if self.view == View::Next { "Next".to_string() } else { format!("Now · {}", clock(t)) };
                if let Some(text) = d.text(&head, &style(44, "700", -0.02, WHITE)) {
                    text.draw(d, 150.0, 98.0, 1.0);
                }
                self.draw_rows(d, now, live, t, &clock);
            }
        }
        self.draw_preview(d, live, now);
        if !(self.over_live_tv && self.view == View::Full) {
            self.draw_rail(d, now);
        }
        self.draw_typed(d, live);
    }

    fn tint(d: &mut Draw, name: &str, solid: usize, fade: f32, strength: f32) -> crate::gfx::Tex {
        let key = format!("guide:tint:{name}");
        if let Some(picture) = d.pictures.find(&key) {
            return picture;
        }
        let w = solid + fade as usize;
        let mut rgba = vec![0u8; w * 4];
        for x in 0..w {
            let a = if x < solid { strength } else { strength * (1.0 - (x - solid) as f32 / fade).powf(2.2) };
            let px = [OVER_TINT_RGB[0] * a, OVER_TINT_RGB[1] * a, OVER_TINT_RGB[2] * a, a].map(|c| (c * 255.0) as u8);
            rgba[x * 4..][..4].copy_from_slice(&px);
        }
        let picture = d.gfx.upload_rgba(&rgba, w as i32, 1);
        d.keep(&key, picture);
        picture
    }

    fn draw_rows(&mut self, d: &mut Draw, now: u32, live: &Live, t: u64, clock: &dyn Fn(u64) -> String) {
        let over = self.over_live_tv;
        let (view_x, view_y, view_w, view_h) = if over { (96.0, 136.0, 784.0, 930.0) } else { (EDGE - 30.0, 192.0, DESIGN_W - 2.0 * EDGE + 60.0, 804.0) };
        let (row_x, row_w) = if over { (120.0, 724.0) } else { (EDGE, DESIGN_W - 2.0 * EDGE) };
        if live.channels.is_empty() {
            if let Some(text) = d.text("No channels yet. Open Live TV once, then come back.", &style(26, "400", 0.0, WHITE)) {
                text.draw(d, view_x + 30.0, view_y + 64.0, 0.8);
            }
            return;
        }
        let (in_view, _) = self.list_window();
        let top = self.scroll.at(now);
        let rows = rows_in_view(top, in_view, live.channels.len());
        d.clip(Some((view_x, view_y, view_w, view_h)));
        let order: Vec<usize> = rows.clone().filter(|&i| i != self.at).chain(rows.filter(|&i| i == self.at)).collect();
        for i in order {
            let y = view_y + 12.0 + (i as f32 - top) * ROW_STEP;
            self.draw_row(d, now, live, t, clock, i, (row_x, y, row_w));
        }
        d.clip(None);
    }

    fn draw_row(&mut self, d: &mut Draw, now: u32, live: &Live, t: u64, clock: &dyn Fn(u64) -> String, i: usize, (x, y, w): (f32, f32, f32)) {
        let over = self.over_live_tv;
        let c = &live.channels[i];
        let focused = i == self.at && !self.on_rail;
        let lift = if focused { self.lift_of(i, now) } else if i == self.dropped { self.dropping.at(now) } else { 0.0 };
        let s = 1.0 + (LIFT_PRESS - 1.0) * lift;
        let (ox, oy) = (if over { x } else { x + w / 2.0 }, y + ROW_H / 2.0);
        let at_x = |px: f32| ox + (px - ox) * s;
        let at_y = |py: f32| oy + (py - oy) * s;
        let plate = if over && i == self.at && self.on_rail { ROW_ON } else { ROW };
        d.plate(&plate, at_x(x), at_y(y), w * s, ROW_H * s, 1.0);
        if focused {
            d.plate(&ROW_FOCUS, at_x(x), at_y(y), w * s, ROW_H * s, 1.0);
        }
        let ink = if focused { FOCUS_INK } else { WHITE };
        let pad = if over { 16.0 } else { 36.0 };
        if let Some(num) = d.text(&c.number, &style(32, "600", 0.0, ink)) {
            num.draw_scaled(d, at_x(x + pad + 32.0 - num.advance / 2.0), at_y(y + 71.0), s, 1.0);
        }
        let card = Self::card(d, c, &live.logos, &GUIDE_CARD);
        let card_x = x + pad + 64.0 + 16.0;
        d.picture(card, at_x(card_x), at_y(y + 20.0), 132.0 * s, 80.0 * s, 1.0);
        let list = live.listings.get(&c.signal).map(Vec::as_slice);
        let nn = livetv::now_next_at(list, t);
        let waiting = nn.now.is_none() && nn.next.is_none() && live.pending.contains(&c.signal);
        let off_alpha = if nn.now.is_some() || (over && self.view == View::Next && nn.next.is_some()) { 1.0 } else if focused { 0.8 } else { 0.55 };
        let block_x = card_x + 132.0 + if over { 28.0 } else { 82.0 };
        let block_w = if over { w - 28.0 - (block_x - x) } else { 750.0 };
        let title_style = style(32, "600", -0.015, ink);
        let meta = style(22, "500", 0.0, ink);
        if waiting {
            let ph = if focused { PH_FOCUS } else { PH };
            d.plate(&ph, at_x(block_x), at_y(y + 29.0), block_w * 0.7 * s, 28.0 * s, 1.0);
            d.plate(&ph, at_x(block_x), at_y(y + 75.0), 150.0 * s, 18.0 * s, 1.0);
            d.plate(&ph, at_x(block_x + 166.0), at_y(y + 81.5), 220.0 * s, 5.0 * s, 1.0);
            if !over {
                d.plate(&ph, at_x(x + 1160.0), at_y(y + 31.5), 474.0 * 0.6 * s, 22.0 * s, 1.0);
                d.plate(&ph, at_x(x + 1160.0), at_y(y + 70.5), 70.0 * s, 18.0 * s, 1.0);
            }
            return;
        }
        let shown_next = over && self.view == View::Next;
        let title = match (shown_next, nn.now, nn.next) {
            (true, _, Some(next)) => next.title.clone(),
            (true, _, None) => "No guide data".to_string(),
            (false, Some(on), _) => on.title.clone(),
            (false, None, Some(_)) => "Off air".to_string(),
            (false, None, None) => "No guide data".to_string(),
        };
        if let Some(text) = d.text_fit(&title, &title_style, block_w) {
            text.draw_scaled(d, at_x(block_x), at_y(y + 55.0), s, off_alpha);
        }
        let meta_y = y + 90.0;
        let meta_alpha = 0.8 * off_alpha;
        match (shown_next, nn.now, nn.next) {
            (true, _, Some(next)) | (false, None, Some(next)) => {
                if let Some(text) = d.text(&format!("Starts at {}", clock(next.start)), &meta) {
                    text.draw_scaled(d, at_x(block_x), at_y(meta_y), s, meta_alpha);
                }
            }
            (false, Some(on), _) => {
                let mut mx = block_x;
                if let Some(span) = d.text(&format!("{} – {}", clock(on.start), clock(on.end)), &meta) {
                    span.draw_scaled(d, at_x(mx), at_y(meta_y), s, meta_alpha);
                    mx += span.advance + 16.0;
                }
                let (bg, fill) = if focused { (METER_FOCUS, METER_FOCUS_FILL) } else { (METER, METER_FILL) };
                let meter_w = if over { (block_x + block_w - mx - 16.0 - 130.0).clamp(48.0, 220.0) } else { 220.0 };
                d.plate(&bg, at_x(mx), at_y(meta_y - 10.2), meter_w * s, 5.0 * s, meta_alpha);
                d.plate(&fill, at_x(mx), at_y(meta_y - 10.2), (meter_w * livetv::progress(on, t)).max(5.0) * s, 5.0 * s, meta_alpha);
                mx += meter_w + 16.0;
                if i == self.at {
                    if let Some(text) = d.text(&livetv::time_left(on, t), &meta) {
                        text.draw_scaled(d, at_x(mx), at_y(meta_y), s, meta_alpha);
                    }
                }
            }
            _ => {}
        }
        if over {
            return;
        }
        if let Some(next) = nn.next {
            let nx = x + 1160.0;
            let nw = w - 36.0 - 1160.0;
            if let Some(text) = d.text_fit(&next.title, &style(26, "500", -0.015, ink), nw) {
                text.draw_scaled(d, at_x(nx), at_y(y + 52.0), s, off_alpha);
            }
            if let Some(text) = d.text(&clock(next.start), &meta) {
                text.draw_scaled(d, at_x(nx), at_y(y + 86.0), s, 0.8 * off_alpha);
            }
        }
    }

    fn draw_full(&mut self, d: &mut Draw, now: u32, live: &Live, t: u64, clock: &dyn Fn(u64) -> String) {
        let base = half_hour(t);
        let last_win = base.max(base + livetv::SCHEDULE_HOURS * 3600 - SPAN);
        let win = self.win.clamp(base, last_win);
        let win_end = win + SPAN;
        let anchor = self.anchor.max(t);
        let loading = live.state == ScheduleState::Loading;
        let list_of = |c: &Channel| live.listings.get(&c.signal).map_or(&[][..], Vec::as_slice);
        let channel = live.channels.get(self.at);
        let focused = channel.and_then(|c| pick(list_of(c), anchor, win_end).map(|i| &list_of(c)[i]));
        let white = |px, weight| style(px, weight, 0.0, WHITE);
        if let Some(c) = channel {
            if let Some(text) = d.text(&format!("{} · {}", c.number, c.name), &white(26, "500")) {
                text.draw(d, EDGE, 175.0, 0.8);
            }
        }
        match focused {
            None if loading => {
                d.plate(&Plate { h: 50.0, radius: 12.0, ..PH }, EDGE, 201.0, 520.0, 50.0, 1.0);
                d.plate(&PH, EDGE, 269.0, 300.0, 22.0, 1.0);
            }
            None => {
                let says = if live.state == ScheduleState::NowNextOnly { "Only now and next in the guide data" } else { "No guide data" };
                if let Some(text) = d.text_fit(says, &style(56, "700", -0.02, WHITE), 1300.0) {
                    text.draw(d, EDGE, 237.0, 1.0);
                }
            }
            Some(p) => {
                if let Some(text) = d.text_fit(&p.title, &style(56, "700", -0.02, WHITE), 1300.0) {
                    text.draw(d, EDGE, 237.0, 1.0);
                }
                let meta = white(26, "500");
                let mut mx = EDGE;
                if let Some(span) = d.text(&format!("{} – {}", clock(p.start), clock(p.end)), &meta) {
                    span.draw(d, mx, 287.0, 0.9);
                    mx += span.advance;
                }
                if p.start <= t {
                    d.plate(&METER, mx + 16.0, 276.5, 200.0, 5.0, 0.9);
                    d.plate(&METER_FILL, mx + 16.0, 276.5, (200.0 * livetv::progress(p, t)).max(5.0), 5.0, 0.9);
                    mx += 232.0;
                    if let Some(text) = d.text(&livetv::time_left(p, t), &meta) {
                        text.draw(d, mx, 287.0, 0.9);
                        mx += text.advance;
                    }
                } else if let Some(text) = d.text(&starts_in(p, t), &meta) {
                    text.draw(d, mx + 16.0, 287.0, 0.9);
                    mx += 16.0 + text.advance;
                }
                for (on, flag) in [(p.subs, "S"), (p.ad, "AD")] {
                    if !on {
                        continue;
                    }
                    if let Some(text) = d.text(flag, &white(18, "600")) {
                        let fw = text.advance + 22.0;
                        d.plate(&FLAG, mx + 14.0, 287.0 - 20.0, fw, 26.0, 0.85);
                        text.draw(d, mx + 14.0 + 11.0, 287.0 - 1.0, 0.85);
                        mx += 14.0 + fw;
                    }
                }
                if !p.desc.is_empty() {
                    let desc_w = if self.preview.is_some() { 1220.0 } else { 1300.0 };
                    let (one, two) = self.wrap(d, &p.desc, desc_w);
                    for (k, line) in [one, two].iter().enumerate() {
                        if let Some(text) = d.text_fit(line, &white(26, "500"), desc_w) {
                            text.draw(d, EDGE, 334.0 + k as f32 * 35.1, 0.8);
                        }
                    }
                }
            }
        }
        for k in 0..4u64 {
            if let Some(text) = d.text(&clock(win + k * HALF_HOUR), &white(22, "500")) {
                text.draw(d, FG_LINE + k as f32 * GRID_W / 4.0, 431.0, 0.75);
            }
        }
        if live.channels.is_empty() {
            if let Some(text) = d.text("No channels yet. Open Live TV once, then come back.", &white(26, "400")) {
                text.draw(d, EDGE, FG_TOP + 64.0, 0.8);
            }
            return;
        }
        let x_of = |at: u64| ((at.clamp(win, win_end) - win) as f32 * GRID_W / SPAN as f32).round();
        let top = self.scroll.at(now);
        d.clip(Some((EDGE - 30.0, FG_TOP, DESIGN_W - 2.0 * EDGE + 60.0, 570.0)));
        for i in rows_in_view(top, FG_IN_VIEW, live.channels.len()) {
            let c = &live.channels[i];
            let y = FG_TOP + 10.0 + (i as f32 - top) * FG_STEP;
            let is_at = i == self.at;
            d.plate(if is_at { &CELL_ON } else { &CELL }, EDGE, y, 360.0, FG_ROW_H, 1.0);
            if let Some(num) = d.text(&c.number, &white(26, "600")) {
                num.draw(d, EDGE + 38.0 - num.advance / 2.0, y + 44.0, 1.0);
            }
            let card = Self::card(d, c, &live.logos, &GRID_CARD);
            d.picture(card, EDGE + 76.0, y + 9.0, 84.0, 52.0, 1.0);
            if let Some(name) = d.text_fit(&c.name, &white(22, "500"), 360.0 - 14.0 - 174.0) {
                name.draw(d, EDGE + 174.0, y + 43.0, 0.85);
            }
            let list = list_of(c);
            let shown = in_window(list, win, win_end);
            let row_focus = is_at && !self.on_rail;
            if shown.is_empty() {
                let cells: &[(f32, f32)] = if loading { &[(0.0, 0.3), (0.3, 0.72), (0.72, 1.0)] } else { &[(0.0, 1.0)] };
                for (k, (a, b)) in cells.iter().enumerate() {
                    let left = FG_LINE + (a * GRID_W).round();
                    let width = ((b - a) * GRID_W).round() - 8.0;
                    let lit = row_focus && k == 0;
                    d.plate(if lit { &CELL_FOCUS } else { &CELL }, left, y, width, FG_ROW_H, if loading || lit { 1.0 } else { 0.6 });
                    if loading {
                        let ph = if lit { PH_FOCUS } else { PH };
                        d.plate(&ph, left + 18.0, y + 15.0, (width - 36.0) * 0.7, 22.0, 1.0);
                        d.plate(&ph, left + 18.0, y + 43.0, 60.0, 14.0, 1.0);
                    } else if let Some(text) = d.text("No guide data", &style(26, "600", -0.01, if lit { FOCUS_INK } else { WHITE })) {
                        text.draw(d, left + 18.0, y + 44.0, if lit { 1.0 } else { 0.6 });
                    }
                }
                continue;
            }
            for p in shown {
                let left = x_of(p.start);
                let width = x_of(p.end) - left - 8.0;
                if width <= 0.0 {
                    continue;
                }
                let lit = row_focus && focused.is_some_and(|f| std::ptr::eq(f, p));
                let alpha = if lit { 1.0 } else if p.end <= t { 0.55 } else { 1.0 };
                d.plate(if lit { &CELL_FOCUS } else { &CELL }, FG_LINE + left, y, width, FG_ROW_H, alpha);
                if width > 60.0 {
                    let ink = if lit { FOCUS_INK } else { WHITE };
                    let began_earlier = p.start < win;
                    let indent = if began_earlier { STARTED_EARLIER_W } else { 0.0 };
                    if began_earlier {
                        let chevron = d.glyph(STARTED_EARLIER, 20, 2.4);
                        d.inked_picture(chevron, FG_LINE + left + 14.0, y + 14.0, 20.0, 20.0, [ink[0], ink[1], ink[2], alpha]);
                    }
                    if let Some(text) = d.text_fit(&p.title, &style(26, "600", -0.01, ink), width - 36.0 - indent) {
                        text.draw(d, FG_LINE + left + 18.0 + indent, y + 32.0, alpha);
                    }
                    if let Some(text) = d.text(&clock(p.start), &style(18, "500", 0.0, ink)) {
                        text.draw(d, FG_LINE + left + 18.0, y + 58.0, 0.75 * alpha);
                    }
                }
            }
        }
        if (win..win_end).contains(&t) {
            let x = FG_LINE + x_of(t);
            d.plate(&Plate { h: 2.0, radius: 0.0, ..NOW_LINE }, x, FG_TOP, 2.0, 570.0, 1.0);
            d.plate(&NOW_LINE, x - 5.0, FG_TOP, 12.0, 12.0, 1.0);
        }
        d.clip(None);
    }

    fn wrap(&mut self, d: &mut Draw, text: &str, max_w: f32) -> (String, String) {
        if let Some(done) = self.wrapped.get(text) {
            return done.clone();
        }
        let words: Vec<&str> = text.split_whitespace().collect();
        let style = style(26, "500", 0.0, WHITE);
        let (mut lo, mut hi) = (1, words.len());
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            if d.text(&words[..mid].join(" "), &style).is_some_and(|t| t.advance <= max_w) {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        let done = (words[..lo.min(words.len())].join(" "), words[lo.min(words.len())..].join(" "));
        if self.wrapped.len() > 200 {
            self.wrapped.clear();
        }
        self.wrapped.insert(text.to_string(), done.clone());
        done
    }

    fn draw_rail(&mut self, d: &mut Draw, now: u32) {
        let open = self.rail.at(now);
        if open > 0.0 {
            let x = -RAIL_W * (1.0 - open);
            d.p.fill.activate();
            d.p.fill.set4(c"u_box", x, 0.0, RAIL_W, DESIGN_H);
            d.p.fill.set1(c"u_radius", 0.0);
            d.p.fill.set4(c"u_ink", 12.0 / 255.0, 14.0 / 255.0, 24.0 / 255.0, 0.94);
            d.gfx.quad(&d.p.fill, x, 0.0, RAIL_W, DESIGN_H);
        }
        let rail_top = if self.over_live_tv { 150.0 } else { 200.0 };
        for (k, (view, label, glyph)) in self.rail_items().iter().enumerate() {
            let y = rail_top + k as f32 * RAIL_STEP;
            let focused = self.on_rail && k == self.rail_at;
            let on = *view == self.view;
            let label_text = d.text(label, &style(30, "600", 0.0, if focused { FOCUS_INK } else { WHITE }));
            let open_w = label_text.map_or(252.0, |t| (72.0 + t.advance + 28.0).max(252.0));
            let w = if self.on_rail { open_w } else { RAIL_ITEM };
            let alpha = if focused || (on && !self.on_rail) { 1.0 } else if self.on_rail { 0.85 } else { 0.6 };
            if on {
                d.plate(&RAIL_ON, 16.0, y, w, RAIL_ITEM, 1.0);
            }
            if focused {
                d.plate(&RAIL_FOCUS, 16.0, y, w, RAIL_ITEM, 1.0);
            }
            let icon = d.glyph(glyph, 32, 1.8);
            let ink = if focused { FOCUS_INK } else { WHITE };
            d.inked_picture(icon, 32.0, y + 16.0, 32.0, 32.0, [ink[0], ink[1], ink[2], alpha]);
            if let Some(text) = label_text.filter(|_| open > 0.0) {
                text.draw(d, 16.0 + 72.0, y + 42.5, alpha * open);
            }
        }
    }

    fn draw_typed(&mut self, d: &mut Draw, live: &Live) {
        if self.typed.is_empty() {
            return;
        }
        let hit = live.channels.iter().any(|c| c.number == self.typed);
        let Some(text) = d.text(&self.typed, &style(56, "600", 0.04, WHITE)) else { return };
        let w = (text.advance + 68.0).max(150.0);
        let (x, y) = (DESIGN_W / 2.0 - w / 2.0, 50.0);
        d.plate(&TYPED, x, y, w, 84.0, 1.0);
        text.draw(d, DESIGN_W / 2.0 - text.advance / 2.0, y + 42.0 + 20.0, if hit { 1.0 } else { 0.55 });
    }
}

impl crate::App {
    pub fn open_guide(&mut self, full: bool) {
        let t = self.unix_minute();
        self.guide = Some(Guide::new(full, self.over_live_tv, t));
        self.live.focus.store(0, std::sync::atomic::Ordering::Relaxed);
        self.live.guide_opened(true, &self.settings, crate::clock::unix_now());
        self.redraw = true;
        livetv::read_current_channel(|got| match got {
            Ok(Some(id)) => crate::post(move |app| {
                let count = app.live.channels.len();
                if let (Some(guide), Some(at)) = (app.guide.as_mut(), app.live.channels.iter().position(|c| c.id == id)) {
                    guide.tv_channel(at, count);
                    app.live.focus.store(guide.at(), std::sync::atomic::Ordering::Relaxed);
                    app.redraw = true;
                }
            }),
            Ok(None) => {}
            Err(why) => crate::note!("guide: current channel: {}", why),
        });
    }

    pub fn close_guide(&mut self) {
        self.guide = None;
        self.live.guide_opened(false, &self.settings, crate::clock::unix_now());
        self.redraw = true;
        if self.over_live_tv {
            self.close_side();
        }
    }

    pub fn guide_key(&mut self, key: Key) {
        let t = self.unix_minute();
        let Some(guide) = self.guide.as_mut() else { return };
        self.redraw = true;
        let outcome = guide.key(&self.draw.look, key, crate::ticks(), &self.live, t);
        self.live.focus.store(guide.at(), std::sync::atomic::Ordering::Relaxed);
        match outcome {
            Outcome::Stay => {}
            Outcome::Close => self.close_guide(),
            Outcome::Mode(_) if self.over_live_tv => {}
            Outcome::Mode(full) => self.edit_settings(|c| {
                c.insert("guideFull".into(), serde_json::json!(full));
            }),
            Outcome::Open(id) => {
                let over = guide.over_live_tv();
                self.watch(id);
                if over {
                    self.close_guide();
                }
            }
        }
    }

    pub fn guide_wait_ms(&mut self) -> Option<u32> {
        let waited = self.guide.as_mut()?.wake_in(crate::ticks())?;
        if waited == 0 {
            self.redraw = true;
        }
        Some(waited.max(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AT_21_17: u64 = 1_795_555_020;
    const AT_21_00: u64 = 1_795_554_000;

    fn channel(n: usize) -> Channel {
        Channel { id: format!("1_41_{n}_1_1_1_1"), signal: format!("{n}_1_1"), number: n.to_string(), name: format!("Channel {n}"), logo_path: None }
    }

    fn programme(title: &str, from_min: u64, to_min: u64) -> Programme {
        Programme { title: title.into(), start: AT_21_00 + from_min * 60, end: AT_21_00 + to_min * 60, subs: false, ad: false, desc: String::new() }
    }

    fn live_with(listing: Vec<Programme>) -> Live {
        let mut live = Live::new();
        live.channels = (1..=3).map(channel).collect();
        live.listings = [("1_1_1".to_string(), listing)].into();
        live
    }

    #[test]
    fn only_the_rows_in_view_and_one_more_while_scrolling_are_drawn() {
        assert_eq!(rows_in_view(0.0, 6, 40), 0..7);
        assert_eq!(rows_in_view(12.5, 6, 40), 12..19);
        assert_eq!(rows_in_view(36.0, 6, 40), 36..40);
        assert_eq!(rows_in_view(0.0, 6, 3), 0..3);
        assert_eq!(rows_in_view(0.0, 6, 0), 0..0);
    }

    #[test]
    fn the_focused_row_stays_at_its_place_until_the_list_runs_out() {
        assert_eq!(window_top(40, 1, 6, 2), 0);
        assert_eq!(window_top(40, 10, 6, 2), 8);
        assert_eq!(window_top(40, 39, 6, 2), 34);
        assert_eq!(window_top(4, 3, 6, 2), 0);
    }

    #[test]
    fn the_full_guide_draws_only_programmes_that_overlap_its_two_hours() {
        let list = vec![programme("Before", 0, 30), programme("Across", 20, 40), programme("In", 40, 60), programme("At the end", 150, 180), programme("After", 160, 200)];
        let titles: Vec<&str> = in_window(&list, AT_21_00 + 30 * 60, AT_21_00 + 150 * 60).iter().map(|p| p.title.as_str()).collect();
        assert_eq!(titles, ["Across", "In"]);
    }

    #[test]
    fn a_programme_later_on_says_how_long_until_it_starts() {
        assert_eq!(starts_in(&programme("Film", 43, 90), AT_21_17), "in 26 min");
        assert_eq!(starts_in(&programme("Film", 137, 180), AT_21_17), "in 2 hr");
        assert_eq!(starts_in(&programme("Film", 140, 180), AT_21_17), "in 2 hr 3 min");
    }

    #[test]
    fn typed_digits_build_a_number_of_up_to_four_and_the_fifth_starts_again() {
        assert_eq!(typed_next("", 1), "1");
        assert_eq!(typed_next("10", 1), "101");
        assert_eq!(typed_next("1011", 7), "7");
    }

    #[test]
    fn a_typed_number_moves_the_focus_to_that_channel_and_ok_opens_it() {
        let live = live_with(Vec::new());
        let mut guide = Guide::new(false, false, AT_21_17);
        assert_eq!(guide.key(&Look::defaults(), Key::Digit(3), 0, &live, AT_21_17), Outcome::Stay);
        assert_eq!(guide.at(), 2);
        assert_eq!(guide.key(&Look::defaults(), Key::Ok, 0, &live, AT_21_17), Outcome::Open("1_41_3_1_1_1_1".into()));
    }

    #[test]
    fn right_in_the_full_guide_steps_through_programmes_and_left_past_now_opens_the_rail() {
        let live = live_with(vec![programme("News", 0, 30), programme("Film", 30, 150), programme("Late", 150, 180)]);
        let mut guide = Guide::new(true, false, AT_21_17);
        let look = Look::defaults();
        guide.key(&look, Key::Right, 0, &live, AT_21_17);
        assert_eq!((guide.anchor, guide.win), (AT_21_00 + 30 * 60, AT_21_00));
        guide.key(&look, Key::Right, 0, &live, AT_21_17);
        assert_eq!((guide.anchor, guide.win), (AT_21_00 + 150 * 60, AT_21_00 + 120 * 60));
        for _ in 0..4 {
            guide.key(&look, Key::Left, 0, &live, AT_21_17);
        }
        assert!(guide.on_rail);
        assert_eq!(guide.key(&look, Key::Ok, 0, &live, AT_21_17), Outcome::Stay);
        guide.key(&look, Key::Left, 0, &live, AT_21_17);
        guide.key(&look, Key::Up, 0, &live, AT_21_17);
        assert_eq!(guide.key(&look, Key::Ok, 0, &live, AT_21_17), Outcome::Mode(false));
    }
}
