use crate::config::Config;

pub struct BarcodeResult {
    pub gray_image: image::GrayImage,
    pub width: u32,
    pub height: u32,
    pub format_name: String,
}

pub fn generate_barcode(config: &Config) -> anyhow::Result<BarcodeResult> {
    use zxingcpp::*;

    const FORMATS: [&str; 8] = [
        "CompactPDF417",
        "PDF417",
        "QRCode",
        "DataMatrix",
        "Code128",
        "Code39",
        "Aztec",
        "EAN13",
    ];
    const SCALES: [i32; 5] = [1, 2, 3, 4, 5];
    const ROTATES: [i32; 4] = [0, 90, 180, 270];

    let format_name = FORMATS
        .get(config.format_index)
        .copied()
        .unwrap_or("CompactPDF417")
        .to_string();

    let format = match format_name.as_str() {
        "CompactPDF417" => BarcodeFormat::CompactPDF417,
        "PDF417" => BarcodeFormat::PDF417,
        "QRCode" => BarcodeFormat::QRCode,
        "DataMatrix" => BarcodeFormat::DataMatrix,
        "Code128" => BarcodeFormat::Code128,
        "Code39" => BarcodeFormat::Code39,
        "Aztec" => BarcodeFormat::Aztec,
        "EAN13" => BarcodeFormat::EAN13,
        _ => BarcodeFormat::CompactPDF417,
    };

    let columns = config.columns_index + 1;
    let eclevel = config.eclevel_index;
    let scale = SCALES.get(config.scale_index).copied().unwrap_or(2);
    let rotate = ROTATES.get(config.rotate_index).copied().unwrap_or(0);
    let options = format!("columns:{},eclevel:{}", columns, eclevel);

    let barcode = create(format).options(&options).from_str(&config.content)?;

    let img = barcode.to_image_with(
        &write()
            .scale(scale)
            .add_quiet_zones(true)
            .add_hrt(false)
            .rotate(rotate),
    )?;

    let mut gray_image = image::GrayImage::from(&img);
    // 按物理尺寸缩放（300 DPI）
    if config.width_cm > 0.0 && config.height_cm > 0.0 {
        let target_w = (config.width_cm / 2.54 * 300.0).round() as u32;
        let target_h = (config.height_cm / 2.54 * 300.0).round() as u32;
        if target_w > 0 && target_h > 0 {
            gray_image = image::imageops::resize(
                &gray_image,
                target_w,
                target_h,
                image::imageops::FilterType::Nearest,
            );
            println!(
                "Scaled to {}x{} px for {} cm x {} cm at 300 DPI",
                target_w, target_h, config.width_cm, config.height_cm
            );
        }
    }
    let width = gray_image.width();
    let height = gray_image.height();
    save_png_300dpi(&gray_image, "out.png")?;

    Ok(BarcodeResult {
        gray_image,
        width,
        height,
        format_name,
    })
}

/// 将灰度图转换为 Slint 可用的图像
pub fn gray_to_slint_image(gray: &image::GrayImage) -> slint::Image {
    let w = gray.width();
    let h = gray.height();
    let pixels: Vec<u8> = gray
        .pixels()
        .flat_map(|p| [p[0], p[0], p[0], 255u8])
        .collect();
    let buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(&pixels, w, h);
    slint::Image::from_rgba8(buffer)
}

/// 以 300 DPI 元数据保存灰度 PNG（pHYs chunk: 11811 像素/米）
pub fn save_png_300dpi(
    gray: &image::GrayImage,
    path: impl AsRef<std::path::Path>,
) -> anyhow::Result<()> {
    // 300 DPI → DPM = 300 / 0.0254 ≈ 11811 像素/米
    const PIXELS_PER_METER: u32 = 11811;

    let file = std::fs::File::create(path)?;
    let buf = std::io::BufWriter::new(file);

    let mut encoder = png::Encoder::new(buf, gray.width(), gray.height());
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_pixel_dims(Some(png::PixelDimensions {
        xppu: PIXELS_PER_METER,
        yppu: PIXELS_PER_METER,
        unit: png::Unit::Meter,
    }));

    let mut writer = encoder.write_header()?;
    writer.write_image_data(gray.as_raw())?;

    Ok(())
}
