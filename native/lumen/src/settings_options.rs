use crate::settings_tv::{side_title, Immich, SideApps, Test, TvState};
use serde_json::{json, Value};

pub const HOUSE: &str = "M3 11.5 12 4l9 7.5M5.5 9.8V20h13V9.8M10 20v-6h4v6";
pub const GUIDE: &str = "M3.5 5.5h17v13h-17zM3.5 9.5h17M9 9.5v9";
pub const PICTURE: &str = "M3.5 5.5h17v11h-17zM8 20h8M12 16.5V20";
pub const MOON: &str = "M20.5 14.2A8.5 8.5 0 0 1 9.8 3.5a8.5 8.5 0 1 0 10.7 10.7z";
pub const SUN: &str = "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.3 5.3l1.4 1.4M17.3 17.3l1.4 1.4M5.3 18.7l1.4-1.4M17.3 6.7l1.4-1.4";
pub const CONTRAST: &str = "M12 3.5a8.5 8.5 0 1 0 0 17 8.5 8.5 0 0 0 0-17zM12 3.5v17M12 7.5h4.5M12 11.5h6.5M12 15.5h5";
pub const IMAGE: &str = "M3.5 5h17v14h-17zM3.5 15.5l5-5 4 4 3-3 5 5M15.5 9.3a.8.8 0 1 0 0-.1";
pub const GLOBE: &str = "M12 3.5a8.5 8.5 0 1 0 0 17 8.5 8.5 0 0 0 0-17zM3.5 12h17M12 3.5c-2.6 2.4-3.8 5.2-3.8 8.5s1.2 6.1 3.8 8.5c2.6-2.4 3.8-5.2 3.8-8.5S14.6 5.9 12 3.5z";
pub const POWER: &str = "M12 3.5v8M7 6.5a7.5 7.5 0 1 0 10 0";
pub const EYE: &str = "M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12zM12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6z";
pub const GRID: &str = "M4 4h6.5v6.5H4zM13.5 4H20v6.5h-6.5zM4 13.5h6.5V20H4zM13.5 13.5H20V20h-6.5z";
pub const GEAR: &str = crate::home::TOP_ITEMS[1].2;
pub const INPUTS: &str = crate::home::TOP_ITEMS[2].2;

const LUMEN_IDS: [&str; 3] = ["com.samrenfrew.lumen", "com.samrenfrew.lumen.dev", "com.samrenfrew.lumen.native"];
const NEEDS_SCRIPTS: &str = "Install the TV scripts first, in Settings > System > TV scripts. ";
const IMMICH_SOURCES: [(&str, &str); 5] = [("immich:recent", "Recent"), ("immich:recent-people", "Recent, with people"), ("immich:memories", "On this day"), ("immich:favorites", "Favourites"), ("immich:all", "All photos")];
const SIDE_DESCRIPTIONS: [&str; 1] = [
    "One app for the Guide over Live TV, Quick Settings on the gear, your screensaver photos over the app you are in, and the Dock on Home. Switch each on or off below. Off removes it from the TV.",
];
const OVERLAY_ROWS: [(&str, &str, &str); 4] = [
    ("overlay-guide", "Guide over Live TV", "LIST opens Now and Next over Live TV, GUIDE the full guide. Off leaves LIST to the TV, and GUIDE opens Lumen's Guide."),
    ("overlay-quick", "Quick Settings on the gear", "Sound, picture and backlight over any app, on the gear button. In Lumen the gear opens Control Center. Off gives the gear back to LG."),
    ("overlay-saver", "Screensaver over apps", "Your screensaver photos over the app you are watching, so after a key the app is where you left it. Off opens Lumen's own screensaver instead."),
    ("overlay-dock", "Dock over apps", "Home over an app brings up Lumen's dock over it: OK opens an app, Down or a second Home opens Lumen. Off makes Home open Lumen at once."),
];
pub const SIDE_OPTIONS: [&str; 1] = ["side-overlay"];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Group {
    Home,
    LiveTv,
    Wallpaper,
    Screensaver,
    Brightness,
    Appearance,
    Photos,
    Region,
    Tv,
    Privacy,
    SideApps,
    System,
}

pub const GROUPS: [(Group, &str, &str); 12] = [
    (Group::Home, "Home Screen", HOUSE),
    (Group::LiveTv, "Live TV", GUIDE),
    (Group::Wallpaper, "Wallpaper", PICTURE),
    (Group::Screensaver, "Screensaver", MOON),
    (Group::Brightness, "Brightness", SUN),
    (Group::Appearance, "Appearance", CONTRAST),
    (Group::Photos, "Immich", IMAGE),
    (Group::Region, "Region", GLOBE),
    (Group::Tv, "Home and Power", POWER),
    (Group::Privacy, "Privacy", EYE),
    (Group::SideApps, "Side apps", GRID),
    (Group::System, "System", GEAR),
];

pub fn group_named(group: Group) -> (&'static str, &'static str) {
    GROUPS.iter().find(|g| g.0 == group).map_or(("", GEAR), |g| (g.1, g.2))
}

