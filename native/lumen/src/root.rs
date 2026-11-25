use crate::bus;
use serde_json::json;

const EXEC: &str = "luna://org.webosbrew.hbchannel.service/exec";
const HOMEHOOK: &str = "/var/lib/homehook";
const LOG: &str = "/var/lib/homehook/lumen.log";
const LOG_MAX_BYTES: usize = 64 * 1024;
const SHARED_FILES: [&str; 4] = ["lumen-config.json", "recent", "immich.json", "backlight-schedule"];
const IMMICH_SERVER: &str = "immich.json";
const SHARED_LOGOS: &str = "/var/lib/homehook/logos";
const PROGRAM_PF: &str = "com.webos.service.legacybroadcast.programPF:4";
const PF_FIELDS: [&str; 5] = ["signalChannelId", "programName", "startTime", "endTime", "isPresent"];
const PROGRAM_SCH: &str = "com.webos.service.legacybroadcast.programSCH:4";
const SCH_FIELDS: [&str; 6] = ["signalChannelId", "programName", "startTime", "endTime", "isSubtitle", "isAD"];
pub const EVENT_BATCH: usize = 10;
const GET_SETTINGS: &str = "luna://com.webos.settingsservice/getSystemSettings";
const SET_SETTINGS: &str = "luna://com.webos.settingsservice/setSystemSettings";
const SCREEN_OFF: &str = "luna://com.webos.service.tvpower/power/turnOffScreen";
pub const PICTURE_MODES: [&str; 7] = ["vivid", "normal", "eco", "cinema", "sports", "game", "filmMaker"];
pub const SLEEP_MINUTES: [&str; 9] = ["off", "10", "20", "30", "60", "90", "120", "180", "240"];
pub const SOUND_MODES: [&str; 7] = ["standard", "news", "movie", "music", "sports", "game", "aiSound"];
const NATIVE_DIR: &str = "/media/developer/apps/usr/palm/applications/com.samrenfrew.lumen.native";
const TV_SCRIPTS: [&str; 6] = ["watch.sh", "stadia.sh", "resume.sh", "home-setup.sh", "privacy.sh", "gear.sh"];
const BOOT_DIR: &str = "/var/lib/webosbrew/init.d";
const BOOT_HOOK: &str = "/var/lib/webosbrew/init.d/60-homehook";
const DEV_APPS: &str = "/media/developer/apps/usr/palm/applications";
const WEB_APPS: [&str; 3] = ["com.samrenfrew.lumen", "com.samrenfrew.lumen.guide", "com.samrenfrew.lumen.quick"];
pub const SIDE_APPS: [&str; 3] = ["com.samrenfrew.lumen.guide", "com.samrenfrew.lumen.quick", "com.samrenfrew.lumen.native.saver"];
const HOME_ACTIONS: [(&str, &str); 6] = [
    ("nativeOn", "native-home on com.samrenfrew.lumen.native"),
    ("nativeOff", "native-home off"),
    ("powerOnOn", "power-on on com.samrenfrew.lumen.native"),
    ("powerOnOff", "power-on off"),
    ("lgHome", "lg-home"),
    ("retireWeb", "retire-web"),
];
const PRIVACY_ACTIONS: [(&str, &str); 12] = [
    ("agreementsOff", "agreements off"),
    ("agreementsRestore", "agreements restore"),
    ("lmtOn", "lmt on"),
    ("lmtOff", "lmt off"),
    ("livePlusOn", "liveplus on"),
    ("livePlusOff", "liveplus off"),
    ("hostsOn", "hosts on"),
    ("hostsOff", "hosts off"),
    ("servicesOn", "services on"),
    ("servicesOff", "services off"),
    ("updatesOn", "updates on"),
    ("updatesOff", "updates off"),
];
const IMMICH_PATHS: [&str; 2] = ["/api/server/version", "/api/albums"];

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

fn luna_command(uri: &str, payload: serde_json::Value) -> String {
    format!("luna-send -n 1 {} {}", shell_quote(uri), shell_quote(&payload.to_string()))
}

