fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        use winres;
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/logo/app.ico");
        res.set_manifest_file("res/app.manifest");
        res.compile().expect("Failed to compile win resources")
    }

    // std::process::Command::new("cargo")
    //     .args([
    //         "clippy",
    //         "--all-targets",
    //         "--verbose",
    //         "--locked",
    //         "--",
    //         "-D",
    //         "warnings",
    //     ])
    //     .status()
    //     .unwrap();
}

// cargo clippy --all-targets --verbose --locked --features "${{ matrix.features }}" -- -D warnings
