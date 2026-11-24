#![allow(dead_code)]

use crate::settings::Settings;
use crate::{apps, bus, note, root, store};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

pub const DEFAULT_ROW_CHANNELS: usize = 5;
pub const FIRST_CHANNELS: usize = 10;
pub const LIVE_INFO_W: f32 = 660.0;
pub const HALF_HOUR_S: u64 = 30 * 60;
pub const SCHEDULE_HOURS: u64 = 8;
pub const READ_STALE_S: u64 = 30 * 60;
pub const FAILED_WAIT_S: u64 = 5 * 60;
pub const CACHE_HOURS: u64 = 4;
pub const EMPTY_RETRY_S: u64 = 60;
pub const TICK_MS: u64 = 15_000;
pub const DRAW_AFTER_MS: u64 = 600;
pub const READ_AFTER_MS: u64 = 2000;
pub const WALLPAPER_WAIT_MS: u64 = 10_000;
const MAX_PF_PAGES: usize = 4;
const MAX_SCH_PAGES: usize = 120;
const PART_EVERY: usize = 3;
const ROWS_ABOVE_FOCUS: usize = 3;
const CACHE_FILE: &str = "livetv.json";
const LOGO_DIR: &str = "logos";

#[derive(Clone, PartialEq, Debug)]
pub struct Channel {
    pub id: String,
    pub signal: String,
    pub number: String,
    pub name: String,
    pub logo_path: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Programme {
    pub title: String,
    pub start: u64,
    pub end: u64,
    pub subs: bool,
    pub ad: bool,
    pub desc: String,
}

pub type Listings = BTreeMap<String, Vec<Programme>>;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct NowNext<'a> {
    pub now: Option<&'a Programme>,
    pub next: Option<&'a Programme>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct LiveCache {
    pub channels: Vec<Channel>,
    pub listings: Listings,
    pub schedule: Listings,
}

pub struct Steer {
    pub focus: Box<dyn Fn() -> usize + Send>,
    pub stopped: Box<dyn Fn() -> bool + Send>,
    pub on_part: Box<dyn FnMut(&Listings, &[Channel]) + Send>,
}

type Done<T> = Box<dyn FnOnce(Result<T, String>) + Send>;

fn signal_ok(signal: &str) -> bool {
    let parts: Vec<&str> = signal.split('_').collect();
    parts.len() == 3 && parts.iter().all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

fn leading_number(text: &str) -> Option<f64> {
    let text = text.trim_start();
    let end = text.char_indices().find(|&(i, c)| !(c.is_ascii_digit() || (c == '.' && !text[..i].contains('.')))).map_or(text.len(), |(i, _)| i);
    text[..end].parse().ok()
}

pub fn parse_channels(reply: &Value) -> Vec<Channel> {
    let raw = match reply["channelList"].as_array() {
        Some(list) => list.as_slice(),
        None => reply.as_object().and_then(|o| o.values().find_map(Value::as_array)).map_or(&[][..], Vec::as_slice),
    };
    let mut out: Vec<Channel> = raw
        .iter()
        .filter(|c| !(c["Radio"].as_bool() == Some(true) || c["Data"].as_bool() == Some(true) || c["skipped"].as_bool() == Some(true)))
        .filter_map(|c| {
            let id = c["channelId"].as_str().filter(|id| root::channel_id_ok(id))?;
            let signal = c["signalChannelId"].as_str().filter(|s| signal_ok(s)).map_or_else(|| id.split('_').skip(4).collect::<Vec<_>>().join("_"), String::from);
            let number = match &c["channelNumber"] {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => String::new(),
            };
            let name = c["channelName"].as_str().filter(|n| !n.is_empty()).map_or_else(|| number.clone(), String::from);
            let logo_path = c["imgUrl"].as_str().filter(|p| !p.is_empty()).map(String::from);
            Some(Channel { id: id.to_string(), signal, number, name, logo_path })
        })
        .collect();
    let key = |c: &Channel| leading_number(&c.number).unwrap_or(f64::INFINITY);
    out.sort_by(|a, b| key(a).total_cmp(&key(b)).then_with(|| a.number.cmp(&b.number)));
    out
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    era * 146_097 + year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year - 719_468
}

pub fn utc_time(text: &str) -> Option<u64> {
    let p: Vec<i64> = text.split(',').map(|part| part.trim().parse().ok()).collect::<Option<_>>()?;
    if p.len() < 6 {
        return None;
    }
    let seconds = days_from_civil(p[0], p[1], p[2]) * 86400 + p[3] * 3600 + p[4] * 60 + p[5];
    u64::try_from(seconds).ok()
}

pub fn local_stamp(unix: u64, offset_s: i64) -> String {
    crate::util::utc_stamp((unix as i64 + offset_s).max(0) as u64)[..19].replace(['-', 'T', ':'], ",")
}

pub fn local_offset(unix: u64) -> i64 {
    let at = crate::clock::local(unix);
    let diff = (at.hour * 3600 + at.minute * 60 + at.second) as i64 - (unix % 86400) as i64;
    match diff {
        d if d > 14 * 3600 => d - 86400,
        d if d < -12 * 3600 => d + 86400,
        d => d,
    }
}

fn blurb(text: &str) -> String {
    let mut rest = text.trim_end();
    while let Some(inner_end) = rest.strip_suffix(']') {
        let Some(open) = inner_end.rfind('[') else { break };
        let inner = &inner_end[open + 1..];
        if inner.is_empty() || inner.chars().count() > 12 || !inner.chars().all(|c| c.is_ascii_uppercase() || c == ',' || c.is_whitespace()) {
            break;
        }
        rest = inner_end[..open].trim_end();
    }
    rest.trim().to_string()
}

pub fn group_listings<'a>(rows: impl IntoIterator<Item = &'a Value>) -> Listings {
    let mut out = Listings::new();
    for r in rows {
        let (Some(signal), Some(start), Some(end)) = (r["signalChannelId"].as_str().filter(|s| !s.is_empty()), r["startTime"].as_str().and_then(utc_time), r["endTime"].as_str().and_then(utc_time)) else { continue };
        if end <= start {
            continue;
        }
        out.entry(signal.to_string()).or_default().push(Programme {
            title: r["programName"].as_str().unwrap_or("").to_string(),
            start,
            end,
            subs: r["isSubtitle"].as_bool() == Some(true),
            ad: r["isAD"].as_bool() == Some(true),
            desc: blurb(r["description"].as_str().unwrap_or("")),
        });
    }
    for list in out.values_mut() {
        list.sort_by_key(|p| p.start);
    }
    out
}

