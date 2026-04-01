use serde::{Deserialize, Serialize};
use std::path::Path;

pub const LAYOUT_CONFIG_PATH: &str = "Setting/rl-clia-layout.json";
pub const LABEL_WIDTH_MM: u32 = 55;
pub const LABEL_HEIGHT_MM: u32 = 45;
pub const OUTPUT_DPI: u32 = 300;
pub const OUTPUT_DPM: u32 = div_round(OUTPUT_DPI * 10_000, 254);
pub const LABEL_WIDTH_PX: u32 = div_round(LABEL_WIDTH_MM * OUTPUT_DPI * 10, 254);
pub const LABEL_HEIGHT_PX: u32 = div_round(LABEL_HEIGHT_MM * OUTPUT_DPI * 10, 254);
pub const LABEL_WIDTH: f32 = LABEL_WIDTH_PX as f32;
pub const LABEL_HEIGHT: f32 = LABEL_HEIGHT_PX as f32;
const LEGACY_LABEL_WIDTH: f32 = 660.0;
const LEGACY_LABEL_HEIGHT: f32 = 580.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageKind {
    Reagent,
    Calibration,
    Consumable,
    Quality,
}

impl PageKind {
    pub fn from_ui(value: &str) -> Self {
        match value {
            "calibration" => Self::Calibration,
            "consumable" => Self::Consumable,
            "quality" => Self::Quality,
            _ => Self::Reagent,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Reagent => "试剂",
            Self::Calibration => "校准品",
            Self::Consumable => "耗材",
            Self::Quality => "质控品",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub reagent: PageLayout,
    pub calibration: PageLayout,
    pub consumable: PageLayout,
    pub quality: PageLayout,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            reagent: reagent_layout(),
            calibration: calibration_layout(),
            consumable: consumable_layout(),
            quality: quality_layout(),
        }
    }
}

impl LayoutConfig {
    pub fn page(&self, page: PageKind) -> &PageLayout {
        match page {
            PageKind::Reagent => &self.reagent,
            PageKind::Calibration => &self.calibration,
            PageKind::Consumable => &self.consumable,
            PageKind::Quality => &self.quality,
        }
    }

    pub fn page_mut(&mut self, page: PageKind) -> &mut PageLayout {
        match page {
            PageKind::Reagent => &mut self.reagent,
            PageKind::Calibration => &mut self.calibration,
            PageKind::Consumable => &mut self.consumable,
            PageKind::Quality => &mut self.quality,
        }
    }

