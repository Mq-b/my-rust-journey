use crate::barcode::{generate_barcode, render_label, LabelContent};
use crate::config;
use crate::encryptor;
use crate::layout::{LayoutConfig, PageKind};
use image::GrayImage;

use super::helpers::{
    blank_or, compute_expire, project_id_at, project_name_at, require_fields,
};
use super::state::{GenerateMode, GeneratedPage};
use super::RLCLIAWindow;

/// 按页面类型分发条码生成流程。
pub(super) fn dispatch_generate(
    page: PageKind,
    window: &RLCLIAWindow,
    proj: &config::ProjectConfig,
    layout: &LayoutConfig,
    mode: GenerateMode,
) -> Result<GeneratedPage, String> {
    match page {
        PageKind::Reagent => gen_reagent(window, proj, layout, mode),
        PageKind::Calibration => gen_calibration(window, proj, layout),
        PageKind::Consumable => gen_consumable(window, layout),
        PageKind::Quality => gen_quality(window, proj, layout),
    }
}

/// 当真实预览生成失败时，构建一份尽量可读的占位内容。
pub(super) fn fallback_preview_content(
    page: PageKind,
    window: &RLCLIAWindow,
    proj: &config::ProjectConfig,
) -> LabelContent {
    match page {
        PageKind::Reagent => {
            let index = window.get_reagent_project_index() as usize;
            let name = project_name_at(proj, index);
            LabelContent {
                title: "试剂二维码信息".into(),
                subtitle1: Some(format!(
                    "{} 测定试剂盒",
                    if name.is_empty() { "项目名称" } else { &name }
                )),
                subtitle2: Some(format!(
                    "(化学发光免疫分析法)  {} 测试/盒",
                    blank_or(window.get_reagent_test_counts().as_str(), "测试数")
                )),
                lot_number: window.get_reagent_lot().to_string(),
                prod_date: window.get_reagent_prod_date().to_string(),
                expire_date: compute_expire(
                    &window.get_reagent_prod_date().to_string(),
                    window
                        .get_reagent_valid_days()
                        .to_string()
                        .parse()
                        .unwrap_or(365),
                ),
            }
        }
        PageKind::Calibration => {
            let index = window.get_calib_project_index() as usize;
            LabelContent {
                title: "校准品二维码".into(),
                subtitle1: Some(blank_or(&project_name_at(proj, index), "项目名称")),
                subtitle2: None,
                lot_number: window.get_calib_lot().to_string(),
                prod_date: window.get_calib_prod_date().to_string(),
                expire_date: compute_expire(
                    &window.get_calib_prod_date().to_string(),
                    window
                        .get_calib_valid_days()
                        .to_string()
                        .parse()
                        .unwrap_or(365),
                ),
            }
        }
        PageKind::Consumable => {
            let title = if window.get_consumable_type_index() == 0 {
                "激发液A二维码"
            } else {
                "激发液B二维码"
            };
            LabelContent {
                title: title.into(),
                subtitle1: None,
                subtitle2: None,
                lot_number: window.get_consumable_lot().to_string(),
                prod_date: window.get_consumable_prod_date().to_string(),
                expire_date: compute_expire(
                    &window.get_consumable_prod_date().to_string(),
                    window
                        .get_consumable_valid_days()
                        .to_string()
                        .parse()
                        .unwrap_or(365),
                ),
            }
        }
        PageKind::Quality => {
            let index = window.get_quality_project_index() as usize;
            LabelContent {
                title: "质控品二维码".into(),
                subtitle1: Some(blank_or(&project_name_at(proj, index), "项目名称")),
                subtitle2: None,
                lot_number: window.get_quality_lot().to_string(),
                prod_date: window.get_quality_prod_date().to_string(),
                expire_date: compute_expire(
                    &window.get_quality_prod_date().to_string(),
                    window
                        .get_quality_valid_days()
                        .to_string()
                        .parse()
                        .unwrap_or(365),
                ),
            }
        }
    }
}

