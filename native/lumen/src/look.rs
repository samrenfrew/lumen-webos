macro_rules! look {
    ($($name:ident = $default:expr, $min:expr, $max:expr;)*) => {
        #[derive(Clone, PartialEq, Debug)]
        pub struct Look { $(pub $name: f32,)* }

        impl Look {
            pub fn defaults() -> Look {
                Look { $($name: $default,)* }
            }

            pub fn names() -> &'static [&'static str] {
                &[$(stringify!($name),)*]
            }

            pub fn get(&self, name: &str) -> Option<f32> {
                match name { $(stringify!($name) => Some(self.$name),)* _ => None }
            }

            pub fn set(&mut self, name: &str, value: f32) -> bool {
                match name {
                    $(stringify!($name) => {
                        let fits = value >= $min && value <= $max;
                        if fits { self.$name = value; }
                        fits
                    })*
                    _ => false,
                }
            }
        }
    };
}

look! {
    down_ms = 600.0, 1.0, 5000.0;
    up_ms = 600.0, 1.0, 5000.0;
    down_x1 = 0.65, 0.0, 1.0;
    down_y1 = 0.0, -2.0, 3.0;
    down_x2 = 0.25, 0.0, 1.0;
    down_y2 = 1.0, -2.0, 3.0;
    up_x1 = 0.65, 0.0, 1.0;
    up_y1 = 0.0, -2.0, 3.0;
    up_x2 = 0.25, 0.0, 1.0;
    up_y2 = 1.0, -2.0, 3.0;
    veil_down_ms = 600.0, 1.0, 5000.0;
    veil_up_ms = 800.0, 1.0, 5000.0;
    focus_ms = 160.0, 1.0, 2000.0;
    scroll_ms = 260.0, 1.0, 2000.0;
    fade_ms = 220.0, 1.0, 2000.0;
    hide_top_wait_ms = 150.0, 0.0, 2000.0;
    show_top_wait_ms = 200.0, 0.0, 2000.0;
    glass_halvings = 0.0, 0.0, 4.0;
    glass_passes = 1.0, 1.0, 8.0;
    veil_halvings = 2.0, 0.0, 4.0;
    veil_passes = 3.0, 1.0, 8.0;
    bezel = 28.0, 1.0, 110.0;
    bend = 0.0, -200.0, 200.0;
    fringe = 0.0, 0.0, 1.0;
    saturate = 1.5, 0.0, 3.0;
    brighten = 1.1, 0.0, 3.0;
    tint = 0.3, 0.0, 1.0;
    finish = 0.0, 0.0, 1.0;
    glass_shadow = 0.1, 0.0, 1.0;
}

impl Look {
    pub fn apply(&mut self, text: &str) {
        for line in text.lines() {
            if let Some((name, value)) = line.split_once('=') {
                if let Ok(value) = value.trim().parse() {
                    self.set(name.trim(), value);
                }
            }
        }
    }

    pub fn changed_from_defaults(&self) -> String {
        let defaults = Look::defaults();
        Look::names().iter().filter(|name| self.get(name) != defaults.get(name)).map(|name| format!("{}={}\n", name, self.get(name).unwrap_or(0.0))).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_in_range_is_taken_and_one_out_of_range_or_unknown_is_not() {
        let mut look = Look::defaults();
        assert!(look.set("down_ms", 450.0));
        assert!(!look.set("up_ms", 99999.0));
        assert!(!look.set("nonsense", 3.0));
        assert_eq!(look.down_ms, 450.0);
        assert_eq!(look.up_ms, Look::defaults().up_ms);
    }

    #[test]
    fn the_values_changed_live_come_back_after_a_restart() {
        let mut tried = Look::defaults();
        tried.set("down_ms", 450.0);
        tried.set("bend", -20.0);
        let kept = tried.changed_from_defaults();
        assert_eq!(kept, "down_ms=450\nbend=-20\n");
        let mut restarted = Look::defaults();
        restarted.apply(&kept);
        assert_eq!(restarted, tried);
    }

    #[test]
    fn a_kept_file_with_junk_in_it_changes_only_what_it_can() {
        let mut look = Look::defaults();
        look.apply("down_ms=450\r\nup_ms=fast\nnonsense=3\n\nfocus_ms = 200\n");
        assert_eq!((look.down_ms, look.up_ms, look.focus_ms), (450.0, 600.0, 200.0));
    }
}
