use crate::barcode::LabelContent;
use crate::layout::{LayoutConfig, PageKind};
use image::GrayImage;
use slint::VecModel;
use std::cell::RefCell;
use std::rc::Rc;

use super::PreviewElementData;

/// 布局编辑器右侧预览列表的数据模型。
pub(super) type PreviewModel = Rc<VecModel<PreviewElementData>>;

/// 页面编辑与预览共享的运行时状态。
pub(super) struct EditorState {
    pub(super) layout: LayoutConfig,
    pub(super) preview_model: PreviewModel,
    pub(super) selected: Option<SelectedElement>,
    pub(super) active_page: PageKind,
}

/// 布局编辑器中当前选中的元素定位信息。
#[derive(Clone)]
pub(super) struct SelectedElement {
    pub(super) page: PageKind,
    pub(super) id: String,
}

/// 统一的条码生成结果对象，供预览、导出复用。
pub(super) struct GeneratedPage {
    pub(super) images: Vec<GrayImage>,
    pub(super) content: LabelContent,
    pub(super) preview_barcode: GrayImage,
    pub(super) label: &'static str,
}

/// 解密后明文解析结果。
pub(super) struct ParsedBarcodeData {
    pub(super) type_label: String,
    pub(super) fields: Vec<(String, String)>,
}

/// 条码生成模式。
#[derive(Clone, Copy)]
pub(super) enum GenerateMode {
    /// 仅用于预览，不消耗序列号。
    Preview,
    /// 用于正式导出，按数量消耗序列号。
    PersistReagentIds,
    /// 导出单张图片，消耗一个序列号。
    ExportSingle,
}

/// 布局元素更新指令。
pub(super) enum LayoutUpdate {
    /// 按偏移量移动元素。
    Move { dx: f32, dy: f32 },
}

/// 共享的编辑器状态句柄。
pub(super) type SharedEditorState = Rc<RefCell<EditorState>>;
