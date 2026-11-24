pub type Curve = [f32; 4];
pub static EASE: Curve = [0.25, 0.1, 0.25, 1.0];
pub static EASE_UP: Curve = [0.65, 0.0, 0.25, 1.0];
pub static EASE_DOWN: Curve = [0.25, 0.1, 0.25, 1.0];
pub static EASE_VEIL: Curve = [0.45, 0.0, 0.55, 1.0];
pub static EASE_OUT: Curve = [0.0, 0.0, 0.58, 1.0];
pub static EASE_SCROLL: Curve = [0.2, 0.7, 0.2, 1.0];

#[derive(Clone, Copy)]
pub struct Anim {
    from: f32,
    to: f32,
    start: u32,
    dur: u32,
    ease: &'static Curve,
}

fn bezier(a: f32, b: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
}

pub fn eased(e: &Curve, x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let (mut lo, mut hi, mut t) = (0.0f32, 1.0f32, x);
    for _ in 0..20 {
        if bezier(e[0], e[2], t) < x {
            lo = t;
        } else {
            hi = t;
        }
        t = (lo + hi) / 2.0;
    }
    bezier(e[1], e[3], t)
}

impl Anim {
    pub fn still(value: f32) -> Anim {
        Anim { from: value, to: value, start: 0, dur: 0, ease: &EASE }
    }

    fn since(&self, now: u32) -> i32 {
        now.wrapping_sub(self.start) as i32
    }

    pub fn at(&self, now: u32) -> f32 {
        let since = self.since(now);
        if self.dur == 0 || since >= self.dur as i32 {
            return self.to;
        }
        if since <= 0 {
            return self.from;
        }
        self.from + (self.to - self.from) * eased(self.ease, since as f32 / self.dur as f32)
    }

    pub fn busy(&self, now: u32) -> bool {
        self.dur != 0 && self.since(now) < self.dur as i32
    }

    pub fn go(&mut self, to: f32, ms: f32, ease: &'static Curve, now: u32) {
        self.go_after(0.0, to, ms, ease, now);
    }

    pub fn go_after(&mut self, wait_ms: f32, to: f32, ms: f32, ease: &'static Curve, now: u32) {
        if self.to == to {
            return;
        }
        self.from = self.at(now);
        self.to = to;
        self.start = now.wrapping_add(wait_ms as u32);
        self.dur = (ms as u32).max(1);
        self.ease = ease;
    }

    pub fn set(&mut self, value: f32) {
        self.from = value;
        self.to = value;
        self.dur = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_curve_starts_at_zero_ends_at_one_and_never_goes_back() {
        for curve in [&EASE_UP, &EASE_DOWN, &EASE_VEIL, &EASE_OUT, &EASE_SCROLL, &EASE] {
            assert_eq!(eased(curve, 0.0), 0.0);
            assert_eq!(eased(curve, 1.0), 1.0);
            let mut last = 0.0;
            for step in 1..=100 {
                let at = eased(curve, step as f32 / 100.0);
                assert!(at >= last - 1e-4, "{curve:?} goes back at {step}");
                last = at;
            }
        }
    }

    #[test]
    fn an_animation_turned_round_mid_way_carries_on_from_where_it_was() {
        let mut slide = Anim::still(0.0);
        slide.go(1.0, 600.0, &EASE_DOWN, 1000);
        let mid_way = slide.at(1300);
        assert!(mid_way > 0.0 && mid_way < 1.0);
        slide.go(0.0, 600.0, &EASE_UP, 1300);
        assert_eq!(slide.at(1300), mid_way);
        assert!(slide.busy(1899));
        assert!(!slide.busy(1900));
        assert_eq!(slide.at(1900), 0.0);
    }

    #[test]
    fn a_delayed_animation_holds_its_value_and_counts_as_moving_until_it_starts() {
        let mut fade = Anim::still(1.0);
        fade.go_after(150.0, 0.0, 220.0, &EASE, 1000);
        assert_eq!(fade.at(1100), 1.0);
        assert!(fade.busy(1100));
        assert!(fade.at(1260) < 1.0);
        assert_eq!(fade.at(1370), 0.0);
        assert!(!fade.busy(1370));
    }

    #[test]
    fn an_animation_still_works_when_the_millisecond_clock_wraps_round() {
        let mut slide = Anim::still(0.0);
        slide.go(1.0, 600.0, &EASE, u32::MAX - 100);
        assert!(slide.busy(200));
        assert_eq!(slide.at(500), 1.0);
    }
}
