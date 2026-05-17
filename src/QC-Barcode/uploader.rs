use serde::Serialize;
use std::sync::OnceLock;
use std::time::Duration;

const OBFUSCATED_URL: &[u8] = b"Z29sLWVkb2NyYWItY3EvaXBhLzAwMDM6bW9jLnJldnJlcy1haWxlci8vOnNwdHRo";

static MACHINE_ID: OnceLock<String> = OnceLock::new();

#[derive(Debug, Serialize)]
struct QcBarcodeLog {
    user: String,
    barcode: String,
    success: u8,
    project_id: u8,
    project_name: String,
    lot_number: u16,
    level: u8,
    error_msg: String,
    input_method: String,
    photo_path: String,
}

/// 解码混淆的 URL
fn decode_url() -> String {
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(OBFUSCATED_URL)
        .unwrap_or_default();
    let s = String::from_utf8_lossy(&decoded);
    s.chars().rev().collect()
}

/// 启动时异步初始化机器 ID
pub fn init() {
    std::thread::spawn(|| {
        let id = get_machine_id_raw();
        let _ = MACHINE_ID.set(id);
    });
}

/// 获取机器唯一标识（跨平台）
fn get_machine_id_raw() -> String {
    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 wmic 获取主板序列号
        if let Ok(output) = std::process::Command::new("wmic")
            .args(["baseboard", "get", "serialnumber"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let serial = stdout
                .lines()
                .skip(1)
                .next()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty() && *s != "To Be Filled By O.E.M.")
                .unwrap_or("");
            if !serial.is_empty() {
                return serial.to_string();
            }
        }
        std::env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 尝试读取 board_serial
        if let Ok(serial) = std::fs::read_to_string("/sys/class/dmi/id/board_serial") {
            let serial = serial.trim();
            if !serial.is_empty() && serial != "To Be Filled By O.E.M." {
                return serial.to_string();
            }
        }
        // fallback: 使用 /etc/machine-id
        if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
            let id = id.trim();
            if !id.is_empty() {
                return id.to_string();
            }
        }
        std::env::var("USER").unwrap_or_else(|_| "unknown".to_string())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        "unknown".to_string()
    }
}

/// 获取机器 ID（读取缓存）
fn get_machine_id() -> &'static str {
    MACHINE_ID.get().map(|s| s.as_str()).unwrap_or("unknown")
}

/// 上传解码日志到服务器
pub fn upload_log(
    barcode: &str,
    success: bool,
    project_id: u8,
    project_name: &str,
    lot_number: u16,
    level: u8,
    error_msg: &str,
) {
    let log = QcBarcodeLog {
        user: get_machine_id().to_string(),
        barcode: barcode.chars().take(6).collect(),
        success: if success { 1 } else { 0 },
        project_id,
        project_name: project_name.to_string(),
        lot_number,
        level,
        error_msg: error_msg.to_string(),
        input_method: "manual".to_string(),
        photo_path: String::new(),
    };

    std::thread::spawn(move || {
        let api_url = decode_url();

        let client = match reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .danger_accept_invalid_certs(true)
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("创建 HTTP 客户端失败: {e}");
                return;
            }
        };

        match client.post(&api_url).json(&log).send() {
            Ok(resp) => {
                if !resp.status().is_success() {
                    eprintln!("上传日志失败: HTTP {}", resp.status());
                }
            }
            Err(e) => eprintln!("上传日志失败: {e}"),
        }
    });
}