#[derive(Clone, PartialEq, Debug)]
pub enum Kind {
    Switch,
    List { labels: Vec<String>, values: Vec<Value>, at: usize, picks: Option<Vec<usize>> },
    Text { value: String, placeholder: &'static str },
    Action,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Opt {
    pub id: &'static str,
    pub group: Group,
    pub label: String,
    pub description: String,
    pub value: String,
    pub kind: Kind,
}

impl Opt {
    pub fn flips(&self) -> bool {
        matches!(&self.kind, Kind::List { labels, picks: None, .. } if labels.len() == 2)
    }

    pub fn opens_a_page(&self) -> bool {
        match &self.kind {
            Kind::Text { .. } => true,
            Kind::List { labels, .. } => !labels.is_empty() && !self.flips(),
            _ => false,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct City {
    pub name: String,
    pub code: String,
    pub country: String,
}

pub fn cities_in(tsv: &str) -> Vec<City> {
    tsv.lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split('\t').collect();
            (f.len() >= 5).then(|| City { name: f[0].into(), code: f[3].into(), country: f[4].into() })
        })
        .collect()
}

pub struct Facts<'a> {
    pub tv: Option<&'a TvState>,
    pub tv_failed: bool,
    pub busy: &'a str,
    pub side: Option<&'a SideApps>,
    pub immich: &'a Immich,
    pub cities: &'a [City],
    pub dev: bool,
}

pub fn flag(config: &Value, key: &str, default: bool) -> bool {
    config[key].as_bool().unwrap_or(default)
}

pub fn text<'a>(config: &'a Value, key: &str, default: &'a str) -> &'a str {
    config[key].as_str().unwrap_or(default)
}

pub fn number(config: &Value, key: &str, default: f64) -> f64 {
    config[key].as_f64().unwrap_or(default)
}

fn on_off(on: bool) -> String {
    (if on { "On" } else { "Off" }).to_string()
}

fn same(a: &Value, b: &Value) -> bool {
    match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) => x == y,
        _ => a == b,
    }
}

fn list(labels: Vec<String>, values: Vec<Value>, now: &Value, fallback: Option<String>) -> (String, Kind) {
    let found = values.iter().position(|v| same(v, now));
    let value = found.map(|i| labels[i].clone()).or(fallback).unwrap_or_else(|| labels.first().cloned().unwrap_or_default());
    (value, Kind::List { labels, values, at: found.unwrap_or(0), picks: None })
}

fn words(labels: &[&str]) -> Vec<String> {
    labels.iter().map(|l| l.to_string()).collect()
}

fn half_hours(from: u32, to: u32) -> Vec<String> {
    (from * 2..=to * 2).map(|half| format!("{:02}:{}", (half / 2) % 24, if half % 2 == 0 { "00" } else { "30" })).collect()
}

struct Rows<'a> {
    config: &'a Value,
    out: Vec<Opt>,
}

impl Rows<'_> {
    fn switch(&mut self, id: &'static str, group: Group, label: &str, description: &str, key: &str, default: bool) {
        let value = on_off(flag(self.config, key, default));
        self.out.push(Opt { id, group, label: label.into(), description: description.into(), value, kind: Kind::Switch });
    }

    fn listed(&mut self, id: &'static str, group: Group, label: &str, description: &str, labels: Vec<String>, values: Vec<Value>, now: Value) {
        let (value, kind) = list(labels, values, &now, None);
        self.out.push(Opt { id, group, label: label.into(), description: description.into(), value, kind });
    }

    fn named(&mut self, id: &'static str, group: Group, label: &str, description: &str, labels: &[&str], values: &[&str], key: &str, default: &str) {
        let now = json!(text(self.config, key, default));
        self.listed(id, group, label, description, words(labels), values.iter().map(|v| json!(v)).collect(), now);
    }

    fn counted(&mut self, id: &'static str, group: Group, label: &str, description: &str, labels: Vec<String>, values: &[f64], key: &str, default: f64) {
        let now = json!(number(self.config, key, default));
        self.listed(id, group, label, description, labels, values.iter().map(|v| json!(v)).collect(), now);
    }

    fn yes_no(&mut self, id: &'static str, group: Group, label: &str, description: &str, labels: &[&str], key: &str, default: bool) {
        let now = json!(if flag(self.config, key, default) { "on" } else { "off" });
        self.listed(id, group, label, description, words(labels), vec![json!("off"), json!("on")], now);
    }

    fn action(&mut self, id: &'static str, group: Group, label: &str, description: String, value: String) {
        self.out.push(Opt { id, group, label: label.into(), description, value, kind: Kind::Action });
    }
}

