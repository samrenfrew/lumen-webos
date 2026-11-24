use crate::store;
use crate::util::http_get;
use serde_json::{json, Value};

pub const AUTO_PLACE: &str = "@auto";
pub const MAX_AGE_S: u64 = 30 * 60;
const KEPT_HOURS: usize = 26;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Clear,
    Partly,
    Cloudy,
    Fog,
    Rain,
    Snow,
    Storm,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Hour {
    pub t: String,
    pub temp: i32,
    pub kind: Kind,
    pub day: bool,
    pub rain: i32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Weather {
    pub temp: i32,
    pub label: String,
    pub kind: Kind,
    pub day: bool,
    pub high: Option<i32>,
    pub low: Option<i32>,
    pub hours: Vec<Hour>,
    pub offset: i64,
    pub taken: u64,
}

const SUN: &str = "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.3 5.3l1.4 1.4M17.3 17.3l1.4 1.4M5.3 18.7l1.4-1.4M17.3 6.7l1.4-1.4";
const MOON: &str = "M19.5 14.5A7.5 7.5 0 0 1 9.5 4.5a7.5 7.5 0 1 0 10 10z";
const CLOUD: &str = "M7 18.5h10.5a3.5 3.5 0 0 0 .4-7 5.5 5.5 0 0 0-10.6-1.2A4.2 4.2 0 0 0 7 18.5z";
const PARTLY: &str = "M9 3v1.4M3.6 8.8H5M4.9 4.7l1 1M13.1 4.7l-1 1M6.4 10.4a3 3 0 0 1 5.3-3.2M9 19h8.5a3 3 0 0 0 .3-6 4.6 4.6 0 0 0-8.9-1A3.5 3.5 0 0 0 9 19z";
const PARTLY_NIGHT: &str = "M11.2 4.6a3.8 3.8 0 0 0 4.4 5.2 3.8 3.8 0 0 1-5.3 1.9M9 19h8.5a3 3 0 0 0 .3-6 4.6 4.6 0 0 0-8.9-1A3.5 3.5 0 0 0 9 19z";
const FOG: &str = "M4 8.5h16M3 12.5h18M5 16.5h14M8 20.5h8";
const RAIN: &str = "M7 14.5h10.5a3.5 3.5 0 0 0 .4-7 5.5 5.5 0 0 0-10.6-1.2A4.2 4.2 0 0 0 7 14.5zM8.5 17.5l-1 2.5M12.5 17.5l-1 2.5M16.5 17.5l-1 2.5";
const SNOW: &str = "M7 14.5h10.5a3.5 3.5 0 0 0 .4-7 5.5 5.5 0 0 0-10.6-1.2A4.2 4.2 0 0 0 7 14.5zM8 18.5h.01M12 18.5h.01M16 18.5h.01M10 21h.01M14 21h.01";
const STORM: &str = "M7 14.5h10.5a3.5 3.5 0 0 0 .4-7 5.5 5.5 0 0 0-10.6-1.2A4.2 4.2 0 0 0 7 14.5zM12.5 15.5l-2 3.5h3l-2 3.5";

pub fn icon(kind: Kind, day: bool) -> &'static str {
    match kind {
        Kind::Clear if day => SUN,
        Kind::Clear => MOON,
        Kind::Partly if day => PARTLY,
        Kind::Partly => PARTLY_NIGHT,
        Kind::Cloudy => CLOUD,
        Kind::Fog => FOG,
        Kind::Rain => RAIN,
        Kind::Snow => SNOW,
        Kind::Storm => STORM,
    }
}

fn kind_of(code: i64) -> Kind {
    match code {
        0 => Kind::Clear,
        1 | 2 => Kind::Partly,
        45 | 48 => Kind::Fog,
        71..=77 | 85 | 86 => Kind::Snow,
        95.. => Kind::Storm,
        51..=67 | 80..=82 => Kind::Rain,
        _ => Kind::Cloudy,
    }
}

fn describe(code: i64) -> &'static str {
    match code {
        0 => "Clear",
        1 | 2 => "Partly cloudy",
        3 => "Cloudy",
        45 | 48 => "Fog",
        51..=67 => "Rain",
        71..=77 => "Snow",
        80..=82 => "Showers",
        95.. => "Storm",
        _ => "",
    }
}

fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Clear => "clear",
        Kind::Partly => "partly",
        Kind::Cloudy => "cloudy",
        Kind::Fog => "fog",
        Kind::Rain => "rain",
        Kind::Snow => "snow",
        Kind::Storm => "storm",
    }
}

