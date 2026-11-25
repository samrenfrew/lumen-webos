use crate::home::{LiveShown, GUIDE_TILE};
use crate::livetv::{self, Channel, Listings, LiveCache, Steer};
use crate::settings::Settings;
use crate::{note, post};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ScheduleState {
    Loading,
    Ready,
    NowNextOnly,
}

pub struct Live {
    pub channels: Vec<Channel>,
    now_next: Listings,
    schedule: Listings,
    pub listings: Listings,
    pub logos: HashMap<String, String>,
    pub pending: HashSet<String>,
    pub state: ScheduleState,
    asked_at: HashMap<String, u64>,
    read_at: u64,
    busy: bool,
    schedule_busy: bool,
    failed_at: u64,
    last_line: String,
    started: bool,
    ticked_at: Option<Instant>,
    shown_minute: u64,
    pub focus: Arc<AtomicUsize>,
    pub guide_open: Arc<AtomicBool>,
}

impl Live {
    pub fn new() -> Live {
        Live {
            channels: Vec::new(),
            now_next: Listings::new(),
            schedule: Listings::new(),
            listings: Listings::new(),
            logos: HashMap::new(),
            pending: HashSet::new(),
            state: ScheduleState::Loading,
            asked_at: HashMap::new(),
            read_at: 0,
            busy: false,
            schedule_busy: false,
            failed_at: 0,
            last_line: String::new(),
            started: false,
            ticked_at: None,
            shown_minute: 0,
            focus: Arc::new(AtomicUsize::new(0)),
            guide_open: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn load_kept(&mut self) {
        let kept = livetv::kept();
        self.logos = livetv::kept_logos(&kept.channels);
        self.channels = kept.channels;
        self.now_next = kept.listings;
        self.schedule = kept.schedule;
        self.merge();
    }

    fn merge(&mut self) {
        self.listings = livetv::merge_listings(&self.schedule, &self.now_next);
    }

    fn save(&self, settings: &Settings, t: u64) {
        let keep = livetv::first_read(&self.channels, settings);
        livetv::keep(&LiveCache { channels: self.channels.clone(), listings: self.now_next.clone(), schedule: livetv::for_cache(&self.schedule, &keep, t) });
    }

    pub fn tick(&mut self, settings: &Settings, talk_to_tv: bool, hidden: bool, since_first_frame_ms: Option<u128>, t: u64) {
        if !settings.live_tv_row || !talk_to_tv || hidden {
            return;
        }
        if !self.started {
            if since_first_frame_ms.is_some_and(|ms| ms >= livetv::READ_AFTER_MS as u128) {
                self.started = true;
                self.ticked_at = Some(Instant::now());
                self.refresh(true, t);
            }
            return;
        }
        if self.ticked_at.is_some_and(|at| at.elapsed().as_millis() < livetv::TICK_MS as u128) {
            return;
        }
        self.ticked_at = Some(Instant::now());
        let row = livetv::row(&self.channels, settings);
        if livetv::ask_now_next_again(&self.channels, &row, &self.now_next, self.read_at, t) {
            self.refresh(self.channels.is_empty(), t);
        } else {
            self.read_guide(settings, t);
        }
    }

    pub fn guide_opened(&mut self, open: bool, settings: &Settings, t: u64) {
        self.guide_open.store(open, Ordering::Relaxed);
        if open && self.started {
            self.refresh(false, t);
            self.read_guide(settings, t);
        }
    }

    fn refresh(&mut self, with_channels: bool, t: u64) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.read_at = t;
        if with_channels || self.channels.is_empty() {
            livetv::read_channels(|got| post(move |app| app.live_channels_read(got)));
        } else {
            let list = self.channels.clone();
            livetv::read_now_next(move |got| post(move |app| app.live_now_next_read(list, got)));
        }
    }

    pub fn channels_read(&mut self, got: Result<Vec<Channel>, String>) {
        match got {
            Ok(list) => livetv::read_now_next(move |got| post(move |app| app.live_now_next_read(list, got))),
            Err(why) => {
                note!("guide: channels failed: {}", why);
                self.busy = false;
            }
        }
    }

    pub fn now_next_read(&mut self, list: Vec<Channel>, got: Result<Listings, String>, settings: &Settings, t: u64) -> bool {
        self.busy = false;
        let read = match got {
            Ok(read) => read,
            Err(why) => {
                note!("guide: now and next failed: {}", why);
                return false;
            }
        };
        let line = format!("now and next: {} channels listed, {} with something on now", read.len(), livetv::on_now(&read, t));
        if line != self.last_line {
            note!("guide: {}", line);
            self.last_line = line;
        }
        let listings = livetv::keep_unfinished(read, &self.now_next, t);
        let changed = list != self.channels || listings != self.now_next;
        if changed {
            self.channels = list.clone();
            self.now_next = listings;
            self.merge();
            self.save(settings, t);
        }
        self.read_guide(settings, t);
        livetv::read_logos(&list, |logos| post(move |app| app.live_logos_read(logos)));
        changed
    }

    pub fn logos_read(&mut self, logos: HashMap<String, String>) -> bool {
        if logos == self.logos {
            return false;
        }
        self.logos = logos;
        true
    }

    fn read_guide(&mut self, settings: &Settings, t: u64) {
        let all = self.guide_open.load(Ordering::Relaxed);
        if self.channels.is_empty() || (!all && t.saturating_sub(self.failed_at) < livetv::FAILED_WAIT_S) {
            return;
        }
        let list = if all { self.channels.clone() } else { livetv::first_read(&self.channels, settings) };
        let fresh = livetv::fresh_reads(&list, &self.asked_at, t);
        if fresh.iter().all(|f| *f) {
            return;
        }
        let waiting: Vec<String> = list.iter().zip(&fresh).filter(|(_, f)| !**f).map(|(c, _)| c.signal.clone()).collect();
        if self.schedule_busy {
            if all {
                self.pending.extend(waiting);
            }
            return;
        }
        self.schedule_busy = true;
        self.pending = waiting.into_iter().collect();
        self.state = ScheduleState::Loading;
        let window = livetv::guide_window(t);
        let (focus, open) = (self.focus.clone(), self.guide_open.clone());
        let steer = Steer {
            focus: Box::new(move || if all { focus.load(Ordering::Relaxed) } else { 0 }),
            stopped: Box::new(move || all && !open.load(Ordering::Relaxed)),
            on_part: Box::new(|part: &Listings, batch: &[Channel]| {
                let (part, batch) = (part.clone(), batch.to_vec());
                post(move |app| app.live_part_read(part, batch));
            }),
        };
        let open = self.guide_open.clone();
        livetv::read_events_near(list, window, fresh, steer, move |got| {
            post(move |app| match got {
                Ok(_) => app.live_schedule_done(ScheduleState::Ready, all),
                Err(_) => {
                    app.live.failed_at = crate::clock::unix_now();
                    if !all {
                        return app.live_schedule_done(ScheduleState::Ready, all);
                    }
                    livetv::read_schedule(
                        window,
                        move || !open.load(Ordering::Relaxed),
                        |part| {
                            let part = part.clone();
                            post(move |app| app.live_part_read(part, Vec::new()));
                        },
                        move |got| post(move |app| app.live_schedule_done(if got.is_ok() { ScheduleState::Ready } else { ScheduleState::NowNextOnly }, all)),
                    );
                }
            })
        });
    }

    pub fn part_read(&mut self, part: Listings, batch: Vec<Channel>) {
        let at = crate::clock::unix_now();
        for c in &batch {
            self.asked_at.insert(c.signal.clone(), at);
            self.pending.remove(&c.signal);
        }
        self.schedule.extend(part);
        self.merge();
    }

    pub fn schedule_done(&mut self, state: ScheduleState, all: bool, settings: &Settings, t: u64) {
        self.schedule_busy = false;
        self.pending.clear();
        self.state = state;
        self.save(settings, t);
        if self.guide_open.load(Ordering::Relaxed) && !all {
            self.read_guide(settings, t);
        }
    }
}

impl crate::App {
    pub fn unix_minute(&self) -> u64 {
        self.home.unix_now
    }

