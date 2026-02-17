fn main() {
    embed_resource::compile("./icon.rc");
    slint_build::compile("ui/barcode.slint").unwrap();
}