fn overlaps(r: &Value, from: u64, to: u64) -> bool {
    let (start, end) = (r["startTime"].as_str().and_then(utc_time), r["endTime"].as_str().and_then(utc_time));
    matches!((start, end), (Some(s), Some(e)) if e > from && s < to)
}

pub fn parse_events(reply: &Value, from: u64, to: u64) -> Listings {
    let rows: Vec<Value> = reply["programList"]
        .as_array()
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .filter(|r| overlaps(r, from, to))
        .map(|r| {
            let mut r = r.clone();
            if r.get("isSubtitle").is_none() && r["description"].as_str().is_some_and(|d| d.contains("[S]")) {
                r["isSubtitle"] = json!(true);
            }
            r
        })
        .collect();
    group_listings(&rows)
}

const CHANNEL_COLORS: &[(&str, &str)] = &[
    ("bbc one", "#b5121b"), ("bbc two", "#0e7c7b"), ("bbc three", "#d1006a"), ("bbc four", "#2b2e83"), ("bbc news", "#a30000"),
    ("bbc parliament", "#1b5e3b"), ("bbc scotland", "#005eb8"), ("bbc alba", "#1f6f8b"), ("cbbc", "#4f9d2d"), ("cbeebies", "#d97a00"),
    ("itv1", "#0b8a8f"), ("itv", "#0b8a8f"), ("stv", "#0b8a8f"), ("itv2", "#c2185b"), ("itv3", "#6d2077"), ("itv4", "#1d3c78"), ("itvbe", "#9e2a8d"),
    ("itv quiz", "#e05a00"), ("channel 4", "#22348c"), ("e4", "#5b2a86"), ("e4 extra", "#5b2a86"), ("more4", "#1f6f63"), ("film4", "#a3162c"),
    ("4seven", "#3a3f9e"), ("5", "#6b2c91"), ("channel 5", "#6b2c91"), ("5usa", "#0b3d91"), ("5star", "#d1007a"), ("5action", "#b3261e"),
    ("sky arts", "#5b1f6e"), ("sky news", "#b0001e"), ("quest", "#d35400"), ("quest red", "#a5122a"), ("dave", "#2f3640"), ("yesterday", "#7a4e2d"),
    ("drama", "#8e1f3a"), ("really", "#c2185b"), ("pick", "#2e7d32"), ("challenge", "#1565c0"), ("gb news", "#0a2a66"), ("al jazeera eng", "#b8860b"),
    ("s4c", "#c8102e"), ("great! movies", "#c62828"), ("legend", "#4e342e"),
];

fn colour_key(name: &str) -> String {
    let mut text = name.to_lowercase();
    while let (Some(open), Some(close)) = (text.find('('), text.find(')')) {
        if close < open {
            break;
        }
        text.replace_range(open..=close, " ");
    }
    let mut words: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        if word == "hd" {
            continue;
        }
        let without_plus: String = match word.find('+') {
            Some(at) if word[at + 1..].chars().all(|c| c.is_ascii_digit()) => word[..at].to_string(),
            _ => word.to_string(),
        };
        if !without_plus.is_empty() {
            words.push(without_plus);
        }
    }
    words.join(" ")
}

pub fn channel_color(name: &str) -> Option<[f32; 3]> {
    let key = colour_key(name);
    let known = CHANNEL_COLORS.iter();
    let exact = known.clone().find(|(k, _)| *k == key);
    let found = exact.or_else(|| known.filter(|(k, _)| key.starts_with(&format!("{k} "))).max_by_key(|(k, _)| k.len()));
    found.and_then(|(_, hex)| apps::hex_color(hex))
}

