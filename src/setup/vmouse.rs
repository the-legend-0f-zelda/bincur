use std::{fs::File, io::{BufRead, BufReader}, sync::OnceLock};
use evdev::{AttributeSet, KeyCode, RelativeAxisCode};
use crate::setup::config;

pub static VMOUSE_KEYS: OnceLock<AttributeSet<KeyCode>> = OnceLock::new();
pub static VMOUSE_REL_AXES: OnceLock<AttributeSet<RelativeAxisCode>> = OnceLock::new();
pub(crate) static VMOUSE_CFG_DEFAULT: OnceLock<Config> = OnceLock::new();

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
        axes
    })
}

pub(crate) fn load_default() -> &'static Config {
    VMOUSE_CFG_DEFAULT.get_or_init(|| {
        let conf_file:File = File::open(
            config::resolve_path().join("vmouse.conf")
        ).unwrap();

        let mut cfg = Config::new();

        for line in BufReader::new(conf_file).lines() {
            let line = line.unwrap();
            let cleaned = line.replace(" ", "").to_uppercase();

            if cleaned.is_empty() || cleaned.starts_with("#") {
                continue;
            }

            let kv:Vec<&str> = cleaned.split(':').collect();
            let [k, v] = kv.as_slice() else {
                  eprintln!("invalid vmouse config: {}", cleaned);
                  std::process::exit(1);
            };

            match *k {
                "STEP_SIZE" => cfg.step_size = v.parse().unwrap(),
                _ => continue
            }
        }

        cfg
    })
}

#[derive(Clone, Copy)]
pub(crate) struct Config {
    pub(crate) step_size: u16,
    pub(crate) mode: u8
}

impl Config {
    pub(crate) fn new() -> Self {
        Self {step_size:0, mode:0}
    }
}
