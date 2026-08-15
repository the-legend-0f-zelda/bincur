use std::{cell::RefCell, collections::HashMap, rc::Rc, str::FromStr};
use evdev::InputEvent;

use crate::{
    config::{KEYCODE_MAX, cleaned_uppercase_lines},
    device::{
        keyboards::{self, physically_all_pressed},
        vmouse::Behavior
    }
};


thread_local! {
    /// KEMAP FORWARD[keyboard index]
    /// Behavior -> Combo: Vec<keycode: uszie>
    static KEYMAP_FWD: RefCell<Vec<HashMap<Behavior, Rc<Vec<usize>>>>> = RefCell::new(Vec::new());

    /// KEYMAP REVERSE[keyboard index]
    /// Keycode -> Related Behaviors: Vec<Behavior>
    static KEYMAP_RVS: RefCell<Vec<[Rc<Vec<Behavior>>; KEYCODE_MAX+1]>> = RefCell::new(Vec::new());

    /// REWIRE FORWARD[keyboard index]
    /// Keycode -> Required key combo: Vec<keycode: u16>
    static REWIRE_FWD: RefCell<Vec<[Rc<Vec<usize>>; KEYCODE_MAX+1]>> = RefCell::new(Vec::new());

    /// REWIRE REVERSE[keyboard index]
    /// Keycode -> Related rewire targets: Vec<keycode: u16>
    static REWIRE_RVS: RefCell<Vec<[Rc<Vec<usize>>; KEYCODE_MAX+1]>> = RefCell::new(Vec::new());
}

pub fn initialize() {
    load_cfg_fwd();
    load_cfg_rvs();
    load_rewire_fwd();
    load_rewire_rvs();
}

pub fn load_cfg_fwd() {
    KEYMAP_FWD.with_borrow_mut(|fwd| {
        *fwd = Vec::new();

        for name in keyboards::names() {
            let mut cfg:HashMap<Behavior, Rc<Vec<usize>>> = HashMap::new();

            let lines = match name {
                Some(name) => {
                    let target_file_name = format!("keymap.{}.conf", name.replace(" ", "_"));
                    cleaned_uppercase_lines(target_file_name.as_str(), Some("keymap.conf"))
                },
                None => cleaned_uppercase_lines("keymap.conf", None)
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

pub fn load_cfg_rvs() {
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

pub fn load_rewire_fwd()
{
    REWIRE_FWD.with_borrow_mut(|rewire_fwd| rewire_fwd.clear());

    for name in keyboards::names() {
        let cfg_lines = match name {
            Some(name) => {
                let target_file_name = format!("rewire.{}.conf", name.replace(" ", "_"));
                cleaned_uppercase_lines(target_file_name.as_str(), Some("rewire.conf"))
            },
            None => cleaned_uppercase_lines("rewire.conf", None)
        };

        let mut cfg_fwd: [Rc<Vec<usize>>; KEYCODE_MAX+1] = std::array::from_fn(|_| Rc::new(Vec::new()));

        for line in cfg_lines {
            let kv:Vec<&str> = line.split("->").collect();
            let [f, t] = kv.as_slice() else {
                  eprintln!("invalid rewire line: {}", line);
                  std::process::exit(1);
            };

            let to = evdev::KeyCode::from_str(format!("KEY_{}", t).as_str())
                .unwrap_or_else(|e| {
                    eprintln!("invalid rewire line: {}", line);
                    eprintln!("rewire error : {}", e);
                    std::process::exit(1);
                })
                .code() as usize;

            let mut combo: Vec<usize> = Vec::new();
            for key in f.split("+") {
                let key_code = evdev::KeyCode::from_str(format!("KEY_{}", key).as_str())
                    .unwrap_or_else(|e| {
                        eprintln!("invalid rewire line: {}", line);
                        eprintln!("rewire error : {}", e);
                        std::process::exit(1);
                    })
                    .code() as usize;

                combo.push(key_code);
            }

            cfg_fwd[to] = Rc::new(combo);
        }

        REWIRE_FWD.with_borrow_mut(|rewire_fwd| rewire_fwd.push(cfg_fwd));
    }
}

pub fn load_rewire_rvs()
{
    REWIRE_RVS.with_borrow_mut(|rewire_rvs| rewire_rvs.clear());

    for cfg_fwd in REWIRE_FWD.with_borrow(|rewire_fwd| rewire_fwd.clone()) {
        let mut reversed: [Vec<usize>; KEYCODE_MAX+1] = std::array::from_fn(|_| Vec::new());

        for keycode in 0..KEYCODE_MAX {
            let related_combo = Rc::clone(&cfg_fwd[keycode]);
            for &key in &*related_combo {
                reversed[key as usize].push(keycode);
            }
        }

        let cfg_rvs: [Rc<Vec<usize>>; KEYCODE_MAX+1] = std::array::from_fn(|idx| {
            Rc::new( std::mem::take(&mut reversed[idx]) )
        });

        REWIRE_RVS.with_borrow_mut(|rewire_rvs| rewire_rvs.push(cfg_rvs));
    }
}

pub fn get_combo(kbd_idx:usize, behavior:&Behavior) -> Option<Rc<Vec<usize>>>
{
    KEYMAP_FWD.with_borrow(|fwd| {
        let dev_cfg = fwd.get(kbd_idx).unwrap();
        match dev_cfg.get(behavior) {
            Some(combo) => Some(Rc::clone(combo)),
            None => None
        }
    })
}

pub fn get_related_behaviors(kbd_idx:usize, key_code:usize) -> Option<Rc<Vec<Behavior>>>
{
    KEYMAP_RVS.with_borrow(|rvs| {
        let dev_cfg = rvs.get(kbd_idx).unwrap();
        match dev_cfg.get(key_code) {
            Some(behaviors) => Some(Rc::clone(behaviors)),
            None => None
        }
    })
}

pub fn rewire(origin: InputEvent, dev_idx:usize) -> InputEvent
{
    let original_code = origin.code() as usize;

    if let Some(rel_targets) = REWIRE_RVS.with_borrow(|rvs|
        rvs.get(dev_idx)?.get(original_code).map(Rc::clone)
    ) {
        for &target in rel_targets.iter() {
            let Some(combo) = REWIRE_FWD.with_borrow(|fwd|
                fwd.get(dev_idx)?.get(target).map(Rc::clone)
            ) else { continue };

            if physically_all_pressed(&combo, Some(original_code)) {
                return InputEvent::new(
                    origin.event_type().0,
                    target as u16,
                    origin.value()
                )
            }
        }
    }

    origin
}