pub fn row_channels(channels: &[Channel], ids: Option<&[String]>) -> Vec<Channel> {
    match ids {
        None => channels.iter().take(DEFAULT_ROW_CHANNELS).cloned().collect(),
        Some(ids) => ids.iter().filter_map(|id| channels.iter().find(|c| &c.id == id).cloned()).collect(),
    }
}

pub fn row(channels: &[Channel], settings: &Settings) -> Vec<Channel> {
    if !settings.live_tv_row {
        return Vec::new();
    }
    row_channels(channels, settings.live_channels.as_deref())
}

pub fn live_slot(first_row: usize, tile_size: f32, gap: f32) -> usize {
    ((first_row as f32 * (tile_size + gap) - gap - LIVE_INFO_W) / (tile_size + gap)).floor().max(0.0) as usize
}

pub fn find_channel(channels: &[Channel], typed: &str) -> Option<usize> {
    if let Some(exact) = channels.iter().position(|c| c.number == typed) {
        return Some(exact);
    }
    let n: f64 = typed.trim().parse().ok()?;
    channels.iter().position(|c| leading_number(&c.number).is_some_and(|m| m >= n))
}

pub fn merge_listings(sch: &Listings, pf: &Listings) -> Listings {
    let mut out = pf.clone();
    for (k, scheduled) in sch {
        let mut keep: Vec<Programme> = pf
            .get(k)
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .map(|q| match scheduled.iter().find(|p| q.desc.is_empty() && p.start == q.start && !p.desc.is_empty()) {
                Some(s) => Programme { desc: s.desc.clone(), ..q.clone() },
                None => q.clone(),
            })
            .collect();
        for p in scheduled {
            if !keep.iter().any(|q| p.start < q.end && q.start < p.end) {
                keep.push(p.clone());
            }
        }
        keep.sort_by_key(|p| p.start);
        out.insert(k.clone(), keep);
    }
    out
}

pub fn now_next_at(list: Option<&[Programme]>, t: u64) -> NowNext<'_> {
    let mut found = NowNext::default();
    for p in list.unwrap_or(&[]) {
        if p.start <= t && t < p.end {
            found.now = Some(p);
        } else if p.start > t {
            found.next = Some(p);
            return found;
        }
    }
    found
}

pub fn progress(p: &Programme, t: u64) -> f32 {
    ((t as f64 - p.start as f64) / (p.end as f64 - p.start as f64)).clamp(0.0, 1.0) as f32
}

pub fn time_left(p: &Programme, t: u64) -> String {
    let min = ((p.end as i64 - t as i64) as f64 / 60.0).ceil().max(1.0) as i64;
    match (min / 60, min % 60) {
        (0, m) => format!("{m} min left"),
        (h, 0) => format!("{h} hr left"),
        (h, m) => format!("{h} hr {m} min left"),
    }
}

pub fn on_now(listings: &Listings, t: u64) -> usize {
    listings.values().filter(|list| list.iter().any(|p| p.start <= t && t < p.end)).count()
}

pub fn keep_unfinished(fresh: Listings, old: &Listings, t: u64) -> Listings {
    let mut out = fresh;
    for (k, list) in old {
        if out.get(k).is_some_and(|l| !l.is_empty()) {
            continue;
        }
        let left: Vec<Programme> = list.iter().filter(|p| p.end > t).cloned().collect();
        if !left.is_empty() {
            out.insert(k.clone(), left);
        }
    }
    out
}

pub fn ask_now_next_again(channels: &[Channel], row: &[Channel], listings: &Listings, read_at: u64, t: u64) -> bool {
    let ended = row.iter().any(|c| listings.get(&c.signal).is_some_and(|l| l.iter().any(|p| p.end > read_at && p.end <= t)));
    let empty = (channels.is_empty() || on_now(listings, t) == 0) && t.saturating_sub(read_at) >= EMPTY_RETRY_S;
    ended || empty
}

pub fn first_read(channels: &[Channel], settings: &Settings) -> Vec<Channel> {
    let mut out = row_channels(channels, settings.live_channels.as_deref());
    for c in channels.iter().take(FIRST_CHANNELS) {
        if !out.contains(c) {
            out.push(c.clone());
        }
    }
    out
}

pub fn fresh_reads(list: &[Channel], asked_at: &HashMap<String, u64>, t: u64) -> Vec<bool> {
    list.iter().map(|c| t.saturating_sub(asked_at.get(&c.signal).copied().unwrap_or(0)) < READ_STALE_S).collect()
}

pub fn guide_window(t: u64) -> (u64, u64) {
    let from = t - t % HALF_HOUR_S;
    (from, from + SCHEDULE_HOURS * 3600)
}

pub fn next_batch(count: usize, done: &[bool], focus: usize) -> Vec<usize> {
    if count == 0 {
        return Vec::new();
    }
    let start = focus.saturating_sub(ROWS_ABOVE_FOCUS).min(count - 1);
    let open = |i: &usize| !done.get(*i).copied().unwrap_or(false);
    (start..count).filter(open).chain((0..start).rev().filter(open)).take(root::EVENT_BATCH).collect()
}

