use arrayvec::ArrayVec;
use evdev::{KeyCode, RelativeAxisCode};
use evdev::{uinput::VirtualDevice, EventType, InputEvent};
use std::cell::RefCell;
use std::collections::HashSet;
use Direction::*;

use crate::config::{self, keymap};
use crate::config::vmouse::Props;
use crate::device::DeviceError;

thread_local! {
    pub static ACTIVATED_SET: RefCell<HashSet<Behavior>> = RefCell::new(HashSet::new());
    pub static VMOUSE_DEVICE: RefCell<VirtualDevice> = RefCell::new(
        VirtualDevice::builder().unwrap()
            .name("bincur")
            .with_relative_axes(config::vmouse::get_rel_axes()).unwrap()
            .with_keys(config::vmouse::get_keys()).unwrap()
            .build().unwrap()
    );
    pub static VMOUSE_PROPS: RefCell<Props> = RefCell::new(*config::vmouse::load_default_props());
}

pub fn mark_active(behavior: &Behavior) -> bool {
    ACTIVATED_SET
        .with_borrow_mut(|a_set| a_set.insert(behavior.clone()))
}

pub fn mark_inactive(behavior: &Behavior) -> bool {
    ACTIVATED_SET
        .with_borrow_mut(|a_set| a_set.remove(behavior))
}

pub fn is_active(behavior: &Behavior) -> bool {
    ACTIVATED_SET.with_borrow(|a_set| a_set.contains(behavior))
}

