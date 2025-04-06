use std::hash::{Hash, Hasher};

fn main() {
    let output = std::process::Command::new("git")
        .arg("log")
        .arg("-1")
        .arg("--format=%H%n%cd%n%GK")
        .arg("--date=short")
        .output()
        .expect("Failed to execute git");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().lines().collect();

    if lines.len() >= 3 {
        let commit_hash = lines[0];
        let commit_date = lines[1];
        let key_id = if lines[2].is_empty() {
            "None"
        } else {
            lines[2]
        };

        println!("cargo:rustc-env=COMMIT_HASH={}", commit_hash);
        println!("cargo:rustc-env=COMMIT_DATE={}", commit_date);
        println!("cargo:rustc-env=COMMIT_KEY={}", key_id);
    } else {
        println!("cargo:rustc-env=COMMIT_HASH=Unknown");
        println!("cargo:rustc-env=COMMIT_DATE=Unknown");
        println!("cargo:rustc-env=COMMIT_KEY=None");
    }

    let source_hash = hash_me_baby();
    println!("cargo:rustc-env=SOURCE_HASH={}", source_hash);

    let target = std::env::var("TARGET").unwrap_or("Unknown".to_string());
    println!("cargo:rustc-env=BUILD_TARGET={}", target);

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        use winres;
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/logo/app.ico");
        res.set_manifest_file("res/app.manifest");
        res.compile().expect("Failed to compile win resources")
    }
}

fn hash_me_baby() -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    let mut paths = Vec::new();
    x_files(std::path::Path::new("."), &mut paths);
    paths.sort();

    for path in paths {
        if let Ok(contents) = std::fs::read(&path) {
            contents.hash(&mut hasher);
        }
    }

    format!("{:x}", hasher.finish())
}

fn x_files(dir: &std::path::Path, paths: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let path_str = path.to_string_lossy();
                if !path_str.contains(".git")
                    && !path_str.contains("target")
                    && !path_str.contains(".vscode")
                {
                    paths.push(path);
                }
            } else if path.is_dir() {
                x_files(&path, paths);
            }
        }
    }
}
