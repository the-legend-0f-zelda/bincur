use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc, str::FromStr, sync::OnceLock};
use evdev::InputEvent;

use crate::{device::{keyboards::{self, KEYBOARDS}, vmouse::Behavior}, setup::{config::{self, cleaned_lines}, keymap}};

pub const KEYCODE_MAX:usize = 248;

thread_local! {
    static KEYMAP_FWD: RefCell<Vec<HashMap<Behavior, Rc<Vec<usize>>>>> = RefCell::new(Vec::new());
    static KEYMAP_RVS: RefCell<Vec<[Rc<Vec<Behavior>>; KEYCODE_MAX+1]>> = RefCell::new(Vec::new());
    static REWIRE_CFG: RefCell<Vec<[u16; KEYCODE_MAX+1]>> = RefCell::new(Vec::new());
}

pub fn initialize() {
    load_cfg_fwd();
    load_cfg_rvs();
    load_cfg_rewire();
}

pub fn load_cfg_fwd() //-> &'static HashMap<Behavior, Vec<usize>>
{
    KEYMAP_FWD.with_borrow_mut(|fwd| {
        *fwd = Vec::new();

        for name in keyboards::names() {
            let mut cfg:HashMap<Behavior, Rc<Vec<usize>>> = HashMap::new();

            let lines = match name {
                Some(name) => {
                    let target_file_name = format!("keymap.{}.conf", name.replace(" ", "_"));
                    config::cleaned_lines(target_file_name.as_str(), Some("keymap.conf"))
                },
                None => config::cleaned_lines("keymap.conf", None)
            };

            for line in lines {
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

                cfg.insert(Behavior::from_str(b), Rc::new(inputs));
            }

            fwd.push(cfg);
        }
    });
}

pub fn load_cfg_rvs() //-> &'static [Vec<Behavior>]
{
    KEYMAP_RVS.with_borrow_mut(|rvs| *rvs = Vec::new());

    for (_kbd_idx, keymap) in KEYMAP_FWD.with_borrow(|fwd| fwd.clone())
        .iter().enumerate()
    {
        let mut flat:HashMap<usize, Vec<Behavior>> = HashMap::new();
        for (behavior, inputs) in keymap {
            for key_code in inputs.iter() {
                match flat.get_mut(key_code) {
                    Some(slot) => slot.push(behavior.clone()),
                    None => {
                        let mut new_slot = Vec::new();
                        new_slot.push(behavior.clone());
                        flat.insert(*key_code, new_slot);
                    }
                }
            }
        }

        let reversed:[Rc<Vec<Behavior>>; KEYCODE_MAX+1] = std::array::from_fn(|idx| {
            match flat.remove(&idx) {
                Some(related) => Rc::new(related),
                None => Rc::new(Vec::new())
            }
        });

        KEYMAP_RVS.with_borrow_mut(|rvs| rvs.push(reversed));
    }
}

pub fn load_cfg_rewire() {
    for name in keyboards::names() {
        let cfg_lines = match name {
            Some(name) => {
                let target_file_name = format!("rewire.{}.conf", name.replace(" ", "_"));
                config::cleaned_lines(target_file_name.as_str(), Some("rewire.conf"))
            },
            None => config::cleaned_lines("rewire.conf", None)
        };

        let mut cfg: [u16; KEYCODE_MAX+1] = [0; KEYCODE_MAX+1];
        for i in 0..KEYCODE_MAX {
            cfg[i] = i as u16;
        }

        for line in cfg_lines {
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

            cfg[from] = to;
        }

        REWIRE_CFG.with_borrow_mut(|rewires| {rewires.push(cfg);})
    }
}

pub fn get_combo(kbd_idx:usize, behavior:&Behavior) -> Option<Rc<Vec<usize>>> {
    KEYMAP_FWD.with_borrow(|fwd| {
        let dev_cfg = fwd.get(kbd_idx).unwrap();
        match dev_cfg.get(behavior) {
            Some(combo) => Some(Rc::clone(combo)),
            None => None
        }
    })
}

pub fn get_related_behaviors(kbd_idx:usize, key_code:usize) -> Option<Rc<Vec<Behavior>>> {
    KEYMAP_RVS.with_borrow(|rvs| {
        let dev_cfg = rvs.get(kbd_idx).unwrap();
        match dev_cfg.get(key_code) {
            Some(behaviors) => Some(Rc::clone(behaviors)),
            None => None
        }
    })
}

pub fn rewire(origin: InputEvent, dev_idx:usize) -> InputEvent {
    let key = origin.code();

    let key_altered = REWIRE_CFG.with_borrow(|r| {
        *r[dev_idx].get(key as usize)
            .unwrap_or(&key)
    });

    InputEvent::new(
        origin.event_type().0,
        key_altered,
        origin.value()
    )
}
