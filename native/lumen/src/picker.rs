use crate::draw::{decoded, decoded_bytes, Decoded};
use crate::util::{http_get, http_request, utc_stamp};
use crate::{clock, note, store, wallpaper};
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::fs;
use std::time::Instant;

const SHOWN_MAX: usize = 40;
const PLACES_MAX: usize = 300;
const STACKS_EVERY_S: u64 = 6 * 3600;
const PAIR_BOX: f64 = (960.0 - 4.0) / 1080.0;
const SCREEN_BOX: f64 = 16.0 / 9.0;
const WIDE_ENOUGH: f64 = 1.2;
const TIERS: [(f64, f64); 3] = [(1.0, 0.5), (1.0 / 3.0, 0.25), (1.0 / 12.0, 0.25)];
const WEB_LUMEN_PICTURES: &str = "/media/developer/apps/usr/palm/applications/com.samrenfrew.lumen/wallpapers";
const NO_FACE: f64 = -1.0;
const FACES_UNREAD: f64 = -2.0;

pub struct Server {
    url: String,
    key: String,
}

pub struct Photo {
    pub picture: Decoded,
    pub x: f32,
    pub y: f32,
}

pub struct Picture {
    pub id: String,
    pub photos: Vec<Photo>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Wants {
    pub immich: bool,
    pub sources: String,
    pub recent_days: u32,
    pub picks: Vec<String>,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Pool {
    pub sel: String,
    pub ids: Vec<String>,
    pub shown: Vec<String>,
    pub last: Vec<String>,
    pub days: Map<String, Value>,
}

pub struct Chance(u64);

impl Chance {
    pub fn new() -> Chance {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64);
        Chance(nanos ^ (std::process::id() as u64) << 32 | 1)
    }

    pub fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        if n == 0 { 0 } else { (self.0 % n as u64) as usize }
    }
}

pub fn state_dir() -> String {
    let dir = format!("/tmp/lumen-saver-{}", unsafe { crate::ffi::getuid() });
    if fs::create_dir(&dir).is_ok() {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o755));
    }
    dir
}

fn state_path(name: &str) -> String {
    format!("{}/{}", state_dir(), name)
}

fn read_json(path: &str) -> Value {
    fs::read_to_string(path).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or(Value::Null)
}

fn write_json(path: &str, value: &Value) {
    let part = format!("{path}.part");
    if fs::write(&part, value.to_string()).is_ok() {
        let _ = fs::rename(&part, path);
    }
}

pub fn server_from(text: &str) -> Option<Server> {
    let j: Value = serde_json::from_str(text).ok()?;
    let url = j["url"].as_str()?.trim().trim_end_matches('/').to_string();
    let key = j["key"].as_str()?.trim().to_string();
    let plain_url = (url.starts_with("http://") || url.starts_with("https://")) && url.bytes().all(|b| b > b' ' && !b"\"'\\".contains(&b));
    let plain_key = !key.is_empty() && key.len() < 128 && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    (plain_url && plain_key).then_some(Server { url, key })
}

pub fn own_server_copy() -> String {
    format!("/tmp/lumen-native-{}/immich.json", unsafe { crate::ffi::getuid() })
}

fn find_server(app_dir: &str) -> Option<Server> {
    [own_server_copy(), format!("{app_dir}/config/immich.json")].iter().find_map(|path| server_from(&fs::read_to_string(path).ok()?))
}

fn valid_id(id: &str) -> bool {
    id.len() == 36 && id.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b) || b == b'-')
}

fn ids_of(pic: &str) -> Vec<&str> {
    pic.split('+').collect()
}

fn ratio(a: &Value) -> f64 {
    let exif = &a["exifInfo"];
    let (mut w, mut h) = (a["width"].as_f64().unwrap_or(0.0), a["height"].as_f64().unwrap_or(0.0));
    let from_exif = w == 0.0;
    if w == 0.0 {
        w = exif["exifImageWidth"].as_f64().unwrap_or(0.0);
    }
    if h == 0.0 {
        h = exif["exifImageHeight"].as_f64().unwrap_or(0.0);
    }
    if from_exif && matches!(exif["orientation"].as_str(), Some("5" | "6" | "7" | "8")) {
        std::mem::swap(&mut w, &mut h);
    }
    if w > 0.0 && h > 0.0 { w / h } else { 0.0 }
}