pub fn options(config: &Value, facts: &Facts) -> Vec<Opt> {
    let mut r = Rows { config, out: Vec::new() };
    let c = config;
    let live_tv_row = flag(c, "liveTvRow", true);
    r.switch("live-tv-row", Group::Home, "Live TV row", "A row of TV channels under your apps, showing what is on now and next, with the Guide at the end. OK on a channel opens Live TV on it. Settings > Channels picks the channels.", "liveTvRow", true);
    r.counted("first-row", Group::Home, "Apps per row", "How many apps a row holds: the dock, and each row of the grid below it. With 7 the tiles are a little smaller, with 5 a little bigger.", words(&["5", "6", "7"]), &[5.0, 6.0, 7.0], "firstRow", 6.0);
    r.switch("hide-names", Group::Home, "Hide app names", "No names under the apps on the home screen, not even the one in focus. Settings still shows them.", "hideNames", false);
    r.switch("show-recents", Group::Home, "Show recents indicator", "A dot over the apps that are open.", "showRecents", true);
    r.switch("wallpaper-captions", Group::Home, "Wallpaper captions", "The picture's title and artist at the bottom right.", "wallpaperCaptions", true);
    r.switch("show-weather", Group::Home, "Date and weather", "Today's date and the temperature at the top left. The temperature needs a Weather city in Settings > Region; it comes from open-meteo.com.", "showWeather", true);
    let home_cast = if flag(c, "homeForecast", false) { text(c, "homeForecastSize", "small") } else { "off" };
    r.listed("home-forecast", Group::Home, "Weather widget", "The next few hours of weather, above the dock on the left. Small: the next 6 hours. Large: the weather now with today's high and low, and the next 7 hours with the chance of rain. Needs a Weather city.", words(&["Off", "Small", "Large"]), vec![json!("off"), json!("small"), json!("large")], json!(home_cast));

    if facts.dev && live_tv_row {
        r.yes_no("guide-preview", Group::LiveTv, "Preview in the Guide", "A live picture of the channel in focus in the Guide. It starts when you stop on a channel for a moment, so flicking through does not tune each one.", &["Off", "On"], "guidePreview", false);
        r.yes_no("home-preview", Group::LiveTv, "Preview on the Live TV row", "On the home screen, the channel in focus on the Live TV row turns into its live picture after a moment.", &["Off", "On"], "homePreview", false);
        if flag(c, "guidePreview", false) || flag(c, "homePreview", false) {
            r.yes_no("guide-sound", Group::LiveTv, "Sound", "Play the channel's sound with the preview, or keep it silent.", &["Muted", "On"], "guideSound", false);
        }
    }

    let immich = facts.immich;
    let (type_labels, type_values): (&[&str], &[&str]) = if immich.ready { (&["Pictures", "Immich photos", "Plain"], &["pictures", "immich", "plain"]) } else { (&["Pictures", "Plain"], &["pictures", "plain"]) };
    let type_description = if immich.ready {
        "Pictures from the library, your Immich photos, or a plain background. Choose wallpaper then picks which ones."
    } else {
        "Pictures from the library, or a plain background. Choose wallpaper then picks which ones. Connect Immich in Settings > Immich to use your own photos."
    };
    r.named("wp-type", Group::Wallpaper, "Wallpaper type", type_description, type_labels, type_values, "wallpaperType", "pictures");
    let kind = text(c, "wallpaperType", "pictures");
    if kind == "pictures" || kind == "immich" {
        r.counted("wp-auto", Group::Wallpaper, "Change wallpaper automatically", "Shows another of your pictures every 30 seconds to 60 minutes.", words(&["Off", "30 sec", "1 min", "5 min", "15 min", "30 min", "60 min"]), &[0.0, 0.5, 1.0, 5.0, 15.0, 30.0, 60.0], "wpAutoMinutes", 0.0);
        r.switch("wp-new", Group::Wallpaper, "New picture on startup", "Opens on another of your pictures each time. Off, it opens on the last one it showed.", "wpNewOnLaunch", true);
    }
    r.named("night-dim", Group::Wallpaper, "Dim the wallpaper", "A darker wallpaper, easier on the eyes in a dark room. At night: between 7 pm and 6 am.", &["Off", "At night", "Always"], &["off", "night", "always"], "wallpaperDim", "off");
    if kind != "plain" {
        r.switch("print-look", Group::Wallpaper, "Print look", "Pictures look printed on paper rather than lit up: a little darker, a faint cream tint, and light falling from the top left.", "printLook", false);
    }

    let saver_on = flag(c, "ambientOn", false);
    r.listed("ambient-on", Group::Screensaver, "Screensaver", "Enabled: this app's screensaver replaces the TV's, a slow crossfade of pictures with the clock; any key returns to the home screen. Disabled: the TV's default LG screensaver runs. The TV's own auto power-off still applies.", words(&["Enabled", "Disabled"]), vec![json!("on"), json!("off")], json!(if saver_on { "on" } else { "off" }));
    if saver_on {
        r.counted("idle-screensaver", Group::Screensaver, "Start on the home screen after", "Starts the screensaver when the home screen has been left alone this long. The TV's own screensaver timer still covers everything else.", words(&["Never", "30 seconds", "1 minute", "2 minutes", "5 minutes"]), &[0.0, 30.0, 60.0, 120.0, 300.0], "idleSeconds", 0.0);
        if immich.ready {
            r.named("saver-show", Group::Screensaver, "Show", "What the screensaver shows. Wallpaper pictures: the pictures picked in Wallpaper > Choose wallpaper. Immich photos: photos from your Immich server.", &["Wallpaper pictures", "Immich photos"], &["wallpaper", "immich"], "saverShow", "wallpaper");
        }
        if immich.ready && text(c, "saverShow", "wallpaper") == "immich" {
            let sources: Vec<(String, String)> = IMMICH_SOURCES.iter().map(|(k, t)| (k.to_string(), t.to_string())).chain(immich.albums.iter().map(|(id, title)| (format!("immich:album:{id}"), title.clone()))).collect();
            let picks: Vec<usize> = text(c, "ambient", "immich:recent").split('|').filter_map(|k| sources.iter().position(|(key, _)| key == k)).collect();
            let value = if picks.len() == 1 { sources[picks[0]].1.clone() } else { format!("{} sources", picks.len()) };
            let kind = Kind::List { labels: sources.iter().map(|s| s.1.clone()).collect(), values: sources.iter().map(|s| json!(s.0)).collect(), at: picks.first().copied().unwrap_or(0), picks: Some(picks) };
            r.out.push(Opt { id: "ambient", group: Group::Screensaver, label: "Immich photos".into(), description: "Which Immich photos the screensaver shows. Pick several and it mixes them.".into(), value, kind });
        }
        r.counted("ambient-seconds", Group::Screensaver, "Time on each picture", "How long each picture stays on the screensaver before the next one fades in.", words(&["10 seconds", "20 seconds", "30 seconds", "1 minute", "2 minutes", "5 minutes"]), &[10.0, 20.0, 30.0, 60.0, 120.0, 300.0], "ambientSeconds", 30.0);
        r.named("ambient-brightness", Group::Screensaver, "Brightness", "Turns the TV's backlight down while the screensaver shows, and back up when you press a key.", &["As the TV is set", "Dim", "Very dim"], &["tv", "dim", "low"], "ambientBrightness", "dim");
        let saver_cast = if flag(c, "saverForecast", false) { text(c, "saverForecastSize", "small") } else { "off" };
        r.listed("saver-forecast", Group::Screensaver, "Weather widget", "The next few hours of weather on the screensaver, at the bottom between the clock and the date. Small: the next 6 hours. Large: the weather now with today's high and low, and the next 7 hours with the chance of rain. Needs a Weather city.", words(&["Off", "Small", "Large"]), vec![json!("off"), json!("small"), json!("large")], json!(saver_cast));
    }

    r.yes_no("brightness-on", Group::Brightness, "Brightness schedule", "Sets the TV's brightness at the start of the day, the evening and the night, and Eye Comfort if you choose. It works while Lumen is closed, and needs the TV scripts (Settings > System). A change by hand stays until the next part of the day. Turn off anything else that sets the brightness on a timer, such as Home Assistant, so the two do not fight.", &["Off", "On"], "brightnessOn", false);
    if flag(c, "brightnessOn", false) {
        let levels: Vec<f64> = (0..=20).map(|i| i as f64 * 5.0).collect();
        let level_labels: Vec<String> = levels.iter().map(|l| l.to_string()).collect();
        let times = |from, to| half_hours(from, to);
        for (id, label, description, key, default, from, to) in [
            ("brightness-day-at", "Day starts", "When the day brightness is set.", "blDayAt", "07:00", 5, 12),
            ("brightness-evening-at", "Evening starts", "When the evening brightness is set.", "blEveningAt", "18:00", 15, 22),
            ("brightness-night-at", "Night starts", "When the night brightness is set. It holds until the day starts again.", "blNightAt", "22:00", 19, 26),
        ] {
            let at = times(from, to);
            let now = json!(text(c, key, default));
            r.listed(id, Group::Brightness, label, description, at.clone(), at.iter().map(|t| json!(t)).collect(), now);
        }
        for (id, label, part, key, default) in [("brightness-day", "Day brightness", "day", "blDay", 80.0), ("brightness-evening", "Evening brightness", "evening", "blEvening", 60.0), ("brightness-night", "Night brightness", "night", "blNight", 30.0)] {
            r.counted(id, Group::Brightness, label, &format!("The TV's backlight through the {part}, from 0 to 100."), level_labels.clone(), &levels, key, default);
        }
        r.counted("brightness-ramp", Group::Brightness, "Change", "How each part of the day moves to its brightness. At once: it jumps when that part starts. Over a time: it starts at the last part's brightness and changes a little each minute until it gets there. While it changes, a change by hand lasts only a minute.", words(&["At once", "Over 30 minutes", "Over 1 hour", "Over 1½ hours", "Over 2 hours"]), &[0.0, 30.0, 60.0, 90.0, 120.0], "blRamp", 0.0);
        r.named("brightness-eye", Group::Brightness, "Eye Comfort", "Eye Comfort makes the picture warmer. As the TV is set: the schedule leaves it alone. Otherwise it is turned on and off with the parts of the day.", &["As the TV is set", "Off all day", "Evening and night", "Night only"], &["keep", "off", "evening", "night"], "blEye", "keep");
        let order = ["brightness-day-at", "brightness-day", "brightness-evening-at", "brightness-evening", "brightness-night-at", "brightness-night", "brightness-ramp", "brightness-eye"];
        let (mut kept, mut schedule): (Vec<Opt>, Vec<Opt>) = r.out.drain(..).partition(|o| !order.contains(&o.id));
        schedule.sort_by_key(|o| order.iter().position(|id| *id == o.id));
        kept.extend(schedule);
        r.out = kept;
    }

    r.switch("reduce-motion", Group::Appearance, "Reduce motion", "Focus, rows and panels change at once, without sliding, fading or zooming.", "reduceMotion", false);
    r.switch("liquid-glass", Group::Appearance, "Liquid glass", "The glass bends the picture behind it at its edges, with a faint colour fringe, on the app bar and every glass button.", "liquidGlass", true);
    if facts.dev {
        r.action("glass-lab", Group::Appearance, "Glass lab", "Opens the home screen with sliders for every part of the glass, to try them on the wallpaper. Back leaves it. What you set stays.".into(), "Open".into());
    }

    let shown_key = if immich.key.is_empty() { "Not set".to_string() } else { format!("\u{2022}\u{2022}\u{2022}\u{2022} {}", immich.key.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect::<String>()) };
    r.out.push(Opt { id: "immich-url", group: Group::Photos, label: "Server address".into(), description: "Your Immich server as you would open it in a browser at home: scheme, address and port, for example http://192.168.1.10:2283.".into(), value: if immich.url.is_empty() { "Not set".into() } else { immich.url.clone() }, kind: Kind::Text { value: immich.url.clone(), placeholder: "http://192.168.1.10:2283" } });
    r.out.push(Opt { id: "immich-key", group: Group::Photos, label: "API key".into(), description: "In Immich: Account Settings > API Keys > New API Key, with only album.read, asset.read, asset.view, memory.read, face.read and stack.read. Read-only, so it can only show photos.".into(), value: shown_key, kind: Kind::Text { value: immich.key.clone(), placeholder: "The key from Immich" } });
    let tested = match immich.test {
        Test::Testing => "Testing",
        Test::Failed => "Failed",
        Test::Idle if immich.ready => "Connected",
        Test::Idle => "Not connected",
    };
    r.action("immich-test", Group::Photos, "Test connection", "Checks the address and the key with your server. When it passes, Lumen saves them on the TV and your Immich photos can be the wallpaper and the screensaver.".into(), tested.into());
    if immich.ready {
        r.counted("immich-recent-days", Group::Photos, "Recent photos from", "How far back the Recent and Recent, with people sources reach, for the wallpaper and the screensaver.", words(&["Last month", "Last 3 months", "Last 6 months", "Last year"]), &[30.0, 90.0, 180.0, 365.0], "immichRecentDays", 180.0);
    }

    r.named("clock-format", Group::Region, "Clock", "Auto follows the TV's own 12/24-hour setting. Choose 12-hour or 24-hour to override it.", &["Auto", "12-hour", "24-hour"], &["auto", "12h", "24h"], "clockFormat", "auto");
    if text(c, "clockFormat", "auto") != "24h" {
        r.switch("show-ampm", Group::Region, "AM/PM on the clock", "Show AM or PM beside the time when the clock is 12-hour.", "showAmPm", true);
    }
    let mut countries: Vec<(&str, &str)> = Vec::new();
    for city in facts.cities {
        if !countries.iter().any(|(code, _)| *code == city.code) {
            countries.push((&city.code, &city.country));
        }
    }
    let country = text(c, "weatherCountry", "");
    r.listed("weather-country", Group::Region, "Weather country", "Pick your country, then the city below. Until a city is chosen no weather is shown and nothing is fetched.", std::iter::once("Any".to_string()).chain(countries.iter().map(|(_, name)| name.to_string())).collect(), std::iter::once(json!("")).chain(countries.iter().map(|(code, _)| json!(code))).collect(), json!(country));
    let in_country: Vec<&str> = facts.cities.iter().filter(|city| country.is_empty() || !countries.iter().any(|(code, _)| *code == country) || city.code == country).map(|city| city.name.as_str()).collect();
    let city = text(c, "weatherCity", "");
    let (value, kind) = list(
        ["Not set", "Auto"].iter().map(|s| s.to_string()).chain(in_country.iter().map(|s| s.to_string())).collect(),
        [json!(""), json!(crate::weather::AUTO_PLACE)].into_iter().chain(in_country.iter().map(|s| json!(s))).collect(),
        &json!(city),
        Some(city.to_string()).filter(|c| !c.is_empty()),
    );
    r.out.push(Opt { id: "weather-city", group: Group::Region, label: "Weather city".into(), description: "Not set: no weather. Auto finds your area from your internet connection, which sends your IP address to ipwho.is and can be a few towns off. Or pick a city.".into(), value, kind });

    tv_rows(&mut r, facts);
    side_rows(&mut r, facts);

    let scripts = match facts.tv {
        _ if facts.busy == "tv-scripts" => "Working",
        None if facts.tv_failed => "Not on a TV",
        None => "Checking",
        Some(tv) => match tv.scripts.as_str() {
            "ok" => "Up to date",
            "old" => "Update",
            "missing" => "Install",
            _ => "Not in this build",
        },
    };
    r.action("tv-scripts", Group::System, "TV scripts", "The scripts Lumen runs on the TV as root: the Home button watcher, Home and Power, and Privacy. They ship inside Lumen; this copies this build's into place and restarts them. Needs the Homebrew Channel with root.".into(), scripts.into());
    r.action("lg-home", Group::System, "LG Home", "Opens LG's own home screen. The Home button opens Lumen again once you leave it.".into(), "Open".into());
    r.switch("show-all-apps", Group::System, "Show system apps", "Offer the TV's own apps (Live TV, LG Channels, AirPlay, Screen Share, Game Optimizer, TV Settings) in Settings > Apps, so you can add them to the home screen.", "showAllApps", false);
    r.action("restore-backup", Group::System, "Restore settings from the TV", "Lumen keeps its settings on the TV (/var/lib/homehook/lumen-config.json), shared with the web Lumen. This reads that copy again, replacing the settings here.".into(), "Restore".into());
    r.out
}

