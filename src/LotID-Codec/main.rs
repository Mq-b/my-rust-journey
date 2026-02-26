#![windows_subsystem = "windows"]

slint::include_modules!();

/// 43字符编码表（与 C 原版 s_barcodeTable 完全一致）
const TABLE: [char; 43] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '-', '+',
    '/', '$', '.', '%', ' ',
];

fn encode(id: u8, lot: u8) -> [char; 3] {
    let mut val = ((lot as u32) << 8) | (id as u32);
    val ^= 0xE19A;
    let mut out = [' '; 3];
    for c in &mut out {
        let remainder = (val % 43) as usize;
        val /= 43;
        *c = TABLE[remainder];
    }
    out
}

fn decode(chars: [char; 3]) -> Option<(u8, u8)> {
    let mut indices = [0u32; 3];
    for (i, ch) in chars.iter().enumerate() {
        let idx = TABLE.iter().position(|&t| t == ch.to_ascii_uppercase())?;
        indices[i] = idx as u32;
    }
    let val = (indices[0] + indices[1] * 43 + indices[2] * 43 * 43) ^ 0xE19A;
    Some(((val & 0xFF) as u8, ((val >> 8) & 0xFF) as u8))
}

fn make_datamatrix(text: &str) -> anyhow::Result<image::GrayImage> {
    use zxingcpp::*;
    let barcode = create(BarcodeFormat::DataMatrix).from_str(text)?;
    let img = barcode.to_image_with(&write().scale(6).add_quiet_zones(true).add_hrt(false))?;
    Ok(image::GrayImage::from(&img))
}

fn gray_to_slint(gray: &image::GrayImage) -> slint::Image {
    let (w, h) = (gray.width(), gray.height());
    let pixels: Vec<u8> = gray
        .pixels()
        .flat_map(|p| [p[0], p[0], p[0], 255u8])
        .collect();
    let buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(&pixels, w, h);
    slint::Image::from_rgba8(buf)
}

fn copy_gray_to_clipboard(gray: &image::GrayImage) -> Result<(), String> {
    let rgba: Vec<u8> = gray
        .pixels()
        .flat_map(|p| [p[0], p[0], p[0], 255u8])
        .collect();
    arboard::Clipboard::new()
        .map_err(|e| e.to_string())?
        .set_image(arboard::ImageData {
            width: gray.width() as usize,
            height: gray.height() as usize,
            bytes: std::borrow::Cow::Owned(rgba),
        })
        .map_err(|e| e.to_string())
}

// ── 回调注册函数 ──────────────────────────────────────────────────────────────

fn setup_encode_callback(
    window: &LotIdWindow,
    last_gray: std::sync::Arc<std::sync::Mutex<Option<image::GrayImage>>>,
) {
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

fn setup_copy_image_callback(
    window: &LotIdWindow,
    last_gray: std::sync::Arc<std::sync::Mutex<Option<image::GrayImage>>>,
) {
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

fn setup_save_image_callback(
    window: &LotIdWindow,
    last_gray: std::sync::Arc<std::sync::Mutex<Option<image::GrayImage>>>,
) {
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

fn main() {
    let window = LotIdWindow::new().unwrap();
    let last_gray: std::sync::Arc<std::sync::Mutex<Option<image::GrayImage>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));

    setup_encode_callback(&window, last_gray.clone());
    setup_decode_callback(&window);
    setup_copy_text_callback(&window);
    setup_copy_image_callback(&window, last_gray.clone());
    setup_save_image_callback(&window, last_gray);

    window.run().unwrap();
}
