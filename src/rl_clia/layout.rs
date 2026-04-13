use serde::{Deserialize, Serialize};
use std::path::Path;

pub const LAYOUT_CONFIG_PATH: &str = "Setting/rl-clia-layout.json";
pub const DEFAULT_LABEL_WIDTH_MM: u32 = 55;
pub const DEFAULT_LABEL_HEIGHT_MM: u32 = 45;
const MIN_LABEL_MM: u32 = 1;
const MAX_LABEL_MM: u32 = 300;
pub const OUTPUT_DPI: u32 = 300;
pub const OUTPUT_DPM: u32 = div_round(OUTPUT_DPI * 10_000, 254);
const LEGACY_LABEL_WIDTH: f32 = 660.0;
const LEGACY_LABEL_HEIGHT: f32 = 580.0;

fn default_label_width_mm() -> u32 {
    DEFAULT_LABEL_WIDTH_MM
}

fn default_label_height_mm() -> u32 {
    DEFAULT_LABEL_HEIGHT_MM
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelSize {
    #[serde(default = "default_label_width_mm")]
    pub width_mm: u32,
    #[serde(default = "default_label_height_mm")]
    pub height_mm: u32,
}

impl Default for LabelSize {
    fn default() -> Self {
        Self {
            width_mm: DEFAULT_LABEL_WIDTH_MM,
            height_mm: DEFAULT_LABEL_HEIGHT_MM,
        }
    }
}

impl LabelSize {
    pub fn normalized(self) -> Self {
        Self {
            width_mm: self.width_mm.clamp(MIN_LABEL_MM, MAX_LABEL_MM),
            height_mm: self.height_mm.clamp(MIN_LABEL_MM, MAX_LABEL_MM),
        }
    }

    pub fn width_px(self) -> u32 {
        div_round(self.normalized().width_mm * OUTPUT_DPI * 10, 254)
    }

    pub fn height_px(self) -> u32 {
        div_round(self.normalized().height_mm * OUTPUT_DPI * 10, 254)
    }

    pub fn width(self) -> f32 {
        self.width_px() as f32
    }

    pub fn height(self) -> f32 {
        self.height_px() as f32
    }
}

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
    #[serde(default)]
    pub label_size: LabelSize,
    pub reagent: PageLayout,
    pub calibration: PageLayout,
    pub consumable: PageLayout,
    pub quality: PageLayout,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        let label_size = LabelSize::default().normalized();
        Self {
            label_size,
            reagent: reagent_layout(label_size),
            calibration: calibration_layout(label_size),
            consumable: consumable_layout(label_size),
            quality: quality_layout(label_size),
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
        let size = self.label_size.normalized();
        *self.page_mut(page) = match page {
            PageKind::Reagent => reagent_layout(size),
            PageKind::Calibration => calibration_layout(size),
            PageKind::Consumable => consumable_layout(size),
            PageKind::Quality => quality_layout(size),
        };
    }

    pub fn label_width_px(&self) -> u32 {
        self.label_size.normalized().width_px()
    }

    pub fn label_height_px(&self) -> u32 {
        self.label_size.normalized().height_px()
    }

    pub fn label_width(&self) -> f32 {
        self.label_size.normalized().width()
    }

    pub fn label_height(&self) -> f32 {
        self.label_size.normalized().height()
    }

    pub fn set_label_size_mm(&mut self, width_mm: u32, height_mm: u32) -> bool {
        let old = self.label_size.normalized();
        let new = LabelSize {
            width_mm,
            height_mm,
        }
        .normalized();

        if old == new {
            return false;
        }

        let scale_x = new.width() / old.width();
        let scale_y = new.height() / old.height();
        scale_page(&mut self.reagent, scale_x, scale_y);
        scale_page(&mut self.calibration, scale_x, scale_y);
        scale_page(&mut self.consumable, scale_x, scale_y);
        scale_page(&mut self.quality, scale_x, scale_y);

        self.label_size = new;
        self.normalize_pages();
        true
    }

    fn normalize_pages(&mut self) {
        self.label_size = self.label_size.normalized();
        let label_w = self.label_width();
        let label_h = self.label_height();
        normalize_page(&mut self.reagent, label_w, label_h);
        normalize_page(&mut self.calibration, label_w, label_h);
        normalize_page(&mut self.consumable, label_w, label_h);
        normalize_page(&mut self.quality, label_w, label_h);
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
            cfg.label_size = cfg.label_size.normalized();
            migrate_config_if_needed(&mut cfg);
            cfg.normalize_pages();
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

fn reagent_layout(size: LabelSize) -> PageLayout {
    let label_w = size.width();
    let label_h = size.height();
    PageLayout {
        elements: vec![
            text(
                "title",
                sx(0.0, label_w),
                sy(18.0, label_h),
                label_w,
                sy(44.0, label_h),
                ss(32.0, label_h),
                true,
            ),
            text(
                "subtitle1",
                sx(0.0, label_w),
                sy(60.0, label_h),
                label_w,
                sy(30.0, label_h),
                ss(22.0, label_h),
                false,
            ),
            text(
                "subtitle2",
                sx(0.0, label_w),
                sy(88.0, label_h),
                label_w,
                sy(24.0, label_h),
                ss(18.0, label_h),
                false,
            ),
            barcode(
                "barcode",
                sx(30.0, label_w),
                sy(130.0, label_h),
                sx(600.0, label_w),
                sy(300.0, label_h),
            ),
            text(
                "lot",
                sx(0.0, label_w),
                sy(450.0, label_h),
                label_w,
                sy(28.0, label_h),
                ss(23.0, label_h),
                false,
            ),
            text(
                "prod_date",
                sx(0.0, label_w),
                sy(490.0, label_h),
                label_w,
                sy(28.0, label_h),
                ss(23.0, label_h),
                false,
            ),
            text(
                "expire_date",
                sx(0.0, label_w),
                sy(530.0, label_h),
                label_w,
                sy(28.0, label_h),
                ss(23.0, label_h),
                false,
            ),
        ],
    }
}

fn calibration_layout(size: LabelSize) -> PageLayout {
    let label_w = size.width();
    let label_h = size.height();
    PageLayout {
        elements: vec![
            text(
                "title",
                sx(0.0, label_w),
                sy(18.0, label_h),
                label_w,
                sy(44.0, label_h),
                ss(32.0, label_h),
                true,
            ),
            text(
                "subtitle1",
                sx(0.0, label_w),
                sy(60.0, label_h),
                label_w,
                sy(30.0, label_h),
                ss(24.0, label_h),
                false,
            ),
            barcode(
                "barcode",
                sx(30.0, label_w),
                sy(98.0, label_h),
                sx(600.0, label_w),
                sy(300.0, label_h),
            ),
            text(
                "lot",
                sx(0.0, label_w),
                sy(418.0, label_h),
                label_w,
                sy(28.0, label_h),
                ss(23.0, label_h),
                false,
            ),
            text(
                "prod_date",
                sx(0.0, label_w),
                sy(458.0, label_h),
                label_w,
                sy(28.0, label_h),
                ss(23.0, label_h),
                false,
            ),
            text(
                "expire_date",
                sx(0.0, label_w),
                sy(498.0, label_h),
                label_w,
                sy(28.0, label_h),
                ss(23.0, label_h),
                false,
            ),
        ],
    }
}

fn consumable_layout(size: LabelSize) -> PageLayout {
    let label_w = size.width();
    let label_h = size.height();
    PageLayout {
        elements: vec![
            text(
                "title",
                sx(0.0, label_w),
                sy(18.0, label_h),
                label_w,
                sy(44.0, label_h),
                ss(32.0, label_h),
                true,
            ),
            barcode(
                "barcode",
                sx(30.0, label_w),
                sy(82.0, label_h),
                sx(600.0, label_w),
                sy(300.0, label_h),
            ),
            text(
                "lot",
                sx(0.0, label_w),
                sy(402.0, label_h),
                label_w,
                sy(28.0, label_h),
                ss(23.0, label_h),
                false,
            ),
            text(
                "prod_date",
                sx(0.0, label_w),
                sy(442.0, label_h),
                label_w,
                sy(28.0, label_h),
                ss(23.0, label_h),
                false,
            ),
            text(
                "expire_date",
                sx(0.0, label_w),
                sy(482.0, label_h),
                label_w,
                sy(28.0, label_h),
                ss(23.0, label_h),
                false,
            ),
        ],
    }
}

fn quality_layout(size: LabelSize) -> PageLayout {
    let label_w = size.width();
    let label_h = size.height();
    PageLayout {
        elements: vec![
            text(
                "title",
                sx(0.0, label_w),
                sy(18.0, label_h),
                label_w,
                sy(44.0, label_h),
                ss(32.0, label_h),
                true,
            ),
            text(
                "subtitle1",
                sx(0.0, label_w),
                sy(60.0, label_h),
                label_w,
                sy(30.0, label_h),
                ss(24.0, label_h),
                false,
            ),
            barcode(
                "barcode",
                sx(30.0, label_w),
                sy(98.0, label_h),
                sx(600.0, label_w),
                sy(300.0, label_h),
            ),
            text(
                "lot",
                sx(0.0, label_w),
                sy(418.0, label_h),
                label_w,
                sy(28.0, label_h),
                ss(23.0, label_h),
                false,
            ),
            text(
                "prod_date",
                sx(0.0, label_w),
                sy(458.0, label_h),
                label_w,
                sy(28.0, label_h),
                ss(23.0, label_h),
                false,
            ),
            text(
                "expire_date",
                sx(0.0, label_w),
                sy(498.0, label_h),
                label_w,
                sy(28.0, label_h),
                ss(23.0, label_h),
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

fn sx(value: f32, label_width: f32) -> f32 {
    value * label_width / LEGACY_LABEL_WIDTH
}

fn sy(value: f32, label_height: f32) -> f32 {
    value * label_height / LEGACY_LABEL_HEIGHT
}

fn ss(value: f32, label_height: f32) -> f32 {
    value * label_height / LEGACY_LABEL_HEIGHT
}

fn migrate_config_if_needed(config: &mut LayoutConfig) {
    let label_w = config.label_width();
    let label_h = config.label_height();
    migrate_page_if_needed(&mut config.reagent, label_w, label_h);
    migrate_page_if_needed(&mut config.calibration, label_w, label_h);
    migrate_page_if_needed(&mut config.consumable, label_w, label_h);
    migrate_page_if_needed(&mut config.quality, label_w, label_h);
}

fn migrate_page_if_needed(page: &mut PageLayout, label_w: f32, label_h: f32) {
    let needs_legacy_migration = page.elements.iter().any(|element| {
        element.width > label_w + 0.5
            || element.height > label_h + 0.5
            || element.x + element.width > label_w + 0.5
            || element.y + element.height > label_h + 0.5
    });

    if needs_legacy_migration {
        for element in &mut page.elements {
            element.x = sx(element.x, label_w);
            element.width = sx(element.width, label_w);
            element.y = sy(element.y, label_h);
            element.height = sy(element.height, label_h);
            if element.kind == LayoutElementKind::Text {
                element.font_size = ss(element.font_size, label_h);
            } else {
                element.font_size = 0.0;
                element.bold = false;
            }
        }
    }
}

fn scale_page(page: &mut PageLayout, scale_x: f32, scale_y: f32) {
    for element in &mut page.elements {
        element.x *= scale_x;
        element.width *= scale_x;
        element.y *= scale_y;
        element.height *= scale_y;
        if element.kind == LayoutElementKind::Text {
            element.font_size *= scale_y;
        }
    }
}

fn normalize_page(page: &mut PageLayout, label_w: f32, label_h: f32) {
    for element in &mut page.elements {
        element.width = element.width.clamp(1.0, label_w);
        element.height = element.height.clamp(1.0, label_h);
        if element.kind == LayoutElementKind::Text {
            element.font_size = element.font_size.clamp(1.0, 200.0);
        } else {
            element.font_size = 0.0;
            element.bold = false;
        }
        element.x = element.x.clamp(0.0, (label_w - element.width).max(0.0));
        element.y = element.y.clamp(0.0, (label_h - element.height).max(0.0));
    }
}

const fn div_round(numerator: u32, denominator: u32) -> u32 {
    (numerator + denominator / 2) / denominator
}
