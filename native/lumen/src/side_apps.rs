use crate::control_center::ControlCenter;
use crate::draw::Draw;
use crate::ffi::*;
use crate::gfx::{Tex, DESIGN_W};
use crate::settings::Settings;
use crate::{guide, note, store, ticks, App, SETTINGS_FILE};
use serde_json::Value;

pub const QUICK_APP_ID: &str = "com.samrenfrew.lumen.native.quick";
const GUIDE_PROGRAM: &str = "lumen-guide";
const QUICK_PROGRAM: &str = "lumen-quick";
pub const IDLE_CLOSE_MS: u32 = 30_000;
pub const CLOSE_AFTER_MS: u32 = 400;
const QUICK_GLASS_RGB: [u8; 3] = [40, 44, 58];
const SHADE_W: f32 = 900.0;
const SHADE_H: f32 = 760.0;
const SHADE_STEPS: [(f32, f32); 3] = [(0.0, 0.6), (0.45, 0.32), (0.72, 0.0)];
const SHADE_RGB: [f32; 3] = [8.0 / 255.0, 10.0 / 255.0, 18.0 / 255.0];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Side {
    Guide,
    Quick,
}

impl Side {
    pub fn app_id(self) -> &'static str {
        match self {
            Side::Guide => guide::GUIDE_APP_ID,
            Side::Quick => QUICK_APP_ID,
        }
    }
}

pub fn side_of(args: &[String]) -> Option<Side> {
    let program = args.first().and_then(|a| a.rsplit('/').next()).unwrap_or("");
    let launched_as = args.get(1).and_then(|a| serde_json::from_str::<Value>(a).ok()).and_then(|l| l["appId"].as_str().map(String::from));
    match (program, launched_as.as_deref()) {
        (GUIDE_PROGRAM, _) | (_, Some(guide::GUIDE_APP_ID)) => Some(Side::Guide),
        (QUICK_PROGRAM, _) | (_, Some(QUICK_APP_ID)) => Some(Side::Quick),
        _ => None,
    }
}

pub fn asked_to_stay_hidden(launch: &Value) -> bool {
    launch["hidden"].as_bool() == Some(true) || launch["parameters"]["hidden"].as_bool() == Some(true)
}

fn shade_alpha(x: f32, y: f32) -> f32 {
    let reach = (((SHADE_W - x) / SHADE_W).powi(2) + (y / SHADE_H).powi(2)).sqrt() / std::f32::consts::SQRT_2;
    let at = SHADE_STEPS.windows(2).find(|w| reach <= w[1].0);
    match at {
        Some(w) => w[0].1 + (w[1].1 - w[0].1) * (reach - w[0].0) / (w[1].0 - w[0].0),
        None => 0.0,
    }
}

fn shade(d: &mut Draw) -> Tex {
    let key = "quick:shade";
    if let Some(picture) = d.pictures.find(key) {
        return picture;
    }
    let (w, h) = ((SHADE_W / 4.0) as usize, (SHADE_H / 4.0) as usize);
    let mut rgba = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let a = shade_alpha((x as f32 + 0.5) * 4.0, (y as f32 + 0.5) * 4.0);
            let px = [SHADE_RGB[0] * a, SHADE_RGB[1] * a, SHADE_RGB[2] * a, a].map(|c| (c * 255.0) as u8);
            rgba[(y * w + x) * 4..][..4].copy_from_slice(&px);
        }
    }
    let picture = d.gfx.upload_rgba(&rgba, w as i32, h as i32);
    d.keep(key, picture);
    picture
}

impl App {
    pub(crate) fn start_side(&mut self, side: Side, launch: &Value) {
        if let Some(text) = store::read(SETTINGS_FILE) {
            if let Some(kept) = Settings::parse(&text) {
                self.settings = kept;
                self.saved = serde_json::from_str(&text).unwrap_or(Value::Null);
            }
        }
        self.tick_clock();
        if asked_to_stay_hidden(launch) {
            note!("{}: asked to start hidden, which a native side app does not need; closing", side.app_id());
            self.running = false;
            return;
        }
        match side {
            Side::Guide => {
                self.live.load_kept();
                self.open_guide(launch["parameters"]["view"].as_str() == Some("full"));
            }
            Side::Quick => {
                let glass = QUICK_GLASS_RGB.iter().chain(&[255]).copied().cycle().take(4 * 4 * 4).collect::<Vec<u8>>();
                self.draw.glass_backdrop = self.draw.gfx.upload_rgba(&glass, 4, 4);
                self.panel = Some(ControlCenter::quick(ticks()));
            }
        }
        self.redraw = true;
    }

