use std::sync::OnceLock;
use evdev::{AttributeSet, KeyCode, RelativeAxisCode};

use crate::config::cleaned_lines;

pub static VMOUSE_KEYS: OnceLock<AttributeSet<KeyCode>> = OnceLock::new();
pub static VMOUSE_REL_AXES: OnceLock<AttributeSet<RelativeAxisCode>> = OnceLock::new();
static VMOUSE_PROPS_DEFAULT: OnceLock<Props> = OnceLock::new();

pub fn get_keys() -> &'static AttributeSet<KeyCode> {
    VMOUSE_KEYS.get_or_init(|| {
        let mut keys = AttributeSet::<KeyCode>::new();
        keys.insert(KeyCode::BTN_LEFT);
        keys.insert(KeyCode::BTN_RIGHT);
        keys.insert(KeyCode::BTN_MIDDLE);
        keys
    })
}

pub fn get_rel_axes() -> &'static AttributeSet<RelativeAxisCode> {
    VMOUSE_REL_AXES.get_or_init(|| {
        let mut axes = AttributeSet::<RelativeAxisCode>::new();
        axes.insert(RelativeAxisCode::REL_X);
        axes.insert(RelativeAxisCode::REL_Y);
        axes.insert(RelativeAxisCode::REL_WHEEL);
        axes.insert(RelativeAxisCode::REL_HWHEEL);
        axes
    })
}

pub fn load_default_props() -> &'static Props {
    VMOUSE_PROPS_DEFAULT.get_or_init(|| {
        let mut cfg = Props::new();

        for line in cleaned_lines("vmouse.conf", None) {
            let kv:Vec<&str> = line.split(':').collect();
            let [k, v] = kv.as_slice() else {
                  eprintln!("invalid vmouse config: {}", line);
                  std::process::exit(1);
            };

            match *k {
                "GRAB_LINEAR" => cfg.grab_linear = v.to_uppercase().eq("TRUE"),
                "GRAB_LOGARITHMIC" => cfg.grab_logarithmic = v.to_uppercase().eq("TRUE"),
                "GRAB_SCROLL" => cfg.grab_scroll = v.to_uppercase().eq("TRUE"),
                "STEP_SIZE_X" => cfg.step_size_x = v.parse().unwrap(),
                "STEP_SIZE_Y" => cfg.step_size_y = v.parse().unwrap(),
                "SCROLL_DIST_X" => cfg.scroll_dist_x = v.parse().unwrap(),
                "SCROLL_DIST_Y" => cfg.scroll_dist_y = v.parse().unwrap(),
                _ => continue
            }
        }

        cfg
    })
}

#[derive(Clone, Copy)]
pub struct Props {
    pub mode: u8,
    pub grab_linear: bool,
    pub grab_logarithmic: bool,
    pub grab_scroll: bool,
    pub step_size_x: i32,
    pub step_size_y: i32,
    pub scroll_dist_x: i32,
    pub scroll_dist_y: i32,
}

impl Props {
    pub fn new() -> Self {
        Self {
            mode: 0,
            grab_linear: false,
            grab_logarithmic: false,
            grab_scroll: false,
            step_size_x: 0,
            step_size_y: 0,
            scroll_dist_x: 0,
            scroll_dist_y: 0
        }
    }

    pub fn reset_xy(&mut self) {
        let default = load_default_props();
        self.step_size_x = default.step_size_x;
        self.step_size_y = default.step_size_y;
        self.scroll_dist_x = default.scroll_dist_x;
        self.scroll_dist_y = default.scroll_dist_y;
    }
}
