use arrayvec::ArrayVec;
use evdev::{Device, EventType};
use crate::{config::{keymap, vmouse::Mode}, device::{self, DeviceError, DeviceHandler, keyboards::update_physical_presss_state, vmouse::Behavior}};


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

fn inspect_keyboard(keyboard: &mut Device, kbd_idx: usize) -> Result<(), DeviceError>
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

fn emulate_mouse(keyboard: &mut Device, kbd_idx: usize) -> Result<(), DeviceError>
{
    for mut ev in keyboard.fetch_events()? {
        if EventType::KEY != ev.event_type() {continue}

        update_physical_presss_state(ev.code() as usize, ev.value());
        ev = keymap::rewire(ev, kbd_idx);

        let code = ev.code() as usize;
        let value = ev.value();

        device::keyboards::update_logical_press_state(code, value);

        let related_behaviors = match keymap::get_related_behaviors(kbd_idx, code)
            {
                Some(behaviors) => behaviors,
                None => {
                    device::keyboards::pass_through(ev);
                    continue;
                }
            };

        let mut to_dispatch:ArrayVec<Behavior, {Behavior::VAR_COUNT}> = ArrayVec::new();

        if value > 0 { // On keydown
            let mut mode_changed:bool = false;

            for &behavior in related_behaviors.iter() {
                let Some(combo) = keymap::get_combo(kbd_idx, behavior)
                else {continue};

                if device::keyboards::logically_all_pressed(&combo, None) {
                    match behavior {
                        Behavior::LinearModeOn
                        | Behavior::LogarithmicModeOn
                        | Behavior::ScrollModeOn
                        | Behavior::Exit => {
                            mode_changed = true;
                            device::vmouse::mark_active(behavior);
                            to_dispatch.push(behavior.clone());
                        },
                        _ => {
                            if device::vmouse::get_mode() != Mode::Inactive {
                                device::vmouse::mark_active(behavior);
                            }
                        }
                    }
                }
            }

            if !mode_changed {
                to_dispatch.extend( device::vmouse::longest_actives(kbd_idx) );
            }
        }

        else { // On keyup
            for &behavior in related_behaviors.iter() {
                if !device::vmouse::mark_inactive(behavior) {continue}
                if let Some(inv) = behavior.inverse() {
                    to_dispatch.push(inv);
                }
            }
        }

        let mut should_grab = false;
        for behavior in to_dispatch {
            should_grab |= behavior.dispatch()?;
        }

        device::keyboards::update_grab_state(code, value, &mut should_grab);
        if !should_grab {
            device::keyboards::pass_through(ev);
        }
    }

    Ok(())
}