    pub(crate) fn close_side(&mut self) {
        if self.quit_at.is_some() {
            return;
        }
        note!("{}: closing", self.side.map_or("", Side::app_id));
        self.guide = None;
        if let Some(panel) = self.panel.as_mut() {
            panel.close(ticks(), &self.draw.look);
        }
        self.quit_at = Some(ticks().wrapping_add(CLOSE_AFTER_MS));
        self.redraw = true;
    }

    pub(crate) fn side_relaunched(&mut self, parameters: &Value) {
        if asked_to_stay_hidden(parameters) {
            return note!("relaunch {}: hidden start while up, left as it is", parameters);
        }
        note!("relaunch {}: closing, as LIST or the gear pressed again does", parameters);
        self.close_side();
    }

    pub(crate) fn side_wait_ms(&mut self) -> Option<u32> {
        self.side?;
        let now = ticks();
        if let Some(at) = self.quit_at {
            let left = at.wrapping_sub(now) as i32;
            if left <= 0 {
                self.running = false;
                return None;
            }
            return Some(left as u32);
        }
        let idle = now.wrapping_sub(self.touched_at);
        if idle >= IDLE_CLOSE_MS {
            note!("closing after {} s without a key", IDLE_CLOSE_MS / 1000);
            self.close_side();
            return Some(CLOSE_AFTER_MS);
        }
        Some(IDLE_CLOSE_MS - idle)
    }

    pub(crate) fn draw_quick(&mut self, now: u32) {
        self.draw.pictures.next_frame();
        self.draw.gfx.into_screen();
        unsafe {
            glClearColor(0.0, 0.0, 0.0, 0.0);
            glClear(GL_COLOR_BUFFER_BIT);
            glEnable(GL_BLEND);
            glBlendFuncSeparate(GL_ONE, GL_ONE_MINUS_SRC_ALPHA, GL_ONE, GL_ONE_MINUS_SRC_ALPHA);
        }
        let Some(panel) = self.panel.as_mut() else { return };
        let grown = panel.opened(now);
        let shade = shade(&mut self.draw);
        self.draw.picture(shade, DESIGN_W - SHADE_W, 0.0, SHADE_W, SHADE_H, grown);
        panel.draw(&mut self.draw, now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(program: &str, launch: &str) -> Vec<String> {
        vec![program.to_string(), launch.to_string()]
    }

    #[test]
    fn the_side_app_is_known_by_its_program_or_the_app_id_it_was_launched_as() {
        assert_eq!(side_of(&args("/apps/x/lumen-guide", "{}")), Some(Side::Guide));
        assert_eq!(side_of(&args("lumen-quick", "{}")), Some(Side::Quick));
        assert_eq!(side_of(&args("lumen", r#"{"appId":"com.samrenfrew.lumen.native.quick"}"#)), Some(Side::Quick));
        assert_eq!(side_of(&args("lumen", r#"{"appId":"com.samrenfrew.lumen.native.guide","parameters":{"view":"full"}}"#)), Some(Side::Guide));
        assert_eq!(side_of(&args("lumen", r#"{"appId":"com.samrenfrew.lumen.native"}"#)), None);
    }

    #[test]
    fn a_hidden_start_is_seen_in_the_launch_or_its_parameters() {
        assert!(asked_to_stay_hidden(&serde_json::json!({"parameters": {"hidden": true}})));
        assert!(asked_to_stay_hidden(&serde_json::json!({"hidden": true})));
        assert!(!asked_to_stay_hidden(&serde_json::json!({"parameters": {"view": "full"}})));
    }

    #[test]
    fn the_dark_corner_is_darkest_at_the_top_right_and_gone_towards_the_far_corner() {
        assert!((shade_alpha(SHADE_W, 0.0) - 0.6).abs() < 0.01);
        assert!(shade_alpha(SHADE_W - 200.0, 100.0) > 0.3);
        assert_eq!(shade_alpha(0.0, SHADE_H), 0.0);
    }
}
