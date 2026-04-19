use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, str::FromStr, sync::OnceLock};
use crate::setup::config::{resolve_config_path};

pub static KEYMAP:OnceLock<HashMap<String, Vec<u16>>> = OnceLock::new();

pub fn load() {
    let keymap_file:File = File::open(
        resolve_config_path().join("keymap.conf")
    )
    .unwrap();

    let mut tmp:HashMap<String, Vec<u16>> = HashMap::new();

    for line in BufReader::new(keymap_file).lines(){
        let line = line.unwrap();
        let cleaned = line.replace(" ", "");

        if cleaned.is_empty() || cleaned.starts_with("#") {
            continue;
        }

        let kv:Vec<&str> = cleaned.split(':').collect();
        let behavior = kv[1];
        let inputs:Vec<u16> = kv[0].split("+").map(|i| {
            let key = format!("KEY_{}", i.to_uppercase());
            let key_code = evdev::KeyCode::from_str(key.as_str()).unwrap();
            return key_code.0
        }).collect();

        tmp.insert(String::from(behavior), inputs);
    }

    println!("keymappings: {:#?}", tmp);
    KEYMAP.get_or_init(|| {tmp});
}