pub fn for_cache(listings: &Listings, keep: &[Channel], t: u64) -> Listings {
    keep.iter()
        .filter_map(|c| {
            let left: Vec<Programme> = listings.get(&c.signal)?.iter().filter(|p| p.end > t && p.start < t + CACHE_HOURS * 3600).cloned().collect();
            (!left.is_empty()).then(|| (c.signal.clone(), left))
        })
        .collect()
}

fn channel_json(c: &Channel) -> Value {
    json!({"id": c.id, "signal": c.signal, "number": c.number, "name": c.name, "logo": c.logo_path})
}

fn channel_from(v: &Value) -> Option<Channel> {
    Some(Channel {
        id: v["id"].as_str()?.to_string(),
        signal: v["signal"].as_str()?.to_string(),
        number: v["number"].as_str().unwrap_or("").to_string(),
        name: v["name"].as_str().unwrap_or("").to_string(),
        logo_path: v["logo"].as_str().map(String::from),
    })
}

fn listings_json(l: &Listings) -> Value {
    l.iter().map(|(k, list)| (k.clone(), list.iter().map(|p| json!({"title": p.title, "start": p.start, "end": p.end, "subs": p.subs, "ad": p.ad})).collect::<Value>())).collect::<serde_json::Map<_, _>>().into()
}

fn listings_from(v: &Value) -> Listings {
    v.as_object()
        .map(|o| {
            o.iter()
                .map(|(k, list)| {
                    let programmes = list.as_array().map_or(&[][..], Vec::as_slice).iter().filter_map(|p| {
                        Some(Programme { title: p["title"].as_str()?.to_string(), start: p["start"].as_u64()?, end: p["end"].as_u64()?, subs: p["subs"].as_bool() == Some(true), ad: p["ad"].as_bool() == Some(true), desc: String::new() })
                    });
                    (k.clone(), programmes.collect())
                })
                .collect()
        })
        .unwrap_or_default()
}

fn cache_json(c: &LiveCache) -> Value {
    json!({"channels": c.channels.iter().map(channel_json).collect::<Vec<_>>(), "listings": listings_json(&c.listings), "schedule": listings_json(&c.schedule)})
}

fn cache_from(v: &Value) -> Option<LiveCache> {
    Some(LiveCache { channels: v["channels"].as_array()?.iter().filter_map(channel_from).collect(), listings: listings_from(&v["listings"]), schedule: listings_from(&v["schedule"]) })
}

pub fn kept() -> LiveCache {
    store::read(CACHE_FILE).and_then(|text| cache_from(&serde_json::from_str(&text).ok()?)).unwrap_or_default()
}

pub fn keep(cache: &LiveCache) -> bool {
    store::write(CACHE_FILE, cache_json(cache).to_string().as_bytes())
}

pub fn logo_name(path: &str) -> String {
    let mut h: u32 = 5381;
    for unit in path.encode_utf16() {
        h = (h << 5).wrapping_add(h).wrapping_add(unit as u32);
    }
    let mut digits = Vec::new();
    loop {
        digits.push(std::char::from_digit(h % 36, 36).unwrap_or('0'));
        h /= 36;
        if h == 0 {
            break;
        }
    }
    format!("l{}", digits.iter().rev().collect::<String>())
}

fn logo_file(path: &str) -> String {
    store::path(&format!("{LOGO_DIR}/{}.png", logo_name(path)))
}

pub fn kept_logos(channels: &[Channel]) -> HashMap<String, String> {
    channels.iter().filter_map(|c| c.logo_path.as_ref()).filter(|p| std::path::Path::new(&logo_file(p)).exists()).map(|p| (p.clone(), logo_file(p))).collect()
}

pub fn read_logos(channels: &[Channel], then: impl FnOnce(HashMap<String, String>) + Send + 'static) {
    let dir = store::path(LOGO_DIR);
    let _ = std::fs::create_dir_all(&dir);
    let mut paths: Vec<String> = channels.iter().filter_map(|c| c.logo_path.clone()).collect();
    paths.sort();
    paths.dedup();
    root::copy_logos(&dir.clone(), move |copied| {
        if let Err(why) = copied {
            note!("guide: logo files: {}", why);
        }
        let missing: Vec<(String, String)> = paths.iter().filter(|p| !std::path::Path::new(&logo_file(p)).exists()).map(|p| (p.clone(), format!("{}.png", logo_name(p)))).collect();
        if missing.is_empty() {
            return then(found_logos(&paths));
        }
        root::copy_icons(&missing, &dir, move |_| then(found_logos(&paths)));
    });
}

fn found_logos(paths: &[String]) -> HashMap<String, String> {
    paths.iter().filter(|p| std::path::Path::new(&logo_file(p)).exists()).map(|p| (p.clone(), logo_file(p))).collect()
}

fn answer(out: Result<String, String>) -> Result<Value, String> {
    out.and_then(|text| bus::answered(text.trim()))
}

pub fn read_channels(then: impl FnOnce(Result<Vec<Channel>, String>) + Send + 'static) {
    root::channel_list(move |out| then(answer(out).map(|reply| parse_channels(&reply))));
}

