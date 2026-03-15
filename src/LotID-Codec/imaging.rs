pub fn make_datamatrix(text: &str) -> anyhow::Result<image::GrayImage> {
    use zxingcpp::*;
    let barcode = create(BarcodeFormat::DataMatrix).from_str(text)?;
    let img = barcode.to_image_with(&write().scale(6).add_quiet_zones(true).add_hrt(false))?;
    Ok(image::GrayImage::from(&img))
}

pub fn gray_to_slint(gray: &image::GrayImage) -> slint::Image {
    let (w, h) = (gray.width(), gray.height());
    let pixels: Vec<u8> = gray
        .pixels()
        .flat_map(|p| [p[0], p[0], p[0], 255u8])
        .collect();
    let buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(&pixels, w, h);
    slint::Image::from_rgba8(buf)
}

pub fn copy_gray_to_clipboard(gray: &image::GrayImage) -> Result<(), String> {
    let rgba: Vec<u8> = gray
        .pixels()
        .flat_map(|p| [p[0], p[0], p[0], 255u8])
        .collect();
    arboard::Clipboard::new()
        .map_err(|e| e.to_string())?
        .set_image(arboard::ImageData {
            width: gray.width() as usize,
            height: gray.height() as usize,
            bytes: std::borrow::Cow::Owned(rgba),
        })
        .map_err(|e| e.to_string())
}
