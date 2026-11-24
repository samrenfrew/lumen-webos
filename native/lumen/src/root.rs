use crate::bus;
use serde_json::json;

const EXEC: &str = "luna://org.webosbrew.hbchannel.service/exec";
const HOMEHOOK: &str = "/var/lib/homehook";
const LOG: &str = "/var/lib/homehook/lumen.log";
const LOG_MAX_BYTES: usize = 64 * 1024;
const SHARED_FILES: [&str; 2] = ["lumen-config.json", "recent"];
const GET_SETTINGS: &str = "luna://com.webos.settingsservice/getSystemSettings";
const SET_SETTINGS: &str = "luna://com.webos.settingsservice/setSystemSettings";
const SCREEN_OFF: &str = "luna://com.webos.service.tvpower/power/turnOffScreen";
pub const PICTURE_MODES: [&str; 7] = ["vivid", "normal", "eco", "cinema", "sports", "game", "filmMaker"];
pub const SOUND_MODES: [&str; 7] = ["standard", "news", "movie", "music", "sports", "game", "aiSound"];

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn plain_absolute_path(path: &str) -> bool {
    path.len() > 1 && path.starts_with('/') && path.chars().all(|c| c.is_ascii_alphanumeric() || "._/+@-".contains(c))
}

fn plain_file_name(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('.') && name.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
}

pub fn base64(bytes: &[u8]) -> String {
    const LETTERS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        for (i, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            out.push(if i <= chunk.len() { LETTERS[(n >> shift & 63) as usize] as char } else { '=' });
        }
    }
    out
}

fn luna_command(uri: &str, payload: serde_json::Value) -> String {
    format!("luna-send -n 1 {} {}", shell_quote(uri), shell_quote(&payload.to_string()))
}

fn setting_category(key: &str) -> Option<&'static str> {
    match key {
        "pictureMode" | "backlight" | "eyeComfortMode" => Some("picture"),
        "soundMode" => Some("sound"),
        _ => None,
    }
}

fn setting_value_allowed(key: &str, value: &str) -> bool {
    match key {
        "pictureMode" => PICTURE_MODES.contains(&value),
        "soundMode" => SOUND_MODES.contains(&value),
        "backlight" => value.parse::<u8>().is_ok_and(|level| level <= 100 && level.to_string() == value),
        "eyeComfortMode" => value == "on" || value == "off",
        _ => false,
    }
}

fn get_setting_command(key: &str) -> Option<String> {
    let category = setting_category(key)?;
    Some(luna_command(GET_SETTINGS, json!({"category": category, "keys": [key]})))
}

fn set_setting_command(key: &str, value: &str) -> Option<String> {
    let category = setting_category(key)?;
    if !setting_value_allowed(key, value) {
        return None;
    }
    Some(luna_command(SET_SETTINGS, json!({"category": category, "settings": {key: value}})))
}

fn setting_in_reply(key: &str, text: &str) -> Result<String, String> {
    let reply: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("not JSON: {e}"))?;
    if reply["returnValue"].as_bool() == Some(false) {
        return Err(format!("the TV did not read {key}"));
    }
    Ok(match &reply["settings"][key] {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(text) => text.clone(),
        other => other.to_string(),
    })
}

fn screen_off_command() -> String {
    luna_command(SCREEN_OFF, json!({}))
}

fn copy_shared_file_command(name: &str, into_dir: &str) -> Option<String> {
    if !SHARED_FILES.contains(&name) || !plain_absolute_path(into_dir) {
        return None;
    }
    Some(format!("cp -f {} {}", shell_quote(&format!("{HOMEHOOK}/{name}")), shell_quote(&format!("{into_dir}/{name}"))))
}

fn copy_icons_command(icons: &[(String, String)], into_dir: &str) -> Option<String> {
    if !plain_absolute_path(into_dir) {
        return None;
    }
    let copies: Vec<String> = icons
        .iter()
        .filter(|(path, name)| plain_absolute_path(path) && plain_file_name(name))
        .map(|(path, name)| format!("cp -f {} {} 2>/dev/null", shell_quote(path), shell_quote(&format!("{into_dir}/{name}"))))
        .collect();
    if copies.is_empty() {
        return None;
    }
    Some(format!("{}; true", copies.join("; ")))
}

