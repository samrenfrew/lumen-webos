use crate::control_center::ControlCenter;
use crate::draw::Draw;
use crate::ffi::*;
use crate::gfx::{Tex, DESIGN_W};
use crate::settings::Settings;
use crate::{guide, note, saver_mode, store, ticks, App, SETTINGS_FILE};
use serde_json::Value;

pub const QUICK_APP_ID: &str = "com.samrenfrew.lumen.native.quick";
const GUIDE_PROGRAM: &str = "lumen-guide";
const QUICK_PROGRAM: &str = "lumen-quick";
pub const OVERLAY_APP_ID: &str = "com.samrenfrew.lumen.native.overlay";
const OVERLAY_PROGRAM: &str = "lumen-overlay";
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Guide,
    Quick,
    Saver,
}

pub fn mode_of(args: &[String]) -> Option<Mode> {
    let program = args.first().and_then(|a| a.rsplit('/').next()).unwrap_or("");
    let first = args.get(1).map_or("", String::as_str);
    let launch = serde_json::from_str::<Value>(first).unwrap_or(Value::Null);
    let launched_as = launch["appId"].as_str();
    if launched_as == Some(crate::APP_ID) {
        return None;
    }
    let asked = match launch["parameters"]["mode"].as_str() {
        Some("guide") => Some(Mode::Guide),
        Some("quick") => Some(Mode::Quick),
        Some("saver") => Some(Mode::Saver),
        _ => None,
    };
    asked.or(match (program, launched_as) {
        (GUIDE_PROGRAM, _) | (_, Some(guide::GUIDE_APP_ID)) => Some(Mode::Guide),
        (QUICK_PROGRAM, _) | (_, Some(QUICK_APP_ID)) => Some(Mode::Quick),
        (saver_mode::SAVER_PROGRAM | OVERLAY_PROGRAM, _) | (_, Some(saver_mode::SAVER_APP_ID | OVERLAY_APP_ID)) => Some(Mode::Saver),
        _ if first == "--saver" => Some(Mode::Saver),
        _ => None,
    })
}

pub fn bus_name(args: &[String]) -> String {
    if let Some(id) = args.get(1).and_then(|a| crate::launched_as(a)) {
        return id;
    }
    match mode_of(args) {
        Some(Mode::Guide) => guide::GUIDE_APP_ID,
        Some(Mode::Quick) => QUICK_APP_ID,
        Some(Mode::Saver) => saver_mode::SAVER_APP_ID,
        None => crate::APP_ID,
    }
    .to_string()
}

pub fn side_of(args: &[String]) -> Option<Side> {
    match mode_of(args)? {
        Mode::Guide => Some(Side::Guide),
        Mode::Quick => Some(Side::Quick),
        Mode::Saver => None,
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
    fn the_mode_in_the_launch_parameters_wins_over_the_program_and_app_id() {
        let overlay = |mode: &str| args("lumen-overlay", &format!(r#"{{"appId":"com.samrenfrew.lumen.native.overlay","parameters":{{"mode":"{mode}"}}}}"#));
        assert_eq!(mode_of(&overlay("guide")), Some(Mode::Guide));
        assert_eq!(mode_of(&overlay("quick")), Some(Mode::Quick));
        assert_eq!(mode_of(&overlay("saver")), Some(Mode::Saver));
        assert_eq!(mode_of(&args("lumen-guide", r#"{"parameters":{"mode":"quick"}}"#)), Some(Mode::Quick));
        assert_eq!(mode_of(&args("lumen-guide", r#"{"parameters":{"mode":"nonsense"}}"#)), Some(Mode::Guide));
    }

    #[test]
    fn it_joins_the_bus_as_the_app_the_tv_launched_whatever_its_mode() {
        assert_eq!(bus_name(&args("lumen-guide", r#"{"appId":"com.samrenfrew.lumen.native.guide","parameters":{"mode":"quick"}}"#)), "com.samrenfrew.lumen.native.guide");
        assert_eq!(bus_name(&args("lumen-quick", "{}")), QUICK_APP_ID);
        assert_eq!(bus_name(&args("lumen", "--saver")), saver_mode::SAVER_APP_ID);
        assert_eq!(bus_name(&args("lumen", "--shot")), crate::APP_ID);
    }

    #[test]
    fn the_overlay_app_with_no_mode_is_the_screensaver() {
        assert_eq!(mode_of(&args("lumen-overlay", "{}")), Some(Mode::Saver));
        assert_eq!(mode_of(&args("lumen", r#"{"appId":"com.samrenfrew.lumen.native.overlay","parameters":{}}"#)), Some(Mode::Saver));
    }

    #[test]
    fn home_stays_home_whatever_mode_it_is_asked_for() {
        assert_eq!(mode_of(&args("lumen", r#"{"appId":"com.samrenfrew.lumen.native","parameters":{"mode":"quick"}}"#)), None);
        assert_eq!(mode_of(&args("lumen", "--shot")), None);
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
