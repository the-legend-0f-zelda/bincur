use std::{cell::RefCell, path::PathBuf};
use evdev::{AttributeSet, Device, InputEvent, KeyCode, uinput::VirtualDevice};

use crate::config::KEYCODE_MAX;


thread_local! {
    pub static KEYBOARDS:RefCell<Vec<(PathBuf, Device)>> = RefCell::new(Vec::new());

    /// Press state of physical keys, before rewire. Indexed by evdev scancode.
    /// [keydown?, rewired_to)] : [(bool, usize)]
    static PHYSICAL_PRESS_STATE:RefCell<[(bool, usize); KEYCODE_MAX+1]> = RefCell::new(std::array::from_fn(|i| (false, i)));

    /// Press state of logical keys, after rewire. Indexed by evdev scancode.
    /// [(keydown?, grabbed?)] : [(bool, bool)]
    static LOGICAL_PRESS_STATE:RefCell<[(bool, bool); KEYCODE_MAX+1]> = RefCell::new([(false, false); KEYCODE_MAX+1]);

    /// Virtual device for forwarding unbound key events.
    static VKEYBOARD_PASSTHROUGH:RefCell<VirtualDevice> = RefCell::new(
        VirtualDevice::builder().unwrap()
            .name("bincur-vkeyboard")
            .with_keys(&{
                let mut keys = AttributeSet::new();
                for key_code in 0..KEYCODE_MAX as u16 {
                    keys.insert(KeyCode::new(key_code));
                }
                keys
            }).unwrap()
            .build().unwrap()
    );
}

pub fn is_keyboard(device: &Device) -> bool
{
    device.supported_keys().map_or(false, |supported|
        supported.contains(evdev::KeyCode::KEY_A)
        && supported.contains(evdev::KeyCode::KEY_ENTER)
        && supported.contains(evdev::KeyCode::KEY_SPACE)
    )
}

pub fn scan() {
    KEYBOARDS.with_borrow_mut(|keyboards| {keyboards.clear();});

    for (path, device) in evdev::enumerate() {
        if device.name().map_or(false, |name| name.starts_with("bincur")) {continue}
        if is_keyboard(&device) {
            KEYBOARDS.with_borrow_mut(|keyboards| keyboards.push((path, device)));
        }
    }
}

pub fn names() -> Vec<Option<String>>
{
    let mut names:Vec<Option<String>> = Vec::new();
    KEYBOARDS.with_borrow(|keyboards| {
        for (_path, kbd) in keyboards {
            let name: Option<String> = match kbd.name() {
                Some(name) => Some(name.to_string()),
                None => None
            };
            names.push(name);
        }
    });
    names
}

pub fn physically_all_pressed(combo: &[usize], exclude: Option<usize>) -> bool
{
    PHYSICAL_PRESS_STATE.with_borrow(|key_states|{
        combo.iter()
            .all(|&key_code|
                Some(key_code) == exclude
                || key_states.get(key_code)
                    .unwrap_or(&(false, 0))
                    .0
            )
    })
}

pub fn logically_all_pressed(combo: &[usize], exclude: Option<usize>) -> bool
{
    LOGICAL_PRESS_STATE.with_borrow(|key_states| {
        combo.iter()
            .all(|&key_code|
                Some(key_code) == exclude
                || key_states.get(key_code).unwrap_or(&(false, false)).0)
    })
}

pub fn update_physical_presss_state(key_code: usize, key_value: i32)
{
    PHYSICAL_PRESS_STATE.with_borrow_mut(|p_state| {
        if let Some((pressed, _)) = p_state.get_mut(key_code) {
            *pressed = key_value > 0;
        }
    });
}

pub fn record_emitted(physical_keycode: usize, emitted_keycode: usize)
{
    PHYSICAL_PRESS_STATE.with_borrow_mut(|p_state| {
        if let Some((_, record)) = p_state.get_mut(physical_keycode) {
            *record = emitted_keycode;
        }
    });
}

pub fn take_emitted(physical_keycode: usize) -> usize
{
    PHYSICAL_PRESS_STATE.with_borrow_mut(|p_state| {
        match p_state.get_mut(physical_keycode) {
            Some((_, record)) => std::mem::replace(record, physical_keycode),
            None => physical_keycode,
        }
    })
}

pub fn peek_emitted(physical_keycode: usize) -> usize
{
    PHYSICAL_PRESS_STATE.with_borrow(|p_state| {
        p_state.get(physical_keycode).map_or(physical_keycode, |&(_, record)| record)
    })
}

pub fn update_logical_press_state(key_code: usize, key_value: i32)
{
    LOGICAL_PRESS_STATE.with_borrow_mut(|p_state| {
        if let Some((pressed, _)) = p_state.get_mut(key_code) {
            *pressed = key_value > 0;
        }
    });
}

pub fn update_grab_state(key_code: usize, key_value: i32, should_grab: &mut bool)
{
    LOGICAL_PRESS_STATE.with_borrow_mut(|p_state| {
        let Some((_, grabbed)) = p_state.get_mut(key_code)
        else {return};

        if key_value > 0 {
            *grabbed = *should_grab;
        }else {
            *should_grab = *grabbed;
            *grabbed = false;
        }
    });
}

pub fn clear_press_state()
{
    LOGICAL_PRESS_STATE.with_borrow_mut(|key_states|
        key_states.iter_mut()
            .for_each(|slot| *slot=(false, false) )
    );
    PHYSICAL_PRESS_STATE.with_borrow_mut(|key_states|
        key_states.iter_mut().enumerate()
            .for_each(|(keycode, slot)| *slot=(false, keycode))
    );
}

pub fn pass_through(event: InputEvent)
{
    VKEYBOARD_PASSTHROUGH.with_borrow_mut(|vkeyboard| {
        vkeyboard.emit(&[event]).unwrap();
    });
}
