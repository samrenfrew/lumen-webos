use serde_json::Value;

pub const HIDDEN_IDS: [&str; 2] = ["com.samrenfrew.lumen.native", "google.assistant.promo"];

#[derive(Clone, PartialEq, Debug)]
pub enum Icon {
    Bundled(String),
    OnTv(String),
    Missing,
}

#[derive(Clone, PartialEq, Debug)]
pub struct App {
    pub id: String,
    pub title: String,
    pub icon: Icon,
    pub color: Option<[f32; 3]>,
}

pub struct Known {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub color: Option<[f32; 3]>,
}

pub fn hex_color(text: &str) -> Option<[f32; 3]> {
    let hex = text.strip_prefix('#')?;
    if hex.len() != 6 || !hex.is_ascii() {
        return None;
    }
    let part = |at: usize| u8::from_str_radix(&hex[at..at + 2], 16).ok().map(|v| v as f32 / 255.0);
    Some([part(0)?, part(2)?, part(4)?])
}

pub fn catalogue(tsv: &str) -> Vec<Known> {
    tsv.lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let (id, title, icon, color) = (fields.next()?, fields.next()?, fields.next().unwrap_or(""), fields.next().unwrap_or(""));
            Some(Known { id: id.into(), title: title.into(), icon: icon.into(), color: hex_color(color) })
        })
        .collect()
}

fn without_tagline(title: &str) -> &str {
    let chars: Vec<(usize, char)> = title.char_indices().collect();
    for window in chars.windows(3) {
        if window[0].1.is_whitespace() && "-–—:|".contains(window[1].1) && window[2].1.is_whitespace() {
            return title[..window[0].0].trim_end();
        }
    }
    title
}

pub fn listed(launch_points: &Value, known: &[Known]) -> Vec<App> {
    let Some(points) = launch_points["launchPoints"].as_array() else { return Vec::new() };
    points
        .iter()
        .filter_map(|point| {
            let (id, title) = (point["id"].as_str()?, point["title"].as_str()?);
            if id.is_empty() || title.is_empty() || HIDDEN_IDS.contains(&id) {
                return None;
            }
            let known = known.iter().find(|k| k.id == id);
            let on_tv = [&point["largeIcon"], &point["icon"]].iter().find_map(|path| path.as_str().filter(|p| !p.is_empty()));
            let icon = match (known.filter(|k| !k.icon.is_empty()), on_tv) {
                (Some(k), _) => Icon::Bundled(k.icon.clone()),
                (None, Some(path)) => Icon::OnTv(path.to_string()),
                (None, None) => Icon::Missing,
            };
            Some(App { id: id.into(), title: known.map_or_else(|| without_tagline(title).to_string(), |k| k.title.clone()), icon, color: known.and_then(|k| k.color) })
        })
        .collect()
}

pub fn in_order(apps: &[App], ids: &[String]) -> Vec<App> {
    ids.iter().filter_map(|id| apps.iter().find(|app| &app.id == id).cloned()).collect()
}

pub fn kept_icon_name(path_on_tv: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in path_on_tv.bytes() {
        hash = (hash ^ byte as u64).wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}.png")
}

