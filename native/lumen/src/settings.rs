use serde_json::Value;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ClockFormat {
    FollowTv,
    Twelve,
    TwentyFour,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Settings {
    pub app_ids: Vec<String>,
    pub source_ids: Vec<String>,
    pub first_row: usize,
    pub clock_format: ClockFormat,
    pub show_am_pm: bool,
    pub wallpaper_type: String,
    pub wallpaper_picks: Vec<String>,
    pub new_wallpaper_on_launch: bool,
    pub reduce_motion: bool,
    pub show_weather: bool,
    pub home_forecast: Forecast,
    pub weather_city: String,
    pub captions: bool,
    pub hide_names: bool,
    pub folders: Vec<Folder>,
    pub wp_auto_minutes: f64,
    pub wallpaper_dim: String,
    pub live_tv_row: bool,
    pub live_channels: Option<Vec<String>>,
    pub saver_immich: bool,
    pub saver_sources: String,
    pub saver_seconds: u32,
    pub saver_forecast: Forecast,
    pub immich_recent_days: u32,
    pub immich_picks: Vec<String>,
    pub guide_full: bool,
    pub idle_seconds: u32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Forecast {
    Off,
    Small,
    Large,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Folder {
    pub name: String,
    pub app_ids: Vec<String>,
}

fn strings(value: &Value) -> Option<Vec<String>> {
    Some(value.as_array()?.iter().filter_map(|item| item.as_str().map(String::from)).collect())
}

impl Settings {
    pub fn defaults() -> Settings {
        Settings {
            app_ids: Vec::new(),
            source_ids: Vec::new(),
            first_row: 6,
            clock_format: ClockFormat::FollowTv,
            show_am_pm: true,
            wallpaper_type: "pictures".into(),
            wallpaper_picks: Vec::new(),
            new_wallpaper_on_launch: true,
            reduce_motion: false,
            show_weather: true,
            home_forecast: Forecast::Off,
            weather_city: String::new(),
            captions: true,
            hide_names: false,
            folders: Vec::new(),
            wp_auto_minutes: 0.0,
            wallpaper_dim: "off".into(),
            live_tv_row: true,
            live_channels: None,
            saver_immich: false,
            saver_sources: "immich:recent".into(),
            saver_seconds: 30,
            saver_forecast: Forecast::Off,
            immich_recent_days: 180,
            immich_picks: vec!["immich:recent".into()],
            guide_full: false,
            idle_seconds: 0,
        }
    }

    pub fn parse(text: &str) -> Option<Settings> {
        let whole: Value = serde_json::from_str(text).ok()?;
        let saved = if whole.get("config").is_some_and(Value::is_object) { &whole["config"] } else { &whole };
        if !saved.is_object() {
            return None;
        }
        let defaults = Settings::defaults();
        Some(Settings {
            app_ids: strings(&saved["appIds"]).unwrap_or(defaults.app_ids),
            source_ids: strings(&saved["sourceIds"]).unwrap_or(defaults.source_ids),
            first_row: match saved["firstRow"].as_u64() {
                Some(5) => 5,
                Some(7) => 7,
                _ => 6,
            },
            clock_format: match saved["clockFormat"].as_str() {
                Some("12h") => ClockFormat::Twelve,
                Some("24h") => ClockFormat::TwentyFour,
                _ => ClockFormat::FollowTv,
            },
            show_am_pm: saved["showAmPm"].as_bool() != Some(false),
            wallpaper_type: match saved["wallpaperType"].as_str() {
                Some(kind @ ("video" | "immich" | "plain" | "pictures")) => kind.to_string(),
                _ => defaults.wallpaper_type,
            },
            wallpaper_picks: strings(&saved["wallpaperPicks"]).unwrap_or(defaults.wallpaper_picks),
            new_wallpaper_on_launch: saved["wpNewOnLaunch"].as_bool() != Some(false),
            reduce_motion: saved["reduceMotion"].as_bool() == Some(true),
            show_weather: saved["showWeather"].as_bool() != Some(false),
            home_forecast: match (saved["homeForecast"].as_bool(), saved["homeForecastSize"].as_str()) {
                (Some(true), Some("large")) => Forecast::Large,
                (Some(true), _) => Forecast::Small,
                _ => Forecast::Off,
            },
            weather_city: saved["weatherCity"].as_str().unwrap_or("").to_string(),
            captions: saved["wallpaperCaptions"].as_bool() != Some(false),
            hide_names: saved["hideNames"].as_bool() == Some(true),
            folders: saved["folders"].as_array().map_or_else(Vec::new, |all| {
                all.iter().filter_map(|f| Some(Folder { name: f["name"].as_str()?.to_string(), app_ids: strings(&f["appIds"])? })).filter(|f| !f.name.is_empty()).collect()
            }),
            wp_auto_minutes: saved["wpAutoMinutes"].as_f64().filter(|m| *m >= 0.0).unwrap_or(0.0),
            wallpaper_dim: match saved["wallpaperDim"].as_str() {
                Some(dim @ ("night" | "always")) => dim.to_string(),
                _ => "off".to_string(),
            },
            live_tv_row: saved["liveTvRow"].as_bool() != Some(false),
            live_channels: strings(&saved["liveChannels"]),
            saver_immich: match saved["saverShow"].as_str() {
                Some("immich") => true,
                Some("wallpaper") => false,
                _ => saved["ambientOn"].as_bool() == Some(true),
            },
            saver_sources: saved["ambient"].as_str().filter(|s| s.starts_with("immich:")).map_or(defaults.saver_sources, String::from),
            saver_seconds: saved["ambientSeconds"].as_u64().filter(|s| [10, 20, 30, 60, 120, 300].contains(s)).map_or(defaults.saver_seconds, |s| s as u32),
            saver_forecast: match (saved["saverForecast"].as_bool(), saved["saverForecastSize"].as_str()) {
                (Some(true), Some("large")) => Forecast::Large,
                (Some(true), _) => Forecast::Small,
                _ => Forecast::Off,
            },
            immich_recent_days: saved["immichRecentDays"].as_u64().filter(|d| [30, 90, 180, 365].contains(d)).map_or(defaults.immich_recent_days, |d| d as u32),
            immich_picks: strings(&saved["immichPicks"]).filter(|p| !p.is_empty()).unwrap_or(defaults.immich_picks),
            guide_full: saved["guideFull"].as_bool() == Some(true),
            idle_seconds: saved["idleSeconds"].as_u64().filter(|s| [0, 30, 60, 120, 300].contains(s)).map_or(defaults.idle_seconds, |s| s as u32),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_copy_the_web_app_keeps_on_the_tv_is_read_from_inside_its_wrapper() {
        let saved = Settings::parse(r#"{"version":1,"config":{"appIds":["a","b"],"firstRow":7,"clockFormat":"24h","wpNewOnLaunch":false,"wallpaperPicks":["cloud:x"]}}"#).unwrap();
        assert_eq!(saved.app_ids, ["a", "b"]);
        assert_eq!(saved.first_row, 7);
        assert_eq!(saved.clock_format, ClockFormat::TwentyFour);
        assert!(!saved.new_wallpaper_on_launch);
        assert_eq!(saved.wallpaper_picks, ["cloud:x"]);
    }

    #[test]
    fn folders_keep_their_names_and_apps_and_a_nameless_one_is_dropped() {
        let saved = Settings::parse(r#"{"folders":[{"name":"Games","appIds":["a","b"]},{"name":"","appIds":["c"]},{"appIds":["d"]}],"homeForecast":true,"homeForecastSize":"small","weatherCity":"Leeds"}"#).unwrap();
        assert_eq!(saved.folders, [Folder { name: "Games".into(), app_ids: vec!["a".into(), "b".into()] }]);
        assert_eq!(saved.home_forecast, Forecast::Small);
        assert_eq!(saved.weather_city, "Leeds");
    }

    #[test]
    fn a_field_that_is_missing_or_wrong_takes_its_default() {
        let saved = Settings::parse(r#"{"appIds":"none","firstRow":9,"clockFormat":"13h","showAmPm":"yes","wallpaperType":"cube","reduceMotion":1}"#).unwrap();
        assert_eq!(saved, Settings::defaults());
    }

    #[test]
    fn the_home_screensaver_waits_only_one_of_the_offered_times_and_otherwise_never_starts() {
        assert_eq!(Settings::parse(r#"{"idleSeconds":120}"#).unwrap().idle_seconds, 120);
        assert_eq!(Settings::parse(r#"{"idleSeconds":45}"#).unwrap().idle_seconds, 0);
        assert_eq!(Settings::parse(r#"{}"#).unwrap().idle_seconds, 0);
    }

    #[test]
    fn the_live_tv_row_is_on_until_switched_off_and_its_channels_are_unpicked_until_a_list_is_saved() {
        let unpicked = Settings::parse(r#"{"liveTvRow":"no","liveChannels":null}"#).unwrap();
        assert_eq!((unpicked.live_tv_row, unpicked.live_channels), (true, None));
        let picked = Settings::parse(r#"{"liveTvRow":false,"liveChannels":["1_41_2_102_16522_17472_9018",3]}"#).unwrap();
        assert_eq!((picked.live_tv_row, picked.live_channels), (false, Some(vec!["1_41_2_102_16522_17472_9018".to_string()])));
    }

    #[test]
    fn the_screensaver_shows_immich_only_when_asked_and_keeps_to_the_web_apps_choices() {
        let saved = Settings::parse(r#"{"saverShow":"immich","ambient":"immich:recent|immich:favorites","ambientSeconds":60,"immichRecentDays":90,"saverForecast":true,"saverForecastSize":"large"}"#).unwrap();
        assert_eq!((saved.saver_immich, saved.saver_sources.as_str(), saved.saver_seconds, saved.immich_recent_days, saved.saver_forecast), (true, "immich:recent|immich:favorites", 60, 90, Forecast::Large));
        let odd = Settings::parse(r#"{"ambient":"/etc/passwd","ambientSeconds":7,"immichRecentDays":1000,"ambientOn":true}"#).unwrap();
        assert_eq!((odd.saver_immich, odd.saver_sources.as_str(), odd.saver_seconds, odd.immich_recent_days), (true, "immich:recent", 30, 180));
    }

    #[test]
    fn text_that_is_not_settings_is_refused() {
        assert_eq!(Settings::parse("not json"), None);
        assert_eq!(Settings::parse("[1,2]"), None);
    }
}
