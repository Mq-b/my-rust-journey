use crate::barcode::{gray_to_slint_image, LabelContent};
use crate::config;
use crate::layout::{
    load_layout_config, save_layout_config, LayoutElement, LayoutElementKind, PageKind,
    LABEL_HEIGHT, LABEL_WIDTH,
};
use image::GrayImage;
use slint::{ComponentHandle, Model};

use super::generator::{dispatch_generate, fallback_preview_content};
use super::state::{GenerateMode, LayoutUpdate, PreviewModel, SelectedElement, SharedEditorState};
use super::{PreviewElementData, RLCLIAWindow};

/// 绑定布局编辑器相关的全部回调。
pub(super) fn bind_layout_editor_callbacks(
    window: &RLCLIAWindow,
    proj: config::ProjectConfig,
    state: SharedEditorState,
) {
    let weak = window.as_weak();
    let proj_load = proj.clone();
    let state_load = state.clone();
    window.on_load_layout_page(move |page| {
        let window = weak.unwrap();
        state_load.borrow_mut().layout = load_layout_config();
        refresh_editor_for_page(
            &window,
            &proj_load,
            &state_load,
            PageKind::from_ui(&page.to_string()),
        );
    });

    let weak = window.as_weak();
    let state_select = state.clone();
    window.on_select_layout_element(move |page, id| {
        let window = weak.unwrap();
        select_layout_element(
            &window,
            &state_select,
            PageKind::from_ui(&page.to_string()),
            &id.to_string(),
        );
    });

    let weak = window.as_weak();
    let state_drag = state.clone();
    window.on_drag_layout_element(move |page, id, dx, dy| {
        let window = weak.unwrap();
        adjust_layout_element(
            &window,
            &state_drag,
            PageKind::from_ui(&page.to_string()),
            &id.to_string(),
            LayoutUpdate::Move { dx, dy },
        );
    });

    let weak = window.as_weak();
    let state_field = state.clone();
    window.on_update_selected_layout(move |page, field, value| {
        let window = weak.unwrap();
        adjust_selected_field(
            &window,
            &state_field,
            PageKind::from_ui(&page.to_string()),
            &field.to_string(),
            &value.to_string(),
        );
    });

    let weak = window.as_weak();
    let state_bold = state.clone();
    window.on_toggle_selected_bold(move |page, value| {
        let window = weak.unwrap();
        adjust_selected_bold(
            &window,
            &state_bold,
            PageKind::from_ui(&page.to_string()),
            value,
        );
    });

    let weak = window.as_weak();
    window.on_reset_layout_page(move |page| {
        let window = weak.unwrap();
        let page = PageKind::from_ui(&page.to_string());
        {
            let mut editor = state.borrow_mut();
            editor.layout.reset_page(page);
            if let Err(err) = save_layout_config(&editor.layout) {
                window.set_status(format!("布局保存失败: {err}").into());
                return;
            }
        }
        refresh_editor_for_page(&window, &proj, &state, page);
        window.set_status(format!("{} 页面布局已重置", page.label()).into());
    });
}

/// 刷新某一页面的预览与选中态。
pub(super) fn refresh_editor_for_page(
    window: &RLCLIAWindow,
    proj: &config::ProjectConfig,
    state: &SharedEditorState,
    page: PageKind,
) {
    let preview = dispatch_generate(
        page,
        window,
        proj,
        &state.borrow().layout,
        GenerateMode::Preview,
    )
    .map(|generated| (generated.content, Some(generated.preview_barcode)))
    .unwrap_or_else(|_| (fallback_preview_content(page, window, proj), None));
    apply_preview_data(window, state, page, &preview.0, preview.1.as_ref());
}

/// 将业务层生成结果同步到布局编辑器和预览区。
pub(super) fn apply_preview_data(
    window: &RLCLIAWindow,
    state: &SharedEditorState,
    page: PageKind,
    content: &LabelContent,
    barcode: Option<&GrayImage>,
) {
    let mut editor = state.borrow_mut();
    editor.active_page = page;
    let elements = build_preview_elements(editor.layout.page(page), content);
    editor.preview_model.set_vec(elements);
    if let Some(image) = barcode {
        window.set_preview_barcode(gray_to_slint_image(image));
        window.set_preview_barcode_visible(true);
    } else {
        window.set_preview_barcode(slint::Image::default());
        window.set_preview_barcode_visible(false);
    }
    window.set_layout_editor_ready(true);

    let keep_selected = editor.selected.as_ref().is_some_and(|selected| {
        selected.page == page && editor.layout.page(page).element(&selected.id).is_some()
    });
    if !keep_selected {
        editor.selected = None;
    }
    drop(editor);
    sync_selected_fields(window, state);
}

