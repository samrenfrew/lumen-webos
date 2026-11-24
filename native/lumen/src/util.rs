use std::fmt::Arguments;
use std::fs;
use std::io::{IsTerminal, Write};
use std::os::unix::fs::PermissionsExt;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_PATH: OnceLock<String> = OnceLock::new();
const LOG_MAX_BYTES: u64 = 256 * 1024;

pub fn init() {
    let dir = format!("/tmp/lumen-native-{}", unsafe { crate::ffi::getuid() });
    let _ = fs::create_dir(&dir);
    let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o755));
    let path = format!("{dir}/log");
    if fs::metadata(&path).is_ok_and(|m| m.len() > LOG_MAX_BYTES) {
        let _ = fs::rename(&path, format!("{path}.old"));
    }
    let _ = LOG_PATH.set(path);
}

pub fn utc_stamp(seconds: u64) -> String {
    let days = (seconds / 86400) as i64;
    let in_day = seconds % 86400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z", in_day / 3600, in_day / 60 % 60, in_day % 60)
}

pub fn note(line: Arguments) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    if let Some(path) = LOG_PATH.get() {
        if let Ok(mut f) = fs::OpenOptions::new().append(true).create(true).open(path) {
            let _ = writeln!(f, "{} {}", utc_stamp(now), line);
        }
    }
    if std::io::stderr().is_terminal() {
        eprintln!("lumen {line}");
    }
}

#[macro_export]
macro_rules! note {
    ($($arg:tt)*) => { $crate::util::note(format_args!($($arg)*)) };
}

pub fn status_kb(field: &str) -> i64 {
    let Ok(status) = fs::read_to_string("/proc/self/status") else { return -1 };
    status.lines().find_map(|line| line.strip_prefix(field)?.strip_prefix(':')?.split_whitespace().next()?.parse().ok()).unwrap_or(-1)
}

#[cfg(test)]
mod tests {
    use super::utc_stamp;

    #[test]
    fn stamps_match_known_dates() {
        assert_eq!(utc_stamp(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc_stamp(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(utc_stamp(1_795_564_799), "2026-11-24T23:59:59Z");
    }
}
