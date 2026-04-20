use evdev::{AbsInfo, AbsoluteAxisCode, KeyCode, UinputAbsSetup};
use evdev::{uinput::VirtualDevice, AttributeSet, EventType, InputEvent};
use std::thread::sleep;
use std::time::Duration;
use MoveDirection::*;

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
        // do something
        match self {
            Self::MoveUp => {
                println!("dispatch! : move up");
            },
            Self::MoveDown => {
                println!("dispatch! : move down");
            },
            Self::MoveLeft => {
                println!("dispatch! : move left");
            },
            Self::MoveRight => {
                println!("dispatch! : move right");
            },

            Self::ClickLeft => {
                println!("dispatch! : click left");
            },
            Self::ClickRight => {
                println!("dispatch! : click right");
            },
            Self::ReleaseLeft => {
                println!("dispatch! : release left");
            },
            Self::ReleaseRight => {
                println!("dispatch! : release right");
            },

            Self::ScrollUp => {
                println!("dispatch! : scroll up");
            },
            Self::ScrollDown => {
                println!("dispatch! : scroll down");
            }
        }
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
        MoveDirection::Up => (AbsoluteAxisCode::ABS_Y, -i32::from(distance)),
        MoveDirection::Down => (AbsoluteAxisCode::ABS_Y, i32::from(distance)),
        MoveDirection::Left => (AbsoluteAxisCode::ABS_X, -i32::from(distance)),
        MoveDirection::Right => (AbsoluteAxisCode::ABS_X, i32::from(distance)),
    };
    InputEvent::new_now(EventType::ABSOLUTE.0, axis.0, distance)
}