fn select_layout_element(window: &RLCLIAWindow, state: &SharedEditorState, page: PageKind, id: &str) {
    let mut editor = state.borrow_mut();
    if editor.layout.page(page).element(id).is_some() {
        editor.selected = Some(SelectedElement {
            page,
            id: id.to_string(),
        });
    }
    drop(editor);
    sync_selected_fields(window, state);
}

fn adjust_selected_field(
    window: &RLCLIAWindow,
    state: &SharedEditorState,
    page: PageKind,
    field: &str,
    value: &str,
) {
    let Ok(value) = value.trim().parse::<f32>() else {
        return;
    };
    adjust_current_element(window, state, page, |element| match field {
        "x" => element.x = value,
        "y" => element.y = value,
        "width" => element.width = value,
        "height" => element.height = value,
        "font_size" => element.font_size = value,
        _ => {}
    });
}

fn adjust_selected_bold(window: &RLCLIAWindow, state: &SharedEditorState, page: PageKind, value: bool) {
    adjust_current_element(window, state, page, |element| {
        if element.kind == LayoutElementKind::Text {
            element.bold = value;
        }
    });
}

fn adjust_layout_element(
    window: &RLCLIAWindow,
    state: &SharedEditorState,
    page: PageKind,
    id: &str,
    update: LayoutUpdate,
) {
    let mut editor = state.borrow_mut();
    let (x, y, width, height, font_size, bold) = {
        let Some(element) = editor.layout.page_mut(page).element_mut(id) else {
            return;
        };
        match update {
            LayoutUpdate::Move { dx, dy } => {
                element.x += dx;
                element.y += dy;
            }
        }
        normalize_element(element);
        (
            element.x,
            element.y,
            element.width,
            element.height,
            element.font_size,
            element.bold,
        )
    };
    if let Err(err) = save_layout_config(&editor.layout) {
        window.set_status(format!("布局保存失败: {err}").into());
        return;
    }
    if let Some(row) = preview_row_index(&editor.preview_model, id) {
        if let Some(mut item) = editor.preview_model.row_data(row) {
            item.x = x;
            item.y = y;
            item.width = width;
            item.height = height;
            item.font_size = font_size;
            item.bold = bold;
            editor.preview_model.set_row_data(row, item);
        }
    }
    editor.selected = Some(SelectedElement {
        page,
        id: id.to_string(),
    });
    drop(editor);
    sync_selected_fields(window, state);
    window.set_status(format!("{} 布局已保存到配置文件", page.label()).into());
}

fn adjust_current_element<F>(
    window: &RLCLIAWindow,
    state: &SharedEditorState,
    page: PageKind,
    mut update: F,
) where
    F: FnMut(&mut LayoutElement),
{
    let mut editor = state.borrow_mut();
    let Some(selected) = editor.selected.clone() else {
        return;
    };
    if selected.page != page {
        return;
    }
    let (x, y, width, height, font_size, bold) = {
        let Some(element) = editor.layout.page_mut(page).element_mut(&selected.id) else {
            return;
        };
        update(element);
        normalize_element(element);
        (
            element.x,
            element.y,
            element.width,
            element.height,
            element.font_size,
            element.bold,
        )
    };
    if let Err(err) = save_layout_config(&editor.layout) {
        window.set_status(format!("布局保存失败: {err}").into());
        return;
    }
    if let Some(row) = preview_row_index(&editor.preview_model, &selected.id) {
        if let Some(mut item) = editor.preview_model.row_data(row) {
            item.x = x;
            item.y = y;
            item.width = width;
            item.height = height;
            item.font_size = font_size;
            item.bold = bold;
            editor.preview_model.set_row_data(row, item);
        }
    }
    drop(editor);
    sync_selected_fields(window, state);
    window.set_status(format!("{} 布局已保存到配置文件", page.label()).into());
}

