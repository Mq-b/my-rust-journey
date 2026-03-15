use crate::codec::{decode, encode};
use crate::imaging::{copy_gray_to_clipboard, gray_to_slint, make_datamatrix};
use crate::LotIdWindow;
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};

type LastGray = Arc<Mutex<Option<image::GrayImage>>>;

pub fn setup_callbacks(window: &LotIdWindow, last_gray: LastGray) {
    setup_encode_callback(window, last_gray.clone());
    setup_decode_callback(window);
    setup_copy_text_callback(window);
    setup_copy_image_callback(window, last_gray.clone());
    setup_save_image_callback(window, last_gray);
}

fn setup_encode_callback(window: &LotIdWindow, last_gray: LastGray) {
    let window_weak = window.as_weak();
    window.on_do_encode(move || {
        let window = window_weak.unwrap();

        let parse_u8_field = |s: slint::SharedString, name: &str| -> Result<u8, String> {
            let v: u32 = s
                .trim()
                .parse()
                .map_err(|_| format!("{name} 格式错误，请输入 0-255 的整数"))?;
            if v > 255 {
                return Err(format!("{name} 超出范围 (0-255)"));
            }
            Ok(v as u8)
        };

        let id = match parse_u8_field(window.get_id_input(), "ID") {
            Ok(v) => v,
            Err(msg) => {
                window.set_status(msg.into());
                return;
            }
        };
        let lot = match parse_u8_field(window.get_lot_input(), "Lot") {
            Ok(v) => v,
            Err(msg) => {
                window.set_status(msg.into());
                return;
            }
        };

        let chars = encode(id, lot);
        let code: String = chars.iter().collect();
        window.set_code_result(code.clone().into());

        match make_datamatrix(&code) {
            Ok(gray) => {
                window.set_preview(gray_to_slint(&gray));
                window.set_has_preview(true);
                *last_gray.lock().unwrap() = Some(gray);
                window.set_status(format!("编码成功: ID={id} Lot={lot} → {code}").into());
            }
            Err(e) => window.set_status(format!("条码生成失败: {e}").into()),
        }
    });
}

fn setup_decode_callback(window: &LotIdWindow) {
    let window_weak = window.as_weak();
    window.on_do_decode(move || {
        let window = window_weak.unwrap();

        let input = window.get_decode_input().to_string();
        let chars: Vec<char> = input.chars().collect();

        if chars.len() != 3 {
            window.set_status(format!("请输入恰好3个字符（当前 {} 个）", chars.len()).into());
            return;
        }

        let [ch1, ch2, ch3] = [chars[0], chars[1], chars[2]];

        match decode([ch1, ch2, ch3]) {
            Some((id, lot)) => {
                window.set_decode_id(id.to_string().into());
                window.set_decode_lot(lot.to_string().into());
                let code: String = [ch1, ch2, ch3].iter().collect();
                window.set_status(format!("解码成功: {code} → ID={id} Lot={lot}").into());
            }
            None => {
                window.set_decode_id("".into());
                window.set_decode_lot("".into());
                window.set_status("解码失败：包含无效字符".into());
            }
        }
    });
}

fn setup_copy_text_callback(window: &LotIdWindow) {
    let window_weak = window.as_weak();
    window.on_copy_text(move || {
        let window = window_weak.unwrap();
        let code = window.get_code_result().to_string();
        if code.is_empty() {
            return;
        }
        let msg = match arboard::Clipboard::new() {
            Ok(mut cb) => match cb.set_text(&code) {
                Ok(_) => format!("已复制文字: {code}"),
                Err(e) => format!("复制失败: {e}"),
            },
            Err(e) => format!("剪贴板错误: {e}"),
        };
        window.set_status(msg.into());
    });
}

fn setup_copy_image_callback(window: &LotIdWindow, last_gray: LastGray) {
    let window_weak = window.as_weak();
    window.on_copy_image(move || {
        let window = window_weak.unwrap();
        let guard = last_gray.lock().unwrap();
        if let Some(gray) = guard.as_ref() {
            let msg = match copy_gray_to_clipboard(gray) {
                Ok(_) => "图片已复制到剪贴板".to_string(),
                Err(e) => format!("复制失败: {e}"),
            };
            window.set_status(msg.into());
        }
    });
}

fn setup_save_image_callback(window: &LotIdWindow, last_gray: LastGray) {
    let window_weak = window.as_weak();
    window.on_save_image(move || {
        let window = window_weak.unwrap();
        let guard = last_gray.lock().unwrap();
        if let Some(gray) = guard.as_ref() {
            let code = window.get_code_result().to_string();
            let filename = if code.is_empty() {
                "lotid.png".to_string()
            } else {
                format!("{}.png", code)
            };
            let msg = match rfd::FileDialog::new()
                .add_filter("PNG Image", &["png"])
                .set_file_name(filename)
                .save_file()
            {
                Some(path) => match gray.save(&path) {
                    Ok(_) => format!("已保存: {}", path.display()),
                    Err(e) => format!("保存失败: {e}"),
                },
                None => return,
            };
            window.set_status(msg.into());
        }
    });
}
