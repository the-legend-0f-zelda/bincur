use evdev::{KeyCode, RelativeAxisCode};
use evdev::{uinput::VirtualDevice, EventType, InputEvent};
use std::cell::RefCell;
use std::collections::HashSet;
use MoveDirection::*;

use crate::setup;
use crate::setup::vmouse::Config;

thread_local! {
    pub static ACTIVATED_SET: RefCell<HashSet<Behavior>> = RefCell::new(HashSet::new());
    pub static VMOUSE_DEVICE: RefCell<VirtualDevice> = RefCell::new(
        VirtualDevice::builder().unwrap()
            .name("bincur")
            .with_relative_axes(setup::vmouse::get_rel_axes()).unwrap()
            .with_keys(setup::vmouse::get_keys()).unwrap()
            .build().unwrap()
    );
    pub static VMOUSE_CFG: RefCell<Config> = RefCell::new(*setup::vmouse::load_default());
}


#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Behavior {
    LinearModeOn,
    LogarithmicModeOn,
    LinearModeOff,
    LogarithmicModeOff,

    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    ClickLeft,
    ClickRight,
    ReleaseLeft,
    ReleaseRight,

    ScrollUp,
    ScrollDown
}

impl Behavior {
    pub fn from_str(behavior: &str) -> Self {
        match behavior.to_uppercase().as_str() {
            "LINEAR_MODE_ON" => Self::LinearModeOn,
            "LOGARITHMIC_MODE_ON" => Self::LogarithmicModeOn,
            "LINEAR_MODE_OFF" => Self::LinearModeOff,
            "LOGARITHMIC_MODE_OFF" => Self::LogarithmicModeOff,

            "MOVE_UP" => Self::MoveUp,
            "MOVE_DOWN" => Self::MoveDown,
            "MOVE_LEFT" => Self::MoveLeft,
            "MOVE_RIGHT" => Self::MoveRight,

            "CLICK_LEFT" => Self::ClickLeft,
            "CLICK_RIGHT" => Self::ClickRight,
            "RELEASE_LEFT" => Self::ReleaseLeft,
            "RELEASE_RIGHT" => Self::ReleaseRight,

            "SCROLL_UP" => Self::ScrollUp,
            "SCROLL_DOWN" => Self::ScrollDown,

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
            _ => None
        }
    }

    pub fn dispatch(&self) {
        let events: Vec<InputEvent> = match self {
            Self::LinearModeOn => {
                VMOUSE_CFG.with_borrow_mut(|cfg| {
                    cfg.mode = 1
                });
                return
            },
            Self::LogarithmicModeOn => {
                VMOUSE_CFG.with_borrow_mut(|cfg| {
                    cfg.mode = 2
                });
                return
            }
            Self::LinearModeOff | Self::LogarithmicModeOff => {
                VMOUSE_CFG.with_borrow_mut(|cfg| {
                    cfg.mode = 0;
                });
                return
            }

            Self::MoveUp => vec![new_move_mouse_event(Up)],
            Self::MoveDown => vec![new_move_mouse_event(Down)],
            Self::MoveLeft => vec![new_move_mouse_event(Left)],
            Self::MoveRight => vec![new_move_mouse_event(Right)],

            Self::ClickLeft => vec![InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_LEFT.code(), 1)],
            Self::ClickRight => vec![InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_RIGHT.code(), 1)],
            Self::ReleaseLeft => vec![InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_LEFT.code(), 0)],
            Self::ReleaseRight => vec![InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_RIGHT.code(), 0)],

            Self::ScrollUp => {
                println!("dispatch! : scroll up");
                vec![]
            },
            Self::ScrollDown => {
                println!("dispatch! : scroll down");
                vec![]
            },
        };

        VMOUSE_DEVICE.with_borrow_mut(|device| {
            if let Err(e) = device.emit(&events) {
                eprintln!("ERROR - emit failed: {}", e);
            }
        });
    }
}


enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

fn new_move_mouse_event(direction: MoveDirection) -> InputEvent {
    let step_size = VMOUSE_CFG.with_borrow(|cfg| cfg.step_size);
    let (axis, distance) = match direction {
        MoveDirection::Up => (RelativeAxisCode::REL_Y, -i32::from(step_size)),
        MoveDirection::Down => (RelativeAxisCode::REL_Y, i32::from(step_size)),
        MoveDirection::Left => (RelativeAxisCode::REL_X, -i32::from(step_size)),
        MoveDirection::Right => (RelativeAxisCode::REL_X, i32::from(step_size)),
    };
    InputEvent::new_now(EventType::RELATIVE.0, axis.0, distance)
}
