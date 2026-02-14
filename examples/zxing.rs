use image;
use show_image::{ImageInfo, ImageView, create_window};
use zxingcpp::*;

#[show_image::main]
fn main() -> anyhow::Result<()> {
    let input = "A06975H91015UN24";

    // 创建 Compact PDF417 条码配置选项：列数为 2，纠错级别为 6
    let barcode = create(BarcodeFormat::CompactPDF417)
        .options("columns:2,eclevel:6")
        .from_str(input)?;

    println!("Creating Compact PDF417 barcode for text '{}'", input);

    // 设置写入选项：缩放倍数为 2，旋转 90 度
    let img = barcode.to_image_with(
        &write()
            .scale(2) // 缩放倍数
            .add_quiet_zones(true) // 添加静默区
            .add_hrt(false) // 不添加人类可读文本
            .rotate(90),
    )?; // 旋转 90 度

    // 转换为 image crate 的灰度图像
    let gray_image = image::GrayImage::from(&img);

    // 保存图像
    gray_image.save("out.png")?;
    println!("Barcode saved to out.png");
    println!("Image size: {}x{}", img.width(), img.height());

    // 显示图像
    let window = create_window("PDF417 Barcode", Default::default())?;
    window.set_image(
        "barcode",
        ImageView::new(
            ImageInfo::mono8(gray_image.width(), gray_image.height()),
            &gray_image,
        ),
    )?;
    println!("Press any key or close the window to exit...");
    window.wait_until_destroyed()?;

    Ok(())
}
