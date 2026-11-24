mod anim;
mod apps;
mod bus;
mod clock;
#[cfg(feature = "dev")]
mod control;
mod draw;
mod ffi;
mod gfx;
mod home;
mod look;
mod pictures;
mod root;
mod settings;
mod shaders;
mod store;
mod text;
mod util;
mod wallpaper;

use draw::{decoded, Draw};
use ffi::*;
use gfx::{DESIGN_H, DESIGN_W};
use home::{Home, Key, Shown};
use look::Look;
use serde_json::{json, Value};
use settings::{ClockFormat, Settings};
use std::collections::HashSet;
use std::ffi::{c_int, CStr, CString};
use std::sync::Mutex;
use std::time::Instant;
use util::status_kb;

pub const APP_ID: &str = "com.samrenfrew.lumen.native";
const WEBOS_BACK_SCANCODE: c_int = 482;
const SHELL_FULL_SCREEN: c_int = 3;
const LAUNCH: &str = "luna://com.webos.applicationManager/launch";
const LAUNCH_POINTS: &str = "luna://com.webos.applicationManager/listLaunchPoints";
const REGISTER: &str = "luna://com.webos.applicationManager/registerApp";
const PREFERENCES: &str = "luna://com.webos.service.systemservice/getPreferences";
const SETTINGS_FILE: &str = "lumen-config.json";
const LAUNCH_POINTS_FILE: &str = "launch-points.json";
const LOOK_FILE: &str = "look";
const APPS_WITHOUT_SETTINGS: usize = 24;
const BOOT_WINDOW_MS: u128 = 5000;
const SHOT_SETTLED_MS: u32 = 100_000;
const SHOT_MAY_DIFFER_BY: i32 = 8;
const SHOT_PIXELS_THAT_MAY_DIFFER: usize = 200;

type Job = Box<dyn FnOnce(&mut App) + Send>;
static INBOX: Mutex<Vec<Job>> = Mutex::new(Vec::new());

pub fn post(job: impl FnOnce(&mut App) + Send + 'static) {
    if let Ok(mut inbox) = INBOX.lock() {
        inbox.push(Box::new(job));
    }
    let mut wake = SDL_Event { padding: [0; 7] };
    wake.kind = SDL_USEREVENT;
    unsafe { SDL_PushEvent(&mut wake) };
}

pub struct App {
    window: *mut SDL_Window,
    draw: Draw,
    home: Home,
    settings: Settings,
    known: Vec<apps::Known>,
    launch_points: Value,
    icons_asked_for: HashSet<String>,
    tv_clock_is_24_hour: bool,
    fixed_clock: Option<(String, String)>,
    fetching_wallpaper: bool,
    fetch_wallpapers: bool,
    hidden: bool,
    redraw: bool,
    running: bool,
    began: Instant,
    boot: Vec<String>,
    boot_written: bool,
}

fn ticks() -> u32 {
    unsafe { SDL_GetTicks() }
}

fn sdl_error() -> String {
    unsafe { CStr::from_ptr(SDL_GetError()) }.to_string_lossy().into_owned()
}

fn gl_string(name: GLenum) -> String {
    let s = unsafe { glGetString(name) };
    if s.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(s) }.to_string_lossy().into_owned()
}

fn key_of(sym: i32, scancode: c_int) -> Option<Key> {
    Some(match sym {
        SDLK_LEFT => Key::Left,
        SDLK_RIGHT => Key::Right,
        SDLK_UP => Key::Up,
        SDLK_DOWN => Key::Down,
        SDLK_RETURN | SDLK_KP_ENTER => Key::Ok,
        SDLK_ESCAPE | SDLK_BACKSPACE | SDLK_AC_BACK => Key::Back,
        _ if scancode == WEBOS_BACK_SCANCODE => Key::Back,
        _ => return None,
    })
}

fn key_named(name: &str) -> Option<Key> {
    Some(match name {
        "left" => Key::Left,
        "right" => Key::Right,
        "up" => Key::Up,
        "down" => Key::Down,
        "ok" | "enter" => Key::Ok,
        "back" => Key::Back,
        _ => return None,
    })
}

