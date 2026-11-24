use serde_json::Value;

pub type Reply = Result<Value, String>;

fn answered(text: &str) -> Reply {
    let reply: Value = serde_json::from_str(text).map_err(|e| format!("not JSON: {e}"))?;
    if reply["returnValue"].as_bool() == Some(false) {
        return Err(reply["errorText"].as_str().unwrap_or("the call failed").to_string());
    }
    Ok(reply)
}

#[cfg(feature = "webos")]
mod tv {
    use super::{answered, Reply};
    use crate::ffi::*;
    use crate::note;
    use serde_json::Value;
    use std::ffi::{c_char, c_void, CStr, CString};
    use std::sync::mpsc::{channel, Sender};
    use std::sync::{Mutex, OnceLock};

    type Job = Box<dyn FnOnce() + Send>;
    type Once = Box<dyn FnOnce(Reply) + Send>;
    type Each = Box<dyn FnMut(Reply) + Send>;
    static JOBS: OnceLock<Mutex<Sender<Job>>> = OnceLock::new();

    fn taken(text: *mut c_char) -> Option<String> {
        if text.is_null() {
            return None;
        }
        let owned = unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned();
        unsafe { lumen_luna_free(text) }
        Some(owned)
    }

    fn heard(reply: *const c_char) -> Reply {
        if reply.is_null() {
            return Err("no reply".into());
        }
        answered(&unsafe { CStr::from_ptr(reply) }.to_string_lossy())
    }

    extern "C" fn heard_once(reply: *const c_char, user: *mut c_void) {
        let then = unsafe { Box::from_raw(user.cast::<Once>()) };
        then(heard(reply));
    }

    extern "C" fn heard_each(reply: *const c_char, user: *mut c_void) {
        let each = unsafe { &mut *user.cast::<Each>() };
        each(heard(reply));
    }

    fn on_bus_thread(job: Job) {
        let Some(jobs) = JOBS.get() else { return };
        if jobs.lock().map(|sender| sender.send(job).is_ok()).unwrap_or(false) {
            unsafe { lumen_luna_wake() }
        }
    }

    pub fn start(app_id: &str) -> bool {
        let (jobs, waiting) = channel::<Job>();
        let (opened, open) = channel::<Option<String>>();
        let name = CString::new(app_id).unwrap_or_default();
        std::thread::spawn(move || {
            let failed = taken(unsafe { lumen_luna_open(name.as_ptr()) });
            let usable = failed.is_none();
            let _ = opened.send(failed);
            while usable {
                while let Ok(job) = waiting.try_recv() {
                    job();
                }
                unsafe { lumen_luna_wait() }
            }
        });
        let failed = open.recv().unwrap_or_else(|_| Some("the bus thread stopped".into()));
        if let Some(why) = &failed {
            note!("bus: not open: {}", why);
            return false;
        }
        JOBS.set(Mutex::new(jobs)).is_ok()
    }

    pub fn call(uri: &str, payload: Value, timeout_ms: i32, then: impl FnOnce(Reply) + Send + 'static) {
        let (uri, payload) = (CString::new(uri).unwrap_or_default(), CString::new(payload.to_string()).unwrap_or_default());
        if JOBS.get().is_none() {
            return then(Err("the bus is not open".into()));
        }
        on_bus_thread(Box::new(move || {
            let then: *mut Once = Box::into_raw(Box::new(Box::new(then)));
            if let Some(why) = taken(unsafe { lumen_luna_send(uri.as_ptr(), payload.as_ptr(), 0, timeout_ms, heard_once, then.cast()) }) {
                (unsafe { Box::from_raw(then) })(Err(why));
            }
        }));
    }

    pub fn subscribe(uri: &str, payload: Value, each: impl FnMut(Reply) + Send + 'static) {
        let (uri, payload) = (CString::new(uri).unwrap_or_default(), CString::new(payload.to_string()).unwrap_or_default());
        on_bus_thread(Box::new(move || {
            let each: *mut Each = Box::into_raw(Box::new(Box::new(each)));
            if let Some(why) = taken(unsafe { lumen_luna_send(uri.as_ptr(), payload.as_ptr(), 1, 0, heard_each, each.cast()) }) {
                (unsafe { &mut *each })(Err(why));
            }
        }));
    }
}

#[cfg(not(feature = "webos"))]
mod tv {
    use super::{answered, Reply};
    use serde_json::Value;

    pub fn start(_app_id: &str) -> bool {
        true
    }

    pub fn call(uri: &str, _payload: Value, _timeout_ms: i32, then: impl FnOnce(Reply) + Send + 'static) {
        let method = uri.rsplit('/').next().unwrap_or("");
        let stood_in = std::env::var("LUMEN_FIXTURES").ok().and_then(|dir| std::fs::read_to_string(format!("{dir}/{method}.json")).ok());
        then(match stood_in {
            Some(text) => answered(&text),
            None => Err("not on a TV".into()),
        });
    }

    pub fn subscribe(_uri: &str, _payload: Value, _each: impl FnMut(Reply) + Send + 'static) {}
}

pub use tv::{call, start, subscribe};

#[cfg(test)]
mod tests {
    use super::answered;

    #[test]
    fn a_reply_that_says_it_failed_is_an_error_with_the_tvs_own_words() {
        assert_eq!(answered(r#"{"returnValue":false,"errorText":"Not permitted"}"#), Err("Not permitted".to_string()));
        assert_eq!(answered(r#"{"returnValue":false}"#), Err("the call failed".to_string()));
    }

    #[test]
    fn a_reply_with_no_verdict_counts_as_an_answer() {
        assert!(answered(r#"{"event":"relaunch","parameters":{"view":"ambient"}}"#).is_ok());
        assert!(answered("half a rep").is_err());
    }
}
