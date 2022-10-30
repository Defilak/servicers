
#[cfg(windows)]
fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("favicon.ico")
        .set("InternalName", env!("CARGO_PKG_NAME"))
        .set("OriginalFilename", env!("CARGO_PKG_NAME"))
        .set("FileDescription", env!("CARGO_PKG_DESCRIPTION"))
        .set("LegalCopyright", env!("CARGO_PKG_LICENSE"))
        .set("ProductVersion", env!("CARGO_PKG_VERSION"));

        res.compile().unwrap();
    }
}
