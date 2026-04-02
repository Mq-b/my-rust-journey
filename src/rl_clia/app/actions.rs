use crate::barcode::{generate_pdf, save_png_with_dpi};
use crate::config;
use crate::layout::PageKind;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use super::generator::dispatch_generate;
use super::helpers::{compute_expire, today_str};
use super::layout_editor::apply_preview_data;
use super::state::{GenerateMode, SharedEditorState};
use super::RLCLIAWindow;

/// 初始化项目名称与项目编号下拉框模型。
pub(super) fn init_project_models(window: &RLCLIAWindow, proj: &config::ProjectConfig) {
    let names: Vec<SharedString> = proj
        .project_name_list
        .iter()
        .map(|s| s.as_str().into())
        .collect();
    let ids: Vec<SharedString> = proj
        .project_id_list
        .iter()
        .map(|s| s.as_str().into())
        .collect();
    window.set_project_names(ModelRc::new(VecModel::from(names)));
    window.set_project_ids(ModelRc::new(VecModel::from(ids)));
}

/// 初始化四类标签的默认生产日期为今天。
pub(super) fn init_default_dates(window: &RLCLIAWindow) {
    let today = today_str();
    window.set_reagent_prod_date(today.clone().into());
    window.set_calib_prod_date(today.clone().into());
    window.set_consumable_prod_date(today.clone().into());
    window.set_quality_prod_date(today.into());
}

/// 绑定失效日期计算回调。
pub(super) fn bind_compute_expiry_callback(window: &RLCLIAWindow) {
    window.on_compute_expiry(|pd, vd| {
        compute_expire(&pd.to_string(), vd.to_string().parse().unwrap_or(365)).into()
    });
}

/// 绑定预览生成回调。
pub(super) fn bind_generate_preview_callback(
    window: &RLCLIAWindow,
    proj: config::ProjectConfig,
    state: SharedEditorState,
) {
    let weak = window.as_weak();
    window.on_generate_preview(move |page| {
        let window = weak.unwrap();
        let page = PageKind::from_ui(&page.to_string());
        let generated = {
            let editor = state.borrow();
            dispatch_generate(page, &window, &proj, &editor.layout, GenerateMode::Preview)
        };
        match generated {
            Ok(result) => {
                apply_preview_data(
                    &window,
                    &state,
                    page,
                    &result.content,
                    Some(&result.preview_barcode),
                );
                window.set_status(format!("{} 预览已生成", result.label).into());
                window.set_toast_msg("预览成功".into());
                window.set_toast_visible(true);
            }
            Err(err) => window.set_status(format!("错误: {err}").into()),
        }
    });
}

/// 绑定单张 PNG 导出回调。
pub(super) fn bind_export_png_callback(
    window: &RLCLIAWindow,
    proj: config::ProjectConfig,
    state: SharedEditorState,
) {
    let weak = window.as_weak();
    window.on_export_png(move |page| {
        let window = weak.unwrap();
        let page = PageKind::from_ui(&page.to_string());
        let generated = {
            let editor = state.borrow();
            dispatch_generate(page, &window, &proj, &editor.layout, GenerateMode::ExportSingle)
        };
        match generated {
            Ok(result) => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("保存PNG图片")
                    .set_file_name(&format!("{}.png", result.label))
                    .add_filter("PNG图片", &["png"])
                    .save_file()
                {
                    let Some(image) = result.images.first() else {
                        window.set_status("保存失败: 生成结果为空".into());
                        return;
                    };
                    match save_png_with_dpi(image, &path) {
                        Ok(_) => {
                            window.set_status(format!("已保存: {}", path.display()).into());
                            window.set_toast_msg("导出成功".into());
                            window.set_toast_visible(true);
                        }
                        Err(err) => window.set_status(format!("保存失败: {err}").into()),
                    }
                }
            }
            Err(err) => window.set_status(format!("错误: {err}").into()),
        }
    });
}

/// 绑定 PDF 导出回调。
pub(super) fn bind_export_pdf_callback(
    window: &RLCLIAWindow,
    proj: config::ProjectConfig,
    state: SharedEditorState,
) {
    let weak = window.as_weak();
    window.on_export_pdf(move |page| {
        let window = weak.unwrap();
        let page = PageKind::from_ui(&page.to_string());
        let generated = {
            let editor = state.borrow();
            dispatch_generate(
                page,
                &window,
                &proj,
                &editor.layout,
                GenerateMode::PersistReagentIds,
            )
        };
        match generated {
            Ok(result) => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("保存PDF")
                    .set_file_name(&format!("{}.pdf", result.label))
                    .add_filter("PDF文件", &["pdf"])
                    .save_file()
                {
                    match generate_pdf(&result.images, path.to_str().unwrap_or("")) {
                        Ok(_) => {
                            window.set_status(format!("已保存: {}", path.display()).into());
                            window.set_toast_msg("导出成功".into());
                            window.set_toast_visible(true);
                        }
                        Err(err) => window.set_status(format!("PDF失败: {err}").into()),
                    }
                }
            }
            Err(err) => window.set_status(format!("错误: {err}").into()),
        }
    });
}
