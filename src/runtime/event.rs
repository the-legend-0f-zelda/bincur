use std::{io, os::fd::AsRawFd};
use evdev::{EventType, FetchEventsSynced};
use mio::{Events, Interest, Poll, Token, unix::SourceFd};
use crate::{device::{self, keyboards::{self, KEYBOARDS, PRESS_STATE, pass_through}, vmouse::{ACTIVATED_SET, Behavior}}, setup::keymap};


pub struct EventDriver {
    pub events:Events,
    poll: Poll
}

impl EventDriver {

    pub fn new() -> Self {
        let mut zelf = Self{
            events: Events::with_capacity(16),
            poll: Poll::new().unwrap()
        };

        zelf.reset();
        zelf
    }

    pub fn reset(&mut self) {
        keyboards::scan();

        KEYBOARDS.with_borrow_mut(|v| {
            for (dev_idx, (_, device)) in v.iter_mut().enumerate() {
                device.grab().unwrap();
                device.set_nonblocking(true).unwrap();

                self.poll.registry().register(
                    &mut SourceFd(&device.as_raw_fd()),
                    Token(dev_idx),
                    Interest::READABLE
                ).unwrap()
            }
        });
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            self.poll.poll(&mut self.events, None)?;

            for ev in self.events.iter() {
                let token = ev.token();
                let kbd_idx = token.0;

                KEYBOARDS.with_borrow_mut(|keyboards| {
                    let target = &mut keyboards.get_mut(kbd_idx).unwrap().1;
                    loop {
                        match target.fetch_events() {
                            Ok(iter) => handle_events(iter),
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                            Err(e) => {
                                eprintln!("fetch_events error (kbd_idx={kbd_idx}): {}", e);
                                break;
                            }
                        }
                    }
                });
            }
        }
    }
}


fn handle_events(events: FetchEventsSynced){
    let keymap_fwd = keymap::load_fwd();

    for ev in events {
        if EventType::KEY != ev.event_type() {continue}

        PRESS_STATE.with_borrow_mut(|states| {
            match states.get_mut(ev.code() as usize) {
                Some(slot) => *slot = ev.value() > 0,
                None => return
            };
        });

        let Some(related_behaviors) = keymap::load_rvs().get(ev.code() as usize)
        else {continue};

        let mut to_dispatch:Vec<Behavior> = Vec::new();

        ACTIVATED_SET.with_borrow_mut(|active| {
            if ev.value() > 0 { // On key down
                for behavior in related_behaviors {
                    let Some(combo) = keymap_fwd.get(behavior)
                    else {continue};

                    PRESS_STATE.with_borrow(|press| {
                        if combo.iter()
                            .all(|&key| *press.get(key as usize).unwrap_or(&false))
                        {
                            match *behavior {
                                Behavior::LinearModeOn
                                | Behavior::LogarithmicModeOn => {
                                    active.insert(behavior.clone());
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
                        Behavior::LinearModeOn | Behavior::LogarithmicModeOn => {
                            to_dispatch.push(a.clone());
                        }
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
        if !grab {pass_through(ev);}
    }
}
