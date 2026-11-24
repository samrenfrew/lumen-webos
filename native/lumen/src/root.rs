use crate::bus;
use serde_json::json;

const EXEC: &str = "luna://org.webosbrew.hbchannel.service/exec";
const HOMEHOOK: &str = "/var/lib/homehook";
const LOG: &str = "/var/lib/homehook/lumen.log";
const LOG_MAX_BYTES: usize = 64 * 1024;
const SHARED_FILES: [&str; 2] = ["lumen-config.json", "recent"];
const IMMICH_SERVER: &str = "immich.json";
const SHARED_LOGOS: &str = "/var/lib/homehook/logos";
const PROGRAM_PF: &str = "com.webos.service.legacybroadcast.programPF:4";
const PF_FIELDS: [&str; 5] = ["signalChannelId", "programName", "startTime", "endTime", "isPresent"];
const PROGRAM_SCH: &str = "com.webos.service.legacybroadcast.programSCH:4";
const SCH_FIELDS: [&str; 6] = ["signalChannelId", "programName", "startTime", "endTime", "isSubtitle", "isAD"];
pub const EVENT_BATCH: usize = 10;

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

fn luna(uri: &str, payload: serde_json::Value) -> String {
    format!("luna-send -n 1 {uri} {}", shell_quote(&payload.to_string()))
}

