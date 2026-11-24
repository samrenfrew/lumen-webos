use crate::draw::Draw;
use crate::home::Key;
use serde_json::{json, Value};

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

pub struct ControlCenter {
    kind: Kind,
}

impl ControlCenter {
    pub fn new(kind: Kind, _clock_format: &str, _now: u32) -> ControlCenter {
        ControlCenter { kind }
    }

    pub fn key(&mut self, _d: &Draw, key: Key, _now: u32) -> Outcome {
        match key {
            Key::Back => Outcome::Close,
            _ => Outcome::Stay,
        }
    }

    pub fn draw(&mut self, _d: &mut Draw, _now: u32) {}

    pub fn moving(&self, _now: u32) -> bool {
        false
    }

    pub fn where_now(&self) -> Value {
        json!({"panel": if self.kind == Kind::Controls { "controls" } else { "inputs" }, "focused": []})
    }
}
