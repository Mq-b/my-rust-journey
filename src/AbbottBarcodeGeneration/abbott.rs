use crate::barcode::{gray_to_slint_image, make_barcode_image, save_png_300dpi};
use crate::config::Config;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

// ---------------------------------------------------------------------------
// Config structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbbottReagent {
    pub name: String,
    /// Prefix character used in short barcode (e.g. "H", "J")
    pub short_prefix: String,
    /// Prefix character used in long barcode (e.g. "G")
    pub long_prefix: String,
    /// Whether this reagent produces a long barcode
    #[serde(default = "bool_true")]
    pub generates_long: bool,
    /// Whether this reagent produces a short barcode
    #[serde(default = "bool_true")]
    pub generates_short: bool,
    /// Fixed project identification bits appended in long barcode
    pub project_bits: String,
    /// Fixed trailing data appended after project_bits in long barcode
    pub long_trailing: String,
    /// Default SN for this reagent slot
    #[serde(default)]
    pub default_sn: String,
}

fn bool_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbbottProject {
    pub name: String,
    /// Suffix appended to control-no number (e.g. "UD00", "UN24")
    pub control_no_suffix: String,
    /// Default batch number pre-filled in the UI (e.g. "80001")
    pub control_no_default_number: String,
    /// How the expiry date is encoded:
    ///   "DDMMYYYY"   → 8-char DDMMYYYY
    ///   "00DDMMYYYY" → 10-char with leading "00"
    pub expiry_format: String,
    pub reagents: Vec<AbbottReagent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AbbottProjectsConfig {
    pub projects: Vec<AbbottProject>,
}

// ---------------------------------------------------------------------------
// Config loading
// ---------------------------------------------------------------------------

pub fn load_abbott_projects() -> AbbottProjectsConfig {
    let path = "assets/abbott_projects.json";
    if let Ok(s) = fs::read_to_string(path) {
        if let Ok(cfg) = serde_json::from_str::<AbbottProjectsConfig>(&s) {
            return cfg;
        }
    }
    let default = default_abbott_projects();
    if let Ok(data) = serde_json::to_string_pretty(&default) {
        let _ = fs::create_dir_all("assets");
        let _ = fs::write(path, data);
    }
    default
}

fn default_abbott_projects() -> AbbottProjectsConfig {
    AbbottProjectsConfig {
        projects: vec![
            // CTNI: 2码一组（红长码 + 黄短码），红黄各有独立SN
            AbbottProject {
                name: "CTNI".to_string(),
                control_no_suffix: "UD00".to_string(),
                control_no_default_number: "81307".to_string(),
                expiry_format: "DDMMYYYY".to_string(),
                reagents: vec![
                    AbbottReagent {
                        name: "CTNI 红".to_string(),
                        short_prefix: "H".to_string(),
                        long_prefix: "G".to_string(),
                        generates_long: true,
                        generates_short: false,
                        project_bits: "6201010300001".to_string(),
                        long_trailing: "H0000162001AAAGOAABTZAAINQABPTEBCAUMDXUWW00000AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
                        default_sn: "01137".to_string(),
                    },
                    AbbottReagent {
                        name: "CTNI 黄".to_string(),
                        short_prefix: "H".to_string(),
                        long_prefix: "G".to_string(),
                        generates_long: false,
                        generates_short: true,
                        project_bits: "6201010300001".to_string(),
                        long_trailing: "".to_string(),
                        default_sn: "01137".to_string(),
                    },
                ],
            },
            // CK-MB: 2码一组（红长码 + 黄短码），红黄各有独立SN
            AbbottProject {
                name: "CK-MB".to_string(),
                control_no_suffix: "UN24".to_string(),
                control_no_default_number: "93880".to_string(),
                expiry_format: "DDMMYYYY".to_string(),
                reagents: vec![
                    AbbottReagent {
                        name: "CK-MB 红".to_string(),
                        short_prefix: "H".to_string(),
                        long_prefix: "G".to_string(),
                        generates_long: true,
                        generates_short: false,
                        project_bits: "4712010300001".to_string(),
                        long_trailing: "H0000000000AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA00000AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
                        default_sn: "03157".to_string(),
                    },
                    AbbottReagent {
                        name: "CK-MB 黄".to_string(),
                        short_prefix: "H".to_string(),
                        long_prefix: "G".to_string(),
                        generates_long: false,
                        generates_short: true,
                        project_bits: "4712010300001".to_string(),
                        long_trailing: "".to_string(),
                        default_sn: "06975".to_string(),
                    },
                ],
            },
            // Myo: 3码一组（红长码 + 黄短码 + 绿短码）
            // 红色试剂：生成长码；黄色/绿色试剂：仅生成短码
            AbbottProject {
                name: "Myo".to_string(),
                control_no_suffix: "UN24".to_string(),
                control_no_default_number: "71084".to_string(),
                expiry_format: "DDMMYYYY".to_string(),
                reagents: vec![
                    AbbottReagent {
                        name: "Myo 红".to_string(),
                        short_prefix: "G".to_string(),
                        long_prefix: "G".to_string(),
                        generates_long: true,
                        generates_short: false,
                        project_bits: "4612010300002".to_string(),
                        long_trailing: "HJ000000000AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA00000AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
                        default_sn: "03157".to_string(),
                    },
                    AbbottReagent {
                        name: "Myo 黄".to_string(),
                        short_prefix: "H".to_string(),
                        long_prefix: "G".to_string(),
                        generates_long: false,
                        generates_short: true,
                        project_bits: "4612010300002".to_string(),
                        long_trailing: "".to_string(),
                        default_sn: "02972".to_string(),
                    },
                    AbbottReagent {
                        name: "Myo 绿".to_string(),
                        short_prefix: "J".to_string(),
                        long_prefix: "G".to_string(),
                        generates_long: false,
                        generates_short: true,
                        project_bits: "4612010300002".to_string(),
                        long_trailing: "".to_string(),
                        default_sn: "03824".to_string(),
                    },
                ],
            },
            // BNP: 3码一组（红长码 + 黄短码 + 绿短码）
            AbbottProject {
                name: "BNP".to_string(),
                control_no_suffix: "UN24".to_string(),
                control_no_default_number: "71084".to_string(),
                expiry_format: "DDMMYYYY".to_string(),
                reagents: vec![
                    AbbottReagent {
                        name: "BNP 红".to_string(),
                        short_prefix: "G".to_string(),
                        long_prefix: "G".to_string(),
                        generates_long: true,
                        generates_short: false,
                        project_bits: "0000000000000".to_string(),
                        long_trailing: "H0000000000AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA00000AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
                        default_sn: "03157".to_string(),
                    },
                    AbbottReagent {
                        name: "BNP 黄".to_string(),
                        short_prefix: "H".to_string(),
                        long_prefix: "G".to_string(),
                        generates_long: false,
                        generates_short: true,
                        project_bits: "0000000000000".to_string(),
                        long_trailing: "".to_string(),
                        default_sn: "02972".to_string(),
                    },
                    AbbottReagent {
                        name: "BNP 绿".to_string(),
                        short_prefix: "J".to_string(),
                        long_prefix: "G".to_string(),
                        generates_long: false,
                        generates_short: true,
                        project_bits: "0000000000000".to_string(),
                        long_trailing: "".to_string(),
                        default_sn: "03824".to_string(),
                    },
                ],
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Barcode content builders
// ---------------------------------------------------------------------------

/// Encode "YYYY-MM-DD" into the format specified by `fmt`.
pub fn encode_expiry(date_str: &str, _fmt: &str) -> String {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return date_str.to_string();
    }
    let (year, month, day) = (parts[0], parts[1], parts[2]);
    // All projects use DDMMYYYY (day first, then month, then year)
    format!("{}{}{}", day, month, year)
}

/// Short barcode: A{SN}{short_prefix}{control_no_number}{control_no_suffix}
pub fn build_short_content(
    sn: &str,
    reagent: &AbbottReagent,
    control_no_number: &str,
    control_no_suffix: &str,
) -> String {
    format!(
        "A{}{}{}{}",
        sn, reagent.short_prefix, control_no_number, control_no_suffix
    )
}

/// Long barcode: A{SN}{long_prefix}{control_no_number}{control_no_suffix}{expiry}{project_bits}{long_trailing}
pub fn build_long_content(
    sn: &str,
    reagent: &AbbottReagent,
    control_no_number: &str,
    control_no_suffix: &str,
    expiry_encoded: &str,
    project_bits_override: Option<&str>,
) -> String {
    let bits = project_bits_override.unwrap_or(&reagent.project_bits);
    format!(
        "A{}{}{}{}{}{}{}",
        sn,
        reagent.long_prefix,
        control_no_number,
        control_no_suffix,
        expiry_encoded,
        bits,
        reagent.long_trailing,
    )
}

// ---------------------------------------------------------------------------
// Barcode generation
// ---------------------------------------------------------------------------

pub struct AbbottBarcodeItem {
    pub label: String,
    pub content: String,
    pub slint_image: slint::Image,
    pub gray_image: image::GrayImage,
}

/// CompactPDF417, EC6, 2 columns, rotated 90°, 7.4×1.8 cm
fn short_config(content: &str) -> Config {
    Config {
        content: content.to_string(),
        format_index: 0,  // CompactPDF417
        scale_index: 1,   // 2x
        rotate_index: 1,  // 90°
        columns_index: 1, // 2 columns
        eclevel_index: 6,
        width_cm: 7.4,
        height_cm: 1.8,
        abbott_mode: false,
        abbott_project_index: 0,
    }
}

/// Standard PDF417, 4.0×2.0 cm
fn long_config(content: &str) -> Config {
    Config {
        content: content.to_string(),
        format_index: 1,  // PDF417
        scale_index: 1,   // 2x
        rotate_index: 0,  // 0°
        columns_index: 4, // columns=4
        eclevel_index: 2, // eclevel=2
        width_cm: 4.0,
        height_cm: 2.0,
        abbott_mode: false,
        abbott_project_index: 0,
    }
}

/// Generate all barcodes for an Abbott project.
///
/// - For reagents with `generates_long = true`: produce long barcode + short barcode
/// - For reagents with `generates_long = false`: produce short barcode only
///
/// `sns` maps 1:1 to `project.reagents` by index.
/// `project_bits_override`: if non-empty, overrides reagent's project_bits for long barcodes.
pub fn generate_abbott_barcodes(
    project: &AbbottProject,
    sns: &[String],
    control_no_number: &str,
    expiry: &str,
    project_bits_override: &str,
) -> Result<Vec<AbbottBarcodeItem>> {
    let expiry_encoded = encode_expiry(expiry, &project.expiry_format);
    let bits_override = if project_bits_override.is_empty() {
        None
    } else {
        Some(project_bits_override)
    };
    let mut items = Vec::new();

    for (i, reagent) in project.reagents.iter().enumerate() {
        let sn = sns.get(i).map(String::as_str).unwrap_or("");

        if reagent.generates_long {
            // Long barcode first
            let long_content = build_long_content(
                sn,
                reagent,
                control_no_number,
                &project.control_no_suffix,
                &expiry_encoded,
                bits_override,
            );
            let long_result = make_barcode_image(&long_config(&long_content))?;
            items.push(AbbottBarcodeItem {
                label: format!("{} 长码", reagent.name),
                content: long_content,
                slint_image: gray_to_slint_image(&long_result.gray_image),
                gray_image: long_result.gray_image,
            });
        }

        // Short barcode
        if reagent.generates_short {
            let short_content =
                build_short_content(sn, reagent, control_no_number, &project.control_no_suffix);
            let short_result = make_barcode_image(&short_config(&short_content))?;
            items.push(AbbottBarcodeItem {
                label: format!("{} 短码", reagent.name),
                content: short_content,
                slint_image: gray_to_slint_image(&short_result.gray_image),
                gray_image: short_result.gray_image,
            });
        }
    }

    Ok(items)
}

/// Export all generated barcodes to a directory.
pub fn export_abbott_barcodes(items: &[AbbottBarcodeItem], dir: &std::path::Path) -> Result<()> {
    fs::create_dir_all(dir)?;
    for (i, item) in items.iter().enumerate() {
        let safe_label = item.label.replace(' ', "_").replace('/', "_");
        let filename = format!("{:02}_{}.png", i + 1, safe_label);
        save_png_300dpi(&item.gray_image, dir.join(&filename))?;
    }
    Ok(())
}
