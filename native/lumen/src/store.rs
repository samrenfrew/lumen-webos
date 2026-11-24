use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::OnceLock;

static DIR: OnceLock<String> = OnceLock::new();

pub fn open(app_dir: &str) {
    let dir = std::env::var("LUMEN_STORE").unwrap_or_else(|_| if cfg!(feature = "webos") { "/media/developer/lumen".into() } else { format!("{app_dir}/store") });
    for folder in [dir.clone(), format!("{dir}/icons")] {
        let _ = fs::create_dir_all(&folder);
        let _ = fs::set_permissions(&folder, fs::Permissions::from_mode(0o777));
    }
    let _ = DIR.set(dir);
}

pub fn dir() -> &'static str {
    DIR.get().map_or(".", String::as_str)
}

pub fn path(name: &str) -> String {
    format!("{}/{}", dir(), name)
}

pub fn read(name: &str) -> Option<String> {
    fs::read_to_string(path(name)).ok()
}

pub fn write(name: &str, bytes: &[u8]) -> bool {
    let (to, part) = (path(name), path(&format!("{name}.part")));
    fs::write(&part, bytes).and_then(|_| fs::rename(&part, &to)).is_ok()
}

pub fn has(name: &str) -> bool {
    Path::new(&path(name)).exists()
}
