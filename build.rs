use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::io;

fn main() {
    let resources = ["config", "lib", "locale", "res"];
    let output_directory = env::var("OUT_DIR").unwrap();

    for dir in resources {
        let source_directory = PathBuf::from(dir);
        let destination_directory = PathBuf::from(&output_directory).join("../../../").join(dir);

        if source_directory.exists() {
            if let Err(err) = copy_dir_recursively(&source_directory, &destination_directory) {
                eprintln!("Failed to copy directory: {}\n\tError: {}", source_directory.display(), err);
                std::process::exit(1);
            }
        }

        if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
            bill_mainfest_gates();
        }
    }
}

fn copy_dir_recursively(source: &Path, destination: &Path) -> io::Result<()> {
    if !destination.exists() {
        fs::create_dir_all(destination)?;
    }

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = destination.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursively(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

fn bill_mainfest_gates() {
    let build_directory = env::var("OUT_DIR").unwrap();
    let bill_path = Path::new("app.manifest");
    let rc_path = PathBuf::from(&build_directory).join("app.rc");
    let res_path = PathBuf::from(&build_directory).join("app.res");

    if !bill_path.exists() {
        eprintln!("Manifest file not found: {}", bill_path.display());
        std::process::exit(1);
    }

    let rc_content = format!(r#"1 24 "{}""#, bill_path.display());
    fs::write(&rc_path, rc_content).expect("Failed to write .rc file");

    let status = std::process::Command::new("x86_64-w64-mingw32-windres")
        .args(&["-o", &res_path.to_string_lossy(), &rc_path.to_string_lossy()])
        .status()
        .expect("Failed to execute windres");

    if !status.success() {
        eprintln!("Failed to compile .rc file into .res file");
        std::process::exit(1);
    }

    println!("cargo:rustc-link-arg-bins={}", res_path.display());

    // fribi pgp missing keys
    // again windows compile failed
    // fcking gates
}