#![windows_subsystem = "windows"]
mod barcode;
mod config;

use barcode::{generate_barcode, gray_to_slint_image, save_png_300dpi};
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
    window.set_width_cm(format!("{}", cfg.width_cm).into());
    window.set_height_cm(format!("{}", cfg.height_cm).into());
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
            width_cm: window.get_width_cm().parse::<f32>().unwrap_or(0.0),
            height_cm: window.get_height_cm().parse::<f32>().unwrap_or(0.0),
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
                Some(path) => match save_png_300dpi(gray, &path) {
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

fn setup_menu_callbacks(window: &BarcodeWindow) {
    window.on_quit(|| {
        slint::quit_event_loop().unwrap();
    });

    let window_weak = window.as_weak();
    window.on_reset_config(move || {
        let window = window_weak.unwrap();
        let cfg = Config::default();
        restore_config(&window, &cfg);
        save_config(&cfg);
        window.set_toast_message("配置已重置".into());
        window.set_toast_visible(true);
    });

    //let window_weak = window.as_weak();
    window.on_show_about(move || {
        // rfd 显示美观弹窗提示软件信息
        rfd::MessageDialog::new()
            .set_title("关于 Barcode Generator")
            .set_description("Barcode Generator v1.0\n使用 Rust 和 Slint 开发的条码生成工具\n支持多种格式和自定义选项\n作者: Mq-b\n".to_string())
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
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
    setup_menu_callbacks(&window);

    window.run().unwrap();
}
