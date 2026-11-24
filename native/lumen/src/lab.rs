use crate::draw::{Draw, Style};
use crate::home::Key;
use crate::look::Look;
use serde_json::{json, Value};

const INK: [f32; 3] = [0.94, 0.94, 0.95];
const DIM_INK: [f32; 3] = [0.70, 0.71, 0.74];
const DARK: [f32; 3] = [0.08, 0.08, 0.10];
const NAME: Style = Style { px: 22, weight: "500", spacing_em: -0.005, feature: "", ink: INK, shadows: &[] };
const VALUE: Style = Style { px: 20, weight: "500", spacing_em: 0.0, feature: "", ink: DIM_INK, shadows: &[] };
const HEAD: Style = Style { px: 20, weight: "600", spacing_em: 0.06, feature: "", ink: DIM_INK, shadows: &[] };
const BUTTON: Style = Style { px: 22, weight: "600", spacing_em: 0.0, feature: "", ink: INK, shadows: &[] };
const BUTTON_ON: Style = Style { ink: DARK, ..BUTTON };

const PANEL_X: f32 = 290.0;
const PANEL_Y: f32 = 160.0;
const PANEL_W: f32 = 1340.0;
const PANEL_H: f32 = 622.0;
const BUTTONS_Y: f32 = 184.0;
const BUTTON_H: f32 = 54.0;
const BUTTON_GAP: f32 = 14.0;
const ROWS_Y: f32 = 262.0;
const ROW_H: f32 = 42.0;
const COL_W: f32 = 640.0;
const NAME_W: f32 = 290.0;
const TRACK_W: f32 = 220.0;
const AUTO_STEP_MS: u32 = 650;
const FAST_REPEAT_MS: u32 = 160;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Toggle {
    Liquid,
    Focus,
    Moving,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Control {
    Slider { name: &'static str, label: &'static str, min: f32, max: f32, step: f32 },
    Switch(Toggle, &'static str),
}

const fn slider(name: &'static str, label: &'static str, min: f32, max: f32, step: f32) -> Control {
    Control::Slider { name, label, min, max, step }
}

pub const CONTROLS: [Control; 23] = [
    Control::Switch(Toggle::Liquid, "Liquid glass"),
    Control::Switch(Toggle::Focus, "Show focus"),
    Control::Switch(Toggle::Moving, "Move focus by itself"),
    slider("bend", "Edge bend", -200.0, 200.0, 4.0),
    slider("bezel", "Edge width", 1.0, 110.0, 1.0),
    slider("edge_shape", "Edge shape, round to flat", 1.0, 8.0, 0.25),
    slider("side_width", "Side edges, width", 0.2, 2.0, 0.05),
    slider("side_bend", "Side edges, bend", 0.0, 1.0, 0.05),
    slider("fringe", "Colour fringe", 0.0, 1.0, 0.02),
    slider("magnify", "Magnify", -0.2, 0.2, 0.01),
    slider("liquid_blur", "Blur, liquid glass", 0.0, 4.0, 0.05),
    slider("glass_passes", "Blur passes", 1.0, 8.0, 1.0),
    slider("frost", "Frost", 0.0, 0.6, 0.01),
    slider("tint", "Dark tint", 0.0, 1.0, 0.02),
    slider("saturate", "Colour boost", 0.0, 3.0, 0.05),
    slider("brighten", "Brightness", 0.0, 3.0, 0.05),
    slider("specular", "Light glint", 0.0, 2.0, 0.05),
    slider("light_angle", "Light from, degrees", -180.0, 180.0, 5.0),
    slider("rim_light", "White rim", 0.0, 3.0, 0.05),
    slider("colour_rims", "Blue and pink rim, bar", 0.0, 3.0, 0.05),
    slider("button_colour_rims", "Blue and pink rim, buttons", 0.0, 3.0, 0.05),
    slider("sheen", "Sheen", 0.0, 3.0, 0.05),
    slider("glow", "Inner glow", 0.0, 8.0, 0.25),
];
const LEFT_COLUMN: usize = 12;
pub const BUTTONS: [&str; 4] = ["Exit", "New wallpaper", "Hide controls", "Reset"];

pub const GLASS_LOOK: [&str; 23] = ["bend", "bezel", "edge_shape", "side_width", "side_bend", "fringe", "magnify", "liquid_blur", "glass_halvings", "glass_passes", "frost", "tint", "saturate", "brighten", "glass_shadow", "specular", "light_angle", "rim_light", "colour_rims", "button_colour_rims", "sheen", "glow", "finish"];

#[derive(Clone, PartialEq, Debug)]
pub enum Outcome {
    Stay,
    Exit,
    Wallpaper,
    Reset,
    Set(&'static str, f32),
    Switched(Toggle, bool),
}

pub struct Lab {
    at: usize,
    button: usize,
    pub hidden: bool,
    pub show_focus: bool,
    pub auto_move: bool,
    last_key: (Option<Key>, u32, u32),
    pointer: Option<(f32, f32)>,
    auto_at: u32,
    auto_right: bool,
    frames: Vec<(u32, f64)>,
}

fn rounded(value: f32, step: f32, min: f32) -> f32 {
    let steps = ((value - min) / step).round();
    let v = min + steps * step;
    (v * 10000.0).round() / 10000.0
}

fn shown(value: f32, step: f32) -> String {
    if step >= 1.0 {
        format!("{}", value.round() as i32)
    } else if step >= 0.1 {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    }
}

impl Lab {
    pub fn new(now: u32) -> Lab {
        Lab { at: 1, button: 0, hidden: false, show_focus: false, auto_move: false, last_key: (None, 0, 0), pointer: None, auto_at: now, auto_right: true, frames: Vec::new() }
    }

    fn switch_on(&self, toggle: Toggle, liquid: bool) -> bool {
        match toggle {
            Toggle::Liquid => liquid,
            Toggle::Focus => self.show_focus,
            Toggle::Moving => self.auto_move,
        }
    }

    fn flip(&mut self, toggle: Toggle, liquid: bool) -> Outcome {
        let on = !self.switch_on(toggle, liquid);
        match toggle {
            Toggle::Liquid => {}
            Toggle::Focus => self.show_focus = on,
            Toggle::Moving => {
                self.auto_move = on;
                if on {
                    self.show_focus = true;
                }
            }
        }
        Outcome::Switched(toggle, on)
    }

    fn press(&mut self, button: usize) -> Outcome {
        match button {
            0 => Outcome::Exit,
            1 => Outcome::Wallpaper,
            2 => {
                self.hidden = true;
                Outcome::Stay
            }
            _ => Outcome::Reset,
        }
    }

    fn speed(&mut self, key: Key, now: u32) -> f32 {
        let (last, at, run) = self.last_key;
        let run = if last == Some(key) && now.wrapping_sub(at) < FAST_REPEAT_MS { run + 1 } else { 0 };
        self.last_key = (Some(key), now, run);
        (1 + run / 6).min(5) as f32
    }

    pub fn key(&mut self, key: Key, look: &Look, liquid: bool, now: u32) -> Outcome {
        if self.hidden {
            self.hidden = false;
            return Outcome::Stay;
        }
        let speed = self.speed(key, now);
        let last = CONTROLS.len();
        match key {
            Key::Back => return Outcome::Exit,
            Key::Up => self.at = self.at.saturating_sub(1),
            Key::Down => self.at = (self.at + 1).min(last),
            Key::Left | Key::Right if self.at == 0 => {
                let by = if key == Key::Right { 1 } else { BUTTONS.len() - 1 };
                self.button = (self.button + by) % BUTTONS.len();
            }
            Key::Ok if self.at == 0 => return self.press(self.button),
            Key::Left | Key::Right | Key::Ok => match CONTROLS[self.at - 1] {
                Control::Switch(toggle, _) => return self.flip(toggle, liquid),
                Control::Slider { name, min, max, step, .. } => {
                    let now_value = look.get(name).unwrap_or(min);
                    let wanted = match (key, self.pointer.and_then(|(x, y)| self.track_value(x, y))) {
                        (Key::Ok, Some(v)) => v,
                        (Key::Ok, None) => return Outcome::Stay,
                        (Key::Left, _) => now_value - step * speed,
                        _ => now_value + step * speed,
                    };
                    let value = rounded(wanted.clamp(min, max), step, min).clamp(min, max);
                    return if value == now_value { Outcome::Stay } else { Outcome::Set(name, value) };
                }
            },
            Key::Digit(_) => {}
        }
        Outcome::Stay
    }

    fn row_place(index: usize) -> (f32, f32) {
        let (col, row) = if index < LEFT_COLUMN { (0, index) } else { (1, index - LEFT_COLUMN) };
        (PANEL_X + 30.0 + col as f32 * (COL_W + 20.0), ROWS_Y + row as f32 * ROW_H)
    }

    fn track_x(index: usize) -> f32 {
        Self::row_place(index).0 + NAME_W + 16.0
    }

    fn button_boxes() -> Vec<(f32, f32)> {
        let widths = [140.0, 240.0, 240.0, 150.0];
        let mut x = PANEL_X + 30.0;
        widths.iter().map(|w| {
            let at = (x, *w);
            x += w + BUTTON_GAP;
            at
        }).collect()
    }

    fn track_value(&self, x: f32, y: f32) -> Option<f32> {
        let index = self.at.checked_sub(1)?;
        let Control::Slider { min, max, .. } = CONTROLS[index] else { return None };
        let (_, top) = Self::row_place(index);
        let left = Self::track_x(index);
        if y < top || y > top + ROW_H || x < left - 20.0 || x > left + TRACK_W + 20.0 {
            return None;
        }
        Some(min + (max - min) * ((x - left) / TRACK_W).clamp(0.0, 1.0))
    }

    pub fn point(&mut self, x: f32, y: f32) -> bool {
        self.pointer = Some((x, y));
        if self.hidden {
            return false;
        }
        let was = (self.at, self.button);
        if (BUTTONS_Y..BUTTONS_Y + BUTTON_H).contains(&y) {
            if let Some(i) = Self::button_boxes().iter().position(|(bx, w)| x >= *bx && x <= bx + w) {
                self.at = 0;
                self.button = i;
            }
        }
        for index in 0..CONTROLS.len() {
            let (left, top) = Self::row_place(index);
            if x >= left && x <= left + COL_W && y >= top && y < top + ROW_H {
                self.at = index + 1;
            }
        }
        was != (self.at, self.button)
    }

    pub fn auto_step(&mut self, now: u32) -> Option<bool> {
        if !self.auto_move || now.wrapping_sub(self.auto_at) < AUTO_STEP_MS {
            return None;
        }
        self.auto_at = now;
        Some(self.auto_right)
    }

    pub fn turn_back(&mut self) {
        self.auto_right = !self.auto_right;
    }

    pub fn moving(&self) -> bool {
        self.auto_move
    }

    pub fn frame_shown(&mut self, now: u32, draw_ms: f64) {
        self.frames.retain(|(at, _)| now.wrapping_sub(*at) < 2000);
        self.frames.push((now, draw_ms));
    }

    fn timing(&self) -> String {
        let Some(&(_, last)) = self.frames.last() else { return String::new() };
        if !self.auto_move || self.frames.len() < 3 {
            return format!("Last frame took {last:.1} ms to draw");
        }
        let span = self.frames.last().unwrap().0.wrapping_sub(self.frames[0].0).max(1);
        let fps = (self.frames.len() - 1) as f32 * 1000.0 / span as f32;
        let average = self.frames.iter().map(|f| f.1).sum::<f64>() / self.frames.len() as f64;
        let worst = self.frames.iter().map(|f| f.1).fold(0.0, f64::max);
        format!("{fps:.0} frames a second, {average:.1} ms to draw, worst {worst:.1} ms")
    }

    pub fn where_now(&self, look: &Look, liquid: bool) -> Value {
        if self.hidden {
            return json!({"screen": "glass lab", "focused": [{"part": "lab", "label": "controls hidden"}]});
        }
        let focused = match self.at.checked_sub(1).map(|i| CONTROLS[i]) {
            None => json!({"part": "lab__button", "label": BUTTONS[self.button], "index": self.button}),
            Some(Control::Slider { name, label, .. }) => json!({"part": "lab__row", "label": label, "name": name, "value": look.get(name)}),
            Some(Control::Switch(toggle, label)) => json!({"part": "lab__row", "label": label, "on": self.switch_on(toggle, liquid)}),
        };
        json!({"screen": "glass lab", "focused": [focused], "timing": self.timing()})
    }

    pub fn draw(&mut self, d: &mut Draw, liquid: bool) {
        if self.hidden {
            return;
        }
        d.fill(PANEL_X, PANEL_Y, PANEL_W, PANEL_H, 36.0, [0.0, 0.0, 0.0, 0.58]);
        for (i, (x, w)) in Self::button_boxes().into_iter().enumerate() {
            let focused = self.at == 0 && self.button == i;
            d.fill(x, BUTTONS_Y, w, BUTTON_H, BUTTON_H / 2.0, if focused { [0.95, 0.95, 0.96, 1.0] } else { [1.0, 1.0, 1.0, 0.14] });
            if let Some(text) = d.text(BUTTONS[i], if focused { &BUTTON_ON } else { &BUTTON }) {
                text.draw(d, x + w / 2.0 - text.advance / 2.0, BUTTONS_Y + 35.0, 1.0);
            }
        }
        let timing = self.timing();
        if let Some(text) = d.text(&timing, &VALUE) {
            text.draw(d, PANEL_X + PANEL_W - 30.0 - text.advance, BUTTONS_Y + 35.0, 1.0);
        }
        if let Some(text) = d.text("GLASS LAB", &HEAD) {
            text.draw(d, PANEL_X + PANEL_W - 30.0 - text.advance, PANEL_Y + PANEL_H - 22.0, 0.8);
        }
        for (index, control) in CONTROLS.iter().enumerate() {
            let (left, top) = Self::row_place(index);
            let focused = self.at == index + 1;
            if focused {
                d.fill(left - 14.0, top + 2.0, COL_W, ROW_H - 4.0, 14.0, [1.0, 1.0, 1.0, 0.10]);
            }
            let middle = top + ROW_H / 2.0;
            let track = Self::track_x(index);
            match *control {
                Control::Slider { name, label, min, max, step } => {
                    if let Some(text) = d.text_fit(label, &NAME, NAME_W) {
                        text.draw(d, left, middle + 8.0, 1.0);
                    }
                    let value = d.look.get(name).unwrap_or(min);
                    let t = ((value - min) / (max - min)).clamp(0.0, 1.0);
                    let thick = if focused { 8.0 } else { 6.0 };
                    d.fill(track, middle - thick / 2.0, TRACK_W, thick, thick / 2.0, [1.0, 1.0, 1.0, 0.14]);
                    let lit = if focused { 0.95 } else { 0.7 };
                    d.fill(track, middle - thick / 2.0, (TRACK_W * t).max(thick), thick, thick / 2.0, [lit, lit, lit, lit]);
                    let knob = if focused { 22.0 } else { 16.0 };
                    d.fill(track + TRACK_W * t - knob / 2.0, middle - knob / 2.0, knob, knob, knob / 2.0, [lit, lit, lit, lit]);
                    if let Some(text) = d.text(&shown(value, step), &VALUE) {
                        text.draw(d, track + TRACK_W + 24.0, middle + 7.0, 1.0);
                    }
                }
                Control::Switch(toggle, label) => {
                    if let Some(text) = d.text_fit(label, &NAME, NAME_W) {
                        text.draw(d, left, middle + 8.0, 1.0);
                    }
                    let on = self.switch_on(toggle, liquid);
                    let (w, h) = (60.0, 32.0);
                    d.fill(track, middle - h / 2.0, w, h, h / 2.0, if on { [0.20, 0.62, 0.36, 1.0] } else { [1.0, 1.0, 1.0, 0.22] });
                    let knob = h - 6.0;
                    let kx = if on { track + w - 3.0 - knob } else { track + 3.0 };
                    d.fill(kx, middle - knob / 2.0, knob, knob, knob / 2.0, [0.96, 0.96, 0.97, 1.0]);
                    if let Some(text) = d.text(if on { "On" } else { "Off" }, &VALUE) {
                        text.draw(d, track + TRACK_W + 24.0, middle + 7.0, 1.0);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index_of(name: &str) -> usize {
        1 + CONTROLS.iter().position(|c| matches!(c, Control::Slider { name: n, .. } if *n == name)).unwrap()
    }

    #[test]
    fn every_slider_names_a_look_value_and_stays_inside_its_range() {
        let look = Look::defaults();
        for control in CONTROLS {
            if let Control::Slider { name, min, max, .. } = control {
                let mut probe = look.clone();
                assert!(probe.set(name, min) && probe.set(name, max), "{name} {min}..{max} is outside the look's range");
                assert!(GLASS_LOOK.contains(&name), "{name} is not reset by Reset");
            }
        }
    }

    #[test]
    fn right_steps_a_slider_up_and_stops_at_its_end() {
        let mut lab = Lab::new(0);
        let mut look = Look::defaults();
        lab.at = index_of("fringe");
        assert_eq!(lab.key(Key::Right, &look, true, 1000), Outcome::Set("fringe", 0.14));
        look.set("fringe", 1.0);
        assert_eq!(lab.key(Key::Right, &look, true, 2000), Outcome::Stay);
    }

    #[test]
    fn holding_a_key_speeds_the_slider_up() {
        let mut lab = Lab::new(0);
        let look = Look::defaults();
        lab.at = index_of("bend");
        let mut last = Outcome::Stay;
        for i in 0..20 {
            last = lab.key(Key::Right, &look, true, 1000 + i * 50);
        }
        assert_eq!(last, Outcome::Set("bend", Look::defaults().bend + 4.0 * 4.0));
    }

    #[test]
    fn down_runs_through_both_columns_and_up_reaches_the_buttons() {
        let mut lab = Lab::new(0);
        let look = Look::defaults();
        for i in 0..40 {
            lab.key(Key::Down, &look, false, i * 1000);
        }
        assert_eq!(lab.at, CONTROLS.len());
        for i in 0..40 {
            lab.key(Key::Up, &look, false, 50_000 + i * 1000);
        }
        lab.key(Key::Left, &look, false, 100_000);
        assert_eq!(lab.key(Key::Ok, &look, false, 101_000), Outcome::Reset);
    }

    #[test]
    fn with_the_controls_hidden_any_key_brings_them_back_and_does_nothing_else() {
        let mut lab = Lab::new(0);
        let look = Look::defaults();
        lab.hidden = true;
        assert_eq!(lab.key(Key::Back, &look, false, 1000), Outcome::Stay);
        assert!(!lab.hidden);
    }

    #[test]
    fn moving_the_focus_by_itself_also_shows_the_focus() {
        let mut lab = Lab::new(0);
        let look = Look::defaults();
        lab.at = 3;
        assert_eq!(lab.key(Key::Ok, &look, false, 1000), Outcome::Switched(Toggle::Moving, true));
        assert!(lab.show_focus && lab.moving());
    }
}
