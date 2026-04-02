use crate::encryptor;
use rfd::FileDialog;
use slint::{ComponentHandle, ModelRc, VecModel};
use std::path::Path;
use zxingcpp::{BarcodeFormat, BarcodeReader};

use super::helpers::restore_special_name;
use super::state::ParsedBarcodeData;
use super::{DecodeFieldData, RLCLIAWindow};

/// 绑定“文本解密”和“图片识别解密”相关回调。
pub(super) fn bind_decrypt_callbacks(window: &RLCLIAWindow) {
    let weak = window.as_weak();
    window.on_decrypt_data(move || {
        let window = weak.unwrap();
        let input = window.get_decrypt_input().to_string();
        if let Err(err) = process_decrypt_input(&window, &input, "文本输入", false) {
            apply_decrypt_error(&window, &err, "解密失败");
        }
    });

    let weak = window.as_weak();
    window.on_pick_decrypt_image(move || {
        let window = weak.unwrap();
        if let Some(path) = FileDialog::new()
            .set_title("选择包含 PDF417 的条码图片")
            .add_filter(
                "图片文件",
                &["png", "jpg", "jpeg", "bmp", "tif", "tiff", "webp"],
            )
            .pick_file()
        {
            match decode_cipher_from_image(&path) {
                Ok(cipher) => {
                    if let Err(err) = process_decrypt_input(&window, &cipher, "图片扫描", true) {
                        apply_decrypt_error(&window, &err, "识别失败");
                    } else {
                        window.set_status(format!("已识别图片: {}", path.display()).into());
                        window.set_toast_msg("图片识别成功".into());
                        window.set_toast_visible(true);
                    }
                }
                Err(err) => apply_decrypt_error(&window, &err, "识别失败"),
            }
        }
    });
}

fn apply_decrypt_error(window: &RLCLIAWindow, err: &str, toast_msg: &str) {
    window.set_decrypt_output(format!("错误: {err}").into());
    window.set_decrypt_type("".into());
    window.set_decrypt_source("".into());
    window.set_decode_fields(ModelRc::new(VecModel::from(Vec::<DecodeFieldData>::new())));
    window.set_status(format!("错误: {err}").into());
    window.set_toast_msg(toast_msg.into());
    window.set_toast_visible(true);
}

fn process_decrypt_input(
    window: &RLCLIAWindow,
    cipher_input: &str,
    source: &str,
    update_input_box: bool,
) -> Result<(), String> {
    let cipher = cipher_input.trim();
    if cipher.is_empty() {
        return Err("请输入密文，或先选择图片".into());
    }

    let plain = match encryptor::decrypt(cipher) {
        Ok(plain) => plain,
        Err(err) => {
            if looks_like_plain_payload(cipher) {
                cipher.to_string()
            } else {
                return Err(err);
            }
        }
    };

    let parsed = parse_barcode_payload(&plain)?;
    apply_parsed_barcode(window, source, cipher, &plain, &parsed, update_input_box);
    Ok(())
}

fn decode_cipher_from_image(path: &Path) -> Result<String, String> {
    let image = image::open(path).map_err(|e| format!("打开图片失败: {e}"))?;

    let reader = BarcodeReader::new()
        .formats(&[BarcodeFormat::PDF417])
        .try_harder(true)
        .try_rotate(true)
        .try_invert(true)
        .try_downscale(true);
    let barcodes = reader
        .from(&image)
        .map_err(|e| format!("识别条码失败: {e}"))?;
    if let Some(text) = first_valid_barcode_text(barcodes) {
        return Ok(text);
    }

    // 兜底尝试：不限制码制，避免上传图像码制标记异常导致漏读。
    let fallback_reader = BarcodeReader::new()
        .try_harder(true)
        .try_rotate(true)
        .try_invert(true)
        .try_downscale(true);
    let fallback = fallback_reader
        .from(&image)
        .map_err(|e| format!("识别条码失败: {e}"))?;
    first_valid_barcode_text(fallback).ok_or_else(|| "未在图片中识别到有效条码内容".to_string())
}

fn first_valid_barcode_text(barcodes: Vec<zxingcpp::Barcode>) -> Option<String> {
    barcodes
        .into_iter()
        .map(|barcode| barcode.text())
        .find(|text| !text.trim().is_empty())
}

