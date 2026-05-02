use evdev::{KeyCode, RelativeAxisCode};
use evdev::{uinput::VirtualDevice, EventType, InputEvent};
use std::cell::RefCell;
use std::collections::HashSet;
use Direction::*;

use crate::setup;
use crate::setup::vmouse::{Config, load_default};

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
    ScrollDown,

    KeyUp
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
            _ => Some(Self::KeyUp)
        }
    }

    pub fn dispatch(&self) -> bool {
        let events: Vec<InputEvent> = match self {
            Self::LinearModeOn => {
                if ACTIVATED_SET.with_borrow(|c| !c.contains(&Behavior::LogarithmicModeOn)) {
                    VMOUSE_CFG.with_borrow_mut(|cfg| {
                        cfg.mode = 1;
                        cfg.step_size = load_default().step_size;
                    });
                }
                return true;
            },
            Self::LogarithmicModeOn => {
                VMOUSE_CFG.with_borrow_mut(|cfg| {
                    cfg.mode = 2;
                });
                return true;
            }
            Self::LinearModeOff => {
                VMOUSE_CFG.with_borrow_mut(|cfg| {
                    if cfg.mode == 1 { cfg.mode = 0; }
                });
                return true;
            },
            Self::LogarithmicModeOff => {
                VMOUSE_CFG.with_borrow_mut(|cfg| {
                    if cfg.mode == 2 {
                        if ACTIVATED_SET.with_borrow(|c| !c.contains(&Behavior::LinearModeOn)) {
                            cfg.mode = 0;
                        }else {
                            cfg.mode = 1;
                        }
                        cfg.step_size = load_default().step_size
                    }
                });
                return true;
            },

            Self::MoveUp => new_move_mouse_event(Up),
            Self::MoveDown => new_move_mouse_event(Down),
            Self::MoveLeft => new_move_mouse_event(Left),
            Self::MoveRight => new_move_mouse_event(Right),

            Self::ClickLeft => new_click_mouse_event(Left, 1),
            Self::ClickRight => new_click_mouse_event(Right, 1),
            Self::ReleaseLeft => new_click_mouse_event(Left, 0),
            Self::ReleaseRight => new_click_mouse_event(Right, 0),

            Self::ScrollUp => {
                println!("dispatch! : scroll up");
                vec![]
            },
            Self::ScrollDown => {
                println!("dispatch! : scroll down");
                vec![]
            },

            Self::KeyUp => return false
        };

        if events.is_empty() {return false;}

        VMOUSE_DEVICE.with_borrow_mut(|device| {
            if let Err(e) = device.emit(&events) {
                eprintln!("ERROR - emit failed: {}", e);
            }
        });

        true
    }
}

enum Direction {Up, Down, Left, Right,}

fn new_move_mouse_event(direction: Direction) -> Vec<InputEvent> {
    VMOUSE_CFG.with_borrow_mut(|cfg| {
        let step_size = match cfg.mode {
            1 => cfg.step_size,
            2 => {
                cfg.step_size /= 2;
                cfg.step_size
            }
            _ => return vec![],
        };

        let (axis, distance) = match direction {
            Direction::Up => (RelativeAxisCode::REL_Y, -i32::from(step_size)),
            Direction::Down => (RelativeAxisCode::REL_Y, i32::from(step_size)),
            Direction::Left => (RelativeAxisCode::REL_X, -i32::from(step_size)),
            Direction::Right => (RelativeAxisCode::REL_X, i32::from(step_size)),
        };

        vec![InputEvent::new_now(EventType::RELATIVE.0, axis.0, distance)]
    })
}

fn new_click_mouse_event(direction: Direction, value: i32) -> Vec<InputEvent> {
    if VMOUSE_CFG.with_borrow(|cfg| cfg.mode) == 0 {return vec![]}
    return match direction {
        Left => vec![InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_LEFT.code(), value)],
        Right => vec![InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_RIGHT.code(), value)],
        _ => vec![]
    }
}
