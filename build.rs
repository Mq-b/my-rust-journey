fn main() {
    embed_resource::compile("./icon.rc");
    slint_build::compile_with_config(
        "ui/barcode.slint",
        slint_build::CompilerConfiguration::new().with_style("fluent".to_string()),
    )
    .unwrap();
}