pub fn background_of(rgba: &[u8], w: usize, h: usize) -> Option<[f32; 3]> {
    const SPOTS: [(usize, usize); 8] = [(1, 1), (14, 1), (1, 14), (14, 14), (8, 1), (1, 8), (14, 8), (8, 14)];
    if w == 0 || h == 0 || rgba.len() < w * h * 4 {
        return None;
    }
    let mut votes: Vec<([u8; 3], [u8; 3], u32)> = Vec::new();
    for (x, y) in SPOTS {
        let at = (((2 * y + 1) * h / 32) * w + (2 * x + 1) * w / 32) * 4;
        let px = &rgba[at..at + 4];
        if px[3] <= 200 {
            continue;
        }
        let bucket = [px[0] >> 4, px[1] >> 4, px[2] >> 4];
        match votes.iter_mut().find(|vote| vote.0 == bucket) {
            Some(vote) => vote.2 += 1,
            None => votes.push((bucket, [px[0], px[1], px[2]], 1)),
        }
    }
    let most = votes.iter().map(|vote| vote.2).max()?;
    let winner = votes.iter().find(|vote| vote.2 == most)?;
    Some([winner.1[0] as f32 / 255.0, winner.1[1] as f32 / 255.0, winner.1[2] as f32 / 255.0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn known() -> Vec<Known> {
        catalogue("youtube.leanback.v4\tYouTube\tyoutube.leanback.v4.png\t\ncom.webos.app.discovery\tApp Store\t\t#a50034\n")
    }

    #[test]
    fn a_known_app_takes_the_catalogues_title_icon_and_colour() {
        let apps = listed(&json!({"launchPoints": [
            {"id": "youtube.leanback.v4", "title": "YouTube AdFree", "largeIcon": "/media/developer/yt/largeIcon.png"},
            {"id": "com.webos.app.discovery", "title": "Apps", "largeIcon": "/mnt/store.png"}
        ]}), &known());
        assert_eq!(apps[0], App { id: "youtube.leanback.v4".into(), title: "YouTube".into(), icon: Icon::Bundled("youtube.leanback.v4.png".into()), color: None });
        assert_eq!(apps[1].title, "App Store");
        assert_eq!(apps[1].icon, Icon::OnTv("/mnt/store.png".into()));
        assert_eq!(apps[1].color, Some([165.0 / 255.0, 0.0, 52.0 / 255.0]));
    }

    #[test]
    fn an_unknown_app_loses_its_tagline_and_falls_back_to_its_small_icon() {
        let apps = listed(&json!({"launchPoints": [
            {"id": "spotify", "title": "Spotify - Music and Podcasts", "largeIcon": "", "icon": "/a/icon.png"},
            {"id": "plain", "title": "Wi-Fi Tool"}
        ]}), &known());
        assert_eq!(apps[0].title, "Spotify");
        assert_eq!(apps[0].icon, Icon::OnTv("/a/icon.png".into()));
        assert_eq!(apps[1].title, "Wi-Fi Tool");
        assert_eq!(apps[1].icon, Icon::Missing);
    }

    #[test]
    fn lumen_itself_and_entries_with_no_id_or_title_are_left_out() {
        let apps = listed(&json!({"launchPoints": [
            {"id": "com.samrenfrew.lumen.native", "title": "Lumen Native"},
            {"id": "", "title": "Nameless"},
            {"id": "untitled"},
            {"id": "kept", "title": "Kept"}
        ]}), &known());
        assert_eq!(apps.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(), ["kept"]);
    }

    #[test]
    fn the_home_screen_shows_the_chosen_apps_in_the_chosen_order_without_the_missing_ones() {
        let apps = listed(&json!({"launchPoints": [{"id": "a", "title": "A"}, {"id": "b", "title": "B"}, {"id": "c", "title": "C"}]}), &known());
        let shown = in_order(&apps, &["c".to_string(), "gone".to_string(), "a".to_string()]);
        assert_eq!(shown.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(), ["c", "a"]);
    }

    #[test]
    fn an_icons_background_is_the_colour_most_of_its_edges_share() {
        let (w, h) = (32, 32);
        let mut rgba = vec![0u8; w * h * 4];
        for (i, px) in rgba.chunks_exact_mut(4).enumerate() {
            let on_logo = i % w < 6 && (12..20).contains(&(i / w));
            px.copy_from_slice(if on_logo { &[255, 0, 0, 255] } else { &[250, 250, 250, 255] });
        }
        assert_eq!(background_of(&rgba, w, h), Some([250.0 / 255.0; 3]));
    }

    #[test]
    fn a_see_through_icon_gives_no_background() {
        assert_eq!(background_of(&vec![0u8; 16 * 16 * 4], 16, 16), None);
    }

    #[test]
    fn kept_icon_names_are_plain_file_names_that_differ_by_path() {
        let (a, b) = (kept_icon_name("/usr/palm/a/icon.png"), kept_icon_name("/usr/palm/b/icon.png"));
        assert_ne!(a, b);
        assert!(a.len() == 20 && a.chars().all(|c| c.is_ascii_hexdigit() || c == '.' || c == 'p' || c == 'n' || c == 'g'));
    }

    #[test]
    fn colours_are_read_from_six_digit_hex_only() {
        assert_eq!(hex_color("#ffffff"), Some([1.0; 3]));
        assert_eq!(hex_color("black"), None);
        assert_eq!(hex_color("#fff"), None);
        assert_eq!(hex_color("#ffééff"), None);
    }
}
