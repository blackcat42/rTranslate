extern crate embed_resource;
fn main() {
    embed_resource::compile("rt-manifest.rc"); //https://github.com/gabdube/native-windows-gui/issues/251#issuecomment-1451273346

    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon(r"dist\icons\tray_icon.ico");
        res.compile().unwrap();
    }
}