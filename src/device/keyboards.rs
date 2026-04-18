use std::{path::PathBuf, sync::Mutex};
use evdev::Device;

pub static KEYBOARDS:Mutex<Vec<(PathBuf, Device)>> = Mutex::new(Vec::new());

pub(crate) fn scan() {
    let mut guard = KEYBOARDS.lock().unwrap();
    guard.clear();

    for (path, dev) in evdev::enumerate() {
        if dev.supported_keys().map_or(false,
            |k| k.contains(evdev::KeyCode::KEY_A)
            && k.contains(evdev::KeyCode::KEY_ENTER)
            && k.contains(evdev::KeyCode::KEY_SPACE)
        ){ guard.push((path, dev)); }
    }
}
