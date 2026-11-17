fn main() {
    slint_build::compile("ui/app.slint").expect("Slint UI");
    // The exe's own icon and version, as Explorer and Task Manager show them.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("mantel.ico");
        res.set("ProductName", "Mantel WsQ");
        res.set("FileDescription", "Mantel WsQ");
        if let Err(e) = res.compile() {
            println!("cargo:warning=no icon resource: {e}");
        }
    }
}
