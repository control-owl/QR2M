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
