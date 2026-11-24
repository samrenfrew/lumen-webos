use crate::{bus, note, post, App};
use serde_json::{json, Value};

const BROADCAST: &str = "luna://com.webos.service.utp.broadcast";
const MUTES: &str = "luna://com.webos.service.avmutemanager/listMuteStatus";
const APP_ID: &str = "com.samrenfrew.lumen.native";
const LIVE_TV_APP_ID: &str = "com.webos.app.livetv";
pub const DWELL_MS: u32 = 600;
const SETTLE_MS: u32 = 4000;
const POLL_MS: u32 = 100;

#[derive(Clone, PartialEq, Debug)]
pub struct Wanted {
    pub channel: String,
    pub rect: [f32; 4],
    pub sound: bool,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    Idle,
    Dwelling,
    Tuning,
    Muted,
    Playing,
    NoPicture,
}

#[derive(PartialEq, Debug)]
pub enum Step {
    Nothing,
    Open,
    Tune(String),
    AskMutes,
    Close(String),
}

pub struct Preview {
    wanted: Option<Wanted>,
    phase: Phase,
    since: u32,
    tuned_at: u32,
    next_poll: u32,
    broadcast: Option<String>,
    opening: bool,
    generation: u32,
}

impl Preview {
    pub fn new() -> Preview {
        Preview { wanted: None, phase: Phase::Idle, since: 0, tuned_at: 0, next_poll: 0, broadcast: None, opening: false, generation: 0 }
    }

    pub fn want(&mut self, wanted: Option<Wanted>, now: u32) -> bool {
        let same_channel = self.wanted.as_ref().map(|w| &w.channel) == wanted.as_ref().map(|w| &w.channel);
        let moved = self.wanted.as_ref().map(|w| w.rect) != wanted.as_ref().map(|w| w.rect);
        self.wanted = wanted;
        if !same_channel {
            self.generation += 1;
            self.since = now;
            self.phase = if self.wanted.is_some() { Phase::Dwelling } else { Phase::Idle };
            return true;
        }
        moved && self.phase != Phase::Dwelling
    }

