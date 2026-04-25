use std::{io, os::fd::AsRawFd};
use evdev::{EventType, FetchEventsSynced};
use mio::{Events, Interest, Poll, Token, unix::SourceFd};
use crate::{device::{keyboards::{self, KEYBOARDS, PRESS_STATE}, vmouse::ACTIVATED_SET}, setup::keymap};


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
            for (dev_idx, (_, device)) in v.iter().enumerate() {
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
    for ev in events {
        if EventType::KEY != ev.event_type() {
            continue;
        }

        let Some(related_behaviors) = keymap::load_rvs()
            .get(ev.code() as usize)
        else {continue};
        if related_behaviors.len() == 0 {continue;}

        PRESS_STATE.with_borrow_mut(|states| {
            let slot = match states.get_mut(ev.code() as usize) {
                Some(slot) => slot,
                None => return
            };
            *slot = ev.value() > 0;
        });

        ACTIVATED_SET.with_borrow_mut(|active| {
            if ev.value() > 0 { // On key down
                for behavior in related_behaviors {
                    let combo = match keymap::load_fwd().get(behavior) {
                        Some(combo) => combo,
                        None => continue
                    };

                    PRESS_STATE.with_borrow(|press| {
                        if combo.iter()
                            .all(|&k| *press.get(k as usize).unwrap_or(&false))
                        { active.insert(behavior.clone()); }
                    });
                }

                println!("syn report start");
                active.iter().for_each(|a| {
                    a.dispatch();
                });
                println!("syn report end");

            }else { // On key up
                for behavior in related_behaviors {
                    if !active.remove(behavior) {continue}
                    if let Some(inv) = behavior.inverse() {
                        inv.dispatch();
                    }
                }
            }
        });
    }
}