fn gen_reagent(
    window: &RLCLIAWindow,
    proj: &config::ProjectConfig,
    layout: &LayoutConfig,
    mode: GenerateMode,
) -> Result<GeneratedPage, String> {
    let idx = window.get_reagent_project_index() as usize;
    let name = project_name_at(proj, idx);
    let id = project_id_at(proj, idx);
    let lot = window.get_reagent_lot().to_string();
    let prod = window.get_reagent_prod_date().to_string();
    let days: i64 = window
        .get_reagent_valid_days()
        .to_string()
        .parse()
        .unwrap_or(365);
    let exp = compute_expire(&prod, days);
    let counts = window.get_reagent_test_counts().to_string();
    let open = window.get_reagent_open_days().to_string();
    let n: usize = window
        .get_reagent_serial_count()
        .to_string()
        .parse()
        .unwrap_or(1)
        .max(1);
    let units = ["pg/mL", "ng/mL", "mg/L", "ng/L", "IU/L"];
    let unit = units
        .get(window.get_reagent_unit_index() as usize)
        .unwrap_or(&"pg/mL");
    let pa = window.get_reagent_param_a().to_string();
    let pb = window.get_reagent_param_b().to_string();
    let pc = window.get_reagent_param_c().to_string();
    let pd = window.get_reagent_param_d().to_string();
    let rl = window.get_reagent_range_low().to_string();
    let ru = window.get_reagent_range_upper().to_string();
    let ll = window.get_reagent_limit_low().to_string();
    let lu = window.get_reagent_limit_upper().to_string();

    require_fields(&[
        ("项目名称", &name),
        ("试剂批号", &lot),
        ("生产日期", &prod),
        ("有效天数", &window.get_reagent_valid_days().to_string()),
        ("测试/盒", &counts),
        ("开瓶天数", &open),
        ("数量", &window.get_reagent_serial_count().to_string()),
        ("曲线参数a", &pa),
        ("曲线参数b", &pb),
        ("曲线参数c", &pc),
        ("曲线参数d", &pd),
        ("范围下限", &rl),
        ("范围上限", &ru),
        ("限值下限", &ll),
        ("限值上限", &lu),
    ])?;

    let content = LabelContent {
        title: "试剂二维码信息".into(),
        subtitle1: Some(format!("{name} 测定试剂盒")),
        subtitle2: Some(format!("(化学发光免疫分析法)  {counts} 测试/盒")),
        lot_number: lot.clone(),
        prod_date: prod.clone(),
        expire_date: exp.clone(),
    };

    let mut preview_barcode = None;
    let mut images = Vec::new();
    let serials = match mode {
        GenerateMode::Preview => config::preview_reagent_ids(&lot, n)?,
        GenerateMode::PersistReagentIds => config::consume_reagent_ids(&lot, n)?,
        GenerateMode::ExportSingle => config::consume_reagent_ids(&lot, 1)?,
    };

    for serial in serials {
        let enc = encryptor::compose_reagent(
            &name, &id, &lot, &prod, &exp, &counts, &open, "direct", &serial, unit, &pa, &pb,
            &pc, &pd, &rl, &ru, &ll, &lu,
        )?;
        let barcode = generate_barcode(&enc)?;
        if preview_barcode.is_none() {
            preview_barcode = Some(barcode.clone());
        }
        images.push(render_label(
            &barcode,
            layout.page(PageKind::Reagent),
            &content,
        ));
    }

    Ok(GeneratedPage {
        images,
        content,
        preview_barcode: preview_barcode.unwrap_or_else(|| GrayImage::new(1, 1)),
        label: "试剂",
    })
}

fn gen_calibration(
    window: &RLCLIAWindow,
    proj: &config::ProjectConfig,
    layout: &LayoutConfig,
) -> Result<GeneratedPage, String> {
    let idx = window.get_calib_project_index() as usize;
    let name = project_name_at(proj, idx);
    let id = project_id_at(proj, idx);
    let lot = window.get_calib_lot().to_string();
    let prod = window.get_calib_prod_date().to_string();
    let days: i64 = window
        .get_calib_valid_days()
        .to_string()
        .parse()
        .unwrap_or(365);
    let exp = compute_expire(&prod, days);
    let n: usize = window
        .get_calib_quantity()
        .to_string()
        .parse()
        .unwrap_or(1)
        .max(1);
    let c1 = window.get_calib_c1().to_string();
    let c2 = window.get_calib_c2().to_string();

    require_fields(&[
        ("项目名称", &name),
        ("校准批号", &lot),
        ("生产日期", &prod),
        ("有效天数", &window.get_calib_valid_days().to_string()),
        ("数量", &window.get_calib_quantity().to_string()),
        ("C1发光值", &c1),
        ("C2发光值", &c2),
    ])?;

    let content = LabelContent {
        title: "校准品二维码".into(),
        subtitle1: Some(name.clone()),
        subtitle2: None,
        lot_number: lot.clone(),
        prod_date: prod.clone(),
        expire_date: exp.clone(),
    };

    let mut preview_barcode = None;
    let mut images = Vec::new();
    for _ in 0..n {
        let enc =
            encryptor::compose_calibration(&name, &id, &lot, &prod, &exp, "direct", &c1, &c2)?;
        let barcode = generate_barcode(&enc)?;
        if preview_barcode.is_none() {
            preview_barcode = Some(barcode.clone());
        }
        images.push(render_label(
            &barcode,
            layout.page(PageKind::Calibration),
            &content,
        ));
    }

    Ok(GeneratedPage {
        images,
        content,
        preview_barcode: preview_barcode.unwrap_or_else(|| GrayImage::new(1, 1)),
        label: "校准品",
    })
}