pub fn launch_view(launch_argument: &str) -> Option<String> {
    let launch: Value = serde_json::from_str(launch_argument).ok()?;
    launch["parameters"]["view"].as_str().map(String::from)
}

impl App {
    fn mark(&mut self, stage: &str) {
        let at = self.began.elapsed().as_millis();
        if at < BOOT_WINDOW_MS {
            self.boot.push(format!("{stage} {at}"));
        }
    }

    fn write_boot_line(&mut self) {
        self.boot_written = true;
        let line = format!("boot native {}, RSS {} kB", self.boot.join(", "), status_kb("VmRSS"));
        note!("{}", line);
        root::append_log(&format!("{} {}\n", util::utc_stamp(clock::unix_now()), line));
    }

    fn load_kept(&mut self) {
        self.known = apps::catalogue(&std::fs::read_to_string(format!("{}/assets/catalog.tsv", self.draw.app_dir)).unwrap_or_default());
        if let Some(kept) = store::read(SETTINGS_FILE).and_then(|text| Settings::parse(&text)) {
            self.settings = kept;
        }
        self.launch_points = store::read(LAUNCH_POINTS_FILE).and_then(|text| serde_json::from_str(&text).ok()).unwrap_or(Value::Null);
        if self.settings.new_wallpaper_on_launch {
            wallpaper::promote_next();
        }
        self.show_wallpaper();
        self.mark("wallpaper");
        self.show_apps();
        self.tick_clock();
    }

    fn show_wallpaper(&mut self) {
        let bundled = format!("{}/wallpaper.jpg", self.draw.app_dir);
        let Some(picture) = decoded(&store::path(wallpaper::SHOWN)).or_else(|| decoded(&bundled)) else { return note!("wallpaper: none could be read") };
        self.home.set_wallpaper(&mut self.draw, &picture);
        self.redraw = true;
    }

    fn icon_file(&mut self, icon: &apps::Icon, to_copy: &mut Vec<(String, String)>) -> Option<String> {
        match icon {
            apps::Icon::Bundled(file) => Some(format!("{}/icons/apps/{}", self.draw.app_dir, file)),
            apps::Icon::Missing => None,
            apps::Icon::OnTv(path) => {
                let name = apps::kept_icon_name(path);
                let kept = store::path(&format!("icons/{name}"));
                if std::fs::File::open(&kept).is_ok() {
                    return Some(kept);
                }
                if std::fs::File::open(path).is_ok() {
                    return Some(path.clone());
                }
                if self.icons_asked_for.insert(path.clone()) {
                    to_copy.push((path.clone(), name));
                }
                None
            }
        }
    }

    fn show_apps(&mut self) {
        let listed = apps::listed(&self.launch_points, &self.known);
        let chosen = if self.settings.app_ids.is_empty() { listed.into_iter().take(APPS_WITHOUT_SETTINGS).collect() } else { apps::in_order(&listed, &self.settings.app_ids) };
        let mut to_copy = Vec::new();
        let shown: Vec<Shown> = chosen.into_iter().map(|app| Shown { icon_file: self.icon_file(&app.icon, &mut to_copy), id: app.id, title: app.title, color: app.color }).collect();
        self.home.reduce_motion = self.settings.reduce_motion;
        self.home.set_apps(&mut self.draw, shown, self.settings.first_row, ticks());
        self.redraw = true;
        if !to_copy.is_empty() {
            note!("icons: asking for {} the app cannot read", to_copy.len());
            root::copy_icons(&to_copy, &store::path("icons"), |copied| match copied {
                Ok(_) => post(|app| app.show_apps()),
                Err(why) => note!("icons: {}", why),
            });
        }
    }

    fn tick_clock(&mut self) -> u32 {
        let now = clock::local(clock::unix_now());
        let twenty_four_hour = match self.settings.clock_format {
            ClockFormat::TwentyFour => true,
            ClockFormat::Twelve => false,
            ClockFormat::FollowTv => self.tv_clock_is_24_hour,
        };
        let (time, period) = clock::time_text(&now, twenty_four_hour);
        let (time, date) = self.fixed_clock.clone().unwrap_or((time, clock::date_text(&now)));
        let period = if self.settings.show_am_pm && self.fixed_clock.is_none() { period } else { "" };
        if self.home.set_clock(&mut self.draw, time, period.to_string(), date) {
            self.redraw = true;
        }
        (60 - now.second.min(59)) * 1000 + 50
    }