fn append_log_command(text: &str) -> String {
    format!(
        "mkdir -p {HOMEHOOK} && echo {} | base64 -d >> {LOG} && if [ $(wc -c < {LOG}) -gt {LOG_MAX_BYTES} ]; then tail -c {} {LOG} > {LOG}.tmp && mv {LOG}.tmp {LOG}; fi",
        base64(text.as_bytes()),
        LOG_MAX_BYTES / 2
    )
}

fn write_shared_file_command(name: &str, text: &str) -> Option<String> {
    if !SHARED_FILES.contains(&name) {
        return None;
    }
    Some(format!("mkdir -p {HOMEHOOK} && echo {} | base64 -d > {HOMEHOOK}/{name}.tmp && mv {HOMEHOOK}/{name}.tmp {HOMEHOOK}/{name}", base64(text.as_bytes())))
}

pub fn can_uninstall(id: &str) -> bool {
    let kept = ["com.samrenfrew.lumen", "com.ammar027.lumen", "com.webos.", "com.palm.", "com.lge."];
    let plain = !id.is_empty() && id.len() < 128 && id.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c));
    plain && !kept.iter().any(|k| id.starts_with(k)) && id != "org.webosbrew.hbchannel"
}

fn uninstall_command(id: &str) -> Option<String> {
    if !can_uninstall(id) {
        return None;
    }
    let luna = |uri: &str, payload: serde_json::Value| format!("luna-send -n 1 {uri} {}", shell_quote(&payload.to_string()));
    let present = format!("{} 2>/dev/null | grep -q '\"returnValue\":true'", luna("luna://com.webos.applicationManager/getAppInfo", json!({"id": id})));
    let remove = |service: &str| format!("if {present}; then {} >/dev/null 2>&1; sleep 2; fi", luna(&format!("luna://com.webos.appInstallService/{service}"), json!({"id": id, "subscribe": false})));
    Some(format!("{}; {}; if {present}; then echo kept; else echo gone; fi", remove("dev/remove"), remove("remove")))
}

fn run(command: Option<String>, timeout_ms: i32, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    let Some(command) = command else { return then(Err("refused".into())) };
    bus::call(EXEC, json!({ "command": command }), timeout_ms, move |reply| then(reply.map(|r| r["stdoutString"].as_str().unwrap_or("").to_string())));
}

pub fn copy_shared_file(name: &str, into_dir: &str, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(copy_shared_file_command(name, into_dir), 6000, then);
}

pub fn copy_icons(icons: &[(String, String)], into_dir: &str, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(copy_icons_command(icons, into_dir), 15000, then);
}

pub fn write_shared_file(name: &str, text: &str, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(write_shared_file_command(name, text), 8000, then);
}

pub fn uninstall(id: &str, then: impl FnOnce(Result<bool, String>) + Send + 'static) {
    run(uninstall_command(id), 30000, move |out| then(out.map(|text| text.contains("gone"))));
}

pub fn get_setting(key: &'static str, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(get_setting_command(key), 6000, move |out| then(out.and_then(|text| setting_in_reply(key, &text))));
}

pub fn set_setting(key: &str, value: &str, then: impl FnOnce(Result<(), String>) + Send + 'static) {
    let refused = format!("the TV refused {key}={value}");
    run(set_setting_command(key, value), 6000, move |out| {
        then(out.and_then(|text| match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(reply) if reply["returnValue"].as_bool() == Some(false) => Err(refused),
            _ => Ok(()),
        }))
    });
}

pub fn screen_off(then: impl FnOnce(Result<(), String>) + Send + 'static) {
    run(Some(screen_off_command()), 6000, move |out| then(out.map(|_| ())));
}

