fn main() {
    // let win_resources = ["win"];
    // let output_directory = std::env::var("OUT_DIR").unwrap();

    // for dir in win_resources {
    //     let source_directory = std::path::PathBuf::from("res").join(dir);
    //     let destination_directory = std::path::PathBuf::from(&output_directory).join("../../../").join(dir);

    //     if source_directory.exists() {
    //         if let Err(err) = copy_dir_recursively(&source_directory, &destination_directory) {
    //             eprintln!("Failed to copy directory: {}\n\tError: {}", source_directory.display(), err);
    //             std::process::exit(1);
    //         }
    //     }

    // }

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        // bill_mainfest_gates();
        use winres;
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/app.ico");
        res.set_manifest_file("res/app.manifest");
        res.compile().expect("Failed to compile win resources")

    }
}

// fn copy_dir_recursively(source: &std::path::Path, destination: &std::path::Path) -> std::io::Result<()> {
//     if !destination.exists() {
//         std::fs::create_dir_all(destination)?;
//     }

//     for entry in std::fs::read_dir(source)? {
//         let entry = entry?;
//         let path = entry.path();
//         let dest_path = destination.join(entry.file_name());

//         if path.is_dir() {
//             copy_dir_recursively(&path, &dest_path)?;
//         } else {
//             std::fs::copy(&path, &dest_path)?;
//         }
//     }

//     Ok(())
// }

// fn bill_mainfest_gates() {
//     let build_directory = std::env::var("OUT_DIR").unwrap();
//     let bill_path = std::path::Path::new("res/win/app.manifest");
//     let rc_path = std::path::Path::new("res/win/app.rc");
//     let res_path = std::path::PathBuf::from(&build_directory).join("app.res");

//     if !bill_path.exists() {
//         eprintln!("Manifest file not found: {}", bill_path.display());
//         std::process::exit(1);
//     }

//     if cfg!(target_os = "windows") {
//         let status = std::process::Command::new("rc.exe")
//             .args(&["/fo", &res_path.to_string_lossy(), &rc_path.to_string_lossy()])
//             .status()
//             .expect("Failed to execute rc.exe");
        
//         if !status.success() {
//             eprintln!("Failed to compile .rc file into .res file");
//             std::process::exit(1);
//         }

//         println!("cargo:rustc-link-arg-bins={}", res_path.display());
//     } else {
//         let status = std::process::Command::new("x86_64-w64-mingw32-windres")
//             .args(&["-o", &res_path.to_string_lossy(), &rc_path.to_string_lossy()])
//             .status()
//             .expect("Failed to execute windres");
        
//         if !status.success() {
//             eprintln!("Failed to compile .rc file into .res file");
//             std::process::exit(1);
//         }

//         println!("cargo:rustc-link-arg-bins={}", res_path.display());
//     }
// }