    pub fn show_live(&mut self) {
        let t = self.unix_minute();
        let mut shown = Vec::new();
        if self.settings.live_tv_row && !self.live.channels.is_empty() {
            shown.push(LiveShown { id: GUIDE_TILE.to_string(), number: String::new(), name: "Guide".to_string(), logo: None, now: None, next: None, loading: false });
            for c in livetv::row(&self.live.channels, &self.settings) {
                let on = livetv::now_next_at(self.live.listings.get(&c.signal).map(Vec::as_slice), t);
                let logo = c.logo_path.as_ref().and_then(|p| self.live.logos.get(p)).cloned();
                shown.push(LiveShown { now: on.now.cloned(), next: on.next.cloned(), loading: self.live.pending.contains(&c.signal), logo, id: c.id, number: c.number, name: c.name });
            }
        }
        self.live.shown_minute = t / 60;
        self.home.set_live(&mut self.draw, shown, crate::ticks());
        self.redraw = true;
    }

    pub fn live_tick(&mut self) {
        let since_first_frame = self.shown_once.then(|| self.began.elapsed().as_millis());
        self.live.tick(&self.settings, self.fetch_wallpapers, self.hidden, since_first_frame, crate::clock::unix_now());
        if self.unix_minute() / 60 != self.live.shown_minute {
            self.show_live();
        }
    }

    pub fn live_channels_read(&mut self, got: Result<Vec<Channel>, String>) {
        self.live.channels_read(got);
    }

    pub fn live_now_next_read(&mut self, list: Vec<Channel>, got: Result<Listings, String>) {
        if self.live.now_next_read(list, got, &self.settings, crate::clock::unix_now()) {
            self.show_live();
        }
    }

    pub fn live_logos_read(&mut self, logos: HashMap<String, String>) {
        if self.live.logos_read(logos) {
            self.show_live();
        }
    }

    pub fn live_part_read(&mut self, part: Listings, batch: Vec<Channel>) {
        self.live.part_read(part, batch);
        self.show_live();
    }

    pub fn live_schedule_done(&mut self, state: ScheduleState, all: bool) {
        self.live.schedule_done(state, all, &self.settings, crate::clock::unix_now());
        self.show_live();
    }

    pub fn watch(&mut self, channel_id: String) {
        note!("watch {}", channel_id);
        let name = self.live.channels.iter().find(|c| c.id == channel_id).map_or_else(|| "the channel".to_string(), |c| c.name.clone());
        let asked = channel_id.clone();
        livetv::open_channel(&asked, move |opened| {
            if let Err(why) = opened {
                note!("watch {}: {}", channel_id, why);
                post(move |app| app.home.toast(&format!("Couldn't open {name}"), true, crate::ticks()));
            }
        });
    }
}