fn tv_rows(r: &mut Rows, facts: &Facts) {
    let tv = facts.tv;
    let ready = tv.is_some_and(TvState::scripts_ready);
    let shown = |id: &str, value: String| -> String {
        if facts.busy == id {
            "Working".into()
        } else if tv.is_none() {
            (if facts.tv_failed { "Not on a TV" } else { "Checking" }).into()
        } else if !ready {
            "Needs TV scripts".into()
        } else {
            value
        }
    };
    let pre = if tv.is_some() && !ready { NEEDS_SCRIPTS } else { "" };
    let on_off = |v: &str| (if v == "on" { "On" } else if v == "off" { "Off" } else { "Not on this TV" }).to_string();
    let lumen_home = tv.is_some_and(|t| LUMEN_IDS.contains(&t.home.as_str()));
    let native_why = tv.map_or("", TvState::native_why);
    r.action("tv-home", Group::Tv, "Home button opens", format!("{pre}{}", if !native_why.is_empty() && !lumen_home { native_why } else { "Which app LG's own Home routes open: the Home button, Exit, and Back out of an app. Lumen's LG Home button still opens LG Home. LG Input Hook can keep its Home mapping." }), shown("tv-home", (if lumen_home { "Lumen" } else if !native_why.is_empty() { "Not available" } else { "LG Home" }).into()));
    let power_why = tv.map_or("", TvState::power_on_why);
    let power_lumen = tv.is_some_and(|t| LUMEN_IDS.contains(&t.power_on.as_str()));
    let pending = tv.is_some_and(TvState::power_on_pending);
    r.action("tv-power-on", Group::Tv, "When the TV turns on", format!("{pre}{}", if !power_why.is_empty() && !power_lumen { power_why } else { "Lumen: the power button and a start after a power cut open Lumen first, before Live TV is drawn. Casting, Wake-on-LAN and HDMI-CEC still go where they ask. Takes effect after the TV restarts." }), shown("tv-power-on", format!("{}{}", if power_lumen { "Lumen" } else if !power_why.is_empty() { "Not available" } else { "Last input" }, if pending { " (after a restart)" } else { "" })));
    let web = tv.map_or("", |t| t.web.as_str());
    if web == "installed" || web == "pending" {
        r.action("retire-web", Group::Tv, "Old Lumen", format!("{pre}The web Lumen, its Guide and its Quick Settings are still on the TV. Remove moves Home and power-on to this Lumen first, then uninstalls them. If the TV still opens the old one at power-on, they go at the next restart."), shown("retire-web", (if web == "pending" { "Goes at the next restart" } else { "Remove" }).into()));
    }
    if pending || web == "pending" {
        r.action("tv-restart", Group::Tv, "Restart the TV now", "The TV reads what it opens at power-on only when it starts, and the old Lumen goes then. This restarts it now.".into(), shown("tv-restart", "Restart".into()));
    }
    let agreements = tv.and_then(|t| t.agreements);
    r.action("privacy-agreements", Group::Privacy, "LG data agreements", format!("{pre}{}Withdraws LG's optional agreements: screen recognition, ads based on what you watch, personalised ads, ad cookies, sharing with data partners, extra usage data, remote diagnostics and marketing. Voice and the terms of use stay. OK again puts back what was there before.", if tv.is_some_and(|t| !t.node) { "This TV has no node to change them safely. " } else { "" }), shown("privacy-agreements", match agreements { None => "Unknown".into(), Some(0) => "Withdrawn".into(), Some(n) => format!("{n} given") }));
    r.action("privacy-lmt", Group::Privacy, "Limit ad tracking", format!("{pre}The TV's own setting: apps are asked not to use its advertising ID to profile you."), shown("privacy-lmt", on_off(tv.map_or("", |t| &t.lmt))));
    r.action("privacy-liveplus", Group::Privacy, "Live Plus", format!("{pre}LG's screen recognition: it samples what is on screen, from apps and HDMI alike, to work out what you watch. Off is the private choice."), shown("privacy-liveplus", on_off(tv.map_or("", |t| &t.live_plus))));
    r.action("privacy-hosts", Group::Privacy, "Block LG ad servers", format!("{pre}{}Blocks LG's ad, tracking and screen recognition servers on the TV itself. The Content Store, app updates and streaming apps keep working.", if tv.is_some_and(|t| !t.hosts_writable) { "This TV's /etc/hosts cannot be written, so nothing can be blocked. " } else { "" }), shown("privacy-hosts", on_off_flag(tv.is_some_and(|t| t.hosts))));
    let running = tv.filter(|t| t.services && t.services_running > 0).map_or(String::new(), |t| format!(" {} still running.", t.services_running));
    r.action("privacy-services", Group::Privacy, "Stop LG tracking services", format!("{pre}Stops LG's log uploaders, suggestion and screen recognition services, and stops them again at each start. Frees about 10 MB. LG's ad service stays, as some streaming apps wait for it.{running}"), shown("privacy-services", on_off_flag(tv.is_some_and(|t| t.services))));
    r.action("privacy-updates", Group::Privacy, "Block TV software updates", format!("{pre}A software update can remove root, and with it Lumen's TV scripts. This is the Homebrew Channel's own switch, and takes effect after the TV restarts."), shown("privacy-updates", on_off_flag(tv.is_some_and(|t| t.updates))));
}

