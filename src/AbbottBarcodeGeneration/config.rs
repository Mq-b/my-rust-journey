use std::fs;
use std::path::PathBuf;

fn default_width_cm() -> f32 {
    5.0
}
fn default_height_cm() -> f32 {
    2.0
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub content: String,
    pub format_index: usize,
    pub scale_index: usize,
    pub rotate_index: usize,
    pub columns_index: usize,
    pub eclevel_index: usize,
    #[serde(default = "default_width_cm")]
    pub width_cm: f32,
    #[serde(default = "default_height_cm")]
    pub height_cm: f32,
    #[serde(default)]
    pub abbott_mode: bool,
    #[serde(default)]
    pub abbott_project_index: usize,
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
            width_cm: 0.0,
            height_cm: 0.0,
            abbott_mode: false,
            abbott_project_index: 0,
        }
    }
}

fn config_path() -> PathBuf {
    PathBuf::from("./assets/barcode_config.json")
}

pub fn load_config() -> Config {
    let path = config_path();

    // 确保 assets 目录存在
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // 读取并解析，失败则写入默认配置
    match fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
    {
        Some(config) => config,
        None => {
            let config = Config::default();
            save_config(&config);
            config
        }
    }
}

pub fn save_config(config: &Config) {
    if let Ok(data) = serde_json::to_string_pretty(config) {
        let _ = fs::write(config_path(), data);
    }
}
