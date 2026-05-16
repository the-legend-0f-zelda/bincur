use evdev::{EventType, FetchEventsSynced};

use crate::{device::{self, keyboards::{KEYBOARDS, PRESS_STATE}, vmouse::{ACTIVATED_SET, Behavior}}, setup::keymap};


pub(crate) fn determine_handler(options: &Vec<String>) -> fn(usize, FetchEventsSynced) {
    match options.get(0)
        .unwrap_or(&String::from(" "))
        .as_str()
    {
        "-i" => inspect_keyboard,
        "-v" => print_version,
        _ => emulate_mouse
    }
}

fn print_version(_kbd_idx: usize, _events:FetchEventsSynced) {
    println!("bincur {}", env!("CARGO_PKG_VERSION"));
    std::process::exit(0);
}

fn emulate_mouse(kbd_idx: usize, events: FetchEventsSynced) {
    for mut ev in events {
        if EventType::KEY != ev.event_type() {continue}

        ev = keymap::rewire(ev, kbd_idx);
        let code = ev.code() as usize;
        let value = ev.value();

        PRESS_STATE.with_borrow_mut(|states| {
            match states.get_mut(code) {
                Some(slot) => slot.0 = value > 0,
                None => return
            };
        });

        let Some(related_behaviors) = keymap::get_related_behaviors(kbd_idx, code)
        else {
            device::keyboards::pass_through(ev);
            continue;
        };

        let mut to_dispatch:Vec<Behavior> = Vec::new();

        ACTIVATED_SET.with_borrow_mut(|active| {
            if value > 0 { // On key down
                for behavior in related_behaviors.iter() {
                    let Some(combo) = keymap::get_combo(kbd_idx, behavior)
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
                            let len = match keymap::get_combo(kbd_idx, a) {
                                Some(combo) => combo.len(),
                                None => 0
                            };
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
                for behavior in related_behaviors.iter() {
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

        if !grab {device::keyboards::pass_through(ev);}
    }
}

fn inspect_keyboard(kbd_idx: usize, events: FetchEventsSynced) {
    println!("[START] keyboard inspect mode");
    println!("Press ESC to quit");

    for ev in events {
        if EventType::KEY!=ev.event_type() ||ev.code()==0 {
            continue
        }

        if ev.code() == 1 {
            println!("[STOP] keyboard inspect mode");
            std::process::exit(0);
        }

        let kbd_name = KEYBOARDS.with_borrow(|kbds| {
            match kbds.get(kbd_idx) {
                Some((_path, device)) => device.name().unwrap_or("").to_string(),
                None => String::from("")
            }
        });

        println!("KEYBOARD_NAME: {}", kbd_name.replace(" ", "_"));
        println!("KEY_EVENT: {:#?}", ev);
    }
}
