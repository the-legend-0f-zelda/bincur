use std::{collections::HashMap, str::FromStr, sync::OnceLock};
use evdev::InputEvent;

use crate::{device::vmouse::Behavior, setup::config};

pub const KEYCODE_MAX:usize = 248;
static KEYMAP_FWD:OnceLock<HashMap<Behavior, Vec<usize>>> = OnceLock::new();
static KEYMAP_RVS:OnceLock<[Vec<Behavior>; KEYCODE_MAX+1]> = OnceLock::new();
static REWIRE_CFG:OnceLock<[u16; KEYCODE_MAX+1]> = OnceLock::new();

pub fn load_fwd() -> &'static HashMap<Behavior, Vec<usize>> {
    KEYMAP_FWD.get_or_init(|| {
        let mut tmp:HashMap<Behavior, Vec<usize>> = HashMap::new();

        for line in config::cleaned_lines("keymap.conf") {
            let kv:Vec<&str> = line.split(':').collect();
            let [i, b] = kv.as_slice() else {
                  eprintln!("invalid keymap line: {}", line);
                  std::process::exit(1);
            };

            let inputs:Vec<usize> = i.split('+').map(|i| {
                let key = format!("KEY_{}", i);
                let key_code = evdev::KeyCode::from_str(key.as_str())
                    .unwrap_or_else(|e| {
                        eprintln!("invalid keymap line: {}", line);
                        eprintln!("keymap error : {}", e);
                        std::process::exit(1);
                    });

                key_code.0 as usize
            }).collect();

            tmp.insert(Behavior::from_str(b), inputs);
        }

        tmp
    })
}

pub fn load_rvs() -> &'static [Vec<Behavior>] {
    KEYMAP_RVS.get_or_init(|| {
        let mut tmp:[Vec<Behavior>; KEYCODE_MAX+1] = std::array::from_fn(|_| Vec::new());

        for (behavior, inputs) in load_fwd() {
            for &i in inputs {
                if let Some(slot) = tmp.get_mut(i) {
                    slot.push(behavior.clone());
                }
            }
        }

        tmp
    })
}

pub fn rewire(origin: InputEvent) -> InputEvent {
    let cfg = REWIRE_CFG.get_or_init(|| {
        let mut tmp: [u16; KEYCODE_MAX+1] = [0; KEYCODE_MAX+1];
        for i in 0..KEYCODE_MAX {
            tmp[i] = i as u16;
        }

        for line in config::cleaned_lines("rewire.conf") {
            let kv:Vec<&str> = line.split("->").collect();
            let [f, t] = kv.as_slice() else {
                  eprintln!("invalid rewire line: {}", line);
                  std::process::exit(1);
            };

            let from = evdev::KeyCode::from_str(format!("KEY_{}", f).as_str())
                .unwrap_or_else(|e| {
                    eprintln!("invalid rewire line: {}", line);
                    eprintln!("rewire error : {}", e);
                    std::process::exit(1);
                })
                .code() as usize;

            let to = evdev::KeyCode::from_str(format!("KEY_{}", t).as_str())
                .unwrap_or_else(|e| {
                    eprintln!("invalid rewire line: {}", line);
                    eprintln!("rewire error : {}", e);
                    std::process::exit(1);
                })
                .code();

            tmp[from] = to;
        }

        tmp
    });

    InputEvent::new(
        origin.event_type().0,
        cfg[origin.code() as usize],
        origin.value()
    )
}