fn on_off_flag(on: bool) -> String {
    on_off(on)
}

fn side_rows(r: &mut Rows, facts: &Facts) {
    let side = facts.side;
    for (at, id) in crate::root::SIDE_APPS.iter().enumerate() {
        let option = SIDE_OPTIONS[at];
        let value = match side.map(|s| s.installed(id)) {
            _ if facts.busy == option => "Working".to_string(),
            None => "Checking".into(),
            Some(Some(version)) => format!("On, {version}"),
            Some(None) => "Off".into(),
        };
        r.action(option, Group::SideApps, side_title(id), SIDE_DESCRIPTIONS[at].into(), value);
    }
    if side.is_some_and(|s| s.installed(crate::root::SIDE_APPS[0]).is_some()) {
        for (option, title, description) in OVERLAY_ROWS {
            let feature = &option["overlay-".len()..];
            let value = match facts.tv {
                _ if facts.busy == option => "Working".to_string(),
                None => "Checking".into(),
                Some(tv) if !tv.scripts_ready() => "Needs TV scripts".into(),
                Some(tv) => on_off(!tv.overlay_off.iter().any(|f| f == feature)),
            };
            r.action(option, Group::SideApps, title, description.into(), value);
        }
        r.counted("dock-hide", Group::SideApps, "Dock hides after", "How long the Dock stays over an app with no key pressed. Back always hides it at once.", words(&["Never", "10 seconds", "30 seconds", "1 minute"]), &[0.0, 10.0, 30.0, 60.0], "dockHideSeconds", 30.0);
    }
    if let Some(side) = side.filter(|s| crate::root::SIDE_APPS.iter().any(|id| s.installed(id).is_some())) {
        let n = side.behind().len();
        let value = if facts.busy == "side-update" {
            "Working".to_string()
        } else if side.latest.is_none() {
            (if side.offline { "Could not check" } else { "Checking" }).into()
        } else if n > 0 {
            format!("Update {n} app{}", if n > 1 { "s" } else { "" })
        } else {
            "Up to date".into()
        };
        r.action("side-update", Group::SideApps, "Update side apps", "Gets the newest version of each side app that is on, from Lumen's releases on GitHub (the ones the Homebrew Channel lists), and installs it.".into(), value);
    }
}

