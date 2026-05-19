use std::io;
use arrayvec::ArrayVec;
use evdev::{Device, EventType};
use crate::{device::{self, DeviceHandler, keyboards::PRESS_STATE, vmouse::{ACTIVATED_SET, Behavior}}, config::keymap};


pub fn determine_handler(options: &Vec<String>) -> (DeviceHandler, bool)
{
    match options.first().map(String::as_str)
    {
        Some("-v") => {
            println!("bincur {}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        },
        Some("-i") => {
            println!("[START] keyboard inspect mode");
            println!("Press ESC to quit");
            (inspect_keyboard, false)
        },
        Some(unknwon) => {
            eprintln!("Unkown option: {}", unknwon);
            std::process::exit(0);
        },
        None => (emulate_mouse, true)
    }
}

fn inspect_keyboard(keyboard: &mut Device, kbd_idx: usize) -> io::Result<()>
{
    let kbd_name = match keyboard.name() {
        Some(name) => name.replace(" ", "_"),
        None => String::from("")
    };

    for ev in keyboard.fetch_events()? {
        if EventType::KEY!=ev.event_type() {continue}

        if ev.code() == 1 {
            println!("[STOP] keyboard inspect mode");
            std::process::exit(0);
        }

        println!("===================================");
        println!("KEYBOARD_NAME: {}", kbd_name);
        println!("KEYBOARD_INDEX: {}", kbd_idx);
        println!("KEY_EVENT: {:#?}", ev);
    }

    Ok(())
}

fn emulate_mouse(keyboard: &mut Device, kbd_idx: usize) -> io::Result<()>
{
    for mut ev in keyboard.fetch_events()? {
        if EventType::KEY != ev.event_type() {continue}

        ev = keymap::rewire(ev, kbd_idx);
        let code = ev.code() as usize;
        let value = ev.value();

        PRESS_STATE.with_borrow_mut(|p_state| {
            match p_state.get_mut(code) {
                Some(slot) => slot.0 = value > 0,
                None => {}
            };
        });

        let Some(related_behaviors) = keymap::get_related_behaviors(kbd_idx, code)
        else {
            device::keyboards::pass_through(ev);
            continue;
        };

        let mut to_dispatch:ArrayVec<Behavior, 16> = ArrayVec::new();

        if value > 0 { // On key down
            for behavior in related_behaviors.iter() {
                let Some(combo) = keymap::get_combo(kbd_idx, behavior)
                else {continue};

                if device::keyboards::all_pressed(&combo) {
                    match *behavior {
                        Behavior::LinearModeOn
                        | Behavior::LogarithmicModeOn
                        | Behavior::ScrollModeOn
                        | Behavior::Exit => {
                            device::vmouse::mark_active(behavior);
                            to_dispatch.push(behavior.clone());
                        },
                        _ => {
                            if device::vmouse::VMOUSE_PROPS
                                .with_borrow(|cfg| cfg.mode) > 0
                            {device::vmouse::mark_active(behavior);}
                        }
                    }
                }
            }

            let mut max_combo_len = 0;
            let mut longest: ArrayVec<Behavior, 16> = ArrayVec::new();

            ACTIVATED_SET.with_borrow(|a_set| {
                for a in a_set.iter() {
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
            });

            to_dispatch.extend(longest);

        }else { // On key up
            for behavior in related_behaviors.iter() {
                if !device::vmouse::mark_inactive(behavior) {continue}
                if let Some(inv) = behavior.inverse() {
                    to_dispatch.push(inv);
                }
            }
        }

        let mut should_grab = false;
        for behavior in to_dispatch {
            // TODO: 디스배치함수에 배열 통째로 넘기고 batch emit, or연산된 값 받게 수정
            should_grab |= behavior.dispatch();
        }

        PRESS_STATE.with_borrow_mut(|p_state| {
            let Some(slot) = p_state.get_mut(code)
            else {return};

            if value > 0 {
                slot.1 = should_grab;
            }else {
                should_grab = slot.1;
                slot.1 = false;
            }
        });

        if !should_grab {device::keyboards::pass_through(ev);}
    }

    Ok(())
}
