use evdev::{Device, EventType, InputEvent};

use crate::{device::{self, keyboards::PRESS_STATE, vmouse::{ACTIVATED_SET, Behavior}}, setup::keymap};


pub(crate) fn determine_handler(options: &Vec<String>) -> (fn(&Device, usize, Vec<InputEvent>), bool) {
    match options.get(0)
        .unwrap_or(&String::from(""))
        .as_str()
    {
        "-v" => {
            println!("bincur {}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        },
        "-i" => {
            println!("[START] keyboard inspect mode");
            println!("Press ESC to quit");
            (inspect_keyboard, false)
        },
        _ => (emulate_mouse, true)
    }
}

fn inspect_keyboard(keyboard: &Device, kbd_idx: usize, events: Vec<InputEvent>) {
    for ev in events {
        if EventType::KEY!=ev.event_type() {continue}

        if ev.code() == 1 {
            println!("[STOP] keyboard inspect mode");
            std::process::exit(0);
        }

        let kbd_name = match keyboard.name() {
            Some(name) => name.replace(" ", "_"),
            None => String::from("")
        };

        println!("KEYBOARD_NAME: {}", kbd_name);
        println!("KEYBOARD_INDEX: {}", kbd_idx);
        println!("KEY_EVENT: {:#?}", ev);
    }
}

fn emulate_mouse(_keyboard: &Device, kbd_idx: usize, events: Vec<InputEvent>) {
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
