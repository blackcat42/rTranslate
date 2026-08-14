extern crate embed_resource;
fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    if target_os == "windows" && target_arch == "x86" {
    //if cfg!(all(target_os = "windows", target_arch = "x86")) {
        thunk::thunk();
    }
    embed_resource::compile("rt-manifest.rc", embed_resource::NONE); //https://github.com/gabdube/native-windows-gui/issues/251#issuecomment-1451273346

    if target_os == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon(r"dist\icons\tray_icon.ico");
        res.set_icon_with_id(r"dist\icons\tray_icon.ico", "tray-default");
        res.compile().unwrap();
    }
}
