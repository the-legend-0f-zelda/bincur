use evdev::{KeyCode, RelativeAxisCode};
use evdev::{uinput::VirtualDevice, EventType, InputEvent};
use std::cell::RefCell;
use std::collections::HashSet;
use Direction::*;

use crate::setup;
use crate::setup::vmouse::Props;

thread_local! {
    pub static ACTIVATED_SET: RefCell<HashSet<Behavior>> = RefCell::new(HashSet::new());
    pub static VMOUSE_DEVICE: RefCell<VirtualDevice> = RefCell::new(
        VirtualDevice::builder().unwrap()
            .name("bincur")
            .with_relative_axes(setup::vmouse::get_rel_axes()).unwrap()
            .with_keys(setup::vmouse::get_keys()).unwrap()
            .build().unwrap()
    );
    pub static VMOUSE_PROPS: RefCell<Props> = RefCell::new(*setup::vmouse::load_default());
}

pub fn mark_active(behavior: &Behavior) -> bool {
    ACTIVATED_SET
        .with_borrow_mut(|a_set| a_set.insert(behavior.clone()))
}

pub fn mark_inactive(behavior: &Behavior) -> bool {
    ACTIVATED_SET
        .with_borrow_mut(|a_set| a_set.remove(behavior))
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Behavior {
    Exit,

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

    pub fn dispatch(&self) -> bool {
        let events: Vec<InputEvent> = match self {
            Self::Exit => {
                println!("Exit bincur.");
                std::process::exit(0);
            }

            Self::LinearModeOn => {
                return VMOUSE_PROPS.with_borrow_mut(|cfg| {
                    if cfg.mode < 1 {
                        cfg.mode = 1;
                        cfg.reset_xy();
                    }
                    cfg.grab_linear
                });
            },
            Self::LinearModeOff => {
                return VMOUSE_PROPS.with_borrow_mut(|cfg| {
                    if cfg.mode == 1 { cfg.mode = 0; }
                    cfg.grab_linear
                });
            },

            Self::LogarithmicModeOn => {
                return VMOUSE_PROPS.with_borrow_mut(|cfg| {
                    if cfg.mode < 2 {
                        cfg.mode = 2;
                    }
                    cfg.grab_logarithmic
                });
            },
            Self::LogarithmicModeOff => {
                return VMOUSE_PROPS.with_borrow_mut(|cfg| {
                    if cfg.mode == 2 {
                        if ACTIVATED_SET.with_borrow(|a| a.contains(&Behavior::LinearModeOn)) {
                            cfg.mode = 1;
                        }else {
                            cfg.mode = 0;
                        }
                        cfg.reset_xy();
                    }
                    cfg.grab_logarithmic
                });
            },

            Self::ScrollModeOn => {
                return VMOUSE_PROPS.with_borrow_mut(|cfg| {
                    cfg.mode = 3;
                    cfg.grab_scroll
                });
            },
            Self::ScrollModeOff => {
                return VMOUSE_PROPS.with_borrow_mut(|cfg| {
                    if cfg.mode == 3 {
                        if ACTIVATED_SET.with_borrow(|a| a.contains(&Behavior::LogarithmicModeOn)) {
                            cfg.mode = 2;
                        }else if ACTIVATED_SET.with_borrow(|a| a.contains(&Behavior::LinearModeOn)) {
                            cfg.mode = 1;
                            cfg.reset_xy();
                        }else {
                            cfg.mode = 0;
                        }
                    }
                    cfg.grab_scroll
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

            Self::KeyUp => return true
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

fn new_move_event(direction: Direction) -> Vec<InputEvent> {
    VMOUSE_PROPS.with_borrow_mut(|cfg| {
        let (axis, step_size) = match (cfg.mode, &direction) {
            (1, Up) => (RelativeAxisCode::REL_Y, -cfg.step_size_y),
            (1, Down) => (RelativeAxisCode::REL_Y, cfg.step_size_y),
            (1, Left) => (RelativeAxisCode::REL_X, -cfg.step_size_x),
            (1, Right) => (RelativeAxisCode::REL_X, cfg.step_size_x),

            (2, Up) => {cfg.step_size_y = (cfg.step_size_y+1)>>1; (RelativeAxisCode::REL_Y, -cfg.step_size_y)},
            (2, Down) => {cfg.step_size_y = (cfg.step_size_y+1)>>1; (RelativeAxisCode::REL_Y, cfg.step_size_y)},
            (2, Left) => {cfg.step_size_x = (cfg.step_size_x+1)>>1; (RelativeAxisCode::REL_X, -cfg.step_size_x)},
            (2, Right) => {cfg.step_size_x = (cfg.step_size_x+1)>>1; (RelativeAxisCode::REL_X, cfg.step_size_x)},

            (3, Up) => (RelativeAxisCode::REL_WHEEL, cfg.scroll_dist_y),
            (3, Down) => (RelativeAxisCode::REL_WHEEL, -cfg.scroll_dist_y),
            (3, Left) => (RelativeAxisCode::REL_HWHEEL, -cfg.scroll_dist_x),
            (3, Right) => (RelativeAxisCode::REL_HWHEEL, cfg.scroll_dist_x),

            _ => return vec![],
        };
        vec![InputEvent::new_now(EventType::RELATIVE.0, axis.0, step_size)]
    })
}

fn new_click_event(direction: Direction, value: i32) -> Vec<InputEvent> {
    if VMOUSE_PROPS.with_borrow(|cfg| cfg.mode) == 0 {return vec![]}
    return match direction {
        Left => vec![InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_LEFT.code(), value)],
        Right => vec![InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_RIGHT.code(), value)],
        _ => vec![]
    }
}
