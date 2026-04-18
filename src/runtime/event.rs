use std::{os::fd::AsRawFd};
use mio::{Events, Interest, Poll, Token, unix::SourceFd};
use crate::device::keyboards::{self, KEYBOARDS};


pub struct EventDriver {
    //devices: Vec<(PathBuf, Device)>,
    pub events:Events,
    poll: Poll
}

impl EventDriver {

    pub fn new() -> Self {
        let mut zelf = Self{
            //devices: Vec::new(),
            events: Events::with_capacity(16),
            poll: Poll::new().unwrap()
        };

        zelf.reset();
        zelf
    }

    pub fn reset(&mut self) {
        //self.devices = keyboards::scan();
        keyboards::scan();

        //for (dev_idx, (_, device)) in &mut self.devices.iter().enumerate() {
        for (dev_idx, (_, device)) in &mut KEYBOARDS.lock().unwrap().iter().enumerate()
        {
            device.set_nonblocking(true).unwrap();
            self.poll.registry()
                .register(
                    &mut SourceFd(&device.as_raw_fd()),
                    Token(dev_idx),
                    Interest::READABLE
                )
                .unwrap();
        }
    }

    pub fn block_ready(&mut self) -> Result<(), std::io::Error> {
        self.poll.poll(&mut self.events, None)
    }

}