pub fn switched(id: &str, config: &Value) -> Option<Vec<(String, Value)>> {
    let (key, default) = match id {
        "live-tv-row" => ("liveTvRow", true),
        "hide-names" => ("hideNames", false),
        "show-recents" => ("showRecents", true),
        "wallpaper-captions" => ("wallpaperCaptions", true),
        "print-look" => ("printLook", false),
        "show-weather" => ("showWeather", true),
        "wp-new" => ("wpNewOnLaunch", true),
        "reduce-motion" => ("reduceMotion", false),
        "liquid-glass" => ("liquidGlass", true),
        "show-ampm" => ("showAmPm", true),
        "show-all-apps" => ("showAllApps", false),
        _ => return None,
    };
    Some(vec![(key.to_string(), json!(!flag(config, key, default)))])
}

pub fn chosen(opt: &Opt, index: usize, config: &Value, cities: &[City]) -> Result<Vec<(String, Value)>, String> {
    let Kind::List { values, .. } = &opt.kind else { return Ok(Vec::new()) };
    let Some(value) = values.get(index).cloned() else { return Ok(Vec::new()) };
    let v = value.as_str().unwrap_or("").to_string();
    let one = |key: &str, value: Value| Ok(vec![(key.to_string(), value)]);
    match opt.id {
        "weather-country" => {
            let known = cities.iter().any(|c| c.code == v);
            let city = if v == text(config, "weatherCountry", "") || !known { json!(text(config, "weatherCity", "")) } else { json!("") };
            Ok(vec![("weatherCountry".into(), json!(v)), ("weatherCity".into(), city)])
        }
        "weather-city" => {
            let owner = cities.iter().find(|c| c.name == v).map_or("", |c| c.code.as_str());
            let country = Some(text(config, "weatherCountry", "")).filter(|c| !c.is_empty()).unwrap_or(owner);
            Ok(vec![("weatherCity".into(), json!(v)), ("weatherCountry".into(), json!(country))])
        }
        "home-forecast" | "saver-forecast" => {
            let (on, size) = if opt.id == "home-forecast" { ("homeForecast", "homeForecastSize") } else { ("saverForecast", "saverForecastSize") };
            if v == "off" {
                one(on, json!(false))
            } else {
                Ok(vec![(on.into(), json!(true)), (size.into(), json!(v))])
            }
        }
        "ambient" => {
            let mut picks: Vec<String> = text(config, "ambient", "immich:recent").split('|').filter(|k| !k.is_empty()).map(String::from).collect();
            if picks.contains(&v) {
                if picks.len() == 1 {
                    return Err("Keep at least one source".into());
                }
                picks.retain(|k| *k != v);
            } else {
                picks.push(v);
            }
            one("ambient", json!(picks.join("|")))
        }
        "guide-preview" => one("guidePreview", json!(v == "on")),
        "home-preview" => one("homePreview", json!(v == "on")),
        "guide-sound" => one("guideSound", json!(v == "on")),
        "ambient-on" => one("ambientOn", json!(v == "on")),
        "brightness-on" => one("brightnessOn", json!(v == "on")),
        "wp-type" => one("wallpaperType", value),
        "night-dim" => one("wallpaperDim", value),
        "saver-show" => one("saverShow", value),
        "ambient-brightness" => one("ambientBrightness", value),
        "clock-format" => one("clockFormat", value),
        "brightness-day-at" => one("blDayAt", value),
        "brightness-evening-at" => one("blEveningAt", value),
        "brightness-night-at" => one("blNightAt", value),
        "brightness-eye" => one("blEye", value),
        "first-row" => one("firstRow", whole(&value)),
        "wp-auto" => one("wpAutoMinutes", value),
        "idle-screensaver" => one("idleSeconds", whole(&value)),
        "dock-hide" => one("dockHideSeconds", whole(&value)),
        "ambient-seconds" => one("ambientSeconds", whole(&value)),
        "immich-recent-days" => one("immichRecentDays", whole(&value)),
        "brightness-day" => one("blDay", whole(&value)),
        "brightness-evening" => one("blEvening", whole(&value)),
        "brightness-night" => one("blNight", whole(&value)),
        "brightness-ramp" => one("blRamp", whole(&value)),
        _ => Ok(Vec::new()),
    }
}