fn apply_parsed_barcode(
    window: &RLCLIAWindow,
    source: &str,
    cipher: &str,
    plain: &str,
    parsed: &ParsedBarcodeData,
    update_input_box: bool,
) {
    if update_input_box {
        window.set_decrypt_input(cipher.into());
    }

    window.set_decrypt_output(plain.into());
    window.set_decrypt_source(source.into());
    window.set_decrypt_type(parsed.type_label.clone().into());

    let rows: Vec<DecodeFieldData> = parsed
        .fields
        .iter()
        .map(|(key, value)| DecodeFieldData {
            key: key.clone().into(),
            value: value.clone().into(),
        })
        .collect();
    window.set_decode_fields(ModelRc::new(VecModel::from(rows)));
    window.set_status(format!("{}解析成功（{}）", parsed.type_label, source).into());
    window.set_toast_msg("解密解析成功".into());
    window.set_toast_visible(true);
}

fn looks_like_plain_payload(input: &str) -> bool {
    let kind = input
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    matches!(
        kind.as_str(),
        "reagent" | "calibration" | "consumable" | "qc"
    )
}

fn parse_barcode_payload(payload: &str) -> Result<ParsedBarcodeData, String> {
    let parts: Vec<&str> = payload.split(';').collect();
    if parts.is_empty() || parts[0].trim().is_empty() {
        return Err("明文内容为空或格式不正确".into());
    }

    let get = |index: usize| {
        parts
            .get(index)
            .map(|value| value.trim().to_string())
            .unwrap_or_default()
    };

    let kind = parts[0].trim().to_ascii_lowercase();
    let parsed = match kind.as_str() {
        "reagent" => {
            let lot = get(3);
            let reagent_id = get(9);
            ParsedBarcodeData {
                type_label: "试剂条码".into(),
                fields: vec![
                    ("项目名称".into(), restore_special_name(&get(1))),
                    ("项目编号".into(), get(2)),
                    ("试剂批号".into(), lot),
                    ("试剂ID".into(), reagent_id),
                    ("结果单位".into(), get(10)),
                    ("测试/盒".into(), get(6)),
                    ("开瓶天数".into(), get(7)),
                    ("反应模式".into(), get(8)),
                    ("生产日期".into(), get(4)),
                    ("失效日期".into(), get(5)),
                    ("曲线参数a".into(), get(11)),
                    ("曲线参数b".into(), get(12)),
                    ("曲线参数c".into(), get(13)),
                    ("曲线参数d".into(), get(14)),
                    ("范围下限".into(), get(15)),
                    ("范围上限".into(), get(16)),
                    ("限值下限".into(), get(17)),
                    ("限值上限".into(), get(18)),
                ],
            }
        }
        "calibration" => ParsedBarcodeData {
            type_label: "校准品条码".into(),
            fields: vec![
                ("项目名称".into(), restore_special_name(&get(1))),
                ("项目编号".into(), get(2)),
                ("校准批号".into(), get(3)),
                ("生产日期".into(), get(4)),
                ("失效日期".into(), get(5)),
                ("反应模式".into(), get(6)),
                ("C1 发光值".into(), get(7)),
                ("C2 发光值".into(), get(8)),
            ],
        },
        "consumable" => ParsedBarcodeData {
            type_label: "耗材条码".into(),
            fields: vec![
                ("耗材名称".into(), restore_special_name(&get(1))),
                ("耗材批号".into(), get(2)),
                ("生产日期".into(), get(3)),
                ("失效日期".into(), get(4)),
                ("可用频次".into(), get(5)),
                ("开瓶天数".into(), get(6)),
            ],
        },
        "qc" => ParsedBarcodeData {
            type_label: "质控品条码".into(),
            fields: vec![
                ("项目名称".into(), restore_special_name(&get(1))),
                ("项目编号".into(), get(2)),
                ("质控批号".into(), get(3)),
                ("生产日期".into(), get(4)),
                ("失效日期".into(), get(5)),
                ("反应模式".into(), get(6)),
                ("Q1".into(), get(7)),
                ("SD1".into(), get(8)),
                ("Q2".into(), get(9)),
                ("SD2".into(), get(10)),
            ],
        },
        _ => {
            let mut fields = Vec::new();
            for (index, value) in parts.iter().enumerate().skip(1) {
                fields.push((format!("字段{index}"), value.trim().to_string()));
            }
            ParsedBarcodeData {
                type_label: "未知类型条码".into(),
                fields,
            }
        }
    };

    Ok(parsed)
}