fn now_next_page(page: Option<String>, slim: bool, n: usize, mut rows: Vec<Value>, then: Done<Listings>) {
    root::now_next(page.as_deref(), slim, move |out| match answer(out) {
        Err(_) if slim && n == 0 => now_next_page(None, false, 0, rows, then),
        Err(why) => then(Err(format!("epgdb find refused: {why}"))),
        Ok(mut reply) => {
            if let Some(Value::Array(got)) = reply.get_mut("results").map(Value::take) {
                rows.extend(got);
            }
            match reply["next"].as_str() {
                Some(next) if n + 1 < MAX_PF_PAGES => now_next_page(Some(next.to_string()), slim, n + 1, rows, then),
                _ => then(Ok(group_listings(&rows))),
            }
        }
    });
}

pub fn read_now_next(then: impl FnOnce(Result<Listings, String>) + Send + 'static) {
    now_next_page(None, true, 0, Vec::new(), Box::new(then));
}

struct ScheduleRead {
    from: u64,
    to: u64,
    kept: Vec<Value>,
    seen: usize,
    began: Instant,
    stopped: Box<dyn Fn() -> bool + Send>,
    on_part: Box<dyn FnMut(&Listings) + Send>,
    then: Done<Listings>,
}

fn schedule_page(page: Option<String>, n: usize, mut read: Box<ScheduleRead>) {
    root::schedule(page.as_deref(), move |out| {
        let reply = match answer(out) {
            Ok(reply) => reply,
            Err(why) => {
                note!("guide: schedule refused: {}", why);
                return (read.then)(Err(format!("schedule refused: {why}")));
            }
        };
        let got = reply["results"].as_array().map_or(&[][..], Vec::as_slice);
        read.seen += got.len();
        let (from, to) = (read.from, read.to);
        read.kept.extend(got.iter().filter(|r| overlaps(r, from, to)).cloned());
        let next = reply["next"].as_str().map(String::from);
        if next.is_some() && n + 1 < MAX_SCH_PAGES && !(read.stopped)() {
            if (n + 1) % PART_EVERY == 0 {
                (read.on_part)(&group_listings(&read.kept));
            }
            return schedule_page(next, n + 1, read);
        }
        note!("guide: schedule: {} pages, {} rows, {} kept, {} ms{}", n + 1, read.seen, read.kept.len(), read.began.elapsed().as_millis(), if next.is_some() { ", stopped early" } else { "" });
        let ScheduleRead { kept, then, .. } = *read;
        then(Ok(group_listings(&kept)));
    });
}

pub fn read_schedule(
    window: (u64, u64),
    stopped: impl Fn() -> bool + Send + 'static,
    on_part: impl FnMut(&Listings) + Send + 'static,
    then: impl FnOnce(Result<Listings, String>) + Send + 'static,
) {
    let read = ScheduleRead { from: window.0, to: window.1, kept: Vec::new(), seen: 0, began: Instant::now(), stopped: Box::new(stopped), on_part: Box::new(on_part), then: Box::new(then) };
    schedule_page(None, 0, Box::new(read));
}

pub fn read_events(channels: &[Channel], window: (u64, u64), then: impl FnOnce(Result<Listings, String>) + Send + 'static) {
    let (from, to) = window;
    let ids: Vec<String> = channels.iter().map(|c| c.id.clone()).collect();
    root::event_list(&ids, &local_stamp(from, local_offset(from)), &local_stamp(to, local_offset(to)), move |out| {
        then(answer(out).map_err(|why| format!("events refused: {why}")).map(|reply| parse_events(&reply, from, to)))
    });
}

struct NearRead {
    channels: Vec<Channel>,
    window: (u64, u64),
    done: Vec<bool>,
    out: Listings,
    calls: usize,
    programmes: usize,
    began: Instant,
    steer: Steer,
    then: Done<Listings>,
}

fn near_step(mut read: Box<NearRead>) {
    let stopped = (read.steer.stopped)();
    let batch = if stopped { Vec::new() } else { next_batch(read.channels.len(), &read.done, (read.steer.focus)()) };
    if batch.is_empty() {
        note!("guide: events: {} calls, {} programmes, {} ms{}", read.calls, read.programmes, read.began.elapsed().as_millis(), if stopped { ", stopped early" } else { "" });
        let NearRead { out, then, .. } = *read;
        return then(Ok(out));
    }
    for &i in &batch {
        read.done[i] = true;
    }
    let first = read.calls == 0;
    read.calls += 1;
    let asked: Vec<Channel> = batch.iter().map(|&i| read.channels[i].clone()).collect();
    let window = read.window;
    read_events(&asked.clone(), window, move |got| match got {
        Ok(got) => {
            for (k, list) in got {
                read.programmes += list.len();
                read.out.insert(k, list);
            }
            if first && read.programmes == 0 {
                note!("guide: events: the first call found nothing");
                return (read.then)(Err("events: the first call found nothing".into()));
            }
            (read.steer.on_part)(&read.out, &asked);
            near_step(read);
        }
        Err(why) => {
            note!("guide: {}", why);
            if first {
                return (read.then)(Err(why));
            }
            near_step(read);
        }
    });
}