fn has_people(a: &Value) -> bool {
    a["people"].as_array().is_some_and(|p| !p.is_empty())
}

fn day_of(a: &Value) -> Option<String> {
    let d = a["localDateTime"].as_str().filter(|d| !d.is_empty()).or_else(|| a["fileCreatedAt"].as_str())?;
    d.get(..10).map(String::from)
}

fn camera(a: &Value) -> bool {
    a["exifInfo"]["make"].as_str().is_some_and(|m| !m.is_empty())
}

pub fn place(frame: Option<[f64; 3]>, area: f64) -> (f32, f32) {
    let Some([shape, face_x, face_y]) = frame.filter(|f| f[0] > 0.0) else { return (0.5, 0.4) };
    let k = (area / shape).max(1.0);
    let (w, h) = (shape * k, k);
    let x = if face_x >= 0.0 && w > area + 0.001 { ((area * 0.5 - face_x * w) / (area - w)).clamp(0.0, 1.0) } else { 0.5 };
    let y = if h <= 1.001 {
        0.5
    } else if face_y >= 0.0 {
        ((0.42 - face_y * h) / (1.0 - h)).clamp(0.0, 1.0)
    } else {
        0.3
    };
    (x as f32, y as f32)
}

fn face_centre(faces: &Value) -> Option<(f64, f64)> {
    let (mut x1, mut y1, mut x2, mut y2) = (1.0f64, 1.0f64, 0.0f64, 0.0f64);
    for f in faces.as_array()? {
        let (w, h) = (f["imageWidth"].as_f64().unwrap_or(0.0), f["imageHeight"].as_f64().unwrap_or(0.0));
        let Some(bx1) = f["boundingBoxX1"].as_f64() else { continue };
        if w == 0.0 || h == 0.0 {
            continue;
        }
        x1 = x1.min(bx1 / w);
        y1 = y1.min(f["boundingBoxY1"].as_f64().unwrap_or(0.0) / h);
        x2 = x2.max(f["boundingBoxX2"].as_f64().unwrap_or(0.0) / w);
        y2 = y2.max(f["boundingBoxY2"].as_f64().unwrap_or(0.0) / h);
    }
    (x2 > x1 && y2 > y1).then_some(((x1 + x2) / 2.0, (y1 + y2) / 2.0))
}

pub fn arrange(list: &[Value], days: &mut Map<String, Value>, singles_only: bool, chance: &mut Chance) -> Vec<String> {
    let mut out = Vec::new();
    let mut tall = Vec::new();
    for a in list {
        let Some(id) = a["id"].as_str().filter(|id| valid_id(id)) else { continue };
        if let Some(d) = day_of(a) {
            days.insert(id.to_string(), json!(d));
        }
        let r = ratio(a);
        if r >= WIDE_ENOUGH || (r == 0.0 && !singles_only) {
            out.push(id.to_string());
        } else if r <= 1.0 / WIDE_ENOUGH && !singles_only {
            tall.push((id, day_of(a)));
        }
    }
    let mut used = vec![false; tall.len()];
    for i in 0..tall.len() {
        if used[i] {
            continue;
        }
        used[i] = true;
        let partner = (i + 1..tall.len()).find(|&j| !used[j] && (tall[i].1.is_none() || tall[j].1.is_none() || tall[i].1 != tall[j].1));
        if let Some(j) = partner {
            used[j] = true;
            out.push(format!("{}+{}", tall[i].0, tall[j].0));
        }
    }
    for i in (1..out.len()).rev() {
        out.swap(i, chance.below(i + 1));
    }
    out
}

pub fn take(pool: &mut Pool, strict: bool, avoided: &[&str]) -> Option<String> {
    let shown = pool.shown.clone();
    pool.ids.retain(|pic| ids_of(pic).iter().all(|id| !shown.iter().any(|s| s == id) && !avoided.contains(id)));
    let clashes = |pic: &str| ids_of(pic).iter().any(|id| pool.days.get(*id).and_then(Value::as_str).is_some_and(|d| pool.last.iter().any(|l| l == d)));
    let at = pool.ids.iter().position(|pic| !strict || !clashes(pic))?;
    Some(pool.ids.remove(at))
}