fn kind_named(name: &str) -> Kind {
    [Kind::Clear, Kind::Partly, Kind::Cloudy, Kind::Fog, Kind::Rain, Kind::Snow, Kind::Storm].into_iter().find(|&k| kind_name(k) == name).unwrap_or(Kind::Cloudy)
}

fn rounded(v: f64) -> i32 {
    (v + 0.5).floor() as i32
}

pub fn parse_forecast(d: &Value, taken: u64) -> Option<Weather> {
    let c = &d["current"];
    let temp = c["temperature_2m"].as_f64()?;
    let code = c["weather_code"].as_f64().unwrap_or(0.0) as i64;
    let from = c["time"].as_str().unwrap_or("");
    let from = &from[..from.len().min(13)];
    let h = &d["hourly"];
    let mut hours = Vec::new();
    if let (Some(times), Some(temps)) = (h["time"].as_array(), h["temperature_2m"].as_array()) {
        for (i, t) in times.iter().enumerate() {
            let (Some(t), Some(temp)) = (t.as_str(), temps.get(i).and_then(Value::as_f64)) else { continue };
            if t.get(..13).unwrap_or(t) < from || hours.len() >= KEPT_HOURS {
                continue;
            }
            hours.push(Hour {
                t: t.to_string(),
                temp: rounded(temp),
                kind: kind_of(h["weather_code"][i].as_f64().unwrap_or(0.0) as i64),
                day: h["is_day"][i].as_f64() != Some(0.0),
                rain: h["precipitation_probability"][i].as_f64().unwrap_or(0.0) as i32,
            });
        }
    }
    let daily = &d["daily"];
    Some(Weather {
        temp: rounded(temp),
        label: describe(code).to_string(),
        kind: kind_of(code),
        day: c["is_day"].as_f64() != Some(0.0),
        high: daily["temperature_2m_max"][0].as_f64().map(rounded),
        low: daily["temperature_2m_min"][0].as_f64().map(rounded),
        hours,
        offset: d["utc_offset_seconds"].as_i64().unwrap_or(0),
        taken,
    })
}

fn to_json(w: &Weather) -> Value {
    json!({
        "temp": w.temp, "label": w.label, "kind": kind_name(w.kind), "day": w.day, "high": w.high, "low": w.low, "offset": w.offset, "taken": w.taken,
        "hours": w.hours.iter().map(|h| json!({"t": h.t, "temp": h.temp, "kind": kind_name(h.kind), "day": h.day, "rain": h.rain})).collect::<Vec<_>>(),
    })
}

fn from_json(v: &Value) -> Option<Weather> {
    Some(Weather {
        temp: v["temp"].as_i64()? as i32,
        label: v["label"].as_str().unwrap_or("").to_string(),
        kind: kind_named(v["kind"].as_str().unwrap_or("")),
        day: v["day"].as_bool().unwrap_or(true),
        high: v["high"].as_i64().map(|x| x as i32),
        low: v["low"].as_i64().map(|x| x as i32),
        offset: v["offset"].as_i64().unwrap_or(0),
        taken: v["taken"].as_u64().unwrap_or(0),
        hours: v["hours"].as_array().map_or_else(Vec::new, |hours| {
            hours
                .iter()
                .filter_map(|h| Some(Hour { t: h["t"].as_str()?.to_string(), temp: h["temp"].as_i64()? as i32, kind: kind_named(h["kind"].as_str().unwrap_or("")), day: h["day"].as_bool().unwrap_or(true), rain: h["rain"].as_i64().unwrap_or(0) as i32 }))
                .collect()
        }),
    })
}

pub fn hours_ahead(w: &Weather, unix_now: u64, count: usize) -> Vec<Hour> {
    let there = crate::util::utc_stamp((unix_now as i64 + w.offset).max(0) as u64);
    let now = &there[..13];
    w.hours.iter().filter(|h| h.t.get(..13).unwrap_or(&h.t) > now).take(count).cloned().collect()
}

pub fn place(cities_tsv: &str, city: &str) -> Option<(f64, f64)> {
    cities_tsv.lines().find_map(|line| {
        let mut fields = line.split('\t');
        (fields.next()? == city).then_some(())?;
        Some((fields.next()?.parse().ok()?, fields.next()?.parse().ok()?))
    })
}

fn kept_name(city: &str) -> String {
    let safe: String = city.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    format!("weather-{safe}.json")
}

pub fn kept(city: &str) -> Option<Weather> {
    from_json(&serde_json::from_str(&store::read(&kept_name(city))?).ok()?)
}

