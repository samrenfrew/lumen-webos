use crate::draw::{decoded, Decoded};
use crate::{store, wallpaper};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

const WIDTH: u32 = 360;
const WAITING_MOST: usize = 40;

#[derive(Default)]
struct Inner {
    wanted: Vec<(String, String)>,
    ready: Vec<(String, Decoded)>,
    asked: HashSet<String>,
    working: bool,
}

#[derive(Clone, Default)]
pub struct Thumbs {
    inner: Arc<Mutex<Inner>>,
}

fn file_of(id: &str) -> Option<String> {
    let plain = !id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
    plain.then(|| format!("thumbs/{id}.jpg"))
}

impl Thumbs {
    pub fn want(&self, id: &str, url: &str, online: bool) {
        let Ok(mut inner) = self.inner.lock() else { return };
        if !inner.asked.insert(id.to_string()) {
            return;
        }
        let kept_on_disk = file_of(id).is_some_and(|f| store::has(&f));
        if !online && !kept_on_disk {
            return;
        }
        inner.wanted.push((id.to_string(), url.to_string()));
        if inner.wanted.len() > WAITING_MOST {
            let (dropped, _) = inner.wanted.remove(0);
            inner.asked.remove(&dropped);
        }
        if inner.working {
            return;
        }
        inner.working = true;
        let shared = self.inner.clone();
        std::thread::spawn(move || loop {
            let next = shared.lock().ok().and_then(|mut inner| {
                let next = inner.wanted.pop();
                if next.is_none() {
                    inner.working = false;
                }
                next
            });
            let Some((id, url)) = next else { return };
            let Some(file) = file_of(&id) else { continue };
            if !store::has(&file) {
                let Ok(bytes) = crate::util::http_get(&wallpaper::resized(&url, WIDTH)) else { continue };
                let _ = std::fs::create_dir_all(store::path("thumbs"));
                store::write(&file, &bytes);
            }
            if let Some(picture) = decoded(&store::path(&file)) {
                if let Ok(mut inner) = shared.lock() {
                    inner.ready.push((id, picture));
                }
                crate::wake();
            }
        });
    }

    pub fn forget(&self, id: &str) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.asked.remove(id);
        }
    }

    pub fn take_ready(&self) -> Vec<(String, Decoded)> {
        self.inner.lock().map(|mut inner| std::mem::take(&mut inner.ready)).unwrap_or_default()
    }
}
