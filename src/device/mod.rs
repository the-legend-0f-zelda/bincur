use evdev::Device;

pub mod keyboards;
pub mod vmouse;

pub type DeviceHandler = fn(&mut Device, usize) -> Result<(),DeviceError>;

#[derive(Debug)]
pub enum DeviceError {
    Disconnected,
    Connected,
    ResetRequested,
    Io(std::io::Error)
}

impl From<std::io::Error> for DeviceError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
