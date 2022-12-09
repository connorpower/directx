// Include the resource definitions we share with the app.
include!("src/resources.rs");

fn main() {
    if !cfg!(target_os = "windows") {
        panic!("target OS was not Windows")
    }

    ::winres::WindowsResource::new()
        .set_icon_with_id(FERRIS_ICON.path(), &FERRIS_ICON.id_string())
        .set_manifest_file("game.manifest")
        .compile()
        .unwrap();
}
