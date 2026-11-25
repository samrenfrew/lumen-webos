use crate::draw::Decoded;
use crate::home::Key;
use crate::picker::Wants;
use crate::saver::{self, Saver, Setup};
use crate::settings::Settings;
use crate::{note, post, root, store, ticks, weather, App, SETTINGS_FILE};
use serde_json::Value;
use std::sync::Arc;

pub const SAVER_APP_ID: &str = "com.samrenfrew.lumen.native.saver";
const SAVER_PROGRAM: &str = "lumen-saver";
const SHOT_PHOTO: &str = "../../app/public/wallpapers/default-2.jpg";

pub fn runs_alone(args: &[String]) -> bool {
    let program = args.first().and_then(|a| a.rsplit('/').next()).unwrap_or("");
    let asked = args.get(1).is_some_and(|a| a == "--saver");
    let launched_as = args.get(1).and_then(|a| serde_json::from_str::<Value>(a).ok()).is_some_and(|l| l["appId"].as_str() == Some(SAVER_APP_ID));
    program == SAVER_PROGRAM || asked || launched_as
}

fn wants(settings: &Settings) -> Wants {
    Wants { immich: settings.saver_immich, sources: settings.saver_sources.clone(), recent_days: settings.immich_recent_days, picks: settings.wallpaper_picks.clone() }
}

pub fn wallpaper_wants(settings: &Settings) -> Wants {
    Wants { immich: true, sources: settings.immich_picks.join("|"), recent_days: settings.immich_recent_days, picks: Vec::new() }
}

impl App {
    fn saver_setup(&self, fetch: bool) -> Setup {
        Setup {
            app_dir: self.draw.app_dir.clone(),
            wants: wants(&self.settings),
            seconds: self.settings.saver_seconds,
            night: self.is_night(),
            reduce_motion: self.settings.reduce_motion,
            alone: self.alone,
            fetch,
        }
    }

    fn start_saver(&mut self, fetch: bool) {
        if let Some(mut old) = self.saver.take() {
            old.let_go(&mut self.draw);
        }
        let deliver: saver::Deliver = Arc::new(|id: u64, frame: Option<Decoded>| post(move |app: &mut App| app.saver_frame(id, frame)));
        let mut s = Saver::new(self.saver_setup(fetch), deliver, ticks());
        s.weather_line = self.settings.show_weather;
        s.forecast = self.settings.saver_forecast;
        s.weather = self.home.weather.clone();
        self.saver = Some(s);
        self.screen = None;
        self.panel = None;
        self.tick_clock();
        self.ask_for_weather();
        self.redraw = true;
    }

    pub(crate) fn open_saver(&mut self) {
        note!("saver: open{}", if self.alone { " as its own app" } else { "" });
        root::copy_immich_server(&format!("/tmp/lumen-native-{}", unsafe { crate::ffi::getuid() }), |copied| {
            if let Err(why) = copied {
                note!("saver: Immich server not copied: {}", why);
            }
        });
        self.start_saver(self.fetch_wallpapers);
    }

    pub(crate) fn open_saver_for_shot(&mut self) {
        self.start_saver(false);
        let photo = crate::draw::decoded(SHOT_PHOTO);
        let frame = photo.as_ref().and_then(|p| crate::compose::frame(&[crate::compose::Placed { picture: p, x: 0.5, y: 0.5 }], 1920, 1080, false));
        if let Some(s) = self.saver.as_mut() {
            s.arrived(&mut self.draw, frame, 0);
        }
    }

    pub(crate) fn start_alone(&mut self) {
        let own_copy = format!("{}/config/{SETTINGS_FILE}", self.draw.app_dir);
        let text = store::read(SETTINGS_FILE).or_else(|| std::fs::read_to_string(&own_copy).ok());
        if let Some(kept) = text.as_deref().and_then(Settings::parse) {
            self.settings = kept;
        }
        self.home.weather = weather::kept(&self.settings.weather_city);
        self.open_saver();
        root::copy_shared_file(SETTINGS_FILE, store::dir(), |copied| match copied {
            Ok(_) => post(|app| {
                let Some(fresh) = store::read(SETTINGS_FILE).as_deref().and_then(Settings::parse) else { return };
                if fresh != app.settings {
                    note!("saver: settings changed since the last copy");
                    app.settings = fresh;
                    app.ask_for_weather();
                }
            }),
            Err(why) => note!("saver: settings not copied: {}", why),
        });
    }

    pub(crate) fn saver_frame(&mut self, id: u64, frame: Option<Decoded>) {
        let Some(s) = self.saver.as_mut().filter(|s| s.id == id) else { return };
        s.arrived(&mut self.draw, frame, ticks());
        self.redraw = true;
    }

    pub(crate) fn close_saver(&mut self) {
        self.saver_at = None;
        self.home.wake(ticks());
        let Some(mut s) = self.saver.take() else { return };
        s.let_go(&mut self.draw);
        self.redraw = true;
        if self.alone {
            note!("saver: closed, so the app ends");
            self.running = false;
        }
    }

    pub(crate) fn saver_key(&mut self, _key: Key) {
        let Some(s) = self.saver.as_mut() else { return };
        self.redraw = true;
        if s.key(ticks()) == saver::Outcome::Close {
            self.close_saver();
        }
    }

    pub(crate) fn sleep_then_saver(&mut self) {
        let spot = saver::clock_spot(&mut self.draw, self.home.weather.as_ref(), self.settings.saver_forecast, self.home.unix_now, self.home.twenty_four_hour);
        let ms = self.home.go_to_sleep(spot, ticks());
        self.saver_at = Some(ticks() + ms);
    }

    pub(crate) fn tick_saver(&mut self, now: u32) -> Option<u32> {
        if let Some(at) = self.saver_at {
            if now < at {
                return Some(at - now);
            }
            self.saver_at = None;
            self.open_saver();
        }
        let s = self.saver.as_mut()?;
        if s.done(now) {
            self.close_saver();
            return None;
        }
        s.weather = self.home.weather.clone();
        if s.zoom_frame_due(now) {
            self.redraw = true;
        }
        s.tick(&mut self.draw, now)
    }

    pub(crate) fn draw_saver(&mut self, now: u32) {
        let Some(s) = self.saver.as_mut() else { return };
        let over_home = !self.alone && !s.covers(now);
        if over_home {
            self.home.draw(&mut self.draw, now);
        }
        s.draw(&mut self.draw, now, over_home);
    }

    pub(crate) fn saver_moving(&self, now: u32) -> Option<bool> {
        let s = self.saver.as_ref()?;
        Some(s.moving(now) || (!self.alone && !s.covers(now) && self.home.moving(now)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn the_same_program_starts_as_the_screensaver_when_packaged_or_launched_as_it() {
        assert!(runs_alone(&args(&["/media/developer/apps/usr/palm/applications/com.samrenfrew.lumen.native.saver/lumen-saver", "{}"])));
        assert!(runs_alone(&args(&["./lumen", r#"{"event":"launch","appId":"com.samrenfrew.lumen.native.saver","parameters":{}}"#])));
        assert!(runs_alone(&args(&["lumen", "--saver"])));
        assert!(!runs_alone(&args(&["/x/lumen", r#"{"event":"launch","appId":"com.samrenfrew.lumen.native","parameters":{"view":"ambient"}}"#])));
        assert!(!runs_alone(&args(&["lumen", "--shot", "a.png", "saver"])));
    }
}
