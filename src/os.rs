// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// module = "OS specific tasks"
// copyright = "Copyright © 2023-2024 D3BUG"
// version = "2024-06-16"


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


use std::env;
use std::path::PathBuf;
use crate::APP_NAME;


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


pub fn detect_os_and_user_dir() -> (String, PathBuf) {
    let os = if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "macos".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else if cfg!(target_os = "android") {
        "android".to_string()
    } else if cfg!(target_os = "ios") {
        "ios".to_string()
    } else if cfg!(target_os = "freebsd") {
        "freebsd".to_string()
    } else if cfg!(target_os = "dragonfly") {
        "dragonfly".to_string()
    } else if cfg!(target_os = "openbsd") {
        "openbsd".to_string()
    } else if cfg!(target_os = "netbsd") {
        "netbsd".to_string()
    } else if cfg!(target_os = "solaris") {
        "solaris".to_string()
    } else if cfg!(target_os = "redox") {
        "redox".to_string()
    } else {
        "unknown".to_string()
    };

    let user_dir = match os.as_str() {
        "windows" => {
            let mut path = env::var("USERPROFILE").unwrap_or("C:\\Users\\Default".to_string());
            path.push_str(&format!("\\{}\\", APP_NAME.unwrap()));
            PathBuf::from(path)
        },
        "linux" => {
            let mut path = env::var("HOME").unwrap_or("/".to_string());
            path.push_str(&format!("/.config/{}/", APP_NAME.unwrap()));
            PathBuf::from(path)
        },
        "android" | "ios" | "macos" | "freebsd" | "dragonfly" | "openbsd" | "netbsd" | "solaris" | "redox" => {
            let mut path = env::var("HOME").unwrap_or("/".to_string());
            path.push_str(&format!("/{}/", APP_NAME.unwrap()));
            PathBuf::from(path)
        },
        _ => PathBuf::from("/"),
    };

    (os, user_dir)
}