use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, str::FromStr, sync::OnceLock};
use crate::setup::config::{resolve_config_path};

pub static KEYMAP:OnceLock<HashMap<String, Vec<u16>>> = OnceLock::new();

pub fn parse_keymap() {
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
        let behavior = kv[0];
        let inputs:Vec<u16> = kv[1].split("+").map(|i| {
            let key = format!("KEY_{}", i);
            let key_code = evdev::KeyCode::from_str(key.as_str()).unwrap();
            return key_code.0
        }).collect();

        tmp.insert(String::from(behavior), inputs);
    }

    KEYMAP.get_or_init(|| {tmp});
}
