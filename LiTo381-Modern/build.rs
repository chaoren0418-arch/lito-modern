fn main() {
    println!("cargo:rerun-if-changed=windows/app.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winres::WindowsResource::new()
            .set_manifest_file("windows/app.manifest")
            .set("ProductName", "LiTo Modern")
            .set("FileDescription", "Standalone modern LiTo client")
            .compile()
            .expect("compile standalone LiTo Windows manifest");
    }
}
