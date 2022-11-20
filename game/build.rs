// Include the resource definitions we share with the app.
include!("src/resources.rs");

fn main() {
    if !cfg!(target_os = "windows") {
        panic!("target OS was not Windows")
    }

    let mut res = ::winres::WindowsResource::new();
    res.set_icon_with_id(FERRIS_ICON.path(), &FERRIS_ICON.id_string());
    res.compile().unwrap();
}
