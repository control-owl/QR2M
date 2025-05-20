// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// license = "CC-BY-NC-ND-4.0  [2023-2025]  Control Owl"

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

use crate::{APP_NAME, FunctionOutput, d3bug};
use std::{
  env, fs,
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
  pub first_run: bool,
  // pub local_do_not_show_file: Option<PathBuf>,
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

pub fn detect_os_and_user_dir() -> FunctionOutput<()> {
  d3bug(">>> detect_os_and_user_dir", "debug");

  let os = match env::consts::OS {
    "windows" => "windows",
    "macos" => "macos",
    "linux" => "linux",
    _ => "unknown",
  };

  d3bug(&format!("OS: {:?}", os), "info");

  let app_name = APP_NAME.ok_or_else(|| crate::AppError::Custom("APP_NAME not set".to_string()))?;
  let local_temp = env::temp_dir();
  let local_temp_dir = local_temp.join(app_name);

  let local_temp_file = local_temp_dir.join(APP_LOCAL_TEMP_FILE);
  d3bug(&format!("Temp file: {:?}", local_temp_file), "info");

  let local_config_dir = match os {
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
  d3bug(&format!("Config file: {:?}", local_config_file), "info");

  let (config_dir, config_file) = if local_config_dir.is_symlink() {
    match fs::read_link(&local_config_dir) {
      Ok(target) => {
        if target.is_dir() {
          let metadata = fs::metadata(&target).map_err(crate::AppError::Io)?;
          if metadata.permissions().readonly() {
            return Err(crate::AppError::Custom(format!(
              "Symlink target is not writable: {:?}",
              target
            )));
          } else {
            d3bug(
              &format!("Using writable symlink target: {:?}", target),
              "debug",
            );

            (target.clone(), target.join(APP_LOCAL_CONFIG_FILE))
          }
        } else {
          return Err(crate::AppError::Custom(format!(
            "Symlink does not point to a directory: {:?}",
            target
          )));
        }
      }
      Err(e) => {
        return Err(crate::AppError::Io(e));
      }
    }
  } else {
    (local_config_dir, local_config_file)
  };

  let mut local_settings = LOCAL_SETTINGS
    .lock()
    .map_err(|e| crate::AppError::Custom(format!("Failed to lock LOCAL_SETTINGS: {}", e)))?;

  local_settings.os = Some(os.to_string());
  local_settings.local_config_dir = Some(config_dir.clone());
  local_settings.local_temp_dir = Some(local_temp_dir.clone());
  local_settings.local_config_file = Some(config_file.clone());
  local_settings.local_temp_file = Some(local_temp_file.clone());

  Ok(())
}

pub fn switch_locale(lang: &str) -> FunctionOutput<()> {
  d3bug(">>> switch_locale", "debug");

  match lang {
    "Deutsch" => rust_i18n::set_locale("de"),
    "Hrvatski" => rust_i18n::set_locale("hr"),
    _ => rust_i18n::set_locale("en"),
  }

  #[cfg(debug_assertions)]
  println!(" - Language: {:?}", lang);

  Ok(())
}

pub fn check_local_config() -> FunctionOutput<()> {
  d3bug(">>> check_local_config", "debug");

  let mut local_settings = LOCAL_SETTINGS
    .lock()
    .map_err(|e| crate::AppError::Custom(format!("Failed to lock LOCAL_SETTINGS: {}", e)))?;

  let local_config_file = local_settings
    .local_config_file
    .clone()
    .ok_or_else(|| crate::AppError::Custom("local_config_file not set".to_string()))?;

  let local_config_dir = local_settings
    .local_config_dir
    .clone()
    .ok_or_else(|| crate::AppError::Custom("local_config_dir not set".to_string()))?;

  if !local_config_dir.exists() {
    d3bug("Local config directory does not exists", "warning");

    fs::create_dir_all(&local_config_dir).map_err(crate::AppError::Io)?;
  } else {
    d3bug("Local config directory found", "debug");
  }

  if !is_directory_writable(&local_config_dir)? {
    return Err(crate::AppError::Custom(
      "Directory is not writable".to_string(),
    ));
  } else {
    d3bug("<<< is_directory_writable", "debug");
  }

  if !Path::new(&local_config_file).exists() {
    local_settings.first_run = true;
    let default_settings = crate::AppSettings::default();
    let serialized = toml::to_string(&default_settings)
      .map_err(|e| crate::AppError::Custom(format!("Failed to serialize settings: {}", e)))?;

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

    fs::write(&local_config_file, toml_string).map_err(crate::AppError::Io)?;

    #[cfg(debug_assertions)]
    println!("\t- New config file created");
  } else {
    local_settings.first_run = false;
  }

  Ok(())
}

fn is_directory_writable(dir: &Path) -> FunctionOutput<bool> {
  d3bug(">>> is_directory_writable", "debug");
  d3bug(&format!("Directory: {:?}", dir), "debug");

  let mut temp_file_path = dir.to_path_buf();
  temp_file_path.push(".tmp");

  match fs::File::create(&temp_file_path) {
    Ok(_) => {
      if let Err(err) = fs::remove_file(&temp_file_path) {
        // eprintln!("Failed to delete temporary file: {}", err);
        return Err(crate::AppError::Io(err));
      }
      d3bug(&format!("Directory is writable: {:?}", dir), "info");
      Ok(true)
    }
    Err(e) => Err(crate::AppError::Io(e)),
  }
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
