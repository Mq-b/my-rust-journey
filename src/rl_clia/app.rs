use crate::config;
use crate::layout::{
    load_layout_config, save_layout_config, PageKind, LABEL_HEIGHT_PX, LABEL_WIDTH_PX,
};
use slint::{ModelRc, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

mod actions;
mod decrypt;
mod generator;
mod helpers;
mod layout_editor;
mod state;

use actions::{
    bind_compute_expiry_callback, bind_export_pdf_callback, bind_export_png_callback,
    bind_generate_preview_callback, init_default_dates, init_project_models,
};
use decrypt::bind_decrypt_callbacks;
use layout_editor::{bind_layout_editor_callbacks, refresh_editor_for_page};
use state::{EditorState, SharedEditorState};

slint::include_modules!();

/// 化学发光条码工具的应用入口。
///
/// 本模块只负责组装窗口、初始化状态并绑定回调；
/// 具体业务实现已拆分到 `app/*` 子模块。
pub fn run() {
    let proj = config::load_project_config();
    let window = RLCLIAWindow::new().expect("创建窗口失败");

    let layout = load_layout_config();
    let _ = save_layout_config(&layout);

    let preview_model = Rc::new(VecModel::from(Vec::<PreviewElementData>::new()));
    let state: SharedEditorState = Rc::new(RefCell::new(EditorState {
        layout,
        preview_model: preview_model.clone(),
        selected: None,
        active_page: PageKind::Reagent,
    }));

    window.set_preview_elements(preview_model.into());
    window.set_decode_fields(ModelRc::new(VecModel::from(Vec::<DecodeFieldData>::new())));
    window.set_label_width_px(LABEL_WIDTH_PX as i32);
    window.set_label_height_px(LABEL_HEIGHT_PX as i32);

    init_project_models(&window, &proj);
    init_default_dates(&window);
    bind_compute_expiry_callback(&window);
    bind_generate_preview_callback(&window, proj.clone(), state.clone());
    bind_export_png_callback(&window, proj.clone(), state.clone());
    bind_export_pdf_callback(&window, proj.clone(), state.clone());
    bind_decrypt_callbacks(&window);
    bind_layout_editor_callbacks(&window, proj.clone(), state.clone());

    refresh_editor_for_page(&window, &proj, &state, PageKind::Reagent);
    window.run().expect("运行失败");
}