pub fn longest_actives(kbd_idx: usize) -> ArrayVec<Behavior, 16>
{
    let mut longest: ArrayVec<Behavior, 16> = ArrayVec::new();
    let mut max_combo_len = 0;

    ACTIVATED_SET.with_borrow(|a_set| {
        for a in a_set.iter() {
            match a {
                Behavior::LinearModeOn
                | Behavior::LogarithmicModeOn
                | Behavior::ScrollModeOn
                | Behavior::Exit => {
                    continue;
                },
                _ => {
                    let len = match keymap::get_combo(kbd_idx, a) {
                        Some(combo) => combo.len(),
                        None => 0
                    };
                    if len < max_combo_len {continue}
                    if len > max_combo_len {
                        longest.clear();
                        max_combo_len = len;
                    }
                    longest.push(a.clone());
                }
            }
        }
    });

    longest
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Behavior {
    Exit,
    Reset,

    LinearModeOn,
    LogarithmicModeOn,

    LinearModeOff,
    LogarithmicModeOff,

    ScrollModeOn,
    ScrollModeOff,

    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    ClickLeft,
    ClickRight,
    ReleaseLeft,
    ReleaseRight,

    KeyUp
}

impl Behavior {
    pub fn from_str(behavior: &str) -> Self {
        match behavior.to_uppercase().as_str() {
            "EXIT" => Self::Exit,
            "RESET" => Self::Reset,

            "LINEAR_MODE" => Self::LinearModeOn,
            "LOGARITHMIC_MODE" => Self::LogarithmicModeOn,
            "SCROLL_MODE" => Self::ScrollModeOn,

            "MOVE_UP" => Self::MoveUp,
            "MOVE_DOWN" => Self::MoveDown,
            "MOVE_LEFT" => Self::MoveLeft,
            "MOVE_RIGHT" => Self::MoveRight,

            "CLICK_LEFT" => Self::ClickLeft,
            "CLICK_RIGHT" => Self::ClickRight,
            "RELEASE_LEFT" => Self::ReleaseLeft,
            "RELEASE_RIGHT" => Self::ReleaseRight,

            _ => {
                eprintln!("ERROR - unknown vmouse behvior: {}", behavior);
                std::process::exit(1);
            }
        }
    }

    pub fn inverse(&self) -> Option<Self> {
        match self {
            Self::ClickLeft => Some(Self::ReleaseLeft),
            Self::ClickRight => Some(Self::ReleaseRight),

            Self::LinearModeOn => Some(Self::LinearModeOff),
            Self::LogarithmicModeOn => Some(Self::LogarithmicModeOff),
            Self::ScrollModeOn => Some(Self::ScrollModeOff),

            _ => Some(Self::KeyUp)
        }
    }

    pub fn dispatch(&self) -> Result<bool, DeviceError> {
        let Some(event) = (match self {
            Self::Exit => {
                println!("Exit bincur.");
                std::process::exit(0);
            },
            Self::Reset => {
                return Err(DeviceError::ResetRequested)
            },

            Self::LinearModeOn => {
                return VMOUSE_PROPS.with_borrow_mut(|mouse| {
                    if mouse.mode < 1 {
                        mouse.mode = 1;
                        mouse.reset_xy();
                    }
                    Ok(mouse.grab_linear)
                });
            },
            Self::LinearModeOff => {
                return VMOUSE_PROPS.with_borrow_mut(|mouse| {
                    if mouse.mode == 1 { mouse.mode = 0; }
                    Ok(mouse.grab_linear)
                });
            },

            Self::LogarithmicModeOn => {
                return VMOUSE_PROPS.with_borrow_mut(|mouse| {
                    if mouse.mode < 2 {
                        mouse.mode = 2;
                    }
                    Ok(mouse.grab_logarithmic)
                });
            },
            Self::LogarithmicModeOff => {
                return VMOUSE_PROPS.with_borrow_mut(|mouse| {
                    if mouse.mode == 2 {
                        if is_active(&Behavior::LinearModeOn) {
                            mouse.mode = 1;
                        }else {
                            mouse.mode = 0;
                        }
                        mouse.reset_xy();
                    }
                    Ok(mouse.grab_logarithmic)
                });
            },

            Self::ScrollModeOn => {
                return VMOUSE_PROPS.with_borrow_mut(|mouse| {
                    mouse.mode = 3;
                    Ok(mouse.grab_scroll)
                });
            },
            Self::ScrollModeOff => {
                return VMOUSE_PROPS.with_borrow_mut(|mouse| {
                    if mouse.mode == 3 {
                        if is_active(&Behavior::LogarithmicModeOn) {
                            mouse.mode = 2;
                        }else if is_active(&Behavior::LinearModeOn) {
                            mouse.mode = 1;
                            mouse.reset_xy();
                        }else {
                            mouse.mode = 0;
                        }
                    }
                    Ok(mouse.grab_scroll)
                });
            },

            Self::MoveUp => new_move_event(Up),
            Self::MoveDown => new_move_event(Down),
            Self::MoveLeft => new_move_event(Left),
            Self::MoveRight => new_move_event(Right),

            Self::ClickLeft => new_click_event(Left, 1),
            Self::ClickRight => new_click_event(Right, 1),
            Self::ReleaseLeft => new_click_event(Left, 0),
            Self::ReleaseRight => new_click_event(Right, 0),

            Self::KeyUp => return Ok(true)

        })else {
            return Ok(false);
        };

        VMOUSE_DEVICE.with_borrow_mut(|device| {
            if let Err(e) = device.emit(&[event]) {
                eprintln!("ERROR - emit failed: {}", e);
            }
        });

        Ok(true)
    }
}

enum Direction {Up, Down, Left, Right,}

fn new_move_event(direction: Direction) -> Option<InputEvent>
{
    VMOUSE_PROPS.with_borrow_mut(|mouse| {
        let (axis, step_size) = match (mouse.mode, &direction) {
            // Linear mode
            (1, Up) => (RelativeAxisCode::REL_Y, -mouse.step_size_y),
            (1, Down) => (RelativeAxisCode::REL_Y, mouse.step_size_y),
            (1, Left) => (RelativeAxisCode::REL_X, -mouse.step_size_x),
            (1, Right) => (RelativeAxisCode::REL_X, mouse.step_size_x),

            // Logarithmic mode
            (2, Up) => {
                mouse.step_size_y = (mouse.step_size_y+1)>>1;
                (RelativeAxisCode::REL_Y, -mouse.step_size_y)
            },
            (2, Down) => {
                mouse.step_size_y = (mouse.step_size_y+1)>>1;
                (RelativeAxisCode::REL_Y, mouse.step_size_y)
            },
            (2, Left) => {
                mouse.step_size_x = (mouse.step_size_x+1)>>1;
                (RelativeAxisCode::REL_X, -mouse.step_size_x)
            },
            (2, Right) => {
                mouse.step_size_x = (mouse.step_size_x+1)>>1;
                (RelativeAxisCode::REL_X, mouse.step_size_x)
            },

            // Scroll mode
            (3, Up) => (RelativeAxisCode::REL_WHEEL, mouse.scroll_dist_y),
            (3, Down) => (RelativeAxisCode::REL_WHEEL, -mouse.scroll_dist_y),
            (3, Left) => (RelativeAxisCode::REL_HWHEEL, -mouse.scroll_dist_x),
            (3, Right) => (RelativeAxisCode::REL_HWHEEL, mouse.scroll_dist_x),

            _ => return None,
        };

        Some( InputEvent::new_now(EventType::RELATIVE.0, axis.0, step_size) )
    })
}

fn new_click_event(direction: Direction, value: i32) -> Option<InputEvent> {
    if VMOUSE_PROPS.with_borrow(|mouse| mouse.mode) == 0 {return None}

    return match direction {
        Left => Some( InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_LEFT.code(), value) ),
        Right => Some( InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_RIGHT.code(), value) ),
        _ => None
    }
}
