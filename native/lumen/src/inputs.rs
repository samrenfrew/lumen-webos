use crate::bus;
use serde_json::{json, Value};
use std::sync::Mutex;

const INPUT_STATUS: &str = "luna://com.webos.service.eim/getAllInputStatus";
const APP_LOAD_STATUS: &str = "luna://com.webos.applicationManager/getAppLoadStatus";
const LIVE_TV_APP: &str = "com.webos.app.livetv";
const LIVE_TV_ICON: &str = "tv.png";
const STAND_IN_ICON: &str = "hdmigeneric.png";

#[derive(Clone, PartialEq, Debug)]
pub struct Input {
    pub app_id: String,
    pub label: String,
    pub icon: String,
}

static LAST: Mutex<Option<Vec<Input>>> = Mutex::new(None);

fn plain_icon_name(name: &str) -> bool {
    name.ends_with(".png") && !name.starts_with('.') && name.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
}

pub fn from_tv(input_status: &Value, has_tuner: bool) -> Vec<Input> {
    let mut devices: Vec<&Value> = input_status["devices"].as_array().map(|d| d.iter().collect()).unwrap_or_default();
    devices.retain(|d| d["appId"].as_str().is_some_and(|s| !s.is_empty()) && d["label"].as_str().is_some_and(|s| !s.is_empty()));
    devices.sort_by_key(|d| d["port"].as_i64().unwrap_or(0));
    let live_tv = has_tuner.then(|| Input { app_id: LIVE_TV_APP.into(), label: "Live TV".into(), icon: LIVE_TV_ICON.into() });
    live_tv
        .into_iter()
        .chain(devices.into_iter().map(|d| Input {
            app_id: d["appId"].as_str().unwrap_or("").to_string(),
            label: d["label"].as_str().unwrap_or("").to_string(),
            icon: d["icon"].as_str().filter(|name| plain_icon_name(name)).unwrap_or(STAND_IN_ICON).to_string(),
        }))
        .collect()
}

pub fn chosen(all: &[Input], ids: &[String]) -> Vec<Input> {
    if ids.is_empty() {
        return all.to_vec();
    }
    ids.iter().filter_map(|id| all.iter().find(|input| &input.app_id == id).cloned()).collect()
}

pub fn last_read() -> Option<Vec<Input>> {
    LAST.lock().ok().and_then(|last| last.clone())
}

pub fn read(then: impl FnOnce(Result<Vec<Input>, String>) + Send + 'static) {
    bus::call(APP_LOAD_STATUS, json!({"appId": LIVE_TV_APP}), 4000, move |tuner| {
        let has_tuner = tuner.is_ok_and(|r| r["exist"].as_bool() == Some(true));
        bus::call(INPUT_STATUS, json!({}), 4000, move |reply| {
            let read = reply.map(|status| from_tv(&status, has_tuner));
            if let (Ok(list), Ok(mut last)) = (&read, LAST.lock()) {
                *last = Some(list.clone());
            }
            then(read);
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_tv_comes_first_then_the_inputs_by_port_with_a_stand_in_icon_for_an_odd_one() {
        let status = json!({"returnValue": true, "devices": [
            {"id": "HDMI_3", "label": "HDMI 3", "port": 3, "appId": "com.webos.app.hdmi3", "icon": "HDMI_3.png"},
            {"id": "HDMI_1", "label": "Chromecast", "port": 1, "appId": "com.webos.app.hdmi1", "icon": "HDMI_1.png"},
            {"id": "HDMI_2", "label": "Odd", "port": 2, "appId": "com.webos.app.hdmi2", "icon": "../../x.png"},
            {"id": "AV_1", "label": "", "port": 4, "appId": "com.webos.app.av1"}
        ]});
        let read = from_tv(&status, true);
        let names: Vec<(&str, &str)> = read.iter().map(|i| (i.label.as_str(), i.icon.as_str())).collect();
        assert_eq!(names, [("Live TV", "tv.png"), ("Chromecast", "HDMI_1.png"), ("Odd", "hdmigeneric.png"), ("HDMI 3", "HDMI_3.png")]);
        assert_eq!(from_tv(&status, false)[0].label, "Chromecast");
    }

    #[test]
    fn the_inputs_chosen_in_settings_show_in_their_order_and_all_show_when_none_are_chosen() {
        let all = from_tv(&json!({"devices": [
            {"label": "One", "port": 1, "appId": "com.webos.app.hdmi1"},
            {"label": "Two", "port": 2, "appId": "com.webos.app.hdmi2"}
        ]}), true);
        let ids = |list: Vec<Input>| list.into_iter().map(|i| i.app_id).collect::<Vec<_>>();
        assert_eq!(ids(chosen(&all, &[])), ["com.webos.app.livetv", "com.webos.app.hdmi1", "com.webos.app.hdmi2"]);
        assert_eq!(ids(chosen(&all, &["com.webos.app.hdmi2".into(), "gone".into(), "com.webos.app.livetv".into()])), ["com.webos.app.hdmi2", "com.webos.app.livetv"]);
    }
}
