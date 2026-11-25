use crate::ffi::*;
use crate::note;
use std::ffi::{c_char, c_void, CStr, CString};
use std::fs;
use std::io::Write;
use std::time::{Instant, SystemTime};

const PROBE_PATH: &str = "/tmp/rtest-probe";
const SHOWN: usize = 900;
const FULLSCREEN: std::ffi::c_int = 3;

pub struct Probe {
    seen: Option<SystemTime>,
    window: *mut SDL_Window,
}

fn taken(text: *mut c_char) -> Option<String> {
    if text.is_null() {
        return None;
    }
    let owned = unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned();
    unsafe { lumen_luna_free(text) }
    Some(owned)
}

fn shortened(text: &str) -> String {
    let cut: String = text.chars().take(SHOWN).collect();
    format!("{} bytes: {}", text.len(), cut)
}

extern "C" fn heard(reply: *const c_char, user: *mut c_void) {
    let uri = unsafe { CStr::from_ptr(user.cast::<c_char>()) }.to_string_lossy();
    let reply = unsafe { CStr::from_ptr(reply) }.to_string_lossy();
    note!("probe heard {}: {}", uri, shortened(&reply));
    if reply.contains("\"event\":\"close\"") {
        note!("close: the app manager asked");
        std::process::exit(0);
    }
    if reply.contains("\"event\":\"relaunch\"") {
        note!("relaunch: asked for full screen, {}", unsafe { lumen_shell_state(FULLSCREEN) });
    }
}

fn open(name: &str) -> bool {
    let c_name = CString::new(name).unwrap_or_default();
    let failed = taken(unsafe { lumen_luna_open(if name == "-" { std::ptr::null() } else { c_name.as_ptr() }) });
    note!("probe luna as {}: {}", name, failed.as_deref().unwrap_or("open"));
    failed.is_none()
}

fn call(uri: &str, payload: &str) {
    let (c_uri, c_payload) = (CString::new(uri).unwrap_or_default(), CString::new(payload).unwrap_or_default());
    let began = Instant::now();
    let reply = taken(unsafe { lumen_luna_call(c_uri.as_ptr(), c_payload.as_ptr(), 8000) }).unwrap_or_default();
    note!("probe call {} {} in {} ms: {}", uri, payload, began.elapsed().as_millis(), shortened(&reply));
}

fn subscribe(uri: &str, payload: &str) {
    let (c_uri, c_payload) = (CString::new(uri).unwrap_or_default(), CString::new(payload).unwrap_or_default());
    let failed = taken(unsafe { lumen_luna_subscribe(c_uri.as_ptr(), c_payload.as_ptr(), heard, c_uri.into_raw().cast()) });
    note!("probe sub {} {}: {}", uri, payload, failed.unwrap_or_else(|| "listening".into()));
}

fn write(path: &str) {
    let made = fs::OpenOptions::new().create(true).append(true).open(path).and_then(|mut f| f.write_all(b"lumen probe\n"));
    note!("probe write {}: {}", path, made.map_or_else(|e| e.to_string(), |_| "written".into()));
}

fn read(path: &str) {
    match fs::read(path) {
        Ok(bytes) => note!("probe read {}: {}", path, shortened(&String::from_utf8_lossy(&bytes))),
        Err(e) => note!("probe read {}: {}", path, e),
    }
}

fn list(path: &str) {
    match fs::read_dir(path) {
        Ok(dir) => {
            let mut names: Vec<String> = dir.flatten().map(|entry| entry.file_name().to_string_lossy().into_owned()).collect();
            names.sort();
            note!("probe ls {}: {} entries: {}", path, names.len(), shortened(&names.join(" ")));
        }
        Err(e) => note!("probe ls {}: {}", path, e),
    }
}

fn whoami() {
    let status = fs::read_to_string("/proc/self/status").unwrap_or_default();
    let ids: Vec<&str> = status.lines().filter(|l| l.starts_with("Uid") || l.starts_with("Gid") || l.starts_with("Groups")).collect();
    note!("probe id: {}, cwd {:?}", ids.join(", ").replace('\t', " "), std::env::current_dir().ok());
    let env: Vec<String> = std::env::vars().map(|(k, v)| format!("{k}={v}")).collect();
    note!("probe env: {}", shortened(&env.join(" ")));
}

pub fn register_first(app_id: &str) {
    if open(app_id) {
        subscribe("luna://com.webos.applicationManager/registerApp", "{}");
    }
}

impl Probe {
    pub fn open(window: *mut SDL_Window) -> Probe {
        let args: Vec<String> = std::env::args().collect();
        note!("probe args: {:?}", args);
        Probe { seen: None, window }
    }

    pub fn tick(&mut self) {
        unsafe { lumen_luna_pump() }
        let changed = fs::metadata(PROBE_PATH).and_then(|m| m.modified()).ok();
        if changed.is_none() || changed == self.seen {
            return;
        }
        self.seen = changed;
        let Ok(script) = fs::read_to_string(PROBE_PATH) else { return };
        for line in script.lines() {
            let mut words = line.splitn(3, ' ');
            let (verb, first, rest) = (words.next().unwrap_or(""), words.next().unwrap_or(""), words.next().unwrap_or("{}"));
            match verb {
                "open" => {
                    open(first);
                }
                "call" => call(first, rest),
                "sub" => subscribe(first, rest),
                "write" => write(first),
                "read" => read(first),
                "ls" => list(first),
                "id" => whoami(),
                "raise" => {
                    match first {
                        "sdl" => unsafe { SDL_RaiseWindow(self.window) },
                        "show" => unsafe { SDL_ShowWindow(self.window) },
                        "hide" => unsafe { SDL_HideWindow(self.window) },
                        _ => note!("probe shell state {}: {}", first, unsafe { lumen_shell_state(first.parse().unwrap_or(0)) }),
                    }
                    note!("probe raise {}", first);
                }
                _ => {}
            }
        }
        note!("probe done");
    }
}
