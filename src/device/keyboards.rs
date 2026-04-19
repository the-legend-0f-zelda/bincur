use std::{cell::{RefCell}, path::PathBuf};
use evdev::Device;

thread_local! {
    pub static KEYBOARDS:RefCell<Vec<(PathBuf, Device)>> = RefCell::new(Vec::new());

    /// Pressed state indexed by evdev scancode (KEY_RESERVED=0 .. KEY_MICMUTE=248).
    pub static PRESS_STATE:RefCell<[bool; 249]> = RefCell::new([false; 249]);
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
