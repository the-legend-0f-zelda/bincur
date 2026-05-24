use std::os::fd::AsRawFd;
use mio::{Events, Interest, Poll, Token, unix::SourceFd};
use udev::MonitorBuilder;
use crate::{config, device::{self, DeviceError, DeviceHandler, keyboards::KEYBOARDS}, event::handler::determine_handler};


pub struct Reactor {
    pub events:Events,
    monitor: udev::MonitorSocket,
    poll: Poll,
    handle_device: DeviceHandler,
    grab_default: bool
}

impl Reactor {

    pub fn new(args:Vec<String>) -> Self {
        let monitor = MonitorBuilder::new().unwrap()
            .match_subsystem("input").unwrap()
            .listen().unwrap();

        let poll = Poll::new().unwrap();
        let (handle_device, grab_default) = determine_handler(&args);

        poll.registry().register(
            &mut SourceFd(&monitor.as_raw_fd()),
            Token(0),
            Interest::READABLE
        ).unwrap();

        let new_reactor = Self{
            events: Events::with_capacity(16),
            monitor,
            poll,
            handle_device,
            grab_default
        };

        new_reactor.reset();
        new_reactor
    }

    fn register_keyboards(&self) {
        KEYBOARDS.with_borrow_mut(|keyboards| {
            keyboards.retain_mut(|(path, keyboard)| {
                if self.grab_default {
                    if let Err(e) = keyboard.grab() {
                        eprintln!("grab failed ({}): {e}", path.display());
                        return false;
                    }
                }
                if let Err(e) = keyboard.set_nonblocking(true) {
                    eprintln!("set_nonblocking failed ({}): {e}", path.display());
                    return false;
                }

                true
            });

            for (dev_idx, (path, keyboard)) in keyboards.iter_mut().enumerate() {
                if let Err(e) = self.poll.registry().register(
                    &mut SourceFd(&keyboard.as_raw_fd()),
                    Token(dev_idx+1),
                    Interest::READABLE
                ) {
                    eprintln!("poll register failed ({}): {e}", path.display());
                }
            }
        });
    }

    fn deregister_keyboards(&self) {
        KEYBOARDS.with_borrow_mut(|v| {
            for (_, device) in v.iter_mut() {
                let _r = self.poll.registry()
                    .deregister(&mut SourceFd(&device.as_raw_fd()));
            }
        });
    }

    pub fn reset(&self) {
        self.deregister_keyboards();
        device::keyboards::scan();
        config::keymap::initialize();
        device::vmouse::clear_activated_set();
        device::keyboards::clear_press_state();
        self.register_keyboards();
    }

    pub fn run(&mut self) -> Result<(), DeviceError>{
        loop {
            self.poll.poll(&mut self.events, None)?;
            let mut needs_reset = false;

            for ev in self.events.iter() {
                let token = ev.token();

                if token.0 == 0 {
                    for device in self.monitor.iter() {
                        if device.syspath().to_string_lossy().contains("/virtual/input") { continue }
                        let Some(node) = device.devnode() else { continue };
                        if !node.to_string_lossy().starts_with("/dev/input/event") { continue }

                        match device.event_type() {
                            udev::EventType::Add => return Err(DeviceError::Connected),
                            udev::EventType::Remove => return Err(DeviceError::Disconnected),
                            udev::EventType::Change => return Err(DeviceError::ResetRequested),
                            _ => {}
                        }
                    }
                    continue;
                }

                needs_reset |= KEYBOARDS.with_borrow_mut(|keyboards| {
                    let kbd_idx = token.0 - 1;
                    let target = &mut keyboards.get_mut(kbd_idx).unwrap().1;
                    loop {
                        match (self.handle_device)(target, kbd_idx) {
                            Ok(iter) => iter,
                            Err(DeviceError::Io(e)) if e.kind() == std::io::ErrorKind::WouldBlock => return false,
                            Err(_) => {
                                eprintln!("[RESET REQUIRED] fetch_events error (kbd_idx={kbd_idx})");
                                return true;
                            }
                        };
                    }
                });
            }

            if needs_reset {return Err(DeviceError::ResetRequested);}
        }
    }
}
