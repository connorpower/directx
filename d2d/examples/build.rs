fn main() {
    if !cfg!(target_os = "windows") {
        panic!("target OS was not Windows")
    }

    // Enable High-DPI support by bundling the `hdpi.manifest`
    ::winres::WindowsResource::new()
        .set_icon_with_id("icon.ico", "1")
        .set_manifest_file("hdpi.manifest")
        .compile()
        .unwrap();
}
