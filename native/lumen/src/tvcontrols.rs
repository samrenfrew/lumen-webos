use crate::root;
use std::sync::Mutex;

const ON_TV: bool = cfg!(feature = "webos");

#[derive(Clone, PartialEq, Debug)]
pub struct Controls {
    pub picture: String,
    pub sound: String,
    pub backlight: i32,
    pub eye_comfort: bool,
}

static LAST: Mutex<Option<Controls>> = Mutex::new(None);

fn before_any_read() -> Controls {
    Controls { picture: "normal".into(), sound: "standard".into(), backlight: 100, eye_comfort: false }
}

pub fn picture_label(mode: &str) -> &str {
    match mode {
        "vivid" => "Vivid",
        "normal" => "Standard",
        "eco" => "Eco",
        "cinema" => "Cinema",
        "sports" => "Sports",
        "game" => "Game",
        "filmMaker" => "FILMMAKER",
        "expert1" => "Expert (bright room)",
        "expert2" => "Expert (dark room)",
        "hdrEffect" => "HDR Effect",
        "hdrStandard" => "HDR Standard",
        "hdrCinema" => "HDR Cinema",
        "hdrGame" => "HDR Game",
        other => other,
    }
}

pub fn sound_label(mode: &str) -> &str {
    match mode {
        "standard" => "Standard",
        "news" => "Clear Voice",
        "movie" => "Cinema",
        "music" => "Music",
        "sports" => "Sports",
        "game" => "Game Optimizer",
        "aiSound" => "AI Sound",
        "aiSoundPlus" => "AI Sound Pro",
        "custom" => "Custom",
        other => other,
    }
}

pub fn values() -> Controls {
    LAST.lock().ok().and_then(|last| last.clone()).unwrap_or_else(before_any_read)
}

pub fn patch(change: impl FnOnce(&mut Controls)) {
    if let Ok(mut last) = LAST.lock() {
        let mut now = last.clone().unwrap_or_else(before_any_read);
        change(&mut now);
        *last = Some(now);
    }
    crate::wake();
}

fn backlight_level(text: &str) -> Option<i32> {
    text.parse::<i32>().ok().filter(|level| (0..=100).contains(level))
}

pub fn read_all() {
    if !ON_TV {
        return;
    }
    root::get_setting("pictureMode", |v| {
        if let Some(mode) = v.ok().filter(|m| !m.is_empty()) {
            patch(|c| c.picture = mode);
        }
    });
    root::get_setting("soundMode", |v| {
        if let Some(mode) = v.ok().filter(|m| !m.is_empty()) {
            patch(|c| c.sound = mode);
        }
    });
    root::get_setting("eyeComfortMode", |v| {
        if let Ok(mode) = v {
            patch(|c| c.eye_comfort = mode == "on");
        }
    });
    read_backlight();
}

pub fn read_backlight() {
    if !ON_TV {
        return;
    }
    root::get_setting("backlight", |v| {
        if let Some(level) = v.ok().as_deref().and_then(backlight_level) {
            patch(|c| c.backlight = level);
        }
    });
}

fn set(key: &str, value: &str, then: impl FnOnce(Result<(), String>) + Send + 'static) {
    if ON_TV {
        root::set_setting(key, value, then);
    } else {
        then(Ok(()));
    }
}

pub fn set_picture_mode(mode: &str, then: impl FnOnce(Result<(), String>) + Send + 'static) {
    set("pictureMode", mode, then);
}

pub fn set_sound_mode(mode: &str, then: impl FnOnce(Result<(), String>) + Send + 'static) {
    set("soundMode", mode, then);
}

pub fn set_backlight(level: i32, then: impl FnOnce(Result<(), String>) + Send + 'static) {
    set("backlight", &level.to_string(), then);
}

pub fn set_eye_comfort(on: bool, then: impl FnOnce(Result<(), String>) + Send + 'static) {
    set("eyeComfortMode", if on { "on" } else { "off" }, then);
}

pub fn screen_off(then: impl FnOnce(Result<(), String>) + Send + 'static) {
    root::screen_off(then);
}