pub fn fetch(city: String, cities_tsv: &str, unix_now: u64, then: impl FnOnce(Result<Weather, String>) + Send + 'static) {
    let known = if city == AUTO_PLACE { None } else { place(cities_tsv, &city) };
    if city != AUTO_PLACE && known.is_none() {
        return then(Err(format!("{city} is not a known city")));
    }
    std::thread::spawn(move || {
        let found = match known {
            Some(place) => Ok(place),
            None => http_get("https://ipwho.is/").and_then(|bytes| {
                let p: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                match (p["success"].as_bool(), p["latitude"].as_f64(), p["longitude"].as_f64()) {
                    (Some(true), Some(lat), Some(lon)) => Ok((lat, lon)),
                    _ => Err("no location".to_string()),
                }
            }),
        };
        let (lat, lon) = match found {
            Ok(place) => place,
            Err(why) => return then(Err(why)),
        };
        let url = format!("https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current=temperature_2m,weather_code,is_day&hourly=temperature_2m,weather_code,is_day,precipitation_probability&daily=temperature_2m_max,temperature_2m_min&forecast_days=2&timezone=auto");
        let read = http_get(&url).and_then(|bytes| serde_json::from_slice::<Value>(&bytes).map_err(|e| e.to_string())).and_then(|d| parse_forecast(&d, unix_now).ok_or_else(|| "no weather in the answer".to_string()));
        if let Ok(w) = &read {
            store::write(&kept_name(&city), to_json(w).to_string().as_bytes());
        }
        then(read);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const ANSWER: &str = r#"{"utc_offset_seconds":0,"current":{"time":"2026-11-24T14:30","temperature_2m":4.4,"weather_code":3,"is_day":1},
        "hourly":{"time":["2026-11-24T13:00","2026-11-24T14:00","2026-11-24T15:00","2026-11-24T16:00"],"temperature_2m":[4.0,4.4,3.6,null],
        "weather_code":[3,3,61,0],"is_day":[1,1,1,0],"precipitation_probability":[5,10,70,0]},
        "daily":{"temperature_2m_max":[6.5],"temperature_2m_min":[1.2]}}"#;

    #[test]
    fn an_answer_gives_the_weather_now_and_the_hours_from_this_one_on() {
        let w = parse_forecast(&serde_json::from_str(ANSWER).unwrap(), 9).unwrap();
        assert_eq!((w.temp, w.label.as_str(), w.kind, w.day, w.high, w.low), (4, "Cloudy", Kind::Cloudy, true, Some(7), Some(1)));
        assert_eq!(w.hours.iter().map(|h| h.t.as_str()).collect::<Vec<_>>(), ["2026-11-24T14:00", "2026-11-24T15:00"]);
        assert_eq!((w.hours[1].kind, w.hours[1].rain, w.hours[1].temp), (Kind::Rain, 70, 4));
    }

    #[test]
    fn the_hours_ahead_start_after_the_current_hour_at_the_place() {
        let w = parse_forecast(&serde_json::from_str(ANSWER).unwrap(), 9).unwrap();
        let at_14_10_utc = 1_795_528_800 + 10 * 60;
        assert_eq!(crate::util::utc_stamp(at_14_10_utc), "2026-11-24T14:10:00Z");
        assert_eq!(hours_ahead(&w, at_14_10_utc, 6).iter().map(|h| h.t.as_str()).collect::<Vec<_>>(), ["2026-11-24T15:00"]);
        let an_hour_east = Weather { offset: 3600, ..w };
        assert!(hours_ahead(&an_hour_east, at_14_10_utc, 6).is_empty());
    }

    #[test]
    fn a_kept_reading_comes_back_the_same() {
        let w = parse_forecast(&serde_json::from_str(ANSWER).unwrap(), 9).unwrap();
        assert_eq!(from_json(&to_json(&w)), Some(w));
    }

    #[test]
    fn a_city_is_found_by_its_whole_name() {
        let tsv = "Leeds\t53.8\t-1.55\nLeed\t1\t2\n";
        assert_eq!(place(tsv, "Leeds"), Some((53.8, -1.55)));
        assert_eq!(place(tsv, "Lee"), None);
    }

    #[test]
    fn night_shows_the_moon_only_where_the_sky_shows() {
        assert_eq!(icon(Kind::Clear, false), MOON);
        assert_eq!(icon(Kind::Partly, false), PARTLY_NIGHT);
        assert_eq!(icon(Kind::Rain, false), RAIN);
    }
}
