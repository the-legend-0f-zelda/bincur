pub mod vmouse;
pub mod keymap;

use std::{fs::File, io::{BufRead, BufReader}, path::PathBuf, sync::OnceLock};

pub const KEYCODE_MAX:usize = 248;

pub static CONF_HOME:OnceLock<PathBuf> = OnceLock::new();
pub fn resolve_path() -> &'static PathBuf {
    CONF_HOME.get_or_init(|| {
        return std::env::var("BINCUR_CONF_HOME")
            .map(|path| PathBuf::from(path))
            .unwrap_or_else(|_e| {
                let home = std::env::var("HOME").unwrap();
                PathBuf::from(home).join(".config/bincur")
            })
    })
}

pub fn cleaned_lines(conf_name:&str, default_name: Option<&str>) -> Vec<String> {
    let mut result:Vec<String> = Vec::new();

    let file = match File::open(resolve_path().join(conf_name)) {
        Ok(file) => file,
        Err(_) => {
            let Some(fallback) = default_name
                else {return result};
            return cleaned_lines(fallback, None)
        }
    };

    let mut lines = BufReader::new(file).lines();
    for line in &mut lines {
        let cleaned = line.unwrap().replace(" ", "").to_uppercase();
        if cleaned.is_empty() || cleaned.starts_with("#") {
            continue;
        }
        result.push(cleaned);
    }

    result
}
