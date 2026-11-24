use crate::bus;
use serde_json::json;

const EXEC: &str = "luna://org.webosbrew.hbchannel.service/exec";
const HOMEHOOK: &str = "/var/lib/homehook";
const LOG: &str = "/var/lib/homehook/lumen.log";
const LOG_MAX_BYTES: usize = 64 * 1024;
const SHARED_FILES: [&str; 2] = ["lumen-config.json", "recent"];

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