    fn refresh_from_tv(&mut self) {
        root::copy_shared_file(SETTINGS_FILE, store::dir(), |copied| match copied {
            Ok(_) => post(|app| app.settings_copied()),
            Err(why) => note!("settings: {}", why),
        });
        bus::call(LAUNCH_POINTS, json!({}), 8000, |reply| match reply {
            Ok(points) => post(move |app| app.launch_points_arrived(points)),
            Err(why) => note!("apps: {}", why),
        });
        bus::call(PREFERENCES, json!({"keys": ["timeFormat"]}), 4000, |reply| {
            if let Ok(preferences) = reply {
                let twenty_four_hour = preferences["timeFormat"].as_str() != Some("HH12");
                post(move |app| {
                    app.tv_clock_is_24_hour = twenty_four_hour;
                    app.tick_clock();
                });
            }
        });
        self.fetch_a_wallpaper_if_wanted();
    }

    fn settings_copied(&mut self) {
        self.mark("settings");
        let Some(copied) = store::read(SETTINGS_FILE).and_then(|text| Settings::parse(&text)) else { return note!("settings: the copy could not be read") };
        if copied == self.settings {
            return;
        }
        self.settings = copied;
        self.show_apps();
        self.tick_clock();
        self.fetch_a_wallpaper_if_wanted();
    }

    fn launch_points_arrived(&mut self, points: Value) {
        self.mark("apps");
        if points == self.launch_points {
            return;
        }
        store::write(LAUNCH_POINTS_FILE, points.to_string().as_bytes());
        self.launch_points = points;
        self.show_apps();
    }

