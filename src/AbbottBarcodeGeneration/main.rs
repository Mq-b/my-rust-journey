#![windows_subsystem = "windows"]
mod abbott;
mod barcode;
mod config;

use abbott::{
    AbbottBarcodeItem, AbbottProjectsConfig, export_abbott_barcodes, generate_abbott_barcodes,
    load_abbott_projects,
};
use barcode::{generate_barcode, gray_to_slint_image, save_png_300dpi};
use config::{Config, load_config, save_config};
use rfd::FileDialog;
use slint::{ModelRc, VecModel};
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
    window.set_abbott_mode(cfg.abbott_mode);
    window.set_abbott_project_index(cfg.abbott_project_index as i32);
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
            abbott_mode: window.get_abbott_mode(),
            abbott_project_index: window.get_abbott_project_index() as usize,
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
                None => return,
            };
            window.set_toast_message(msg.into());
            window.set_toast_visible(true);
        } else {
            window.set_toast_message("没有可导出的图像".into());
            window.set_toast_visible(true);
        }
    });
}

fn default_expiry() -> String {
    let now = chrono::Local::now();
    let next = now + chrono::Months::new(1);
    next.format("%Y-%m-%d").to_string()
}

fn apply_project_defaults(window: &BarcodeWindow, project: &abbott::AbbottProject) {
    window.set_abbott_reagent_count(project.reagents.len() as i32);
    window.set_abbott_control_no(project.control_no_default_number.clone().into());
    // 项目位：取第一个生成长码的试剂的 project_bits
    if let Some(first_long) = project.reagents.iter().find(|r| r.generates_long) {
        window.set_abbott_project_bits(first_long.project_bits.clone().into());
    }
    // 各试剂槽的默认 SN
    let sns: Vec<&str> = project
        .reagents
        .iter()
        .map(|r| r.default_sn.as_str())
        .collect();
    window.set_abbott_sn1(sns.first().copied().unwrap_or("").into());
    window.set_abbott_sn2(sns.get(1).copied().unwrap_or("").into());
    window.set_abbott_sn3(sns.get(2).copied().unwrap_or("").into());
    // 有效期：当前时间+1个月
    window.set_abbott_expiry(default_expiry().into());
}

fn setup_abbott_callbacks(
    window: &BarcodeWindow,
    projects_cfg: Arc<AbbottProjectsConfig>,
    last_abbott: Arc<Mutex<Vec<AbbottBarcodeItem>>>,
) {
    // Project changed → update reagent count, project bits, defaults
    {
        let window_weak = window.as_weak();
        let cfg = projects_cfg.clone();
        window.on_abbott_project_changed(move |idx| {
            let window = window_weak.unwrap();
            if let Some(project) = cfg.projects.get(idx as usize) {
                apply_project_defaults(&window, project);
            }
        });
    }

    // Generate Abbott barcodes
    {
        let window_weak = window.as_weak();
        let cfg = projects_cfg.clone();
        let last = last_abbott.clone();
        window.on_abbott_generate(move || {
            let window = window_weak.unwrap();

            // 每次生成前清空上次结果
            window
                .set_abbott_result_labels(ModelRc::new(VecModel::<slint::SharedString>::default()));
            window.set_abbott_result_contents(ModelRc::new(
                VecModel::<slint::SharedString>::default(),
            ));
            window.set_abbott_result_images(ModelRc::new(VecModel::<slint::Image>::default()));

            let idx = window.get_abbott_project_index() as usize;
            let project = match cfg.projects.get(idx) {
                Some(p) => p,
                None => {
                    window.set_status("未找到项目配置".into());
                    return;
                }
            };

            let sns = vec![
                window.get_abbott_sn1().to_string(),
                window.get_abbott_sn2().to_string(),
                window.get_abbott_sn3().to_string(),
            ];
            let control_no = window.get_abbott_control_no().to_string();
            let expiry = window.get_abbott_expiry().to_string();
            let project_bits = window.get_abbott_project_bits().to_string();

            match generate_abbott_barcodes(project, &sns, &control_no, &expiry, &project_bits) {
                Ok(items) => {
                    let labels: Vec<slint::SharedString> =
                        items.iter().map(|it| it.label.clone().into()).collect();
                    let contents: Vec<slint::SharedString> =
                        items.iter().map(|it| it.content.clone().into()).collect();
                    let images: Vec<slint::Image> =
                        items.iter().map(|it| it.slint_image.clone()).collect();

                    window.set_abbott_result_labels(ModelRc::new(VecModel::from(labels)));
                    window.set_abbott_result_contents(ModelRc::new(VecModel::from(contents)));
                    window.set_abbott_result_images(ModelRc::new(VecModel::from(images)));

                    let count = items.len();
                    *last.lock().unwrap() = items;
                    window.set_status(format!("已生成 {} 个条码", count).into());
                }
                Err(e) => {
                    window.set_status(format!("生成失败: {}", e).into());
                }
            }
        });
    }

    // Copy single barcode content to clipboard
    {
        let window_weak = window.as_weak();
        let last = last_abbott.clone();
        window.on_abbott_copy_content(move |idx| {
            let window = window_weak.unwrap();
            let items = last.lock().unwrap();
            if let Some(item) = items.get(idx as usize) {
                let content = item.content.clone();
                drop(items);
                let msg = match arboard::Clipboard::new() {
                    Ok(mut clipboard) => match clipboard.set_text(&content) {
                        Ok(_) => format!("已复制: {}", &content[..content.len().min(20)]),
                        Err(e) => format!("复制失败: {}", e),
                    },
                    Err(e) => format!("剪贴板错误: {}", e),
                };
                window.set_toast_message(msg.into());
                window.set_toast_visible(true);
            }
        });
    }

    // Export all Abbott barcodes to a folder
    {
        let window_weak = window.as_weak();
        window.on_abbott_export_all(move || {
            let window = window_weak.unwrap();
            let items = last_abbott.lock().unwrap();
            if items.is_empty() {
                window.set_toast_message("没有可导出的条码".into());
                window.set_toast_visible(true);
                return;
            }
            if let Some(dir) = FileDialog::new().pick_folder() {
                let msg = match export_abbott_barcodes(&items, &dir) {
                    Ok(_) => format!("已导出 {} 个文件到: {}", items.len(), dir.display()),
                    Err(e) => format!("导出失败: {}", e),
                };
                window.set_toast_message(msg.into());
                window.set_toast_visible(true);
            }
        });
    }
}