    pub fn wanted(&self) -> Option<&Wanted> {
        self.wanted.as_ref()
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn hole(&self) -> Option<[f32; 4]> {
        (self.phase == Phase::Playing).then(|| self.wanted.as_ref().map(|w| w.rect)).flatten()
    }

    pub fn loading(&self) -> Option<&str> {
        let waiting = matches!(self.phase, Phase::Tuning | Phase::Muted) || (self.phase == Phase::Dwelling && self.opening);
        self.wanted.as_ref().filter(|_| waiting).map(|w| w.channel.as_str())
    }

    pub fn shows(&self, channel: &str) -> bool {
        self.phase == Phase::Playing && self.wanted.as_ref().is_some_and(|w| w.channel == channel)
    }

    pub fn step(&mut self, now: u32) -> Step {
        let Some(wanted) = self.wanted.clone() else {
            return match self.broadcast.take() {
                Some(id) => Step::Close(id),
                None => Step::Nothing,
            };
        };
        match self.phase {
            Phase::Dwelling if now.wrapping_sub(self.since) >= DWELL_MS => match (&self.broadcast, self.opening) {
                (Some(_), _) => {
                    self.phase = Phase::Tuning;
                    self.tuned_at = now;
                    Step::Tune(wanted.channel)
                }
                (None, false) => {
                    self.opening = true;
                    Step::Open
                }
                (None, true) => Step::Nothing,
            },
            Phase::Muted if now.wrapping_sub(self.tuned_at) >= SETTLE_MS => {
                self.phase = Phase::Playing;
                Step::Nothing
            }
            Phase::Muted if now.wrapping_sub(self.next_poll) < u32::MAX / 2 => {
                self.next_poll = now + POLL_MS;
                Step::AskMutes
            }
            _ => Step::Nothing,
        }
    }

    pub fn opened(&mut self, id: Option<String>) {
        self.opening = false;
        match id {
            Some(id) => self.broadcast = Some(id),
            None => self.phase = Phase::NoPicture,
        }
    }

    pub fn tuned(&mut self, generation: u32, ok: bool, now: u32) {
        if generation != self.generation || self.phase != Phase::Tuning {
            return;
        }
        self.phase = if ok { Phase::Muted } else { Phase::NoPicture };
        self.next_poll = now;
    }

    pub fn mutes_read(&mut self, generation: u32, picture_shown: bool) {
        if generation == self.generation && self.phase == Phase::Muted && picture_shown {
            self.phase = Phase::Playing;
        }
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn wait_ms(&self, now: u32) -> Option<u32> {
        match self.phase {
            Phase::Dwelling => Some((self.since + DWELL_MS).wrapping_sub(now).min(DWELL_MS).max(1)),
            Phase::Muted => Some(self.next_poll.wrapping_sub(now).min(POLL_MS).max(1)),
            _ => None,
        }
    }

    pub fn closing_id(&mut self) -> Option<String> {
        self.wanted = None;
        self.phase = Phase::Idle;
        self.broadcast.take()
    }
}

pub fn picture_shown(reply: &Value) -> bool {
    reply["videoMute"].as_array().is_some_and(|list| !list.is_empty() && list.iter().all(|m| m["videoMuteStatus"].as_bool() == Some(false)))
}

pub fn open_over_home(window_id: &str, sound: bool) {
    let window_id = window_id.to_string();
    bus::call("luna://com.webos.applicationManager/closeByAppId", json!({"id": LIVE_TV_APP_ID}), 3000, move |reply| {
        if reply.is_ok() {
            note!("preview: closed Live TV in the background, as its muted picture hid the preview");
        }
        open(&window_id, sound);
    });
}

pub fn open(window_id: &str, sound: bool) {
    let options = json!({"options": {"appId": APP_ID, "updateLastChannelId": false, "windowId": window_id}});
    bus::call(&format!("{BROADCAST}/open"), options, 4000, move |reply| {
        let id = reply.as_ref().ok().and_then(|r| r["broadcastId"].as_str().map(String::from));
        if id.is_none() {
            note!("preview: open: {:?}", reply);
        }
        if let Some(id) = &id {
            bus::call(&format!("{BROADCAST}/setAudioMute"), json!({"broadcastId": id, "isMuteOn": !sound, "muteOwner": APP_ID}), 3000, |r| {
                if let Err(why) = r {
                    note!("preview: mute: {}", why);
                }
            });
        }
        post(move |app| {
            app.preview.opened(id);
            app.redraw = true;
        });
    });
}

pub fn tune(broadcast: &str, channel: &str, generation: u32) {
    bus::call(&format!("{BROADCAST}/changeChannel"), json!({"broadcastId": broadcast, "channelId": channel}), 6000, move |reply| {
        let ok = reply.as_ref().is_ok_and(|r| r["returnValue"].as_bool() != Some(false));
        if !ok {
            note!("preview: tune: {:?}", reply);
        }
        post(move |app| {
            app.preview.tuned(generation, ok, crate::ticks());
            app.redraw = true;
        });
    });
}

pub fn ask_mutes(generation: u32) {
    bus::call(MUTES, json!({}), 1000, move |reply| {
        let shown = reply.as_ref().is_ok_and(picture_shown);
        post(move |app| {
            app.preview.mutes_read(generation, shown);
            app.redraw = true;
        });
    });
}

pub fn close(broadcast: &str) {
    bus::call(&format!("{BROADCAST}/close"), json!({"broadcastId": broadcast}), 3000, |reply| {
        if let Err(why) = reply {
            note!("preview: close: {}", why);
        }
    });
}

impl App {
    pub fn preview_tick(&mut self, now: u32) {
        let wanted = if self.hidden || self.saver.is_some() || self.screen.is_some() || self.panel.is_some() {
            None
        } else if let Some(guide) = self.guide.as_ref() {
            self.settings.guide_preview.then(|| guide.preview_box(&self.live)).flatten()
        } else if self.settings.home_preview {
            self.home.live_preview_box()
        } else {
            None
        };
        let sound = self.settings.preview_sound;
        if self.preview.want(wanted.map(|(channel, rect)| Wanted { channel, rect, sound }), now) {
            self.redraw = true;
        }
        let showing = self.preview.hole().and(self.preview.wanted().map(|w| w.channel.clone()));
        if self.home.preview_showing != showing {
            self.home.preview_showing = showing.clone();
            self.redraw = true;
        }
        let loading = self.preview.loading().map(String::from);
        if self.home.preview_loading != loading {
            self.home.preview_loading = loading.clone();
            self.redraw = true;
        }
        let guide_preview = self.settings.guide_preview.then_some(showing.is_some());
        if let Some(guide) = self.guide.as_mut() {
            if guide.preview != guide_preview || guide.preview_loading != loading.is_some() {
                guide.preview = guide_preview;
                guide.preview_loading = loading.is_some();
                self.redraw = true;
            }
        }
        if let Some(rect) = self.preview.wanted().map(|w| w.rect) {
            self.place_video(rect);
        }
        match self.preview.step(now) {
            Step::Nothing => {}
            Step::Open => {
                let window = self.video_window_id();
                if window.is_empty() {
                    note!("preview: the window could not be exported for video");
                    self.preview.opened(None);
                } else if self.guide.is_some() {
                    open(&window, sound);
                } else {
                    open_over_home(&window, sound);
                }
            }
            Step::Tune(channel) => {
                if let Some(id) = self.preview.broadcast_id() {
                    tune(&id, &channel, self.preview.generation());
                }
            }
            Step::AskMutes => ask_mutes(self.preview.generation()),
            Step::Close(id) => close(&id),
        }
    }

    pub fn close_preview_now(&mut self) {
        if let Some(id) = self.preview.closing_id() {
            close(&id);
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
    }

    #[cfg(feature = "webos")]
    fn video_window_id(&mut self) -> String {
        let id = unsafe { crate::ffi::lumen_export_video_window(self.window) };
        if id.is_null() {
            return String::new();
        }
        unsafe { std::ffi::CStr::from_ptr(id) }.to_string_lossy().into_owned()
    }

    #[cfg(not(feature = "webos"))]
    fn video_window_id(&mut self) -> String {
        String::new()
    }

    #[cfg(feature = "webos")]
    fn place_video(&mut self, rect: [f32; 4]) {
        let k = self.draw.gfx.screen_w as f32 / crate::DESIGN_W;
        let at = rect.map(|v| (v * k).round() as i32);
        if self.placed_video != Some(at) {
            let (w, h) = (self.draw.gfx.screen_w, (self.draw.gfx.screen_w as f32 * 9.0 / 16.0) as i32);
            if unsafe { crate::ffi::lumen_place_video(w, h, at[0], at[1], at[2], at[3]) } == 1 {
                self.placed_video = Some(at);
                note!("preview: placed at {:?}", at);
            }
        }
    }

    #[cfg(not(feature = "webos"))]
    fn place_video(&mut self, _rect: [f32; 4]) {}
}

impl Preview {
    pub fn broadcast_id(&self) -> Option<String> {
        self.broadcast.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bbc(rect: [f32; 4]) -> Option<Wanted> {
        Some(Wanted { channel: "1_41_101".into(), rect, sound: false })
    }

    #[test]
    fn flicking_past_a_channel_never_tunes_it_and_resting_on_one_opens_the_tuner_once() {
        let mut p = Preview::new();
        p.want(bbc([0.0; 4]), 0);
        assert_eq!(p.step(DWELL_MS - 1), Step::Nothing);
        p.want(Some(Wanted { channel: "1_41_102".into(), rect: [0.0; 4], sound: false }), DWELL_MS - 1);
        assert_eq!(p.step(DWELL_MS + 100), Step::Nothing);
        assert_eq!(p.step(2 * DWELL_MS), Step::Open);
        assert_eq!(p.step(2 * DWELL_MS + 10), Step::Nothing);
        p.opened(Some("b1".into()));
        assert_eq!(p.step(2 * DWELL_MS + 20), Step::Tune("1_41_102".into()));
    }

    #[test]
    fn the_spinner_shows_from_the_end_of_the_dwell_until_the_picture_shows_and_not_while_flicking_past() {
        let mut p = Preview::new();
        p.want(bbc([0.0; 4]), 0);
        assert_eq!(p.loading(), None);
        p.step(DWELL_MS);
        assert!(p.loading().is_some());
        p.opened(Some("b1".into()));
        p.step(DWELL_MS);
        p.tuned(p.generation(), true, 1000);
        assert!(p.loading().is_some());
        p.mutes_read(p.generation(), true);
        assert_eq!(p.loading(), None);
        let mut lost = Preview::new();
        lost.want(bbc([0.0; 4]), 0);
        lost.step(DWELL_MS);
        lost.opened(None);
        assert_eq!(lost.loading(), None);
    }

    #[test]
    fn the_hole_opens_only_once_the_tv_unmutes_the_picture_or_after_it_has_had_long_enough() {
        let mut p = Preview::new();
        p.want(bbc([10.0, 20.0, 300.0, 170.0]), 0);
        p.step(DWELL_MS);
        p.opened(Some("b1".into()));
        p.step(DWELL_MS);
        p.tuned(p.generation(), true, 1000);
        assert_eq!(p.hole(), None);
        assert_eq!(p.step(1000), Step::AskMutes);
        p.mutes_read(p.generation(), false);
        assert_eq!(p.hole(), None);
        p.mutes_read(p.generation(), true);
        assert_eq!(p.hole(), Some([10.0, 20.0, 300.0, 170.0]));
        let mut slow = Preview::new();
        slow.want(bbc([0.0; 4]), 0);
        slow.step(DWELL_MS);
        slow.opened(Some("b1".into()));
        slow.step(DWELL_MS);
        slow.tuned(slow.generation(), true, 1000);
        slow.step(1000 + SETTLE_MS);
        assert_eq!(slow.phase(), Phase::Playing);
    }

    #[test]
    fn a_tune_answered_after_focus_moved_on_is_ignored_and_leaving_closes_the_tuner() {
        let mut p = Preview::new();
        p.want(bbc([0.0; 4]), 0);
        p.step(DWELL_MS);
        p.opened(Some("b1".into()));
        p.step(DWELL_MS);
        let old = p.generation();
        p.want(Some(Wanted { channel: "1_41_102".into(), rect: [0.0; 4], sound: false }), 700);
        p.tuned(old, true, 800);
        assert_eq!(p.phase(), Phase::Dwelling);
        p.want(None, 900);
        assert_eq!(p.step(900), Step::Close("b1".into()));
        assert_eq!(p.step(950), Step::Nothing);
    }

    #[test]
    fn the_picture_counts_as_shown_only_when_every_video_mute_is_off() {
        assert!(picture_shown(&json!({"videoMute": [{"videoMuteStatus": false}]})));
        assert!(!picture_shown(&json!({"videoMute": [{"videoMuteStatus": false}, {"videoMuteStatus": true}]})));
        assert!(!picture_shown(&json!({"videoMute": []})));
    }
}
