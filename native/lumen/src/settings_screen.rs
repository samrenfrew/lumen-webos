use crate::apps::App;
use crate::draw::Draw;
use crate::home::Key;
use serde_json::{json, Value};

#[derive(Clone, PartialEq, Debug)]
pub enum Outcome {
    Stay,
    Close,
    Setting(String, Value),
    Toast(String, bool),
}

pub struct SettingsScreen {
    config: Value,
}

impl SettingsScreen {
    pub fn new(config: Value, _apps: Vec<App>, _now: u32) -> SettingsScreen {
        SettingsScreen { config }
    }

    pub fn key(&mut self, _d: &Draw, key: Key, _now: u32) -> Outcome {
        match key {
            Key::Back => Outcome::Close,
            _ => Outcome::Stay,
        }
    }

    pub fn draw(&mut self, d: &mut Draw, _now: u32) {
        d.copy_whole(d.veil_backdrop, 1.0, 1.0, 0.0, 0.0);
    }

    pub fn moving(&self, _now: u32) -> bool {
        false
    }

    pub fn where_now(&self) -> Value {
        json!({"screen": "settings", "focused": [], "keys": self.config.as_object().map_or(0, |c| c.len())})
    }
}
