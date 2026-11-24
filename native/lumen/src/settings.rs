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
    pub first_row: usize,
    pub clock_format: ClockFormat,
    pub show_am_pm: bool,
    pub wallpaper_type: String,
    pub wallpaper_picks: Vec<String>,
    pub new_wallpaper_on_launch: bool,
    pub reduce_motion: bool,
}

fn strings(value: &Value) -> Option<Vec<String>> {
    Some(value.as_array()?.iter().filter_map(|item| item.as_str().map(String::from)).collect())
}

impl Settings {
    pub fn defaults() -> Settings {
        Settings {
            app_ids: Vec::new(),
            first_row: 6,
            clock_format: ClockFormat::FollowTv,
            show_am_pm: true,
            wallpaper_type: "pictures".into(),
            wallpaper_picks: Vec::new(),
            new_wallpaper_on_launch: true,
            reduce_motion: false,
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
    fn a_field_that_is_missing_or_wrong_takes_its_default() {
        let saved = Settings::parse(r#"{"appIds":"none","firstRow":9,"clockFormat":"13h","showAmPm":"yes","wallpaperType":"cube","reduceMotion":1}"#).unwrap();
        assert_eq!(saved, Settings::defaults());
    }

    #[test]
    fn text_that_is_not_settings_is_refused() {
        assert_eq!(Settings::parse("not json"), None);
        assert_eq!(Settings::parse("[1,2]"), None);
    }
}