fn setting_category(key: &str) -> Option<&'static str> {
    match key {
        "pictureMode" | "backlight" | "eyeComfortMode" => Some("picture"),
        "soundMode" => Some("sound"),
        "sleepTimer" => Some("time"),
        _ => None,
    }
}

fn setting_value_allowed(key: &str, value: &str) -> bool {
    match key {
        "pictureMode" => PICTURE_MODES.contains(&value),
        "soundMode" => SOUND_MODES.contains(&value),
        "sleepTimer" => SLEEP_MINUTES.contains(&value),
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

fn tv_status_command(app_dir: &str) -> Option<String> {
    if app_dir != NATIVE_DIR {
        return None;
    }
    let scripts = format!("{app_dir}/tv");
    let check: Vec<String> = TV_SCRIPTS.iter().map(|f| format!("if [ ! -f {HOMEHOOK}/{f} ]; then m=1; elif ! cmp -s {scripts}/{f} {HOMEHOOK}/{f}; then d=1; fi")).collect();
    Some(
        [
            "m=0; d=0".to_string(),
            check.join("; "),
            format!("if [ ! -f {BOOT_HOOK} ]; then m=1; elif ! cmp -s {scripts}/60-homehook {BOOT_HOOK}; then d=1; fi"),
            format!("if [ ! -d {scripts} ]; then echo scripts=unshipped; elif [ $m = 1 ]; then echo scripts=missing; elif [ $d = 1 ]; then echo scripts=old; else echo scripts=ok; fi"),
            web_state_command(),
            format!("[ -f {HOMEHOOK}/home-setup.sh ] && sh {HOMEHOOK}/home-setup.sh state"),
            format!("[ -f {HOMEHOOK}/privacy.sh ] && sh {HOMEHOOK}/privacy.sh state"),
            "true".to_string(),
        ]
        .join("; "),
    )
}

fn web_state_command() -> String {
    let files: Vec<String> = WEB_APPS.iter().map(|id| format!("{DEV_APPS}/{id}/appinfo.json")).collect();
    format!("w=none; for f in {}; do [ -f $f ] && w=installed; done; [ -f {HOMEHOOK}/retire-web ] && w=pending; echo web=$w", files.join(" "))
}

fn install_tv_scripts_command(app_dir: &str) -> Option<String> {
    if app_dir != NATIVE_DIR {
        return None;
    }
    let scripts = format!("{app_dir}/tv");
    let copy = |from: &str, to: &str| format!("cp {from} {to}.new && chmod +x {to}.new && mv -f {to}.new {to}");
    let mut steps = vec![format!("[ -d {scripts} ]"), format!("mkdir -p {HOMEHOOK} {BOOT_DIR}")];
    steps.extend(TV_SCRIPTS.iter().map(|f| copy(&format!("{scripts}/{f}"), &format!("{HOMEHOOK}/{f}"))));
    steps.push(copy(&format!("{scripts}/60-homehook"), BOOT_HOOK));
    steps.push(format!("{{ [ -f {HOMEHOOK}/hosts-extra ] || cp {scripts}/hosts-extra {HOMEHOOK}/hosts-extra; }}"));
    steps.push(format!("{BOOT_HOOK} restart"));
    steps.push("echo installed".to_string());
    Some(steps.join(" && "))
}

fn home_setup_command(action: &str) -> Option<String> {
    let verb = HOME_ACTIONS.iter().find(|(name, _)| *name == action)?.1;
    Some(format!("sh {HOMEHOOK}/home-setup.sh {verb} 2>&1; true"))
}

fn privacy_command(action: &str) -> Option<String> {
    let verb = PRIVACY_ACTIONS.iter().find(|(name, _)| *name == action)?.1;
    Some(format!("sh {HOMEHOOK}/privacy.sh {verb} 2>&1; true"))
}

fn is_version(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    parts.len() == 3 && parts.iter().zip([4, 4, 7]).all(|(part, most)| !part.is_empty() && part.len() <= most && part.bytes().all(|b| b.is_ascii_digit()))
}

fn side_app_versions_command() -> String {
    let each: Vec<String> = SIDE_APPS
        .iter()
        .map(|id| format!("v=$(sed -n 's/.*\"version\": *\"\\([0-9.]*\\)\".*/\\1/p' {DEV_APPS}/{id}/appinfo.json 2>/dev/null | head -1); [ -n \"$v\" ] && echo \"{id} $v\""))
        .collect();
    format!("{}; true", each.join("; "))
}

pub fn side_app_versions_read(out: &str) -> Vec<(String, String)> {
    out.lines()
        .filter_map(|line| {
            let (id, version) = line.trim().split_once(' ')?;
            (SIDE_APPS.contains(&id) && is_version(version)).then(|| (id.to_string(), version.to_string()))
        })
        .collect()
}

fn release_version_of(id: &str, url: &str) -> Option<String> {
    let rest = url.strip_prefix("https://github.com/samrenfrew/lumen-webos/releases/download/v")?;
    let (tag, file) = rest.split_once('/')?;
    let file = file.strip_prefix(id)?.strip_prefix('_')?;
    let version = file.strip_suffix("_all.ipk").or_else(|| file.strip_suffix("_arm.ipk"))?;
    (is_version(tag) && is_version(version)).then(|| version.to_string())
}

fn install_side_app_command(id: &str, url: &str, sha256: &str) -> Option<String> {
    if !SIDE_APPS.contains(&id) || sha256.len() != 64 || !sha256.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return None;
    }
    let version = release_version_of(id, url)?;
    let (ipk, info, quoted) = (format!("/tmp/{id}.ipk"), format!("{DEV_APPS}/{id}/appinfo.json"), shell_quote(url));
    Some(
        [
            format!("rm -f {ipk}"),
            format!("curl -sfL -m 120 -o {ipk} {quoted}; c=$?; [ $c = 60 ] && {{ curl -sfLk -m 120 -o {ipk} {quoted}; c=$?; }}"),
            format!("if [ $c != 0 ]; then echo \"Could not download it (curl $c)\"; rm -f {ipk}; exit 0; fi"),
            format!("if [ \"$(sha256sum {ipk} | cut -d' ' -f1)\" != {sha256} ]; then echo \"The download did not match its SHA-256, so it was not installed\"; rm -f {ipk}; exit 0; fi"),
            format!(
                "luna-send -w 90000 -i 'luna://com.webos.appInstallService/dev/install' {} >/dev/null 2>&1 & p=$!",
                shell_quote(&json!({"id": "com.ares.defaultName", "ipkUrl": ipk, "subscribe": true}).to_string())
            ),
            format!("n=0; while [ $n -lt 90 ] && ! grep -q '\"version\": *\"{version}\"' {info} 2>/dev/null; do sleep 1; n=$((n+1)); done"),
            format!("sleep 2; kill $p 2>/dev/null; rm -f {ipk}"),
            format!("[ -f {BOOT_HOOK} ] && {BOOT_HOOK} restart >/dev/null 2>&1"),
            format!("if grep -q '\"version\": *\"{version}\"' {info} 2>/dev/null; then echo installed; else echo \"Could not install it\"; fi"),
        ]
        .join("\n"),
    )
}

fn uninstall_side_app_command(id: &str) -> Option<String> {
    if !SIDE_APPS.contains(&id) {
        return None;
    }
    let present = format!("{} 2>/dev/null | grep -q '\"returnValue\":true'", luna_command("luna://com.webos.applicationManager/getAppInfo", json!({"id": id})));
    let dir = format!("{DEV_APPS}/{id}");
    Some(
        [
            format!("if {present} || [ -d {dir} ]; then {} >/dev/null 2>&1; fi", luna_command("luna://com.webos.appInstallService/dev/remove", json!({"id": id, "subscribe": false}))),
            format!("n=0; while [ $n -lt 30 ] && [ -d {dir} ]; do sleep 1; n=$((n+1)); done"),
            format!("[ -f {BOOT_HOOK} ] && {BOOT_HOOK} restart >/dev/null 2>&1"),
            format!("if {present} || [ -d {dir} ]; then echo kept; else echo gone; fi"),
        ]
        .join("; "),
    )
}

pub fn immich_address_ok(base: &str) -> bool {
    let Some(rest) = base.strip_prefix("http://").or_else(|| base.strip_prefix("https://")) else { return false };
    let (host, port) = match rest.split_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (rest, None),
    };
    let port_ok = port.is_none_or(|p| (1..=5).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_digit()));
    !host.is_empty() && host.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-') && port_ok
}

