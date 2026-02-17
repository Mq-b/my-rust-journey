use std::fs;
use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub content: String,
    pub format_index: usize,
    pub scale_index: usize,
    pub rotate_index: usize,
    pub columns_index: usize,
    pub eclevel_index: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            content: "12345678".to_string(),
            format_index: 0,
            scale_index: 1,   // 2x
            rotate_index: 0,  // 0°
            columns_index: 1, // 列数 2
            eclevel_index: 2, // 纠错等级 2（PDF417 默认）
        }
    }
}

fn config_path() -> PathBuf {
    PathBuf::from("./assets/barcode_config.json")
}

pub fn load_config() -> Config {
    fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_config(config: &Config) {
    if let Ok(data) = serde_json::to_string_pretty(config) {
        let _ = fs::write(config_path(), data);
    }
}
