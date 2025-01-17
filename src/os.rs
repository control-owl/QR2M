// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// module = "OS specific tasks"
// copyright = "Copyright © 2023-2025 Control Owl"
// version = "2024-12-09"


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


use std::{
    env,
    path::{Path, PathBuf},
    fs,
    io::{self},
};
use crate::APP_NAME;

const APP_LOCAL_CONFIG_FILE: &str = "qr2m.conf";
const APP_LOCAL_TEMP_FILE: &str = "qr2m.log";

lazy_static::lazy_static! {
    pub static ref LOCAL_SETTINGS: std::sync::Arc<std::sync::Mutex<LocalSettings>> = std::sync::Arc::new(std::sync::Mutex::new(LocalSettings::default()));
}

#[derive(Debug, Default)]
pub struct LocalSettings {
    pub os: Option<String>,
    pub local_config_dir: Option<PathBuf>,
    pub local_config_file: Option<PathBuf>,
    pub local_temp_dir: Option<PathBuf>,
    pub local_temp_file: Option<PathBuf>,
    // pub local_do_not_show_file: Option<PathBuf>,
}


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


pub fn detect_os_and_user_dir() {
    println!("[+] {}", &t!("log.detecting-local-os").to_string());

    let os = match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "linux",
        _ => "unknown",
    }.to_string();

    let app_name = APP_NAME.unwrap();

    let local_temp = env::temp_dir();
    let local_temp_dir = local_temp.join(app_name);
    let local_temp_file = local_temp_dir.join(APP_LOCAL_TEMP_FILE);

    let local_config_dir = match os.as_str() {
        "windows" => {
            // C:\Users\<Username>\AppData\Roaming\<AppName>\
            let mut path = PathBuf::from(env::var("APPDATA").unwrap_or_else(|_| "C:\\".to_string()));
            path.push(app_name);
            path
        },
        "linux" => {
            // /home/<Username>/.config/<AppName>/
            let mut path = PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/".to_string()));
            path.push(".config");
            path.push(app_name);
            path
        },
        "macos" => {
            // /home/<Username>/<AppName>/
            let mut path = PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/".to_string()));
            path.push(app_name);
            path
        },
        _ => PathBuf::from("/"),
    };

    let local_config_file = local_config_dir.join(APP_LOCAL_CONFIG_FILE);

    let (config_dir, config_file) = if local_config_dir.is_symlink() {
        match fs::read_link(&local_config_dir) {
            Ok(target) => {
                if target.is_dir() {
                    if fs::metadata(&target).map(|m| m.permissions().readonly()).unwrap_or(true) {
                        println!("[!] Symlink target is not writable: {:?}", &target);
                        (local_temp_dir.clone(), local_temp_dir.join(APP_LOCAL_CONFIG_FILE))
                    } else {
                        println!("[+] Using writable symlink target: {:?}", &target);
                        (target.clone(), target.join(APP_LOCAL_CONFIG_FILE))
                    }
                } else {
                    println!("[!] Symlink does not point to a directory: {:?}", &target);
                    (local_temp_dir.clone(), local_temp_dir.join(APP_LOCAL_CONFIG_FILE))
                }
            },
            Err(e) => {
                println!("[!] Failed to read symlink target: {:?}, Error: {}", &local_config_dir, e);
                (local_temp_dir.clone(), local_temp_dir.join(APP_LOCAL_CONFIG_FILE))
            }
        }
    } else {
        (local_config_dir, local_config_file)
    };

    let mut local_settings = LOCAL_SETTINGS.lock().unwrap();
    local_settings.os = Some(os.clone());
    local_settings.local_config_dir = Some(config_dir.clone());
    local_settings.local_temp_dir = Some(local_temp_dir.clone());
    local_settings.local_config_file = Some(config_file.clone());
    local_settings.local_temp_file = Some(local_temp_file.clone());

    println!("\t OS: {:?}", &os);
    println!("\t Config directory: {:?}", &config_dir);
    println!("\t Configuration file: {:?}", &config_file);
    println!("\t Temp directory: {:?}", &local_temp_dir);
    println!("\t Temp file: {:?}", &local_temp_file);
}

pub fn switch_locale(lang: &str) {
    match lang {
        "Deutsch" => rust_i18n::set_locale("de"),
        "Hrvatski" => rust_i18n::set_locale("hr"),
        "English"| _ => rust_i18n::set_locale("en"),
    }
}

pub fn create_local_files() -> Result<(), Box<dyn std::error::Error>> {
    
    let local_settings = LOCAL_SETTINGS.lock().unwrap();
    let local_config_file = local_settings.local_config_file.clone().unwrap();
    let local_config_dir = local_settings.local_config_dir.clone().unwrap();
    
    if !local_config_dir.exists() {
        eprintln!("Local config directory not found. Creating it.");
        fs::create_dir_all(&local_config_dir)?;
    }
    
    if !is_directory_writable(&local_config_dir)? {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Directory not writable").into());
    }

    if !Path::new(&local_config_file).exists() {
        eprintln!(
            "Local config file '{:?}' does not exist. Creating it from the default configuration.",
            local_config_file
        );

        let default_config = qr2m_lib::get_text_from_resources(crate::APP_DEFAULT_CONFIG_FILE);

        if default_config.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Default configuration is empty or missing in resources",
            )
            .into());
        }

        fs::write(&local_config_file, default_config)?;
        println!("Local config file created successfully.");
    }

    Ok(())
}


fn is_directory_writable(dir: &Path) -> Result<bool, io::Error> {
    let mut temp_file_path = dir.to_path_buf();
    temp_file_path.push(".tmp");

    match fs::File::create(&temp_file_path) {
        Ok(_) => {
            if let Err(err) = fs::remove_file(&temp_file_path) {
                eprintln!("Failed to delete temporary file: {}", err);
            }
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.