fn finish(pool: &mut Pool, pic: Option<&str>) {
    if let Some(pic) = pic {
        let ids = ids_of(pic);
        let last = ids.iter().filter_map(|id| pool.days.get(*id).and_then(Value::as_str).map(String::from)).collect();
        let mut shown: Vec<String> = ids.iter().map(|s| s.to_string()).collect();
        shown.extend(pool.shown.iter().cloned());
        shown.truncate(SHOWN_MAX);
        pool.shown = shown;
        pool.last = last;
    }
    let mut kept = Map::new();
    for pic in pool.ids.iter().map(String::as_str).chain(pic) {
        for id in ids_of(pic) {
            if let Some(d) = pool.days.get(id) {
                kept.insert(id.to_string(), d.clone());
            }
        }
    }
    pool.days = kept;
}

fn pool_from(v: &Value) -> Pool {
    let strings = |key: &str| v[key].as_array().map_or_else(Vec::new, |a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect());
    Pool { sel: v["sel"].as_str().unwrap_or("").to_string(), ids: strings("ids"), shown: strings("shown"), last: strings("last"), days: v["days"].as_object().cloned().unwrap_or_default() }
}

fn pool_json(p: &Pool) -> Value {
    json!({"sel": p.sel, "ids": p.ids, "shown": p.shown, "last": p.last, "days": p.days})
}

fn local_number(pick: &str) -> Option<u32> {
    let n = pick.strip_prefix("local:")?;
    (!n.is_empty() && n.len() <= 2 && n.bytes().all(|b| b.is_ascii_digit())).then(|| n.parse().ok()).flatten()
}

pub struct Picker {
    pool_file: &'static str,
    keeps_ready: bool,
    singles_only: bool,
    app_dir: String,
    wants: Wants,
    server: Option<Server>,
    places: Vec<(String, [f64; 3])>,
    places_read: bool,
    last_id: String,
    opened: bool,
    chance: Chance,
}

impl Picker {
    pub fn for_screensaver(app_dir: &str, wants: Wants) -> Picker {
        Picker::new("pool.json", true, false, app_dir, wants)
    }

    pub fn for_wallpaper(app_dir: &str, wants: Wants) -> Picker {
        Picker::new("wallpaper-pool.json", false, true, app_dir, wants)
    }

    fn new(pool_file: &'static str, keeps_ready: bool, singles_only: bool, app_dir: &str, wants: Wants) -> Picker {
        Picker { pool_file, keeps_ready, singles_only, app_dir: app_dir.to_string(), wants, server: None, places: Vec::new(), places_read: false, last_id: String::new(), opened: false, chance: Chance::new() }
    }

    pub fn set_wants(&mut self, wants: Wants) {
        self.wants = wants;
    }

    fn api(&self, path: &str, body: Option<&Value>) -> Option<Value> {
        let s = self.server.as_ref()?;
        let text = body.map(Value::to_string);
        match http_request(&format!("{}{}", s.url, path), Some(&s.key), text.as_deref()) {
            Ok(bytes) => serde_json::from_slice(&bytes).ok(),
            Err(why) => {
                note!("immich: {} {}", path.split('?').next().unwrap_or(""), why);
                None
            }
        }
    }

    fn ask(&self, size: usize, taken_after: Option<String>, favorite: bool, album: Option<&str>) -> Vec<Value> {
        let mut body = json!({"type": "IMAGE", "withExif": true, "withPeople": true, "size": size});
        if let Some(after) = taken_after {
            body["takenAfter"] = json!(after);
        }
        if favorite {
            body["isFavorite"] = json!(true);
        }
        if let Some(album) = album {
            body["albumIds"] = json!([album]);
        }
        self.api("/api/search/random", Some(&body)).and_then(|v| v.as_array().cloned()).unwrap_or_default()
    }

