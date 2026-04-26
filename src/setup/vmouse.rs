use std::sync::OnceLock;

use evdev::{AttributeSet, KeyCode, RelativeAxisCode};


pub static VMOUSE_KEYS: OnceLock<AttributeSet<KeyCode>> = OnceLock::new();
pub static VMOUSE_REL_AXES: OnceLock<AttributeSet<RelativeAxisCode>> = OnceLock::new();

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
