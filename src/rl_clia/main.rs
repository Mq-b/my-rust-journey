#![windows_subsystem = "windows"]
mod app;
mod barcode;
mod config;
mod encryptor;

fn main() {
    app::run();
}
