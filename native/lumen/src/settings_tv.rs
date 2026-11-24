use crate::{bus, root};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

const INPUT_STATUS: &str = "luna://com.webos.service.eim/getAllInputStatus";
const APP_LOAD_STATUS: &str = "luna://com.webos.applicationManager/getAppLoadStatus";
const LIVE_TV: &str = "com.webos.app.livetv";
const SIDE_LISTING: &str = "https://raw.githubusercontent.com/samrenfrew/lumen-webos/hbc-repo/stable/apps.json";
const FALLBACK_INPUT_ICON: &str = "hdmigeneric.png";

#[derive(Clone, Default, PartialEq, Debug)]
pub struct TvState {
    pub scripts: String,
    pub home: String,
    pub native: String,
    pub power_on: String,
    pub power_on_support: String,
    pub power_on_loaded: String,
    pub agreements: Option<i64>,
    pub backup: bool,
    pub lmt: String,
    pub live_plus: String,
    pub hosts: bool,
    pub hosts_writable: bool,
    pub services: bool,
    pub services_running: i64,
    pub updates: bool,
    pub node: bool,
}

pub fn parse_tv_state(text: &str) -> TvState {
    let field = |name: &str| text.lines().filter_map(|line| line.split_once('=')).find(|(key, _)| key.trim() == name).map_or(String::new(), |(_, value)| value.trim().to_string());
    let number = |name: &str| field(name).parse::<i64>().ok();
    TvState {
        scripts: field("scripts"),
        home: field("home"),
        native: field("native"),
        power_on: field("poweron"),
        power_on_support: field("poweron_support"),
        power_on_loaded: Some(field("poweron_loaded")).filter(|v| !v.is_empty()).unwrap_or_else(|| "unknown".into()),
        agreements: number("agreements"),
        backup: field("backup") == "yes",
        lmt: field("lmt"),
        live_plus: field("liveplus").to_lowercase(),
        hosts: field("hosts") == "on",
        hosts_writable: field("hosts_writable") == "yes",
        services: field("services") == "on",
        services_running: number("services_running").unwrap_or(0),
        updates: field("updates") == "on",
        node: field("node") == "yes",
    }
}

impl TvState {
    pub fn scripts_ready(&self) -> bool {
        self.scripts == "ok"
    }

    pub fn power_on_pending(&self) -> bool {
        self.power_on_loaded != "unknown" && self.power_on != "foreign" && self.power_on != self.power_on_loaded
    }

