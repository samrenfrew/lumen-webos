use crate::compose::{self, Placed};
use crate::draw::{Decoded, Draw, Style};
use crate::ffi::*;
use crate::gfx::{Tex, DESIGN_H, DESIGN_W};
use crate::picker::{self, Picker, Wants};
use crate::settings::Forecast;
use crate::text::Shadow;
use crate::weather::{self, Weather};
use crate::note;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

const FADE_IN_MS: u32 = 600;
const FADE_OUT_MS: u32 = 500;
const CROSSFADE_MS: u32 = 2400;
const SHOW_ANYWAY_MS: u32 = 8000;
const FETCH_LEAD_MS: u32 = 10_000;
const RETRY_MS: u32 = 5000;
const SHADE_H: f32 = 520.0;
const SHADE_STOPS: [(f32, f32); 6] = [(0.0, 0.0), (0.25, 0.03), (0.45, 0.09), (0.65, 0.17), (0.82, 0.24), (1.0, 0.28)];
const LEFT: f32 = 125.0;
const RIGHT: f32 = DESIGN_W - 125.0;
const BOTTOM: f32 = DESIGN_H - 72.0;
const CLOCK_LEFT: f32 = LEFT - 6.0;
const CLOCK_BASELINE: f32 = 992.0;
const PERIOD_GAP: f32 = 10.0;
const PERIOD_BASELINE: f32 = 989.0;
const DATE_BASELINE: f32 = 988.0;
const WEATHER_BASELINE: f32 = 943.0;
const WEATHER_ICON: f32 = 36.0;
const WEATHER_ICON_TOP: f32 = 913.6;
const ROW_BOTTOM: f32 = BOTTOM - 12.0;
const CAST_COL: f32 = 72.0;
const CAST_GAP: f32 = 6.0;
const CAST_ICON: f32 = 30.0;
const CAST_HEIGHT: f32 = 103.8;
const CAST_HOURS: usize = 6;
const RAIN_LIKELY: i32 = 50;
const INK: [f32; 3] = [0.933, 0.933, 0.933];
const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
const RAIN_BLUE: [f32; 3] = [0.486, 0.769, 1.0];

const TIME_SHADOW: [Shadow; 1] = [Shadow { drop: 4, blur: 24, strength: 0.28 }];
const STATUS_SHADOW: [Shadow; 2] = [Shadow { drop: 1, blur: 16, strength: 0.5 }, Shadow { drop: 0, blur: 2, strength: 0.22 }];
const TIME: Style = Style { px: 120, weight: "600-clock", spacing_em: 0.0, feature: "case", ink: INK, shadows: &TIME_SHADOW };
const TIME_TNUM: Style = Style { feature: "tnum", ..TIME };
const PERIOD: Style = Style { px: 32, weight: "400", spacing_em: -0.012, feature: "", ink: INK, shadows: &TIME_SHADOW };
const LINE: Style = Style { px: 32, weight: "500", spacing_em: -0.012, feature: "", ink: INK, shadows: &STATUS_SHADOW };
const CAST_TEMP: Style = Style { px: 26, weight: "600", spacing_em: -0.011, feature: "", ink: WHITE, shadows: &STATUS_SHADOW };
const CAST_TIME: Style = Style { px: 18, weight: "400", spacing_em: -0.011, feature: "", ink: WHITE, shadows: &STATUS_SHADOW };

static OPENED: AtomicU64 = AtomicU64::new(0);
static PICKER: Mutex<Option<Picker>> = Mutex::new(None);

pub type Deliver = Arc<dyn Fn(u64, Option<Decoded>) + Send + Sync>;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Outcome {
    Stay,
    Close,
}

pub struct Setup {
    pub app_dir: String,
    pub wants: Wants,
    pub seconds: u32,
    pub night: bool,
    pub reduce_motion: bool,
    pub alone: bool,
    pub fetch: bool,
}

