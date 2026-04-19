use bincur::{device::{keyboards::{KEYBOARDS, PRESS_STATE}, vmouse::{self, Behavior}}, runtime::event::EventDriver, setup::keymap};
use evdev::EventType;

fn main() -> std::io::Result<()> {
    let mut ed = EventDriver::new();
    //keymap::load();

    loop {
        let _r = ed.block_ready();

        for ev in ed.events.iter() {
            let token = ev.token();
            let kbd_idx = token.0;

            KEYBOARDS.with_borrow_mut(|v| {
                let target = &mut v.get_mut(kbd_idx).unwrap().1;
                loop {
                    match target.fetch_events() {
                        Ok(iter) => for pressed in iter {
                            if EventType::KEY != pressed.event_type(){
                                continue;
                            }

                            PRESS_STATE.with_borrow_mut(|states| {
                                // update keypress state
                                if let Some(slot) = states.get_mut(pressed.code() as usize) {
                                    *slot = pressed.value()>0;
                                    if !*slot {return;} // Early return for key release events
                                }

                                // possibly completed combos
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
                                        if !*states.get(*required as usize).unwrap(){
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
                        },
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
