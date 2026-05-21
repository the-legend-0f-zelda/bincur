use std::io;
use arrayvec::ArrayVec;
use evdev::{Device, EventType};
use crate::{config::keymap, device::{self, DeviceHandler, vmouse::Behavior}};


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
            println!("===================================");
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

        device::keyboards::update_press_state(code, value);

        let related_behaviors = match keymap::get_related_behaviors(kbd_idx, code)
            {
                Some(behaviors) => behaviors,
                None => {
                    device::keyboards::pass_through(ev);
                    continue;
            }
        };

        let mut to_dispatch:ArrayVec<Behavior, 16> = ArrayVec::new();

        if value > 0 { // On keydown
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
                            { device::vmouse::mark_active(behavior); }
                        }
                    }
                }
            }

            to_dispatch.extend( device::vmouse::longest_actives(kbd_idx) );
        }

        else { // On keyup
            for behavior in related_behaviors.iter() {
                if !device::vmouse::mark_inactive(behavior) {continue}
                if let Some(inv) = behavior.inverse() {
                    to_dispatch.push(inv);
                }
            }
        }

        let mut should_grab = false;

        for behavior in to_dispatch {
            should_grab |= behavior.dispatch();
        }

        device::keyboards::update_grab_state(code, value, &mut should_grab);
        if !should_grab {
            device::keyboards::pass_through(ev);
        }
    }

    Ok(())
}