pub fn immich_key_ok(key: &str) -> bool {
    (16..=128).contains(&key.len()) && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn immich_request_command(base: &str, key: &str, path: &str) -> Option<String> {
    if !immich_address_ok(base) || !immich_key_ok(key) || !IMMICH_PATHS.contains(&path) {
        return None;
    }
    Some(format!("curl -sf -m 15 -H {} -H 'accept: application/json' {}", shell_quote(&format!("x-api-key: {key}")), shell_quote(&format!("{base}{path}"))))
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

pub fn read_shared_file(name: &str, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    let kept = name.to_string();
    copy_shared_file(name, crate::store::dir(), move |copied| then(copied.and_then(|_| crate::store::read(&kept).ok_or_else(|| "not there".to_string()))));
}

pub fn tv_status(app_dir: &str, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(tv_status_command(app_dir), 25000, then);
}

pub fn install_tv_scripts(app_dir: &str, then: impl FnOnce(Result<bool, String>) + Send + 'static) {
    run(install_tv_scripts_command(app_dir), 20000, move |out| then(out.map(|text| text.trim_end().ends_with("installed"))));
}

pub fn home_setup(action: &str, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(home_setup_command(action), 20000, then);
}

pub fn privacy(action: &str, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(privacy_command(action), 30000, then);
}

pub fn reboot(then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(Some("(sleep 1; reboot) >/dev/null 2>&1 &".to_string()), 6000, then);
}

pub fn side_app_versions(then: impl FnOnce(Result<Vec<(String, String)>, String>) + Send + 'static) {
    run(Some(side_app_versions_command()), 6000, move |out| then(out.map(|text| side_app_versions_read(&text))));
}

pub fn install_side_app(id: &str, url: &str, sha256: &str, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(install_side_app_command(id, url, sha256), 150000, move |out| then(out.map(|text| text.trim().lines().last().unwrap_or("Could not install it").to_string())));
}

pub fn uninstall_side_app(id: &str, then: impl FnOnce(Result<bool, String>) + Send + 'static) {
    run(uninstall_side_app_command(id), 45000, move |out| then(out.map(|text| text.contains("gone"))));
}

pub fn immich_request(base: &str, key: &str, path: &str, then: impl FnOnce(Result<String, String>) + Send + 'static) {
    run(immich_request_command(base, key, path), 20000, then);
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
    fn a_setting_is_changed_only_to_a_value_on_its_list() {
        assert_eq!(
            set_setting_command("pictureMode", "vivid").as_deref(),
            Some(r#"luna-send -n 1 'luna://com.webos.settingsservice/setSystemSettings' '{"category":"picture","settings":{"pictureMode":"vivid"}}'"#)
        );
        assert!(set_setting_command("soundMode", "aiSound").is_some());
        assert!(set_setting_command("backlight", "0").is_some() && set_setting_command("backlight", "100").is_some());
        assert_eq!(
            set_setting_command("sleepTimer", "30").as_deref(),
            Some(r#"luna-send -n 1 'luna://com.webos.settingsservice/setSystemSettings' '{"category":"time","settings":{"sleepTimer":"30"}}'"#)
        );
        for (key, value) in [("sleepTimer", "45"), ("pictureMode", "expert1"), ("backlight", "101"), ("backlight", "07"), ("backlight", "-1"), ("eyeComfortMode", "on'; reboot"), ("energySaving", "off")] {
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
    fn the_tv_scripts_are_read_and_installed_only_from_lumen_natives_own_folder() {
        let status = tv_status_command("/media/developer/apps/usr/palm/applications/com.samrenfrew.lumen.native").unwrap();
        assert!(status.contains("sh /var/lib/homehook/privacy.sh state"));
        assert!(install_tv_scripts_command("/media/developer/apps/usr/palm/applications/com.samrenfrew.lumen.native").unwrap().ends_with("echo installed"));
        for elsewhere in ["/tmp", "/media/developer/apps/usr/palm/applications/com.samrenfrew.lumen.native; reboot", ""] {
            assert_eq!(tv_status_command(elsewhere), None);
            assert_eq!(install_tv_scripts_command(elsewhere), None);
        }
    }

    #[test]
    fn only_known_home_and_privacy_changes_reach_the_scripts() {
        assert_eq!(privacy_command("hostsOn").as_deref(), Some("sh /var/lib/homehook/privacy.sh hosts on 2>&1; true"));
        assert_eq!(home_setup_command("nativeOff").as_deref(), Some("sh /var/lib/homehook/home-setup.sh native-home off 2>&1; true"));
        assert_eq!(privacy_command("hosts on; reboot"), None);
        assert_eq!(home_setup_command("nativeOn").as_deref(), Some("sh /var/lib/homehook/home-setup.sh native-home on com.samrenfrew.lumen.native 2>&1; true"));
        assert_eq!(home_setup_command("native-home on com.samrenfrew.lumen"), None);
    }

    #[test]
    fn a_side_app_installs_only_from_this_repos_releases_with_a_proper_checksum() {
        let id = "com.samrenfrew.lumen.guide";
        let url = "https://github.com/samrenfrew/lumen-webos/releases/download/v2.3.0/com.samrenfrew.lumen.guide_2.3.0_all.ipk";
        let sum = "a".repeat(64);
        assert!(install_side_app_command(id, url, &sum).unwrap().contains("\"version\": *\"2.3.0\""));
        assert_eq!(install_side_app_command(id, "https://evil.example/com.samrenfrew.lumen.guide_2.3.0_all.ipk", &sum), None);
        assert_eq!(install_side_app_command(id, &url.replace("guide_", "quick_"), &sum), None);
        assert_eq!(install_side_app_command(id, url, "'; reboot"), None);
        assert_eq!(install_side_app_command("bbc.iplayer.3.0", url, &sum), None);
        assert_eq!(uninstall_side_app_command("com.webos.app.home"), None);
        assert_eq!(side_app_versions_read("com.samrenfrew.lumen.quick 1.2.3\nother.app 1.0.0\ncom.samrenfrew.lumen.guide 1.x\n"), [("com.samrenfrew.lumen.quick".to_string(), "1.2.3".to_string())]);
    }

    #[test]
    fn an_immich_request_takes_only_a_plain_address_a_plain_key_and_a_known_path() {
        let key = "abcdefghijklmnop1234";
        assert!(immich_request_command("http://192.168.1.10:2283", key, "/api/albums").unwrap().ends_with("'http://192.168.1.10:2283/api/albums'"));
        assert_eq!(immich_request_command("http://192.168.1.10:2283/x", key, "/api/albums"), None);
        assert_eq!(immich_request_command("http://a b", key, "/api/albums"), None);
        assert_eq!(immich_request_command("http://photos", "short", "/api/albums"), None);
        assert_eq!(immich_request_command("http://photos", key, "/api/users"), None);
    }

    #[test]
    fn a_log_line_goes_into_the_command_as_base64_only() {
        let command = append_log_command("boot '; reboot #\n");
        assert!(!command.contains("reboot"));
        assert!(command.contains(&format!("echo {} |", base64(b"boot '; reboot #\n"))));
    }
}
