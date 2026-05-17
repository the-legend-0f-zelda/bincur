use std::{io, os::fd::AsRawFd};
use evdev::{Device, InputEvent};
use mio::{Events, Interest, Poll, Token, unix::SourceFd};
use udev::MonitorBuilder;
use crate::{device::{keyboards::{self, KEYBOARDS, PRESS_STATE}, vmouse::ACTIVATED_SET}, event::handler::determine_handler, setup::keymap};


pub struct Reactor {
    pub events:Events,
    monitor: udev::MonitorSocket,
    poll: Poll,
    handle_events: fn(&Device, usize, Vec<InputEvent>),
    grab: bool
}

impl Reactor {

    pub fn new(args:Vec<String>) -> Self {
        let monitor = MonitorBuilder::new().unwrap()
            .match_subsystem("input").unwrap()
            .listen().unwrap();
        let poll = Poll::new().unwrap();

        poll.registry().register(
            &mut SourceFd(&monitor.as_raw_fd()),
            Token(0),
            Interest::READABLE
        ).unwrap();

        let (handle_events, grab) = determine_handler(&args);

        let zelf = Self{
            events: Events::with_capacity(16),
            monitor,
            poll,
            handle_events,
            grab
        };

        zelf.reset();
        zelf
    }

    fn register_keyboards(&self) {
        KEYBOARDS.with_borrow_mut(|v| {
            v.retain_mut(|(path, device)| {
                if self.grab {
                    if let Err(e) = device.grab() {
                        eprintln!("grab failed ({}): {e}", path.display());
                        return false;
                    }
                }
                if let Err(e) = device.set_nonblocking(true) {
                    eprintln!("set_nonblocking failed ({}): {e}", path.display());
                    return false;
                }

                true
            });

            for (dev_idx, (path, device)) in v.iter_mut().enumerate() {
                if let Err(e) = self.poll.registry().register(
                    &mut SourceFd(&device.as_raw_fd()),
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
        keyboards::scan();
        keymap::initialize();

        ACTIVATED_SET.with_borrow_mut(|active| active.clear());
        PRESS_STATE.with_borrow_mut(|press| press.iter_mut().for_each(|s| *s=(false, false) ));

        self.register_keyboards();
    }

    pub fn run(&mut self) -> io::Result<()> {
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
                            udev::EventType::Add | udev::EventType::Remove => needs_reset = true,
                            _ => {}
                        }
                    }
                    continue;
                }

                needs_reset |= KEYBOARDS.with_borrow_mut(|keyboards| {
                    let kbd_idx = token.0 - 1;
                    let target = &mut keyboards.get_mut(kbd_idx).unwrap().1;
                    loop {
                        let events:Vec<InputEvent> = match target.fetch_events() {
                            Ok(iter) => iter.collect(),
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return false,
                            Err(e) => {
                                eprintln!("fetch_events error (kbd_idx={kbd_idx}): {}", e);
                                return true;
                            }
                        };
                        (self.handle_events)(target, kbd_idx, events);
                    }
                });
            }

            if needs_reset {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "reset required"))
            }
        }
    }
}
