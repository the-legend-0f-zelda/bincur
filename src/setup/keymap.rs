use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, str::FromStr, sync::OnceLock};
use crate::{device::vmouse::Behavior, setup::config};

static KEYMAP_FWD:OnceLock<HashMap<Behavior, Vec<u16>>> = OnceLock::new();
static KEYMAP_RVS:OnceLock<[Vec<Behavior>; 249]> = OnceLock::new();

pub fn load_fwd() -> &'static HashMap<Behavior, Vec<u16>> {
    KEYMAP_FWD.get_or_init(|| {
        let keymap_file:File = File::open(
            config::resolve_path().join("keymap.conf")
        ).unwrap();

        let mut tmp:HashMap<Behavior, Vec<u16>> = HashMap::new();

        for line in BufReader::new(keymap_file).lines(){
            let line = line.unwrap();
            let cleaned = line.replace(" ", "");

            if cleaned.is_empty() || cleaned.starts_with("#") {
                continue;
            }

            let kv:Vec<&str> = cleaned.split(':').collect();
            let [i, b] = kv.as_slice() else {
                  eprintln!("invalid keymap line: {}", cleaned);
                  std::process::exit(1);
            };

            let inputs:Vec<u16> = i.split('+').map(|i| {
                let key = format!("KEY_{}", i.to_uppercase());
                let key_code = evdev::KeyCode::from_str(key.as_str())
                    .unwrap_or_else(|e| {
                        eprintln!("invalid keymap line: {}", cleaned);
                        eprintln!("keymap error : {}", e);
                        std::process::exit(1);
                    });

                key_code.0
            }).collect();

            tmp.insert(Behavior::from_str(b), inputs);
        }
        println!("keymap: {:#?}", tmp);
        tmp
    })
}

pub fn load_rvs() -> &'static [Vec<Behavior>] {
    KEYMAP_RVS.get_or_init(|| {
        let mut tmp:[Vec<Behavior>; 249] = std::array::from_fn(|_| Vec::new());

        for (behavior, inputs) in load_fwd() {
            for i in inputs {
                if let Some(slot) = tmp.get_mut(*i as usize) {
                    slot.push(behavior.clone());
                }
            }
        }

        tmp
    })
}
