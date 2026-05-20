use std::io;

use evdev::Device;

pub mod keyboards;
pub mod vmouse;

pub type DeviceHandler = fn(&mut Device, usize) -> io::Result<()>;