pub struct Saver {
    pub id: u64,
    setup: Setup,
    deliver: Deliver,
    photo_ms: u32,
    opened_at: u32,
    shown: Tex,
    shown_at: Option<u32>,
    incoming: Tex,
    swap_at: u32,
    leaving_at: Option<u32>,
    ask_at: Option<u32>,
    asking: bool,
    pub clock: (String, String, String),
    pub twenty_four_hour: bool,
    pub weather: Option<Weather>,
    pub weather_line: bool,
    pub forecast: Forecast,
    pub unix_now: u64,
}

fn ease(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn showing_flag() -> String {
    format!("{}/showing", picker::state_dir())
}

fn hour_text(t: &str, twenty_four_hour: bool) -> String {
    let hour: u32 = t.get(11..13).and_then(|h| h.parse().ok()).unwrap_or(0);
    if twenty_four_hour {
        format!("{hour:02}:00")
    } else {
        format!("{} {}", if hour % 12 == 0 { 12 } else { hour % 12 }, if hour < 12 { "AM" } else { "PM" })
    }
}

pub fn shade_alpha(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    SHADE_STOPS.windows(2).find(|w| t <= w[1].0).map_or(SHADE_STOPS[5].1, |w| w[0].1 + (w[1].1 - w[0].1) * (t - w[0].0) / (w[1].0 - w[0].0))
}

fn fetch(setup: &Setup, w: i32, h: i32, then: impl FnOnce(Option<Decoded>) + Send + 'static) {
    let (app_dir, wants, night) = (setup.app_dir.clone(), setup.wants.clone(), setup.night);
    std::thread::spawn(move || {
        let Ok(mut held) = PICKER.lock() else { return then(None) };
        let picker = held.get_or_insert_with(|| Picker::for_screensaver(&app_dir, wants.clone()));
        picker.set_wants(wants);
        let Some(pic) = picker.next() else { return then(None) };
        drop(held);
        let began = std::time::Instant::now();
        let placed: Vec<Placed> = pic.photos.iter().map(|p| Placed { picture: &p.picture, x: p.x, y: p.y }).collect();
        let frame = compose::frame(&placed, w, h, night);
        note!("saver: {} composed in {} ms", pic.id, began.elapsed().as_millis());
        then(frame);
    });
}

impl Saver {
    pub fn new(setup: Setup, deliver: Deliver, now: u32) -> Saver {
        let photo_ms = setup.seconds.max(5) * 1000;
        let fetching = setup.fetch;
        Saver {
            id: OPENED.fetch_add(1, Ordering::Relaxed) + 1,
            setup,
            deliver,
            photo_ms,
            opened_at: now,
            shown: Tex::default(),
            shown_at: None,
            incoming: Tex::default(),
            swap_at: 0,
            leaving_at: None,
            ask_at: fetching.then_some(now),
            asking: false,
            clock: (String::new(), String::new(), String::new()),
            twenty_four_hour: true,
            weather: None,
            weather_line: true,
            forecast: Forecast::Off,
            unix_now: 0,
        }
    }

    fn fade_ms(&self, ms: u32) -> u32 {
        if self.setup.reduce_motion { 1 } else { ms }
    }

    fn ask(&mut self, w: i32, h: i32) {
        self.asking = true;
        self.ask_at = None;
        let (id, deliver) = (self.id, self.deliver.clone());
        fetch(&self.setup, w, h, move |frame| deliver(id, frame));
    }

    pub fn arrived(&mut self, d: &mut Draw, frame: Option<Decoded>, now: u32) {
        self.asking = false;
        let Some(frame) = frame.filter(|_| self.leaving_at.is_none()) else {
            self.ask_at = Some(now + RETRY_MS);
            return;
        };
        let began = std::time::Instant::now();
        let picture = d.gfx.upload_rgba(&frame.rgba, frame.w, frame.h);
        note!("saver: upload {} ms", began.elapsed().as_millis());
        if self.shown.tex == 0 {
            self.shown = picture;
            if self.shown_at.is_none() {
                self.show(now);
            }
            self.swap_at = now + self.photo_ms;
            self.ask_at = Some(self.next_ask());
        } else {
            d.gfx.free(&mut self.incoming);
            self.incoming = picture;
            self.swap_at = self.swap_at.max(now);
            self.ask_at = None;
        }
    }

    fn next_ask(&self) -> u32 {
        self.swap_at.saturating_sub(FETCH_LEAD_MS.min(self.photo_ms))
    }

    fn show(&mut self, now: u32) {
        self.shown_at = Some(now);
        let _ = std::fs::write(showing_flag(), format!("{}\n", std::process::id()));
        note!("saver: first picture {} ms after it opened", now.wrapping_sub(self.opened_at));
    }

    pub fn tick(&mut self, d: &mut Draw, now: u32) -> Option<u32> {
        if self.shown_at.is_none() && now.wrapping_sub(self.opened_at) >= SHOW_ANYWAY_MS {
            self.show(now);
            self.swap_at = now + self.photo_ms;
        }
        let crossfade = self.fade_ms(CROSSFADE_MS);
        if self.incoming.tex != 0 && now >= self.swap_at + crossfade {
            std::mem::swap(&mut self.shown, &mut self.incoming);
            d.gfx.free(&mut self.incoming);
            self.swap_at = now + self.photo_ms;
            self.ask_at = Some(self.next_ask());
        }
        if self.leaving_at.is_none() && !self.asking && self.incoming.tex == 0 && self.ask_at.is_some_and(|at| now >= at) {
            self.ask(DESIGN_W as i32, DESIGN_H as i32);
        }
        let mut next = [self.ask_at, (self.shown_at.is_none()).then_some(self.opened_at + SHOW_ANYWAY_MS), (self.incoming.tex != 0).then_some(self.swap_at)];
        next.sort();
        next.into_iter().flatten().find(|&at| at > now).map(|at| at - now)
    }

    pub fn let_go(&mut self, d: &mut Draw) {
        d.gfx.free(&mut self.shown);
        d.gfx.free(&mut self.incoming);
        let _ = std::fs::remove_file(showing_flag());
    }

    pub fn key(&mut self, now: u32) -> Outcome {
        if self.shown_at.is_none() || self.setup.reduce_motion {
            return Outcome::Close;
        }
        if self.leaving_at.is_none() {
            self.leaving_at = Some(now);
            let _ = std::fs::remove_file(showing_flag());
            note!("saver: a key, leaving");
        }
        Outcome::Stay
    }

    pub fn done(&self, now: u32) -> bool {
        self.leaving_at.is_some_and(|at| now.wrapping_sub(at) >= FADE_OUT_MS)
    }

    fn scene(&self, now: u32) -> f32 {
        let Some(shown_at) = self.shown_at else { return 0.0 };
        let mut scene = ease(now.wrapping_sub(shown_at) as f32 / self.fade_ms(FADE_IN_MS) as f32);
        if let Some(at) = self.leaving_at {
            scene *= 1.0 - ease(now.wrapping_sub(at) as f32 / FADE_OUT_MS as f32);
        }
        scene
    }

    pub fn covers(&self, now: u32) -> bool {
        self.scene(now) >= 1.0
    }

    pub fn moving(&self, now: u32) -> bool {
        let fading_in = self.shown_at.is_some_and(|at| now.wrapping_sub(at) < self.fade_ms(FADE_IN_MS) + 50);
        let crossfading = self.incoming.tex != 0 && now >= self.swap_at;
        fading_in || crossfading || self.leaving_at.is_some()
    }

    #[cfg(feature = "dev")]
    pub fn where_now(&self, now: u32) -> serde_json::Value {
        serde_json::json!({"screen": "saver", "focused": [], "alone": self.setup.alone, "showing": self.shown_at.is_some(), "leaving": self.leaving_at.is_some(), "next_in_ms": self.swap_at.saturating_sub(now), "photos_held": [self.shown.tex != 0, self.incoming.tex != 0]})
    }

    pub fn draw(&mut self, d: &mut Draw, now: u32, over_home: bool) {
        d.pictures.next_frame();
        d.gfx.into_screen();
        unsafe {
            if !over_home {
                glClearColor(0.0, 0.0, 0.0, if self.setup.alone { 0.0 } else { 1.0 });
                glClear(GL_COLOR_BUFFER_BIT);
            }
            glEnable(GL_BLEND);
            glBlendFuncSeparate(GL_ONE, GL_ONE_MINUS_SRC_ALPHA, GL_ONE, GL_ONE_MINUS_SRC_ALPHA);
        }
        let scene = self.scene(now);
        if scene <= 0.0 {
            return;
        }
        d.p.dim.activate();
        d.p.dim.set1(c"u_alpha", scene);
        d.gfx.quad(&d.p.dim, 0.0, 0.0, DESIGN_W, DESIGN_H);
        if self.shown.tex != 0 {
            d.picture(self.shown, 0.0, 0.0, DESIGN_W, DESIGN_H, scene);
        }
        if self.incoming.tex != 0 && now >= self.swap_at {
            let cross = ease(now.saturating_sub(self.swap_at) as f32 / self.fade_ms(CROSSFADE_MS) as f32);
            d.picture(self.incoming, 0.0, 0.0, DESIGN_W, DESIGN_H, scene * cross);
        }
        let shade = Self::shade(d);
        d.picture(shade, 0.0, DESIGN_H - SHADE_H, DESIGN_W, SHADE_H, scene);
        self.draw_status(d, scene);
    }

    fn shade(d: &mut Draw) -> Tex {
        if let Some(p) = d.pictures.find("saver:shade") {
            return p;
        }
        let h = SHADE_H as usize;
        let rgba: Vec<u8> = (0..h).flat_map(|y| {
            let a = (shade_alpha((y as f32 + 0.5) / h as f32) * 255.0 + 0.5) as u8;
            [0, 0, 0, a].repeat(4)
        }).collect();
        let picture = d.gfx.upload_rgba(&rgba, 4, h as i32);
        d.keep("saver:shade", picture);
        picture
    }

    fn draw_status(&mut self, d: &mut Draw, alpha: f32) {
        let (time, period, date) = self.clock.clone();
        let time_style = if d.fonts.is_sf("600-clock") { TIME } else { TIME_TNUM };
        let mut clock_w = 0.0;
        if let Some(t) = d.text(&time, &time_style) {
            t.draw(d, CLOCK_LEFT, CLOCK_BASELINE, alpha);
            clock_w = t.advance;
            if let Some(p) = d.text(&period, &PERIOD) {
                p.draw(d, CLOCK_LEFT + t.advance + PERIOD_GAP, PERIOD_BASELINE, alpha);
                clock_w += PERIOD_GAP + p.advance;
            }
        }
        let mut row_w = 0.0f32;
        if let Some(t) = d.text(&date, &LINE) {
            t.draw(d, RIGHT - t.advance, DATE_BASELINE, alpha);
            row_w = t.advance;
        }
        let large = self.forecast == Forecast::Large;
        if let Some(w) = self.weather.clone().filter(|_| self.weather_line && !large) {
            let temp = d.text(&format!("{}°", w.temp), &LINE);
            let cond = d.text(&w.label, &LINE);
            let width = WEATHER_ICON + 10.0 + temp.map_or(0.0, |t| t.advance) + cond.map_or(0.0, |c| 12.0 + c.advance);
            let x = RIGHT - width;
            let icon = d.glyph(weather::icon(w.kind, w.day), WEATHER_ICON as i32, 1.8);
            d.inked_picture(icon, x, WEATHER_ICON_TOP + 2.0, WEATHER_ICON, WEATHER_ICON, [0.0, 0.0, 0.0, 0.6 * alpha]);
            d.inked_picture(icon, x, WEATHER_ICON_TOP, WEATHER_ICON, WEATHER_ICON, [INK[0], INK[1], INK[2], alpha]);
            let mut at = x + WEATHER_ICON + 10.0;
            if let Some(t) = temp {
                t.draw(d, at, WEATHER_BASELINE, alpha);
                at += t.advance + 12.0;
            }
            if let Some(c) = cond {
                c.draw(d, at, WEATHER_BASELINE, alpha);
            }
            row_w = row_w.max(width);
        }
        // ponytail: the large forecast draws as the small one, between the clock and the date, as on home
        let Some(w) = self.weather.clone().filter(|_| self.forecast != Forecast::Off) else { return };
        let hours = weather::hours_ahead(&w, self.unix_now, CAST_HOURS);
        if hours.is_empty() {
            return;
        }
        let cast_w = hours.len() as f32 * (CAST_COL + CAST_GAP) - CAST_GAP;
        let free = (RIGHT - LEFT) - (clock_w - 6.0) - (cast_w + 48.0) - row_w;
        let left = LEFT + (clock_w - 6.0) + free / 2.0 + 24.0;
        let top = ROW_BOTTOM - CAST_HEIGHT;
        for (i, hour) in hours.iter().enumerate() {
            let centre = left + i as f32 * (CAST_COL + CAST_GAP) + CAST_COL / 2.0;
            let icon = d.glyph(weather::icon(hour.kind, hour.day), CAST_ICON as i32, 1.8);
            d.inked_picture(icon, centre - CAST_ICON / 2.0, top + 1.0, CAST_ICON, CAST_ICON, [0.0, 0.0, 0.0, 0.35 * alpha]);
            d.inked_picture(icon, centre - CAST_ICON / 2.0, top, CAST_ICON, CAST_ICON, [1.0, 1.0, 1.0, alpha]);
            if let Some(t) = d.text(&format!("{}°", hour.temp), &CAST_TEMP) {
                t.draw(d, centre - t.advance / 2.0, top + 57.4, alpha);
            }
            if let Some(t) = d.text(&hour_text(&hour.t, self.twenty_four_hour), &CAST_TIME) {
                t.draw(d, centre - t.advance / 2.0, top + 85.8, alpha * 0.6);
            }
            let pill_y = ROW_BOTTOM - 5.0;
            Self::pill(d, centre - 22.0, pill_y, 44.0, [1.0, 1.0, 1.0], 0.24 * alpha);
            if hour.rain > 0 {
                let strength = if hour.rain >= RAIN_LIKELY { 1.0 } else { 0.6 };
                Self::pill(d, centre - 22.0, pill_y, (44.0 * hour.rain.min(100) as f32 / 100.0).max(5.0), RAIN_BLUE, strength * alpha);
            }
        }
    }

    fn pill(d: &Draw, x: f32, y: f32, w: f32, ink: [f32; 3], alpha: f32) {
        d.p.fill.activate();
        d.p.fill.set4(c"u_box", x, y, w, 5.0);
        d.p.fill.set1(c"u_radius", 2.5);
        d.p.fill.set4(c"u_ink", ink[0], ink[1], ink[2], alpha);
        d.gfx.quad(&d.p.fill, x, y, w, 5.0);
    }
}

pub fn fetch_immich_wallpaper(app_dir: &str, wants: Wants, as_next: bool, then: impl FnOnce(Result<(), String>) + Send + 'static) {
    static WALLPAPER_PICKER: Mutex<Option<Picker>> = Mutex::new(None);
    let app_dir = app_dir.to_string();
    std::thread::spawn(move || {
        let Ok(mut held) = WALLPAPER_PICKER.lock() else { return then(Err("busy".into())) };
        let picker = held.get_or_insert_with(|| Picker::for_wallpaper(&app_dir, wants.clone()));
        picker.set_wants(wants);
        let Some((pick, bytes)) = picker.next_wallpaper() else { return then(Err("no Immich photo".into())) };
        let (picture, key) = if as_next { (crate::wallpaper::NEXT, crate::wallpaper::NEXT_PICK) } else { (crate::wallpaper::SHOWN, crate::wallpaper::SHOWN_PICK) };
        let kept = crate::store::write(picture, &bytes) && crate::store::write(key, pick.as_bytes());
        then(if kept { Ok(()) } else { Err(format!("{pick}: not kept")) });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shade_behind_the_clock_has_no_top_edge_and_darkens_towards_the_bottom() {
        assert_eq!(shade_alpha(0.0), 0.0);
        assert!((shade_alpha(0.25) - 0.03).abs() < 1e-6);
        assert!((shade_alpha(1.0) - 0.28).abs() < 1e-6);
        let steps: Vec<f32> = (0..=20).map(|i| shade_alpha(i as f32 / 20.0)).collect();
        assert!(steps.windows(2).all(|w| w[1] >= w[0]));
    }
}
