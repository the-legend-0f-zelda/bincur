use evdev::{AbsInfo, AbsoluteAxisCode, KeyCode, RelativeAxisCode, UinputAbsSetup};
use evdev::{uinput::VirtualDevice, EventType, InputEvent};
use std::cell::RefCell;
use std::collections::HashSet;
use Direction::*;

use crate::setup;
use crate::setup::vmouse::{ABS_INIT, ABS_MAX, ABS_MIN, Config, load_default};

thread_local! {
    pub static ACTIVATED_SET: RefCell<HashSet<Behavior>> = RefCell::new(HashSet::new());
    pub static VMOUSE_DEVICE: RefCell<VirtualDevice> = RefCell::new({
        let abs_info = AbsInfo::new(ABS_INIT, ABS_MIN, ABS_MAX, 0, 0, 0);
        VirtualDevice::builder().unwrap()
            .name("bincur")
            .with_absolute_axis(&UinputAbsSetup::new(AbsoluteAxisCode::ABS_X, abs_info)).unwrap()
            .with_absolute_axis(&UinputAbsSetup::new(AbsoluteAxisCode::ABS_Y, abs_info)).unwrap()
            .with_relative_axes(setup::vmouse::get_rel_axes()).unwrap()
            .with_keys(setup::vmouse::get_keys()).unwrap()
            .build().unwrap()
    });
    pub static VMOUSE_CFG: RefCell<Config> = RefCell::new(*setup::vmouse::load_default());
    pub static VMOUSE_POS: RefCell<(i32, i32)> = RefCell::new((ABS_INIT, ABS_INIT));
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
    ScrollLeft,
    ScrollRight,

    KeyUp
}

impl Behavior {
    pub fn from_str(behavior: &str) -> Self {
        match behavior.to_uppercase().as_str() {
            "LINEAR_MODE" => Self::LinearModeOn,
            "LOGARITHMIC_MODE" => Self::LogarithmicModeOn,
            //"LINEAR_MODE_OFF" => Self::LinearModeOff,
            //"LOGARITHMIC_MODE_OFF" => Self::LogarithmicModeOff,

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
            "SCROLL_LEFT" => Self::ScrollLeft,
            "SCROLL_RIGHT" => Self::ScrollRight,

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

            Self::ScrollUp
            | Self::ScrollDown
            | Self::ScrollLeft
            | Self::ScrollRight => None,

            _ => Some(Self::KeyUp)
        }
    }

    pub fn dispatch(&self) -> bool {
        let events: Vec<InputEvent> = match self {
            Self::LinearModeOn => {
                return VMOUSE_CFG.with_borrow_mut(|cfg| {
                    if ACTIVATED_SET.with_borrow(|c| !c.contains(&Behavior::LogarithmicModeOn)) {
                        cfg.mode = 1;
                        cfg.step_size_x = load_default().step_size_x;
                        cfg.step_size_y = load_default().step_size_y;
                        cfg.scroll_dist_x = load_default().scroll_dist_x;
                        cfg.scroll_dist_y = load_default().scroll_dist_y;
                    }
                    cfg.grab_linear
                });
            },
            Self::LinearModeOff => {
                return VMOUSE_CFG.with_borrow_mut(|cfg| {
                    if cfg.mode == 1 { cfg.mode = 0; }
                    cfg.grab_linear
                });
            },
            Self::LogarithmicModeOn => {
                return VMOUSE_CFG.with_borrow_mut(|cfg| {
                    cfg.mode = 2;
                    cfg.grab_logarithmic
                });
            },
            Self::LogarithmicModeOff => {
                return VMOUSE_CFG.with_borrow_mut(|cfg| {
                    if cfg.mode == 2 {
                        if ACTIVATED_SET.with_borrow(|c| c.contains(&Behavior::LinearModeOn)) {
                            cfg.mode = 1;
                        }else {
                            cfg.mode = 0;
                        }
                        cfg.step_size_x = load_default().step_size_x;
                        cfg.step_size_y = load_default().step_size_y;
                        cfg.scroll_dist_x = load_default().scroll_dist_x;
                        cfg.scroll_dist_y = load_default().scroll_dist_y;
                    }
                    cfg.grab_logarithmic
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

            Self::ScrollUp => new_scroll_event(Up),
            Self::ScrollDown => new_scroll_event(Down),
            Self::ScrollLeft => new_scroll_event(Left),
            Self::ScrollRight => new_scroll_event(Right),

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
    VMOUSE_CFG.with_borrow_mut(|cfg| {
        let step_size = match (cfg.mode, &direction) {
            (1, Up) | (1, Down) => cfg.step_size_y,
            (1, Left) | (1, Right) => cfg.step_size_x,

            (2, Up) | (2, Down) => {cfg.step_size_y /= 2; cfg.step_size_y},
            (2, Left) | (2, Right) => {cfg.step_size_x /= 2; cfg.step_size_x},

            _ => return vec![],
        };

        VMOUSE_POS.with_borrow_mut(|(x, y)| {
            let (axis, value) = match &direction {
                Up    => { *y = (*y - step_size).clamp(ABS_MIN, ABS_MAX); (AbsoluteAxisCode::ABS_Y, *y) },
                Down  => { *y = (*y + step_size).clamp(ABS_MIN, ABS_MAX); (AbsoluteAxisCode::ABS_Y, *y) },
                Left  => { *x = (*x - step_size).clamp(ABS_MIN, ABS_MAX); (AbsoluteAxisCode::ABS_X, *x) },
                Right => { *x = (*x + step_size).clamp(ABS_MIN, ABS_MAX); (AbsoluteAxisCode::ABS_X, *x) },
            };

            vec![InputEvent::new_now(EventType::ABSOLUTE.0, axis.0, value)]
        })
    })
}

fn new_click_event(direction: Direction, value: i32) -> Vec<InputEvent> {
    if VMOUSE_CFG.with_borrow(|cfg| cfg.mode) == 0 {return vec![]}
    return match direction {
        Left => vec![InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_LEFT.code(), value)],
        Right => vec![InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_RIGHT.code(), value)],
        _ => vec![]
    }
}

fn new_scroll_event(direction: Direction) -> Vec<InputEvent> {
    VMOUSE_CFG.with_borrow_mut(|cfg| {
        let scroll_dist = match (cfg.mode, &direction) {
            (1, Up) | (1, Down) => cfg.scroll_dist_y,
            (1, Left) | (1, Right) => cfg.scroll_dist_x,

            (2, Up) | (2, Down) => {cfg.scroll_dist_y /= 2; cfg.scroll_dist_y},
            (2, Left) | (2, Right) => {cfg.scroll_dist_x /= 2; cfg.scroll_dist_x},

            _ => return vec![],
        };

        let (axis, distance) = match &direction {
            Up => (RelativeAxisCode::REL_WHEEL, scroll_dist),
            Down => (RelativeAxisCode::REL_WHEEL, -scroll_dist),
            Left => (RelativeAxisCode::REL_HWHEEL, -scroll_dist),
            Right => (RelativeAxisCode::REL_HWHEEL, scroll_dist),
        };

        vec![InputEvent::new_now(EventType::RELATIVE.0, axis.0, distance)]
    })
}
