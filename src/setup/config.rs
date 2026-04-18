use std::{path::PathBuf, sync::OnceLock};

pub(crate) static CONF_HOME:OnceLock<PathBuf> = OnceLock::new();
pub fn resolve_config_path() -> &'static PathBuf {
    CONF_HOME.get_or_init(|| {
        return std::env::var("BINCUR_CONF_HOME")
            .map(|path| PathBuf::from(path))
            .unwrap_or_else(|_e| {
                let home = std::env::var("HOME").unwrap();
                PathBuf::from(home).join(".config/bincur")
            })
    })
}