pub fn channel_id_ok(id: &str) -> bool {
    let parts: Vec<&str> = id.split('_').collect();
    parts.len() == 7 && parts.iter().all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

fn db8_page_ok(page: &str) -> bool {
    (1..=128).contains(&page.len()) && page.chars().all(|c| c.is_ascii_alphanumeric() || "+/=_.-".contains(c))
}

fn local_time_ok(stamp: &str) -> bool {
    let parts: Vec<&str> = stamp.split(',').collect();
    parts.len() == 6 && parts.iter().enumerate().all(|(i, p)| p.len() == if i == 0 { 4 } else { 2 } && p.bytes().all(|b| b.is_ascii_digit()))
}

fn channel_list_command() -> String {
    luna("luna://com.webos.service.iepg/getChannelList", json!({"dataType": 0, "sort": 0, "channelMode": "Terrestrial", "attribute": [{"filterType": "Invisible", "value": false}]}))
}

fn db8_find_command(mut query: serde_json::Value, page: Option<&str>) -> Option<String> {
    if let Some(page) = page {
        if !db8_page_ok(page) {
            return None;
        }
        query["page"] = json!(page);
    }
    Some(luna("luna://com.webos.epgdb/find", json!({ "query": query })))
}

fn now_next_command(page: Option<&str>, slim: bool) -> Option<String> {
    let mut query = json!({"from": PROGRAM_PF, "limit": 500});
    if slim {
        query["select"] = json!(PF_FIELDS);
    }
    db8_find_command(query, page)
}

fn schedule_command(page: Option<&str>) -> Option<String> {
    db8_find_command(json!({"from": PROGRAM_SCH, "select": SCH_FIELDS, "limit": 500}), page)
}

fn event_list_command(channel_ids: &[String], local_start: &str, local_end: &str) -> Option<String> {
    if channel_ids.is_empty() || channel_ids.len() > EVENT_BATCH || !channel_ids.iter().all(|id| channel_id_ok(id)) || !local_time_ok(local_start) || !local_time_ok(local_end) {
        return None;
    }
    Some(luna(
        "luna://com.webos.service.iepg/getEventList",
        json!({"bFVPChannelId": true, "channelId": channel_ids, "dataType": 0, "fepgType": "Forward", "localStartTime": local_start, "localEndTime": local_end, "sort": 0}),
    ))
}

fn current_channel_command() -> String {
    luna("luna://com.webos.service.utp.boot/getLastChannelId", json!({}))
}

fn open_channel_command(channel_id: &str) -> Option<String> {
    channel_id_ok(channel_id).then(|| luna("luna://com.webos.service.apiadapter/tv/openChannel", json!({ "channelId": channel_id })))
}

fn copy_logos_command(into_dir: &str) -> Option<String> {
    if !plain_absolute_path(into_dir) {
        return None;
    }
    let dir = shell_quote(into_dir);
    Some(format!("mkdir -p {dir}; for f in {SHARED_LOGOS}/*.png; do [ -f \"$f\" ] || continue; d={dir}/${{f##*/}}; [ -f \"$d\" ] || cp \"$f\" \"$d\"; done; ls {dir}"))
}

fn copy_shared_file_command(name: &str, into_dir: &str) -> Option<String> {
    if !SHARED_FILES.contains(&name) || !plain_absolute_path(into_dir) {
        return None;
    }
    Some(format!("cp -f {} {}", shell_quote(&format!("{HOMEHOOK}/{name}")), shell_quote(&format!("{into_dir}/{name}"))))
}

fn own_tmp_folder(dir: &str) -> bool {
    dir.strip_prefix("/tmp/lumen-native-").is_some_and(|uid| !uid.is_empty() && uid.len() <= 10 && uid.bytes().all(|b| b.is_ascii_digit()))
}

fn copy_immich_server_command(into_dir: &str) -> Option<String> {
    if !own_tmp_folder(into_dir) {
        return None;
    }
    let part = format!("{into_dir}/{IMMICH_SERVER}.part");
    Some(format!(
        "[ -f {HOMEHOOK}/{IMMICH_SERVER} ] || exit 0; cp -f {HOMEHOOK}/{IMMICH_SERVER} {part} && chown $(stat -c %u {into_dir}) {part} && chmod 600 {part} && mv -f {part} {into_dir}/{IMMICH_SERVER}"
    ))
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

pub fn copy_immich_server(into_dir: &str, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(copy_immich_server_command(into_dir), 6000, then);
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

pub fn channel_list(then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(Some(channel_list_command()), 12000, then);
}

pub fn now_next(page: Option<&str>, slim: bool, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(now_next_command(page, slim), 12000, then);
}

pub fn schedule(page: Option<&str>, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(schedule_command(page), 12000, then);
}

pub fn event_list(channel_ids: &[String], local_start: &str, local_end: &str, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(event_list_command(channel_ids, local_start, local_end), 12000, then);
}

pub fn current_channel(then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(Some(current_channel_command()), 6000, then);
}

pub fn open_channel(channel_id: &str, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(open_channel_command(channel_id), 6000, then);
}

pub fn copy_logos(into_dir: &str, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(copy_logos_command(into_dir), 15000, then);
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
    fn the_immich_key_is_copied_only_into_the_apps_own_folder_and_left_readable_by_that_app_alone() {
        assert_eq!(
            copy_immich_server_command("/tmp/lumen-native-6220").as_deref(),
            Some("[ -f /var/lib/homehook/immich.json ] || exit 0; cp -f /var/lib/homehook/immich.json /tmp/lumen-native-6220/immich.json.part && chown $(stat -c %u /tmp/lumen-native-6220) /tmp/lumen-native-6220/immich.json.part && chmod 600 /tmp/lumen-native-6220/immich.json.part && mv -f /tmp/lumen-native-6220/immich.json.part /tmp/lumen-native-6220/immich.json")
        );
        for elsewhere in ["/media/developer/lumen", "/tmp/lumen-native-", "/tmp/lumen-native-1/../x", "/tmp/lumen-native-1;reboot"] {
            assert_eq!(copy_immich_server_command(elsewhere), None, "{elsewhere}");
        }
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
    fn base64_matches_the_standard_examples() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64("11° é".as_bytes()), "MTHCsCDDqQ==");
    }

    #[test]
    fn live_tv_calls_take_only_channel_ids_paging_keys_and_times_of_the_tvs_own_shape() {
        let ids = ["1_41_1_101_16522_17546_9018".to_string()];
        let (from, to) = ("2026,11,24,15,00,00", "2026,11,24,23,00,00");
        assert_eq!(
            event_list_command(&ids, from, to).as_deref(),
            Some(r#"luna-send -n 1 luna://com.webos.service.iepg/getEventList '{"bFVPChannelId":true,"channelId":["1_41_1_101_16522_17546_9018"],"dataType":0,"fepgType":"Forward","localEndTime":"2026,11,24,23,00,00","localStartTime":"2026,11,24,15,00,00","sort":0}'"#)
        );
        assert_eq!(event_list_command(&["1_41_1_101_16522_17546".to_string()], from, to), None);
        assert_eq!(event_list_command(&ids, "2026,11,24,15,00", to), None);
        assert_eq!(event_list_command(&vec![ids[0].clone(); EVENT_BATCH + 1], from, to), None);
        assert_eq!(open_channel_command("1_41_1_101_16522_17546_9018';reboot'"), None);
        assert_eq!(
            now_next_command(Some("1mo403xMB5CPD5OWTZO1+F"), true).as_deref(),
            Some(r#"luna-send -n 1 luna://com.webos.epgdb/find '{"query":{"from":"com.webos.service.legacybroadcast.programPF:4","limit":500,"page":"1mo403xMB5CPD5OWTZO1+F","select":["signalChannelId","programName","startTime","endTime","isPresent"]}}'"#)
        );
        assert_eq!(now_next_command(Some("x' y"), false), None);
        assert_eq!(schedule_command(Some("")), None);
    }

    #[test]
    fn logos_are_copied_only_into_a_plain_folder() {
        assert_eq!(
            copy_logos_command("/media/developer/lumen/logos").as_deref(),
            Some(r#"mkdir -p '/media/developer/lumen/logos'; for f in /var/lib/homehook/logos/*.png; do [ -f "$f" ] || continue; d='/media/developer/lumen/logos'/${f##*/}; [ -f "$d" ] || cp "$f" "$d"; done; ls '/media/developer/lumen/logos'"#)
        );
        assert_eq!(copy_logos_command("/tmp/a b"), None);
    }

    #[test]
    fn a_log_line_goes_into_the_command_as_base64_only() {
        let command = append_log_command("boot '; reboot #\n");
        assert!(!command.contains("reboot"));
        assert!(command.contains(&format!("echo {} |", base64(b"boot '; reboot #\n"))));
    }
}
