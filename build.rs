fn main() {
    //     let _ = std::fs::remove_dir_all("sec/");
    //
    //     println!("cargo:rerun-if-changed=src/");
    //     println!("cargo:rerun-if-changed=res/");

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        use winres;
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/logo/app.ico");
        res.set_manifest_file("res/app.manifest");
        res.compile().expect("Failed to compile win resources")
    }
}
