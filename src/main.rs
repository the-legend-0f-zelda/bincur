// Create a virtual mouse, just while this is running.
// Generally this requires root.

use evdev::{AbsInfo, AbsoluteAxisCode, Device, KeyCode, UinputAbsSetup};
use evdev::{uinput::VirtualDevice, AttributeSet, EventType, InputEvent, RelativeAxisCode};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use MoveDirection::*;

fn main() -> std::io::Result<()> {

    // find keyboards
    let mut keyboards:Vec<(PathBuf, Device)> = Vec::new();
    for (path, dev) in evdev::enumerate() {
        if dev.supported_keys().map_or(false, |k| k.contains(evdev::KeyCode::KEY_A)) {
              keyboards.push((path, dev));
        }
    }
    println!("keyboards: {:#?}", keyboards);

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