fn whole(value: &Value) -> Value {
    value.as_f64().map_or(value.clone(), |n| if n.fract() == 0.0 { json!(n as i64) } else { json!(n) })
}

pub fn brightness_schedule(config: &Value) -> String {
    if !flag(config, "brightnessOn", false) {
        return String::new();
    }
    let eye_mode = text(config, "blEye", "keep");
    let eye = |on: bool| if eye_mode == "keep" { "keep" } else if on { "on" } else { "off" };
    let level = |key: &str, default: f64| number(config, key, default) as i64;
    let mut steps = [
        (text(config, "blDayAt", "07:00").to_string(), level("blDay", 80.0), eye(false)),
        (text(config, "blEveningAt", "18:00").to_string(), level("blEvening", 60.0), eye(eye_mode == "evening")),
        (text(config, "blNightAt", "22:00").to_string(), level("blNight", 30.0), eye(eye_mode == "evening" || eye_mode == "night")),
    ];
    steps.sort_by(|a, b| a.0.cmp(&b.0));
    let lines: String = steps.iter().map(|(at, backlight, eye)| format!("{at} {backlight} {eye}\n")).collect();
    format!("{lines}ramp {}\n", level("blRamp", 0.0))
}

pub fn toggled(ids: &[String], id: &str) -> Vec<String> {
    if ids.iter().any(|x| x == id) {
        ids.iter().filter(|x| *x != id).cloned().collect()
    } else {
        ids.iter().cloned().chain(std::iter::once(id.to_string())).collect()
    }
}