    pub fn native_why(&self) -> &'static str {
        match self.native.as_str() {
            "ok" => "",
            "no-lumen" => "No Lumen build is installed where the TV scripts look for it.",
            "no-block" => "This TV's app manager cannot be made to read a replacement Home app, so this cannot work here.",
            "no-lghome" => "LG Home's app folder is not where the TV scripts look for it. Left alone.",
            _ => "The TV did not say.",
        }
    }

    pub fn power_on_why(&self) -> &'static str {
        if self.power_on == "foreign" {
            return "Another tool has written the start-up file. Lumen leaves it alone.";
        }
        match self.power_on_support.as_str() {
            "ok" => "",
            "no-bootd" => "This TV's start-up service does not read a replacement file, so this cannot work here.",
            "no-table" => "This TV's start-up file has no table of power-on apps, so this cannot work here.",
            "no-node" => "This TV has no node to write the start-up file safely.",
            _ => "The TV did not say.",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Release {
    pub version: String,
    pub url: String,
    pub sha256: String,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct SideApps {
    pub installed: Vec<(String, String)>,
    pub latest: Option<Vec<(String, Release)>>,
    pub offline: bool,
}

pub fn side_title(id: &str) -> &'static str {
    match id {
        "com.samrenfrew.lumen.guide" => "Lumen Guide",
        "com.samrenfrew.lumen.quick" => "Lumen Quick Settings",
        _ => "Lumen Screensaver",
    }
}

pub fn compare_versions(a: &str, b: &str) -> Ordering {
    let parts = |v: &str| v.split('.').map(|p| p.parse::<u64>().unwrap_or(0)).collect::<Vec<_>>();
    let (x, y) = (parts(a), parts(b));
    (0..x.len().max(y.len())).map(|i| x.get(i).unwrap_or(&0).cmp(y.get(i).unwrap_or(&0))).find(|o| *o != Ordering::Equal).unwrap_or(Ordering::Equal)
}

pub fn releases_in(listing: &Value) -> Vec<(String, Release)> {
    listing["packages"]
        .as_array()
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .filter_map(|p| {
            let id = p["id"].as_str().filter(|id| root::SIDE_APPS.contains(id))?;
            let m = &p["manifest"];
            let release = Release { version: m["version"].as_str()?.into(), url: m["ipkUrl"].as_str()?.into(), sha256: m["ipkHash"]["sha256"].as_str()?.into() };
            Some((id.to_string(), release))
        })
        .collect()
}

impl SideApps {
    pub fn installed(&self, id: &str) -> Option<&str> {
        self.installed.iter().find(|(i, _)| i == id).map(|(_, v)| v.as_str())
    }

    pub fn latest(&self, id: &str) -> Option<&Release> {
        self.latest.as_ref()?.iter().find(|(i, _)| i == id).map(|(_, r)| r)
    }

    pub fn behind(&self) -> Vec<&'static str> {
        root::SIDE_APPS.into_iter().filter(|id| matches!((self.installed(id), self.latest(id)), (Some(have), Some(next)) if compare_versions(have, &next.version) == Ordering::Less)).collect()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Input {
    pub app_id: String,
    pub label: String,
    pub icon: String,
}

pub fn inputs_from(reply: &Value, has_tuner: bool) -> Vec<Input> {
    let mut devices: Vec<&Value> = reply["devices"].as_array().map_or(Vec::new(), |d| d.iter().collect());
    devices.sort_by_key(|d| d["port"].as_i64().unwrap_or(0));
    let tuner = has_tuner.then(|| Input { app_id: LIVE_TV.into(), label: "Live TV".into(), icon: "tv.png".into() });
    tuner
        .into_iter()
        .chain(devices.into_iter().filter_map(|d| {
            let (app_id, label) = (d["appId"].as_str().filter(|s| !s.is_empty())?, d["label"].as_str().filter(|s| !s.is_empty())?);
            let icon = d["icon"].as_str().filter(|s| !s.is_empty() && !s.contains('/')).unwrap_or(FALLBACK_INPUT_ICON);
            Some(Input { app_id: app_id.into(), label: label.into(), icon: icon.into() })
        }))
        .collect()
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub enum Test {
    #[default]
    Idle,
    Testing,
    Failed,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Immich {
    pub ready: bool,
    pub albums: Vec<(String, String)>,
    pub url: String,
    pub key: String,
    pub test: Test,
}

pub fn tidy_address(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    let bare = trimmed.strip_suffix("/api").unwrap_or(trimmed);
    if bare.is_empty() || bare.starts_with("http://") || bare.starts_with("https://") {
        bare.to_string()
    } else {
        format!("http://{bare}")
    }
}

pub fn albums_in(text: &str) -> Option<Vec<(String, String)>> {
    let list: Value = serde_json::from_str(text).ok()?;
    Some(
        list.as_array()?
            .iter()
            .filter(|a| a["assetCount"].as_u64().unwrap_or(0) > 0)
            .filter_map(|a| Some((a["id"].as_str()?.to_string(), a["albumName"].as_str()?.to_string())))
            .collect(),
    )
}

#[derive(Default)]
pub struct Live {
    pub tv: Option<TvState>,
    pub tv_failed: bool,
    pub busy: String,
    pub side: Option<SideApps>,
    pub inputs: Option<Vec<Input>>,
    pub immich: Immich,
    pub changed: u32,
}

pub type Shared = Arc<Mutex<Live>>;

fn change(shared: &Shared, edit: impl FnOnce(&mut Live)) {
    if let Ok(mut live) = shared.lock() {
        edit(&mut live);
        live.changed = live.changed.wrapping_add(1);
    }
    crate::wake();
}

pub fn read_tv(shared: &Shared, app_dir: &str) {
    let to = shared.clone();
    root::tv_status(app_dir, move |out| {
        change(&to, |live| match out {
            Ok(text) => {
                live.tv = Some(parse_tv_state(&text));
                live.tv_failed = false;
            }
            Err(_) => live.tv_failed = true,
        });
    });
}

pub fn read_side(shared: &Shared, online: bool) {
    let to = shared.clone();
    root::side_app_versions(move |found| {
        change(&to, |live| {
            let side = live.side.get_or_insert_with(SideApps::default);
            side.installed = found.unwrap_or_default();
        });
    });
    if !online {
        return;
    }
    let to = shared.clone();
    std::thread::spawn(move || {
        let listing = crate::util::http_get(SIDE_LISTING).ok().and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
        change(&to, |live| {
            let side = live.side.get_or_insert_with(SideApps::default);
            side.offline = listing.is_none();
            side.latest = listing.map(|l| releases_in(&l));
        });
    });
}

pub fn read_inputs(shared: &Shared) {
    let to = shared.clone();
    bus::call(APP_LOAD_STATUS, json!({"appId": LIVE_TV}), 4000, move |tuner| {
        let has_tuner = tuner.is_ok_and(|r| r["exist"].as_bool() == Some(true));
        bus::call(INPUT_STATUS, json!({}), 4000, move |reply| {
            change(&to, |live| live.inputs = Some(reply.map(|r| inputs_from(&r, has_tuner)).unwrap_or_else(|_| inputs_from(&Value::Null, has_tuner))));
        });
    });
}

pub fn read_immich(shared: &Shared) {
    let to = shared.clone();
    root::read_shared_file("immich.json", move |text| {
        let saved: Value = text.ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(Value::Null);
        let (url, key) = (tidy_address(saved["url"].as_str().unwrap_or("")), saved["key"].as_str().unwrap_or("").trim().to_string());
        if key.is_empty() || url.is_empty() {
            return change(&to, |live| live.immich.ready = false);
        }
        change(&to, |live| {
            live.immich.url = url.clone();
            live.immich.key = key.clone();
            live.immich.ready = true;
        });
        let to_albums = to.clone();
        root::immich_request(&url, &key, "/api/albums", move |text| {
            change(&to_albums, |live| live.immich.albums = text.ok().and_then(|t| albums_in(&t)).unwrap_or_default());
        });
    });
}

pub fn test_immich(shared: &Shared) {
    let (url, key) = match shared.lock() {
        Ok(live) => (tidy_address(&live.immich.url), live.immich.key.trim().to_string()),
        Err(_) => return,
    };
    let problem = if url.is_empty() {
        Some("Type the server address first")
    } else if !root::immich_address_ok(&url) {
        Some("The address should look like http://192.168.1.10:2283, with no path")
    } else if key.is_empty() {
        Some("Type the API key first")
    } else if !root::immich_key_ok(&key) {
        Some("That API key has characters an Immich key never has")
    } else {
        None
    };
    if let Some(problem) = problem {
        change(shared, |live| live.immich.test = Test::Failed);
        return crate::toast(problem.to_string(), true);
    }
    change(shared, |live| live.immich.test = Test::Testing);
    let to = shared.clone();
    root::immich_request(&url.clone(), &key.clone(), "/api/server/version", move |answered| {
        if answered.is_err() {
            change(&to, |live| live.immich.test = Test::Failed);
            return crate::toast(format!("No answer from {url}"), true);
        }
        root::immich_request(&url.clone(), &key.clone(), "/api/albums", move |text| {
            let Some(albums) = text.ok().and_then(|t| albums_in(&t)) else {
                change(&to, |live| live.immich.test = Test::Failed);
                return crate::toast("The server answered, but it refused the API key".into(), true);
            };
            let saved = json!({"url": url, "key": key}).to_string();
            root::write_shared_file("immich.json", &saved, move |written| {
                if written.is_err() {
                    change(&to, |live| live.immich.test = Test::Failed);
                    return crate::toast("Couldn't save the Immich settings on the TV".into(), true);
                }
                change(&to, |live| {
                    live.immich = Immich { ready: true, albums, url, key, test: Test::Idle };
                });
                crate::toast("Connected to Immich".into(), false);
            });
        });
    });
}

pub fn run_job(shared: &Shared, id: &str, app_dir: &str, job: impl FnOnce(Box<dyn FnOnce(Result<String, String>) + Send>)) {
    if let Ok(mut live) = shared.lock() {
        if !live.busy.is_empty() {
            return;
        }
        live.busy = id.to_string();
    }
    crate::wake();
    let (to, dir) = (shared.clone(), app_dir.to_string());
    job(Box::new(move |out| {
        match out {
            Ok(text) => {
                let last = text.lines().map(str::trim).filter(|l| !l.is_empty()).last().unwrap_or("Done").to_string();
                let lower = last.to_lowercase();
                let error = ["refused", "could not", "cannot", "not ours", "no node", "failed"].iter().any(|w| lower.contains(w));
                let shown = if last.chars().count() > 160 { format!("{}...", last.chars().take(157).collect::<String>()) } else { last };
                crate::toast(shown, error);
            }
            Err(why) => crate::toast(why, true),
        }
        let after = to.clone();
        root::tv_status(&dir, move |state| {
            change(&after, |live| {
                if let Ok(text) = state {
                    live.tv = Some(parse_tv_state(&text));
                }
                live.busy.clear();
            });
        });
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tv_scripts_state_lines_are_read_and_a_missing_field_has_its_default() {
        let state = parse_tv_state("scripts=ok\nhome=com.samrenfrew.lumen\nagreements=3\nliveplus=ON\nhosts=on\nnoise\n");
        assert!(state.scripts_ready());
        assert_eq!(state.home, "com.samrenfrew.lumen");
        assert_eq!(state.agreements, Some(3));
        assert_eq!(state.live_plus, "on");
        assert!(state.hosts && !state.services);
        assert_eq!(state.power_on_loaded, "unknown");
        assert!(!state.power_on_pending());
    }

    #[test]
    fn a_side_app_is_behind_only_when_a_newer_release_is_listed() {
        let listing = json!({"packages": [
            {"id": "com.samrenfrew.lumen.guide", "manifest": {"version": "1.10.0", "ipkUrl": "u", "ipkHash": {"sha256": "s"}}},
            {"id": "com.samrenfrew.lumen.quick", "manifest": {"version": "1.0.0", "ipkUrl": "u", "ipkHash": {"sha256": "s"}}},
            {"id": "someone.else", "manifest": {"version": "9.0.0", "ipkUrl": "u", "ipkHash": {"sha256": "s"}}}
        ]});
        let side = SideApps { installed: vec![("com.samrenfrew.lumen.guide".into(), "1.9.3".into()), ("com.samrenfrew.lumen.quick".into(), "1.0.0".into())], latest: Some(releases_in(&listing)), offline: false };
        assert_eq!(side.behind(), ["com.samrenfrew.lumen.guide"]);
    }

    #[test]
    fn sources_are_live_tv_then_the_inputs_by_port() {
        let reply = json!({"devices": [
            {"id": "HDMI_3", "label": "HDMI 3", "port": 3, "appId": "com.webos.app.hdmi3", "icon": "HDMI_3.png"},
            {"id": "HDMI_1", "label": "Chromecast", "port": 1, "appId": "com.webos.app.hdmi1", "icon": "../x.png"},
            {"id": "X", "label": "", "port": 2, "appId": "com.webos.app.x"}
        ]});
        let labels: Vec<(String, String)> = inputs_from(&reply, true).into_iter().map(|i| (i.label, i.icon)).collect();
        assert_eq!(labels, [("Live TV".to_string(), "tv.png".to_string()), ("Chromecast".into(), "hdmigeneric.png".into()), ("HDMI 3".into(), "HDMI_3.png".into())]);
    }

    #[test]
    fn a_typed_immich_address_loses_its_slash_and_api_and_gains_a_scheme() {
        assert_eq!(tidy_address(" 192.168.1.10:2283/api/ "), "http://192.168.1.10:2283");
        assert_eq!(tidy_address("https://photos.home/"), "https://photos.home");
        assert_eq!(tidy_address(""), "");
    }
}
