// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// license = "CC-BY-NC-ND-4.0  [2023-2025]  Control Owl"

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

// #![windows_subsystem = "windows"]
// #![allow(non_snake_case)]
// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(unused_assignments)]
// #![allow(dead_code)]
// #![allow(unused_mut)]

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
use adw::prelude::*;
use glib::{self, SourceId};
use gtk::{Stack, StackSidebar, gio, glib::clone};
use gtk4::{self as gtk};
use libadwaita as adw;
use rand::Rng;
use std::{
  cell::RefCell,
  fs::{self, File},
  io::{self, BufRead},
  rc::Rc,
  time::SystemTime,
};

#[cfg(feature = "full")]
mod anu;
mod coin_db;
#[cfg(feature = "dev")]
mod dev;
mod keys;
mod os;
#[cfg(feature = "dev")]
mod sec;
mod test_vectors;

#[macro_use]
extern crate rust_i18n;
i18n!("res/locale", fallback = "en");

const APP_NAME: Option<&str> = option_env!("CARGO_PKG_NAME");
const APP_DESCRIPTION: Option<&str> = option_env!("CARGO_PKG_DESCRIPTION");
const APP_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
const APP_AUTHOR: Option<&str> = option_env!("CARGO_PKG_AUTHORS");
const APP_LANGUAGE: &[&str] = &["English", "Deutsch", "Hrvatski"];
const VALID_MNEMONIC_DICTIONARY: &[&str] = &[
  "English",
  "Czech",
  "French",
  "Italian",
  "Portuguese",
  "Spanish",
  "Chinese simplified",
  "Chinese traditional",
  "Japanese",
  "Korean",
];
const VALID_ENTROPY_LENGTHS: [u32; 5] = [128, 160, 192, 224, 256];
const VALID_BIP_DERIVATIONS: &[&str] = &[
  "32",
  "44",
  // #[cfg(feature = "dev")]
  // "49",
  // #[cfg(feature = "dev")]
  // "84",
  // #[cfg(feature = "dev")]
  // "86",
  #[cfg(feature = "dev")]
  "Custom",
];
const VALID_ENTROPY_SOURCES: &[&str] = &["RNG+", "File", "QRNG"];
// #[cfg(feature = "dev")]
// const VALID_IMPORT_SOURCES: &[&str] = &["Entropy", "Mnemonic", "Seed"];
const VALID_WALLET_PURPOSE: &[&str] = &["Internal", "External"];
const VALID_ANU_API_DATA_FORMAT: &[&str] = &[
  "uint8",
  #[cfg(feature = "dev")]
  "uint16",
  #[cfg(feature = "dev")]
  "hex16",
];
const WALLET_DEFAULT_EXTENSION: &str = "qr2m";
const WALLET_CURRENT_VERSION: u32 = 1;
const WALLET_MAX_ADDRESSES: u32 = 2147483647;
const ANU_MAXIMUM_ARRAY_LENGTH: u32 = 1024;
const ANU_MAXIMUM_CONNECTION_TIMEOUT: u32 = 60;
const WINDOW_SETTINGS_DEFAULT_WIDTH: u32 = 700;
const WINDOW_SETTINGS_DEFAULT_HEIGHT: u32 = 500;
const VALID_GUI_THEMES: &[&str] = &["System", "Light", "Dark"];
const VALID_GUI_ICONS: &[&str] = &["Thin", "Bold", "Fill"];
const VALID_COIN_SEARCH_PARAMETER: &[&str] = &["Name", "Symbol", "Index"];
const APP_LOG_LEVEL: &[&str] = &["Standard", "Verbose", "Ultimate"];
const GUI_IMAGE_EXTENSION: &str = if cfg!(windows) { "png" } else { "svg" };

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

type FunctionOutput<T> = Result<T, AppError>;

#[derive(Debug)]
enum AppError {
  Io(io::Error),
  Custom(String),
}

impl std::fmt::Display for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      AppError::Io(e) => write!(f, "IO error: {}", e),
      AppError::Custom(msg) => write!(f, "{}", msg),
    }
  }
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

