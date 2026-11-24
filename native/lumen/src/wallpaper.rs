use crate::store;
use crate::util::http_get;

const RESIZER: &str = "https://images.weserv.nl/";
pub const SHOWN: &str = "wallpaper.jpg";
pub const SHOWN_PICK: &str = "wallpaper.pick";
pub const NEXT: &str = "wallpaper-next.jpg";
pub const NEXT_PICK: &str = "wallpaper-next.pick";

pub fn url_of(pictures_tsv: &str, pick: &str) -> Option<String> {
    // ponytail: local:N picks (the five bundled pictures) are skipped; bundle them when someone picks one
    let id = pick.strip_prefix("cloud:")?;
    listed(pictures_tsv, id).map(|fields| fields[1].to_string())
}

fn listed<'a>(pictures_tsv: &'a str, id: &str) -> Option<Vec<&'a str>> {
    pictures_tsv.lines().map(|line| line.split('\t').collect::<Vec<_>>()).find(|fields| fields.len() >= 2 && fields[0] == id)
}

pub fn caption_of(pictures_tsv: &str, pick: &str) -> Option<(String, String)> {
    let fields = listed(pictures_tsv, pick.strip_prefix("cloud:")?)?;
    let title = fields.get(2).copied().unwrap_or("");
    (!title.is_empty()).then(|| (title.to_string(), fields.get(3).copied().unwrap_or("").to_string()))
}

fn url_part(text: &str) -> String {
    text.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

pub fn resized(url: &str, width: u32) -> String {
    let bare = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")).unwrap_or(url);
    format!("{RESIZER}?url={}&w={}&h={}&fit=cover&q={}&output=jpg", url_part(bare), width, (width as f32 * 9.0 / 16.0).round() as u32, if width > 800 { 82 } else { 70 })
}

pub fn another(picks: &[String], shown: Option<&str>, chance: u64) -> Option<String> {
    let others: Vec<&String> = picks.iter().filter(|pick| Some(pick.as_str()) != shown).collect();
    let from = if others.is_empty() { picks.iter().collect() } else { others };
    if from.is_empty() {
        return None;
    }
    Some(from[(chance % from.len() as u64) as usize].clone())
}

pub fn promote_next() -> bool {
    if !store::has(NEXT) {
        return false;
    }
    std::fs::rename(store::path(NEXT), store::path(SHOWN)).is_ok() && std::fs::rename(store::path(NEXT_PICK), store::path(SHOWN_PICK)).is_ok()
}

pub fn fetch(pick: String, url: String, as_next: bool, then: impl FnOnce(Result<(), String>) + Send + 'static) {
    std::thread::spawn(move || {
        let bytes = match http_get(&resized(&url, 1920)) {
            Ok(bytes) => bytes,
            Err(why) => return then(Err(format!("{pick}: {why}"))),
        };
        let (picture, key) = if as_next { (NEXT, NEXT_PICK) } else { (SHOWN, SHOWN_PICK) };
        let kept = store::write(picture, &bytes) && store::write(key, pick.as_bytes());
        then(if kept { Ok(()) } else { Err(format!("{pick}: not kept")) });
    });
}

pub const SETTLE_ON_HOME_MS: u64 = 30_000;

#[derive(Default)]
pub struct HomeClock {
    last_tick: Option<u32>,
    was_on_home: bool,
    on_home_ms: u64,
    settled_ms: u64,
}

impl HomeClock {
    pub fn due(&mut self, on_home: bool, now: u32, minutes: f64) -> bool {
        let step = self.last_tick.map_or(0, |last| now.wrapping_sub(last) as u64);
        self.last_tick = Some(now);
        if self.was_on_home && on_home {
            self.on_home_ms += step;
            self.settled_ms += step;
        } else if self.was_on_home {
            self.on_home_ms += step;
        }
        if !on_home {
            self.settled_ms = 0;
        }
        self.was_on_home = on_home;
        minutes > 0.0 && on_home && self.settled_ms >= SETTLE_ON_HOME_MS && self.on_home_ms as f64 >= minutes * 60_000.0
    }

    pub fn changed(&mut self) {
        self.on_home_ms = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wallpaper_timer_counts_only_home_time_and_waits_half_a_minute_after_coming_back() {
        let mut clock = HomeClock::default();
        assert!(!clock.due(true, 0, 1.0));
        assert!(!clock.due(false, 50_000, 1.0));
        assert!(!clock.due(false, 400_000, 1.0));
        assert!(!clock.due(true, 410_000, 1.0));
        assert!(!clock.due(true, 430_000, 1.0));
        assert!(clock.due(true, 441_000, 1.0));
        clock.changed();
        assert!(!clock.due(true, 442_000, 1.0));
        assert!(!clock.due(true, 600_000, 0.0));
    }

    fn picks(keys: &[&str]) -> Vec<String> {
        keys.iter().map(|k| k.to_string()).collect()
    }

    #[test]
    fn a_pick_finds_its_address_in_the_list_and_an_unknown_or_bundled_one_does_not() {
        let tsv = "abc\thttps://i.redd.it/abc.jpeg\tA lake\t\nabcd\thttps://example.org/d.jpg\tA dell\tGainsborough\n";
        assert_eq!(url_of(tsv, "cloud:abcd").as_deref(), Some("https://example.org/d.jpg"));
        assert_eq!(url_of(tsv, "cloud:ab"), None);
        assert_eq!(url_of(tsv, "local:0"), None);
    }

    #[test]
    fn a_picture_is_captioned_with_its_title_and_any_credit() {
        let tsv = "abc\thttps://i.redd.it/abc.jpeg\tA lake\t\nabcd\thttps://example.org/d.jpg\tA dell\tGainsborough\nxyz\thttps://x.org/x.jpg\n";
        assert_eq!(caption_of(tsv, "cloud:abcd"), Some(("A dell".to_string(), "Gainsborough".to_string())));
        assert_eq!(caption_of(tsv, "cloud:abc"), Some(("A lake".to_string(), String::new())));
        assert_eq!(caption_of(tsv, "cloud:xyz"), None);
    }

    #[test]
    fn the_resized_copy_is_asked_for_at_screen_shape_with_the_address_escaped() {
        assert_eq!(
            resized("https://thumb.wikimedia.org/a/1920px-Van Gogh,_É.jpg", 1920),
            "https://images.weserv.nl/?url=thumb.wikimedia.org%2Fa%2F1920px-Van%20Gogh%2C_%C3%89.jpg&w=1920&h=1080&fit=cover&q=82&output=jpg"
        );
    }

    #[test]
    fn the_next_picture_is_never_the_one_on_screen_unless_it_is_the_only_one() {
        let three = picks(&["cloud:a", "cloud:b", "cloud:c"]);
        for chance in 0..20 {
            assert_ne!(another(&three, Some("cloud:b"), chance).as_deref(), Some("cloud:b"));
        }
        assert_eq!(another(&picks(&["cloud:a"]), Some("cloud:a"), 7).as_deref(), Some("cloud:a"));
        assert_eq!(another(&[], None, 7), None);
    }
}