    fn tiered(&self, days: u32, size: usize, favorite: bool, all: bool) -> Vec<Value> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for (part, share) in TIERS {
            let back = clock::unix_now().saturating_sub((days as f64 * part).round() as u64 * 86400);
            let after = (!(part == 1.0 && all)).then(|| utc_stamp(back).replace('Z', ".000Z"));
            for a in self.ask((size as f64 * share).round() as usize, after, favorite, None) {
                if a["id"].as_str().is_some_and(|id| seen.insert(id.to_string())) {
                    out.push(a);
                }
            }
        }
        out
    }

    fn read_places(&mut self) {
        if self.places_read {
            return;
        }
        self.places_read = true;
        if let Some(all) = read_json(&state_path("places.json")).as_array() {
            self.places = all.iter().filter_map(|e| Some((e[0].as_str()?.to_string(), [e[1].as_f64()?, e[2].as_f64()?, e[3].as_f64()?]))).collect();
        }
    }

    fn save_places(&mut self) {
        let over = self.places.len().saturating_sub(PLACES_MAX);
        self.places.drain(..over);
        write_json(&state_path("places.json"), &Value::Array(self.places.iter().map(|(id, f)| json!([id, f[0], f[1], f[2]])).collect()));
    }

    fn place_of(&self, id: &str) -> Option<[f64; 3]> {
        self.places.iter().find(|(i, _)| i == id).map(|(_, f)| *f)
    }

    fn set_place(&mut self, id: &str, f: [f64; 3]) {
        self.places.retain(|(i, _)| i != id);
        self.places.push((id.to_string(), f));
    }

    fn remember_shapes(&mut self, list: &[Value]) {
        self.read_places();
        for a in list {
            let Some(id) = a["id"].as_str() else { continue };
            match self.place_of(id) {
                Some(known) if known[1] >= 0.0 => self.set_place(id, known),
                _ => self.set_place(id, [ratio(a), if has_people(a) { FACES_UNREAD } else { NO_FACE }, NO_FACE]),
            }
        }
        self.save_places();
    }

    fn read_faces(&mut self, ids: &[&str]) {
        self.read_places();
        let mut changed = false;
        for id in ids {
            let Some(f) = self.place_of(id).filter(|f| f[1] == FACES_UNREAD) else { continue };
            let centre = self.api(&format!("/api/faces?id={id}"), None).and_then(|faces| face_centre(&faces)).unwrap_or((NO_FACE, NO_FACE));
            self.set_place(id, [f[0], centre.0, centre.1]);
            changed = true;
        }
        if changed {
            self.save_places();
        }
    }

    fn stacked_away(&self) -> HashSet<String> {
        let path = state_path("stacks.json");
        let saved = read_json(&path);
        if clock::unix_now().saturating_sub(saved["at"].as_u64().unwrap_or(0)) < STACKS_EVERY_S {
            return saved["ids"].as_array().map_or_else(HashSet::new, |ids| ids.iter().filter_map(|v| v.as_str().map(String::from)).collect());
        }
        let mut away = HashSet::new();
        for stack in self.api("/api/stacks", None).and_then(|v| v.as_array().cloned()).unwrap_or_default() {
            let cover = stack["primaryAssetId"].as_str();
            for a in stack["assets"].as_array().into_iter().flatten() {
                if let Some(id) = a["id"].as_str().filter(|id| Some(*id) != cover) {
                    away.insert(id.to_string());
                }
            }
        }
        write_json(&path, &json!({"at": clock::unix_now(), "ids": away.iter().collect::<Vec<_>>()}));
        away
    }

    fn keep(&mut self, list: Vec<Value>, broad: bool, away: &HashSet<String>, days: &mut Map<String, Value>) -> Vec<String> {
        let kept: Vec<Value> = list
            .into_iter()
            .filter(|a| a["id"].as_str().is_some_and(|id| !away.contains(id)) && a["type"].as_str().is_none_or(|t| t == "IMAGE") && (!broad || camera(a)))
            .collect();
        self.remember_shapes(&kept);
        arrange(&kept, days, self.singles_only, &mut self.chance)
    }

    fn search_in(&mut self, sel: &str, away: &HashSet<String>, days: &mut Map<String, Value>) -> Vec<String> {
        if sel.contains('|') {
            let parts: Vec<&str> = sel.split('|').filter(|p| !p.is_empty()).take(16).collect();
            let one = parts[self.chance.below(parts.len())].to_string();
            return self.search_in(&one, away, days);
        }
        let recent_days = self.wants.recent_days;
        match sel {
            "immich:memories" => {
                let today = clock::local(clock::unix_now());
                let year = utc_stamp(clock::unix_now())[..4].to_string();
                let path = format!("/api/memories?for={year}-{:02}-{:02}", today.month + 1, today.day);
                let all: Vec<Value> = self.api(&path, None).and_then(|v| v.as_array().cloned()).unwrap_or_default().iter().flat_map(|m| m["assets"].as_array().cloned().unwrap_or_default()).collect();
                let out = self.keep(all, false, away, days);
                if out.is_empty() { self.search_in("immich:recent", away, days) } else { out }
            }
            "immich:favorites" => {
                let list = self.tiered(365, 40, true, true);
                self.keep(list, false, away, days)
            }
            "immich:recent" => {
                let list = self.tiered(recent_days, 40, false, false);
                self.keep(list, true, away, days)
            }
            "immich:recent-people" => {
                let people: Vec<Value> = self.tiered(recent_days, 120, false, false).into_iter().filter(has_people).collect();
                let out = self.keep(people, true, away, days);
                if out.is_empty() { self.search_in("immich:recent", away, days) } else { out }
            }
            album if album.strip_prefix("immich:album:").is_some_and(valid_id) => {
                let list = self.ask(40, None, false, album.strip_prefix("immich:album:"));
                self.keep(list, false, away, days)
            }
            _ => {
                let list = self.ask(40, None, false, None);
                self.keep(list, true, away, days)
            }
        }
    }

    fn next_asset(&mut self, sel: &str, avoid: &str) -> Option<String> {
        let tag = if sel.contains("immich:recent") { format!("{sel}@{}", self.wants.recent_days) } else { sel.to_string() };
        let path = state_path(self.pool_file);
        let mut pool = pool_from(&read_json(&path));
        if pool.sel != tag {
            pool = Pool { sel: tag, ..Pool::default() };
        }
        let avoided: Vec<&str> = if avoid.is_empty() { Vec::new() } else { ids_of(avoid) };
        let mut found = take(&mut pool, true, &avoided);
        if found.is_none() {
            let away = self.stacked_away();
            let mut days = Map::new();
            let batch = self.search_in(sel, &away, &mut days);
            pool.days.extend(days);
            let mut merged = batch.clone();
            merged.extend(pool.ids.iter().filter(|id| !batch.contains(id)).cloned());
            pool.ids = merged;
            found = take(&mut pool, true, &avoided).or_else(|| take(&mut pool, false, &avoided));
            if found.is_none() && !batch.is_empty() {
                pool.shown.clear();
                pool.ids = batch.clone();
                found = take(&mut pool, true, &avoided).or_else(|| take(&mut pool, false, &avoided));
            }
            note!("picker: {} pictures from {}", batch.len(), sel);
        }
        finish(&mut pool, found.as_deref());
        write_json(&path, &pool_json(&pool));
        found
    }

    fn preview(&self, id: &str) -> Option<Vec<u8>> {
        let s = self.server.as_ref()?;
        http_request(&format!("{}/api/assets/{id}/thumbnail?size=preview", s.url), Some(&s.key), None).map_err(|why| note!("immich: preview {}", why)).ok()
    }

    fn from_immich(&mut self) -> Option<(Picture, Vec<Vec<u8>>)> {
        let sources = self.wants.sources.clone();
        let last = self.last_id.clone();
        let pic = self.next_asset(&sources, &last)?;
        let ids = ids_of(&pic);
        self.read_faces(&ids);
        let mut photos = Vec::new();
        let mut jpegs = Vec::new();
        for (i, id) in ids.iter().enumerate() {
            let (x, y) = if ids.len() == 2 { (i as f32, place(self.place_of(id), PAIR_BOX).1) } else { place(self.place_of(id), SCREEN_BOX) };
            let bytes = self.preview(id)?;
            photos.push(Photo { picture: decoded_bytes(&bytes)?, x, y });
            jpegs.push(bytes);
        }
        Some((Picture { id: pic, photos }, jpegs))
    }

    fn keep_ready(&self, pic: &Picture, jpegs: &[Vec<u8>]) {
        let described = state_path("ready.json");
        let _ = fs::remove_file(&described);
        for (i, bytes) in jpegs.iter().enumerate() {
            if fs::write(state_path(&format!("ready-{i}.jpg")), bytes).is_err() {
                return;
            }
        }
        write_json(&described, &json!({"id": pic.id, "xy": pic.photos.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>()}));
    }

    fn take_ready(&self) -> Option<Picture> {
        let meta = read_json(&state_path("ready.json"));
        let id = meta["id"].as_str()?.to_string();
        let xy = meta["xy"].as_array()?;
        if xy.is_empty() || xy.len() > 2 {
            return None;
        }
        let photos = xy.iter().enumerate().map(|(i, p)| Some(Photo { picture: decoded(&state_path(&format!("ready-{i}.jpg")))?, x: p[0].as_f64()? as f32, y: p[1].as_f64()? as f32 })).collect::<Option<Vec<_>>>()?;
        Some(Picture { id, photos })
    }

    fn load_pick(&self, pick: &str) -> Option<Decoded> {
        if let Some(n) = local_number(pick) {
            return [format!("{}/photos/default-{n}.jpg", self.app_dir), format!("{WEB_LUMEN_PICTURES}/default-{n}.jpg")].iter().find_map(|p| decoded(p));
        }
        let id = pick.strip_prefix("cloud:")?;
        let kept = state_path(&format!("cloud-{id}.jpg"));
        if let Some(d) = decoded(&kept) {
            return Some(d);
        }
        let listed = fs::read_to_string(format!("{}/assets/wallpapers.tsv", self.app_dir)).unwrap_or_default();
        let bytes = http_get(&wallpaper::resized(&wallpaper::url_of(&listed, pick)?, 1920)).ok()?;
        let d = decoded_bytes(&bytes)?;
        let _ = fs::write(&kept, &bytes);
        Some(d)
    }

    fn from_picks(&mut self) -> Option<Picture> {
        let listed = fs::read_to_string(format!("{}/assets/wallpapers.tsv", self.app_dir)).unwrap_or_default();
        let usable: Vec<String> = self.wants.picks.iter().filter(|p| local_number(p).is_some() || wallpaper::url_of(&listed, p).is_some()).cloned().collect();
        let others: Vec<&String> = usable.iter().filter(|p| **p != self.last_id).collect();
        let from: Vec<&String> = if others.is_empty() { usable.iter().collect() } else { others };
        let pick = from.get(self.chance.below(from.len()))?.to_string();
        let picture = self.load_pick(&pick)?;
        Some(Picture { id: pick, photos: vec![Photo { picture, x: 0.5, y: 0.5 }] })
    }

    fn from_home_wallpaper(&self) -> Option<Picture> {
        let (id, path) = [("home", store::path(wallpaper::SHOWN)), ("bundled", format!("{}/wallpaper.jpg", self.app_dir))].into_iter().find(|(_, p)| fs::metadata(p).is_ok())?;
        Some(Picture { id: id.to_string(), photos: vec![Photo { picture: decoded(&path)?, x: 0.5, y: 0.5 }] })
    }

    pub fn next(&mut self) -> Option<Picture> {
        let began = Instant::now();
        self.server = if self.wants.immich { find_server(&self.app_dir) } else { None };
        let opening = !self.opened;
        self.opened = true;
        let immich = self.server.is_some();
        let (pic, how) = if let Some(p) = (opening && immich && self.keeps_ready).then(|| self.take_ready()).flatten() {
            (p, "kept from last time")
        } else if let Some((p, jpegs)) = immich.then(|| self.from_immich()).flatten() {
            if self.keeps_ready {
                self.keep_ready(&p, &jpegs);
            }
            (p, "from Immich")
        } else if let Some(p) = self.from_picks() {
            (p, "from the wallpaper pictures")
        } else {
            (self.from_home_wallpaper()?, "the home wallpaper")
        };
        self.last_id = pic.id.clone();
        note!("picker: {} {}, {} photo(s), {} ms", pic.id, how, pic.photos.len(), began.elapsed().as_millis());
        Some(pic)
    }

    pub fn next_wallpaper(&mut self) -> Option<(String, Vec<u8>)> {
        self.server = find_server(&self.app_dir);
        if self.server.is_none() {
            note!("picker: no Immich server");
            return None;
        }
        let sources = self.wants.sources.clone();
        let last = self.last_id.clone();
        let pic = self.next_asset(&sources, &last)?;
        let bytes = self.preview(&pic)?;
        self.last_id = pic.clone();
        Some((format!("immich:{pic}"), bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(id: &str, w: u32, h: u32, day: &str) -> Value {
        json!({"id": id, "width": w, "height": h, "localDateTime": format!("{day}T10:00:00.000Z")})
    }

    fn id(n: u8) -> String {
        format!("00000000-0000-0000-0000-0000000000{n:02x}")
    }

    #[test]
    fn portraits_are_paired_only_with_one_from_another_day_and_a_wide_photo_stands_alone() {
        let list = [asset(&id(1), 3000, 4000, "2026-01-01"), asset(&id(2), 3000, 4000, "2026-01-01"), asset(&id(3), 3000, 4000, "2026-02-02"), asset(&id(4), 4000, 3000, "2026-03-03"), asset(&id(5), 1000, 1000, "2026-03-03")];
        let mut days = Map::new();
        let mut arranged = arrange(&list, &mut days, false, &mut Chance(7));
        arranged.sort();
        assert_eq!(arranged, vec![format!("{}+{}", id(1), id(3)), id(4)]);
        assert_eq!(days.get(&id(2)), Some(&json!("2026-01-01")));
        assert_eq!(arrange(&list, &mut days, true, &mut Chance(7)), vec![id(4)]);
    }

    #[test]
    fn a_face_is_kept_in_view_and_a_photo_without_one_sits_a_little_high() {
        let (x, y) = place(Some([3.0, 0.9, 0.5]), SCREEN_BOX);
        assert!(x > 0.8, "{x}");
        assert_eq!(y, 0.5);
        assert_eq!(place(Some([0.75, NO_FACE, NO_FACE]), PAIR_BOX), (0.5, 0.3));
        assert_eq!(place(None, SCREEN_BOX), (0.5, 0.4));
    }

    #[test]
    fn a_photo_shown_lately_is_not_picked_and_one_from_the_last_photos_day_waits_its_turn() {
        let mut pool = Pool { ids: vec!["a".into(), "b".into(), "c+d".into()], shown: vec!["a".into()], last: vec!["2026-05-05".into()], ..Pool::default() };
        pool.days.insert("b".into(), json!("2026-05-05"));
        pool.days.insert("c".into(), json!("2026-06-06"));
        assert_eq!(take(&mut pool, true, &[]).as_deref(), Some("c+d"));
        assert_eq!(take(&mut pool, true, &[]), None);
        assert_eq!(take(&mut pool, false, &["b"]), None);
        let mut again = Pool { ids: vec!["b".into()], ..Pool::default() };
        assert_eq!(take(&mut again, false, &[]).as_deref(), Some("b"));
        finish(&mut pool, Some("c+d"));
        assert_eq!(pool.shown, ["c", "d", "a"]);
        assert_eq!(pool.last, ["2026-06-06"]);
    }

    #[test]
    fn an_immich_server_with_odd_characters_in_its_address_or_key_is_refused() {
        assert!(server_from(r#"{"url":"http://192.168.0.5:2283/","key":"abc-DEF_123"}"#).is_some_and(|s| s.url == "http://192.168.0.5:2283"));
        for bad in [r#"{"url":"file:///etc","key":"abc"}"#, r#"{"url":"http://a b","key":"abc"}"#, r#"{"url":"http://a","key":"a'b"}"#, r#"{"url":"http://a","key":""}"#, "nope"] {
            assert!(server_from(bad).is_none(), "{bad}");
        }
    }
}