    fn fetch_a_wallpaper_if_wanted(&mut self) {
        // ponytail: pictures only, fetched at start; Immich, plain, and a new picture every few minutes come with the wallpaper settings
        if !self.fetch_wallpapers || self.fetching_wallpaper || self.settings.wallpaper_type != "pictures" {
            return;
        }
        let as_next = store::has(wallpaper::SHOWN);
        if as_next && (!self.settings.new_wallpaper_on_launch || store::has(wallpaper::NEXT)) {
            return;
        }
        let shown = store::read(wallpaper::SHOWN_PICK);
        let listed = std::fs::read_to_string(format!("{}/assets/wallpapers.tsv", self.draw.app_dir)).unwrap_or_default();
        let known_picks: Vec<String> = self.settings.wallpaper_picks.iter().filter(|pick| wallpaper::url_of(&listed, pick).is_some()).cloned().collect();
        let chance = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos() as u64);
        let Some(pick) = wallpaper::another(&known_picks, shown.as_deref(), chance) else { return };
        let Some(url) = wallpaper::url_of(&listed, &pick) else { return };
        self.fetching_wallpaper = true;
        wallpaper::fetch(pick.clone(), url, as_next, move |fetched| {
            note!("wallpaper {}: {}", pick, fetched.as_ref().map_or_else(|why| why.as_str(), |_| if as_next { "kept for the next launch" } else { "fetched" }));
            post(move |app| {
                app.fetching_wallpaper = false;
                if fetched.is_ok() && !as_next {
                    app.show_wallpaper();
                }
            });
        });
    }

    fn key(&mut self, key: Key) {
        self.redraw = true;
        if let Some(id) = self.home.key(&self.draw, key, ticks()) {
            note!("open {}", id);
            bus::call(LAUNCH, json!({"id": id}), 6000, move |reply| {
                if let Err(why) = reply {
                    note!("open {}: {}", id, why);
                }
            });
        }
    }

    fn relaunched(&mut self, parameters: Value) {
        #[cfg(feature = "webos")]
        let came_forward = unsafe { lumen_shell_state(SHELL_FULL_SCREEN) };
        #[cfg(not(feature = "webos"))]
        let came_forward = SHELL_FULL_SCREEN;
        note!("relaunch {}: asked for full screen, {}", parameters, came_forward);
        if !self.hidden {
            self.home.go_home(&self.draw, ticks());
        }
        self.redraw = true;
    }

    fn window_event(&mut self, event: u8) {
        match event {
            SDL_WINDOWEVENT_FOCUS_LOST => {
                self.hidden = true;
                self.home.hidden(&mut self.draw);
                note!("hidden: {} kB of pictures kept", (self.draw.pictures.bytes() + self.home.backdrop_bytes()) / 1024);
            }
            SDL_WINDOWEVENT_FOCUS_GAINED => {
                self.hidden = false;
                self.redraw = true;
                note!("shown");
            }
            _ => self.redraw = true,
        }
    }

    fn stats(&self) -> Value {
        json!({
            "rss_kb": status_kb("VmRSS"),
            "peak_kb": status_kb("VmHWM"),
            "pictures": self.draw.pictures.count(),
            "picture_kb": self.draw.pictures.bytes() / 1024,
            "backdrop_kb": self.home.backdrop_bytes() / 1024,
            "refresh_ms": self.home.refresh_ms,
            "last_move": self.home.last_move,
            "hidden": self.hidden,
            "up_ms": self.began.elapsed().as_millis() as u64,
        })
    }

    fn frame_pixels(&mut self, now: u32) -> *mut SDL_Surface {
        self.home.draw(&mut self.draw, now);
        unsafe { glFinish() }
        self.draw.gfx.read_screen()
    }

    #[cfg(feature = "dev")]
    pub fn control(&mut self, line: &str, answer: control::Answer) {
        use control::said;
        let mut words = line.splitn(3, ' ');
        let (verb, first, rest) = (words.next().unwrap_or(""), words.next().unwrap_or(""), words.next().unwrap_or(""));
        match verb {
            "where" => said(&answer, self.home.where_now()),
            "stats" => said(&answer, self.stats()),
            "get" => said(&answer, Look::names().iter().map(|name| (name.to_string(), json!(self.draw.look.get(name)))).collect::<serde_json::Map<_, _>>().into()),
            "set" | "reset" => {
                let taken = if verb == "reset" {
                    self.draw.look = Look::defaults();
                    true
                } else {
                    rest.parse().is_ok_and(|value| self.draw.look.set(first, value))
                };
                store::write(LOOK_FILE, self.draw.look.changed_from_defaults().as_bytes());
                self.home.look_changed(&mut self.draw);
                self.redraw = true;
                said(&answer, json!({"set": taken, "kept": self.draw.look.changed_from_defaults()}));
            }
            "key" => {
                let keys: Vec<Key> = line.split(' ').skip(1).filter_map(key_named).collect();
                for key in &keys {
                    self.key(*key);
                }
                said(&answer, json!({"pressed": keys.len()}));
            }
            "shot" => {
                let path = format!("/tmp/lumen-native-{}/shot.png", unsafe { getuid() });
                let shot = self.frame_pixels(ticks());
                let saved = !shot.is_null() && unsafe { IMG_SavePNG(shot, CString::new(path.as_str()).unwrap_or_default().as_ptr()) } == 0;
                unsafe { SDL_FreeSurface(shot) }
                self.redraw = true;
                match std::fs::read(&path).ok().filter(|_| saved) {
                    Some(png) => {
                        let mut bytes = format!("{}\n", json!({"png": png.len()})).into_bytes();
                        bytes.extend(png);
                        let _ = answer.send(bytes);
                    }
                    None => said(&answer, json!({"error": "no picture"})),
                }
            }
            "call" => match serde_json::from_str::<Value>(if rest.is_empty() { "{}" } else { rest }) {
                Ok(payload) => {
                    let began = Instant::now();
                    bus::call(first, payload, 8000, move |reply| {
                        let ms = began.elapsed().as_millis() as u64;
                        said(&answer, match reply {
                            Ok(reply) => json!({"ms": ms, "reply": reply}),
                            Err(why) => json!({"ms": ms, "error": why}),
                        });
                    });
                }
                Err(e) => said(&answer, json!({"error": e.to_string()})),
            },
            "quit" => {
                self.running = false;
                said(&answer, json!({"quit": true}));
            }
            _ => said(&answer, json!({"error": "where, stats, get, set <name> <value>, reset, key <left|right|up|down|ok|back>..., shot, call <uri> <json>, quit"})),
        }
    }
}