    pub fn reset_page(&mut self, page: PageKind) {
        *self.page_mut(page) = match page {
            PageKind::Reagent => reagent_layout(),
            PageKind::Calibration => calibration_layout(),
            PageKind::Consumable => consumable_layout(),
            PageKind::Quality => quality_layout(),
        };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLayout {
    pub elements: Vec<LayoutElement>,
}

impl PageLayout {
    pub fn element(&self, id: &str) -> Option<&LayoutElement> {
        self.elements.iter().find(|element| element.id == id)
    }

    pub fn element_mut(&mut self, id: &str) -> Option<&mut LayoutElement> {
        self.elements.iter_mut().find(|element| element.id == id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LayoutElementKind {
    Text,
    Barcode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutElement {
    pub id: String,
    pub kind: LayoutElementKind,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub bold: bool,
}

pub fn load_layout_config() -> LayoutConfig {
    let path = Path::new(LAYOUT_CONFIG_PATH);
    if let Ok(data) = std::fs::read_to_string(path) {
        if let Ok(mut cfg) = serde_json::from_str::<LayoutConfig>(&data) {
            migrate_config_if_needed(&mut cfg);
            return cfg;
        }
    }
    LayoutConfig::default()
}

pub fn save_layout_config(config: &LayoutConfig) -> Result<(), String> {
    if let Some(parent) = Path::new(LAYOUT_CONFIG_PATH).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {e}"))?;
    }
    let content =
        serde_json::to_string_pretty(config).map_err(|e| format!("布局配置序列化失败: {e}"))?;
    std::fs::write(LAYOUT_CONFIG_PATH, content).map_err(|e| format!("写入布局配置失败: {e}"))
}

fn reagent_layout() -> PageLayout {
    PageLayout {
        elements: vec![
            text(
                "title",
                sx(0.0),
                sy(18.0),
                LABEL_WIDTH,
                sy(44.0),
                ss(32.0),
                true,
            ),
            text(
                "subtitle1",
                sx(0.0),
                sy(60.0),
                LABEL_WIDTH,
                sy(30.0),
                ss(22.0),
                false,
            ),
            text(
                "subtitle2",
                sx(0.0),
                sy(88.0),
                LABEL_WIDTH,
                sy(24.0),
                ss(18.0),
                false,
            ),
            barcode("barcode", sx(30.0), sy(130.0), sx(600.0), sy(300.0)),
            text(
                "lot",
                sx(0.0),
                sy(450.0),
                LABEL_WIDTH,
                sy(28.0),
                ss(23.0),
                false,
            ),
            text(
                "prod_date",
                sx(0.0),
                sy(490.0),
                LABEL_WIDTH,
                sy(28.0),
                ss(23.0),
                false,
            ),
            text(
                "expire_date",
                sx(0.0),
                sy(530.0),
                LABEL_WIDTH,
                sy(28.0),
                ss(23.0),
                false,
            ),
        ],
    }
}

fn calibration_layout() -> PageLayout {
    PageLayout {
        elements: vec![
            text(
                "title",
                sx(0.0),
                sy(18.0),
                LABEL_WIDTH,
                sy(44.0),
                ss(32.0),
                true,
            ),
            text(
                "subtitle1",
                sx(0.0),
                sy(60.0),
                LABEL_WIDTH,
                sy(30.0),
                ss(24.0),
                false,
            ),
            barcode("barcode", sx(30.0), sy(98.0), sx(600.0), sy(300.0)),
            text(
                "lot",
                sx(0.0),
                sy(418.0),
                LABEL_WIDTH,
                sy(28.0),
                ss(23.0),
                false,
            ),
            text(
                "prod_date",
                sx(0.0),
                sy(458.0),
                LABEL_WIDTH,
                sy(28.0),
                ss(23.0),
                false,
            ),
            text(
                "expire_date",
                sx(0.0),
                sy(498.0),
                LABEL_WIDTH,
                sy(28.0),
                ss(23.0),
                false,
            ),
        ],
    }
}

fn consumable_layout() -> PageLayout {
    PageLayout {
        elements: vec![
            text(
                "title",
                sx(0.0),
                sy(18.0),
                LABEL_WIDTH,
                sy(44.0),
                ss(32.0),
                true,
            ),
            barcode("barcode", sx(30.0), sy(82.0), sx(600.0), sy(300.0)),
            text(
                "lot",
                sx(0.0),
                sy(402.0),
                LABEL_WIDTH,
                sy(28.0),
                ss(23.0),
                false,
            ),
            text(
                "prod_date",
                sx(0.0),
                sy(442.0),
                LABEL_WIDTH,
                sy(28.0),
                ss(23.0),
                false,
            ),
            text(
                "expire_date",
                sx(0.0),
                sy(482.0),
                LABEL_WIDTH,
                sy(28.0),
                ss(23.0),
                false,
            ),
        ],
    }
}

fn quality_layout() -> PageLayout {
    PageLayout {
        elements: vec![
            text(
                "title",
                sx(0.0),
                sy(18.0),
                LABEL_WIDTH,
                sy(44.0),
                ss(32.0),
                true,
            ),
            text(
                "subtitle1",
                sx(0.0),
                sy(60.0),
                LABEL_WIDTH,
                sy(30.0),
                ss(24.0),
                false,
            ),
            barcode("barcode", sx(30.0), sy(98.0), sx(600.0), sy(300.0)),
            text(
                "lot",
                sx(0.0),
                sy(418.0),
                LABEL_WIDTH,
                sy(28.0),
                ss(23.0),
                false,
            ),
            text(
                "prod_date",
                sx(0.0),
                sy(458.0),
                LABEL_WIDTH,
                sy(28.0),
                ss(23.0),
                false,
            ),
            text(
                "expire_date",
                sx(0.0),
                sy(498.0),
                LABEL_WIDTH,
                sy(28.0),
                ss(23.0),
                false,
            ),
        ],
    }
}

fn text(
    id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    font_size: f32,
    bold: bool,
) -> LayoutElement {
    LayoutElement {
        id: id.into(),
        kind: LayoutElementKind::Text,
        x,
        y,
        width,
        height,
        font_size,
        bold,
    }
}

fn barcode(id: &str, x: f32, y: f32, width: f32, height: f32) -> LayoutElement {
    LayoutElement {
        id: id.into(),
        kind: LayoutElementKind::Barcode,
        x,
        y,
        width,
        height,
        font_size: 0.0,
        bold: false,
    }
}

fn sx(value: f32) -> f32 {
    value * LABEL_WIDTH / LEGACY_LABEL_WIDTH
}

fn sy(value: f32) -> f32 {
    value * LABEL_HEIGHT / LEGACY_LABEL_HEIGHT
}

fn ss(value: f32) -> f32 {
    value * LABEL_HEIGHT / LEGACY_LABEL_HEIGHT
}

fn migrate_config_if_needed(config: &mut LayoutConfig) {
    migrate_page_if_needed(&mut config.reagent);
    migrate_page_if_needed(&mut config.calibration);
    migrate_page_if_needed(&mut config.consumable);
    migrate_page_if_needed(&mut config.quality);
}

fn migrate_page_if_needed(page: &mut PageLayout) {
    let needs_legacy_migration = page.elements.iter().any(|element| {
        element.width > LABEL_WIDTH + 0.5
            || element.height > LABEL_HEIGHT + 0.5
            || element.x + element.width > LABEL_WIDTH + 0.5
            || element.y + element.height > LABEL_HEIGHT + 0.5
    });

    if needs_legacy_migration {
        for element in &mut page.elements {
            element.x = sx(element.x);
            element.width = sx(element.width);
            element.y = sy(element.y);
            element.height = sy(element.height);
            if element.kind == LayoutElementKind::Text {
                element.font_size = ss(element.font_size);
            } else {
                element.font_size = 0.0;
                element.bold = false;
            }
        }
    }

    for element in &mut page.elements {
        element.width = element.width.clamp(1.0, LABEL_WIDTH);
        element.height = element.height.clamp(1.0, LABEL_HEIGHT);
        element.x = element.x.clamp(0.0, (LABEL_WIDTH - element.width).max(0.0));
        element.y = element
            .y
            .clamp(0.0, (LABEL_HEIGHT - element.height).max(0.0));
    }
}

const fn div_round(numerator: u32, denominator: u32) -> u32 {
    (numerator + denominator / 2) / denominator
}