fn preview_row_index(model: &PreviewModel, id: &str) -> Option<usize> {
    (0..model.row_count()).find(|index| {
        model
            .row_data(*index)
            .is_some_and(|row| row.id.as_str() == id)
    })
}

fn sync_selected_fields(window: &RLCLIAWindow, state: &SharedEditorState) {
    let editor = state.borrow();
    let Some(selected) = editor.selected.as_ref() else {
        clear_selected_fields(window);
        return;
    };
    let Some(element) = editor.layout.page(selected.page).element(&selected.id) else {
        clear_selected_fields(window);
        return;
    };
    window.set_selected_has_element(true);
    window.set_selected_element_id(selected.id.clone().into());
    window.set_selected_element_name(element_name(&selected.id).into());
    window.set_selected_element_kind(match element.kind {
        LayoutElementKind::Text => "text".into(),
        LayoutElementKind::Barcode => "barcode".into(),
    });
    window.set_selected_layout_x(format_number(element.x).into());
    window.set_selected_layout_y(format_number(element.y).into());
    window.set_selected_layout_width(format_number(element.width).into());
    window.set_selected_layout_height(format_number(element.height).into());
    window.set_selected_font_size(format_number(element.font_size).into());
    window.set_selected_bold(element.bold);
}

fn clear_selected_fields(window: &RLCLIAWindow) {
    window.set_selected_has_element(false);
    window.set_selected_element_id("".into());
    window.set_selected_element_name("".into());
    window.set_selected_element_kind("".into());
    window.set_selected_layout_x("".into());
    window.set_selected_layout_y("".into());
    window.set_selected_layout_width("".into());
    window.set_selected_layout_height("".into());
    window.set_selected_font_size("".into());
    window.set_selected_bold(false);
}

fn build_preview_elements(
    layout: &crate::layout::PageLayout,
    content: &LabelContent,
) -> Vec<PreviewElementData> {
    layout
        .elements
        .iter()
        .map(|element| PreviewElementData {
            id: element.id.clone().into(),
            name: element_name(&element.id).into(),
            kind: match element.kind {
                LayoutElementKind::Text => "text".into(),
                LayoutElementKind::Barcode => "barcode".into(),
            },
            text: preview_text(&element.id, content).into(),
            x: element.x,
            y: element.y,
            width: element.width,
            height: element.height,
            font_size: element.font_size,
            bold: element.bold,
            visible: true,
        })
        .collect()
}

fn preview_text(id: &str, content: &LabelContent) -> String {
    let actual = match id {
        "title" => content.title.clone(),
        "subtitle1" => content.subtitle1.clone().unwrap_or_default(),
        "subtitle2" => content.subtitle2.clone().unwrap_or_default(),
        "lot" => format!("产品批号: {}", content.lot_number),
        "prod_date" => format!("生产日期: {}", content.prod_date),
        "expire_date" => format!("失效日期: {}", content.expire_date),
        "barcode" => String::new(),
        _ => String::new(),
    };
    if actual.trim().is_empty() {
        format!("[{}]", element_name(id))
    } else {
        actual
    }
}

fn element_name(id: &str) -> &'static str {
    match id {
        "title" => "一级标题",
        "subtitle1" => "二级标题",
        "subtitle2" => "三级标题",
        "barcode" => "条码区域",
        "lot" => "产品批号",
        "prod_date" => "生产日期",
        "expire_date" => "失效日期",
        _ => "元素",
    }
}

fn normalize_element(element: &mut LayoutElement) {
    element.width = element.width.clamp(1.0, LABEL_WIDTH);
    element.height = element.height.clamp(1.0, LABEL_HEIGHT);
    if element.kind == LayoutElementKind::Text {
        element.font_size = element.font_size.clamp(1.0, 200.0);
    } else {
        element.font_size = 0.0;
        element.bold = false;
    }
    element.x = element.x.clamp(0.0, (LABEL_WIDTH - element.width).max(0.0));
    element.y = element
        .y
        .clamp(0.0, (LABEL_HEIGHT - element.height).max(0.0));
}

fn format_number(value: f32) -> String {
    if (value - value.round()).abs() < 0.05 {
        format!("{}", value.round() as i32)
    } else {
        format!("{value:.1}")
    }
}