fn differing_pixels(a: *mut SDL_Surface, b: *mut SDL_Surface) -> Option<usize> {
    unsafe {
        if a.is_null() || b.is_null() || (*a).w != (*b).w || (*a).h != (*b).h {
            return None;
        }
        let (w, h) = ((*a).w as usize, (*a).h as usize);
        let mut differing = 0;
        for y in 0..h {
            let row = |s: *mut SDL_Surface| std::slice::from_raw_parts((*s).pixels.cast::<u8>().add(y * (*s).pitch as usize), w * 4);
            differing += row(a).chunks_exact(4).zip(row(b).chunks_exact(4)).filter(|(p, q)| (0..3).any(|c| (p[c] as i32 - q[c] as i32).abs() > SHOT_MAY_DIFFER_BY)).count();
        }
        Some(differing)
    }
}

fn save_shot(app: &mut App, args: &[String]) -> i32 {
    let mut out = None;
    let (mut at_ms, mut against) = (SHOT_SETTLED_MS, None);
    let mut rest = args.iter().skip(2);
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--at" => at_ms = rest.next().and_then(|ms| ms.parse().ok()).unwrap_or(SHOT_SETTLED_MS),
            "--against" => against = rest.next().cloned(),
            name => match key_named(name) {
                Some(key) => {
                    let _ = app.home.key(&app.draw, key, 1000);
                }
                None if out.is_none() => out = Some(name.to_string()),
                None => {}
            },
        }
    }
    let Some(out) = out else {
        eprintln!("usage: lumen --shot <out.png> [--at <ms after the last key>] [--against <kept.png>] [left|right|up|down|back ...]");
        return 2;
    };
    let shot = app.frame_pixels(1000 + at_ms);
    let saved = !shot.is_null() && unsafe { IMG_SavePNG(shot, CString::new(out.as_str()).unwrap_or_default().as_ptr()) } == 0;
    println!("{} {}", if saved { "saved" } else { "could not save" }, out);
    let Some(kept_path) = against else { return if saved { 0 } else { 1 } };
    let kept = unsafe {
        let loaded = IMG_Load(CString::new(kept_path.as_str()).unwrap_or_default().as_ptr());
        if loaded.is_null() { loaded } else { SDL_ConvertSurfaceFormat(loaded, SDL_PIXELFORMAT_RGBA32, 0) }
    };
    match differing_pixels(shot, kept) {
        Some(differing) if differing <= SHOT_PIXELS_THAT_MAY_DIFFER => {
            println!("same as {kept_path} ({differing} pixels differ)");
            0
        }
        Some(differing) => {
            println!("DIFFERENT from {kept_path}: {differing} pixels differ");
            1
        }
        None => {
            println!("DIFFERENT from {kept_path}: it is missing or another size");
            1
        }
    }
}