fn gen_consumable(window: &RLCLIAWindow, layout: &LayoutConfig) -> Result<GeneratedPage, String> {
    let types = ["激发液A", "激发液B"];
    let type_index = window.get_consumable_type_index() as usize;
    let type_name = types.get(type_index).unwrap_or(&"激发液A");
    let lot = window.get_consumable_lot().to_string();
    let prod = window.get_consumable_prod_date().to_string();
    let days: i64 = window
        .get_consumable_valid_days()
        .to_string()
        .parse()
        .unwrap_or(365);
    let exp = compute_expire(&prod, days);
    let freq = window.get_consumable_freq().to_string();
    let open = window.get_consumable_open_days().to_string();
    let n: usize = window
        .get_consumable_quantity()
        .to_string()
        .parse()
        .unwrap_or(1)
        .max(1);

    require_fields(&[
        ("耗材批号", &lot),
        ("生产日期", &prod),
        ("有效天数", &window.get_consumable_valid_days().to_string()),
        ("可用频次", &freq),
        ("开瓶天数", &open),
        ("数量", &window.get_consumable_quantity().to_string()),
    ])?;

    let content = LabelContent {
        title: format!("{type_name}二维码"),
        subtitle1: None,
        subtitle2: None,
        lot_number: lot.clone(),
        prod_date: prod.clone(),
        expire_date: exp.clone(),
    };

    let mut preview_barcode = None;
    let mut images = Vec::new();
    for _ in 0..n {
        let enc = encryptor::compose_consumable(type_name, &lot, &prod, &exp, &freq, &open)?;
        let barcode = generate_barcode(&enc)?;
        if preview_barcode.is_none() {
            preview_barcode = Some(barcode.clone());
        }
        images.push(render_label(
            &barcode,
            layout.page(PageKind::Consumable),
            &content,
        ));
    }

    Ok(GeneratedPage {
        images,
        content,
        preview_barcode: preview_barcode.unwrap_or_else(|| GrayImage::new(1, 1)),
        label: "耗材",
    })
}

fn gen_quality(
    window: &RLCLIAWindow,
    proj: &config::ProjectConfig,
    layout: &LayoutConfig,
) -> Result<GeneratedPage, String> {
    let idx = window.get_quality_project_index() as usize;
    let name = project_name_at(proj, idx);
    let id = project_id_at(proj, idx);
    let lot = window.get_quality_lot().to_string();
    let prod = window.get_quality_prod_date().to_string();
    let days: i64 = window
        .get_quality_valid_days()
        .to_string()
        .parse()
        .unwrap_or(365);
    let exp = compute_expire(&prod, days);
    let n: usize = window
        .get_quality_quantity()
        .to_string()
        .parse()
        .unwrap_or(1)
        .max(1);
    let q1 = window.get_quality_q1().to_string();
    let sd1 = window.get_quality_sd1().to_string();
    let q2 = window.get_quality_q2().to_string();
    let sd2 = window.get_quality_sd2().to_string();

    require_fields(&[
        ("项目名称", &name),
        ("质控批号", &lot),
        ("生产日期", &prod),
        ("有效天数", &window.get_quality_valid_days().to_string()),
        ("数量", &window.get_quality_quantity().to_string()),
        ("Q1", &q1),
        ("SD1", &sd1),
        ("Q2", &q2),
        ("SD2", &sd2),
    ])?;

    let content = LabelContent {
        title: "质控品二维码".into(),
        subtitle1: Some(name.clone()),
        subtitle2: None,
        lot_number: lot.clone(),
        prod_date: prod.clone(),
        expire_date: exp.clone(),
    };

    let mut preview_barcode = None;
    let mut images = Vec::new();
    for _ in 0..n {
        let enc = encryptor::compose_quality(
            &name, &id, &lot, &prod, &exp, "direct", &q1, &sd1, &q2, &sd2,
        )?;
        let barcode = generate_barcode(&enc)?;
        if preview_barcode.is_none() {
            preview_barcode = Some(barcode.clone());
        }
        images.push(render_label(
            &barcode,
            layout.page(PageKind::Quality),
            &content,
        ));
    }

    Ok(GeneratedPage {
        images,
        content,
        preview_barcode: preview_barcode.unwrap_or_else(|| GrayImage::new(1, 1)),
        label: "质控品",
    })
}