pub fn read_events_near(channels: Vec<Channel>, window: (u64, u64), skip: Vec<bool>, steer: Steer, then: impl FnOnce(Result<Listings, String>) + Send + 'static) {
    let done = (0..channels.len()).map(|i| skip.get(i).copied().unwrap_or(false)).collect();
    near_step(Box::new(NearRead { channels, window, done, out: Listings::new(), calls: 0, programmes: 0, began: Instant::now(), steer, then: Box::new(then) }));
}

pub fn parse_current_channel(reply: &Value) -> Option<String> {
    reply["channelId"].as_str().filter(|id| root::channel_id_ok(id)).map(String::from)
}

pub fn read_current_channel(then: impl FnOnce(Result<Option<String>, String>) + Send + 'static) {
    root::current_channel(move |out| then(answer(out).map(|reply| parse_current_channel(&reply))));
}

pub fn open_channel(channel_id: &str, then: impl FnOnce(Result<(), String>) + Send + 'static) {
    root::open_channel(channel_id, move |out| match out {
        Ok(text) if text.contains("\"returnValue\":false") => then(Err("the TV refused the channel".into())),
        Ok(_) => then(Ok(())),
        Err(why) => then(Err(why)),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHANNELS: &str = r#"{"returnValue":true,"channelListCount":188,"channelList":[
        {"channelId":"1_44_10_10_8217_8294_9018","signalChannelId":"8217_8294_9018","channelNumber":"10","channelName":"ITV3","imgUrl":"/tmp/gibbips/logo/s4b9c5363866d7a55988564c4bd4cb424","Radio":false,"Data":false,"skipped":false},
        {"channelId":"1_44_3_3_8217_8281_9018","signalChannelId":"","channelNumber":"3","channelName":"","imgUrl":"","Radio":false,"Data":false,"skipped":false},
        {"channelId":"1_41_1_101_16522_17546_9018","signalChannelId":"16522_17546_9018","channelNumber":"1","channelName":"BBC ONE Yks HD","imgUrl":"/tmp/gibbips/logo/s210a81942c909451daf38c567c71af39","Radio":false,"Data":false,"skipped":false},
        {"channelId":"1_27_16529_711_4169_6153_9018","signalChannelId":"4169_6153_9018","channelNumber":"145","channelName":"BBC Newcastle","imgUrl":"","Radio":true,"Data":false,"skipped":false},
        {"channelId":"1_41_2_102_16522_17472_9018","signalChannelId":"16522_17472_9018","channelNumber":"2","channelName":"BBC TWO HD","imgUrl":"/tmp/gibbips/logo/sd3c3ccc2a1e5976563f95d6c74890b15","Radio":false,"Data":false,"skipped":true},
        {"channelId":"1_41_9_106","channelNumber":"9","channelName":"Broken"}]}"#;

    const NOW_NEXT: &str = r#"{"returnValue":true,"results":[
        {"endTime":"2026,11,24,15,25,56","isPresent":false,"programName":"Don't Quit Your Gay Job","signalChannelId":"32815_34048_9018","startTime":"2026,11,24,15,00,50"},
        {"endTime":"2026,11,24,15,00,50","isPresent":true,"programName":"Hey Qween","signalChannelId":"32815_34048_9018","startTime":"2026,11,24,14,09,28"},
        {"endTime":"2026,11,24,15,00,00","isPresent":true,"programName":"Newshour","signalChannelId":"4170_6016_9018","startTime":"2026,11,24,15,00,00"},
        {"endTime":"2026,11,24,15,06,00","isPresent":false,"programName":"BBC News","signalChannelId":"","startTime":"2026,11,24,15,00,00"}],
        "next":"1mo403xMB5CPD5OWTZO1+F"}"#;

    const EVENTS: &str = r#"{"returnValue":true,"programList":[
        {"description":"41/50. Sonali Shah helps Norman and Lynda swap London for a breath of fresh sea air in Dorset. [S] [HD]","isAD":false,"signalChannelId":"16522_17546_9018","programName":"Escape to the Country","DSTEndTime":"2026,11,24,15,45,00","isPresent":false,"endTime":"2026,11,24,15,45,00","DSTStartTime":"2026,11,24,15,00,00","startTime":"2026,11,24,15,00,00","isSubtitle":true,"localStartTime":"2026,11,24,15,00,00"},
        {"description":"The team head to Wiltshire to rescue an RAF memorial bench. [S,AD] [HD]","isAD":true,"signalChannelId":"16522_17546_9018","programName":"The Repair Shop on the Road","endTime":"2026,11,24,16,30,00","startTime":"2026,11,24,15,45,00"},
        {"description":"Roman Kemp hosts the quiz show. [S]","signalChannelId":"16522_17472_9018","programName":"Sneaky Pre-Quiz","endTime":"2026,11,24,15,00,00","startTime":"2026,11,24,14,15,00"},
        {"description":"Roman Kemp hosts the quiz show where knowledge equals speed. [S]","signalChannelId":"16522_17472_9018","programName":"Quiz","endTime":"2026,11,24,15,45,00","startTime":"2026,11,24,15,00,00"}]}"#;

    const AT_15_00: u64 = 1_795_532_400;

    fn programme(title: &str, start_min: u64, end_min: u64) -> Programme {
        Programme { title: title.into(), start: AT_15_00 + start_min * 60, end: AT_15_00 + end_min * 60, subs: false, ad: false, desc: String::new() }
    }

    fn channels() -> Vec<Channel> {
        parse_channels(&serde_json::from_str(CHANNELS).unwrap())
    }

    #[test]
    fn the_channel_list_keeps_tv_channels_in_number_order_without_radio_skipped_or_broken_ones() {
        let list = channels();
        assert_eq!(list.iter().map(|c| c.number.as_str()).collect::<Vec<_>>(), ["1", "3", "10"]);
        assert_eq!(list[0], Channel { id: "1_41_1_101_16522_17546_9018".into(), signal: "16522_17546_9018".into(), number: "1".into(), name: "BBC ONE Yks HD".into(), logo_path: Some("/tmp/gibbips/logo/s210a81942c909451daf38c567c71af39".into()) });
        assert_eq!(list[1], Channel { id: "1_44_3_3_8217_8281_9018".into(), signal: "8217_8281_9018".into(), number: "3".into(), name: "3".into(), logo_path: None });
    }

    #[test]
    fn the_tvs_times_are_utc() {
        assert_eq!(utc_time("2026,11,24,15,00,00"), Some(AT_15_00));
        assert_eq!(utc_time("2000,02,29,00,00,00"), Some(951_782_400));
        assert_eq!(utc_time("2026,11,24"), None);
    }

    #[test]
    fn now_and_next_rows_are_grouped_by_channel_earliest_first_and_rows_without_a_channel_or_length_are_dropped() {
        let reply: Value = serde_json::from_str(NOW_NEXT).unwrap();
        let l = group_listings(reply["results"].as_array().unwrap());
        assert_eq!(l.keys().collect::<Vec<_>>(), ["32815_34048_9018"]);
        assert_eq!(l["32815_34048_9018"].iter().map(|p| p.title.as_str()).collect::<Vec<_>>(), ["Hey Qween", "Don't Quit Your Gay Job"]);
        assert_eq!(l["32815_34048_9018"][0].end, AT_15_00 + 50);
    }

    #[test]
    fn guide_events_keep_the_window_their_flags_and_a_blurb_without_freeviews_tags() {
        let l = parse_events(&serde_json::from_str(EVENTS).unwrap(), AT_15_00, AT_15_00 + 3600);
        let one = &l["16522_17546_9018"];
        assert_eq!(one[0].desc, "41/50. Sonali Shah helps Norman and Lynda swap London for a breath of fresh sea air in Dorset.");
        assert_eq!((one[0].subs, one[0].ad, one[1].subs, one[1].ad), (true, false, false, true));
        assert_eq!(l["16522_17472_9018"].iter().map(|p| (p.title.as_str(), p.subs)).collect::<Vec<_>>(), [("Quiz", true)]);
    }

    #[test]
    fn what_is_on_comes_from_the_times_and_a_gap_shows_only_what_is_next() {
        let list = [programme("News", 0, 30), programme("Film", 60, 150)];
        let at = |min: u64| now_next_at(Some(&list), AT_15_00 + min * 60);
        assert_eq!((at(10).now.map(|p| p.title.as_str()), at(10).next.map(|p| p.title.as_str())), (Some("News"), Some("Film")));
        assert_eq!((at(30).now, at(30).next.map(|p| p.title.as_str())), (None, Some("Film")));
        assert_eq!((at(149).now.map(|p| p.title.as_str()), at(149).next), (Some("Film"), None));
        assert_eq!(now_next_at(None, AT_15_00), NowNext::default());
    }

    #[test]
    fn the_row_shows_the_first_five_until_some_are_picked_then_the_picked_ones_the_tv_still_has() {
        let many: Vec<Channel> = (1..=7).map(|n| Channel { id: format!("1_41_{n}_1_1_1_1"), signal: format!("{n}_1_1"), number: n.to_string(), name: n.to_string(), logo_path: None }).collect();
        assert_eq!(row_channels(&many, None).iter().map(|c| c.number.as_str()).collect::<Vec<_>>(), ["1", "2", "3", "4", "5"]);
        let picked = ["1_41_6_1_1_1_1".to_string(), "gone".to_string(), "1_41_2_1_1_1_1".to_string()];
        assert_eq!(row_channels(&many, Some(&picked)).iter().map(|c| c.number.as_str()).collect::<Vec<_>>(), ["6", "2"]);
        let off = Settings { live_tv_row: false, ..Settings::defaults() };
        assert!(row(&many, &off).is_empty());
    }

    #[test]
    fn the_channel_info_keeps_a_focused_channel_far_enough_left_for_its_now_and_next() {
        assert_eq!(live_slot(6, 240.0, 40.0), 3);
        assert_eq!(live_slot(1, 240.0, 40.0), 0);
    }

    #[test]
    fn a_typed_number_finds_that_channel_or_the_next_one_up() {
        let list = channels();
        assert_eq!(find_channel(&list, "3"), Some(1));
        assert_eq!(find_channel(&list, "4"), Some(2));
        assert_eq!(find_channel(&list, "11"), None);
    }

    #[test]
    fn the_schedule_fills_round_now_and_next_and_lends_it_its_blurb() {
        let pf: Listings = [("a".to_string(), vec![programme("News", 0, 30), programme("Film", 30, 120)])].into();
        let sch: Listings = [("a".to_string(), vec![Programme { desc: "The day's news.".into(), ..programme("News at Three", 0, 30) }, programme("Film again", 60, 90), programme("Late", 120, 150)])].into();
        let merged = merge_listings(&sch, &pf);
        assert_eq!(merged["a"].iter().map(|p| p.title.as_str()).collect::<Vec<_>>(), ["News", "Film", "Late"]);
        assert_eq!(merged["a"][0].desc, "The day's news.");
    }

    #[test]
    fn a_thin_read_keeps_the_last_reads_unfinished_programmes_for_the_channels_it_missed() {
        let old: Listings = [("a".to_string(), vec![programme("Over", 0, 10), programme("On", 10, 40)]), ("b".to_string(), vec![programme("Old", 0, 40)])].into();
        let fresh: Listings = [("b".to_string(), vec![programme("New", 0, 40)])].into();
        let kept = keep_unfinished(fresh, &old, AT_15_00 + 20 * 60);
        assert_eq!(kept["a"].iter().map(|p| p.title.as_str()).collect::<Vec<_>>(), ["On"]);
        assert_eq!(kept["b"][0].title, "New");
    }

    #[test]
    fn now_and_next_is_asked_again_when_a_row_programme_ends_or_nothing_is_on_for_a_minute() {
        let list = channels();
        let row = row_channels(&list, None);
        let listings: Listings = [("16522_17546_9018".to_string(), vec![programme("News", 0, 30), programme("Film", 30, 120)])].into();
        let read_at = AT_15_00 + 10 * 60;
        assert!(!ask_now_next_again(&list, &row, &listings, read_at, AT_15_00 + 29 * 60));
        assert!(ask_now_next_again(&list, &row, &listings, read_at, AT_15_00 + 30 * 60));
        let empty = Listings::new();
        assert!(!ask_now_next_again(&list, &row, &empty, read_at, read_at + 59));
        assert!(ask_now_next_again(&list, &row, &empty, read_at, read_at + 60));
    }

    #[test]
    fn the_full_guide_reads_from_just_above_the_focus_down_then_upward() {
        let done: Vec<bool> = (0..30).map(|i| i == 11).collect();
        assert_eq!(next_batch(30, &done, 13), [10, 12, 13, 14, 15, 16, 17, 18, 19, 20]);
        let near_end: Vec<bool> = (0..30).map(|i| i < 20).collect();
        assert_eq!(next_batch(30, &near_end, 29), [26, 27, 28, 29, 25, 24, 23, 22, 21, 20]);
    }

    #[test]
    fn regional_and_hd_channel_names_find_their_brand_colour() {
        assert_eq!(channel_color("BBC ONE Yks HD"), apps::hex_color("#b5121b"));
        assert_eq!(channel_color("ITV1 +1"), apps::hex_color("#0b8a8f"));
        assert_eq!(channel_color("Leeds TV"), None);
    }

    #[test]
    fn logo_file_names_are_the_ones_the_web_lumen_keeps_on_the_tv() {
        assert_eq!(logo_name("/tmp/gibbips/logo/s210a81942c909451daf38c567c71af39"), "lma63u");
        assert_eq!(logo_name("/tmp/gibbips/logo/sd3c3ccc2a1e5976563f95d6c74890b15"), "l1jnpgnn");
        assert_eq!(logo_name("/tmp/gibbips/logo/sc3ba25353c0ef5ab7fde0cc03fae1904"), "la0ng0w");
    }

    #[test]
    fn the_guide_service_is_asked_in_the_tvs_local_time() {
        assert_eq!(local_stamp(AT_15_00, 0), "2026,11,24,15,00,00");
        assert_eq!(local_stamp(AT_15_00 + 9 * 3600, 3600), "2026,11,25,01,00,00");
    }

    #[test]
    fn the_kept_answer_comes_back_without_its_blurbs() {
        let cache = LiveCache {
            channels: channels(),
            listings: [("a".to_string(), vec![Programme { subs: true, desc: "Long blurb.".into(), ..programme("News", 0, 30) }])].into(),
            schedule: [("b".to_string(), vec![Programme { ad: true, ..programme("Film", 30, 120) }])].into(),
        };
        let back = cache_from(&serde_json::from_str(&cache_json(&cache).to_string()).unwrap()).unwrap();
        assert_eq!(back.listings["a"][0], Programme { subs: true, ..programme("News", 0, 30) });
        assert_eq!(LiveCache { listings: cache.listings.clone(), ..back.clone() }, cache);
    }

    #[test]
    fn the_last_channel_is_taken_only_when_it_has_the_tvs_id_shape() {
        assert_eq!(parse_current_channel(&json!({"returnValue": true, "channelId": "1_41_202_204_16522_18176_9018"})), Some("1_41_202_204_16522_18176_9018".into()));
        assert_eq!(parse_current_channel(&json!({"returnValue": true, "channelId": ""})), None);
    }
}
