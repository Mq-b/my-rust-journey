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
    /// Prefix character used in short barcode (e.g. "H", "J", "L")
    pub short_prefix: String,
    /// Prefix character used in long barcode (e.g. "G")
    pub long_prefix: String,
    /// Fixed project identification bits appended in long barcode
    pub project_bits: String,
    /// Fixed trailing data appended after project_bits in long barcode
    pub long_trailing: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbbottProject {
    pub name: String,
    /// Suffix appended to control-no number (e.g. "UD00", "UN24")
    pub control_no_suffix: String,
    /// Default middle number pre-filled in the UI (e.g. "80001")
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
            AbbottProject {
                name: "CTNI".to_string(),
                control_no_suffix: "UD00".to_string(),
                control_no_default_number: "80001".to_string(),
                expiry_format: "00DDMMYYYY".to_string(),
                reagents: vec![AbbottReagent {
                    name: "CTNI".to_string(),
                    short_prefix: "H".to_string(),
                    long_prefix: "G".to_string(),
                    project_bits: "6201010300001".to_string(),
                    long_trailing: "H0000162001AAAG".to_string(),
                }],
            },
            AbbottProject {
                name: "CK-MB".to_string(),
                control_no_suffix: "UN24".to_string(),
                control_no_default_number: "80001".to_string(),
                expiry_format: "DDMMYYYY".to_string(),
                reagents: vec![AbbottReagent {
                    name: "CK-MB".to_string(),
                    short_prefix: "H".to_string(),
                    long_prefix: "G".to_string(),
                    project_bits: "0300062024701".to_string(),
                    long_trailing: "H0000000000AAAB".to_string(),
                }],
            },
            AbbottProject {
                name: "Myo".to_string(),
                control_no_suffix: "UN24".to_string(),
                control_no_default_number: "80001".to_string(),
                expiry_format: "DDMMYYYY".to_string(),
                reagents: vec![
                    AbbottReagent {
                        name: "Myo-1".to_string(),
                        short_prefix: "H".to_string(),
                        long_prefix: "G".to_string(),
                        project_bits: "4612010300002".to_string(),
                        long_trailing: "HJ000000000AAAA".to_string(),
                    },
                    AbbottReagent {
                        name: "Myo-2".to_string(),
                        short_prefix: "J".to_string(),
                        long_prefix: "G".to_string(),
                        project_bits: "4612010300002".to_string(),
                        long_trailing: "HJ000000000AAAA".to_string(),
                    },
                    AbbottReagent {
                        name: "Myo-3".to_string(),
                        short_prefix: "L".to_string(),
                        long_prefix: "G".to_string(),
                        project_bits: "4612010300002".to_string(),
                        long_trailing: "HJ000000000AAAA".to_string(),
                    },
                ],
            },
            AbbottProject {
                name: "BNP".to_string(),
                control_no_suffix: "UN24".to_string(),
                control_no_default_number: "80001".to_string(),
                expiry_format: "DDMMYYYY".to_string(),
                reagents: vec![
                    AbbottReagent {
                        name: "BNP-1".to_string(),
                        short_prefix: "H".to_string(),
                        long_prefix: "G".to_string(),
                        project_bits: "0000000000000".to_string(),
                        long_trailing: "H0000000000AAAC".to_string(),
                    },
                    AbbottReagent {
                        name: "BNP-2".to_string(),
                        short_prefix: "J".to_string(),
                        long_prefix: "G".to_string(),
                        project_bits: "0000000000000".to_string(),
                        long_trailing: "H0000000000AAAC".to_string(),
                    },
                    AbbottReagent {
                        name: "BNP-3".to_string(),
                        short_prefix: "L".to_string(),
                        long_prefix: "G".to_string(),
                        project_bits: "0000000000000".to_string(),
                        long_trailing: "H0000000000AAAC".to_string(),
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
pub fn encode_expiry(date_str: &str, fmt: &str) -> String {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return date_str.to_string();
    }
    let (year, month, day) = (parts[0], parts[1], parts[2]);
    match fmt {
        "DDMMYYYY" => format!("{}{}{}", day, month, year),
        "00DDMMYYYY" => format!("00{}{}{}", day, month, year),
        _ => format!("{}{}{}", day, month, year),
    }
}

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

pub fn build_long_content(
    sn: &str,
    reagent: &AbbottReagent,
    control_no_number: &str,
    control_no_suffix: &str,
    expiry_encoded: &str,
) -> String {
    format!(
        "A{}{}{}{}{}{}{}",
        sn,
        reagent.long_prefix,
        control_no_number,
        control_no_suffix,
        expiry_encoded,
        reagent.project_bits,
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
        columns_index: 1, // 2 columns (index 1 → columns+1 = 2)
        eclevel_index: 6,
        width_cm: 7.4,
        height_cm: 1.8,
        abbott_mode: false,
        abbott_project_index: 0,
    }
}

/// Standard PDF417, 2×4 cm
fn long_config(content: &str) -> Config {
    Config {
        content: content.to_string(),
        format_index: 1,  // PDF417
        scale_index: 1,   // 2x
        rotate_index: 0,  // 0°
        columns_index: 1, // 2 columns
        eclevel_index: 2, // default EC level
        width_cm: 2.0,
        height_cm: 4.0,
        abbott_mode: false,
        abbott_project_index: 0,
    }
}

/// Generate all barcodes for an Abbott project (short + long per reagent).
pub fn generate_abbott_barcodes(
    project: &AbbottProject,
    sns: &[String],   // one SN per reagent; if shorter, repeats last
    control_no_number: &str,
    expiry: &str,
) -> Result<Vec<AbbottBarcodeItem>> {
    let expiry_encoded = encode_expiry(expiry, &project.expiry_format);
    let mut items = Vec::new();

    for (i, reagent) in project.reagents.iter().enumerate() {
        let sn = sns.get(i).or_else(|| sns.last()).map(String::as_str).unwrap_or("");

        // Short barcode
        let short_content = build_short_content(sn, reagent, control_no_number, &project.control_no_suffix);
        let short_result = make_barcode_image(&short_config(&short_content))?;
        items.push(AbbottBarcodeItem {
            label: format!("{} - 短码", reagent.name),
            content: short_content,
            slint_image: gray_to_slint_image(&short_result.gray_image),
            gray_image: short_result.gray_image,
        });

        // Long barcode
        let long_content = build_long_content(
            sn,
            reagent,
            control_no_number,
            &project.control_no_suffix,
            &expiry_encoded,
        );
        let long_result = make_barcode_image(&long_config(&long_content))?;
        items.push(AbbottBarcodeItem {
            label: format!("{} - 长码", reagent.name),
            content: long_content,
            slint_image: gray_to_slint_image(&long_result.gray_image),
            gray_image: long_result.gray_image,
        });
    }

    Ok(items)
}

/// Export all generated barcodes to a directory.  Files are named
/// `{index:02}_{label}.png` (spaces replaced by underscores).
pub fn export_abbott_barcodes(items: &[AbbottBarcodeItem], dir: &std::path::Path) -> Result<()> {
    fs::create_dir_all(dir)?;
    for (i, item) in items.iter().enumerate() {
        let safe_label = item.label.replace(' ', "_").replace('/', "_");
        let filename = format!("{:02}_{}.png", i + 1, safe_label);
        save_png_300dpi(&item.gray_image, dir.join(&filename))?;
    }
    Ok(())
}
