use evdev::{AbsInfo, AbsoluteAxisCode, KeyCode, RelativeAxisCode, UinputAbsSetup};
use evdev::{uinput::VirtualDevice, AttributeSet, EventType, InputEvent};
use std::cell::RefCell;
use std::collections::HashSet;
use std::thread::sleep;
use std::time::Duration;
use MoveDirection::*;

use crate::setup;


thread_local! {
    pub static ACTIVATED_SET: RefCell<HashSet<Behavior>> = RefCell::new(HashSet::new());
    pub static VMOUSE_DEVICE :RefCell<VirtualDevice> = RefCell::new(
        VirtualDevice::builder().unwrap()
            .name("bincur")
            .with_relative_axes(setup::vmouse::get_rel_axes()).unwrap()
            .with_keys(setup::vmouse::get_keys()).unwrap()
            .build().unwrap()
    );
}

pub fn example() -> std::io::Result<()> {
    // Build virtual absolute mouse
    let abs_info = AbsInfo::new(
        0,        // value
        0,        // min
        32767,    // max
        0,        // fuzz
        0,        // flat
        1,        // resolution
    );

    let abs_x = UinputAbsSetup::new(AbsoluteAxisCode::ABS_X, abs_info);
    let abs_y = UinputAbsSetup::new(AbsoluteAxisCode::ABS_Y, abs_info);
    let mut keys = AttributeSet::<KeyCode>::new();
    keys.insert(KeyCode::BTN_LEFT);
    keys.insert(KeyCode::BTN_RIGHT);
    keys.insert(KeyCode::BTN_MIDDLE);

    let mut device = VirtualDevice::builder()?
        .name("bincur")
        .with_absolute_axis(&abs_x)?
        .with_absolute_axis(&abs_y)?
        .with_keys(&keys)?
        .build()?;

    for path in device.enumerate_dev_nodes_blocking()? {
        let path = path?;
        println!("Available as {}", path.display());
    }

    loop {
        let ev = new_move_mouse_event(Up, 500);
        device.emit(&[ev]).unwrap();
        println!("Moved mouse up");
        sleep(Duration::from_millis(100));

        let ev = new_move_mouse_event(Down, 50);
        device.emit(&[ev]).unwrap();
        println!("Moved mouse down");
        sleep(Duration::from_millis(100));

        let ev = new_move_mouse_event(Left, 50);
        device.emit(&[ev]).unwrap();
        println!("Moved mouse left");
        sleep(Duration::from_millis(100));

        let ev = new_move_mouse_event(Right, 50);
        device.emit(&[ev]).unwrap();
        println!("Moved mouse right");
        sleep(Duration::from_millis(100));
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Behavior {
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
            _ => None
        }
    }

    pub fn dispatch(&self) {
        VMOUSE_DEVICE.with_borrow_mut(|device| {
            let events: Vec<InputEvent> = match self {
                Self::MoveUp => {
                    println!("dispatch! : move up");
                    vec![new_move_mouse_event(Up, 50)]
                },
                Self::MoveDown => {
                    println!("dispatch! : move down");
                    vec![new_move_mouse_event(Down, 50)]
                },
                Self::MoveLeft => {
                    println!("dispatch! : move left");
                    vec![new_move_mouse_event(Left, 50)]
                },
                Self::MoveRight => {
                    println!("dispatch! : move right");
                    vec![new_move_mouse_event(Right, 50)]
                },

                Self::ClickLeft => {
                    println!("dispatch! : click left");
                    vec![InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_LEFT.code(), 1)]
                },
                Self::ClickRight => {
                    println!("dispatch! : click right");
                    vec![InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_RIGHT.code(), 1)]
                },
                Self::ReleaseLeft => {
                    println!("dispatch! : release left");
                    vec![InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_LEFT.code(), 0)]
                },
                Self::ReleaseRight => {
                    println!("dispatch! : release right");
                    vec![InputEvent::new_now(EventType::KEY.0, KeyCode::BTN_RIGHT.code(), 0)]
                },

                Self::ScrollUp => {
                    println!("dispatch! : scroll up");
                    vec![]
                },
                Self::ScrollDown => {
                    println!("dispatch! : scroll down");
                    vec![]
                }
            };

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

fn new_move_mouse_event(direction: MoveDirection, distance: u16) -> InputEvent {
    let (axis, distance) = match direction {
        MoveDirection::Up => (RelativeAxisCode::REL_Y, -i32::from(distance)),
        MoveDirection::Down => (RelativeAxisCode::REL_Y, i32::from(distance)),
        MoveDirection::Left => (RelativeAxisCode::REL_X, -i32::from(distance)),
        MoveDirection::Right => (RelativeAxisCode::REL_X, i32::from(distance)),
    };
    InputEvent::new_now(EventType::RELATIVE.0, axis.0, distance)
}
