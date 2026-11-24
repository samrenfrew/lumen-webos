use std::time::{SystemTime, UNIX_EPOCH};

const DAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Local {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub weekday: usize,
    pub day: u32,
    pub month: usize,
}

pub fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

pub fn local(unix_seconds: u64) -> Local {
    let mut out = [0i32; 6];
    unsafe { crate::ffi::lumen_local_time(unix_seconds as i64, out.as_mut_ptr()) }
    Local { hour: out[0] as u32, minute: out[1] as u32, second: out[2] as u32, weekday: out[3].rem_euclid(7) as usize, day: out[4] as u32, month: out[5].rem_euclid(12) as usize }
}

pub fn time_text(at: &Local, twenty_four_hour: bool) -> (String, &'static str) {
    if twenty_four_hour {
        return (format!("{:02}:{:02}", at.hour, at.minute), "");
    }
    let hour = if at.hour % 12 == 0 { 12 } else { at.hour % 12 };
    (format!("{}:{:02}", hour, at.minute), if at.hour < 12 { "AM" } else { "PM" })
}

pub fn date_text(at: &Local) -> String {
    format!("{} {} {}", DAYS[at.weekday], at.day, MONTHS[at.month])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(hour: u32, minute: u32) -> Local {
        Local { hour, minute, second: 0, weekday: 2, day: 24, month: 10 }
    }

    #[test]
    fn the_24_hour_clock_keeps_its_leading_zero_and_has_no_period() {
        assert_eq!(time_text(&at(7, 5), true), ("07:05".to_string(), ""));
        assert_eq!(time_text(&at(0, 0), true), ("00:00".to_string(), ""));
    }

    #[test]
    fn the_12_hour_clock_calls_midnight_and_noon_twelve() {
        assert_eq!(time_text(&at(0, 9), false), ("12:09".to_string(), "AM"));
        assert_eq!(time_text(&at(12, 0), false), ("12:00".to_string(), "PM"));
        assert_eq!(time_text(&at(15, 42), false), ("3:42".to_string(), "PM"));
    }

    #[test]
    fn the_date_reads_weekday_day_month() {
        assert_eq!(date_text(&at(9, 0)), "Tuesday 24 November");
    }
}
