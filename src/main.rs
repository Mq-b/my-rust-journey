#![windows_subsystem = "windows"]
mod barcode;
mod config;

use barcode::{generate_barcode, gray_to_slint_image};
use config::{Config, load_config, save_config};
use rfd::FileDialog;
use std::sync::{Arc, Mutex};

slint::include_modules!();

fn restore_config(window: &BarcodeWindow, cfg: &Config) {
    window.set_content(cfg.content.clone().into());
    window.set_format_index(cfg.format_index as i32);
    window.set_scale_index(cfg.scale_index as i32);
    window.set_rotate_index(cfg.rotate_index as i32);
    window.set_columns_index(cfg.columns_index as i32);
    window.set_eclevel_index(cfg.eclevel_index as i32);
}

fn setup_generate_callback(
    window: &BarcodeWindow,
    last_gray: Arc<Mutex<Option<image::GrayImage>>>,
) {
    let window_weak = window.as_weak();
    window.on_generate(move || {
        let window = window_weak.unwrap();
        let config = Config {
            content: window.get_content().to_string(),
            format_index: window.get_format_index() as usize,
            scale_index: window.get_scale_index() as usize,
            rotate_index: window.get_rotate_index() as usize,
            columns_index: window.get_columns_index() as usize,
            eclevel_index: window.get_eclevel_index() as usize,
        };
        match generate_barcode(&config) {
            Ok(result) => {
                save_config(&config);
                let msg = format!(
                    "{} | {}x{} px | out.png",
                    result.format_name, result.width, result.height
                );
                let slint_img = gray_to_slint_image(&result.gray_image);
                *last_gray.lock().unwrap() = Some(result.gray_image);
                window.set_preview(slint_img);
                window.set_has_preview(true);
                window.set_status(msg.into());
            }
            Err(e) => {
                window.set_status(format!("错误: {}", e).into());
            }
        }
    });
}

fn setup_clipboard_callback(
    window: &BarcodeWindow,
    last_gray: Arc<Mutex<Option<image::GrayImage>>>,
) {
    let window_weak = window.as_weak();
    window.on_copy_to_clipboard(move || {
        let window = window_weak.unwrap();
        let guard = last_gray.lock().unwrap();
        if let Some(gray) = guard.as_ref() {
            let w = gray.width() as usize;
            let h = gray.height() as usize;
            let rgba: Vec<u8> = gray
                .pixels()
                .flat_map(|p| [p[0], p[0], p[0], 255u8])
                .collect();
            let msg = match arboard::Clipboard::new() {
                Ok(mut clipboard) => {
                    let img_data = arboard::ImageData {
                        width: w,
                        height: h,
                        bytes: std::borrow::Cow::Owned(rgba),
                    };
                    match clipboard.set_image(img_data) {
                        Ok(_) => "已复制到剪贴板".to_string(),
                        Err(e) => format!("复制失败: {}", e),
                    }
                }
                Err(e) => format!("剪贴板错误: {}", e),
            };
            window.set_toast_message(msg.into());
            window.set_toast_visible(true);
        }
    });
}

fn setup_export_image_callback(
    window: &BarcodeWindow,
    last_gray: Arc<Mutex<Option<image::GrayImage>>>,
) {
    let window_weak = window.as_weak();

    window.on_export_image(move || {
        let window = window_weak.unwrap();
        let guard = last_gray.lock().unwrap();
        // 文件名为当前日期时间 YYYYMMDD_HHMMSS.png
        let now = chrono::Local::now();
        let filename = format!("barcode_{}.png", now.format("%Y%m%d_%H%M%S"));

        if let Some(gray) = guard.as_ref() {
            let msg = match FileDialog::new()
                .add_filter("PNG Image", &["png"])
                .set_file_name(filename)
                .save_file()
            {
                Some(path) => match gray.save(&path) {
                    Ok(_) => format!("导出成功: {}", path.display()),
                    Err(e) => format!("导出失败: {}", e),
                },
                None => {
                    // 用户取消
                    return;
                }
            };

            window.set_toast_message(msg.into());
            window.set_toast_visible(true);
        } else {
            window.set_toast_message("没有可导出的图像".into());
            window.set_toast_visible(true);
        }
    });
}

fn main() {
    let cfg = load_config();
    let window = BarcodeWindow::new().unwrap();
    restore_config(&window, &cfg);

    let last_gray: Arc<Mutex<Option<image::GrayImage>>> = Arc::new(Mutex::new(None));
    setup_generate_callback(&window, last_gray.clone());
    setup_clipboard_callback(&window, last_gray.clone());
    setup_export_image_callback(&window, last_gray.clone());

    window.run().unwrap();
}
