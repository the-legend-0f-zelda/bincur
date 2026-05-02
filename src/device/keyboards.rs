use std::{cell::RefCell, path::PathBuf};
use evdev::{AttributeSet, Device, InputEvent, KeyCode, uinput::VirtualDevice};

thread_local! {
    pub(crate) static KEYBOARDS:RefCell<Vec<(PathBuf, Device)>> = RefCell::new(Vec::new());

    /// Pressed state indexed by evdev scancode (KEY_RESERVED=0 .. KEY_MICMUTE=248).
    pub(crate) static PRESS_STATE:RefCell<[bool; 249]> = RefCell::new([false; 249]);

    /// Virtual device for forwarding unbound key events.
    /// evdev 0.13.2: highest defined key code is 0x2e7 (BTN_TRIGGER_HAPPY40)
    pub(crate) static VKEYBOARD_PASSTHROUGH:RefCell<VirtualDevice> = RefCell::new(
        VirtualDevice::builder().unwrap()
            .name("bincur-vkeyboard")
            .with_keys(&{
                let mut keys = AttributeSet::new();
                for code in 1..0x2e7 {
                    keys.insert(KeyCode::new(code));
                }
                keys
            }).unwrap()
            .build().unwrap()
    );
}


pub(crate) fn scan() {
    for (path, dev) in evdev::enumerate() {
        if dev.supported_keys().map_or(false,
            |k| k.contains(evdev::KeyCode::KEY_A)
            && k.contains(evdev::KeyCode::KEY_ENTER)
            && k.contains(evdev::KeyCode::KEY_SPACE)
        )
        { KEYBOARDS.with_borrow_mut(|v| v.push((path, dev))); }
    }
}


pub(crate) fn pass_through(event: InputEvent) {
    VKEYBOARD_PASSTHROUGH.with_borrow_mut(|vkeyboard| {
        vkeyboard.emit(&[event]).unwrap();
    });
}