pub fn append_log(text: &str) {
    run(Some(append_log_command(text)), 6000, |_| {});
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quoted_value_cannot_end_its_own_quotes() {
        assert_eq!(shell_quote("it's"), r"'it'\''s'");
        assert_eq!(shell_quote("$(reboot)"), "'$(reboot)'");
    }

    #[test]
    fn only_the_shared_files_lumen_knows_are_copied() {
        assert_eq!(copy_shared_file_command("lumen-config.json", "/media/developer/lumen").as_deref(), Some("cp -f '/var/lib/homehook/lumen-config.json' '/media/developer/lumen/lumen-config.json'"));
        assert_eq!(copy_shared_file_command("../../etc/shadow", "/media/developer/lumen"), None);
        assert_eq!(copy_shared_file_command("recent", "/tmp/a b"), None);
    }

    #[test]
    fn an_icon_path_with_shell_syntax_never_reaches_the_command() {
        let icons = [
            ("/usr/palm/applications/a/icon.png".to_string(), "00ff.png".to_string()),
            ("/usr/palm/x.png; reboot".to_string(), "11ff.png".to_string()),
            ("/usr/palm/b's.png".to_string(), "22ff.png".to_string()),
            ("relative.png".to_string(), "33ff.png".to_string()),
            ("/usr/palm/c.png".to_string(), "../up.png".to_string()),
        ];
        assert_eq!(copy_icons_command(&icons, "/media/developer/lumen/icons").as_deref(), Some("cp -f '/usr/palm/applications/a/icon.png' '/media/developer/lumen/icons/00ff.png' 2>/dev/null; true"));
        assert_eq!(copy_icons_command(&icons[1..], "/media/developer/lumen/icons"), None);
    }

    #[test]
    fn settings_are_written_as_base64_and_only_to_a_known_file() {
        let command = write_shared_file_command("lumen-config.json", "{\"a\":\"'; reboot\"}").unwrap();
        assert!(!command.contains("reboot"));
        assert!(command.ends_with("mv /var/lib/homehook/lumen-config.json.tmp /var/lib/homehook/lumen-config.json"));
        assert_eq!(write_shared_file_command("../x", "{}"), None);
    }

    #[test]
    fn lumen_the_tvs_own_apps_and_the_homebrew_channel_are_never_uninstalled() {
        assert!(can_uninstall("bbc.iplayer.3.0"));
        for kept in ["com.samrenfrew.lumen.native", "com.webos.app.livetv", "com.palm.app.settings", "com.lge.app.x", "org.webosbrew.hbchannel", "a;reboot", ""] {
            assert!(!can_uninstall(kept), "{kept}");
            assert_eq!(uninstall_command(kept), None);
        }
        let command = uninstall_command("bbc.iplayer.3.0").unwrap();
        assert!(command.contains("dev/remove") && command.contains("'{\"id\":\"bbc.iplayer.3.0\",\"subscribe\":false}'"));
    }

    #[test]
    fn a_setting_is_changed_only_to_a_value_on_its_list() {
        assert_eq!(
            set_setting_command("pictureMode", "vivid").as_deref(),
            Some(r#"luna-send -n 1 'luna://com.webos.settingsservice/setSystemSettings' '{"category":"picture","settings":{"pictureMode":"vivid"}}'"#)
        );
        assert!(set_setting_command("soundMode", "aiSound").is_some());
        assert!(set_setting_command("backlight", "0").is_some() && set_setting_command("backlight", "100").is_some());
        for (key, value) in [("pictureMode", "expert1"), ("backlight", "101"), ("backlight", "07"), ("backlight", "-1"), ("eyeComfortMode", "on'; reboot"), ("energySaving", "off")] {
            assert_eq!(set_setting_command(key, value), None, "{key}={value}");
        }
    }

    #[test]
    fn only_known_settings_are_read_and_the_value_comes_back_as_text() {
        assert_eq!(get_setting_command("backlight").as_deref(), Some(r#"luna-send -n 1 'luna://com.webos.settingsservice/getSystemSettings' '{"category":"picture","keys":["backlight"]}'"#));
        assert_eq!(get_setting_command("soundMode; reboot"), None);
        let reply = r#"{"subscribed":false,"category":"picture","settings":{"pictureMode":"normal","backlight":"80"},"returnValue":true}"#;
        assert_eq!(setting_in_reply("backlight", reply), Ok("80".to_string()));
        assert_eq!(setting_in_reply("eyeComfortMode", reply), Ok(String::new()));
        assert!(setting_in_reply("backlight", r#"{"returnValue":false}"#).is_err());
    }

    #[test]
    fn the_screen_goes_off_through_the_tvs_power_service() {
        assert_eq!(screen_off_command(), "luna-send -n 1 'luna://com.webos.service.tvpower/power/turnOffScreen' '{}'");
    }

    #[test]
    fn base64_matches_the_standard_examples() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64("11° é".as_bytes()), "MTHCsCDDqQ==");
    }

    #[test]
    fn a_log_line_goes_into_the_command_as_base64_only() {
        let command = append_log_command("boot '; reboot #\n");
        assert!(!command.contains("reboot"));
        assert!(command.contains(&format!("echo {} |", base64(b"boot '; reboot #\n"))));
    }
}