fn heard_from_the_app_manager(reply: bus::Reply) {
    let Ok(event) = reply else { return note!("app manager: {:?}", reply) };
    match event["event"].as_str() {
        Some("relaunch") => post(move |app| app.relaunched(event["parameters"].clone())),
        Some("close") => {
            note!("close: the app manager asked");
            post(|app| app.running = false);
        }
        other => note!("app manager: {}", other.unwrap_or("?")),
    }
}

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    let began = Instant::now();
    if std::env::var_os("EGL_PLATFORM").is_none() {
        std::env::set_var("EGL_PLATFORM", "wayland");
    }
    if std::env::var_os("XDG_RUNTIME_DIR").is_none() {
        std::env::set_var("XDG_RUNTIME_DIR", "/tmp/xdg");
    }
    util::init();
    let args: Vec<String> = std::env::args().collect();
    let taking_a_shot = args.get(1).is_some_and(|a| a == "--shot");
    note!("start: {}", args.get(1).map_or("no launch argument", String::as_str));
    if let Some(view) = args.get(1).and_then(|launch| launch_view(launch)) {
        note!("start: asked for the {} view, which is not built yet", view);
    }

    let (window, context, app_dir);
    let (mut screen_w, mut screen_h) = (0, 0);
    unsafe {
        lumen_abi_checked();
        SDL_SetHint(c"SDL_WEBOS_ACCESS_POLICY_KEYS_BACK".as_ptr(), c"true".as_ptr());
        if bus::start(APP_ID) {
            bus::subscribe(REGISTER, json!({}), heard_from_the_app_manager);
        }
        if SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS) != 0 {
            note!("SDL_Init: {}", sdl_error());
            return 1;
        }
        IMG_Init(IMG_INIT_JPG | IMG_INIT_PNG);
        let base = SDL_GetBasePath();
        app_dir = if base.is_null() {
            ".".to_string()
        } else {
            let dir = CStr::from_ptr(base).to_string_lossy().into_owned();
            SDL_free(base.cast());
            if dir.len() > 1 { dir.trim_end_matches('/').to_string() } else { dir }
        };
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_ES);
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 2);
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 0);
        SDL_GL_SetAttribute(SDL_GL_RED_SIZE, 8);
        SDL_GL_SetAttribute(SDL_GL_GREEN_SIZE, 8);
        SDL_GL_SetAttribute(SDL_GL_BLUE_SIZE, 8);
        SDL_GL_SetAttribute(SDL_GL_ALPHA_SIZE, 8);
        SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
        SDL_GL_SetAttribute(SDL_GL_DEPTH_SIZE, 0);
        SDL_GL_SetAttribute(SDL_GL_STENCIL_SIZE, 0);
        #[cfg(feature = "webos")]
        {
            let mut display = SDL_DisplayMode { format: 0, w: 0, h: 0, refresh_rate: 0, driverdata: std::ptr::null_mut() };
            if SDL_GetCurrentDisplayMode(0, &mut display) != 0 || display.w == 0 {
                display.w = DESIGN_W as c_int;
                display.h = DESIGN_H as c_int;
            }
            window = SDL_CreateWindow(c"Lumen".as_ptr(), 0, 0, display.w, display.h, SDL_WINDOW_OPENGL | SDL_WINDOW_FULLSCREEN | SDL_WINDOW_SHOWN);
        }
        #[cfg(not(feature = "webos"))]
        {
            window = SDL_CreateWindow(c"Lumen".as_ptr(), 0, 0, DESIGN_W as c_int, DESIGN_H as c_int, SDL_WINDOW_OPENGL | SDL_WINDOW_SHOWN);
        }
        if window.is_null() {
            note!("window: {}", sdl_error());
            return 1;
        }
        #[cfg(feature = "webos")]
        match lumen_claim_window(window) {
            1 => note!("window: no wayland info"),
            2 => note!("window: no wl_webos_shell"),
            _ => {}
        }
        context = SDL_GL_CreateContext(window);
        if context.is_null() {
            note!("context: {}", sdl_error());
            return 1;
        }
        SDL_GL_SetSwapInterval(1);
        SDL_GL_GetDrawableSize(window, &mut screen_w, &mut screen_h);
    }
    let window_at = began.elapsed().as_millis();
    note!("open: {}x{}, {}, {}, {} ms after start", screen_w, screen_h, gl_string(GL_RENDERER), gl_string(GL_VERSION), window_at);

    store::open(&app_dir);
    let mut look = Look::defaults();
    look.apply(&store::read(LOOK_FILE).unwrap_or_default());
    let mut app = App {
        window,
        draw: Draw::new(screen_w, screen_h, &app_dir, look),
        home: Home::new(),
        settings: Settings::defaults(),
        known: Vec::new(),
        launch_points: Value::Null,
        icons_asked_for: HashSet::new(),
        tv_clock_is_24_hour: true,
        fixed_clock: std::env::var("LUMEN_CLOCK").ok().and_then(|clock| clock.split_once('|').map(|(time, date)| (time.to_string(), date.to_string()))),
        fetching_wallpaper: false,
        fetch_wallpapers: !taking_a_shot,
        hidden: false,
        redraw: true,
        running: true,
        began,
        boot: vec![format!("window {window_at}")],
        boot_written: false,
    };
    app.mark("programs");
    app.load_kept();
    app.refresh_from_tv();

    if taking_a_shot {
        for job in INBOX.lock().map(|mut inbox| std::mem::take(&mut *inbox)).unwrap_or_default() {
            job(&mut app);
        }
        return save_shot(&mut app, &args);
    }

    #[cfg(feature = "dev")]
    control::start();
    let per_ms = 1000.0 / unsafe { SDL_GetPerformanceFrequency() } as f64;
    let mut first_frame_shown = false;
    let mut wait_ms = 0;

    while app.running {
        let mut e = SDL_Event { padding: [0; 7] };
        let mut waited = unsafe { if wait_ms > 0 { SDL_WaitEventTimeout(&mut e, wait_ms as c_int) } else { SDL_PollEvent(&mut e) } };
        while waited != 0 {
            match unsafe { e.kind } {
                SDL_QUIT => {
                    note!("close: quit");
                    app.running = false;
                }
                SDL_WINDOWEVENT => app.window_event(unsafe { e.window.event }),
                SDL_KEYDOWN => {
                    let (sym, scancode, repeat) = unsafe { (e.key.keysym.sym, e.key.keysym.scancode, e.key.repeat != 0) };
                    match key_of(sym, scancode) {
                        Some(Key::Ok | Key::Back) if repeat => {}
                        Some(key) => app.key(key),
                        None => note!("key {}, scancode {}", sym, scancode),
                    }
                }
                _ => {}
            }
            waited = unsafe { SDL_PollEvent(&mut e) };
        }
        for job in INBOX.lock().map(|mut inbox| std::mem::take(&mut *inbox)).unwrap_or_default() {
            job(&mut app);
        }
        if !app.running {
            break;
        }

        let until_next_minute = app.tick_clock();
        if !app.boot_written && first_frame_shown && app.began.elapsed().as_millis() >= BOOT_WINDOW_MS {
            app.write_boot_line();
        }
        let now = ticks();
        let moving = app.home.moving(now);
        if !moving && app.home.move_ended(now) {
            app.redraw = true;
        }
        if app.hidden || (!moving && !app.redraw) {
            wait_ms = if app.boot_written { until_next_minute } else { until_next_minute.min(1000) };
            continue;
        }
        wait_ms = 0;
        app.redraw = false;

        let draw_from = unsafe { SDL_GetPerformanceCounter() };
        app.home.draw(&mut app.draw, now);
        if app.draw.look.finish != 0.0 {
            unsafe { glFinish() }
        }
        let draw_ms = (unsafe { SDL_GetPerformanceCounter() } - draw_from) as f64 * per_ms;
        unsafe { SDL_GL_SwapWindow(app.window) }
        app.home.frame_shown(ticks(), draw_ms);
        if !first_frame_shown {
            first_frame_shown = true;
            app.mark("first-frame");
            note!("first frame shown {} ms after start, RSS {} kB", app.began.elapsed().as_millis(), status_kb("VmRSS"));
        }
    }

    note!("RSS {} kB, peak {} kB", status_kb("VmRSS"), status_kb("VmHWM"));
    unsafe {
        SDL_GL_DeleteContext(context);
        SDL_DestroyWindow(window);
        IMG_Quit();
        SDL_Quit();
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_view_asked_for_at_launch_is_read_from_the_app_managers_argument() {
        assert_eq!(launch_view(r#"{"event":"launch","appId":"com.samrenfrew.lumen.native","parameters":{"view":"ambient"},"interfaceVersion":2}"#).as_deref(), Some("ambient"));
        assert_eq!(launch_view(r#"{"event":"launch","parameters":{}}"#), None);
        assert_eq!(launch_view("--shot"), None);
    }

    #[test]
    fn the_remotes_back_key_is_known_by_its_scancode_alone() {
        assert_eq!(key_of(0, WEBOS_BACK_SCANCODE), Some(Key::Back));
        assert_eq!(key_of(SDLK_RETURN, 0), Some(Key::Ok));
        assert_eq!(key_of(0, 0), None);
    }
}