pub fn moved(ids: &[String], from: usize, to: usize) -> Vec<String> {
    let mut out = ids.to_vec();
    if from < out.len() && to < out.len() {
        let item = out.remove(from);
        out.insert(to, item);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts<'a>(immich: &'a Immich, cities: &'a [City]) -> Facts<'a> {
        Facts { tv: None, tv_failed: true, busy: "", side: None, immich, cities, dev: false }
    }

    fn find<'a>(all: &'a [Opt], id: &str) -> &'a Opt {
        all.iter().find(|o| o.id == id).unwrap()
    }

    #[test]
    fn each_row_shows_the_value_set_now_or_the_default_when_none_is() {
        let immich = Immich::default();
        let all = options(&json!({"firstRow": 7, "clockFormat": "24h", "homeForecast": true, "homeForecastSize": "large"}), &facts(&immich, &[]));
        assert_eq!(find(&all, "first-row").value, "7");
        assert_eq!(find(&all, "clock-format").value, "24-hour");
        assert_eq!(find(&all, "home-forecast").value, "Large");
        assert_eq!(find(&all, "show-weather").value, "On");
        assert_eq!(find(&all, "night-dim").value, "Off");
        assert_eq!(find(&all, "print-look").value, "Off");
        assert_eq!(switched("print-look", &json!({})), Some(vec![("printLook".to_string(), json!(true))]));
        assert!(all.iter().all(|o| o.id != "show-ampm"));
        assert_eq!(find(&all, "tv-home").value, "Not on a TV");
    }

    #[test]
    fn the_old_lumen_row_shows_only_while_the_web_lumen_is_on_the_tv() {
        let immich = Immich::default();
        let shown = |web: &str| {
            let tv = TvState { scripts: "ok".into(), web: web.into(), ..Default::default() };
            let all = options(&json!({}), &Facts { tv: Some(&tv), ..facts(&immich, &[]) });
            (all.iter().find(|o| o.id == "retire-web").map(|o| o.value.clone()), all.iter().any(|o| o.id == "tv-restart"))
        };
        assert_eq!(shown("installed"), (Some("Remove".into()), false));
        assert_eq!(shown("pending"), (Some("Goes at the next restart".into()), true));
        assert_eq!(shown("none"), (None, false));
        assert_eq!(shown(""), (None, false));
    }

    #[test]
    fn rows_for_dropped_features_never_show() {
        let immich = Immich::default();
        let all = options(&json!({}), &facts(&immich, &[]));
        for dropped in ["dark-mode", "focus-strength", "dock-hide", "dock-dim", "dock-blur", "liquid-dock", "language", "video-hours", "fps-counter"] {
            assert!(all.iter().all(|o| o.id != dropped), "{dropped}");
        }
        let Kind::List { labels, .. } = &find(&all, "wp-type").kind else { panic!() };
        assert_eq!(labels, &["Pictures", "Plain"]);
    }

    #[test]
    fn a_list_of_two_flips_in_place_and_a_longer_one_opens_its_page() {
        let immich = Immich::default();
        let all = options(&json!({}), &facts(&immich, &[]));
        assert!(find(&all, "brightness-on").flips());
        assert!(!find(&all, "brightness-on").opens_a_page());
        assert!(find(&all, "night-dim").opens_a_page());
        assert!(find(&all, "immich-url").opens_a_page());
        assert!(!find(&all, "tv-scripts").opens_a_page());
    }

    #[test]
    fn picking_a_choice_writes_the_web_apps_keys() {
        let immich = Immich::default();
        let cities = [City { name: "Leeds".into(), code: "GB".into(), country: "United Kingdom".into() }, City { name: "Paris".into(), code: "FR".into(), country: "France".into() }];
        let config = json!({"weatherCity": "Leeds", "weatherCountry": "GB"});
        let all = options(&config, &facts(&immich, &cities));
        assert_eq!(chosen(find(&all, "home-forecast"), 1, &config, &cities), Ok(vec![("homeForecast".into(), json!(true)), ("homeForecastSize".into(), json!("small"))]));
        assert_eq!(chosen(find(&all, "first-row"), 0, &config, &cities), Ok(vec![("firstRow".into(), json!(5))]));
        assert_eq!(chosen(find(&all, "weather-country"), 2, &config, &cities), Ok(vec![("weatherCountry".into(), json!("FR")), ("weatherCity".into(), json!(""))]));
        assert_eq!(switched("show-weather", &config), Some(vec![("showWeather".into(), json!(false))]));
    }

    #[test]
    fn a_city_list_follows_the_country_and_a_city_sets_its_country_when_none_is() {
        let immich = Immich::default();
        let cities = [City { name: "Leeds".into(), code: "GB".into(), country: "United Kingdom".into() }, City { name: "Paris".into(), code: "FR".into(), country: "France".into() }];
        let in_france = options(&json!({"weatherCountry": "FR"}), &facts(&immich, &cities));
        let Kind::List { labels, .. } = &find(&in_france, "weather-city").kind else { panic!() };
        assert_eq!(labels, &["Not set", "Auto", "Paris"]);
        let anywhere = json!({});
        let all = options(&anywhere, &facts(&immich, &cities));
        assert_eq!(chosen(find(&all, "weather-city"), 2, &anywhere, &cities), Ok(vec![("weatherCity".into(), json!("Leeds")), ("weatherCountry".into(), json!("GB"))]));
    }

    #[test]
    fn the_screensaver_keeps_at_least_one_immich_source() {
        let immich = Immich { ready: true, ..Immich::default() };
        let config = json!({"ambientOn": true, "saverShow": "immich", "ambient": "immich:recent"});
        let all = options(&config, &facts(&immich, &[]));
        let sources = find(&all, "ambient");
        assert_eq!(chosen(sources, 0, &config, &[]), Err("Keep at least one source".into()));
        assert_eq!(chosen(sources, 3, &config, &[]), Ok(vec![("ambient".into(), json!("immich:recent|immich:favorites"))]));
    }

    #[test]
    fn the_brightness_schedule_is_written_in_time_order_as_watch_sh_reads_it() {
        let config = json!({"brightnessOn": true, "blDayAt": "07:30", "blDay": 90, "blEveningAt": "18:00", "blEvening": 50, "blNightAt": "00:30", "blNight": 10, "blEye": "night", "blRamp": 30});
        assert_eq!(brightness_schedule(&config), "00:30 10 on\n07:30 90 off\n18:00 50 off\nramp 30\n");
        assert_eq!(brightness_schedule(&json!({})), "");
    }
}
