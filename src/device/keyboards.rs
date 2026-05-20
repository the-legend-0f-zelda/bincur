use std::{cell::RefCell, path::PathBuf};
use evdev::{AttributeSet, Device, InputEvent, KeyCode, uinput::VirtualDevice};

use crate::config::KEYCODE_MAX;


thread_local! {
    pub static KEYBOARDS:RefCell<Vec<(PathBuf, Device)>> = RefCell::new(Vec::new());

    /// Pressed state indexed by evdev scancode
    pub static PRESS_STATE:RefCell<[(bool, bool); KEYCODE_MAX+1]> = RefCell::new([(false, false); KEYCODE_MAX+1]);

    /// Virtual device for forwarding unbound key events.
    pub static VKEYBOARD_PASSTHROUGH:RefCell<VirtualDevice> = RefCell::new(
        VirtualDevice::builder().unwrap()
            .name("bincur-vkeyboard")
            .with_keys(&{
                let mut keys = AttributeSet::new();
                for code in 0..KEYCODE_MAX as u16 {
                    keys.insert(KeyCode::new(code));
                }
                keys
            }).unwrap()
            .build().unwrap()
    );
}

pub fn all_pressed(combo: &[usize]) -> bool {
    PRESS_STATE.with_borrow(|ps| {
        combo.iter()
            .all(|&key| ps.get(key).unwrap_or(&(false, false)).0)
    })
}

pub fn scan() {
    KEYBOARDS.with_borrow_mut(|v| {v.clear();});

    for (path, dev) in evdev::enumerate() {
        if dev.name().map_or(false, |n| n.starts_with("bincur")) {continue}
        if dev.supported_keys().map_or(false,
            |k| k.contains(evdev::KeyCode::KEY_A)
            && k.contains(evdev::KeyCode::KEY_ENTER)
            && k.contains(evdev::KeyCode::KEY_SPACE)
        )
        { KEYBOARDS.with_borrow_mut(|v| v.push((path, dev))); }
    }
}

pub fn names() -> Vec<Option<String>> {
    let mut names:Vec<Option<String>> = Vec::new();
    KEYBOARDS.with_borrow(|kbds| {
        for (_path, kbd) in kbds {
            let name: Option<String> = match kbd.name() {
                Some(name) => Some(name.to_string()),
                None => None
            };
            names.push(name);
        }
    });
    names
}

pub fn update_press_state(key_code: usize, key_value: i32) {
    PRESS_STATE.with_borrow_mut(|p_state| {
        match p_state.get_mut(key_code) {
            Some(slot) => slot.0 = key_value > 0,
            None => {}
        };
    });
}

pub fn update_grab_state(key_code: usize, key_value: i32, should_grab: &mut bool) {
    PRESS_STATE.with_borrow_mut(|p_state| {
        let Some(slot) = p_state.get_mut(key_code)
        else {return};

        if key_value > 0 {
            slot.1 = *should_grab;
        }else {
            *should_grab = slot.1;
            slot.1 = false;
        }
    });
}

pub fn pass_through(event: InputEvent) {
    VKEYBOARD_PASSTHROUGH.with_borrow_mut(|vkeyboard| {
        vkeyboard.emit(&[event]).unwrap();
    });
}
