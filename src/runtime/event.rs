use std::{io, os::fd::AsRawFd};
use evdev::{EventType, FetchEventsSynced};
use mio::{Events, Interest, Poll, Token, unix::SourceFd};
use crate::{device::{keyboards::{self, KEYBOARDS, PRESS_STATE}, vmouse::{self, Behavior}}, setup::keymap};


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
    for pressed in events {
        if EventType::KEY != pressed.event_type() {
            continue;
        }

        PRESS_STATE.with_borrow_mut(|states| {
            if let Some(slot) = states.get_mut(pressed.code() as usize) {
                *slot = pressed.value()>0;
                if !*slot {return;}
            }

            let candidates = keymap::load_rvs()
                .get(pressed.code() as usize)
                .unwrap();

            let mut intended:Option<Behavior> = None;
            let mut max_combo_cnt:usize = 0;

            for behavior in candidates {
                // check keybind combo
                let combo = keymap::load_fwd().get(behavior).unwrap();
                if combo.len() <= max_combo_cnt {break;}

                let mut completed = true;
                for required in combo {
                    if !*states.get(*required as usize).unwrap() {
                        completed = false;
                        break;
                    }
                }

                if completed {
                    max_combo_cnt = combo.len();
                    intended = Some(behavior.clone());
                }
            }

            if let Some(todo) = intended {
                vmouse::dispatch(&todo);
            }
        });
    }
}