fn setup_menu_callbacks(window: &BarcodeWindow, projects_cfg: Arc<AbbottProjectsConfig>) {
    window.on_quit(|| {
        slint::quit_event_loop().unwrap();
    });

    {
        let window_weak = window.as_weak();
        window.on_reset_config(move || {
            let window = window_weak.unwrap();
            let cfg = Config::default();
            restore_config(&window, &cfg);
            save_config(&cfg);
            window.set_toast_message("配置已重置".into());
            window.set_toast_visible(true);
        });
    }

    window.on_show_about(move || {
        rfd::MessageDialog::new()
            .set_title("关于 Barcode Generator")
            .set_description(
                "Barcode Generator v1.0\n使用 Rust 和 Slint 开发的条码生成工具\n支持多种格式和自定义选项\n作者: Mq-b",
            )
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    });

    // Toggle Abbott mode from menu
    {
        let window_weak = window.as_weak();
        let cfg = projects_cfg.clone();
        window.on_toggle_abbott_mode(move || {
            let window = window_weak.unwrap();
            let new_mode = !window.get_abbott_mode();
            window.set_abbott_mode(new_mode);
            if new_mode {
                let idx = window.get_abbott_project_index() as usize;
                if let Some(project) = cfg.projects.get(idx) {
                    apply_project_defaults(&window, project);
                }
            }
        });
    }
}

fn main() {
    let cfg = load_config();
    let projects_cfg = Arc::new(load_abbott_projects());

    let window = BarcodeWindow::new().unwrap();

    // Populate Abbott project names dropdown
    let project_names: Vec<slint::SharedString> = projects_cfg
        .projects
        .iter()
        .map(|p| p.name.clone().into())
        .collect();
    window.set_abbott_project_names(ModelRc::new(VecModel::from(project_names)));

    // Initialize defaults for the saved project index
    if let Some(project) = projects_cfg.projects.get(cfg.abbott_project_index) {
        apply_project_defaults(&window, project);
    }

    // Empty result models
    window.set_abbott_result_labels(ModelRc::new(VecModel::<slint::SharedString>::default()));
    window.set_abbott_result_contents(ModelRc::new(VecModel::<slint::SharedString>::default()));
    window.set_abbott_result_images(ModelRc::new(VecModel::<slint::Image>::default()));

    restore_config(&window, &cfg);

    let last_gray: Arc<Mutex<Option<image::GrayImage>>> = Arc::new(Mutex::new(None));
    let last_abbott: Arc<Mutex<Vec<AbbottBarcodeItem>>> = Arc::new(Mutex::new(Vec::new()));

    setup_generate_callback(&window, last_gray.clone());
    setup_clipboard_callback(&window, last_gray.clone());
    setup_export_image_callback(&window, last_gray.clone());
    setup_abbott_callbacks(&window, projects_cfg.clone(), last_abbott);
    setup_menu_callbacks(&window, projects_cfg.clone());

    window.run().unwrap();
}
