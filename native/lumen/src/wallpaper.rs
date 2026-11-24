use crate::ffi::{lumen_http_free, lumen_http_get};
use crate::store;
use std::ffi::CString;
use std::ptr;

const RESIZER: &str = "https://images.weserv.nl/";
pub const SHOWN: &str = "wallpaper.jpg";
pub const SHOWN_PICK: &str = "wallpaper.pick";
pub const NEXT: &str = "wallpaper-next.jpg";
pub const NEXT_PICK: &str = "wallpaper-next.pick";

pub fn url_of(pictures_tsv: &str, pick: &str) -> Option<String> {
    // ponytail: local:N picks (the five bundled pictures) are skipped; bundle them when someone picks one
    let id = pick.strip_prefix("cloud:")?;
    pictures_tsv.lines().find_map(|line| line.split_once('\t').filter(|(listed, _)| *listed == id).map(|(_, url)| url.to_string()))
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
        let Ok(c_url) = CString::new(resized(&url, 1920)) else { return then(Err("bad url".into())) };
        let (mut data, mut len) = (ptr::null_mut(), 0usize);
        let status = unsafe { lumen_http_get(c_url.as_ptr(), &mut data, &mut len) };
        if status != 200 || data.is_null() {
            return then(Err(format!("{pick}: status {status}")));
        }
        let (picture, key) = if as_next { (NEXT, NEXT_PICK) } else { (SHOWN, SHOWN_PICK) };
        let kept = store::write(picture, unsafe { std::slice::from_raw_parts(data, len) }) && store::write(key, pick.as_bytes());
        unsafe { lumen_http_free(data) }
        then(if kept { Ok(()) } else { Err(format!("{pick}: not kept")) });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picks(keys: &[&str]) -> Vec<String> {
        keys.iter().map(|k| k.to_string()).collect()
    }

    #[test]
    fn a_pick_finds_its_address_in_the_list_and_an_unknown_or_bundled_one_does_not() {
        let tsv = "abc\thttps://i.redd.it/abc.jpeg\nabcd\thttps://example.org/d.jpg\n";
        assert_eq!(url_of(tsv, "cloud:abcd").as_deref(), Some("https://example.org/d.jpg"));
        assert_eq!(url_of(tsv, "cloud:ab"), None);
        assert_eq!(url_of(tsv, "local:0"), None);
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
