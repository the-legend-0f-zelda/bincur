use std::{cell::{RefCell}, path::PathBuf};
use evdev::Device;

thread_local! {
    pub static KEYBOARDS:RefCell<Vec<(PathBuf, Device)>> = RefCell::new(Vec::new());
}

pub(crate) fn scan() {
    for (path, dev) in evdev::enumerate() {
        if dev.supported_keys().map_or(false,
            |k| k.contains(evdev::KeyCode::KEY_A)
            && k.contains(evdev::KeyCode::KEY_ENTER)
            && k.contains(evdev::KeyCode::KEY_SPACE)
        ){ KEYBOARDS.with_borrow_mut(|v| v.push((path, dev))); }
    }
}
