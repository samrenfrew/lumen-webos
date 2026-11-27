use crate::home::{DockExit, Key, Shown};
use crate::side_apps::Side;
use crate::{apps, bus, note, post, root, store, ticks, App, LAUNCH, LAUNCH_POINTS, LAUNCH_POINTS_FILE};
use serde_json::{json, Value};

const DOCK_GLASS_RGB: [u8; 3] = [40, 44, 58];
const HAND_OFF_WAIT_MS: u32 = 3000;
const HAND_OFF_CLOSE_MS: u32 = 250;

pub fn opens_in_place(app: &Shown) -> bool {
    app.inside.is_none() && !app.id.starts_with("lumen:")
}

pub fn hand_off(col: usize, exit: DockExit) -> Value {
    let exit = match exit {
        DockExit::Bar => "bar",
        DockExit::Down => "down",
        DockExit::Up => "up",
        DockExit::Open => "open",
    };
    json!({"view": "dock", "col": col, "exit": exit})
}

pub fn arrival(parameters: &Value) -> Option<(usize, DockExit)> {
    if parameters["view"].as_str() != Some("dock") {
        return None;
    }
    let col = parameters["col"].as_u64().unwrap_or(0) as usize;
    let exit = match parameters["exit"].as_str() {
        Some("down") => DockExit::Down,
        Some("up") => DockExit::Up,
        Some("open") => DockExit::Open,
        _ => DockExit::Bar,
    };
    Some((col, exit))
}

impl App {
    pub(crate) fn open_dock(&mut self, app: Option<&str>) {
        self.known = apps::catalogue(&std::fs::read_to_string(format!("{}/assets/catalog.tsv", self.draw.app_dir)).unwrap_or_default());
        self.launch_points = store::read(LAUNCH_POINTS_FILE).and_then(|text| serde_json::from_str(&text).ok()).unwrap_or(Value::Null);
        if self.launch_points.is_null() {
            bus::call(LAUNCH_POINTS, json!({}), 8000, |reply| match reply {
                Ok(points) => post(move |app| app.launch_points_arrived(points)),
                Err(why) => note!("dock: apps: {}", why),
            });
        }
        self.show_apps();
        let glass = DOCK_GLASS_RGB.iter().chain(&[255]).copied().cycle().take(4 * 4 * 4).collect::<Vec<u8>>();
        let tint = self.draw.gfx.upload_rgba(&glass, 4, 4);
        self.home.open_as_dock(&mut self.draw, tint, app, ticks());
        note!("dock: open over {}", app.unwrap_or("an unknown app"));
        if self.settings.show_recents {
            root::running_apps(|found| match found {
                Ok(ids) => post(move |app| {
                    app.home.open_apps = ids;
                    app.redraw = true;
                }),
                Err(why) => note!("dock: open apps: {}", why),
            });
        }
    }

    pub(crate) fn open_dock_for_shot(&mut self) {
        self.side = Some(Side::Dock);
        self.open_dock(None);
    }

    pub(crate) fn dock_key(&mut self, key: Key) {
        self.redraw = true;
        let now = ticks();
        match key {
            Key::Back => self.close_side(),
            Key::Down => self.leave_dock(DockExit::Down),
            Key::Up => self.leave_dock(DockExit::Up),
            Key::Ok => self.dock_ok(),
            Key::Digit(n) => {
                if n >= 1 && self.home.dock_select(&self.draw, n - 1, now) {
                    self.dock_ok();
                }
            }
            Key::Left | Key::Right => {
                self.home.key(&self.draw, key, now);
            }
        }
    }

    fn dock_ok(&mut self) {
        let Some((_, shown)) = self.home.dock_focus() else { return self.close_side() };
        if !opens_in_place(shown) {
            return self.leave_dock(DockExit::Open);
        }
        let id = shown.id.clone();
        self.open(id);
        self.close_side();
    }

    pub(crate) fn leave_dock(&mut self, exit: DockExit) {
        if self.quit_at.is_some() {
            return;
        }
        let col = self.home.dock_focus().map_or(0, |(at, _)| at);
        let params = hand_off(col, exit);
        note!("dock: home opens, {}", params);
        self.quit_at = Some(ticks().wrapping_add(HAND_OFF_WAIT_MS));
        bus::call(LAUNCH, json!({"id": crate::APP_ID, "params": params}), 6000, |reply| {
            if let Err(why) = &reply {
                note!("dock: home did not open: {}", why);
            }
            post(|app| app.quit_at = Some(ticks().wrapping_add(HAND_OFF_CLOSE_MS)));
        });
    }

    pub(crate) fn arrive_from_dock(&mut self, col: usize, exit: DockExit) {
        note!("dock: arrived at column {} ({:?})", col, exit);
        self.close_saver();
        self.screen = None;
        self.panel = None;
        self.close_lab();
        if self.guide.is_some() && !self.over_live_tv {
            self.close_guide();
        }
        if self.hidden && exit != DockExit::Bar {
            self.home.from_dock(&self.draw, col, DockExit::Bar, ticks());
            self.dock_exit_waiting = Some(exit);
        } else if let Some(action) = self.home.from_dock(&self.draw, col, exit, ticks()) {
            self.act(action);
        }
        self.redraw = true;
    }

    pub(crate) fn dock_exit_after_frame(&mut self) {
        if self.hidden {
            return;
        }
        let Some(exit) = self.dock_exit_waiting.take() else { return };
        let col = self.home.dock_focus().map_or(0, |(at, _)| at);
        if let Some(action) = self.home.from_dock(&self.draw, col, exit, ticks()) {
            self.act(action);
        }
        self.redraw = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_opens_where_the_dock_was_left_and_goes_on_from_there() {
        for (col, exit) in [(0, DockExit::Bar), (3, DockExit::Down), (2, DockExit::Up), (5, DockExit::Open)] {
            assert_eq!(arrival(&hand_off(col, exit)), Some((col, exit)));
        }
        assert_eq!(arrival(&json!({"view": "dock"})), Some((0, DockExit::Bar)));
        assert_eq!(arrival(&json!({"view": "ambient", "col": 2})), None);
    }

    #[test]
    fn folders_and_lumens_own_tiles_open_on_home_and_apps_open_over_the_dock() {
        let app = |id: &str| Shown { id: id.to_string(), title: String::new(), icon_file: None, glyph: None, color: None, inside: None };
        assert!(opens_in_place(&app("youtube.leanback.v4")));
        assert!(!opens_in_place(&app(apps::SETTINGS_TILE)));
        assert!(!opens_in_place(&app(apps::INPUTS_TILE)));
        assert!(!opens_in_place(&Shown { inside: Some(vec![app("x")]), ..app("folder:Games") }));
    }
}
