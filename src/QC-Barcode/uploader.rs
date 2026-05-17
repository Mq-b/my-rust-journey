use serde::Serialize;
use std::time::Duration;
use winreg::RegKey;
use winreg::enums::*;

const OBFUSCATED_URL: &[u8] = b"Z29sLWVkb2NyYWItY3EvaXBhLzAwMDM6bW9jLnJldnJlcy1haWxlci8vOnNwdHRo";

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
    // 反转字符串得到原始 URL
    s.chars().rev().collect()
}

/// 获取 Windows MachineGuid 作为硬件唯一标识
pub fn get_machine_id() -> String {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey("SOFTWARE\\Microsoft\\Cryptography") {
        if let Ok(guid) = key.get_value::<String, _>("MachineGuid") {
            return guid;
        }
    }
    // fallback: 使用随机 UUID（每次启动不同）
    uuid::Uuid::new_v4().to_string()
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
        user: get_machine_id(),
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

    // 异步发送，不阻塞 UI
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