lazy_static::lazy_static! {
    static ref APP_SETTINGS: std::sync::Arc<std::sync::RwLock<AppSettings>> = std::sync::Arc::new(std::sync::RwLock::new(AppSettings::default()));
    static ref APP_LOG: std::sync::Arc<std::sync::Mutex<AppLog>> = std::sync::Arc::new(std::sync::Mutex::new(AppLog::new()));
    static ref WALLET_SETTINGS: std::sync::Arc<std::sync::Mutex<WalletSettings>> = std::sync::Arc::new(std::sync::Mutex::new(WalletSettings::new()));
    static ref CRYPTO_ADDRESS: std::sync::Arc<dashmap::DashMap<String, CryptoAddresses>> = std::sync::Arc::new(dashmap::DashMap::new());
    static ref DERIVATION_PATH: std::sync::Arc<std::sync::RwLock<DerivationPath>> = std::sync::Arc::new(std::sync::RwLock::new(DerivationPath::default()));
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

struct GuiState {
  gui_language: Option<String>,
  gui_theme: Option<String>,
  gui_icon_theme: Option<String>,
  gui_log_status: Option<bool>,
  gui_main_buttons: Rc<RefCell<std::collections::HashMap<String, Vec<Rc<gtk::Button>>>>>,
  gui_button_images: Option<std::collections::HashMap<String, gtk::gdk::Texture>>,
}

impl GuiState {
  fn default_config() -> Self {
    Self {
      gui_language: APP_LANGUAGE.first().map(|s| s.to_string()),
      gui_theme: VALID_GUI_THEMES.first().map(|s| s.to_string()),
      gui_icon_theme: VALID_GUI_ICONS.first().map(|s| s.to_string()),
      gui_log_status: None,
      gui_main_buttons: Rc::new(RefCell::new(std::collections::HashMap::new())),
      gui_button_images: None,
    }
  }

  fn apply_language(&mut self) {
    #[cfg(debug_assertions)]
    println!(
      "[+] {}",
      &t!("log.process_wallet_file_from_path").to_string()
    );

    if let Some(language) = &self.gui_language {
      let language_code = match language.as_str() {
        "Deutsch" => "de",
        "Hrvatski" => "hr",
        _ => "en",
      };

      rust_i18n::set_locale(language_code);
    } else {
      rust_i18n::set_locale("en");
    }
  }

  fn reload_gui_icons(&mut self) {
    #[cfg(debug_assertions)]
    println!("[+] {}", &t!("log.reload_gui_icons").to_string());

    let settings = gtk::Settings::default().unwrap();
    let theme_subdir = if settings.is_gtk_application_prefer_dark_theme() {
      "dark"
    } else {
      "light"
    };

    let gui_icons = self
      .gui_icon_theme
      .clone()
      .unwrap_or(VALID_GUI_ICONS[0].to_string());

    let theme_base_path = std::path::Path::new("theme")
      .join("basic")
      .join(theme_subdir)
      .join(gui_icons);

    let extension = GUI_IMAGE_EXTENSION;

    let icon_files = [
      ("new", format!("new.{}", extension)),
      ("open", format!("open.{}", extension)),
      ("save", format!("save.{}", extension)),
      ("about", format!("about.{}", extension)),
      ("settings", format!("settings.{}", extension)),
      ("log", format!("log.{}", extension)),
      ("notif", format!("notif.{}", extension)),
      ("random", format!("random.{}", extension)),
    ];

    let mut icons = std::collections::HashMap::new();
    for (name, file) in icon_files.iter() {
      let icon_path = theme_base_path.join(file);

      #[cfg(debug_assertions)]
      println!("\t- Icon: {:?}", icon_path);

      if let Some(icon_str) = icon_path.to_str() {
        let texture = qr2m_lib::get_texture_from_resource(icon_str);
        icons.insert(name.to_string(), texture);
      } else {
        #[cfg(debug_assertions)]
        eprintln!("Warning: Invalid UTF-8 in path {:?}", icon_path);
      }
    }

    #[cfg(feature = "dev")]
    {
      let security_icon_path = std::path::Path::new("theme").join("color");

      let security = sec::SECURITY_STATUS.read().unwrap();

      let security_texture = if security.app_key && security.author_key && !security.code_modified {
        qr2m_lib::get_texture_from_resource(
          security_icon_path
            .join(format!("sec-good.{}", GUI_IMAGE_EXTENSION))
            .to_str()
            .unwrap_or(&format!("theme/color/sec-good.{}", GUI_IMAGE_EXTENSION)),
        )
      } else if security.app_key && security.author_key {
        qr2m_lib::get_texture_from_resource(
          security_icon_path
            .join(format!("sec-warn.{}", GUI_IMAGE_EXTENSION))
            .to_str()
            .unwrap_or(&format!("theme/color/sec-warn.{}", GUI_IMAGE_EXTENSION)),
        )
      } else {
        qr2m_lib::get_texture_from_resource(
          security_icon_path
            .join(format!("sec-error.{}", GUI_IMAGE_EXTENSION))
            .to_str()
            .unwrap_or(&format!("theme/color/sec-error.{}", GUI_IMAGE_EXTENSION)),
        )
      };

      icons.insert("security".to_string(), security_texture);
    }

    self.gui_button_images = Some(icons);

    let button_map = self.gui_main_buttons.borrow();
    if let Some(texture_map) = &self.gui_button_images {
      for (name, buttons) in button_map.iter() {
        if let Some(texture) = texture_map.get(name.as_str()) {
          for button in buttons.iter() {
            let picture = gtk::Picture::new();
            picture.set_paintable(Some(texture));
            button.set_child(Some(&picture));
          }
        }
      }
    }
  }

  fn reload_gui_theme(&mut self) {
    #[cfg(debug_assertions)]
    println!("[+] {}", &t!("log.reload_gui_theme").to_string());

    if let Some(theme) = &self.gui_theme {
      let preferred_theme = match theme.as_str() {
        "Light" => adw::ColorScheme::ForceLight,
        "Dark" => adw::ColorScheme::ForceDark,
        _ => adw::ColorScheme::PreferLight,
      };
      adw::StyleManager::default().set_color_scheme(preferred_theme);

      #[cfg(debug_assertions)]
      println!("\t- GUI theme: {:?}", preferred_theme);
    } else {
      adw::StyleManager::default().set_color_scheme(adw::ColorScheme::PreferLight);

      #[cfg(debug_assertions)]
      eprintln!("\t- Problem with GUI theme, revert to default theme");
    }
  }

  fn register_button(&self, name: String, button: Rc<gtk::Button>) {
    #[cfg(debug_assertions)]
    println!("[+] {}", &t!("log.register_button").to_string());

    let mut button_map = self.gui_main_buttons.borrow_mut();
    button_map.entry(name.to_string()).or_default().push(button);

    #[cfg(debug_assertions)]
    println!("\t- Button: {:?}", name)
  }
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

#[derive(serde::Serialize, Clone)]
struct AppSettings {
  wallet_entropy_source: Option<String>,
  wallet_entropy_length: Option<u32>,
  wallet_mnemonic_length: Option<u32>,
  wallet_bip: Option<u32>,
  wallet_address_count: Option<u32>,
  wallet_hardened_address: Option<bool>,
  wallet_mnemonic_dictionary: Option<String>,
  gui_save_size: Option<bool>,
  gui_last_width: Option<u32>,
  gui_last_height: Option<u32>,
  gui_maximized: Option<bool>,
  gui_theme: Option<String>,
  gui_icons: Option<String>,
  gui_language: Option<String>,
  gui_search: Option<String>,
  gui_notifications: Option<bool>,
  gui_notification_timeout: Option<u32>,
  gui_log: Option<bool>,
  gui_log_level: Option<String>,
  anu_enabled: Option<bool>,
  anu_data_format: Option<String>,
  anu_array_length: Option<u32>,
  anu_hex_block_size: Option<u32>,
  anu_log: Option<bool>,
  anu_timeout: Option<u32>,
  proxy_status: Option<bool>,
  proxy_server_address: Option<String>,
  proxy_server_port: Option<u32>,
  proxy_use_pac: Option<bool>,
  proxy_script_address: Option<String>,
  proxy_login_credentials: Option<bool>,
  proxy_login_username: Option<String>,
  proxy_login_password: Option<String>,
  proxy_use_ssl: Option<bool>,
  proxy_ssl_certificate: Option<String>,
  proxy_retry_attempts: Option<u32>,
  proxy_timeout: Option<u32>,
}

impl Default for AppSettings {
  fn default() -> Self {
    AppSettings {
      wallet_entropy_source: Some("RNG+".to_string()),
      wallet_entropy_length: Some(256),
      wallet_mnemonic_length: Some(512),
      wallet_bip: Some(44),
      wallet_address_count: Some(10),
      wallet_hardened_address: Some(true),
      wallet_mnemonic_dictionary: Some("English".to_string()),
      gui_save_size: Some(true),
      gui_last_width: Some(1024),
      gui_last_height: Some(768),
      gui_maximized: Some(false),
      gui_theme: Some("System".to_string()),
      gui_icons: Some("Thin".to_string()),
      gui_language: Some("English".to_string()),
      gui_search: Some("Name".to_string()),
      gui_notifications: Some(true),
      gui_notification_timeout: Some(3),
      gui_log: Some(true),
      gui_log_level: Some("Standard".to_string()),
      anu_enabled: Some(false),
      anu_data_format: Some("uint8".to_string()),
      anu_array_length: Some(128),
      anu_hex_block_size: Some(128),
      anu_log: Some(true),
      anu_timeout: Some(5),
      proxy_status: Some(false),
      proxy_server_address: Some("".to_string()),
      proxy_server_port: Some(8080),
      proxy_use_pac: Some(false),
      proxy_script_address: Some("".to_string()),
      proxy_login_credentials: Some(false),
      proxy_login_username: Some("".to_string()),
      proxy_login_password: Some("".to_string()),
      proxy_use_ssl: Some(false),
      proxy_ssl_certificate: Some("".to_string()),
      proxy_retry_attempts: Some(3),
      proxy_timeout: Some(5000),
    }
  }
}

impl AppSettings {
  fn load_settings() -> FunctionOutput<()> {
    d3bug(">>> load_settings", "debug");

    let settings = AppSettings::default();

    let local_settings = os::LOCAL_SETTINGS.lock().unwrap();
    let local_config_file = local_settings.local_config_file.clone().unwrap();

    #[cfg(debug_assertions)]
    println!("\t- Settings file: {:?}", local_config_file);

    let config_str = match fs::read_to_string(&local_config_file) {
      Ok(contents) => contents,
      Err(err) => {
        if err.kind() == io::ErrorKind::NotFound {
          #[cfg(debug_assertions)]
          println!("\t- Config file not found, using default settings.");

          match os::check_local_config() {
            Ok(_) => {
              d3bug("<<< check_local_config", "debug");
            }
            Err(err) => d3bug(&format!("check_local_config: {:?}", err), "error"),
          };
        } else {
          #[cfg(debug_assertions)]
          eprintln!(
            "\t- Failed to read local config file: {:?} \n Error: {:?}",
            local_config_file, err
          );
        }
        String::new()
      }
    };

    let config: toml::Value = config_str.parse().unwrap_or_else(|_err| {
      #[cfg(debug_assertions)]
      println!("\t- {}", &t!("error.settings.config", error = _err));

      toml::Value::Table(toml::value::Table::new())
    });

    fn get_str(section: &toml::Value, key: &str, default: Option<String>) -> Option<String> {
      section
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or(default)
    }

    fn get_u32(section: &toml::Value, key: &str, default: Option<u32>) -> Option<u32> {
      section
        .get(key)
        .and_then(|v| v.as_integer())
        .map(|v| v as u32)
        .or(default)
    }

    fn get_bool(section: &toml::Value, key: &str, default: Option<bool>) -> Option<bool> {
      section.get(key).and_then(|v| v.as_bool()).or(default)
    }

    fn load_section(config: &toml::Value, section_name: &str) -> toml::Value {
      config
        .get(section_name)
        .cloned()
        .unwrap_or_else(|| toml::Value::Table(toml::value::Table::new()))
    }

    let gui_section = load_section(&config, "gui");
    let wallet_section = load_section(&config, "wallet");
    let anu_section = load_section(&config, "anu");
    let proxy_section = load_section(&config, "proxy");

    let gui_save_size = get_bool(&gui_section, "save_size", settings.gui_save_size);
    let gui_last_width = get_u32(&gui_section, "last_width", settings.gui_last_width);
    let gui_last_height = get_u32(&gui_section, "last_height", settings.gui_last_height);
    let gui_maximized = get_bool(&gui_section, "maximized", settings.gui_maximized);
    let gui_theme = get_str(&gui_section, "theme", settings.gui_theme);
    let gui_icons = get_str(&gui_section, "icons", settings.gui_icons);
    let gui_language = get_str(&gui_section, "language", settings.gui_language);
    let gui_search = get_str(&gui_section, "search", settings.gui_search);
    let gui_notifications = get_bool(
      &gui_section,
      "notification_enabled",
      settings.gui_notifications,
    );
    let gui_notification_timeout = get_u32(
      &gui_section,
      "notification_timeout",
      settings.gui_notification_timeout,
    );
    let gui_log = get_bool(&gui_section, "gui_log", settings.gui_log);
    let gui_log_level = get_str(&gui_section, "log_level", settings.gui_log_level);

    #[cfg(debug_assertions)]
    {
      println!("\t- Save last window size: {:?}", gui_save_size);
      println!("\t- GUI width: {:?}", gui_last_width);
      println!("\t- GUI height: {:?}", gui_last_height);
      println!("\t- Maximized: {:?}", gui_maximized);
      println!("\t- Theme: {:?}", gui_theme);
      println!("\t- Icons: {:?}", gui_icons);
      println!("\t- Language: {:?}", gui_language);
      println!("\t- Search: {:?}", gui_search);
      println!("\t- Notification enabled: {:?}", gui_notifications);
      println!("\t- Notification timeout: {:?}", gui_notification_timeout);
      println!("\t- Log enabled: {:?}", gui_log);
      println!("\t- Log level: {:?}", gui_log_level);
    }

    let wallet_entropy_source = get_str(
      &wallet_section,
      "entropy_source",
      settings.wallet_entropy_source,
    );
    let wallet_entropy_length = get_u32(
      &wallet_section,
      "entropy_length",
      settings.wallet_entropy_length,
    );
    let wallet_mnemonic_length = get_u32(
      &gui_section,
      "mnemonic_length",
      settings.wallet_mnemonic_length,
    );
    let wallet_bip = get_u32(&wallet_section, "bip", settings.wallet_bip);
    let wallet_address_count = get_u32(
      &wallet_section,
      "address_count",
      settings.wallet_address_count,
    );
    let wallet_hardened_address = get_bool(
      &wallet_section,
      "hardened_address",
      settings.wallet_hardened_address,
    );
    let wallet_mnemonic_dictionary = get_str(
      &wallet_section,
      "mnemonic_dictionary",
      settings.wallet_mnemonic_dictionary,
    );
    #[cfg(debug_assertions)]
    {
      println!("\t- Entropy source: {:?}", wallet_entropy_source);
      println!("\t- Entropy length: {:?}", wallet_entropy_length);
      println!(
        "\t- Mnemonic passphrase length: {:?}",
        wallet_mnemonic_length
      );
      println!("\t- BIP: {:?}", wallet_bip);
      println!("\t- Address count: {:?}", wallet_address_count);
      println!("\t- Hard address: {:?}", wallet_hardened_address);
      println!("\t- Mnemonic dictionary: {:?}", wallet_mnemonic_dictionary);
    }

    let anu_enabled = get_bool(&anu_section, "enabled", settings.anu_enabled);
    let anu_data_format = get_str(&anu_section, "data_format", settings.anu_data_format);
    let anu_array_length = get_u32(&anu_section, "array_length", settings.anu_array_length);
    let anu_hex_block_size = get_u32(&anu_section, "hex_block_size", settings.anu_hex_block_size);
    let anu_log = get_bool(&anu_section, "log", settings.anu_log);
    let anu_timeout = get_u32(&anu_section, "timeout", settings.anu_timeout);

    #[cfg(debug_assertions)]
    {
      println!("\t- Use ANU: {:?}", anu_enabled);
      println!("\t- ANU data format: {:?}", anu_data_format);
      println!("\t- ANU array length: {:?}", anu_array_length);
      println!("\t- ANU hex block size: {:?}", anu_hex_block_size);
      println!("\t- ANU log: {:?}", anu_log);
      println!("\t- ANU timeout: {:?}", anu_timeout);
    }

    let proxy_status = get_bool(&proxy_section, "status", settings.proxy_status);
    let proxy_server_address = get_str(
      &proxy_section,
      "server_address",
      settings.proxy_server_address,
    );
    let proxy_server_port = get_u32(&proxy_section, "server_port", settings.proxy_server_port);
    let proxy_use_pac = get_bool(&proxy_section, "use_pac", settings.proxy_use_pac);
    let proxy_script_address = get_str(
      &proxy_section,
      "script_address",
      settings.proxy_script_address,
    );
    let proxy_login_credentials = get_bool(
      &proxy_section,
      "login_credentials",
      settings.proxy_login_credentials,
    );
    let proxy_login_username = get_str(
      &proxy_section,
      "login_username",
      settings.proxy_login_username,
    );
    let proxy_login_password = get_str(
      &proxy_section,
      "login_password",
      settings.proxy_login_password,
    );
    let proxy_use_ssl = get_bool(&proxy_section, "use_ssl", settings.proxy_use_ssl);
    let proxy_ssl_certificate = get_str(
      &proxy_section,
      "ssl_certificate",
      settings.proxy_ssl_certificate,
    );
    let proxy_retry_attempts = get_u32(
      &proxy_section,
      "retry_attempts",
      settings.proxy_retry_attempts,
    );
    let proxy_timeout = get_u32(&proxy_section, "timeout", settings.proxy_timeout);

    #[cfg(debug_assertions)]
    {
      println!("\t- Use proxy: {:?}", proxy_status);
      println!("\t- Proxy server address: {:?}", proxy_server_address);
      println!("\t- Proxy server port: {:?}", proxy_server_port);
      println!("\t- Use proxy PAC: {:?}", proxy_use_pac);
      println!("\t- Proxy script address: {:?}", proxy_script_address);
      println!(
        "\t- Use proxy login credentials: {:?}",
        proxy_login_credentials
      );
      println!("\t- Proxy username: {:?}", proxy_login_username);
      println!("\t- Proxy password: {:?}", proxy_login_password);
      println!("\t- Use proxy SSL: {:?}", proxy_use_ssl);
      println!("\t- Proxy SSL certificate: {:?}", proxy_ssl_certificate);
      println!("\t- Proxy retry attempts: {:?}", proxy_retry_attempts);
      println!("\t- Proxy timeout: {:?}", proxy_timeout);
    }

    let mut application_settings = APP_SETTINGS.write().unwrap();
    application_settings.wallet_entropy_source = wallet_entropy_source.clone();
    application_settings.wallet_entropy_length = wallet_entropy_length;
    application_settings.wallet_mnemonic_length = wallet_mnemonic_length;
    application_settings.wallet_bip = wallet_bip;
    application_settings.wallet_address_count = wallet_address_count;
    application_settings.wallet_hardened_address = wallet_hardened_address;
    application_settings.wallet_mnemonic_dictionary = wallet_mnemonic_dictionary;

    application_settings.gui_save_size = gui_save_size;
    application_settings.gui_last_width = gui_last_width;
    application_settings.gui_last_height = gui_last_height;
    application_settings.gui_maximized = gui_maximized;
    application_settings.gui_theme = gui_theme.clone();
    application_settings.gui_icons = gui_icons.clone();
    application_settings.gui_language = gui_language.clone();
    application_settings.gui_search = gui_search.clone();
    application_settings.gui_notifications = gui_notifications;
    application_settings.gui_notification_timeout = gui_notification_timeout;
    application_settings.gui_log = gui_log;
    application_settings.gui_log_level = gui_log_level.clone();

    application_settings.anu_enabled = anu_enabled;
    application_settings.anu_data_format = anu_data_format.clone();
    application_settings.anu_array_length = anu_array_length;
    application_settings.anu_hex_block_size = anu_hex_block_size;
    application_settings.anu_log = anu_log;
    application_settings.anu_timeout = anu_timeout;

    application_settings.proxy_status = proxy_status;
    application_settings.proxy_server_address = proxy_server_address.clone();
    application_settings.proxy_server_port = proxy_server_port;
    application_settings.proxy_use_pac = proxy_use_pac;
    application_settings.proxy_script_address = proxy_script_address.clone();
    application_settings.proxy_login_credentials = proxy_login_credentials;
    application_settings.proxy_login_username = proxy_login_username.clone();
    application_settings.proxy_login_password = proxy_login_password.clone();
    application_settings.proxy_use_ssl = proxy_use_ssl;
    application_settings.proxy_ssl_certificate = proxy_ssl_certificate.clone();
    application_settings.proxy_retry_attempts = proxy_retry_attempts;
    application_settings.proxy_timeout = proxy_timeout;

    Ok(())
  }

  fn update_value(
    &mut self,
    key: &str,
    new_value: toml_edit::Item,
    gui_state: Option<Rc<RefCell<GuiState>>>,
  ) {
    // println!("[+] {}", &t!("log.app_settings.update_value").to_string());

    match key {
      "wallet_entropy_source" => {
        if let Some(value) = new_value.as_str() {
          if Some(value.to_string()) != self.wallet_entropy_source {
            self.wallet_entropy_source = Some(value.to_string());

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "wallet_entropy_length" => {
        if let Some(value) = new_value.as_integer() {
          let value = value as u32;
          if Some(value) != self.wallet_entropy_length {
            self.wallet_entropy_length = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "wallet_mnemonic_length" => {
        if let Some(value) = new_value.as_integer() {
          let value = value as u32;
          if Some(value) != self.wallet_mnemonic_length {
            self.wallet_mnemonic_length = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "wallet_bip" => {
        if let Some(value) = new_value.as_integer() {
          let value = value as u32;
          if Some(value) != self.wallet_bip {
            self.wallet_bip = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "wallet_address_count" => {
        if let Some(value) = new_value.as_integer() {
          let value = value as u32;
          if Some(value) != self.wallet_address_count {
            self.wallet_address_count = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "wallet_hardened_address" => {
        if let Some(value) = new_value.as_bool() {
          if Some(value) != self.wallet_hardened_address {
            self.wallet_hardened_address = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "wallet_mnemonic_dictionary" => {
        if let Some(value) = new_value.as_str() {
          if Some(value.to_string()) != self.wallet_mnemonic_dictionary {
            self.wallet_mnemonic_dictionary = Some(value.to_string());

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "gui_save_size" => {
        if let Some(value) = new_value.as_bool() {
          if Some(value) != self.gui_save_size {
            self.gui_save_size = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "gui_last_width" => {
        if let Some(value) = new_value.as_integer() {
          let value = value as u32;
          if Some(value) != self.gui_last_width {
            self.gui_last_width = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "gui_last_height" => {
        if let Some(value) = new_value.as_integer() {
          let value = value as u32;
          if Some(value) != self.gui_last_height {
            self.gui_last_height = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "gui_maximized" => {
        if let Some(value) = new_value.as_bool() {
          if Some(value) != self.gui_maximized {
            self.gui_maximized = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "gui_theme" => {
        if let Some(new_theme) = new_value.as_str() {
          if Some(new_theme.to_string()) != self.gui_theme {
            self.gui_theme = Some(new_theme.to_string());

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);

            let preferred_theme = match new_theme {
              "Light" => adw::ColorScheme::ForceLight,
              "Dark" => adw::ColorScheme::ForceDark,
              _ => adw::ColorScheme::PreferLight,
            };

            adw::StyleManager::default().set_color_scheme(preferred_theme);

            if let Some(state) = gui_state {
              let mut state = state.borrow_mut();
              state.gui_theme = Some(new_theme.to_string());
            } else {
              #[cfg(debug_assertions)]
              println!("State in gui_theme is None");
            }
          }
        } else {
          #[cfg(debug_assertions)]
          eprintln!("Received invalid value for gui_theme: {:?}", new_value);
        }
      }
      "gui_icons" => {
        if let Some(new_icons) = new_value.as_str() {
          if Some(new_icons.to_string()) != self.gui_icons {
            self.gui_icons = Some(new_icons.to_string());

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);

            if let Some(state) = gui_state {
              let mut state = state.borrow_mut();
              state.gui_icon_theme = self.gui_icons.clone();
              state.reload_gui_icons();
            } else {
              #[cfg(debug_assertions)]
              println!("State in gui_icons is None");
            }
          }
        } else {
          #[cfg(debug_assertions)]
          eprintln!("Received invalid value for gui_icons: {:?}", new_value);
        }
      }
      "gui_language" => {
        if let Some(value) = new_value.as_str() {
          if Some(value.to_string()) != self.gui_language {
            self.gui_language = Some(value.to_string());

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "gui_search" => {
        if let Some(value) = new_value.as_str() {
          if Some(value.to_string()) != self.gui_search {
            self.gui_search = Some(value.to_string());

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "gui_notifications" => {
        if let Some(value) = new_value.as_bool() {
          if Some(value) != self.gui_notifications {
            self.gui_notifications = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "gui_notification_timeout" => {
        if let Some(value) = new_value.as_integer() {
          let value = value as u32;
          if Some(value) != self.gui_notification_timeout {
            self.gui_notification_timeout = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "gui_log" => {
        if let Some(value) = new_value.as_bool() {
          if Some(value) != self.gui_log {
            self.gui_log = Some(value);

            if let Some(state) = gui_state {
              let mut gui_state_lock = state.borrow_mut();
              gui_state_lock.gui_log_status = Some(value);
            } else {
              #[cfg(debug_assertions)]
              println!("State in gui_theme is None");
            }

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "gui_log_level" => {
        if let Some(value) = new_value.as_str() {
          if Some(value.to_string()) != self.gui_log_level {
            self.gui_log_level = Some(value.to_string());

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "anu_enabled" => {
        if let Some(value) = new_value.as_bool() {
          if Some(value) != self.anu_enabled {
            self.anu_enabled = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "anu_data_format" => {
        if let Some(value) = new_value.as_str() {
          if Some(value.to_string()) != self.anu_data_format {
            self.anu_data_format = Some(value.to_string());

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "anu_array_length" => {
        if let Some(value) = new_value.as_integer() {
          let value = value as u32;
          if Some(value) != self.anu_array_length {
            self.anu_array_length = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "anu_hex_block_size" => {
        if let Some(value) = new_value.as_integer() {
          let value = value as u32;
          if Some(value) != self.anu_hex_block_size {
            self.anu_hex_block_size = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "anu_log" => {
        if let Some(value) = new_value.as_bool() {
          if Some(value) != self.anu_log {
            self.anu_log = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "anu_timeout" => {
        if let Some(value) = new_value.as_integer() {
          let value = value as u32;
          if Some(value) != self.anu_timeout {
            self.anu_timeout = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "proxy_status" => {
        if let Some(value) = new_value.as_bool() {
          if Some(value) != self.proxy_status {
            self.proxy_status = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "proxy_server_address" => {
        if let Some(value) = new_value.as_str() {
          if Some(value.to_string()) != self.proxy_server_address {
            self.proxy_server_address = Some(value.to_string());

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "proxy_server_port" => {
        if let Some(value) = new_value.as_integer() {
          let value = value as u32;
          if Some(value) != self.proxy_server_port {
            self.proxy_server_port = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "proxy_use_pac" => {
        if let Some(value) = new_value.as_bool() {
          if Some(value) != self.proxy_use_pac {
            self.proxy_use_pac = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "proxy_script_address" => {
        if let Some(value) = new_value.as_str() {
          if Some(value.to_string()) != self.proxy_script_address {
            self.proxy_script_address = Some(value.to_string());

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "proxy_login_credentials" => {
        if let Some(value) = new_value.as_bool() {
          if Some(value) != self.proxy_login_credentials {
            self.proxy_login_credentials = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "proxy_login_username" => {
        if let Some(value) = new_value.as_str() {
          if Some(value.to_string()) != self.proxy_login_username {
            self.proxy_login_username = Some(value.to_string());

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "proxy_login_password" => {
        if let Some(value) = new_value.as_str() {
          if Some(value.to_string()) != self.proxy_login_password {
            self.proxy_login_password = Some(value.to_string());

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "proxy_use_ssl" => {
        if let Some(value) = new_value.as_bool() {
          if Some(value) != self.proxy_use_ssl {
            self.proxy_use_ssl = Some(value);

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      "proxy_ssl_certificate" => {
        if let Some(value) = new_value.as_str() {
          if Some(value.to_string()) != self.proxy_ssl_certificate {
            self.proxy_ssl_certificate = Some(value.to_string());

            #[cfg(debug_assertions)]
            println!("\t- Updating key  {:?} = {:?}", key, new_value);
          }
        }
      }
      _ => {}
    }
  }

  fn save_settings(&self) {
    #[cfg(debug_assertions)]
    println!("[+] {}", &t!("log.app_settings.save_settings").to_string());

    let local_config_file = {
      let local_settings = os::LOCAL_SETTINGS.lock().unwrap();
      local_settings.local_config_file.clone().unwrap()
    };

    let config_str = qr2m_lib::read_config_from_file(&local_config_file).unwrap_or_else(|_| {
      eprintln!("Failed to read config, using defaults.");
      String::new()
    });

    let mut doc = config_str
      .parse::<toml_edit::DocumentMut>()
      .map_err(|e| io::Error::other(format!("\t- Failed to parse config string: {}", e)))
      .unwrap_or(toml_edit::DocumentMut::new());

    {
      let wallet_section = doc["wallet"].or_insert(toml_edit::Item::Table(Default::default()));
      if let toml_edit::Item::Table(wallet_table) = wallet_section {
        wallet_table["entropy_source"] =
          toml_edit::value(self.wallet_entropy_source.clone().unwrap());
        wallet_table["entropy_length"] =
          toml_edit::value(self.wallet_entropy_length.unwrap() as i64);
        wallet_table["mnemonic_length"] =
          toml_edit::value(self.wallet_mnemonic_length.unwrap() as i64);
        wallet_table["bip"] = toml_edit::value(self.wallet_bip.unwrap() as i64);
        wallet_table["address_count"] = toml_edit::value(self.wallet_address_count.unwrap() as i64);
        wallet_table["hardened_address"] = toml_edit::value(self.wallet_hardened_address.unwrap());
        wallet_table["mnemonic_dictionary"] =
          toml_edit::value(self.wallet_mnemonic_dictionary.clone().unwrap());
      }
    }

    {
      let gui_section = doc["gui"].or_insert(toml_edit::Item::Table(Default::default()));
      if let toml_edit::Item::Table(gui_table) = gui_section {
        gui_table["save_size"] = toml_edit::value(self.gui_save_size.unwrap());
        gui_table["last_width"] = toml_edit::value(self.gui_last_width.unwrap() as i64);
        gui_table["last_height"] = toml_edit::value(self.gui_last_height.unwrap() as i64);
        gui_table["maximized"] = toml_edit::value(self.gui_maximized.unwrap());
        gui_table["theme"] = toml_edit::value(self.gui_theme.clone().unwrap());
        gui_table["icons"] = toml_edit::value(self.gui_icons.clone().unwrap());
        gui_table["language"] = toml_edit::value(self.gui_language.clone().unwrap());
        gui_table["search"] = toml_edit::value(self.gui_search.clone().unwrap());
        gui_table["notification_enabled"] = toml_edit::value(self.gui_notifications.unwrap());
        gui_table["notification_timeout"] =
          toml_edit::value(self.gui_notification_timeout.unwrap() as i64);
        gui_table["log"] = toml_edit::value(self.gui_log.unwrap());
        gui_table["log_level"] = toml_edit::value(self.gui_log_level.clone().unwrap());
      }
    }

    {
      let anu_section = doc["anu"].or_insert(toml_edit::Item::Table(Default::default()));
      if let toml_edit::Item::Table(anu_table) = anu_section {
        anu_table["enabled"] = toml_edit::value(self.anu_enabled.unwrap());
        anu_table["data_format"] = toml_edit::value(self.anu_data_format.clone().unwrap());
        anu_table["array_length"] = toml_edit::value(self.anu_array_length.unwrap() as i64);
        anu_table["hex_block_size"] = toml_edit::value(self.anu_hex_block_size.unwrap() as i64);
        anu_table["log"] = toml_edit::value(self.anu_log.unwrap());
        anu_table["timeout"] = toml_edit::value(self.anu_timeout.unwrap() as i64);
      }
    }

    {
      let proxy_section = doc["proxy"].or_insert(toml_edit::Item::Table(Default::default()));
      if let toml_edit::Item::Table(proxy_table) = proxy_section {
        proxy_table["status"] = toml_edit::value(self.proxy_status.unwrap());
        proxy_table["server_address"] =
          toml_edit::value(self.proxy_server_address.clone().unwrap());
        proxy_table["server_port"] = toml_edit::value(self.proxy_server_port.unwrap() as i64);
        proxy_table["use_pac"] = toml_edit::value(self.proxy_use_pac.unwrap());
        proxy_table["script_address"] =
          toml_edit::value(self.proxy_script_address.clone().unwrap());
        proxy_table["login_credentials"] = toml_edit::value(self.proxy_login_credentials.unwrap());
        proxy_table["login_username"] =
          toml_edit::value(self.proxy_login_username.clone().unwrap());
        proxy_table["login_password"] =
          toml_edit::value(self.proxy_login_password.clone().unwrap());
        proxy_table["use_ssl"] = toml_edit::value(self.proxy_use_ssl.unwrap());
        proxy_table["ssl_certificate"] =
          toml_edit::value(self.proxy_ssl_certificate.clone().unwrap());
      }
    }

    let toml_str = doc.to_string();

    if let Err(_err) = qr2m_lib::save_config_to_file(&local_config_file, &toml_str) {
      #[cfg(debug_assertions)]
      eprintln!("\t- Error saving config file: {}", _err);
    };
  }
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

#[derive(Clone)]
struct WalletSettings {
  entropy_string: Option<String>,
  entropy_checksum: Option<String>,
  mnemonic_words: Option<String>,
  mnemonic_passphrase: Option<String>,
  seed: Option<String>,
  master_private_key: Option<String>,
  master_public_key: Option<String>,
  master_private_key_bytes: Option<Vec<u8>>,
  master_chain_code_bytes: Option<Vec<u8>>,
  master_public_key_bytes: Option<Vec<u8>>,
  coin_index: Option<u32>,
  coin_name: Option<String>,
  wallet_import_format: Option<String>,
  public_key_hash: Option<String>,
  key_derivation: Option<String>,
  hash: Option<String>,
}

impl WalletSettings {
  fn new() -> Self {
    Self {
      entropy_string: None,
      entropy_checksum: None,
      mnemonic_words: None,
      mnemonic_passphrase: None,
      seed: None,
      master_private_key: None,
      master_public_key: None,
      master_private_key_bytes: None,
      master_chain_code_bytes: None,
      master_public_key_bytes: None,
      coin_index: None,
      coin_name: None,
      wallet_import_format: None,
      public_key_hash: None,
      key_derivation: None,
      hash: None,
    }
  }
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

#[derive(Clone)]
struct CryptoAddresses {
  id: Option<String>,
  coin_name: Option<String>,
  derivation_path: Option<String>,
  address: Option<String>,
  public_key: Option<String>,
  private_key: Option<String>,
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

struct AppMessages {
  gui_info_bar: Option<gtk::Revealer>,
  message_queue:
    std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<(String, gtk::MessageType)>>>,
  processing: std::sync::Arc<std::sync::Mutex<bool>>,
}

impl AppMessages {
  fn new(info_bar: Option<gtk::Revealer>) -> Self {
    Self {
      gui_info_bar: info_bar,
      message_queue: std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new())),
      processing: std::sync::Arc::new(std::sync::Mutex::new(false)),
    }
  }

  fn queue_message(
    &self,
    new_message: String,
    message_type: gtk::MessageType,
  ) -> FunctionOutput<()> {
    d3bug(">>> queue_message", "debug");
    d3bug(&format!("new_message: {:?}", new_message), "debug");
    d3bug(&format!("message_type: {:?}", message_type), "debug");

    let mut queue = self.message_queue.lock().unwrap();
    let last_message_in_queue = queue.get(queue.len().wrapping_sub(1));
    let default_message: &(String, gtk::MessageType) = &(String::new(), gtk::MessageType::Info);
    let some_message = last_message_in_queue.unwrap_or(default_message);
    let last_message = &some_message.0;

    if &new_message != last_message {
      d3bug(&format!("New notification: {:?}", new_message), "info");
      queue.push_back((new_message, message_type));

      if !*self.processing.lock().unwrap() {
        self.start_message_processor();
        Ok(())
      } else {
        // Queue is empty
        Ok(())
        // Err(AppError::Custom("Not processing message".to_string()))
      }
    } else {
      // d3bug(&format!("Duplicated message: {:?}", new_message), "debug");
      Err(AppError::Custom("Duplicated message".to_string()))
    }
  }

  fn start_message_processor(&self) {
    #[cfg(debug_assertions)]
    println!(
      "[+] {}",
      &t!("log.app_messages.start_message_processor").to_string()
    );

    let info_bar = match &self.gui_info_bar {
      Some(info_bar) => info_bar.clone(),
      None => {
        #[cfg(debug_assertions)]
        eprintln!("\t- Error: info_bar is not initialized.");

        return;
      }
    };

    let queue = self.message_queue.clone();
    let processing = self.processing.clone();

    {
      let mut is_processing = processing.lock().unwrap();
      if *is_processing {
        return;
      }
      *is_processing = true;
    }

    glib::timeout_add_local(std::time::Duration::from_millis(50), {
      let queue = queue.clone();
      let info_bar = info_bar.clone();
      let processing = processing.clone();

      move || {
        let notification_status = {
          let lock_app_settings = APP_SETTINGS.read().unwrap();
          lock_app_settings.gui_notifications.unwrap()
        };

        if !notification_status {
          *processing.lock().unwrap() = false;
          return glib::ControlFlow::Break;
        }

        let mut queue_lock = queue.lock().unwrap();
        if let Some((message, message_type)) = queue_lock.pop_front() {
          AppMessages::create_info_message(&info_bar, &message, message_type);

          let notification_timeout = {
            let lock_app_settings = APP_SETTINGS.read().unwrap();
            lock_app_settings.gui_notification_timeout.unwrap()
          };

          glib::timeout_add_local(
            std::time::Duration::from_secs(notification_timeout as u64),
            {
              let queue = queue.clone();
              let info_bar = info_bar.clone();
              let processing = processing.clone();

              move || {
                info_bar.set_reveal_child(false);

                AppMessages::start_next_message(
                  &queue,
                  &info_bar,
                  &processing,
                  notification_timeout,
                );

                glib::ControlFlow::Break
              }
            },
          );

          glib::ControlFlow::Break
        } else {
          *processing.lock().unwrap() = false;
          glib::ControlFlow::Break
        }
      }
    });
  }

  fn start_next_message(
    queue: &std::sync::Arc<
      std::sync::Mutex<std::collections::VecDeque<(String, gtk::MessageType)>>,
    >,
    info_bar: &gtk::Revealer,
    processing: &std::sync::Arc<std::sync::Mutex<bool>>,
    timeout: u32,
  ) {
    #[cfg(debug_assertions)]
    println!(
      "[+] {}",
      &t!("log.app_messages.start_next_message").to_string()
    );

    let mut queue_lock = queue.lock().unwrap();
    if let Some((message, message_type)) = queue_lock.pop_front() {
      AppMessages::create_info_message(info_bar, &message, message_type);

      let queue = queue.clone();
      let info_bar = info_bar.clone();
      let processing = processing.clone();

      glib::timeout_add_local(std::time::Duration::from_secs(timeout as u64), move || {
        info_bar.set_reveal_child(false);

        AppMessages::start_next_message(&queue, &info_bar, &processing, timeout);

        glib::ControlFlow::Break
      });
    } else {
      *processing.lock().unwrap() = false;
    }
  }

  fn create_info_message(revealer: &gtk::Revealer, message: &str, message_type: gtk::MessageType) {
    #[cfg(debug_assertions)]
    println!(
      "[+] {}",
      &t!("log.app_messages.create_info_message").to_string()
    );

    let message_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let message_label = gtk::Label::new(Some(message));
    message_label.set_hexpand(true);

    match message_type {
      gtk::MessageType::Error => message_box.set_css_classes(&["error-message"]),
      gtk::MessageType::Warning => message_box.set_css_classes(&["warning-message"]),
      _ => message_box.set_css_classes(&["info-message"]),
    }

    let close_button = gtk::Button::with_label(&t!("UI.button.close"));
    let gesture = gtk::GestureClick::new();

    gesture.connect_pressed(clone!(
      #[weak]
      revealer,
      move |_gesture, _n_press, _x, _y| {
        revealer.set_reveal_child(false);
      }
    ));

    message_box.append(&message_label);
    message_box.append(&close_button);

    revealer.add_controller(gesture);
    revealer.set_child(Some(&message_box));
    revealer.set_reveal_child(true);
  }
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

struct AppLog {
  status: std::sync::Arc<std::sync::Mutex<bool>>,
  // log_button: Option<gtk::Button>,
  _messages: Option<std::collections::VecDeque<(String, gtk::MessageType)>>,
}

impl AppLog {
  fn new() -> Self {
    Self {
      status: std::sync::Arc::new(std::sync::Mutex::new(false)),
      // log_button: Some(log_button),
      _messages: None,
    }
  }

  fn initialize_app_log(
    &mut self,
    // log_button: gtk::Button,
    gui_state: Rc<RefCell<GuiState>>,
  ) {
    let status = self.status.clone();
    let is_active = status.lock().unwrap();

    #[cfg(debug_assertions)]
    println!("\t- AppLog status: {}", is_active);

    let new_icon = match *is_active {
      true => "notif",
      false => "log",
    };

    let lock_gui_state = gui_state.borrow();

    if let Some(texture_map) = &lock_gui_state.gui_button_images {
      if let Some(new_texture) = texture_map.get(new_icon) {
        let mut lock_buttons = lock_gui_state.gui_main_buttons.borrow_mut();

        if let Some(log_buttons) = lock_buttons.get_mut("log") {
          for button in log_buttons.iter() {
            let picture = gtk::Picture::new();
            picture.set_paintable(Some(new_texture));
            button.set_child(Some(&picture));
          }
        }
      } else {
        #[cfg(debug_assertions)]
        eprintln!("\t- Error: 'notif' texture not found in gui_button_images");
      }
    } else {
      #[cfg(debug_assertions)]
      eprintln!("\t- Error: gui_button_images is None");
    }

    #[cfg(debug_assertions)]
    println!("\t- Icon changed. Logging starts...");

    // TODO: Show log messages
  }
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

#[derive(Debug, Clone, Copy)]
struct DerivationPath {
  bip: Option<u32>,
  hardened_bip: Option<bool>,
  coin: Option<u32>,
  hardened_coin: Option<bool>,
  address: Option<u32>,
  hardened_address: Option<bool>,
  purpose: Option<u32>,
}

impl Default for DerivationPath {
  fn default() -> Self {
    DerivationPath {
      bip: Some(44),
      hardened_bip: Some(true),
      coin: Some(0),
      hardened_coin: Some(true),
      address: Some(0),
      hardened_address: Some(true),
      purpose: Some(0),
    }
  }
}

impl DerivationPath {
  fn update_field(&mut self, field: &str, value: Option<FieldValue>) {
    match field {
      "bip" => self.bip = value.and_then(|v| v.into_u32()),
      "hardened_bip" => self.hardened_bip = value.and_then(|v| v.into_bool()),
      "coin" => self.coin = value.and_then(|v| v.into_u32()),
      "hardened_coin" => self.hardened_coin = value.and_then(|v| v.into_bool()),
      "address" => self.address = value.and_then(|v| v.into_u32()),
      "hardened_address" => self.hardened_address = value.and_then(|v| v.into_bool()),
      "purpose" => self.purpose = value.and_then(|v| v.into_u32()),
      _ => eprintln!("{}", &t!("\t- error.DP.read")),
    }
  }
}

#[derive(Debug)]
enum FieldValue {
  U32(u32),
  Bool(bool),
}

impl FieldValue {
  fn into_u32(self) -> Option<u32> {
    match self {
      FieldValue::U32(value) => Some(value),
      _ => None,
    }
  }

  fn into_bool(self) -> Option<bool> {
    match self {
      FieldValue::Bool(value) => Some(value),
      _ => None,
    }
  }
}

mod address_implementation {
  use glib::{
    ParamSpecBuilderExt,
    prelude::*,
    subclass::{object::ObjectImpl, types::ObjectSubclass},
  };

  #[derive(Default)]
  pub struct AddressDatabase {
    pub id: std::cell::RefCell<String>,
    pub coin: std::cell::RefCell<String>,
    pub path: std::cell::RefCell<String>,
    pub address: std::cell::RefCell<String>,
    pub public_key: std::cell::RefCell<String>,
    pub private_key: std::cell::RefCell<String>,
  }

  #[glib::object_subclass]
  impl ObjectSubclass for AddressDatabase {
    const NAME: &'static str = "AddressDatabase";

    type Type = super::AddressDatabase;
    type ParentType = glib::Object;
  }

  impl ObjectImpl for AddressDatabase {
    fn properties() -> &'static [glib::ParamSpec] {
      static PROPERTIES: std::sync::OnceLock<Vec<glib::ParamSpec>> = std::sync::OnceLock::new();

      PROPERTIES.get_or_init(|| {
        vec![
          glib::ParamSpecString::builder("id")
            .blurb("Id")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("coin")
            .blurb("Coin")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("path")
            .blurb("Derivation path")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("address")
            .blurb("Address")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("public-key")
            .blurb("Public key")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("private-key")
            .blurb("Private key")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
        ]
      })
    }

    fn set_property(&self, _id: usize, value: &glib::Value, specification: &glib::ParamSpec) {
      match specification.name() {
        "id" => *self.id.borrow_mut() = value.get().unwrap_or_default(),
        "coin" => *self.coin.borrow_mut() = value.get().unwrap_or_default(),
        "path" => *self.path.borrow_mut() = value.get().unwrap_or_default(),
        "address" => *self.address.borrow_mut() = value.get().unwrap_or_default(),
        "public-key" => *self.public_key.borrow_mut() = value.get().unwrap_or_default(),
        "private-key" => *self.private_key.borrow_mut() = value.get().unwrap_or_default(),
        _ => {
          #[cfg(debug_assertions)]
          eprintln!("Unknown property");
        }
      }
    }

    fn property(&self, _id: usize, specification: &glib::ParamSpec) -> glib::Value {
      match specification.name() {
        "id" => self.id.borrow().to_value(),
        "coin" => self.coin.borrow().to_value(),
        "path" => self.path.borrow().to_value(),
        "address" => self.address.borrow().to_value(),
        "public-key" => self.public_key.borrow().to_value(),
        "private-key" => self.private_key.borrow().to_value(),
        _ => unimplemented!(),
      }
    }
  }
}

glib::wrapper! {
    pub struct AddressDatabase(ObjectSubclass<address_implementation::AddressDatabase>);
}

impl AddressDatabase {
  pub fn new(
    id: &str,
    coin: &str,
    path: &str,
    address: &str,
    public_key: &str,
    private_key: &str,
  ) -> Self {
    let builder = glib::Object::builder::<AddressDatabase>()
      .property("id", id)
      .property("coin", coin)
      .property("path", path)
      .property("address", address)
      .property("public-key", public_key)
      .property("private-key", private_key);

    builder.build()
  }
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

#[tokio::main]
async fn main() {
  // #[cfg(feature = "dev")]
  // let start_time = std::time::Instant::now();

  match print_program_info() {
    Ok(_) => {
      d3bug("<<< print_program_info", "debug");
    }
    Err(err) => d3bug(&format!("print_program_info: {:?}", err), "error"),
  };

  match os::detect_os_and_user_dir() {
    Ok(_) => {
      d3bug("<<< detect_os_and_user_dir", "debug");
    }
    Err(err) => d3bug(&format!("detect_os_and_user_dir: {:?}", err), "error"),
  };

  match os::check_local_config() {
    Ok(_) => {
      d3bug("<<< check_local_config", "debug");
    }
    Err(err) => d3bug(&format!("check_local_config: {:?}", err), "error"),
  };

  match AppSettings::load_settings() {
    Ok(_) => {
      d3bug("<<< load_settings", "debug");
    }
    Err(err) => d3bug(&format!("load_settings: {:?}", err), "error"),
  };

  let application = adw::Application::builder()
    .application_id("wtf.r_o0_t.qr2m")
    .build();

  let gui_state = Rc::new(RefCell::new(GuiState::default_config()));

  #[cfg(feature = "dev")]
  match sec::check_security_level() {
    Ok(_) => {
      d3bug("<<< check_security_level", "debug");
    }
    Err(err) => d3bug(&format!("check_security_level: {:?}", err), "error"),
  };

  application.connect_activate(clone!(
    #[strong]
    gui_state,
    move |app| {
      //       #[cfg(not(feature = "dev"))]
      //       match create_main_window(app, gui_state.clone()) {
      //         Ok(window) => {
      //           window.present();
      //           d3bug("<<< create_main_window", "debug");
      //         }
      //         Err(err) => d3bug(&format!("create_main_window: {:?}", err), "error"),
      //       };
      //
      //       #[cfg(feature = "dev")]
      //       {
      //         let local_settings = os::LOCAL_SETTINGS.lock().unwrap();
      //         if local_settings.first_run {
      //           match create_welcome_window(app, gui_state.clone(), None) {
      //             Ok(window) => {
      //               window.present();
      //               d3bug("<<< create_welcome_window", "debug");
      //             }
      //             Err(err) => d3bug(&format!("create_welcome_window: {:?}", err), "error"),
      //           };
      //         } else {
      //           match create_main_window(app, gui_state.clone(), None, Some(start_time)) {
      //             Ok(window) => {
      //               window.present();
      //               d3bug("<<< create_main_window", "debug");
      //             }
      //             Err(err) => d3bug(&format!("create_main_window: {:?}", err), "error"),
      //           };
      //         }
      //       }

      match create_welcome_window(app, gui_state.clone()) {
        Ok(window) => {
          window.present();
          d3bug("<<< create_welcome_window", "debug");
        }
        Err(err) => d3bug(&format!("create_welcome_window: {:?}", err), "error"),
      };
    }
  ));

  application.run();
}

fn print_program_info() -> FunctionOutput<()> {
  d3bug(">>> print_program_info", "debug");
  d3bug(" ██████╗ ██████╗ ██████╗ ███╗   ███╗", "info");
  d3bug("██╔═══██╗██╔══██╗╚════██╗████╗ ████║", "info");
  d3bug("██║   ██║██████╔╝ █████╔╝██╔████╔██║", "info");
  d3bug("██║▄▄ ██║██╔══██╗██╔═══╝ ██║╚██╔╝██║", "info");
  d3bug("╚██████╔╝██║  ██║███████╗██║ ╚═╝ ██║", "info");
  d3bug(" ╚══▀▀═╝ ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝", "info");

  let current_time = SystemTime::now();
  let timestamp = current_time
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs();
  let feature = qr2m_lib::get_active_app_feature();

  d3bug(
    &format!(
      "{} {} ({})",
      APP_DESCRIPTION.unwrap(),
      APP_VERSION.unwrap(),
      feature
    ),
    "info",
  );
  d3bug(&format!("Start time (UNIX): {}", timestamp), "debug");

  d3bug(
    "-.-. --- .--. -.-- .-. .. --. .... - --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.",
    "info",
  );

  Ok(())
}

fn setup_app_actions(
  application: &adw::Application,
  gui_state: Rc<RefCell<GuiState>>,
  app_messages_state: Rc<RefCell<AppMessages>>,
) -> FunctionOutput<()> {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.setup_app_actions").to_string());

  let new = gio::SimpleAction::new("new", None);
  let open = gio::SimpleAction::new("open", None);
  let save = gio::SimpleAction::new("save", None);
  let about = gio::SimpleAction::new("about", None);
  let settings = gio::SimpleAction::new("settings", None);
  let quit = gio::SimpleAction::new("quit", None);

  #[cfg(feature = "dev")]
  let security = gio::SimpleAction::new("security", None);

  #[cfg(feature = "dev")]
  let log = gio::SimpleAction::new("log", None);

  #[cfg(feature = "dev")]
  let test = gio::SimpleAction::new("test", None);

  #[cfg(feature = "dev")]
  let welcome = gio::SimpleAction::new("welcome", None);

  new.connect_activate(clone!(
    #[strong]
    application,
    #[strong]
    gui_state,
    move |_action, _parameter| {
      match create_main_window(
        &application,
        gui_state.clone(),
        None,
        #[cfg(feature = "dev")]
        None,
      ) {
        Ok(window) => {
          window.present();
          d3bug("<<< create_main_window", "debug");
        }
        Err(err) => d3bug(&format!("create_main_window: {:?}", err), "error"),
      };
    }
  ));

  open.connect_activate(clone!(
    #[weak]
    app_messages_state,
    move |_action, _parameter| {
      open_wallet_from_file(&app_messages_state);
    }
  ));

  save.connect_activate(clone!(
    #[weak]
    app_messages_state,
    move |_action, _parameter| {
      save_wallet_to_file(&app_messages_state);
    }
  ));

  about.connect_activate(move |_action, _parameter| {
    create_about_window();
  });

  #[cfg(feature = "dev")]
  log.connect_activate(clone!(
    #[strong]
    gui_state,
    move |_action, _parameter| {
      match create_log_window(gui_state.clone()) {
        Ok(window) => window.present(),
        Err(err) => eprintln!("Error with log window: {:?}", err),
      };
    }
  ));

  #[cfg(feature = "dev")]
  security.connect_activate(move |_action, _parameter| {
    match sec::create_security_window() {
      Ok(window) => {
        window.present();
        d3bug("<<< create_security_window", "debug");
      }
      Err(err) => d3bug(&format!("create_security_window: \n{:?}", err), "error"),
    };
  });

  settings.connect_activate(clone!(
    #[strong]
    gui_state,
    #[weak]
    app_messages_state,
    move |_action, _parameter| {
      match create_settings_window(gui_state.clone(), app_messages_state) {
        Ok(window) => window.present(),
        Err(err) => eprintln!("Error with settings window: {:?}", err),
      };
    }
  ));

  quit.connect_activate(clone!(
    #[strong]
    application,
    move |_action, _parameter| {
      application.quit();
    }
  ));

  #[cfg(feature = "dev")]
  test.connect_activate(clone!(
    // #[strong] gui_state,
    // #[weak] app_messages_state,
    move |_action, _parameter| {
      let anu_window = dev::anu_window();
      anu_window.present();
    }
  ));

  #[cfg(feature = "dev")]
  welcome.connect_activate(clone!(
    #[strong]
    application,
    #[strong]
    gui_state,
    move |_action, _parameter| {
      match create_welcome_window(&application, gui_state.clone()) {
        Ok(window) => {
          d3bug("<<< create_welcome_window", "debug");
          window.present();
        }
        Err(err) => d3bug(&format!("create_welcome_window: {:?}", err), "error"),
      };
    }
  ));

  application.set_accels_for_action("app.new", &["<Primary>N"]);
  application.set_accels_for_action("app.open", &["<Primary>O"]);
  application.set_accels_for_action("app.save", &["<Primary>S"]);
  application.set_accels_for_action("app.quit", &["<Primary>Q"]);
  application.set_accels_for_action("app.about", &["F1"]);
  application.set_accels_for_action("app.settings", &["F5"]);

  #[cfg(feature = "dev")]
  application.set_accels_for_action("app.security", &["F2"]);

  #[cfg(feature = "dev")]
  application.set_accels_for_action("app.test", &["<Primary>T"]);

  #[cfg(feature = "dev")]
  application.set_accels_for_action("app.welcome", &["<Primary>W"]);

  application.add_action(&new);
  application.add_action(&open);
  application.add_action(&save);
  application.add_action(&about);
  application.add_action(&settings);
  application.add_action(&quit);

  #[cfg(feature = "dev")]
  application.add_action(&security);

  #[cfg(feature = "dev")]
  application.add_action(&test);

  #[cfg(feature = "dev")]
  application.add_action(&welcome);

  Ok(())
}

fn create_main_window(
  application: &adw::Application,
  gui_state: Rc<RefCell<GuiState>>,
  last_window: Option<gtk::ApplicationWindow>,
  #[cfg(feature = "dev")] start_time: Option<std::time::Instant>,
) -> FunctionOutput<gtk::ApplicationWindow> {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.create_main_window").to_string());

  let feature = qr2m_lib::get_active_app_feature();

  let window = gtk::ApplicationWindow::builder()
    .application(application)
    .title(format!(
      "{} {} ({})",
      APP_NAME.unwrap(),
      APP_VERSION.unwrap(),
      feature
    ))
    .show_menubar(true)
    .decorated(true)
    // .hide_on_close(true)
    .build();

  if let Some(window) = last_window.as_ref() {
    window.set_visible(false);
  }

  let lock_app_settings = APP_SETTINGS.read().unwrap();
  let window_width = lock_app_settings.gui_last_width.unwrap();
  let window_height = lock_app_settings.gui_last_height.unwrap();
  let gui_maximized = lock_app_settings.gui_maximized.unwrap();
  let gui_language = lock_app_settings.gui_language.clone().unwrap();
  let save_gui_size = lock_app_settings.gui_save_size.unwrap();

  if save_gui_size {
    if gui_maximized {
      window.set_maximized(true);
    } else {
      window.set_default_width(window_width as i32);
      window.set_default_height(window_height as i32);
    }
  }

  match os::switch_locale(&gui_language) {
    Ok(_) => {
      d3bug("<<< switch_locale", "debug");
    }
    Err(err) => d3bug(&format!("switch_locale: {:?}", err), "error"),
  };

  qr2m_lib::setup_css();

  let header_bar = gtk::HeaderBar::new();
  let info_bar = gtk::Revealer::new();
  info_bar.set_transition_type(gtk::RevealerTransitionType::SlideDown);
  info_bar.set_hexpand(true);
  info_bar.add_css_class("info-bar");
  window.set_titlebar(Some(&header_bar));

  let button_names = [
    "new",
    "open",
    "save",
    "about",
    "settings",
    "random",
    #[cfg(feature = "dev")]
    "security",
    #[cfg(feature = "dev")]
    "log",
  ];
  let mut buttons = std::collections::HashMap::new();

  for &name in &button_names {
    let button = gtk::Button::new();
    buttons.insert(name.to_string(), Rc::new(button));
  }

  let gui_theme = lock_app_settings.gui_theme.clone().unwrap();
  let gui_icons = lock_app_settings.gui_icons.clone().unwrap();
  let app_log_status = lock_app_settings.gui_log.unwrap();

  {
    let mut lock_gui_state = gui_state.borrow_mut();
    lock_gui_state.gui_language = Some(gui_language);
    lock_gui_state.gui_theme = Some(gui_theme);
    lock_gui_state.gui_icon_theme = Some(gui_icons);

    #[cfg(feature = "dev")]
    {
      lock_gui_state.gui_log_status = Some(app_log_status);
    }

    for (name, button) in &buttons {
      lock_gui_state.register_button(name.clone(), button.clone());
    }

    lock_gui_state.reload_gui_theme();
    lock_gui_state.reload_gui_icons();
  }

  {
    let settings = gtk::Settings::default().expect("Failed to get GtkSettings");
    settings.connect_gtk_application_prefer_dark_theme_notify(clone!(
      #[strong]
      gui_state,
      move |_| {
        let mut lock_gui_state = gui_state.borrow_mut();
        lock_gui_state.reload_gui_icons();
      }
    ));
  }

  {
    let mut lock_app_log = APP_LOG.lock().unwrap();
    lock_app_log.status = std::sync::Arc::new(std::sync::Mutex::new(app_log_status));

    lock_app_log.initialize_app_log(gui_state.clone());
  }

  let app_messages_state = Rc::new(RefCell::new(AppMessages::new(Some(info_bar.clone()))));

  let button_tooltips = [
    ("new", "Ctrl+N"),
    ("open", "Ctrl+O"),
    ("save", "Ctrl+S"),
    ("about", "F1"),
    ("settings", "F5"),
    ("random", ""),
    #[cfg(feature = "dev")]
    ("security", "F2"),
    #[cfg(feature = "dev")]
    ("log", "F11"),
  ];

  for (name, shortcut) in button_tooltips {
    if let Some(button) = buttons.get(name) {
      button.set_tooltip_text(Some(&t!(
        format!("UI.main.tooltips.{}", name),
        value = shortcut
      )));
    }
  }

  match setup_app_actions(application, gui_state.clone(), app_messages_state.clone()) {
    Ok(_) => {
      #[cfg(debug_assertions)]
      println!("setup_app_actions done");
    }
    Err(err) => eprintln!("\t- Error in function setup_app_actions : {:?}", err),
  };

  header_bar.pack_start(&*buttons["new"]);
  header_bar.pack_start(&*buttons["open"]);
  header_bar.pack_start(&*buttons["save"]);
  header_bar.pack_end(&*buttons["settings"]);
  header_bar.pack_end(&*buttons["about"]);
  #[cfg(feature = "dev")]
  header_bar.pack_end(&*buttons["log"]);
  #[cfg(feature = "dev")]
  header_bar.pack_end(&*buttons["security"]);

  // JUMP: Action: Settings button action
  buttons["settings"].connect_clicked(clone!(
    #[strong]
    gui_state,
    #[strong]
    app_messages_state,
    move |_| {
      match create_settings_window(gui_state.clone(), app_messages_state.clone()) {
        Ok(window) => window.present(),
        Err(err) => eprintln!("Error with settings window: {:?}", err),
      };
    }
  ));

  buttons["about"].connect_clicked(move |_| {
    create_about_window();
  });

  #[cfg(feature = "dev")]
  buttons["log"].connect_clicked(clone!(
    #[strong]
    gui_state,
    move |_| {
      match create_log_window(gui_state.clone()) {
        Ok(window) => window.present(),
        Err(err) => eprintln!("Error with log window: {:?}", err),
      };
    }
  ));

  #[cfg(feature = "dev")]
  buttons["security"].connect_clicked(move |_| {
    match sec::create_security_window() {
      Ok(window) => {
        window.present();
        d3bug("<<< create_security_window", "debug");
      }
      Err(err) => d3bug(&format!("create_security_window: \n{:?}", err), "error"),
    };
  });

  buttons["new"].connect_clicked(clone!(
    #[strong]
    application,
    #[strong]
    gui_state,
    move |_| {
      match create_main_window(
        &application,
        gui_state.clone(),
        None,
        #[cfg(feature = "dev")]
        None,
      ) {
        Ok(window) => {
          window.present();
          d3bug("<<< create_main_window", "debug");
        }
        Err(err) => d3bug(&format!("create_main_window: {:?}", err), "error"),
      };
    }
  ));

  buttons["save"].connect_clicked(clone!(
    #[strong]
    app_messages_state,
    move |_| {
      save_wallet_to_file(&app_messages_state.clone());
    }
  ));

  let stack = Stack::new();
  let stack_sidebar = StackSidebar::new();
  stack_sidebar.set_stack(&stack);

  // JUMP: Sidebar 1: Seed
  let sidebar_seed_main_box = gtk::Box::builder()
    .orientation(gtk::Orientation::Vertical)
    .spacing(20)
    .margin_bottom(10)
    .margin_start(10)
    .margin_top(10)
    .margin_end(10)
    .build();

  let sidebar_seed_header_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
  let sidebar_seed_button_box = gtk::Box::builder()
    .orientation(gtk::Orientation::Horizontal)
    .spacing(20)
    .halign(gtk::Align::Center)
    .build();

  let sidebar_seed_result_box = gtk::Box::new(gtk::Orientation::Vertical, 10);

  sidebar_seed_main_box.append(&sidebar_seed_header_box);
  sidebar_seed_main_box.append(&sidebar_seed_button_box);
  sidebar_seed_main_box.append(&sidebar_seed_result_box);

  let sidebar_seed_header_entropy_options = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  let sidebar_seed_header_mnemonic_options = gtk::Box::new(gtk::Orientation::Vertical, 10);

  // Entropy Source
  let entropy_source_box = gtk::Box::builder()
    .orientation(gtk::Orientation::Vertical)
    .spacing(10)
    .hexpand(true)
    .build();

  let entropy_source_frame = gtk::Frame::builder()
    .label(t!("UI.main.seed.entropy.source").to_string())
    .hexpand(true)
    .build();

  let anu_enabled = cfg!(feature = "full") && lock_app_settings.anu_enabled.unwrap_or(false);
  let valid_entropy_sources: Vec<&str> = if anu_enabled {
    VALID_ENTROPY_SOURCES.to_vec()
  } else {
    VALID_ENTROPY_SOURCES
      .iter()
      .filter(|&&x| x != "QRNG")
      .copied()
      .collect()
  };

  let valid_entropy_source_as_strings: Vec<String> =
    valid_entropy_sources.iter().map(|&x| x.into()).collect();

  let valid_entropy_source_as_str_refs: Vec<&str> = valid_entropy_source_as_strings
    .iter()
    .map(|s| s.as_ref())
    .collect();

  let entropy_source_dropdown = gtk::DropDown::from_strings(&valid_entropy_source_as_str_refs);
  let wallet_entropy_source = lock_app_settings.wallet_entropy_source.clone().unwrap();
  let default_entropy_source = valid_entropy_source_as_strings
    .iter()
    .position(|s| *s == wallet_entropy_source)
    .unwrap_or(0);

  entropy_source_dropdown.set_selected(default_entropy_source.try_into().unwrap());

  // Entropy length
  let entropy_length_box = gtk::Box::builder()
    .orientation(gtk::Orientation::Vertical)
    .spacing(20)
    .hexpand(true)
    .build();

  let entropy_length_frame = gtk::Frame::builder()
    .label(t!("UI.main.seed.entropy.length").to_string())
    .hexpand(true)
    .build();

  let valid_entropy_lengths_as_strings: Vec<String> = VALID_ENTROPY_LENGTHS
    .iter()
    .map(|&x| x.to_string())
    .collect();

  let valid_entropy_lengths_as_str_refs: Vec<&str> = valid_entropy_lengths_as_strings
    .iter()
    .map(|s| s.as_ref())
    .collect();

  let entropy_length_dropdown = gtk::DropDown::from_strings(&valid_entropy_lengths_as_str_refs);
  let wallet_entropy_length = lock_app_settings.wallet_entropy_length.unwrap();
  let default_entropy_length = valid_entropy_lengths_as_strings
    .iter()
    .position(|x| x.parse::<u32>().unwrap() == wallet_entropy_length)
    .unwrap_or(0);

  entropy_length_dropdown.set_selected(default_entropy_length as u32);

  // Mnemonic length
  let mnemonic_passphrase_length_box = gtk::Box::builder()
    .orientation(gtk::Orientation::Horizontal)
    .spacing(10)
    .build();

  let mnemonic_passphrase_items_box = gtk::Box::builder()
    .orientation(gtk::Orientation::Horizontal)
    .spacing(0)
    .build();

  let mnemonic_passphrase_scale_box = gtk::Box::builder()
    .orientation(gtk::Orientation::Vertical)
    .spacing(10)
    .hexpand(true)
    .build();

  let mnemonic_passphrase_info_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
  let mnemonic_passphrase_length_frame = gtk::Frame::builder()
    .label(t!("UI.main.seed.mnemonic.length").to_string())
    .hexpand(true)
    .build();

  let default_mnemonic_length = lock_app_settings.wallet_mnemonic_length.unwrap();
  let mnemonic_passphrase_adjustment = gtk::Adjustment::new(
    default_mnemonic_length as f64,
    0.0,
    8.0 * 128.0 * 4.0,
    1.0,
    100.0,
    0.0,
  );

  let mnemonic_passphrase_scale = gtk::Scale::new(
    gtk::Orientation::Horizontal,
    Some(&mnemonic_passphrase_adjustment),
  );
  let mnemonic_passphrase_length_info = gtk::Entry::new();

  mnemonic_passphrase_length_info.set_editable(false);
  mnemonic_passphrase_length_info.set_width_request(50);
  mnemonic_passphrase_length_info.set_input_purpose(gtk::InputPurpose::Digits);
  mnemonic_passphrase_length_info.set_text(&default_mnemonic_length.to_string());

  let mnemonic_passphrase_main_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  mnemonic_passphrase_main_box.set_hexpand(true);

  let mnemonic_passphrase_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let mnemonic_passphrase_frame = gtk::Frame::new(Some(&t!("UI.main.seed.mnemonic.pass")));
  let mnemonic_passphrase_text = gtk::Entry::new();
  mnemonic_passphrase_text.set_hexpand(true);

  let mnemonic_dictionary_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let mnemonic_dictionary_frame = gtk::Frame::new(Some(&t!("UI.main.seed.mnemonic.dictionary")));
  mnemonic_dictionary_box.append(&mnemonic_dictionary_frame);

  let valid_dictionaries_as_strings: Vec<String> = VALID_MNEMONIC_DICTIONARY
    .iter()
    .map(|&x| x.to_string())
    .collect();

  let valid_dictionaries_as_str_ref: Vec<&str> = valid_dictionaries_as_strings
    .iter()
    .map(|s| s.as_ref())
    .collect();

  let mnemonic_dictionary_dropdown = gtk::DropDown::from_strings(&valid_dictionaries_as_str_ref);

  let default_mnemonic_dictionary = valid_dictionaries_as_strings
    .iter()
    .position(|x| {
      x.parse::<String>().unwrap()
        == lock_app_settings
          .wallet_mnemonic_dictionary
          .clone()
          .unwrap()
    })
    .unwrap_or(0);

  mnemonic_dictionary_dropdown.set_selected(default_mnemonic_dictionary.try_into().unwrap());

  mnemonic_dictionary_frame.set_child(Some(&mnemonic_dictionary_dropdown));

  let generate_seed_button = gtk::Button::new();
  generate_seed_button.set_width_request(200);
  generate_seed_button.set_label(&t!("UI.main.seed.generate"));

  let delete_seed_button = gtk::Button::new();
  delete_seed_button.set_width_request(200);
  delete_seed_button.set_label(&t!("UI.main.seed.delete"));

  // Copy button!
  let entropy_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  let entropy_frame = gtk::Frame::new(Some(&t!("UI.main.seed.entropy")));
  let entropy_text = gtk::TextView::new();
  entropy_text.set_vexpand(true);
  entropy_text.set_hexpand(true);
  entropy_text.set_wrap_mode(gtk::WrapMode::Char);
  entropy_text.set_editable(false);
  entropy_text.set_left_margin(5);
  entropy_text.set_top_margin(5);

  let entropy_inner_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
  entropy_inner_box.append(&entropy_text);

  let copy_entropy_button = gtk::Button::new();
  copy_entropy_button.set_icon_name("edit-copy");
  copy_entropy_button.set_tooltip_text(Some(&t!("UI.button.copy")));
  entropy_inner_box.append(&copy_entropy_button);

  let import_entropy_button = gtk::Button::new();
  import_entropy_button.set_icon_name("document-revert");
  import_entropy_button.set_tooltip_text(Some(&t!("UI.button.import")));
  entropy_inner_box.append(&import_entropy_button);

  entropy_frame.set_child(Some(&entropy_inner_box));
  entropy_box.append(&entropy_frame);

  copy_entropy_button.connect_clicked(clone!(
    #[weak]
    entropy_text,
    move |button| {
      let buffer = entropy_text.buffer();
      let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
      if !text.is_empty() {
        let display = button.display();
        let clipboard = display.clipboard();
        clipboard.set_text(&text);
      };
    }
  ));

  let mnemonic_words_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
  let mnemonic_words_frame = gtk::Frame::new(Some(&t!("UI.main.seed.mnemonic.words")));
  let mnemonic_words_text = gtk::TextView::new();
  mnemonic_words_text.set_vexpand(true);
  mnemonic_words_text.set_hexpand(true);
  mnemonic_words_text.set_wrap_mode(gtk::WrapMode::Word);
  mnemonic_words_text.set_editable(false);
  mnemonic_words_text.set_left_margin(5);
  mnemonic_words_text.set_top_margin(5);

  let mnemonic_inner_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
  mnemonic_inner_box.append(&mnemonic_words_text);

  let copy_mnemonic_button = gtk::Button::new();
  copy_mnemonic_button.set_icon_name("edit-copy");
  copy_mnemonic_button.set_tooltip_text(Some(&t!("UI.button.copy")));
  mnemonic_inner_box.append(&copy_mnemonic_button);

  let import_mnemonic_button = gtk::Button::new();
  import_mnemonic_button.set_icon_name("document-revert");
  import_mnemonic_button.set_tooltip_text(Some(&t!("UI.button.import")));
  mnemonic_inner_box.append(&import_mnemonic_button);

  mnemonic_words_frame.set_child(Some(&mnemonic_inner_box));
  mnemonic_words_box.append(&mnemonic_words_frame);

  copy_mnemonic_button.connect_clicked(clone!(
    #[weak]
    mnemonic_words_text,
    move |button| {
      let buffer = mnemonic_words_text.buffer();
      let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
      if !text.is_empty() {
        let display = button.display();
        let clipboard = display.clipboard();
        clipboard.set_text(&text);
      };
    }
  ));

  let seed_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
  let seed_frame = gtk::Frame::new(Some(&t!("UI.main.seed")));
  let seed_text = gtk::TextView::new();
  seed_text.set_vexpand(true);
  seed_text.set_hexpand(true);
  seed_text.set_wrap_mode(gtk::WrapMode::Char);
  seed_text.set_editable(false);
  seed_text.set_left_margin(5);
  seed_text.set_top_margin(5);

  let seed_inner_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
  seed_inner_box.append(&seed_text);

  let copy_seed_button = gtk::Button::new();
  copy_seed_button.set_icon_name("edit-copy");
  copy_seed_button.set_tooltip_text(Some(&t!("UI.button.copy")));
  seed_inner_box.append(&copy_seed_button);

  let import_seed_button = gtk::Button::new();
  import_seed_button.set_icon_name("document-revert");
  import_seed_button.set_tooltip_text(Some(&t!("UI.button.import")));
  seed_inner_box.append(&import_seed_button);

  seed_frame.set_child(Some(&seed_inner_box));
  seed_box.append(&seed_frame);

  copy_seed_button.connect_clicked(clone!(
    #[weak]
    seed_text,
    move |button| {
      let buffer = seed_text.buffer();
      let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
      if !text.is_empty() {
        let display = button.display();
        let clipboard = display.clipboard();
        clipboard.set_text(&text);
      };
    }
  ));

  import_entropy_button.connect_clicked(clone!(
    #[strong]
    app_messages_state,
    #[weak]
    entropy_text,
    #[weak]
    mnemonic_words_text,
    #[weak]
    mnemonic_passphrase_text,
    #[weak]
    seed_text,
    #[weak]
    mnemonic_dictionary_dropdown,
    move |_| {
      let import_entropy_dialog = gtk::ApplicationWindow::builder()
        .title("Import Entropy")
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .height_request(150)
        .width_request(500)
        .resizable(false)
        .modal(true)
        .build();

      let main_dialog_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
      main_dialog_box.set_margin_bottom(10);
      main_dialog_box.set_margin_top(10);
      main_dialog_box.set_margin_start(10);
      main_dialog_box.set_margin_end(10);
      main_dialog_box.set_hexpand(true);
      main_dialog_box.set_vexpand(true);

      let dialog_frame = gtk::Frame::new(Some(&t!("UI.main.seed.entropy")));

      let entropy_import_text = gtk::TextView::new();
      entropy_import_text.set_wrap_mode(gtk::WrapMode::Char);
      entropy_import_text.set_hexpand(true);
      entropy_import_text.set_vexpand(true);

      dialog_frame.set_child(Some(&entropy_import_text));

      let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
      button_box.set_hexpand(true);
      button_box.set_halign(gtk::Align::Center);

      let import_button = gtk::Button::with_label("Import");
      let close_button = gtk::Button::with_label("Close");
      button_box.append(&import_button);
      button_box.append(&close_button);

      main_dialog_box.append(&dialog_frame);
      main_dialog_box.append(&button_box);

      import_entropy_dialog.set_child(Some(&main_dialog_box));

      import_button.connect_clicked(clone!(
        #[strong]
        app_messages_state,
        #[weak]
        import_entropy_dialog,
        #[weak]
        entropy_text,
        #[weak]
        entropy_import_text,
        #[weak]
        mnemonic_words_text,
        #[weak]
        mnemonic_passphrase_text,
        #[weak]
        seed_text,
        #[weak]
        mnemonic_dictionary_dropdown,
        move |_| {
          let buffer = entropy_import_text.buffer();
          let text = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), false)
            .to_string();

          if qr2m_lib::is_valid_entropy(&text) {
            #[cfg(debug_assertions)]
            println!("Imported entropy: {}", text);

            entropy_text.buffer().set_text(&text);

            let mnemonic_dictionary = {
              let lock_app_settings = APP_SETTINGS.read().unwrap();
              lock_app_settings
                .wallet_mnemonic_dictionary
                .clone()
                .unwrap()
            };

            let valid_mnemonic_dictionary_as_strings: Vec<String> = VALID_MNEMONIC_DICTIONARY
              .iter()
              .map(|&x| x.to_string())
              .collect();

            let default_mnemonic_dictionary = valid_mnemonic_dictionary_as_strings
              .iter()
              .position(|s| *s == mnemonic_dictionary)
              .unwrap_or(0);

            if let Ok(index) = default_mnemonic_dictionary.try_into() {
              mnemonic_dictionary_dropdown.set_selected(index);
            } else {
              eprintln!("\t- Invalid index for coin_search_filter_dropdown");
              mnemonic_dictionary_dropdown.set_selected(0);
            }

            let mnemonic_words =
              match keys::generate_mnemonic_words(&text, Some(&mnemonic_dictionary)) {
                Ok(mnemonic) => {
                  d3bug("<<< generate_mnemonic_words", "debug");
                  mnemonic
                }
                Err(e) => {
                  d3bug(&format!("generate_mnemonic_words: {:?}", e), "error");
                  return;
                }
              };
            mnemonic_words_text.buffer().set_text(&mnemonic_words);

            let seed = match keys::generate_seed_from_mnemonic(
              &mnemonic_words,
              &mnemonic_passphrase_text.buffer().text(),
            ) {
              Ok(seed) => {
                d3bug("<<< generate_seed_from_mnemonic", "debug");
                seed
              }
              Err(e) => {
                d3bug(&format!("generate_seed_from_mnemonic: {:?}", e), "error");
                return;
              }
            };

            let seed = match keys::convert_seed_to_mnemonic(&seed) {
              Ok(seed) => {
                d3bug("<<< convert_seed_to_mnemonic", "debug");
                seed
              }
              Err(e) => {
                d3bug(&format!("convert_seed_to_mnemonic: {:?}", e), "error");
                return;
              }
            };

            seed_text.buffer().set_text(&seed);
          } else {
            eprintln!("\t Imported entropy invalid: {}", text);

            let lock_app_messages = app_messages_state.borrow();
            match lock_app_messages.queue_message(
              t!("error.entropy.invalid").to_string(),
              gtk::MessageType::Error,
            ) {
              Ok(_) => {}
              Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
            };
          };

          import_entropy_dialog.close();
        }
      ));

      close_button.connect_clicked(clone!(
        #[weak]
        import_entropy_dialog,
        move |_| {
          import_entropy_dialog.close();
        }
      ));

      import_entropy_dialog.present();
    }
  ));

  import_mnemonic_button.connect_clicked(clone!(
    #[weak]
    mnemonic_words_text,
    #[weak]
    mnemonic_passphrase_text,
    #[weak]
    seed_text,
    #[weak]
    mnemonic_dictionary_dropdown,
    move |_| {
      let import_mnemonic_dialog = gtk::ApplicationWindow::builder()
        .title("Import Mnemonic")
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .height_request(150)
        .width_request(500)
        .resizable(false)
        .modal(true)
        .build();

      let main_dialog_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
      main_dialog_box.set_margin_bottom(10);
      main_dialog_box.set_margin_top(10);
      main_dialog_box.set_margin_start(10);
      main_dialog_box.set_margin_end(10);
      main_dialog_box.set_hexpand(true);
      main_dialog_box.set_vexpand(true);

      let dialog_frame = gtk::Frame::new(Some(&t!("UI.main.seed.mnemonic.words")));

      let mnemonic_import_text = gtk::TextView::new();
      mnemonic_import_text.set_wrap_mode(gtk::WrapMode::Word);
      mnemonic_import_text.set_hexpand(true);
      mnemonic_import_text.set_vexpand(true);

      dialog_frame.set_child(Some(&mnemonic_import_text));

      let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
      button_box.set_hexpand(true);
      button_box.set_halign(gtk::Align::Center);

      let import_button = gtk::Button::with_label("Import");
      let close_button = gtk::Button::with_label("Close");
      button_box.append(&import_button);
      button_box.append(&close_button);

      main_dialog_box.append(&dialog_frame);
      main_dialog_box.append(&button_box);

      import_mnemonic_dialog.set_child(Some(&main_dialog_box));

      import_button.connect_clicked(clone!(
        #[weak]
        import_mnemonic_dialog,
        #[weak]
        mnemonic_words_text,
        #[weak]
        mnemonic_passphrase_text,
        #[weak]
        seed_text,
        #[weak]
        mnemonic_dictionary_dropdown,
        move |_| {
          let buffer = mnemonic_import_text.buffer();
          let text = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), false)
            .to_string();

          // TODO: check if imported mnemonic is valid
          // if qr2m_lib::is_valid_mnemonic(&text) {
          #[cfg(debug_assertions)]
          println!("Imported mnemonic: {}", text);
          mnemonic_words_text.buffer().set_text(&text);

          let mnemonic_dictionary = {
            let lock_app_settings = APP_SETTINGS.read().unwrap();
            lock_app_settings
              .wallet_mnemonic_dictionary
              .clone()
              .unwrap()
          };

          let valid_mnemonic_dictionary_as_strings: Vec<String> = VALID_MNEMONIC_DICTIONARY
            .iter()
            .map(|&x| x.to_string())
            .collect();

          let default_mnemonic_dictionary = valid_mnemonic_dictionary_as_strings
            .iter()
            .position(|s| *s == mnemonic_dictionary)
            .unwrap_or(0);

          if let Ok(index) = default_mnemonic_dictionary.try_into() {
            mnemonic_dictionary_dropdown.set_selected(index);
          } else {
            eprintln!("\t- Invalid index for coin_search_filter_dropdown");
            mnemonic_dictionary_dropdown.set_selected(0);
          }

          let seed = match keys::generate_seed_from_mnemonic(
            &text,
            &mnemonic_passphrase_text.buffer().text(),
          ) {
            Ok(seed) => {
              d3bug("<<< generate_seed_from_mnemonic", "debug");
              seed
            }
            Err(e) => {
              d3bug(&format!("generate_seed_from_mnemonic: {:?}", e), "error");
              return;
            }
          };

          let seed = match keys::convert_seed_to_mnemonic(&seed) {
            Ok(seed) => {
              d3bug("<<< convert_seed_to_mnemonic", "debug");
              seed
            }
            Err(e) => {
              d3bug(&format!("convert_seed_to_mnemonic: {:?}", e), "error");
              return;
            }
          };

          seed_text.buffer().set_text(&seed);
          import_mnemonic_dialog.close();
        }
      ));

      close_button.connect_clicked(clone!(
        #[weak]
        import_mnemonic_dialog,
        move |_| {
          import_mnemonic_dialog.close();
        }
      ));

      import_mnemonic_dialog.present();
    }
  ));

  import_seed_button.connect_clicked(clone!(
    #[strong]
    app_messages_state,
    #[weak]
    seed_text,
    move |_| {
      let import_seed_dialog = gtk::ApplicationWindow::builder()
        .title("Import Seed")
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .height_request(150)
        .width_request(500)
        .resizable(false)
        .modal(true)
        .build();

      let main_dialog_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
      main_dialog_box.set_margin_bottom(10);
      main_dialog_box.set_margin_top(10);
      main_dialog_box.set_margin_start(10);
      main_dialog_box.set_margin_end(10);
      main_dialog_box.set_hexpand(true);
      main_dialog_box.set_vexpand(true);

      let dialog_frame = gtk::Frame::new(Some(&t!("UI.main.seed")));

      let seed_import_text = gtk::TextView::new();
      seed_import_text.set_wrap_mode(gtk::WrapMode::Char);
      seed_import_text.set_hexpand(true);
      seed_import_text.set_vexpand(true);

      dialog_frame.set_child(Some(&seed_import_text));

      let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
      button_box.set_hexpand(true);
      button_box.set_halign(gtk::Align::Center);

      let import_button = gtk::Button::with_label("Import");
      let close_button = gtk::Button::with_label("Close");
      button_box.append(&import_button);
      button_box.append(&close_button);

      main_dialog_box.append(&dialog_frame);
      main_dialog_box.append(&button_box);

      import_seed_dialog.set_child(Some(&main_dialog_box));

      import_button.connect_clicked(clone!(
        #[strong]
        app_messages_state,
        #[weak]
        seed_text,
        #[weak]
        import_seed_dialog,
        move |_| {
          let buffer = seed_import_text.buffer();
          let text = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), false)
            .to_string();

          if qr2m_lib::is_valid_seed(&text) {
            #[cfg(debug_assertions)]
            println!("Imported seed: {}", text);
            seed_text.buffer().set_text(&text);
          } else {
            eprintln!("\t Imported seed invalid: {}", text);

            let lock_app_messages = app_messages_state.borrow();
            match lock_app_messages.queue_message(
              t!("error.seed.invalid").to_string(),
              gtk::MessageType::Error,
            ) {
              Ok(_) => {}
              Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
            };
          };

          import_seed_dialog.close();
        }
      ));

      close_button.connect_clicked(clone!(
        #[weak]
        import_seed_dialog,
        move |_| {
          import_seed_dialog.close();
        }
      ));

      import_seed_dialog.present();
    }
  ));

  mnemonic_passphrase_info_box.append(&mnemonic_passphrase_length_info);
  mnemonic_passphrase_scale_box.append(&mnemonic_passphrase_scale);

  mnemonic_passphrase_items_box.append(&mnemonic_passphrase_scale_box);
  mnemonic_passphrase_items_box.append(&mnemonic_passphrase_info_box);

  mnemonic_passphrase_content_box.append(&mnemonic_passphrase_text);
  mnemonic_passphrase_content_box.append(&*buttons["random"]);

  entropy_length_frame.set_child(Some(&entropy_length_dropdown));
  entropy_source_frame.set_child(Some(&entropy_source_dropdown));

  mnemonic_passphrase_length_frame.set_child(Some(&mnemonic_passphrase_items_box));
  mnemonic_passphrase_frame.set_child(Some(&mnemonic_passphrase_content_box));

  entropy_source_box.append(&entropy_source_frame);
  entropy_length_box.append(&entropy_length_frame);

  mnemonic_passphrase_main_box.append(&mnemonic_passphrase_frame);
  mnemonic_passphrase_length_box.append(&mnemonic_passphrase_length_frame);

  sidebar_seed_header_entropy_options.append(&entropy_source_box);
  sidebar_seed_header_entropy_options.append(&entropy_length_box);

  sidebar_seed_header_mnemonic_options.append(&mnemonic_passphrase_main_box);

  let extra_mnemonic_options = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  extra_mnemonic_options.append(&mnemonic_passphrase_length_box);
  extra_mnemonic_options.append(&mnemonic_dictionary_box);

  sidebar_seed_header_mnemonic_options.append(&extra_mnemonic_options);

  sidebar_seed_header_box.append(&sidebar_seed_header_entropy_options);
  sidebar_seed_header_box.append(&sidebar_seed_header_mnemonic_options);

  sidebar_seed_button_box.append(&generate_seed_button);
  sidebar_seed_button_box.append(&delete_seed_button);

  sidebar_seed_result_box.append(&entropy_box);
  sidebar_seed_result_box.append(&mnemonic_words_box);
  sidebar_seed_result_box.append(&seed_box);

  stack.add_titled(
    &sidebar_seed_main_box,
    Some("sidebar-seed"),
    &t!("UI.main.seed"),
  );

  // JUMP: Sidebar 2: Coin
  let coin_main_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
  let coin_main_content_box = gtk::Box::new(gtk::Orientation::Vertical, 20);

  coin_main_box.append(&coin_main_content_box);
  coin_main_box.set_margin_top(10);
  coin_main_box.set_margin_start(10);
  coin_main_box.set_margin_end(10);
  coin_main_box.set_margin_bottom(10);

  // JUMP: Filter coins
  // 0      not supported
  // 1      verified
  // 2      not verified
  // 3      in plan
  // Coin filter
  let coin_filter_main_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let coin_filter_main_frame = gtk::Frame::new(Some(&t!("UI.main.coin.filter")));
  let coin_filter_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  coin_filter_main_frame.set_child(Some(&coin_filter_content_box));
  coin_filter_main_box.append(&coin_filter_main_frame);
  coin_filter_main_box.set_hexpand(true);
  coin_filter_main_frame.set_hexpand(true);
  coin_filter_content_box.set_hexpand(true);

  let filter_top10_coins_button_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
  let filter_top10_coins_button =
    gtk::Button::with_label(&t!("UI.main.coin.filter.status.top", value = 10));
  filter_top10_coins_button_box.append(&filter_top10_coins_button);
  coin_filter_content_box.append(&filter_top10_coins_button_box);
  filter_top10_coins_button_box.set_hexpand(true);

  let filter_top100_coins_button_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
  let filter_top100_coins_button =
    gtk::Button::with_label(&t!("UI.main.coin.filter.status.top", value = 100));
  filter_top100_coins_button_box.append(&filter_top100_coins_button);
  coin_filter_content_box.append(&filter_top100_coins_button_box);
  filter_top100_coins_button_box.set_hexpand(true);

  let filter_verified_coins_button_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
  let filter_verified_coins_button = gtk::Button::with_label(&t!(
    "UI.main.coin.filter.status.verified",
    value = coin_db::COIN_STATUS_VERIFIED
  ));
  filter_verified_coins_button_box.append(&filter_verified_coins_button);
  coin_filter_content_box.append(&filter_verified_coins_button_box);
  filter_verified_coins_button_box.set_hexpand(true);

  let filter_not_verified_coins_button_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
  let filter_not_verified_coins_button = gtk::Button::with_label(&t!(
    "UI.main.coin.filter.status.not_verified",
    value = coin_db::COIN_STATUS_NOT_VERIFIED
  ));
  filter_not_verified_coins_button_box.append(&filter_not_verified_coins_button);
  coin_filter_content_box.append(&filter_not_verified_coins_button_box);
  filter_not_verified_coins_button_box.set_hexpand(true);

  let filter_not_supported_coins_button_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
  let filter_not_supported_coins_button = gtk::Button::with_label(&t!(
    "UI.main.coin.filter.status.not_supported",
    value = coin_db::COIN_STATUS_NOT_SUPPORTED
  ));
  filter_not_supported_coins_button_box.append(&filter_not_supported_coins_button);
  coin_filter_content_box.append(&filter_not_supported_coins_button_box);
  filter_not_supported_coins_button_box.set_hexpand(true);
  coin_main_content_box.append(&coin_filter_main_box);

  // Coin search
  let coin_filter_main_frame = gtk::Frame::new(Some(&t!("UI.main.coin.search")));
  let search_coin_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  coin_filter_main_frame.set_child(Some(&search_coin_content_box));

  // Search entry
  let coin_search = gtk::SearchEntry::new();
  search_coin_content_box.append(&coin_search);
  coin_search.set_hexpand(true);

  // Advance search
  let advance_search_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let advance_search_label = gtk::Label::new(Some(&t!("UI.main.coin.search.advance")));
  let advance_search_checkbox = gtk::CheckButton::new();

  advance_search_content_box.append(&advance_search_label);
  advance_search_content_box.append(&advance_search_checkbox);
  search_coin_content_box.append(&advance_search_content_box);

  // Search filter
  let advance_search_filter_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let valid_coin_search_filter_as_strings: Vec<String> = VALID_COIN_SEARCH_PARAMETER
    .iter()
    .map(|&x| x.to_string())
    .collect();
  let valid_coin_search_filter_as_str_refs: Vec<&str> = valid_coin_search_filter_as_strings
    .iter()
    .map(|s| s.as_ref())
    .collect();
  let coin_search_filter_dropdown =
    gtk::DropDown::from_strings(&valid_coin_search_filter_as_str_refs);

  let gui_search = lock_app_settings.gui_search.clone().unwrap();
  let default_coin_search_filter = valid_coin_search_filter_as_strings
    .iter()
    .position(|s| *s == gui_search)
    .unwrap_or(0);

  if let Ok(index) = default_coin_search_filter.try_into() {
    coin_search_filter_dropdown.set_selected(index);
  } else {
    eprintln!("\t- Invalid index for coin_search_filter_dropdown");
    coin_search_filter_dropdown.set_selected(0);
  }

  advance_search_filter_box.set_visible(false);
  advance_search_filter_box.append(&coin_search_filter_dropdown);
  search_coin_content_box.append(&advance_search_filter_box);
  coin_main_content_box.append(&coin_filter_main_frame);
  coin_search.set_placeholder_text(Some(&t!(
    "UI.main.coin.search.text",
    value = valid_coin_search_filter_as_strings[default_coin_search_filter]
  )));

  // Coin treeview
  let scrolled_window = gtk::ScrolledWindow::new();
  let coin_frame = gtk::Frame::new(Some(&t!("UI.main.coin")));

  match coin_db::create_coin_completion_model() {
    Ok(_) => {
      d3bug("<<< create_coin_completion_model", "debug");
    }
    Err(err) => d3bug(&format!("create_coin_completion_model: {:?}", err), "error"),
  };

  let coin_store = coin_db::create_coin_store()?;
  let cmc_top_filter = coin_db::create_coin_store_filters("Cmc_top", "100")?;
  let status_filter = coin_db::create_coin_store_filters("Status", "Verified")?;
  let combined_filter = gtk::EveryFilter::new();

  combined_filter.append(cmc_top_filter);
  combined_filter.append(status_filter);

  let filter_model = gtk::FilterListModel::new(Some(coin_store), Some(combined_filter));
  let sorter = coin_db::create_sorter()?;
  let sort_model = gtk::SortListModel::new(Some(filter_model.clone()), Some(sorter));

  let coin_selection_model = gtk::SingleSelection::new(Some(sort_model.clone()));
  let column_view = gtk::ColumnView::new(Some(coin_selection_model.clone()));

  let coin_single_selection = coin_selection_model
    .clone()
    .downcast::<gtk::SingleSelection>()
    .expect("The selection model is not a SingleSelection");

  column_view.set_vexpand(true);
  column_view.set_show_column_separators(true);

  let columns = [
    ("status", &t!("UI.main.database.column.status").to_string()),
    (
      "coin-index",
      &t!("UI.main.database.column.index").to_string(),
    ),
    (
      "coin-symbol",
      &t!("UI.main.database.column.symbol").to_string(),
    ),
    ("coin-name", &t!("UI.main.database.column.coin").to_string()),
    (
      "key-derivation",
      &t!("UI.main.database.column.key_derivation").to_string(),
    ),
    ("hash", &t!("UI.main.database.column.hash").to_string()),
    #[cfg(feature = "dev")]
    (
      "private-header",
      &t!("UI.main.database.column.priv_header").to_string(),
    ),
    #[cfg(feature = "dev")]
    (
      "public-header",
      &t!("UI.main.database.column.pub_header").to_string(),
    ),
    #[cfg(feature = "dev")]
    (
      "public-key-hash",
      &t!("UI.main.database.column.pub_hash").to_string(),
    ),
    #[cfg(feature = "dev")]
    (
      "script-hash",
      &t!("UI.main.database.column.script").to_string(),
    ),
    #[cfg(feature = "dev")]
    (
      "wallet-import-format",
      &t!("UI.main.database.column.wif").to_string(),
    ),
    ("evm", &t!("UI.main.database.column.evm").to_string()),
    ("ucid", &t!("UI.main.database.column.UCID").to_string()),
    ("cmc-top", &t!("UI.main.database.column.cmc").to_string()),
  ];

  let create_string_factory = |property: &str| {
    let property = property.to_string();
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
      let list_item = list_item
        .downcast_ref::<gtk::ListItem>()
        .expect("Needs to be ListItem");
      let label = gtk::Label::new(None);
      list_item.set_child(Some(&label));
    });
    factory.connect_bind(move |_, list_item| {
      let list_item = list_item
        .downcast_ref::<gtk::ListItem>()
        .expect("Needs to be ListItem");
      let item = list_item
        .item()
        .unwrap()
        .downcast::<coin_db::CoinDatabase>()
        .unwrap();
      let label = list_item.child().unwrap().downcast::<gtk::Label>().unwrap();
      let value = item.property::<String>(&property);
      label.set_text(&value);
    });
    factory
  };

  for (property, title) in columns.iter() {
    let factory = if *property == "coin-index" {
      let factory = gtk::SignalListItemFactory::new();
      factory.connect_setup(move |_, list_item| {
        let list_item = list_item
          .downcast_ref::<gtk::ListItem>()
          .expect("Needs to be ListItem");
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
      });
      factory.connect_bind(move |_, list_item| {
        let list_item = list_item
          .downcast_ref::<gtk::ListItem>()
          .expect("Needs to be ListItem");
        let item = list_item
          .item()
          .unwrap()
          .downcast::<coin_db::CoinDatabase>()
          .unwrap();
        let label = list_item.child().unwrap().downcast::<gtk::Label>().unwrap();
        let value = item.property::<u32>("coin-index").to_string();
        label.set_text(&value);
      });
      factory
    } else {
      create_string_factory(property)
    };

    let column = gtk::ColumnViewColumn::new(Some(title), Some(factory));
    column.set_resizable(true);
    column_view.append_column(&column);
  }

  scrolled_window.set_child(Some(&column_view));
  coin_frame.set_child(Some(&scrolled_window));
  coin_main_content_box.append(&coin_frame);

  // let selection_model = column_view.model().unwrap();

  coin_single_selection.connect_selection_changed(move |coin_single_selection, _, _| {
    if let Some(selected_coin) = coin_single_selection.selected_item() {
      let coin = selected_coin
        .downcast::<coin_db::CoinDatabase>()
        .expect("The selected item is not a CoinDatabase");
      println!("Selected coin: {}", coin.property::<String>("coin-name"));
    }
  });

  // Generate master keys button
  let generate_master_keys_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  let generate_master_keys_button = gtk::Button::new();
  let delete_master_keys_button = gtk::Button::new();

  generate_master_keys_button.set_label(&t!("UI.main.coin.generate"));
  delete_master_keys_button.set_label(&t!("UI.main.coin.delete"));
  generate_master_keys_box.set_halign(gtk::Align::Center);
  generate_master_keys_box.append(&generate_master_keys_button);
  generate_master_keys_box.append(&delete_master_keys_button);
  coin_main_content_box.append(&generate_master_keys_box);

  // Master private keys entries
  let master_keys_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
  let master_xprv_frame = gtk::Frame::new(Some(&t!("UI.main.coin.keys.priv")));
  let master_private_key_text = gtk::TextView::new();
  master_private_key_text.set_editable(false);
  master_private_key_text.set_wrap_mode(gtk::WrapMode::Char);
  master_private_key_text.set_left_margin(5);
  master_private_key_text.set_top_margin(5);
  master_private_key_text.set_hexpand(true);

  let master_xprv_inner_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
  master_xprv_inner_box.append(&master_private_key_text);

  let copy_master_xprv_button = gtk::Button::new();
  copy_master_xprv_button.set_icon_name("edit-copy");
  copy_master_xprv_button.set_tooltip_text(Some(&t!("UI.button.copy")));
  master_xprv_inner_box.append(&copy_master_xprv_button);

  master_xprv_frame.set_child(Some(&master_xprv_inner_box));
  master_keys_box.append(&master_xprv_frame);

  copy_master_xprv_button.connect_clicked(clone!(
    #[weak]
    master_private_key_text,
    move |button| {
      let buffer = master_private_key_text.buffer();
      let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
      if !text.is_empty() {
        let display = button.display();
        let clipboard = display.clipboard();
        clipboard.set_text(&text);
      };
    }
  ));

  let master_xpub_frame = gtk::Frame::new(Some(&t!("UI.main.coin.keys.pub")));
  let master_public_key_text = gtk::TextView::new();
  master_public_key_text.set_editable(false);
  master_public_key_text.set_wrap_mode(gtk::WrapMode::Char);
  master_public_key_text.set_left_margin(5);
  master_public_key_text.set_top_margin(5);
  master_public_key_text.set_hexpand(true);

  let master_xpub_inner_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
  master_xpub_inner_box.append(&master_public_key_text);

  let copy_master_xpub_button = gtk::Button::new();
  copy_master_xpub_button.set_icon_name("edit-copy");
  copy_master_xpub_button.set_tooltip_text(Some(&t!("UI.button.copy")));
  master_xpub_inner_box.append(&copy_master_xpub_button);

  master_xpub_frame.set_child(Some(&master_xpub_inner_box));
  master_keys_box.append(&master_xpub_frame);

  copy_master_xpub_button.connect_clicked(clone!(
    #[weak]
    master_public_key_text,
    move |button| {
      let buffer = master_public_key_text.buffer();
      let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
      if !text.is_empty() {
        let display = button.display();
        let clipboard = display.clipboard();
        clipboard.set_text(&text);
      };
    }
  ));

  coin_main_content_box.append(&master_keys_box);
  stack.add_titled(&coin_main_box, Some("sidebar-coin"), &t!("UI.main.coin"));

  // JUMP: Sidebar 3: Address
  let main_address_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
  main_address_box.set_hexpand(true);
  main_address_box.set_vexpand(true);
  main_address_box.set_margin_top(10);
  main_address_box.set_margin_start(10);
  main_address_box.set_margin_end(10);
  main_address_box.set_margin_bottom(10);

  // Derivation options
  let derivation_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let bip_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  let coin_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  let address_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  let purpose_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  let main_bip_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation.bip")));
  let main_coin_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation.coin")));
  let main_address_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation.address")));
  let main_purpose_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation.purpose")));

  main_bip_frame.set_hexpand(true);
  main_coin_frame.set_hexpand(true);
  main_address_frame.set_hexpand(true);
  main_purpose_frame.set_hexpand(true);

  let bip_hardened_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation.hard")));
  let coin_hardened_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation.hard")));
  let address_hardened_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation.hard")));

  let valid_bip_as_string: Vec<String> = VALID_BIP_DERIVATIONS
    .iter()
    .map(|&x| x.to_string())
    .collect();
  let valid_bip_as_ref: Vec<&str> = valid_bip_as_string.iter().map(|s| s.as_ref()).collect();
  let bip_dropdown = gtk::DropDown::from_strings(&valid_bip_as_ref);

  let wallet_bip = lock_app_settings.wallet_bip.unwrap().to_string();
  let default_index = VALID_BIP_DERIVATIONS
    .iter()
    .position(|&x| x == wallet_bip)
    .unwrap_or_else(|| {
      eprintln!("\t- {}", &t!("error.bip.value", value = wallet_bip));
      1
    });

  bip_dropdown.set_selected(default_index.try_into().unwrap());
  bip_dropdown.set_hexpand(true);

  let bip_hardened_checkbox = gtk::CheckButton::new();
  bip_hardened_checkbox.set_active(true);
  bip_hardened_checkbox.set_halign(gtk::Align::Center);

  let coin_entry = gtk::Entry::new();
  coin_entry.set_editable(false);
  coin_entry.set_hexpand(true);

  let coin_hardened_checkbox = gtk::CheckButton::new();
  coin_hardened_checkbox.set_active(true);
  coin_hardened_checkbox.set_halign(gtk::Align::Center);

  let adjustment = gtk::Adjustment::new(0.0, 0.0, WALLET_MAX_ADDRESSES as f64, 1.0, 100.0, 0.0);

  let address_spinbutton = gtk::SpinButton::new(Some(&adjustment), 1.0, 0);
  address_spinbutton.set_hexpand(true);

  let address_hardened_checkbox = gtk::CheckButton::new();
  address_hardened_checkbox.set_active(true);
  address_hardened_checkbox.set_halign(gtk::Align::Center);

  let valid_wallet_purpose_as_strings: Vec<String> = VALID_WALLET_PURPOSE
    .iter()
    .map(|&x| x.to_string())
    .collect();
  let valid_wallet_purpose_as_ref: Vec<&str> = valid_wallet_purpose_as_strings
    .iter()
    .map(|s| s.as_ref())
    .collect();
  let purpose_dropdown = gtk::DropDown::from_strings(&valid_wallet_purpose_as_ref);
  purpose_dropdown.set_selected(0);
  purpose_dropdown.set_hexpand(true);

  bip_hardened_frame.set_child(Some(&bip_hardened_checkbox));
  coin_hardened_frame.set_child(Some(&coin_hardened_checkbox));
  address_hardened_frame.set_child(Some(&address_hardened_checkbox));

  // Derivation label
  let derivation_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let derivation_label_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation")));
  derivation_label_frame.set_hexpand(true);

  let default_bip_label = if wallet_bip == "32" {
    main_purpose_frame.set_visible(false);
    format!("m/{}'/0'/0'", wallet_bip)
  } else {
    main_purpose_frame.set_visible(true);
    format!("m/{}'/0'/0'/0", wallet_bip)
  };

  // let derivation_label_text = gtk4::Label::builder()
  //   .label(&default_bip_label)
  //   .halign(gtk::Align::Center)
  //   .valign(gtk::Align::Center)
  //   .css_classes(["large-title"])
  //   .build();

  let derivation_label_text = gtk::TextView::new();
  derivation_label_text.set_editable(false);
  derivation_label_text.set_wrap_mode(gtk::WrapMode::Char);
  derivation_label_text.set_justification(gtk::Justification::Center);
  derivation_label_text.set_hexpand(true);
  derivation_label_text.set_css_classes(&["h1"]);
  derivation_label_text.buffer().set_text(&default_bip_label);

  let address_generation_buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  address_generation_buttons_box.set_halign(gtk::Align::Center);

  let generate_addresses_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let generate_addresses_button = gtk::Button::with_label(&t!("UI.main.address.generate"));
  generate_addresses_button_box.append(&generate_addresses_button);

  let delete_addresses_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let delete_addresses_button = gtk::Button::with_label(&t!("UI.main.address.generate.clean"));
  delete_addresses_button_box.append(&delete_addresses_button);
  delete_addresses_button_box.set_visible(false);

  let generator_handler = std::sync::Arc::new(std::sync::Mutex::new(
    None::<(
      tokio::task::JoinHandle<()>,
      tokio::sync::watch::Sender<bool>,
      tokio::sync::watch::Sender<bool>,
    )>,
  ));
  let stop_addresses_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let stop_address_generation_button =
    gtk::Button::with_label(&t!("UI.main.address.generate.stop"));
  stop_addresses_button_box.append(&stop_address_generation_button);
  stop_addresses_button_box.set_visible(false);

  address_generation_buttons_box.append(&generate_addresses_button_box);
  address_generation_buttons_box.append(&delete_addresses_button_box);
  address_generation_buttons_box.append(&stop_addresses_button_box);

  // Address tree
  let address_scrolled_window = gtk::ScrolledWindow::new();
  let address_treeview_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let address_treeview_frame = gtk::Frame::new(Some(&t!("UI.main.address")));
  address_treeview_frame.set_hexpand(true);
  address_treeview_frame.set_vexpand(true);

  let address_store = gio::ListStore::new::<AddressDatabase>();

  let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
    let entry1 = obj1.downcast_ref::<AddressDatabase>().unwrap();
    let entry2 = obj2.downcast_ref::<AddressDatabase>().unwrap();

    let id1 = entry1.property::<String>("id");
    let id2 = entry2.property::<String>("id");
    let coin1 = entry1.property::<String>("coin");
    let coin2 = entry2.property::<String>("coin");

    if id1 != id2 {
      id1.cmp(&id2).into()
    } else {
      coin1.cmp(&coin2).into()
    }
  });

  let address_sorted_model = gtk::SortListModel::new(Some(address_store.clone()), Some(sorter));
  let address_selection_model = gtk::SingleSelection::new(Some(address_sorted_model));
  let address_treeview = gtk::ColumnView::new(Some(address_selection_model.clone()));

  address_treeview.set_show_column_separators(true);
  address_treeview.set_show_row_separators(true);

  let columns = [
    &t!("UI.main.address.table.id"),
    &t!("UI.main.address.table.coin"),
    &t!("UI.main.address.table.path"),
    &t!("UI.main.address.table.address"),
    &t!("UI.main.address.table.pub"),
    &t!("UI.main.address.table.priv"),
  ];

  for (i, column_title) in columns.iter().enumerate() {
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_factory, list_item| {
      let list_item = list_item
        .downcast_ref::<gtk::ListItem>()
        .expect("Needs to be ListItem");
      let label = gtk::Label::new(None);
      list_item.set_child(Some(&label));
    });

    factory.connect_bind(move |_factory, list_item| {
      let list_item = list_item
        .downcast_ref::<gtk::ListItem>()
        .expect("Needs to be ListItem");
      let label = list_item.child().unwrap().downcast::<gtk::Label>().unwrap();
      let entry = list_item
        .item()
        .unwrap()
        .downcast::<AddressDatabase>()
        .unwrap();

      let text = match i {
        0 => entry.property::<String>("id"),
        1 => entry.property::<String>("coin"),
        2 => entry.property::<String>("path"),
        3 => entry.property::<String>("address"),
        4 => entry.property::<String>("public-key"),
        5 => entry.property::<String>("private-key"),
        _ => unreachable!(),
      };
      label.set_text(&text);
    });

    let column = gtk::ColumnViewColumn::new(Some(column_title), Some(factory));
    column.set_expand(true);

    #[cfg(not(feature = "dev"))]
    {
      if i == 0 {
        column.set_visible(false);
      }
    }

    address_treeview.append_column(&column);
  }

  // Address options main box
  let address_options_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let address_options_content = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  address_options_box.append(&address_options_content);

  // Address count
  let address_options_frame = gtk::Frame::new(Some(&t!("UI.main.address.options.count")));
  let address_options_address_count_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let wallet_address_count = lock_app_settings.wallet_address_count.unwrap();
  let address_options_adjustment = gtk::Adjustment::new(
    wallet_address_count as f64,
    1.0,
    WALLET_MAX_ADDRESSES as f64,
    1.0,
    10.0,
    0.0,
  );
  let address_count_spinbutton = gtk::SpinButton::new(Some(&address_options_adjustment), 1.0, 0);

  address_options_frame.set_child(Some(&address_options_address_count_box));
  address_options_address_count_box.append(&address_count_spinbutton);

  // Address start
  let address_start_frame = gtk::Frame::new(Some(&t!("UI.main.address.options.start")));
  let address_start_address_count_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let address_start_adjustment = gtk::Adjustment::new(
    0.0,
    0.0,
    WALLET_MAX_ADDRESSES as f64 - wallet_address_count as f64,
    1.0,
    10.0,
    0.0,
  );
  let address_start_spinbutton = gtk::SpinButton::new(Some(&address_start_adjustment), 1.0, 0);

  address_start_frame.set_child(Some(&address_start_address_count_box));
  address_start_address_count_box.append(&address_start_spinbutton);

  // Hardened address
  let address_options_hardened_address_frame =
    gtk::Frame::new(Some(&t!("UI.main.address.options.hardened")));
  let address_options_hardened_address_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let address_options_hardened_address_checkbox = gtk::CheckButton::new();
  let wallet_hardened_address = lock_app_settings.wallet_hardened_address;

  address_options_hardened_address_checkbox.set_active(wallet_hardened_address.unwrap());
  address_options_hardened_address_box.set_halign(gtk4::Align::Center);
  address_options_hardened_address_frame.set_child(Some(&address_options_hardened_address_box));
  address_options_hardened_address_box.append(&address_options_hardened_address_checkbox);

  // Address count
  let address_total_generated_count_frame =
    gtk::Frame::new(Some(&t!("UI.main.address.speed.count")));
  let address_total_generated_count_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let address_total_generated_count_label = gtk::Label::new(Some("0"));

  address_total_generated_count_box.set_halign(gtk4::Align::Center);
  address_total_generated_count_frame.set_child(Some(&address_total_generated_count_box));
  address_total_generated_count_box.append(&address_total_generated_count_label);

  // Address speed
  let items_added_in_last_second = std::sync::Arc::new(std::sync::Mutex::new(0u64));
  let counts = Rc::new(RefCell::new(vec![0u64; 10]));
  let index = Rc::new(RefCell::new(0usize));
  let max_speed = Rc::new(RefCell::new(0u64));
  let ema_speed = Rc::new(RefCell::new(0.0));
  let address_generation_speed_frame = gtk::Frame::new(Some(&t!("UI.main.address.speed")));
  let address_generation_speed_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let address_generation_speed_label = gtk::Label::new(Some("0/sec"));

  address_generation_speed_frame.set_visible(false);
  address_generation_speed_box.set_halign(gtk4::Align::Center);
  address_generation_speed_frame.set_child(Some(&address_generation_speed_box));
  address_generation_speed_box.append(&address_generation_speed_label);

  address_store.connect_items_changed(clone!(
    #[strong]
    address_store,
    move |_store, _position, _removed, _added| {
      let count = address_store.n_items();
      address_total_generated_count_label.set_label(&count.to_string());
    }
  ));

  // Address progress box
  let address_generation_progress_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let address_generation_progress_bar = gtk::ProgressBar::new();
  address_generation_progress_bar.set_hexpand(true);
  address_generation_progress_box.append(&address_generation_progress_bar);

  bip_box.append(&bip_dropdown);
  bip_box.append(&bip_hardened_frame);
  coin_box.append(&coin_entry);
  coin_box.append(&coin_hardened_frame);
  address_box.append(&address_spinbutton);
  address_box.append(&address_hardened_frame);
  purpose_box.append(&purpose_dropdown);
  main_bip_frame.set_child(Some(&bip_box));
  main_coin_frame.set_child(Some(&coin_box));
  main_address_frame.set_child(Some(&address_box));
  main_purpose_frame.set_child(Some(&purpose_box));
  derivation_box.append(&main_bip_frame);
  derivation_box.append(&main_coin_frame);
  derivation_box.append(&main_address_frame);
  derivation_box.append(&main_purpose_frame);
  derivation_label_box.append(&derivation_label_frame);
  derivation_label_frame.set_child(Some(&derivation_label_text));
  address_treeview_box.append(&address_treeview_frame);
  address_treeview_frame.set_child(Some(&address_scrolled_window));
  address_scrolled_window.set_child(Some(&address_treeview));
  address_options_content.append(&address_options_frame);
  address_options_content.append(&address_start_frame);
  address_options_content.append(&address_options_hardened_address_frame);
  address_options_content.append(&address_total_generated_count_frame);
  address_options_content.append(&address_generation_speed_frame);
  main_address_box.append(&derivation_box);
  main_address_box.append(&derivation_label_box);
  main_address_box.append(&address_generation_buttons_box);
  main_address_box.append(&address_treeview_box);
  main_address_box.append(&address_options_box);
  main_address_box.append(&address_generation_progress_box);

  stack.add_titled(
    &main_address_box,
    Some("sidebar-address"),
    &t!("UI.main.address"),
  );

  // JUMP: Action: Open Wallet
  buttons["open"].connect_clicked(clone!(
    #[strong]
    app_messages_state,
    #[weak]
    entropy_text,
    #[weak]
    mnemonic_passphrase_text,
    #[weak]
    mnemonic_words_text,
    #[weak]
    seed_text,
    move |_| {
      let (entropy, passphrase) = open_wallet_from_file(&app_messages_state);

      if !entropy.is_empty() {
        #[cfg(debug_assertions)]
        println!("\t-Wallet entropy: {:?}", entropy);

        entropy_text.buffer().set_text(&entropy);

        match passphrase {
          Some(pass) => {
            #[cfg(debug_assertions)]
            println!("\t- Mnemonic passphrase: {:?}", pass);

            mnemonic_passphrase_text.buffer().set_text(&pass);
          }
          None => {
            #[cfg(debug_assertions)]
            println!("\t- No Mnemonic passphrase available");
          }
        }

        let buffer = entropy_text.buffer();
        let start_iter = buffer.start_iter();
        let end_iter = buffer.end_iter();
        let full_entropy = buffer.text(&start_iter, &end_iter, false);

        if !full_entropy.is_empty() {
          let mnemonic_dictionary = {
            let lock_app_settings = APP_SETTINGS.read().unwrap();
            lock_app_settings
              .wallet_mnemonic_dictionary
              .clone()
              .unwrap()
          };

          let mnemonic_words =
            match keys::generate_mnemonic_words(&full_entropy, Some(&mnemonic_dictionary)) {
              Ok(mnemonic) => {
                d3bug("<<< generate_mnemonic_words", "debug");
                mnemonic
              }
              Err(e) => {
                d3bug(&format!("generate_mnemonic_words: {:?}", e), "error");
                return;
              }
            };

          mnemonic_words_text.buffer().set_text(&mnemonic_words);

          let seed = match keys::generate_seed_from_mnemonic(
            &mnemonic_words,
            &mnemonic_passphrase_text.buffer().text(),
          ) {
            Ok(seed) => {
              d3bug("<<< generate_seed_from_mnemonic", "debug");
              seed
            }
            Err(e) => {
              d3bug(&format!("generate_seed_from_mnemonic: {:?}", e), "error");
              return;
            }
          };

          let seed_hex = hex::encode(&seed[..]);
          seed_text.buffer().set_text(&seed_hex.to_string());

          #[cfg(debug_assertions)]
          println!("\t- Seed (hex): {:?}", seed_hex);
        }
      }
    }
  ));

  // JUMP: Action: Generate Seed button
  generate_seed_button.connect_clicked(clone!(
    #[strong]
    app_messages_state,
    #[weak]
    entropy_source_dropdown,
    #[weak]
    entropy_text,
    #[weak]
    entropy_length_dropdown,
    #[weak]
    mnemonic_words_text,
    #[weak]
    mnemonic_passphrase_text,
    #[weak]
    master_private_key_text,
    #[weak]
    master_public_key_text,
    #[weak]
    mnemonic_dictionary_dropdown,
    #[weak]
    seed_text,
    #[weak(rename_to = random_mnemonic_passphrase_button)]
    buttons["random"],
    move |_| {
      entropy_text.buffer().set_text("");
      mnemonic_words_text.buffer().set_text("");
      seed_text.buffer().set_text("");
      mnemonic_passphrase_text.buffer().set_text("");

      random_mnemonic_passphrase_button.emit_by_name::<()>("clicked", &[]);

      let selected_entropy_source_index = entropy_source_dropdown.selected() as usize;
      let selected_entropy_length_index = entropy_length_dropdown.selected() as usize;
      let selected_entropy_source_value = VALID_ENTROPY_SOURCES.get(selected_entropy_source_index);
      let selected_entropy_length_value = VALID_ENTROPY_LENGTHS.get(selected_entropy_length_index);
      let source = selected_entropy_source_value.unwrap().to_string();
      let entropy_length = selected_entropy_length_value.unwrap();

      // let pre_entropy = keys::generate_entropy(&source, *entropy_length as u64);

      let pre_entropy =
        keys::generate_entropy(&source, *entropy_length as u64).unwrap_or_else(|err| {
          d3bug(&format!("generate_entropy: {:?}", err), "error");
          String::new()
        });

      if !pre_entropy.is_empty() {
        let checksum = qr2m_lib::calculate_checksum_for_entropy(&pre_entropy);
        let full_entropy = format!("{}{}", &pre_entropy, &checksum);

        // let lock_app_settings = APP_SETTINGS.read().unwrap();
        // let mnemonic_dictionary = lock_app_settings
        //     .wallet_mnemonic_dictionary
        //     .clone()
        //     .unwrap();

        let value = mnemonic_dictionary_dropdown.selected() as usize;
        let selected_dictionary = VALID_MNEMONIC_DICTIONARY.get(value);
        let mnemonic_dictionary = selected_dictionary.unwrap();

        let mnemonic_words =
          match keys::generate_mnemonic_words(&full_entropy, Some(mnemonic_dictionary)) {
            Ok(mnemonic) => {
              d3bug("<<< generate_mnemonic_words", "debug");
              mnemonic
            }
            Err(e) => {
              d3bug(&format!("generate_mnemonic_words: {:?}", e), "error");
              return;
            }
          };
        let passphrase_text = mnemonic_passphrase_text.text().to_string();
        let seed = match keys::generate_seed_from_mnemonic(&mnemonic_words, &passphrase_text) {
          Ok(seed) => {
            d3bug("<<< generate_seed_from_mnemonic", "debug");
            seed
          }
          Err(e) => {
            d3bug(&format!("generate_seed_from_mnemonic: {:?}", e), "error");
            return;
          }
        };
        let seed_hex = hex::encode(&seed[..]);

        entropy_text.buffer().set_text(&full_entropy);
        mnemonic_words_text.buffer().set_text(&mnemonic_words);
        seed_text.buffer().set_text(&seed_hex.to_string());

        #[cfg(debug_assertions)]
        {
          println!("\t- Entropy checksum: {:?}", checksum);
          println!("\t- Final entropy: {:?}", full_entropy);
          println!("\t- Seed (hex): {:?}", seed_hex);
        }

        {
          let mut wallet_settings = WALLET_SETTINGS.lock().unwrap();
          wallet_settings.entropy_checksum = Some(checksum.clone());
          wallet_settings.entropy_string = Some(full_entropy.clone());
          wallet_settings.mnemonic_passphrase = Some(passphrase_text.clone());
          wallet_settings.mnemonic_words = Some(mnemonic_words.clone());
          wallet_settings.seed = Some(seed_hex.clone());
        }

        master_private_key_text.buffer().set_text("");
        master_public_key_text.buffer().set_text("");
      } else {
        #[cfg(debug_assertions)]
        eprintln!("\t- {}", &t!("error.entropy.empty"));

        let lock_app_messages = app_messages_state.borrow();
        match lock_app_messages.queue_message(
          t!("error.entropy.empty").to_string(),
          gtk::MessageType::Warning,
        ) {
          Ok(_) => {}
          Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
        };
      }
    }
  ));

  delete_seed_button.connect_clicked(clone!(
    #[strong]
    address_store,
    #[weak]
    entropy_text,
    #[weak]
    mnemonic_words_text,
    #[weak]
    mnemonic_passphrase_text,
    #[weak]
    seed_text,
    #[weak]
    master_private_key_text,
    #[weak]
    master_public_key_text,
    move |_| {
      entropy_text.buffer().set_text("");
      mnemonic_passphrase_text.buffer().set_text("");
      mnemonic_words_text.buffer().set_text("");
      seed_text.buffer().set_text("");
      master_private_key_text.buffer().set_text("");
      master_public_key_text.buffer().set_text("");
      address_store.remove_all();
    }
  ));

  buttons["random"].connect_clicked(clone!(
    #[weak]
    mnemonic_passphrase_text,
    #[weak]
    mnemonic_passphrase_scale,
    move |_| {
      let scale_value = mnemonic_passphrase_scale.value() as u32;

      let mnemonic_rng_string: String = (0..scale_value)
        .map(|_| char::from(rand::rng().random_range(32..127)))
        .collect();

      #[cfg(debug_assertions)]
      println!("\t- RNG Mnemonic Passphrase: {:?}", mnemonic_rng_string);

      mnemonic_passphrase_text.set_text(&mnemonic_rng_string);
    }
  ));

  delete_master_keys_button.connect_clicked(clone!(
    #[weak]
    master_private_key_text,
    #[weak]
    master_public_key_text,
    move |_| {
      master_private_key_text.buffer().set_text("");
      master_public_key_text.buffer().set_text("");

      let mut wallet_settings = WALLET_SETTINGS.lock().unwrap();
      wallet_settings.master_chain_code_bytes = None;
      wallet_settings.master_private_key_bytes = None;
      wallet_settings.master_public_key_bytes = None;
    }
  ));

  // JUMP: Action: Generate Master Keys button
  generate_master_keys_button.connect_clicked(clone!(
    #[strong]
    coin_entry,
    #[strong]
    app_messages_state,
    #[strong]
    coin_selection_model,
    #[weak]
    seed_text,
    #[weak]
    master_private_key_text,
    #[weak]
    master_public_key_text,
    move |_| {
      let buffer = seed_text.buffer();
      let start_iter = buffer.start_iter();
      let end_iter = buffer.end_iter();
      let text = buffer.text(&start_iter, &end_iter, false);

      let single_selection = coin_selection_model
        .clone()
        .downcast::<gtk::SingleSelection>()
        .expect("The selection model is not a SingleSelection");

      if !text.is_empty() {
        if let Some(model) = single_selection.selected_item() {
          let _status = model.property::<String>("status");
          let coin_index = model.property::<u32>("coin-index");
          let _coin_symbol = model.property::<String>("coin-symbol");
          let coin_name = model.property::<String>("coin-name");
          let key_derivation = model.property::<String>("key-derivation");
          let hash = model.property::<String>("hash");
          let private_header = model.property::<String>("private-header");
          let public_header = model.property::<String>("public-header");
          let public_key_hash = model.property::<String>("public-key-hash");
          let _script_hash = model.property::<String>("script-hash");
          let wallet_import_format = model.property::<String>("wallet-import-format");
          let _evm = model.property::<String>("evm");
          let _ucid = model.property::<String>("ucid");
          let _cmc_top = model.property::<String>("cmc-top");

          master_private_key_text.buffer().set_text("");
          master_public_key_text.buffer().set_text("");

          #[cfg(debug_assertions)]
          {
            println!("\n#### Coin info ####");
            println!("\t- status: {}", _status);
            println!("\t- index: {}", coin_index);
            println!("\t- coin_symbol: {}", _coin_symbol);
            println!("\t- coin_name: {}", coin_name);
            println!("\t- key_derivation: {}", key_derivation);
            println!("\t- hash: {}", hash);
            println!("\t- private_header: {}", private_header);
            println!("\t- public_header: {}", public_header);
            println!("\t- public_key_hash: {}", public_key_hash);
            println!("\t- script_hash: {}", _script_hash);
            println!("\t- wallet_import_format: {}", wallet_import_format);
            println!("\t- EVM: {}", _evm);
            println!("\t- UCID: {}", _ucid);
            println!("\t- cmc_top: {}", _cmc_top);
          }

          let buffer = seed_text.buffer();
          let start_iter = buffer.start_iter();
          let end_iter = buffer.end_iter();
          let seed_string = buffer.text(&start_iter, &end_iter, true);

          if key_derivation == "secp256k1" {
            match keys::generate_master_keys_secp256k1(
              &seed_string,
              &private_header,
              &public_header,
            ) {
              Ok(_) => {
                d3bug("<<< generate_master_keys_secp256k1", "debug");
              }
              Err(err) => d3bug(
                &format!("generate_master_keys_secp256k1: \n{:?}", err),
                "error",
              ),
            };
          } else {
            #[cfg(feature = "dev")]
            dev::generate_master_keys_ed25519(&seed_string);
          };

          coin_entry.set_text(&coin_index.to_string());

          let mut wallet_settings = WALLET_SETTINGS.lock().unwrap();

          let master_private_key = wallet_settings
            .master_private_key
            .clone()
            .unwrap_or_default();

          let master_public_key = wallet_settings
            .master_public_key
            .clone()
            .unwrap_or_default();

          master_private_key_text
            .buffer()
            .set_text(&master_private_key);
          master_public_key_text.buffer().set_text(&master_public_key);

          wallet_settings.public_key_hash = Some(public_key_hash.clone());
          wallet_settings.wallet_import_format = Some(wallet_import_format.to_string());
          wallet_settings.key_derivation = Some(key_derivation.to_string());
          wallet_settings.hash = Some(hash.to_string());
          wallet_settings.coin_index = Some(coin_index);
          wallet_settings.coin_name = Some(coin_name.parse().unwrap());
        }
      } else {
        let app_messages_state = app_messages_state.borrow();
        match app_messages_state.queue_message(
          t!("error.entropy.seed").to_string(),
          gtk::MessageType::Warning,
        ) {
          Ok(_) => {}
          Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
        };
      }
    }
  ));

  entropy_source_dropdown.connect_selected_notify(clone!(
    #[weak]
    generate_seed_button,
    // #[weak] random_mnemonic_passphrase_button,
    move |entropy_source_dropdown| {
      let value = entropy_source_dropdown.selected() as usize;
      let selected_entropy_source_value = VALID_ENTROPY_SOURCES.get(value);
      let source = selected_entropy_source_value.unwrap();

      // if *source == "RNG+" {
      //     mnemonic_passphrase_length_box.set_visible(true);
      //     random_mnemonic_passphrase_button.set_visible(true);
      // } else {
      //     mnemonic_passphrase_length_box.set_visible(false);
      //     random_mnemonic_passphrase_button.set_visible(false);
      // }

      if *source == "File" {
        generate_seed_button.set_label(&t!("UI.main.seed.generate.file"));
      } else {
        generate_seed_button.set_label(&t!("UI.main.seed.generate"));
      }
    }
  ));

  mnemonic_dictionary_dropdown.connect_selected_notify(clone!(
    #[weak]
    entropy_text,
    #[weak]
    mnemonic_passphrase_text,
    #[weak]
    mnemonic_words_text,
    #[weak]
    seed_text,
    // #[weak] random_mnemonic_passphrase_button,
    move |dropdown| {
      let selected: usize = dropdown.selected().try_into().unwrap_or(0);
      let selected_dictionary = VALID_MNEMONIC_DICTIONARY[selected];

      let entropy_buffer = entropy_text.buffer();
      let start_iter = entropy_buffer.start_iter();
      let end_iter = entropy_buffer.end_iter();
      let entropy_text = entropy_buffer.text(&start_iter, &end_iter, false);

      if !entropy_text.is_empty() {
        let mnemonic_words =
          match keys::generate_mnemonic_words(&entropy_text, Some(selected_dictionary)) {
            Ok(mnemonic) => {
              d3bug("<<< generate_mnemonic_words", "debug");
              mnemonic
            }
            Err(e) => {
              d3bug(&format!("generate_mnemonic_words: {:?}", e), "error");
              return;
            }
          };
        mnemonic_words_text.buffer().set_text(&mnemonic_words);

        let seed = match keys::generate_seed_from_mnemonic(
          &mnemonic_words,
          &mnemonic_passphrase_text.buffer().text(),
        ) {
          Ok(seed) => {
            d3bug("<<< generate_seed_from_mnemonic", "debug");
            seed
          }
          Err(e) => {
            d3bug(&format!("generate_seed_from_mnemonic: {:?}", e), "error");
            return;
          }
        };

        let seed_hex = hex::encode(&seed[..]);
        seed_text.buffer().set_text(&seed_hex.to_string());
      }
    }
  ));

  advance_search_checkbox.connect_active_notify(clone!(
    // #[weak] advance_search_checkbox,
    move |checkbox| {
      if checkbox.is_active() {
        advance_search_filter_box.set_visible(true);
      } else {
        advance_search_filter_box.set_visible(false);
      }
    }
  ));

  coin_search_filter_dropdown.connect_selected_notify(clone!(
    #[weak]
    coin_search,
    move |dropdown| {
      let selected: usize = dropdown.selected().try_into().unwrap_or(0);
      coin_search.set_placeholder_text(Some(&t!(
        "UI.main.coin.search.text",
        value = VALID_COIN_SEARCH_PARAMETER[selected]
      )));
      coin_search.set_text("");
    }
  ));

  mnemonic_passphrase_text.connect_changed(clone!(
    #[weak]
    entropy_text,
    #[weak]
    mnemonic_words_text,
    #[weak]
    seed_text,
    move |mnemonic_passphrase_text| {
      let entropy_buffer = entropy_text.buffer();
      let start_iter = entropy_buffer.start_iter();
      let end_iter = entropy_buffer.end_iter();
      let entropy_text = entropy_buffer.text(&start_iter, &end_iter, false);

      if !entropy_text.is_empty() {
        // let entropy_length = entropy_text.len();
        // let cut_entropy = entropy_length / 32;
        // let new_pre_entropy = entropy_text[0..entropy_length - cut_entropy].to_string();

        let final_entropy = entropy_text.clone().to_string();
        let mnemonic_words_buffer = mnemonic_words_text.buffer();
        let start_iter = mnemonic_words_buffer.start_iter();
        let end_iter = mnemonic_words_buffer.end_iter();
        let final_mnemonic_words = mnemonic_words_buffer
          .text(&start_iter, &end_iter, false)
          .to_string();
        let final_mnemonic_passphrase = mnemonic_passphrase_text.buffer().text().to_string();

        let seed = match keys::generate_seed_from_mnemonic(
          &final_mnemonic_words,
          &mnemonic_passphrase_text.buffer().text(),
        ) {
          Ok(seed) => {
            d3bug("<<< generate_seed_from_mnemonic", "debug");
            seed
          }
          Err(e) => {
            d3bug(&format!("generate_seed_from_mnemonic: {:?}", e), "error");
            return;
          }
        };
        let seed_hex = hex::encode(&seed[..]);
        seed_text.buffer().set_text(&seed_hex.to_string());

        let mut wallet_settings = WALLET_SETTINGS.lock().unwrap();
        wallet_settings.entropy_string = Some(final_entropy);
        wallet_settings.mnemonic_words = Some(final_mnemonic_words);
        wallet_settings.mnemonic_passphrase = Some(final_mnemonic_passphrase);
        wallet_settings.seed = Some(seed_hex.clone());
      }
    }
  ));

  mnemonic_passphrase_scale.connect_value_changed(clone!(
    #[weak]
    mnemonic_passphrase_length_info,
    #[weak(rename_to = random_mnemonic_passphrase_button)]
    buttons["random"],
    move |mnemonic_passphrase_scale| {
      let scale_value = mnemonic_passphrase_scale.value() as u32;
      mnemonic_passphrase_length_info.set_text(&scale_value.to_string());
      random_mnemonic_passphrase_button.emit_by_name::<()>("clicked", &[]);
    }
  ));

  coin_search.connect_search_changed({
    let filter_model = filter_model.clone();
    move |coin_search| {
      let search_text = coin_search.text().to_lowercase();
      let selected = coin_search_filter_dropdown.selected() as usize;
      let selected_search_parameter = VALID_COIN_SEARCH_PARAMETER.get(selected).unwrap_or(&"Name");
      let min_search_length = if selected_search_parameter == &"Index" {
        1
      } else {
        2
      };

      if search_text.len() >= min_search_length {
        let filter = gtk::CustomFilter::new(move |obj| {
          let coin = obj.downcast_ref::<coin_db::CoinDatabase>().unwrap();
          match *selected_search_parameter {
            "Name" => coin
              .property::<String>("coin-name")
              .to_lowercase()
              .contains(&search_text),
            "Index" => coin
              .property::<u32>("coin-index")
              .to_string()
              .contains(&search_text),
            "Status" => coin
              .property::<String>("status")
              .to_lowercase()
              .contains(&search_text),
            "Symbol" => coin
              .property::<String>("coin-symbol")
              .to_lowercase()
              .contains(&search_text),
            _ => false,
          }
        });
        filter_model.set_filter(Some(&filter));
        // } else {
        // filter_model.set_filter(None);
      }
    }
  });

  filter_top10_coins_button.connect_clicked({
    let filter_model = filter_model.clone();
    move |_| {
      let filter = gtk::CustomFilter::new(|obj| {
        let coin = obj.downcast_ref::<coin_db::CoinDatabase>().unwrap();
        coin.property::<String>("cmc-top") == "10"
      });
      filter_model.set_filter(Some(&filter));
    }
  });

  filter_top100_coins_button.connect_clicked({
    let filter_model = filter_model.clone();
    move |_| {
      let filter = gtk::CustomFilter::new(|obj| {
        let coin = obj.downcast_ref::<coin_db::CoinDatabase>().unwrap();
        coin.property::<String>("cmc-top") == "100" || coin.property::<String>("cmc-top") == "10"
      });
      filter_model.set_filter(Some(&filter));
    }
  });

  filter_verified_coins_button.connect_clicked({
    let filter_model = filter_model.clone();
    move |_| {
      let filter = gtk::CustomFilter::new(|obj| {
        let coin = obj.downcast_ref::<coin_db::CoinDatabase>().unwrap();
        coin.property::<String>("status") == coin_db::VALID_COIN_STATUS_NAME[1]
      });
      filter_model.set_filter(Some(&filter));
    }
  });

  filter_not_verified_coins_button.connect_clicked({
    let filter_model = filter_model.clone();
    move |_| {
      let filter = gtk::CustomFilter::new(|obj| {
        let coin = obj.downcast_ref::<coin_db::CoinDatabase>().unwrap();
        coin.property::<String>("status") == coin_db::VALID_COIN_STATUS_NAME[2]
      });
      filter_model.set_filter(Some(&filter));
    }
  });

  filter_not_supported_coins_button.connect_clicked({
    let filter_model = filter_model.clone();
    move |_| {
      let filter = gtk::CustomFilter::new(|obj| {
        let coin = obj.downcast_ref::<coin_db::CoinDatabase>().unwrap();
        coin.property::<String>("status") == coin_db::VALID_COIN_STATUS_NAME[0]
      });
      filter_model.set_filter(Some(&filter));
    }
  });

  // JUMP: BIP switching
  bip_dropdown.connect_selected_notify(clone!(
    #[weak]
    derivation_label_text,
    #[weak]
    bip_hardened_checkbox,
    #[weak]
    coin_hardened_checkbox,
    #[weak]
    address_hardened_checkbox,
    #[weak]
    purpose_dropdown,
    #[weak]
    address_spinbutton,
    move |bip_dropdown| {
      let value = bip_dropdown.selected() as usize;
      let selected_bip_value = VALID_BIP_DERIVATIONS.get(value);
      let bip = selected_bip_value.unwrap();
      let bip_number;
      let mut dp = DERIVATION_PATH.write().unwrap();

      if *bip == "Custom" {
        println!("custom");
        derivation_label_text.set_editable(true);
        bip_hardened_checkbox.set_can_target(false);
        coin_hardened_checkbox.set_can_target(false);
        address_hardened_checkbox.set_can_target(false);
        purpose_dropdown.set_can_target(false);
        address_spinbutton.set_can_target(false);

        // TODO: Process custom path, then update DP
        // dp.update_field("bip", Some(FieldValue::U32(bip_number)));
        // update_derivation_label(*dp, derivation_label_text)
      } else if *bip == "32" {
        derivation_label_text.set_editable(false);
        main_purpose_frame.set_visible(false);
        bip_hardened_frame.set_visible(false);

        bip_hardened_checkbox.set_can_target(true);
        coin_hardened_checkbox.set_can_target(true);
        address_hardened_checkbox.set_can_target(true);
        purpose_dropdown.set_can_target(true);
        address_spinbutton.set_can_target(true);

        bip_number = 32;
        dp.update_field("bip", Some(FieldValue::U32(bip_number)));
        update_derivation_label(*dp, derivation_label_text)
      } else {
        derivation_label_text.set_editable(false);
        main_purpose_frame.set_visible(true);
        bip_hardened_frame.set_visible(true);

        bip_hardened_checkbox.set_can_target(true);
        coin_hardened_checkbox.set_can_target(true);
        address_hardened_checkbox.set_can_target(true);
        purpose_dropdown.set_can_target(true);
        address_spinbutton.set_can_target(true);

        bip_number = bip.parse().unwrap();
        dp.update_field("bip", Some(FieldValue::U32(bip_number)));
        update_derivation_label(*dp, derivation_label_text)
      }
    }
  ));

  bip_hardened_checkbox.connect_active_notify(clone!(
    #[weak]
    derivation_label_text,
    #[weak]
    bip_hardened_checkbox,
    move |_| {
      let mut dp = DERIVATION_PATH.write().unwrap();
      dp.update_field(
        "hardened_bip",
        Some(FieldValue::Bool(bip_hardened_checkbox.is_active())),
      );
      update_derivation_label(*dp, derivation_label_text)
    }
  ));

  coin_hardened_checkbox.connect_active_notify(clone!(
    #[weak]
    derivation_label_text,
    #[weak]
    coin_hardened_checkbox,
    move |_| {
      let mut dp = DERIVATION_PATH.write().unwrap();

      dp.update_field(
        "hardened_coin",
        Some(FieldValue::Bool(coin_hardened_checkbox.is_active())),
      );
      update_derivation_label(*dp, derivation_label_text)
    }
  ));

  address_hardened_checkbox.connect_active_notify(clone!(
    #[weak]
    derivation_label_text,
    #[weak]
    address_hardened_checkbox,
    move |_| {
      let mut dp = DERIVATION_PATH.write().unwrap();

      dp.update_field(
        "hardened_address",
        Some(FieldValue::Bool(address_hardened_checkbox.is_active())),
      );
      update_derivation_label(*dp, derivation_label_text)
    }
  ));

  purpose_dropdown.connect_selected_notify(clone!(
    #[weak]
    derivation_label_text,
    #[weak]
    purpose_dropdown,
    move |_| {
      let purpose = purpose_dropdown.selected();
      let mut dp = DERIVATION_PATH.write().unwrap();

      dp.update_field("purpose", Some(FieldValue::U32(purpose)));
      update_derivation_label(*dp, derivation_label_text);
    }
  ));

  coin_entry.connect_changed(clone!(
    #[weak]
    derivation_label_text,
    #[strong]
    coin_entry,
    move |_| {
      let coin_number = coin_entry.buffer().text();
      let ff = coin_number.as_str();
      let my_int = ff.parse::<u32>();

      if my_int.is_ok() {
        let mut dp = DERIVATION_PATH.write().unwrap();
        dp.update_field("coin", Some(FieldValue::U32(my_int.unwrap())));
        update_derivation_label(*dp, derivation_label_text);
      }
    }
  ));

  address_spinbutton.connect_changed(clone!(
    #[weak]
    derivation_label_text,
    #[weak]
    address_spinbutton,
    move |_| {
      let address_number = address_spinbutton.text();
      let ff = address_number.as_str();
      let my_int = ff.parse::<u32>();

      if my_int.is_ok() {
        let mut dp = DERIVATION_PATH.write().unwrap();
        dp.update_field("address", Some(FieldValue::U32(my_int.unwrap())));
        update_derivation_label(*dp, derivation_label_text);
      }
    }
  ));

  address_count_spinbutton.connect_changed(clone!(
    #[weak]
    address_start_spinbutton,
    move |address_count_spinbutton| {
      let address_count = address_count_spinbutton.text();
      let address_count_str = address_count.as_str();
      let address_count_int = address_count_str.parse::<u32>().unwrap_or(0);

      let maximum = if address_count_int == 1 {
        WALLET_MAX_ADDRESSES
      } else {
        WALLET_MAX_ADDRESSES - address_count_int
      };

      let new_adjustment = gtk::Adjustment::new(0.0, 0.0, maximum as f64, 1.0, 10.0, 0.0);

      let old_status = address_start_spinbutton.text();
      let old_status_str = old_status.as_str();
      let mut old_status_int = old_status_str.parse::<u32>().unwrap_or(0);
      if old_status_int > maximum {
        old_status_int = maximum
      }

      address_start_spinbutton.set_adjustment(&new_adjustment);
      if old_status_int <= WALLET_MAX_ADDRESSES {
        address_start_spinbutton.set_text(&old_status_int.to_string());
      }
    }
  ));

  let address_generation_active = std::sync::Arc::new(std::sync::Mutex::new(false));

  // JUMP: Generate Addresses button
  generate_addresses_button.connect_clicked(clone!(
    #[strong]
    address_store,
    #[strong]
    stop_addresses_button_box,
    #[strong]
    generator_handler,
    #[strong]
    app_messages_state,
    #[weak]
    derivation_label_text,
    #[weak]
    master_private_key_text,
    #[weak]
    address_start_spinbutton,
    #[weak]
    address_count_spinbutton,
    #[weak]
    address_options_hardened_address_checkbox,
    #[weak]
    address_generation_progress_bar,
    #[weak]
    delete_addresses_button_box,
    #[weak]
    window,
    #[strong]
    address_generation_speed_label,
    #[weak]
    address_generation_speed_frame,
    #[weak]
    address_generation_active,
    move |_| {
      let buffer = master_private_key_text.buffer();
      let start_iter = buffer.start_iter();
      let end_iter = buffer.end_iter();
      let master_private_key_string = buffer.text(&start_iter, &end_iter, true);

      if master_private_key_string.is_empty() {
        let lock_app_messages = app_messages_state.borrow();
        match lock_app_messages.queue_message(
          t!("error.address.master").to_string(),
          gtk::MessageType::Warning,
        ) {
          Ok(_) => {}
          Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
        };
        return;
      }

      if *address_generation_active.lock().unwrap() {
        return;
      }
      address_generation_speed_label.set_label("0/sec");

      let wallet_settings = {
        let lock = WALLET_SETTINGS.lock().unwrap();
        lock.clone()
      };

      address_generation_progress_bar.set_fraction(0.0);
      address_generation_progress_bar.set_show_text(true);
      stop_addresses_button_box.set_visible(true);
      delete_addresses_button_box.set_visible(false);

      let coin_name = wallet_settings.coin_name.clone().unwrap_or_default();
      let _key_derivation = wallet_settings.key_derivation.clone().unwrap_or_default();

      #[cfg(not(feature = "dev"))]
      if _key_derivation != "secp256k1" {
        let lock_app_messages = app_messages_state.borrow();
        match lock_app_messages.queue_message(
          t!("error.address.unsupported").to_string(),
          gtk::MessageType::Error,
        ) {
          Ok(_) => {}
          Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
        };
        return;
      }

      let derivation_path = {
        let buffer = derivation_label_text.buffer();
        let start_iter = buffer.start_iter();
        let end_iter = buffer.end_iter();
        buffer.text(&start_iter, &end_iter, false)
      };

      let hardened_address = address_options_hardened_address_checkbox.is_active();
      let address_start_point = address_start_spinbutton.text();
      let mut address_start_point_int = address_start_point.parse::<usize>().unwrap_or(0);

      let address_count = address_count_spinbutton.text();
      let address_count_int = address_count.parse::<usize>().unwrap_or(1);

      let (channel_sender_addresses, channel_receiver_addresses) = std::sync::mpsc::channel();
      let (channel_sender_progress, channel_receiver_progress) = std::sync::mpsc::channel();
      let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
      let (cancel_speed_tx, cancel_speed_rx) = tokio::sync::watch::channel(false);

      let cpu_threads = num_cpus::get();

      let generating_threads = if address_count_int <= cpu_threads || address_count_int < 100 {
        1
      } else {
        cpu_threads
      };

      let addresses_per_thread = address_count_int / generating_threads;
      let extra_addresses = address_count_int % generating_threads;

      let generated_addresses = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
      let progress_status = std::sync::Arc::new(std::sync::Mutex::new(0.0));

      let next_generator = generated_addresses.clone();

      let start_time = std::time::Instant::now();

      let busy_cursor = gtk::gdk::Cursor::from_name("wait", None);
      window.set_cursor(busy_cursor.as_ref());

      let address_generation_speed_label = address_generation_speed_label.clone();
      let items_added_in_last_second = items_added_in_last_second.clone();
      let counts = counts.clone();
      let index = index.clone();
      let address_generation_active = address_generation_active.clone();

      // JUMP: Address speed
      // FIX: Close this loop !! create handler and close by abort/stop
      let speed_generator_id: Rc<RefCell<Option<SourceId>>> = Rc::new(RefCell::new(None));
      *speed_generator_id.borrow_mut() = Some(glib::timeout_add_local(
        std::time::Duration::from_millis(500),
        {
          let items_added_in_last_second = items_added_in_last_second.clone();
          let counts = counts.clone();
          let index = index.clone();
          let max_speed = max_speed.clone();
          let ema_speed = ema_speed.clone();
          let address_generation_speed_label = address_generation_speed_label.clone();
          let timeout_id = speed_generator_id.clone();

          move || {
            if *cancel_speed_rx.borrow() {
              let _ = timeout_id.borrow_mut().take().map(SourceId::remove);

              // address_generation_speed_label.set_label("0/sec");
              *items_added_in_last_second.lock().unwrap() = 0;
              counts.borrow_mut().fill(0);
              *ema_speed.borrow_mut() = 0.0;
              return glib::ControlFlow::Break;
            }

            let current_count = *items_added_in_last_second.lock().unwrap();
            let mut counts = counts.borrow_mut();
            let mut idx = *index.borrow_mut();
            let mut max = max_speed.borrow_mut();

            counts[idx] = current_count;
            idx = (idx + 1) % 2;
            *index.borrow_mut() = idx;

            let raw_speed = current_count as f64 * 2.0;
            if raw_speed as u64 > *max {
              *max = raw_speed as u64;
            }

            // JUMP ema speed
            let alpha = 0.5;
            let mut ema = ema_speed.borrow_mut();
            *ema = alpha * raw_speed + (1.0 - alpha) * *ema;
            let ema_avg_speed = *ema as u64;

            address_generation_speed_frame.set_visible(true);
            address_generation_speed_label
              .set_label(&format!("{}/sec (max: {}/sec)", ema_avg_speed, *max));

            *items_added_in_last_second.lock().unwrap() = 0;

            glib::ControlFlow::Continue
          }
        },
      ));

      *address_generation_active.lock().unwrap() = true;

      let address_loop = tokio::spawn(async move {
        let mut handles = vec![];
        let cancel_rx = std::sync::Arc::new(tokio::sync::Mutex::new(cancel_rx));

        for thread_id in 0..generating_threads {
          let num_addresses = if thread_id < extra_addresses {
            addresses_per_thread + 1
          } else {
            addresses_per_thread
          };

          if num_addresses == 0 {
            continue;
          }

          let channel_sender_addresses = channel_sender_addresses.clone();
          let channel_sender_progress = channel_sender_progress.clone();
          let cancel_rx = cancel_rx.clone();
          let wallet_settings = wallet_settings.clone();
          let derivation_path = derivation_path.clone();
          let coin_name = coin_name.clone();
          let generated_addresses = generated_addresses.clone();
          let progress_status = progress_status.clone();
          let items_added_in_last_second = items_added_in_last_second.clone();

          let handle = tokio::spawn(async move {
            let mut generated_count = 0;
            let mut current_index = address_start_point_int;
            let mut batch: Vec<CryptoAddresses> = Vec::new();

            while generated_count < num_addresses {
              let cancel_rx = cancel_rx.lock().await;
              if *cancel_rx.borrow() {
                #[cfg(debug_assertions)]
                println!("Address generation aborted (thread {})", thread_id);

                if !batch.is_empty() {
                  channel_sender_addresses.send(batch).unwrap_or_default()
                }

                let _ = channel_sender_progress.send(1.0);
                return;
              }
              drop(cancel_rx);

              if current_index > WALLET_MAX_ADDRESSES as usize {
                break;
              }

              let derivation_path = if hardened_address {
                format!("{}/{}'", derivation_path, current_index)
              } else {
                format!("{}/{}", derivation_path, current_index)
              };

              let coin_path_id = match qr2m_lib::derivation_path_to_integer(&derivation_path) {
                Ok(value) => value,
                Err(_) => return,
              };

              match CRYPTO_ADDRESS.entry(coin_path_id.clone()) {
                dashmap::mapref::entry::Entry::Vacant(_) => {
                  let magic_ingredients = keys::AddressHocusPokus {
                    coin_index: wallet_settings.coin_index.unwrap_or_default(),
                    derivation_path: derivation_path.clone(),
                    master_private_key_bytes: wallet_settings
                      .master_private_key_bytes
                      .clone()
                      .unwrap_or_default(),
                    master_chain_code_bytes: wallet_settings
                      .master_chain_code_bytes
                      .clone()
                      .unwrap_or_default(),
                    public_key_hash: wallet_settings.public_key_hash.clone().unwrap_or_default(),
                    key_derivation: wallet_settings.key_derivation.clone().unwrap_or_default(),
                    wallet_import_format: wallet_settings
                      .wallet_import_format
                      .clone()
                      .unwrap_or_default(),
                    hash: wallet_settings.hash.clone().unwrap_or_default(),
                    // seed: wallet_settings.seed.clone().unwrap_or_default(),
                  };

                  if let Ok(Some(address)) = keys::generate_address(magic_ingredients) {
                    let new_entry = CryptoAddresses {
                      id: Some(coin_path_id.clone()),
                      coin_name: Some(coin_name.clone()),
                      derivation_path: Some(derivation_path.clone()),
                      address: Some(address.address.clone()),
                      public_key: Some(address.public_key.clone()),
                      private_key: Some(address.private_key.clone()),
                    };

                    batch.push(new_entry);

                    // JUMP: batch size
                    if batch.len() >= 100
                      || batch.len() >= addresses_per_thread
                      || batch.len() >= address_count_int
                    {
                      channel_sender_addresses
                        .send(batch.clone())
                        .unwrap_or_default();
                      batch.clear();
                      // batch_size = std::cmp::min(batch_size * 2, max_batch_size);
                    }

                    let current_total =
                      generated_addresses.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;

                    let new_progress = if address_count_int > 0 {
                      (current_total as f64) / (address_count_int as f64)
                    } else {
                      0.0
                    };

                    let mut last = progress_status.lock().unwrap();
                    if new_progress > *last + 0.01 || new_progress >= 1.0 {
                      *last = new_progress;
                      let _ = channel_sender_progress.send(new_progress);
                    }

                    generated_count += 1;
                    current_index += 1;
                    *items_added_in_last_second.lock().unwrap() += 1u64;
                  } else {
                    eprintln!("problem with generating address");
                    break;
                  }
                }
                dashmap::mapref::entry::Entry::Occupied(_) => {
                  current_index += 1;
                  continue;
                }
              }
            }

            if !batch.is_empty() {
              channel_sender_addresses.send(batch).unwrap_or_default();
            }

            #[cfg(debug_assertions)]
            println!(
              "Thread {} generating derivation_path: {}",
              thread_id, &derivation_path
            );
          });

          handles.push(handle);
          address_start_point_int += num_addresses;
        }

        for handle in handles {
          handle.await.unwrap();
        }
        let _ = channel_sender_progress.send(1.0);
      });

      *generator_handler.lock().unwrap() = Some((address_loop, cancel_tx, cancel_speed_tx));

      glib::idle_add_local(clone!(
        #[strong]
        address_store,
        #[strong]
        app_messages_state,
        #[strong]
        address_generation_progress_bar,
        #[strong]
        stop_addresses_button_box,
        #[strong]
        delete_addresses_button_box,
        #[strong]
        generator_handler,
        move || {
          while let Ok(new_entry) = channel_receiver_addresses.try_recv() {
            let entries: Vec<AddressDatabase> = new_entry
              .into_iter()
              .filter(|new_coin| {
                let id = new_coin.id.as_deref().unwrap_or("");
                if CRYPTO_ADDRESS.contains_key(id) {
                  false
                } else {
                  CRYPTO_ADDRESS.insert(id.to_owned(), new_coin.clone());
                  true
                }
              })
              .map(|mut new_coin| {
                AddressDatabase::new(
                  &new_coin.id.take().unwrap_or_default(),
                  &new_coin.coin_name.take().unwrap_or_default(),
                  &new_coin.derivation_path.take().unwrap_or_default(),
                  &new_coin.address.take().unwrap_or_default(),
                  &new_coin.public_key.take().unwrap_or_default(),
                  &new_coin.private_key.take().unwrap_or_default(),
                )
              })
              .collect();

            address_store.extend_from_slice(&entries);
          }

          while let Ok(progress) = channel_receiver_progress.try_recv() {
            address_generation_progress_bar.set_fraction(progress);

            if progress >= 1.0 {
              let duration = start_time.elapsed();
              let message = format!("Address generation completed in {:.2?}", duration);

              #[cfg(debug_assertions)]
              println!("{}", message);

              let lock_app_messages = app_messages_state.borrow();
              match lock_app_messages.queue_message(message.to_string(), gtk::MessageType::Info) {
                Ok(_) => {}
                Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
              };

              stop_addresses_button_box.set_visible(false);
              delete_addresses_button_box.set_visible(true);
              // address_generation_speed_label.set_label("0/sec");

              window.set_cursor(None);

              let next_address = next_generator
                .as_ref()
                .load(std::sync::atomic::Ordering::SeqCst)
                // + address_count_int
                + address_start_point_int;

              address_start_spinbutton.set_value(next_address as f64);

              if let Some((handle, cancel_tx, cancel_speed_tx)) =
                generator_handler.lock().unwrap().take()
              {
                cancel_tx.send(true).ok();
                cancel_speed_tx.send(true).ok();
                handle.abort();
              } else {
                #[cfg(debug_assertions)]
                eprintln!("No generator handle!");
              }

              *address_generation_active.lock().unwrap() = false;

              return glib::ControlFlow::Break;
            }
          }

          glib::ControlFlow::Continue
        }
      ));
    }
  ));

  // JUMP: Delete Addresses button
  delete_addresses_button.connect_clicked(clone!(
    #[strong]
    address_store,
    #[weak]
    delete_addresses_button_box,
    #[weak]
    address_start_spinbutton,
    #[weak]
    address_generation_progress_bar,
    #[weak]
    window,
    #[strong]
    address_generation_speed_label,
    move |_| {
      let busy_cursor = gtk::gdk::Cursor::from_name("wait", None);
      window.set_cursor(busy_cursor.as_ref());
      address_store.remove_all();
      CRYPTO_ADDRESS.clear();
      address_start_spinbutton.set_text("0");
      address_generation_progress_bar.set_fraction(0.0);
      address_generation_progress_bar.set_show_text(false);
      delete_addresses_button_box.set_visible(false);
      address_generation_speed_label.set_label("0/sec");
      window.set_cursor(None);
    }
  ));

  // JUMP: Stop Addresses button
  stop_address_generation_button.connect_clicked(clone!(
    #[strong]
    generator_handler,
    #[strong]
    app_messages_state,
    #[weak]
    delete_addresses_button_box,
    #[weak]
    stop_addresses_button_box,
    #[weak]
    window,
    #[strong]
    address_generation_active,
    move |_| {
      window.set_cursor(None);

      if let Some((handle, cancel_tx, cancel_speed_tx)) = generator_handler.lock().unwrap().take() {
        cancel_tx.send(true).ok();
        cancel_speed_tx.send(true).ok();
        handle.abort();
      } else {
        #[cfg(debug_assertions)]
        eprintln!("No generator handle!");
      }

      *address_generation_active.lock().unwrap() = false;

      let message = "Address generation aborted";
      #[cfg(debug_assertions)]
      println!("{}", message);

      let lock_app_messages = app_messages_state.borrow();
      match lock_app_messages.queue_message(message.to_string(), gtk::MessageType::Warning) {
        Ok(_) => {}
        Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
      };

      // address_generation_speed_label.set_label("0/sec");
      delete_addresses_button_box.set_visible(true);
      stop_addresses_button_box.set_visible(false);
    }
  ));

  // Main sidebar
  let main_window_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
  let main_sidebar_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let main_infobar_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

  main_sidebar_box.append(&stack_sidebar);
  main_sidebar_box.append(&stack);
  main_infobar_box.append(&info_bar);
  main_infobar_box.set_hexpand(true);
  main_window_box.append(&main_sidebar_box);
  main_window_box.append(&main_infobar_box);
  main_window_box.set_hexpand(true);

  {
    let lock_app_messages = app_messages_state.borrow();
    match lock_app_messages.queue_message(t!("hello").to_string(), gtk::MessageType::Info) {
      Ok(_) => {}
      Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
    };
  }

  window.set_child(Some(&main_window_box));

  #[cfg(feature = "dev")]
  {
    if let Some(value) = start_time {
      let elapsed = value.elapsed();

      let message = format!("Application startup time: {:.2?}", elapsed);
      // println!("{}", message);

      let lock_app_messages = app_messages_state.borrow();
      match lock_app_messages.queue_message(message, gtk::MessageType::Info) {
        Ok(_) => {}
        Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
      };
    };
  }

  window.connect_close_request(clone!(
    #[strong]
    last_window,
    #[strong]
    generator_handler,
    move |window| {
      let gui_last_width = window.width() as i64;
      let gui_last_height = window.height() as i64;
      let gui_maximized = window.is_maximized();

      let gui_save_size = {
        let app_settings_lock = APP_SETTINGS.read().unwrap();
        app_settings_lock.gui_save_size.unwrap()
      };

      if gui_save_size {
        std::thread::spawn(move || {
          let mut settings = APP_SETTINGS.write().unwrap();
          settings.update_value("gui_last_width", toml_edit::value(gui_last_width), None);
          settings.update_value("gui_last_height", toml_edit::value(gui_last_height), None);
          settings.update_value("gui_maximized", toml_edit::value(gui_maximized), None);
          AppSettings::save_settings(&settings);
        });
      }

      if let Some(window) = last_window.as_ref() {
        window.set_visible(true);
        window.present()
      }

      if let Some((handle, cancel_tx, cancel_speed_tx)) = generator_handler.lock().unwrap().take() {
        cancel_tx.send(true).ok();
        cancel_speed_tx.send(true).ok();
        handle.abort();
      } else {
        #[cfg(debug_assertions)]
        eprintln!("No generator handle!");
      }

      glib::Propagation::Proceed
    }
  ));

  Ok(window)
}

#[cfg(feature = "dev")]
fn create_log_window(
  gui_state: Rc<RefCell<GuiState>>,
  // resources: std::sync::Arc<std::sync::Mutex<GuiResources>>,
  // log: std::sync::Arc<std::sync::Mutex<AppLog>>,
) -> FunctionOutput<gtk::ApplicationWindow> {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.create_log_window").to_string());

  let log_window = gtk::ApplicationWindow::builder()
    .title(t!("UI.main.log").to_string())
    // .default_width(WINDOW_SETTINGS_DEFAULT_WIDTH.try_into().unwrap())
    // .default_height(WINDOW_SETTINGS_DEFAULT_HEIGHT.try_into().unwrap())
    .resizable(true)
    .modal(false)
    .build();

  let lock_gui_state = gui_state.borrow_mut();
  let new_log_button = Rc::new(gtk::Button::new());
  lock_gui_state.register_button("log".to_string(), new_log_button);

  Ok(log_window)
}

fn create_settings_window(
  gui_state: Rc<RefCell<GuiState>>,
  app_messages_state: Rc<RefCell<AppMessages>>,
) -> FunctionOutput<gtk::ApplicationWindow> {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.create_settings_window").to_string());

  let lock_app_settings = APP_SETTINGS.read().unwrap();

  let settings_window = gtk::ApplicationWindow::builder()
    .title(t!("UI.settings").to_string())
    .default_width(WINDOW_SETTINGS_DEFAULT_WIDTH.try_into().unwrap())
    .default_height(WINDOW_SETTINGS_DEFAULT_HEIGHT.try_into().unwrap())
    .resizable(false)
    .modal(true)
    .build();

  let stack = Stack::new();
  let stack_sidebar = StackSidebar::new();
  stack_sidebar.set_stack(&stack);

  // -.-. --- .--. -.-- .-. .. --. .... -
  // JUMP: Settings: Sidebar 1: General settings
  // -.-. --- .--. -.-- .-. .. --. .... -
  let general_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
  let general_settings_frame = gtk::Frame::new(Some(&t!("UI.settings.general")));
  let content_general_box = gtk::Box::new(gtk::Orientation::Vertical, 20);

  general_settings_box.set_margin_top(10);
  general_settings_box.set_margin_bottom(0);
  general_settings_box.set_margin_start(10);
  general_settings_box.set_margin_end(10);
  content_general_box.set_margin_start(20);
  content_general_box.set_margin_bottom(20);

  general_settings_frame.set_hexpand(true);
  general_settings_frame.set_vexpand(true);
  general_settings_box.append(&general_settings_frame);
  general_settings_frame.set_child(Some(&content_general_box));

  // GUI theme color
  let default_gui_theme_color_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let default_gui_theme_color_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_gui_theme_color_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_gui_theme_color_label = gtk::Label::new(Some(&t!("UI.settings.general.theme")));
  let valid_gui_themes_as_strings: Vec<String> =
    VALID_GUI_THEMES.iter().map(|&x| x.to_string()).collect();
  let valid_gui_themes_as_str_refs: Vec<&str> = valid_gui_themes_as_strings
    .iter()
    .map(|s| s.as_ref())
    .collect();
  let gui_theme_dropdown = gtk::DropDown::from_strings(&valid_gui_themes_as_str_refs);
  let default_gui_theme = valid_gui_themes_as_strings
    .iter()
    .position(|s| *s == lock_app_settings.gui_theme.clone().unwrap())
    .unwrap_or(0);

  gui_theme_dropdown.set_selected(default_gui_theme.try_into().unwrap());
  gui_theme_dropdown.set_size_request(200, 10);
  default_gui_theme_color_box.set_hexpand(true);
  default_gui_theme_color_item_box.set_hexpand(true);
  default_gui_theme_color_item_box.set_margin_end(20);
  default_gui_theme_color_item_box.set_halign(gtk::Align::End);

  default_gui_theme_color_label_box.append(&default_gui_theme_color_label);
  default_gui_theme_color_item_box.append(&gui_theme_dropdown);
  default_gui_theme_color_box.append(&default_gui_theme_color_label_box);
  default_gui_theme_color_box.append(&default_gui_theme_color_item_box);
  content_general_box.append(&default_gui_theme_color_box);

  // GUI icons
  let default_gui_icons_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let default_gui_icons_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_gui_icons_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_gui_icons_label = gtk::Label::new(Some(&t!("UI.settings.general.icons")));
  let valid_gui_icons_as_strings: Vec<String> =
    VALID_GUI_ICONS.iter().map(|&x| x.to_string()).collect();
  let valid_gui_icons_as_str_refs: Vec<&str> = valid_gui_icons_as_strings
    .iter()
    .map(|s| s.as_ref())
    .collect();
  let gui_icons_dropdown = gtk::DropDown::from_strings(&valid_gui_icons_as_str_refs);
  let default_gui_icons = valid_gui_icons_as_strings
    .iter()
    .position(|s| *s == lock_app_settings.gui_icons.clone().unwrap())
    .unwrap_or(0);

  gui_icons_dropdown.set_selected(default_gui_icons.try_into().unwrap());
  gui_icons_dropdown.set_size_request(200, 10);
  default_gui_icons_box.set_hexpand(true);
  default_gui_icons_item_box.set_hexpand(true);
  default_gui_icons_item_box.set_margin_end(20);
  default_gui_icons_item_box.set_halign(gtk::Align::End);

  default_gui_icons_label_box.append(&default_gui_icons_label);
  default_gui_icons_item_box.append(&gui_icons_dropdown);
  default_gui_icons_box.append(&default_gui_icons_label_box);
  default_gui_icons_box.append(&default_gui_icons_item_box);
  content_general_box.append(&default_gui_icons_box);

  // GUI language
  let default_gui_language_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let default_gui_language_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_gui_language_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_gui_language_label = gtk::Label::new(Some(&t!("UI.settings.general.language")));
  let valid_gui_languages_as_strings: Vec<String> =
    APP_LANGUAGE.iter().map(|&x| x.to_string()).collect();
  let valid_gui_languages_as_str_refs: Vec<&str> = valid_gui_languages_as_strings
    .iter()
    .map(|s| s.as_ref())
    .collect();
  let default_gui_language_dropdown = gtk::DropDown::from_strings(&valid_gui_languages_as_str_refs);
  let default_gui_language = valid_gui_languages_as_strings
    .iter()
    .position(|s| *s == lock_app_settings.gui_language.clone().unwrap())
    .unwrap_or(0);

  default_gui_language_dropdown.set_selected(default_gui_language.try_into().unwrap());
  default_gui_language_dropdown.set_size_request(200, 10);
  default_gui_language_box.set_hexpand(true);
  default_gui_language_item_box.set_hexpand(true);
  default_gui_language_item_box.set_margin_end(20);
  default_gui_language_item_box.set_halign(gtk::Align::End);

  default_gui_language_label_box.append(&default_gui_language_label);
  default_gui_language_item_box.append(&default_gui_language_dropdown);
  default_gui_language_box.append(&default_gui_language_label_box);
  default_gui_language_box.append(&default_gui_language_item_box);
  content_general_box.append(&default_gui_language_box);

  // GUI: Save last window size
  let window_save_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
  let window_save_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let window_save_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let save_window_size_label = gtk::Label::new(Some(&t!("UI.settings.general.save_window")));
  let save_window_size_checkbox = gtk::CheckButton::new();
  let is_checked = lock_app_settings.gui_save_size.unwrap();

  save_window_size_checkbox.set_active(is_checked);
  window_save_label_box.set_hexpand(true);
  window_save_item_box.set_hexpand(true);
  window_save_item_box.set_margin_end(20);
  window_save_item_box.set_halign(gtk::Align::End);

  window_save_label_box.append(&save_window_size_label);
  window_save_item_box.append(&save_window_size_checkbox);
  window_save_box.append(&window_save_label_box);
  window_save_box.append(&window_save_item_box);
  content_general_box.append(&window_save_box);

  // Default search parameter
  let default_search_parameter_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let default_search_parameter_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_search_parameter_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_search_parameter_label = gtk::Label::new(Some(&t!("UI.settings.general.search")));
  let valid_search_parameters_as_strings: Vec<String> = VALID_COIN_SEARCH_PARAMETER
    .iter()
    .map(|&x| x.to_string())
    .collect();
  let valid_search_parameters_as_str_refs: Vec<&str> = valid_search_parameters_as_strings
    .iter()
    .map(|s| s.as_ref())
    .collect();
  let default_search_parameter_dropdown =
    gtk::DropDown::from_strings(&valid_search_parameters_as_str_refs);
  let default_search_parameter = valid_search_parameters_as_strings
    .iter()
    .position(|s| *s == lock_app_settings.gui_search.clone().unwrap())
    .unwrap_or(0);

  default_search_parameter_dropdown.set_selected(default_search_parameter.try_into().unwrap());
  default_search_parameter_dropdown.set_size_request(200, 10);
  default_search_parameter_box.set_hexpand(true);
  default_search_parameter_item_box.set_hexpand(true);
  default_search_parameter_item_box.set_margin_end(20);
  default_search_parameter_item_box.set_halign(gtk::Align::End);

  default_search_parameter_label_box.append(&default_search_parameter_label);
  default_search_parameter_item_box.append(&default_search_parameter_dropdown);
  default_search_parameter_box.append(&default_search_parameter_label_box);
  default_search_parameter_box.append(&default_search_parameter_item_box);
  content_general_box.append(&default_search_parameter_box);

  // APP notifications
  let notification_main_box = gtk::Box::new(gtk::Orientation::Vertical, 20);

  // Notification Status
  let enable_gui_notifications_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let enable_gui_notifications_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let enable_gui_notifications_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let enable_gui_notifications_label = gtk::Label::new(Some(&t!("UI.main.notification")));
  let enable_gui_notifications_checkbox = gtk::CheckButton::new();
  let enable_gui_notifications = lock_app_settings.gui_notifications.unwrap();

  enable_gui_notifications_checkbox.set_active(enable_gui_notifications);
  enable_gui_notifications_box.set_hexpand(true);
  enable_gui_notifications_item_box.set_hexpand(true);
  enable_gui_notifications_item_box.set_margin_end(20);
  enable_gui_notifications_item_box.set_halign(gtk::Align::End);

  enable_gui_notifications_label_box.append(&enable_gui_notifications_label);
  enable_gui_notifications_item_box.append(&enable_gui_notifications_checkbox);
  enable_gui_notifications_box.append(&enable_gui_notifications_label_box);
  enable_gui_notifications_box.append(&enable_gui_notifications_item_box);

  // Notification timeout
  let notification_timeout_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let notification_timeout_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let notification_timeout_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let notification_timeout_label =
    gtk::Label::new(Some(&t!("UI.settings.wallet.notification_timeout")));
  let notification_timeout = lock_app_settings.gui_notification_timeout.unwrap() as f64;
  let notification_timeout_adjustment =
    gtk::Adjustment::new(notification_timeout, 0.0, 120.0, 1.0, 10.0, 0.0);
  let notification_timeout_spinbutton =
    gtk::SpinButton::new(Some(&notification_timeout_adjustment), 1.0, 0);

  notification_timeout_spinbutton.set_size_request(200, 10);
  notification_timeout_box.set_hexpand(true);
  notification_timeout_item_box.set_hexpand(true);
  notification_timeout_item_box.set_margin_end(20);
  notification_timeout_item_box.set_halign(gtk::Align::End);

  notification_timeout_label_box.append(&notification_timeout_label);
  notification_timeout_item_box.append(&notification_timeout_spinbutton);
  notification_timeout_box.append(&notification_timeout_label_box);
  notification_timeout_box.append(&notification_timeout_item_box);

  if !enable_gui_notifications {
    notification_timeout_box.set_visible(false);
  }

  notification_main_box.append(&enable_gui_notifications_box);
  notification_main_box.append(&notification_timeout_box);
  content_general_box.append(&notification_main_box);

  // Log
  let enable_gui_log_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let enable_gui_log_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let enable_gui_log_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let enable_gui_log_label = gtk::Label::new(Some(&t!("UI.main.log")));
  let enable_gui_log_checkbox = gtk::CheckButton::new();
  let enable_gui_log = lock_app_settings.gui_log.unwrap();

  enable_gui_log_checkbox.set_active(enable_gui_log);
  enable_gui_log_box.set_hexpand(true);
  enable_gui_log_item_box.set_hexpand(true);
  enable_gui_log_item_box.set_margin_end(20);
  enable_gui_log_item_box.set_halign(gtk::Align::End);

  enable_gui_log_label_box.append(&enable_gui_log_label);
  enable_gui_log_item_box.append(&enable_gui_log_checkbox);
  enable_gui_log_box.append(&enable_gui_log_label_box);
  enable_gui_log_box.append(&enable_gui_log_item_box);
  content_general_box.append(&enable_gui_log_box);

  // Log level
  let default_gui_log_level_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let default_gui_log_level_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_gui_log_level_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_gui_log_level_label = gtk::Label::new(Some(&t!("UI.settings.general.log_level")));
  let valid_gui_log_level_as_strings: Vec<String> =
    APP_LOG_LEVEL.iter().map(|&x| x.to_string()).collect();
  let valid_gui_log_level_as_str_refs: Vec<&str> = valid_gui_log_level_as_strings
    .iter()
    .map(|s| s.as_ref())
    .collect();
  let default_gui_log_level_dropdown =
    gtk::DropDown::from_strings(&valid_gui_log_level_as_str_refs);
  let default_gui_log_level = valid_gui_log_level_as_strings
    .iter()
    .position(|s| *s == lock_app_settings.gui_log_level.clone().unwrap())
    .unwrap_or(0);

  default_gui_log_level_dropdown.set_selected(default_gui_log_level.try_into().unwrap());
  default_gui_log_level_dropdown.set_size_request(200, 10);
  default_gui_log_level_box.set_hexpand(true);
  default_gui_log_level_item_box.set_hexpand(true);
  default_gui_log_level_item_box.set_margin_end(20);
  default_gui_log_level_item_box.set_halign(gtk::Align::End);

  default_gui_log_level_label_box.append(&default_gui_log_level_label);
  default_gui_log_level_item_box.append(&default_gui_log_level_dropdown);
  default_gui_log_level_box.append(&default_gui_log_level_label_box);
  default_gui_log_level_box.append(&default_gui_log_level_item_box);
  content_general_box.append(&default_gui_log_level_box);

  if enable_gui_log {
    default_gui_log_level_box.set_visible(true);
  } else {
    default_gui_log_level_box.set_visible(false);
  };

  enable_gui_log_checkbox.connect_active_notify(clone!(
    #[weak]
    default_gui_log_level_box,
    move |cb| {
      default_gui_log_level_box.set_visible(cb.is_active());
    }
  ));

  enable_gui_notifications_checkbox.connect_active_notify(clone!(
    #[weak]
    notification_timeout_box,
    move |cb| {
      notification_timeout_box.set_visible(cb.is_active());
    }
  ));

  stack.add_titled(
    &general_settings_box,
    Some("sidebar-settings-general"),
    &t!("UI.settings.sidebar.general"),
  );

  // -.-. --- .--. -.-- .-. .. --. .... -
  // JUMP: Settings: Sidebar 2: Wallet settings
  // -.-. --- .--. -.-- .-. .. --. .... -
  let wallet_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
  let wallet_settings_frame = gtk::Frame::new(Some(&t!("UI.settings.wallet")));
  let content_wallet_box = gtk::Box::new(gtk::Orientation::Vertical, 20);

  wallet_settings_box.set_margin_top(10);
  wallet_settings_box.set_margin_bottom(0);
  wallet_settings_box.set_margin_start(10);
  wallet_settings_box.set_margin_end(10);
  content_wallet_box.set_margin_start(20);
  content_wallet_box.set_margin_bottom(20);

  wallet_settings_frame.set_hexpand(true);
  wallet_settings_frame.set_vexpand(true);
  wallet_settings_box.append(&wallet_settings_frame);
  wallet_settings_frame.set_child(Some(&content_wallet_box));

  // Default entropy source
  let qrng_enabled = cfg!(feature = "full") && lock_app_settings.anu_enabled.unwrap();
  let valid_entropy_sources: Vec<&str> = if qrng_enabled {
    VALID_ENTROPY_SOURCES.to_vec()
  } else {
    VALID_ENTROPY_SOURCES
      .iter()
      .filter(|&&x| x != "QRNG")
      .cloned()
      .collect()
  };

  let default_entropy_source_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let default_entropy_source_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_entropy_source_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_entropy_source_label =
    gtk::Label::new(Some(&t!("UI.settings.wallet.entropy.source")));
  let valid_entropy_source_as_strings: Vec<String> = valid_entropy_sources
    .iter()
    .map(|&x| x.to_string())
    .collect();

  let valid_entropy_source_as_str_refs: Vec<&str> = valid_entropy_source_as_strings
    .iter()
    .map(|s| s.as_ref())
    .collect();

  let entropy_source_dropdown = gtk::DropDown::from_strings(&valid_entropy_source_as_str_refs);
  let default_entropy_source = valid_entropy_source_as_strings
    .iter()
    .position(|s| *s == lock_app_settings.wallet_entropy_source.clone().unwrap())
    .unwrap_or(0);

  entropy_source_dropdown.set_selected(default_entropy_source.try_into().unwrap());
  entropy_source_dropdown.set_size_request(200, 10);
  default_entropy_source_box.set_hexpand(true);
  default_entropy_source_item_box.set_hexpand(true);
  default_entropy_source_item_box.set_margin_end(20);
  default_entropy_source_item_box.set_halign(gtk::Align::End);

  default_entropy_source_label_box.append(&default_entropy_source_label);
  default_entropy_source_item_box.append(&entropy_source_dropdown);
  default_entropy_source_box.append(&default_entropy_source_label_box);
  default_entropy_source_box.append(&default_entropy_source_item_box);
  content_wallet_box.append(&default_entropy_source_box);

  // Default entropy length
  let default_entropy_length_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let default_entropy_length_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_entropy_length_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_entropy_length_label =
    gtk::Label::new(Some(&t!("UI.settings.wallet.entropy.length")));
  let valid_entropy_lengths_as_strings: Vec<String> = VALID_ENTROPY_LENGTHS
    .iter()
    .map(|&x| x.to_string())
    .collect();

  let valid_entropy_lengths_as_str_refs: Vec<&str> = valid_entropy_lengths_as_strings
    .iter()
    .map(|s| s.as_ref())
    .collect();

  let entropy_length_dropdown = gtk::DropDown::from_strings(&valid_entropy_lengths_as_str_refs);
  let default_entropy_length = valid_entropy_lengths_as_strings
    .iter()
    .position(|x| x.parse::<u32>().unwrap() == lock_app_settings.wallet_entropy_length.unwrap())
    .unwrap_or(0);

  entropy_length_dropdown.set_selected(default_entropy_length.try_into().unwrap());
  entropy_length_dropdown.set_size_request(200, 10);
  default_entropy_length_box.set_hexpand(true);
  default_entropy_length_item_box.set_hexpand(true);
  default_entropy_length_item_box.set_margin_end(20);
  default_entropy_length_item_box.set_halign(gtk::Align::End);

  default_entropy_length_label_box.append(&default_entropy_length_label);
  default_entropy_length_item_box.append(&entropy_length_dropdown);
  default_entropy_length_box.append(&default_entropy_length_label_box);
  default_entropy_length_box.append(&default_entropy_length_item_box);
  content_wallet_box.append(&default_entropy_length_box);

  // Default mnemonic passphrase length
  let mnemonic_length_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let mnemonic_length_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let mnemonic_length_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let mnemonic_length_label = gtk::Label::new(Some(&t!("UI.settings.wallet.mnemonic_length")));
  let mnemonic_length = lock_app_settings.wallet_mnemonic_length.unwrap() as f64;
  let mnemonic_length_adjustment =
    gtk::Adjustment::new(mnemonic_length, 0.0, 8.0 * 128.0, 1.0, 100.0, 0.0);
  let mnemonic_length_spinbutton = gtk::SpinButton::new(Some(&mnemonic_length_adjustment), 1.0, 0);

  mnemonic_length_spinbutton.set_size_request(200, 10);
  mnemonic_length_box.set_hexpand(true);
  mnemonic_length_item_box.set_hexpand(true);
  mnemonic_length_item_box.set_margin_end(20);
  mnemonic_length_item_box.set_halign(gtk::Align::End);

  mnemonic_length_label_box.append(&mnemonic_length_label);
  mnemonic_length_item_box.append(&mnemonic_length_spinbutton);
  mnemonic_length_box.append(&mnemonic_length_label_box);
  mnemonic_length_box.append(&mnemonic_length_item_box);
  content_wallet_box.append(&mnemonic_length_box);

  let default_mnemonic_dictionary_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let default_mnemonic_dictionary_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_mnemonic_dictionary_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_mnemonic_dictionary_label =
    gtk::Label::new(Some(&t!("UI.settings.wallet.dictionary")));
  let valid_dictionary_as_strings: Vec<String> = VALID_MNEMONIC_DICTIONARY
    .iter()
    .map(|&x| x.to_string())
    .collect();

  let valid_dictionary_as_str_refs: Vec<&str> = valid_dictionary_as_strings
    .iter()
    .map(|s| s.as_ref())
    .collect();
  let mnemonic_dictionary_dropdown = gtk::DropDown::from_strings(&valid_dictionary_as_str_refs);
  let default_mnemonic_dictionary = valid_dictionary_as_strings
    .iter()
    .position(|x| {
      x.parse::<String>().unwrap()
        == lock_app_settings
          .wallet_mnemonic_dictionary
          .clone()
          .unwrap()
    })
    .unwrap_or(0);

  mnemonic_dictionary_dropdown.set_selected(default_mnemonic_dictionary.try_into().unwrap());
  mnemonic_dictionary_dropdown.set_size_request(200, 10);
  default_mnemonic_dictionary_box.set_hexpand(true);
  default_mnemonic_dictionary_item_box.set_hexpand(true);
  default_mnemonic_dictionary_item_box.set_margin_end(20);
  default_mnemonic_dictionary_item_box.set_halign(gtk::Align::End);

  default_mnemonic_dictionary_label_box.append(&default_mnemonic_dictionary_label);
  default_mnemonic_dictionary_item_box.append(&mnemonic_dictionary_dropdown);
  default_mnemonic_dictionary_box.append(&default_mnemonic_dictionary_label_box);
  default_mnemonic_dictionary_box.append(&default_mnemonic_dictionary_item_box);
  content_wallet_box.append(&default_mnemonic_dictionary_box);

  // Default BIP
  let default_bip_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let default_bip_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_bip_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_bip_label = gtk::Label::new(Some(&t!("UI.settings.wallet.bip")));
  let valid_bips_as_strings: Vec<String> = VALID_BIP_DERIVATIONS
    .iter()
    .map(|&x| x.to_string())
    .filter(|s| s != "Custom")
    .collect();

  let valid_bips_as_str_refs: Vec<&str> =
    valid_bips_as_strings.iter().map(|s| s.as_ref()).collect();
  let bip_dropdown = gtk::DropDown::from_strings(&valid_bips_as_str_refs);
  let default_bip = valid_bips_as_strings
    .iter()
    .position(|x| x.parse::<u32>().unwrap() == lock_app_settings.wallet_bip.unwrap())
    .unwrap_or(1); // Default BIP44

  bip_dropdown.set_selected(default_bip.try_into().unwrap());
  bip_dropdown.set_size_request(200, 10);
  default_bip_box.set_hexpand(true);
  default_bip_item_box.set_hexpand(true);
  default_bip_item_box.set_margin_end(20);
  default_bip_item_box.set_halign(gtk::Align::End);

  default_bip_label_box.append(&default_bip_label);
  default_bip_item_box.append(&bip_dropdown);
  default_bip_box.append(&default_bip_label_box);
  default_bip_box.append(&default_bip_item_box);
  content_wallet_box.append(&default_bip_box);

  // Default address count
  let default_address_count_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let default_address_count_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_address_count_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let default_address_count_label = gtk::Label::new(Some(&t!("UI.settings.wallet.address_count")));
  let default_address_count = lock_app_settings.wallet_address_count.unwrap() as f64;
  let address_count_adjustment =
    gtk::Adjustment::new(default_address_count, 1.0, 2147483647.0, 1.0, 10.0, 0.0);
  let address_count_spinbutton = gtk::SpinButton::new(Some(&address_count_adjustment), 1.0, 0);

  address_count_spinbutton.set_size_request(200, 10);
  default_address_count_box.set_hexpand(true);
  default_address_count_item_box.set_hexpand(true);
  default_address_count_item_box.set_margin_end(20);
  default_address_count_item_box.set_halign(gtk::Align::End);

  default_address_count_label_box.append(&default_address_count_label);
  default_address_count_item_box.append(&address_count_spinbutton);
  default_address_count_box.append(&default_address_count_label_box);
  default_address_count_box.append(&default_address_count_item_box);
  content_wallet_box.append(&default_address_count_box);

  // Hardened addresses
  let hardened_addresses_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
  let hardened_addresses_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let hardened_addresses_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let hardened_addresses_label = gtk::Label::new(Some(&t!("UI.settings.wallet.hardened")));
  let hardened_addresses_checkbox = gtk::CheckButton::new();
  let is_checked = lock_app_settings.wallet_hardened_address.unwrap();

  hardened_addresses_checkbox.set_active(is_checked);
  hardened_addresses_label_box.set_hexpand(true);
  hardened_addresses_item_box.set_hexpand(true);
  hardened_addresses_item_box.set_margin_end(20);
  hardened_addresses_item_box.set_halign(gtk::Align::End);

  hardened_addresses_label_box.append(&hardened_addresses_label);
  hardened_addresses_item_box.append(&hardened_addresses_checkbox);
  hardened_addresses_box.append(&hardened_addresses_label_box);
  hardened_addresses_box.append(&hardened_addresses_item_box);
  content_wallet_box.append(&hardened_addresses_box);

  stack.add_titled(
    &wallet_settings_box,
    Some("sidebar-settings-wallet"),
    &t!("UI.settings.sidebar.wallet"),
  );

  // -.-. --- .--. -.-- .-. .. --. .... -
  // JUMP: Settings: Sidebar 3: ANU settings
  // -.-. --- .--. -.-- .-. .. --. .... -
  let _use_anu_api_checkbox = gtk::CheckButton::new();
  let _log_anu_api_checkbox = gtk::CheckButton::new();

  let mut default_connection_timeout = lock_app_settings.anu_timeout.unwrap();
  default_connection_timeout = std::cmp::max(1, default_connection_timeout);
  default_connection_timeout =
    std::cmp::min(ANU_MAXIMUM_CONNECTION_TIMEOUT, default_connection_timeout);

  let anu_connection_timeout_adjustment = gtk::Adjustment::new(
    default_connection_timeout as f64,
    1.0,
    ANU_MAXIMUM_CONNECTION_TIMEOUT as f64,
    1.0,
    10.0,
    0.0,
  );
  let _anu_connection_timeout_spinbutton =
    gtk::SpinButton::new(Some(&anu_connection_timeout_adjustment), 1.0, 0);

  let valid_api_data_formats_as_strings: Vec<String> = VALID_ANU_API_DATA_FORMAT
    .iter()
    .map(|&x| x.into())
    .collect();

  let valid_api_data_formats_as_str_refs: Vec<&str> = valid_api_data_formats_as_strings
    .iter()
    .map(|s| s.as_ref())
    .collect();

  let _anu_data_format_dropdown = gtk::DropDown::from_strings(&valid_api_data_formats_as_str_refs);

  let mut default_array_length = lock_app_settings.anu_array_length.unwrap();
  default_array_length = std::cmp::max(1, default_array_length);
  default_array_length = std::cmp::min(ANU_MAXIMUM_ARRAY_LENGTH, default_array_length);

  let array_length_adjustment = gtk::Adjustment::new(
    default_array_length as f64,
    1.0,
    ANU_MAXIMUM_ARRAY_LENGTH as f64,
    1.0,
    10.0,
    0.0,
  );

  let _default_anu_array_length_spinbutton =
    gtk::SpinButton::new(Some(&array_length_adjustment), 1.0, 0);

  let mut default_hex_size = lock_app_settings.anu_hex_block_size.unwrap();
  default_hex_size = std::cmp::max(1, default_hex_size);
  default_hex_size = std::cmp::min(ANU_MAXIMUM_ARRAY_LENGTH, default_hex_size);

  let hex_block_size_adjustment = gtk::Adjustment::new(
    default_hex_size as f64,
    1.0,
    ANU_MAXIMUM_ARRAY_LENGTH as f64,
    1.0,
    10.0,
    0.0,
  );
  let _default_anu_hex_length_spinbutton =
    gtk::SpinButton::new(Some(&hex_block_size_adjustment), 1.0, 0);

  #[cfg(feature = "full")]
  {
    let anu_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let anu_settings_frame = gtk::Frame::new(Some(&t!("UI.settings.anu")));
    let content_anu_box = gtk::Box::new(gtk::Orientation::Vertical, 20);

    anu_settings_box.set_margin_top(10);
    anu_settings_box.set_margin_bottom(0);
    anu_settings_box.set_margin_start(10);
    anu_settings_box.set_margin_end(10);
    content_anu_box.set_margin_start(20);
    content_anu_box.set_margin_bottom(20);
    anu_settings_box.append(&anu_settings_frame);
    anu_settings_frame.set_child(Some(&content_anu_box));
    anu_settings_frame.set_hexpand(true);
    anu_settings_frame.set_vexpand(true);

    // Use ANU QRNG API
    let use_anu_api_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
    let use_anu_api_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let use_anu_api_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let use_anu_api_label = gtk::Label::new(Some(&t!("UI.settings.anu.use_anu")));
    let is_checked = lock_app_settings.anu_enabled.unwrap();

    _use_anu_api_checkbox.set_active(is_checked);
    use_anu_api_label_box.set_hexpand(true);
    use_anu_api_item_box.set_hexpand(true);
    use_anu_api_item_box.set_margin_end(20);
    use_anu_api_item_box.set_halign(gtk::Align::End);

    use_anu_api_label_box.append(&use_anu_api_label);
    use_anu_api_item_box.append(&_use_anu_api_checkbox);
    use_anu_api_box.append(&use_anu_api_label_box);
    use_anu_api_box.append(&use_anu_api_item_box);
    content_anu_box.append(&use_anu_api_box);

    // Log ANU QRNG API
    let log_anu_api_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
    let log_anu_api_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let log_anu_api_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let log_anu_api_label = gtk::Label::new(Some(&t!("UI.settings.anu.log")));

    _log_anu_api_checkbox.set_active(lock_app_settings.anu_log.unwrap());
    log_anu_api_label_box.set_hexpand(true);
    log_anu_api_item_box.set_hexpand(true);
    log_anu_api_item_box.set_margin_end(20);
    log_anu_api_item_box.set_halign(gtk::Align::End);

    log_anu_api_label_box.append(&log_anu_api_label);
    log_anu_api_item_box.append(&_log_anu_api_checkbox);
    log_anu_api_box.append(&log_anu_api_label_box);
    log_anu_api_box.append(&log_anu_api_item_box);
    content_anu_box.append(&log_anu_api_box);

    // ANU API data type
    let default_api_data_format_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let default_api_data_format_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_api_data_format_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_api_data_format_label = gtk::Label::new(Some(&t!("UI.settings.anu.data.type")));

    let default_api_data_format = valid_api_data_formats_as_strings
      .iter()
      .position(|x| {
        x.parse::<String>().unwrap() == lock_app_settings.anu_data_format.clone().unwrap()
      })
      .unwrap_or(0);

    _anu_data_format_dropdown.set_selected(default_api_data_format.try_into().unwrap());
    _anu_data_format_dropdown.set_size_request(200, 10);
    default_api_data_format_box.set_hexpand(true);
    default_api_data_format_item_box.set_hexpand(true);
    default_api_data_format_item_box.set_margin_end(20);
    default_api_data_format_item_box.set_halign(gtk::Align::End);

    default_api_data_format_label_box.append(&default_api_data_format_label);
    default_api_data_format_item_box.append(&_anu_data_format_dropdown);
    default_api_data_format_box.append(&default_api_data_format_label_box);
    default_api_data_format_box.append(&default_api_data_format_item_box);
    content_anu_box.append(&default_api_data_format_box);

    // ANU array length
    let default_anu_array_length_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let default_anu_array_length_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_anu_array_length_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_anu_array_length_label = gtk::Label::new(Some(&t!("UI.settings.anu.data.array")));

    default_anu_array_length_label_box.set_hexpand(true);
    default_anu_array_length_item_box.set_hexpand(true);
    default_anu_array_length_item_box.set_margin_end(20);
    default_anu_array_length_item_box.set_halign(gtk::Align::End);
    _default_anu_array_length_spinbutton.set_size_request(200, 10);

    default_anu_array_length_label_box.append(&default_anu_array_length_label);
    default_anu_array_length_item_box.append(&_default_anu_array_length_spinbutton);
    default_anu_array_length_box.append(&default_anu_array_length_label_box);
    default_anu_array_length_box.append(&default_anu_array_length_item_box);
    content_anu_box.append(&default_anu_array_length_box);

    // ANU hex block size
    let default_anu_hex_length_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let default_anu_hex_length_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_anu_hex_length_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_anu_hex_length_label = gtk::Label::new(Some(&t!("UI.settings.anu.data.hex")));

    default_anu_hex_length_label_box.set_hexpand(true);
    default_anu_hex_length_item_box.set_hexpand(true);
    default_anu_hex_length_item_box.set_margin_end(20);
    default_anu_hex_length_item_box.set_halign(gtk::Align::End);
    _default_anu_hex_length_spinbutton.set_size_request(200, 10);

    default_anu_hex_length_label_box.append(&default_anu_hex_length_label);
    default_anu_hex_length_item_box.append(&_default_anu_hex_length_spinbutton);
    default_anu_hex_length_box.append(&default_anu_hex_length_label_box);
    default_anu_hex_length_box.append(&default_anu_hex_length_item_box);
    content_anu_box.append(&default_anu_hex_length_box);

    if _anu_data_format_dropdown.selected() == 2 {
      default_anu_hex_length_box.set_visible(true);
    } else {
      default_anu_hex_length_box.set_visible(false);
    };

    if _use_anu_api_checkbox.is_active() {
      default_api_data_format_box.set_visible(true);
      log_anu_api_box.set_visible(true);
      default_anu_array_length_box.set_visible(true);
      if _anu_data_format_dropdown.selected() as usize == 2 {
        default_anu_hex_length_box.set_visible(true);
      } else {
        default_anu_hex_length_box.set_visible(false);
      }
    } else {
      log_anu_api_box.set_visible(false);
      default_api_data_format_box.set_visible(false);
      default_anu_array_length_box.set_visible(false);
      default_anu_hex_length_box.set_visible(false);
    };

    // Anu timeout
    let anu_connection_timeout_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
    let anu_connection_timeout_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let anu_connection_timeout_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let anu_connection_timeout_label = gtk::Label::new(Some(&t!("UI.settings.anu.timeout")));

    _anu_connection_timeout_spinbutton.set_size_request(200, 10);
    anu_connection_timeout_label_box.set_hexpand(true);
    anu_connection_timeout_item_box.set_hexpand(true);
    anu_connection_timeout_item_box.set_margin_end(20);
    anu_connection_timeout_item_box.set_halign(gtk::Align::End);

    anu_connection_timeout_label_box.append(&anu_connection_timeout_label);
    anu_connection_timeout_item_box.append(&_anu_connection_timeout_spinbutton);
    anu_connection_timeout_box.append(&anu_connection_timeout_label_box);
    anu_connection_timeout_box.append(&anu_connection_timeout_item_box);
    content_anu_box.append(&anu_connection_timeout_box);

    // Actions
    let default_anu_hex_length_box_clone = default_anu_hex_length_box.clone();
    let anu_data_format_dropdown_clone = _anu_data_format_dropdown.clone();

    _use_anu_api_checkbox.connect_toggled(move |checkbox| {
      if checkbox.is_active() {
        default_api_data_format_box.set_visible(true);
        log_anu_api_box.set_visible(true);
        default_anu_array_length_box.set_visible(true);
        if anu_data_format_dropdown_clone.selected() as usize == 2 {
          default_anu_hex_length_box_clone.set_visible(true);
        } else {
          default_anu_hex_length_box_clone.set_visible(false);
        }
      } else {
        default_api_data_format_box.set_visible(false);
        log_anu_api_box.set_visible(false);
        default_anu_array_length_box.set_visible(false);
        default_anu_hex_length_box_clone.set_visible(false);
      }
    });

    _anu_data_format_dropdown.connect_selected_notify(clone!(
      #[weak]
      default_anu_hex_length_box,
      // #[weak] _anu_data_format_dropdown,
      move |dd| {
        if dd.selected() as usize == 2 {
          default_anu_hex_length_box.set_visible(true);
        } else {
          default_anu_hex_length_box.set_visible(false);
        }
      }
    ));

    stack.add_titled(
      &anu_settings_box,
      Some("sidebar-settings-anu"),
      &t!("UI.settings.sidebar.anu"),
    );
  }

  // -.-. --- .--. -.-- .-. .. --. .... -
  // JUMP: Settings: Sidebar 4: Proxy settings
  // -.-. --- .--. -.-- .-. .. --. .... -
  let scrolled_window = gtk::ScrolledWindow::new();
  scrolled_window.set_max_content_height(400);

  let proxy_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
  let proxy_settings_frame = gtk::Frame::new(Some(&t!("UI.settings.proxy")));
  let content_proxy_box = gtk::Box::new(gtk::Orientation::Vertical, 20);

  proxy_settings_box.set_margin_top(10);
  proxy_settings_box.set_margin_bottom(0);
  proxy_settings_box.set_margin_start(10);
  proxy_settings_box.set_margin_end(10);
  content_proxy_box.set_margin_start(20);
  content_proxy_box.set_margin_bottom(20);

  proxy_settings_box.append(&proxy_settings_frame);
  proxy_settings_frame.set_child(Some(&content_proxy_box));
  proxy_settings_frame.set_hexpand(true);
  proxy_settings_frame.set_vexpand(true);
  scrolled_window.set_child(Some(&proxy_settings_box));

  // Use proxy settings
  let use_proxy_settings_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
  let use_proxy_settings_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let use_proxy_settings_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let use_proxy_settings_label = gtk::Label::new(Some(&t!("UI.settings.proxy.use")));
  let use_proxy_settings_checkbox = gtk::CheckButton::new();
  let proxy_status = lock_app_settings.proxy_status.unwrap();

  use_proxy_settings_checkbox.set_active(proxy_status);
  use_proxy_settings_label_box.set_hexpand(true);
  use_proxy_settings_item_box.set_hexpand(true);
  use_proxy_settings_item_box.set_margin_end(20);
  use_proxy_settings_item_box.set_halign(gtk::Align::End);

  use_proxy_settings_label_box.append(&use_proxy_settings_label);
  use_proxy_settings_item_box.append(&use_proxy_settings_checkbox);
  use_proxy_settings_box.append(&use_proxy_settings_label_box);
  use_proxy_settings_box.append(&use_proxy_settings_item_box);
  content_proxy_box.append(&use_proxy_settings_box);

  let proxy_manual_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 20);

  if proxy_status {
    proxy_manual_settings_box.set_visible(true);
  } else {
    proxy_manual_settings_box.set_visible(false);
  }

  // Proxy server address
  let proxy_server_address_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
  let proxy_server_address_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let proxy_server_address_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let proxy_server_address_label = gtk::Label::new(Some(&t!("UI.settings.proxy.address")));
  let proxy_server_address_entry = gtk::Entry::new();

  proxy_server_address_entry.set_size_request(200, 10);
  proxy_server_address_label_box.set_hexpand(true);
  proxy_server_address_item_box.set_hexpand(true);
  proxy_server_address_item_box.set_margin_end(20);
  proxy_server_address_item_box.set_halign(gtk::Align::End);
  proxy_server_address_entry.set_text(&lock_app_settings.proxy_server_address.clone().unwrap());

  proxy_server_address_label_box.append(&proxy_server_address_label);
  proxy_server_address_item_box.append(&proxy_server_address_entry);
  proxy_server_address_box.append(&proxy_server_address_label_box);
  proxy_server_address_box.append(&proxy_server_address_item_box);
  proxy_manual_settings_box.append(&proxy_server_address_box);

  // Proxy server port
  let proxy_server_port_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
  let proxy_server_port_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let proxy_server_port_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let proxy_server_port_label = gtk::Label::new(Some(&t!("UI.settings.proxy.port")));
  let proxy_server_port_entry = gtk::Entry::new();

  proxy_server_port_entry.set_size_request(200, 10);
  proxy_server_port_label_box.set_hexpand(true);
  proxy_server_port_item_box.set_hexpand(true);
  proxy_server_port_item_box.set_margin_end(20);
  proxy_server_port_item_box.set_halign(gtk::Align::End);

  proxy_server_port_entry.set_text(&lock_app_settings.proxy_server_port.unwrap().to_string());

  proxy_server_port_label_box.append(&proxy_server_port_label);
  proxy_server_port_item_box.append(&proxy_server_port_entry);
  proxy_server_port_box.append(&proxy_server_port_label_box);
  proxy_server_port_box.append(&proxy_server_port_item_box);
  proxy_manual_settings_box.append(&proxy_server_port_box);

  // Use proxy credentials
  let use_proxy_credentials_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
  let use_proxy_credentials_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let use_proxy_credentials_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let use_proxy_credentials_label = gtk::Label::new(Some(&t!("UI.settings.proxy.creds")));
  let use_proxy_credentials_checkbox = gtk::CheckButton::new();
  let is_checked = lock_app_settings.proxy_login_credentials.unwrap();

  use_proxy_credentials_checkbox.set_active(is_checked);
  use_proxy_credentials_label_box.set_hexpand(true);
  use_proxy_credentials_item_box.set_hexpand(true);
  use_proxy_credentials_item_box.set_margin_end(20);
  use_proxy_credentials_item_box.set_halign(gtk::Align::End);

  use_proxy_credentials_label_box.append(&use_proxy_credentials_label);
  use_proxy_credentials_item_box.append(&use_proxy_credentials_checkbox);
  use_proxy_credentials_box.append(&use_proxy_credentials_label_box);
  use_proxy_credentials_box.append(&use_proxy_credentials_item_box);
  proxy_manual_settings_box.append(&use_proxy_credentials_box);

  // Proxy credentials
  let use_proxy_credentials_content_box = gtk::Box::new(gtk::Orientation::Vertical, 20);

  if lock_app_settings.proxy_login_credentials.unwrap() {
    use_proxy_credentials_content_box.set_visible(true);
  } else {
    use_proxy_credentials_content_box.set_visible(false);
  }

  // Proxy username
  let proxy_username_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
  let proxy_username_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let proxy_username_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let proxy_username_label = gtk::Label::new(Some(&t!("UI.settings.proxy.username")));
  let proxy_username_entry = gtk::Entry::new();

  proxy_username_entry.set_size_request(200, 10);
  proxy_username_label_box.set_hexpand(true);
  proxy_username_item_box.set_hexpand(true);
  proxy_username_item_box.set_margin_end(20);
  proxy_username_item_box.set_halign(gtk::Align::End);
  proxy_username_entry.set_text(&lock_app_settings.proxy_login_username.clone().unwrap());

  proxy_username_label_box.append(&proxy_username_label);
  proxy_username_item_box.append(&proxy_username_entry);
  proxy_username_box.append(&proxy_username_label_box);
  proxy_username_box.append(&proxy_username_item_box);
  use_proxy_credentials_content_box.append(&proxy_username_box);

  // Proxy password
  let proxy_password_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
  let proxy_password_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let proxy_password_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let proxy_password_label = gtk::Label::new(Some(&t!("UI.settings.proxy.password")));
  let proxy_password_entry = gtk::PasswordEntry::new();

  proxy_password_entry.set_size_request(200, 10);
  proxy_password_label_box.set_hexpand(true);
  proxy_password_item_box.set_hexpand(true);
  proxy_password_item_box.set_margin_end(20);
  proxy_password_item_box.set_halign(gtk::Align::End);

  proxy_password_entry.set_show_peek_icon(true);
  proxy_password_entry.set_text(&lock_app_settings.proxy_login_password.clone().unwrap());

  proxy_password_label_box.append(&proxy_password_label);
  proxy_password_item_box.append(&proxy_password_entry);
  proxy_password_box.append(&proxy_password_label_box);
  proxy_password_box.append(&proxy_password_item_box);
  use_proxy_credentials_content_box.append(&proxy_password_box);
  proxy_manual_settings_box.append(&use_proxy_credentials_content_box);

  // Use proxy PAC
  let use_proxy_pac_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
  let use_proxy_pac_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let use_proxy_pac_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let use_proxy_pac_label = gtk::Label::new(Some(&t!("UI.settings.proxy.pac")));
  let use_proxy_pac_checkbox = gtk::CheckButton::new();
  let is_checked = lock_app_settings.proxy_use_pac.unwrap();

  use_proxy_pac_checkbox.set_active(is_checked);
  use_proxy_pac_label_box.set_hexpand(true);
  use_proxy_pac_item_box.set_hexpand(true);
  use_proxy_pac_item_box.set_margin_end(20);
  use_proxy_pac_item_box.set_halign(gtk::Align::End);

  use_proxy_pac_label_box.append(&use_proxy_pac_label);
  use_proxy_pac_item_box.append(&use_proxy_pac_checkbox);
  use_proxy_pac_box.append(&use_proxy_pac_label_box);
  use_proxy_pac_box.append(&use_proxy_pac_item_box);
  proxy_manual_settings_box.append(&use_proxy_pac_box);

  // Proxy PAC
  let use_proxy_pac_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);

  if lock_app_settings.proxy_use_pac.unwrap() {
    use_proxy_pac_content_box.set_visible(true);
  } else {
    use_proxy_pac_content_box.set_visible(false);
  }

  // Proxy PAC path
  let proxy_pac_path_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
  let proxy_pac_path_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let proxy_pac_path_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let proxy_pac_path_label = gtk::Label::new(Some(&t!("UI.settings.proxy.pac.path")));
  let proxy_pac_path_entry = gtk::Entry::new();

  proxy_pac_path_entry.set_size_request(200, 10);
  proxy_pac_path_label_box.set_hexpand(true);
  proxy_pac_path_item_box.set_hexpand(true);
  proxy_pac_path_item_box.set_margin_end(20);
  proxy_pac_path_item_box.set_halign(gtk::Align::End);
  proxy_pac_path_entry.set_text(&lock_app_settings.proxy_script_address.clone().unwrap());

  proxy_pac_path_label_box.append(&proxy_pac_path_label);
  proxy_pac_path_item_box.append(&proxy_pac_path_entry);
  proxy_pac_path_box.append(&proxy_pac_path_label_box);
  proxy_pac_path_box.append(&proxy_pac_path_item_box);
  use_proxy_pac_content_box.append(&proxy_pac_path_box);

  proxy_manual_settings_box.append(&use_proxy_pac_content_box);

  // Use proxy SSL
  let use_proxy_ssl_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
  let use_proxy_ssl_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let use_proxy_ssl_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let use_proxy_ssl_label = gtk::Label::new(Some(&t!("UI.settings.proxy.ssl")));
  let use_proxy_ssl_checkbox = gtk::CheckButton::new();
  let is_checked = lock_app_settings.proxy_use_ssl.unwrap();

  use_proxy_ssl_checkbox.set_active(is_checked);
  use_proxy_ssl_label_box.set_hexpand(true);
  use_proxy_ssl_item_box.set_hexpand(true);
  use_proxy_ssl_item_box.set_margin_end(20);
  use_proxy_ssl_item_box.set_halign(gtk::Align::End);

  use_proxy_ssl_label_box.append(&use_proxy_ssl_label);
  use_proxy_ssl_item_box.append(&use_proxy_ssl_checkbox);
  use_proxy_ssl_box.append(&use_proxy_ssl_label_box);
  use_proxy_ssl_box.append(&use_proxy_ssl_item_box);
  proxy_manual_settings_box.append(&use_proxy_ssl_box);

  // Proxy SSL certificate
  let use_proxy_ssl_certificate_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);

  if lock_app_settings.proxy_use_ssl.unwrap() {
    use_proxy_ssl_certificate_content_box.set_visible(true);
  } else {
    use_proxy_ssl_certificate_content_box.set_visible(false);
  }

  // Proxy SSL certificate path
  let proxy_ssl_certificate_path_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
  let proxy_ssl_certificate_path_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let proxy_ssl_certificate_path_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let proxy_ssl_certificate_path_label = gtk::Label::new(Some(&t!("UI.settings.proxy.ssl.path")));
  let proxy_ssl_certificate_path_entry = gtk::Entry::new();

  proxy_ssl_certificate_path_entry.set_size_request(200, 10);
  proxy_ssl_certificate_path_label_box.set_hexpand(true);
  proxy_ssl_certificate_path_item_box.set_hexpand(true);
  proxy_ssl_certificate_path_item_box.set_margin_end(20);
  proxy_ssl_certificate_path_item_box.set_halign(gtk::Align::End);
  proxy_ssl_certificate_path_entry
    .set_text(&lock_app_settings.proxy_ssl_certificate.clone().unwrap());

  proxy_ssl_certificate_path_label_box.append(&proxy_ssl_certificate_path_label);
  proxy_ssl_certificate_path_item_box.append(&proxy_ssl_certificate_path_entry);
  proxy_ssl_certificate_path_box.append(&proxy_ssl_certificate_path_label_box);
  proxy_ssl_certificate_path_box.append(&proxy_ssl_certificate_path_item_box);
  use_proxy_ssl_certificate_content_box.append(&proxy_ssl_certificate_path_box);
  proxy_manual_settings_box.append(&use_proxy_ssl_certificate_content_box);

  content_proxy_box.append(&proxy_manual_settings_box);

  stack.add_titled(
    &scrolled_window,
    Some("sidebar-settings-proxy"),
    &t!("UI.settings.sidebar.proxy"),
  );

  use_proxy_settings_checkbox.connect_active_notify(clone!(
    #[weak]
    proxy_manual_settings_box,
    move |dd| {
      if dd.is_active() {
        proxy_manual_settings_box.set_visible(true);
      } else {
        proxy_manual_settings_box.set_visible(false);
      }
    }
  ));

  use_proxy_credentials_checkbox.connect_active_notify(clone!(
    #[weak]
    use_proxy_credentials_content_box,
    move |cb| {
      use_proxy_credentials_content_box.set_visible(cb.is_active());
    }
  ));

  use_proxy_pac_checkbox.connect_active_notify(clone!(
    #[weak]
    use_proxy_pac_content_box,
    move |cb| {
      use_proxy_pac_content_box.set_visible(cb.is_active());
    }
  ));

  use_proxy_ssl_checkbox.connect_active_notify(clone!(
    // #[weak] use_proxy_ssl_checkbox,
    move |cb| {
      use_proxy_ssl_certificate_content_box.set_visible(cb.is_active());
    }
  ));

  // Compose settings window
  let main_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
  let main_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  main_content_box.append(&stack_sidebar);
  main_content_box.append(&stack);

  // Buttons
  let main_buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let left_buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let right_buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);

  let save_button = gtk::Button::with_label(&t!("UI.button.save"));
  let cancel_button = gtk::Button::with_label(&t!("UI.button.cancel"));
  let default_button = gtk::Button::with_label(&t!("UI.button.default"));

  // JUMP: Save settings button
  save_button.connect_clicked(clone!(
    #[weak]
    settings_window,
    #[strong]
    gui_state,
    #[weak]
    app_messages_state,
    move |_| {
      let mut settings = APP_SETTINGS.write().unwrap();

      let updates = vec![
        (
          "wallet_entropy_source",
          toml_edit::value(VALID_ENTROPY_SOURCES[entropy_source_dropdown.selected() as usize]),
        ),
        (
          "wallet_entropy_length",
          toml_edit::value(
            VALID_ENTROPY_LENGTHS[entropy_length_dropdown.selected() as usize] as i64,
          ),
        ),
        (
          "wallet_bip",
          toml_edit::value(VALID_BIP_DERIVATIONS[bip_dropdown.selected() as usize]),
        ),
        (
          "wallet_address_count",
          toml_edit::value(address_count_spinbutton.value_as_int() as i64),
        ),
        (
          "wallet_hardened_address",
          toml_edit::value(hardened_addresses_checkbox.is_active()),
        ),
        (
          "wallet_mnemonic_dictionary",
          toml_edit::value(
            VALID_MNEMONIC_DICTIONARY[mnemonic_dictionary_dropdown.selected() as usize],
          ),
        ),
        (
          "gui_save_size",
          toml_edit::value(save_window_size_checkbox.is_active()),
        ),
        (
          "gui_theme",
          toml_edit::value(VALID_GUI_THEMES[gui_theme_dropdown.selected() as usize]),
        ),
        (
          "gui_icons",
          toml_edit::value(VALID_GUI_ICONS[gui_icons_dropdown.selected() as usize]),
        ),
        (
          "gui_language",
          toml_edit::value(APP_LANGUAGE[default_gui_language_dropdown.selected() as usize]),
        ),
        (
          "gui_search",
          toml_edit::value(
            VALID_COIN_SEARCH_PARAMETER[default_search_parameter_dropdown.selected() as usize],
          ),
        ),
        (
          "gui_notifications",
          toml_edit::value(enable_gui_notifications_checkbox.is_active()),
        ),
        (
          "gui_notification_timeout",
          toml_edit::value(notification_timeout_spinbutton.value_as_int() as i64),
        ),
        (
          "wallet_mnemonic_length",
          toml_edit::value(mnemonic_length_spinbutton.value_as_int() as i64),
        ),
        (
          "gui_log",
          toml_edit::value(enable_gui_log_checkbox.is_active()),
        ),
        (
          "gui_log_level",
          toml_edit::value(APP_LOG_LEVEL[default_gui_log_level_dropdown.selected() as usize]),
        ),
        (
          "proxy_status",
          toml_edit::value(use_proxy_settings_checkbox.is_active()),
        ),
        (
          "proxy_server_address",
          toml_edit::value(proxy_server_address_entry.text().to_string()),
        ),
        (
          "proxy_server_port",
          toml_edit::value(
            proxy_server_port_entry
              .text()
              .parse::<u32>()
              .unwrap_or(8080) as i64,
          ),
        ),
        (
          "proxy_use_pac",
          toml_edit::value(use_proxy_ssl_checkbox.is_active()),
        ),
        (
          "proxy_script_address",
          toml_edit::value(proxy_pac_path_entry.text().to_string()),
        ),
        (
          "proxy_login_credentials",
          toml_edit::value(use_proxy_credentials_checkbox.is_active()),
        ),
        (
          "proxy_login_username",
          toml_edit::value(proxy_username_entry.text().to_string()),
        ),
        (
          "proxy_login_password",
          toml_edit::value(proxy_password_entry.text().to_string()),
        ),
        (
          "proxy_use_ssl",
          toml_edit::value(use_proxy_ssl_checkbox.is_active()),
        ),
        (
          "proxy_ssl_certificate",
          toml_edit::value(proxy_ssl_certificate_path_entry.text().to_string()),
        ),
      ];

      #[cfg(feature = "full")]
      let mut updates = updates;

      #[cfg(feature = "full")]
      {
        updates.extend([
          (
            "anu_enabled",
            toml_edit::value(_use_anu_api_checkbox.is_active()),
          ),
          (
            "anu_log",
            toml_edit::value(_log_anu_api_checkbox.is_active()),
          ),
          (
            "anu_timeout",
            toml_edit::value(_anu_connection_timeout_spinbutton.value_as_int() as i64),
          ),
          (
            "anu_data_format",
            toml_edit::value(
              VALID_ANU_API_DATA_FORMAT[_anu_data_format_dropdown.selected() as usize],
            ),
          ),
          (
            "anu_array_length",
            toml_edit::value(_default_anu_array_length_spinbutton.value_as_int() as i64),
          ),
          (
            "anu_hex_block_size",
            toml_edit::value(_default_anu_hex_length_spinbutton.value_as_int() as i64),
          ),
        ]);
      }

      updates.iter().for_each(|(key, value)| {
        let gui_related = matches!(*key, "gui_theme" | "gui_log" | "gui_icons");
        settings.update_value(key, value.clone(), gui_related.then(|| gui_state.clone()));
      });

      AppSettings::save_settings(&settings);

      if enable_gui_notifications_checkbox.is_active() {
        {
          let lock_app_messages = app_messages_state.borrow();
          match lock_app_messages.queue_message(
            t!("UI.messages.dialog.settings_saved").to_string(),
            gtk::MessageType::Info,
          ) {
            Ok(_) => {}
            Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
          };
        }
      }

      settings_window.close();
    }
  ));

  cancel_button.connect_clicked(clone!(
    #[weak]
    settings_window,
    move |_| settings_window.close()
  ));

  default_button.connect_clicked(clone!(
    #[weak]
    settings_window,
    #[weak]
    app_messages_state,
    #[strong]
    gui_state,
    move |_| {
      let dialog = gtk::AlertDialog::builder()
        .modal(true)
        .message(t!("UI.settings.reset.title"))
        .detail(t!("UI.settings.reset.message"))
        .buttons(gtk::glib::StrV::from(vec![
          t!("UI.button.cancel").to_string(),
          t!("UI.button.ok").to_string(),
        ]))
        .build();

      let gui_state = gui_state.clone();

      dialog.choose(
        Some(&settings_window.clone()),
        None::<&gio::Cancellable>,
        move |response| match response {
          Ok(1) => {
            let lock_app_messages = app_messages_state.borrow();
            // let ok = t!("UI.button.ok").to_string();

            match reset_user_settings() {
              Ok(_) => {
                settings_window.close();

                match AppSettings::load_settings() {
                  Ok(_) => {
                    d3bug("<<< load_settings", "debug");
                  }
                  Err(err) => d3bug(&format!("load_settings: {:?}", err), "error"),
                };

                adw::StyleManager::default().set_color_scheme(adw::ColorScheme::PreferLight);

                let new_gui_state = Rc::new(RefCell::new(GuiState::default_config()));
                let mut lock_new_gui_state = new_gui_state.borrow_mut();
                let mut lock_gui_state = gui_state.borrow_mut();

                lock_gui_state.reload_gui_icons();
                lock_new_gui_state.gui_main_buttons = lock_gui_state.gui_main_buttons.clone();

                lock_new_gui_state.reload_gui_icons();
                lock_new_gui_state.apply_language();

                match lock_app_messages.queue_message(
                  t!("UI.messages.dialog.settings_reset").to_string(),
                  gtk::MessageType::Info,
                ) {
                  Ok(_) => {}
                  Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
                };
              }
              Err(_) => {
                match lock_app_messages.queue_message(
                  t!("error.settings.reset").to_string(),
                  gtk::MessageType::Error,
                ) {
                  Ok(_) => {}
                  Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
                };
              }
            }
          }
          _ => {
            #[cfg(debug_assertions)]
            eprintln!("Reset canceled");
          }
        },
      );
    }
  ));

  main_buttons_box.append(&left_buttons_box);
  main_buttons_box.append(&right_buttons_box);
  left_buttons_box.append(&default_button);

  right_buttons_box.append(&save_button);
  right_buttons_box.append(&cancel_button);
  main_buttons_box.set_margin_bottom(10);
  main_buttons_box.set_margin_top(10);
  main_buttons_box.set_margin_start(10);
  main_buttons_box.set_margin_end(10);

  main_buttons_box.set_hexpand(true);
  left_buttons_box.set_hexpand(true);
  right_buttons_box.set_hexpand(true);

  right_buttons_box.set_direction(gtk::TextDirection::Rtl);
  main_settings_box.append(&main_content_box);
  main_settings_box.append(&main_buttons_box);
  settings_window.set_child(Some(&main_settings_box));

  Ok(settings_window)
}

fn reset_user_settings() -> FunctionOutput<()> {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.reset_user_settings").to_string());

  {
    let local_settings = os::LOCAL_SETTINGS.lock().unwrap();
    let local_config_file = local_settings.local_config_file.clone().unwrap();

    #[cfg(debug_assertions)]
    println!("\t- Local config file: {:?}", local_config_file);

    match fs::remove_file(local_config_file) {
      Ok(_) => {
        #[cfg(debug_assertions)]
        println!("\t- Local config file deleted");
      }
      Err(_err) => {
        #[cfg(debug_assertions)]
        eprintln!("\t- Local config file NOT deleted \n Error: {}", _err);
      }
    };
  }

  match os::check_local_config() {
    Ok(_) => {
      d3bug("<<< check_local_config", "debug");
    }
    Err(err) => d3bug(&format!("check_local_config: {:?}", err), "error"),
  };

  Ok(())
}

fn create_about_window() {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.create_about_window").to_string());

  let pixy: gtk4::gdk::Texture =
    qr2m_lib::get_texture_from_resource(&format!("logo/logo.{}", GUI_IMAGE_EXTENSION));
  let logo_picture = gtk::Picture::new();
  logo_picture.set_paintable(Some(&pixy));

  let my_license = std::path::Path::new("licenses").join("QR2M.license");
  let app_license = qr2m_lib::get_text_from_resources(&my_license.to_string_lossy());

  let their_license = std::path::Path::new("licenses").join("GTK.license");
  let lgpl_license = qr2m_lib::get_text_from_resources(&their_license.to_string_lossy());

  let licenses = format!("{}\n\n---\n\n{}", app_license, lgpl_license);

  let gtk_license = t!("UI.dialog.gtk").to_string();

  let my_key = std::path::Path::new("keys").join("Control-Owl-Public-key.asc");
  let my_public = qr2m_lib::get_text_from_resources(&my_key.to_string_lossy());

  let author = format!(
    "{}\n\n\
        Public key:\n\
        {}",
    APP_AUTHOR.unwrap(),
    my_public,
  );

  let about_window = gtk::AboutDialog::builder()
    .modal(true)
    // .default_width(600)
    .default_height(400)
    .program_name(APP_DESCRIPTION.unwrap())
    .version(APP_VERSION.unwrap())
    .website("https://www.github.com/control-owl/qr2m")
    .website_label("GitHub project")
    .authors([author])
    .copyright("CC-BY-NC-ND-4.0 [2023-2025] Control Owl")
    .license(licenses)
    .wrap_license(true)
    .comments(gtk_license)
    .logo(
      &logo_picture
        .paintable()
        .unwrap_or(gtk::gdk::Paintable::new_empty(32, 32)),
    )
    .build();

  about_window.present();
}

fn open_wallet_from_file(
  app_messages_state: &Rc<RefCell<AppMessages>>,
) -> (String, Option<String>) {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.open_wallet_from_file").to_string());

  let open_context = glib::MainContext::default();
  let open_loop = glib::MainLoop::new(Some(&open_context), false);
  let (tx, rx) = std::sync::mpsc::channel::<Option<(String, Option<String>)>>();

  let open_window = gtk::Window::new();
  let open_dialog = gtk::FileDialog::builder()
    .title("Open Wallet File")
    .modal(true)
    .build();

  let filter = gtk::FileFilter::new();
  filter.add_pattern("*.qr2m");
  filter.set_name(Some("Wallet file (*.qr2m)"));
  open_dialog.set_default_filter(Some(&filter));

  let app_messages_state_open = app_messages_state.clone();

  let open_loop_clone = open_loop.clone();
  open_dialog.open(
    Some(&open_window),
    None::<&gio::Cancellable>,
    move |response| {
      match response {
        Ok(file) => {
          if let Some(path) = file.path() {
            let file_path = path.to_string_lossy().to_string();

            #[cfg(debug_assertions)]
            println!("\t- Wallet file chosen: {:?}", file_path);

            match process_wallet_file_from_path(&file_path) {
              Ok((_version, entropy, password)) => {
                let passphrase = password.map(|s| s.to_string());
                let lock_app_messages = app_messages_state_open.borrow();
                if tx.send(Some((entropy, passphrase.clone()))).is_err() {
                  match lock_app_messages.queue_message(
                    format!("{} : {}", t!("error.wallet.send"), "Channel send failed"),
                    gtk::MessageType::Error,
                  ) {
                    Ok(_) => {}
                    Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
                  };
                } else {
                  match lock_app_messages.queue_message(
                    t!("UI.messages.wallet.open").to_string(),
                    gtk::MessageType::Info,
                  ) {
                    Ok(_) => {}
                    Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
                  };
                }
              }
              Err(err) => {
                let lock_app_messages = app_messages_state_open.borrow();
                match lock_app_messages.queue_message(
                  format!("{} : {}", t!("error.wallet.process"), err),
                  gtk::MessageType::Error,
                ) {
                  Ok(_) => {}
                  Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
                };
                let _ = tx.send(None);
              }
            }
          } else {
            let _ = tx.send(None);
          }
        }
        Err(_) => {
          let _ = tx.send(None);
        }
      }
      open_loop_clone.quit();
    },
  );

  open_loop.run();

  match rx.recv() {
    Ok(Some((entropy, passphrase))) => (entropy, passphrase),
    Ok(None) => (String::new(), None),
    Err(err) => {
      let lock_state = app_messages_state.borrow();
      match lock_state.queue_message(
        format!("{} : {}", t!("error.wallet.open"), err),
        gtk::MessageType::Error,
      ) {
        Ok(_) => {}
        Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
      };
      (String::new(), None)
    }
  }
}

fn save_wallet_to_file(app_messages_state: &Rc<RefCell<AppMessages>>) {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.save_wallet_to_file").to_string());

  let save_context = glib::MainContext::default();
  let save_loop = glib::MainLoop::new(Some(&save_context), false);

  let wallet_settings = WALLET_SETTINGS.lock().unwrap();
  let entropy_string = wallet_settings.entropy_string.clone().unwrap_or_default();
  let mnemonic_passphrase = wallet_settings
    .mnemonic_passphrase
    .clone()
    .unwrap_or_default();

  let save_window = gtk::Window::new();
  let save_dialog = gtk::FileDialog::builder()
    .title(t!("UI.dialog.save").to_string())
    .modal(true)
    .accept_label(t!("UI.button.save").to_string())
    .build();

  let filter = gtk::FileFilter::new();
  filter.add_pattern(&format!("*.{}", WALLET_DEFAULT_EXTENSION));
  filter.set_name(Some(&format!(
    "Wallet file (*.{})",
    WALLET_DEFAULT_EXTENSION
  )));
  save_dialog.set_default_filter(Some(&filter));

  let app_messages_state_clone = app_messages_state.clone();
  let save_loop_clone = save_loop.clone();

  save_dialog.save(
    Some(&save_window),
    None::<&gio::Cancellable>,
    move |response| {
      match response {
        Ok(file) => {
          if let Some(path) = file.path() {
            let path_with_extension = if path
              .extension()
              .map(|ext| ext.to_string_lossy().to_string())
              .unwrap_or_default()
              != WALLET_DEFAULT_EXTENSION
            {
              path.with_extension(WALLET_DEFAULT_EXTENSION)
            } else {
              path
            };

            let wallet_data = format!(
              "version = {}\n{}\n{}",
              WALLET_CURRENT_VERSION, entropy_string, mnemonic_passphrase
            );

            match fs::write(&path_with_extension, &wallet_data) {
              Ok(_) => {
                let lock_app_messages = app_messages_state_clone.borrow();
                match lock_app_messages.queue_message(
                  t!("UI.messages.wallet.save").to_string(),
                  gtk::MessageType::Info,
                ) {
                  Ok(_) => {}
                  Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
                };
              }
              Err(err) => {
                let lock_app_messages = app_messages_state_clone.borrow();
                match lock_app_messages.queue_message(
                  format!("{} : {}", t!("error.wallet.save"), err),
                  gtk::MessageType::Error,
                ) {
                  Ok(_) => {}
                  Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
                };
              }
            }
          }
        }
        Err(err) => {
          let lock_app_messages = app_messages_state_clone.borrow();
          match lock_app_messages.queue_message(
            format!("{} : {}", t!("error.wallet.cancel"), err),
            gtk::MessageType::Error,
          ) {
            Ok(_) => {}
            Err(err) => d3bug(&format!("queue_message: {:?}", err), "error"),
          };
        }
      }
      save_loop_clone.quit();
    },
  );

  save_loop.run();
}

fn update_derivation_label(dp: DerivationPath, label: gtk::TextView) {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.update_derivation_label").to_string());

  let mut path = String::new();
  path.push('m');

  path.push_str(&format!("/{}", dp.bip.unwrap_or_default()));
  if dp.hardened_bip.unwrap_or_default() {
    path.push('\'')
  }

  path.push_str(&format!("/{}", dp.coin.unwrap_or_default()));
  if dp.hardened_coin.unwrap_or_default() {
    path.push('\'')
  }

  path.push_str(&format!("/{}", dp.address.unwrap_or_default()));
  if dp.hardened_address.unwrap_or_default() {
    path.push('\'')
  }

  if dp.bip.unwrap() != 32 {
    path.push_str(&format!("/{}", dp.purpose.unwrap_or_default()));
  }

  #[cfg(debug_assertions)]
  println!("\t- Derivation path: {:?}", &path);

  label.buffer().set_text(&path);
}

fn process_wallet_file_from_path(file_path: &str) -> Result<(u8, String, Option<String>), String> {
  #[cfg(debug_assertions)]
  println!(
    "[+] {}",
    &t!("log.process_wallet_file_from_path").to_string()
  );

  let file = File::open(file_path).map_err(|_| "Error: Could not open wallet file".to_string())?;
  let mut lines = io::BufReader::new(file).lines();

  let version_line = match lines.next() {
    Some(Ok(line)) => line,
    Some(Err(_)) => return Err("Error: Failed to read the version line".to_string()),
    None => return Err("Error: File is empty, missing version line".to_string()),
  };

  let version = parse_wallet_version(&version_line)?;

  match version {
    1 => {
      let entropy = match lines.next() {
        Some(Ok(line)) => line,
        Some(Err(_)) => return Err("Error: Failed to read entropy line".to_string()),
        None => return Err("Error: Missing entropy line for version 1 wallet".to_string()),
      };

      if !qr2m_lib::is_valid_entropy(&entropy) {
        return Err("Error: Invalid entropy size.".to_string());
      }

      let passphrase = match lines.next() {
        Some(Ok(line)) => Some(line),
        Some(Err(_)) => return Err("Error: Failed to read passphrase line".to_string()),
        None => None,
      };

      Ok((version, entropy, passphrase))
    }
    _ => Err(format!("Unsupported wallet version '{}'", version)),
  }
}

fn parse_wallet_version(line: &str) -> Result<u8, String> {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.parse_wallet_version").to_string());

  if let Some(stripped) = line.strip_prefix("version = ") {
    match stripped.parse::<u8>() {
      Ok(version) => Ok(version),
      Err(_) => Err("Error: Invalid version format, expected an integer".to_string()),
    }
  } else {
    Err("Error: Version line is malformed, expected 'version = X' format".to_string())
  }
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

fn d3bug(message: &str, msg_type: &str) {
  let (color_code, prefix) = match msg_type {
    "info" => ("\x1b[34m", "[INFO] "),       // Blue
    "debug" => ("\x1b[32m", "[DEBUG] "),     // Green
    "error" => ("\x1b[31m", "[ERROR] "),     // Red
    "warning" => ("\x1b[33m", "[WARNING] "), // Yellow
    _ => ("\x1b[0m", "[UNKNOWN] "),          // Default/reset
  };

  let reset = "\x1b[0m";

  #[cfg(debug_assertions)]
  if msg_type == "debug" {
    println!("{}{}{}{}", color_code, prefix, message, reset);
  }

  if msg_type != "debug" {
    println!("{}{}{}{}", color_code, prefix, message, reset);
  }
}

fn create_welcome_window(
  application: &adw::Application,
  gui_state: Rc<RefCell<GuiState>>,
) -> FunctionOutput<gtk::ApplicationWindow> {
  d3bug(">>> create_welcome_window", "debug");

  let welcome_window = gtk::ApplicationWindow::builder()
    .application(application)
    .title(format!("{} {}", APP_NAME.unwrap(), APP_VERSION.unwrap()))
    .decorated(true)
    .valign(gtk::Align::Center)
    .resizable(false)
    .build();

  let welcome_main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
  welcome_main_box.set_margin_top(50);
  welcome_main_box.set_margin_start(50);
  welcome_main_box.set_margin_end(50);
  welcome_window.set_child(Some(&welcome_main_box));

  let logo_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  let pixy: gtk4::gdk::Texture =
    qr2m_lib::get_texture_from_resource(&format!("logo/logo.{}", GUI_IMAGE_EXTENSION));

  let logo_picture = gtk::Image::from_paintable(Some(&pixy));
  logo_picture.set_size_request(256, 256);
  logo_picture.set_paintable(Some(&pixy));

  logo_box.append(&logo_picture);
  logo_box.set_halign(gtk::Align::Center);
  welcome_main_box.append(&logo_box);

  let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  button_box.set_margin_start(50);
  button_box.set_margin_end(50);
  button_box.set_margin_top(50);
  button_box.set_margin_bottom(50);
  button_box.set_halign(gtk::Align::Center);
  welcome_main_box.append(&button_box);

  let new_wallet_button = gtk::Button::with_label("New wallet");
  button_box.append(&new_wallet_button);

  #[cfg(feature = "dev")]
  new_wallet_button.connect_clicked(clone!(
    #[strong]
    application,
    #[weak]
    welcome_window,
    move |_| {
      d3bug("new_wallet_button.connect_clicked", "debug");

      let busy_cursor = gtk::gdk::Cursor::from_name("wait", None);
      welcome_window.set_cursor(busy_cursor.as_ref());

      match create_new_wallet_window(&application, Some(welcome_window.clone())) {
        Ok(window) => {
          window.present();
          d3bug("<<< create_new_wallet_window", "debug");
        }
        Err(err) => d3bug(&format!("create_new_wallet_window: {:?}", err), "error"),
      };

      welcome_window.set_cursor(None);
    }
  ));

  let open_wallet_button = gtk::Button::with_label("Open wallet");
  button_box.append(&open_wallet_button);

  let advance_wallet_button = gtk::Button::with_label("Advance");
  button_box.append(&advance_wallet_button);

  advance_wallet_button.connect_clicked(clone!(
    #[strong]
    application,
    #[weak]
    welcome_window,
    #[strong]
    gui_state,
    move |_| {
      d3bug("advance_wallet_button.connect_clicked", "debug");

      let busy_cursor = gtk::gdk::Cursor::from_name("wait", None);
      welcome_window.set_cursor(busy_cursor.as_ref());

      match create_main_window(
        &application,
        gui_state.clone(),
        Some(welcome_window.clone()),
        #[cfg(feature = "dev")]
        Some(std::time::Instant::now()),
      ) {
        Ok(window) => {
          window.present();
          d3bug("<<< create_main_window", "debug");
        }
        Err(err) => d3bug(&format!("create_main_window: {:?}", err), "error"),
      };

      welcome_window.set_cursor(None);
    }
  ));

  Ok(welcome_window)
}

#[cfg(feature = "dev")]
fn create_new_wallet_window(
  application: &adw::Application,
  last_window: Option<gtk::ApplicationWindow>,
) -> FunctionOutput<gtk::ApplicationWindow> {
  d3bug(">>> create_new_wallet_window", "debug");

  let new_wallet_window = gtk::ApplicationWindow::builder()
    .application(application)
    .title(format!("{} {}", APP_NAME.unwrap(), APP_VERSION.unwrap(),))
    .width_request(600)
    .height_request(400)
    .build();

  if let Some(window) = last_window.as_ref() {
    window.set_visible(false);
  }

  new_wallet_window.connect_close_request(clone!(
    #[strong]
    last_window,
    #[strong]
    application,
    move |_| {
      if let Some(window) = last_window.as_ref() {
        window.set_visible(true);
        window.present()
      } else {
        application.quit();
      }

      glib::Propagation::Proceed
    }
  ));

  let wallet_main_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
  wallet_main_box.set_hexpand(true);
  wallet_main_box.set_vexpand(true);
  wallet_main_box.set_margin_top(10);
  wallet_main_box.set_margin_start(10);
  wallet_main_box.set_margin_end(10);
  wallet_main_box.set_margin_bottom(10);

  let wallet_main_content_box = gtk::Box::new(gtk::Orientation::Vertical, 20);

  // Source
  let source_main_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let source_label = gtk::Label::new(Some(&t!("UI.wallet.source")));
  source_main_box.append(&source_label);

  let valid_source_as_string: Vec<String> = VALID_ENTROPY_SOURCES
    .iter()
    .map(|&x| x.to_string())
    .collect();
  let valid_source_as_ref: Vec<&str> = valid_source_as_string.iter().map(|s| s.as_ref()).collect();
  let source_dropdown = gtk::DropDown::from_strings(&valid_source_as_ref);
  source_main_box.append(&source_dropdown);
  wallet_main_content_box.append(&source_main_box);

  // Derivation path
  let derivation_main_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  let derivation_label = gtk::Label::new(Some(&t!("UI.main.address.derivation")));
  derivation_main_box.append(&derivation_label);

  let valid_derivation_as_string: Vec<String> = VALID_BIP_DERIVATIONS
    .iter()
    .map(|&x| x.to_string())
    .filter(|s| s != "Custom")
    .collect();

  let valid_derivation_as_ref: Vec<&str> = valid_derivation_as_string
    .iter()
    .map(|s| s.as_ref())
    .collect();

  let derivation_dropdown = gtk::DropDown::from_strings(&valid_derivation_as_ref);
  derivation_main_box.append(&derivation_dropdown);
  wallet_main_content_box.append(&derivation_main_box);

  // Button
  let generate_wallet_button = gtk::Button::new();
  generate_wallet_button.set_label(&t!("UI.wallet.button"));
  wallet_main_content_box.append(&generate_wallet_button);

  // Address table
  let address_scrolled_window = gtk::ScrolledWindow::new();
  let address_store_new = gio::ListStore::new::<AddressDatabase>();
  let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
    let entry1 = obj1.downcast_ref::<AddressDatabase>().unwrap();
    let entry2 = obj2.downcast_ref::<AddressDatabase>().unwrap();

    let id1 = entry1.property::<String>("id");
    let id2 = entry2.property::<String>("id");
    let coin1 = entry1.property::<String>("coin");
    let coin2 = entry2.property::<String>("coin");

    if id1 != id2 {
      id1.cmp(&id2).into()
    } else {
      coin1.cmp(&coin2).into()
    }
  });
  let address_sorted_model = gtk::SortListModel::new(Some(address_store_new.clone()), Some(sorter));
  let address_selection_model = gtk::SingleSelection::new(Some(address_sorted_model));
  let address_treeview = gtk::ColumnView::new(Some(address_selection_model.clone()));

  address_treeview.set_show_column_separators(true);
  address_treeview.set_show_row_separators(true);
  address_treeview.set_vexpand(true);
  address_treeview.set_hexpand(true);
  address_scrolled_window.set_child(Some(&address_treeview));

  let columns = [
    &t!("UI.main.address.table.id"),
    &t!("UI.main.address.table.coin"),
    &t!("UI.main.address.table.path"),
    &t!("UI.main.address.table.address"),
    &t!("UI.main.address.table.pub"),
    &t!("UI.main.address.table.priv"),
  ];

  for (i, column_title) in columns.iter().enumerate() {
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_factory, list_item| {
      let list_item = list_item
        .downcast_ref::<gtk::ListItem>()
        .expect("Needs to be ListItem");
      let label = gtk::Label::new(None);
      list_item.set_child(Some(&label));
    });

    factory.connect_bind(move |_factory, list_item| {
      let list_item = list_item
        .downcast_ref::<gtk::ListItem>()
        .expect("Needs to be ListItem");
      let label = list_item.child().unwrap().downcast::<gtk::Label>().unwrap();
      let entry = list_item
        .item()
        .unwrap()
        .downcast::<AddressDatabase>()
        .unwrap();

      let text = match i {
        0 => entry.property::<String>("id"),
        1 => entry.property::<String>("coin"),
        2 => entry.property::<String>("path"),
        3 => entry.property::<String>("address"),
        4 => entry.property::<String>("public-key"),
        5 => entry.property::<String>("private-key"),
        _ => unreachable!(),
      };
      label.set_text(&text);
    });

    let column = gtk::ColumnViewColumn::new(Some(column_title), Some(factory));
    column.set_expand(true);

    address_treeview.append_column(&column);
  }
  wallet_main_content_box.append(&address_scrolled_window);

  wallet_main_box.append(&wallet_main_content_box);
  wallet_main_box.set_margin_top(10);
  wallet_main_box.set_margin_start(10);
  wallet_main_box.set_margin_end(10);
  wallet_main_box.set_margin_bottom(10);

  new_wallet_window.set_child(Some(&wallet_main_box));

  Ok(new_wallet_window)
}
