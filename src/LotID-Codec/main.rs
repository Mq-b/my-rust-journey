#![windows_subsystem = "windows"]

mod callbacks;
mod codec;
mod imaging;

slint::include_modules!();

fn main() {
    let window = LotIdWindow::new().unwrap();
    let last_gray: std::sync::Arc<std::sync::Mutex<Option<image::GrayImage>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));

    callbacks::setup_callbacks(&window, last_gray);

    window.run().unwrap();
}
