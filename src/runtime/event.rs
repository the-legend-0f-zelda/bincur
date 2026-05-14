use std::{io, os::fd::AsRawFd};
use evdev::{EventType, FetchEventsSynced};
use mio::{Events, Interest, Poll, Token, unix::SourceFd};
use udev::MonitorBuilder;
use crate::{device::{self, keyboards::{self, KEYBOARDS, PRESS_STATE, pass_through}, vmouse::{ACTIVATED_SET, Behavior}}, setup::{self, keymap}};


pub struct EventDriver {
    pub events:Events,
    monitor: udev::MonitorSocket,
    poll: Poll
}

impl EventDriver {

    pub fn new() -> Self {
        let monitor = MonitorBuilder::new().unwrap()
            .match_subsystem("input").unwrap()
            .listen().unwrap();
        let poll = Poll::new().unwrap();

        poll.registry().register(
            &mut SourceFd(&monitor.as_raw_fd()),
            Token(0),
            Interest::READABLE
        ).unwrap();

        let mut zelf = Self{
            events: Events::with_capacity(16),
            monitor,
            poll
        };

        zelf.reset();
        zelf
    }

    pub fn reset(&mut self) {
        KEYBOARDS.with_borrow_mut(|v| {
            for (_, device) in v.iter_mut() {
                let _r = self.poll.registry()
                    .deregister(&mut SourceFd(&device.as_raw_fd()));
            }
        });

        keyboards::scan();

        KEYBOARDS.with_borrow_mut(|v| {
            v.retain_mut(|(path, device)| {
                if let Err(e) = device.grab() {
                    eprintln!("grab failed ({}): {e}", path.display());
                    return false;
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

        ACTIVATED_SET.with_borrow_mut(|active| active.clear());
        PRESS_STATE.with_borrow_mut(|press| press.iter_mut().for_each(|s| *s=(false, false) ));
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            self.poll.poll(&mut self.events, None)?;
            let mut needs_reset = false;

            for ev in self.events.iter() {
                let token = ev.token();

                if token.0 == 0 {
                    for dev in self.monitor.iter() {
                        if dev.syspath().to_string_lossy().contains("/virtual/input") { continue }
                        let Some(node) = dev.devnode() else { continue };
                        if !node.to_string_lossy().starts_with("/dev/input/event") { continue }
                        match dev.event_type() {
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
                        match target.fetch_events() {
                            Ok(iter) => handle_events(iter),
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return false,
                            Err(e) => {
                                eprintln!("fetch_events error (kbd_idx={kbd_idx}): {}", e);
                                return true;
                            }
                        }
                    }
                });
            }

            if needs_reset {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "reset required"))
            }
        }
    }
}


fn handle_events(events: FetchEventsSynced){
    let keymap_fwd = keymap::load_fwd();

    for mut ev in events {
        if EventType::KEY != ev.event_type() {continue}
        ev = setup::keymap::rewire(ev);

        let code = ev.code() as usize;
        let value = ev.value();

        PRESS_STATE.with_borrow_mut(|states| {
            match states.get_mut(code) {
                Some(slot) => slot.0 = value > 0,
                None => return
            };
        });

        let Some(related_behaviors) = keymap::load_rvs().get(code)
        else {
            pass_through(ev);
            continue;
        };

        let mut to_dispatch:Vec<Behavior> = Vec::new();

        ACTIVATED_SET.with_borrow_mut(|active| {
            if value > 0 { // On key down
                for behavior in related_behaviors {
                    let Some(combo) = keymap_fwd.get(behavior)
                    else {continue};

                    PRESS_STATE.with_borrow(|press| {
                        if combo.iter()
                            .all(|&key| press.get(key as usize).unwrap_or( &(false, false) ).0 )
                        {
                            match *behavior {
                                Behavior::LinearModeOn
                                | Behavior::LogarithmicModeOn
                                | Behavior::ScrollModeOn
                                | Behavior::Exit => {
                                    active.insert(behavior.clone());
                                    to_dispatch.push(behavior.clone());
                                },
                                _ => {
                                    if device::vmouse::VMOUSE_CFG
                                        .with_borrow(|cfg| cfg.mode) > 0
                                    {active.insert(behavior.clone());}
                                }
                            }
                        }
                    });
                }

                let mut max_combo_len = 0;
                let mut longest: Vec<Behavior> = Vec::new();

                for a in active.iter() {
                    match a {
                        Behavior::LinearModeOn
                        | Behavior::LogarithmicModeOn
                        | Behavior::ScrollModeOn
                        | Behavior::Exit => {
                            continue;
                        },
                        _ => {
                            let len = keymap_fwd.get(a).map_or(0, |c| c.len());
                            if len < max_combo_len {continue}
                            if len > max_combo_len {
                                longest.clear();
                                max_combo_len = len;
                            }
                            longest.push(a.clone());
                        }
                    }
                }

                to_dispatch.extend(longest);

            }else { // On key up
                for behavior in related_behaviors {
                    if !active.remove(behavior) {continue}
                    if let Some(inv) = behavior.inverse() {
                        to_dispatch.push(inv);
                    }
                }
            }
        });

        let mut grab = false;
        for behavior in to_dispatch {
            grab |= behavior.dispatch();
        }

        PRESS_STATE.with_borrow_mut(|s| {
            let Some(slot) = s.get_mut(code)
            else {return};

            if value > 0 {
                slot.1 = grab;
            }else {
                grab = slot.1;
                slot.1 = false;
            }
        });

        if !grab {pass_through(ev);}
    }
}
