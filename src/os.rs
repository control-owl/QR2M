// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// license = "CC-BY-NC-ND-4.0  [2023-2025]  Control Owl"

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

use crate::APP_NAME;
use std::{
  env, fs,
  io::{self},
  path::{Path, PathBuf},
};

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
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.detecting_local_os").to_string());

  let os = match env::consts::OS {
    "windows" => "windows",
    "macos" => "macos",
    "linux" => "linux",
    _ => "unknown",
  }
  .to_string();

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
    }
    "linux" => {
      // /home/<Username>/.config/<AppName>/
      let mut path = PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/".to_string()));
      path.push(".config");
      path.push(app_name);
      path
    }
    "macos" => {
      // /home/<Username>/<AppName>/
      let mut path = PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/".to_string()));
      path.push(app_name);
      path
    }
    _ => PathBuf::from("/"),
  };

  let local_config_file = local_config_dir.join(APP_LOCAL_CONFIG_FILE);

  let (config_dir, config_file) = if local_config_dir.is_symlink() {
    match fs::read_link(&local_config_dir) {
      Ok(target) => {
        if target.is_dir() {
          if fs::metadata(&target)
            .map(|m| m.permissions().readonly())
            .unwrap_or(true)
          {
            eprintln!("\t- Symlink target is not writable: {:?}", &target);
            (
              local_temp_dir.clone(),
              local_temp_dir.join(APP_LOCAL_CONFIG_FILE),
            )
          } else {
            #[cfg(debug_assertions)]
            println!("\t- Using writable symlink target: {:?}", &target);
            (target.clone(), target.join(APP_LOCAL_CONFIG_FILE))
          }
        } else {
          eprintln!("\t- Symlink does not point to a directory: {:?}", &target);
          (
            local_temp_dir.clone(),
            local_temp_dir.join(APP_LOCAL_CONFIG_FILE),
          )
        }
      }
      Err(e) => {
        eprintln!(
          "\t- Failed to read symlink target: {:?}\n\tError: {}",
          &local_config_dir, e
        );
        (
          local_temp_dir.clone(),
          local_temp_dir.join(APP_LOCAL_CONFIG_FILE),
        )
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

  #[cfg(debug_assertions)]
  {
    println!("\t- OS: {:?}", &os);
    println!("\t- Config directory: {:?}", &config_dir);
    println!("\t- Configuration file: {:?}", &config_file);
    println!("\t- Temp directory: {:?}", &local_temp_dir);
    println!("\t- Temp file: {:?}", &local_temp_file);
  }
}

pub fn switch_locale(lang: &str) {
  #[cfg(debug_assertions)]
  println!(" - {}", &t!("log.switch_locale").to_string());

  match lang {
    "Deutsch" => rust_i18n::set_locale("de"),
    "Hrvatski" => rust_i18n::set_locale("hr"),
    _ => rust_i18n::set_locale("en"),
  }

  #[cfg(debug_assertions)]
  println!(" - Language: {:?}", lang);
}

pub fn check_local_config() -> Result<(), Box<dyn std::error::Error>> {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.check_local_config").to_string());

  let local_settings = LOCAL_SETTINGS.lock().unwrap();
  let local_config_file = local_settings.local_config_file.clone().unwrap();
  let local_config_dir = local_settings.local_config_dir.clone().unwrap();

  if !local_config_dir.exists() {
    fs::create_dir_all(&local_config_dir)?;
  }

  if !is_directory_writable(&local_config_dir)? {
    return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Directory not writable").into());
  }

  if !Path::new(&local_config_file).exists() {
    let default_settings = crate::AppSettings::default();
    let serialized = toml::to_string(&default_settings)?;
    let mut config_map: std::collections::BTreeMap<
      String,
      std::collections::BTreeMap<String, String>,
    > = std::collections::BTreeMap::new();
    let mut toml_string = String::new();

    for line in serialized.lines() {
      if let Some((key, value)) = line.split_once(" = ") {
        let (section, key) = key.split_once('_').unwrap_or(("general", key));
        config_map
          .entry(section.to_string())
          .or_default()
          .insert(key.to_string(), value.to_string());
      }
    }

    for (section, entries) in config_map {
      toml_string.push_str(&format!("[{}]\n", section));
      for (key, value) in entries {
        toml_string.push_str(&format!("{} = {}\n", key, value));
      }
      toml_string.push('\n');
    }

    fs::write(&local_config_file, toml_string)?;

    #[cfg(debug_assertions)]
    println!("\t- New config file created");
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
