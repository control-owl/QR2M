// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// module = "OS specific tasks"
// copyright = "Copyright © 2023-2024 D3BUG"
// version = "2024-06-16"


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


use std::{
    env,
    path::{Path, PathBuf},
    fs,
    io,
};
use crate::APP_NAME;

const APP_LOCAL_CONFIG_FILE: &str = "settings.conf";
const APP_LOCAL_TEMP_FILE: &str = "qr2m.log";
const APP_DEFAULT_CONFIG_FILE: &str = "config/default.conf";

thread_local! {
    pub static LOCAL_DATA: std::cell::RefCell<LocalSettings> = std::cell::RefCell::new(LocalSettings::default());
}

#[derive(Debug, Default)]
pub struct LocalSettings {
    pub os: Option<String>,
    pub local_config_dir: Option<PathBuf>,
    pub local_temp_dir: Option<PathBuf>,
    pub local_config_file: Option<PathBuf>,
    pub local_temp_file: Option<PathBuf>,
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


pub fn detect_os_and_user_dir() {
    println!("{}", &t!("log.detecting-local-os").to_string());

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

    let app_name = APP_NAME.unwrap();

    let local_temp_dir = env::temp_dir();
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
        "android" | "ios" | "macos" | "freebsd" | "dragonfly" | "openbsd" | "netbsd" | "solaris" | "redox" => {
            // /home/<Username>/<AppName>/
            let mut path = PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/".to_string()));
            path.push(app_name);
            path
        },
        _ => PathBuf::from("/"),
    };

    let local_config_file = local_config_dir.join(APP_LOCAL_CONFIG_FILE);

    let (config_dir, config_file) = if local_config_dir.exists() && local_config_dir.is_dir() {
        (local_config_dir, local_config_file)
    } else {
        (local_temp_dir.clone(), local_temp_file.clone())
    };

    LOCAL_DATA.with(|data| {
        let mut data = data.borrow_mut();
        println!("\t OS: {:?}", &os);
        println!("\t Config directory: {:?}", &config_dir);
        println!("\t Temp directory: {:?}", &local_temp_dir);
        println!("\t Configuration file: {:?}", &config_file);
        println!("\t Temp file: {:?}", &local_temp_file);
        
        data.os = Some(os.clone());
        data.local_config_dir = Some(config_dir.clone());
        data.local_temp_dir = Some(local_temp_dir.clone());
        data.local_config_file = Some(config_file.clone());
        data.local_temp_file = Some(local_temp_file.clone());
    });


}

pub fn switch_locale(lang: &str) {
    match lang {
        "Deutsch" => rust_i18n::set_locale("de"),
        "Hrvatski" => rust_i18n::set_locale("hr"),
        "English"| _ => rust_i18n::set_locale("en"),
    }
}



pub fn create_local_files() -> Result<(), Box<dyn std::error::Error>> {
    let local_config_dir = LOCAL_DATA.with(|data| {
        let data = data.borrow();
        data.local_config_dir.clone().unwrap()
    });

    let local_config_file = LOCAL_DATA.with(|data| {
        let data = data.borrow();
        data.local_config_file.clone().unwrap()
    });

    if !local_config_dir.exists() {
        eprintln!("Local config directory not found!");
        fs::create_dir_all(&local_config_dir)?;
    }

    if !is_directory_writable(&local_config_dir)? {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Directory not writable").into());
    }

    if !Path::new(&local_config_file).exists() {
        eprintln!("Default config file '{:?}' does not exist", local_config_file);
        match fs::copy(APP_DEFAULT_CONFIG_FILE, &local_config_file) {
            Ok(_) => {
                println!("Local config file created");
                return Ok(())
            }
            Err(err) => {
                eprintln!("Failed to copy default config file: {}", err);
                return Err(err.into())
            }
        }
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