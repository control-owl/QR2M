// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// copyright = "Copyright © 2023-2025 Control Owl"

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
use gtk::{Stack, StackSidebar, gio, glib::clone};
use gtk4::{self as gtk};
use libadwaita as adw;
use rand::Rng;
use std::{
    fs::{self, File},
    io::{self, BufRead, Write},
    time::SystemTime,
};

mod anu;
mod coin_db;
mod dev;
mod keys;
mod os;
mod test_vectors;

#[macro_use]
extern crate rust_i18n;
i18n!("res/locale", fallback = "en");

const APP_NAME: Option<&str> = option_env!("CARGO_PKG_NAME");
const APP_DESCRIPTION: Option<&str> = option_env!("CARGO_PKG_DESCRIPTION");
const APP_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
const APP_AUTHOR: Option<&str> = option_env!("CARGO_PKG_AUTHORS");
const APP_LANGUAGE: &[&str] = &["English", "Deutsch", "Hrvatski"];
const WORDLIST_FILE: &str = "bip39-mnemonic-words-english.txt";
const VALID_ENTROPY_LENGTHS: [u32; 5] = [128, 160, 192, 224, 256];
const VALID_BIP_DERIVATIONS: [u32; 5] = [32, 44, 49, 84, 86];
const VALID_ENTROPY_SOURCES: &[&str] = &["RNG+", "File", "QRNG"];
const VALID_WALLET_PURPOSE: &[&str] = &["Internal", "External"];
const VALID_ANU_API_DATA_FORMAT: &[&str] = &["uint8", "uint16", "hex16"];
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
const GUI_IMAGE_EXTENSION: &str = "svg";
#[cfg(windows)]
const GUI_IMAGE_EXTENSION: &str = "png";

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

lazy_static::lazy_static! {
    static ref APP_SETTINGS: std::sync::Arc<std::sync::RwLock<AppSettings>> = std::sync::Arc::new(std::sync::RwLock::new(AppSettings::default()));
    static ref APP_LOG: std::sync::Arc<std::sync::Mutex<AppLog>> = std::sync::Arc::new(std::sync::Mutex::new(AppLog::new()));
    static ref WALLET_SETTINGS: std::sync::Arc<std::sync::Mutex<WalletSettings>> = std::sync::Arc::new(std::sync::Mutex::new(WalletSettings::new()));
    static ref CRYPTO_ADDRESS: std::sync::Arc<dashmap::DashMap<u32, CryptoAddresses>> = std::sync::Arc::new(dashmap::DashMap::new());
    static ref DERIVATION_PATH: std::sync::Arc<std::sync::RwLock<DerivationPath>> = std::sync::Arc::new(std::sync::RwLock::new(DerivationPath::default()));
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

struct GuiState {
    gui_language: Option<String>,
    gui_theme: Option<String>,
    gui_icon_theme: Option<String>,
    gui_log_status: Option<bool>,
    gui_main_buttons: std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, Vec<std::rc::Rc<gtk::Button>>>>>,
    gui_button_images: Option<std::collections::HashMap<String, gtk::gdk::Texture>>,
}

impl GuiState {
    fn default_config() -> Self {
        Self {
            gui_language: APP_LANGUAGE.first().map(|s| s.to_string()),
            gui_theme: VALID_GUI_THEMES.first().map(|s| s.to_string()),
            gui_icon_theme: VALID_GUI_ICONS.first().map(|s| s.to_string()),
            gui_log_status: None,
            gui_main_buttons: std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new())),
            gui_button_images: None,
        }
    }

    fn apply_language(&mut self) {
        println!("[+] {}", &t!("log.process_wallet_file_from_path").to_string());

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
            println!("\t- Icon: {:?}", icon_path);
            let texture = qr2m_lib::get_texture_from_resource(icon_path.to_str().unwrap());
            icons.insert((*name).to_string(), texture);
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
        println!("[+] {}", &t!("log.reload_gui_theme").to_string());

        if let Some(theme) = &self.gui_theme {
            let preferred_theme = match theme.as_str() {
                "Light" => adw::ColorScheme::ForceLight,
                "Dark" => adw::ColorScheme::ForceDark,
                _ => adw::ColorScheme::PreferLight,
            };
            adw::StyleManager::default().set_color_scheme(preferred_theme);
            println!("\t- GUI theme: {:?}", preferred_theme);
        } else {
            adw::StyleManager::default().set_color_scheme(adw::ColorScheme::PreferLight);
            eprintln!("\t- Problem with GUI theme, revert to default theme");
        }
    }

    fn register_button(&self, name: String, button: std::rc::Rc<gtk::Button>) {
        println!("[+] {}", &t!("log.register_button").to_string());

        let mut button_map = self.gui_main_buttons.borrow_mut();
        button_map
            .entry(name.to_string())
            .or_default()
            .push(button);

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
    gui_save_size: Option<bool>,
    gui_last_width: Option<u32>,
    gui_last_height: Option<u32>,
    gui_maximized: Option<bool>,
    gui_theme: Option<String>,
    gui_icons: Option<String>,
    gui_language: Option<String>,
    gui_search: Option<String>,
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
            gui_save_size: Some(true),
            gui_last_width: Some(1024),
            gui_last_height: Some(768),
            gui_maximized: Some(false),
            gui_theme: Some("System".to_string()),
            gui_icons: Some("Thin".to_string()),
            gui_language: Some("English".to_string()),
            gui_search: Some("Name".to_string()),
            gui_notification_timeout: Some(5),
            gui_log: Some(true),
            gui_log_level: Some("Standard".to_string()),
            anu_enabled: Some(false),
            anu_data_format: Some("uint8".to_string()),
            anu_array_length: Some(24),
            anu_hex_block_size: Some(1024),
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
    fn load_settings() {
        println!("[+] {}", &t!("log.load_settings").to_string());

        let settings = AppSettings::default();

        let local_settings = os::LOCAL_SETTINGS.lock().unwrap();
        let local_config_file = local_settings.local_config_file.clone().unwrap();

        println!("\t- Settings file: {:?}", local_config_file);

        let config_str = match fs::read_to_string(&local_config_file) {
            Ok(contents) => contents,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    println!("\t- Config file not found, using default settings.");
                    match os::check_local_config() {
                        Ok(_) => {
                            println!("\t- New config file created");
                        }
                        Err(err) => {
                            eprintln!("\t- New config file NOT created \n {}", err);
                        }
                    }
                } else {
                    eprintln!(
                        "\t- Failed to read local config file: {:?} \n Error: {:?}",
                        local_config_file, err
                    );
                }
                String::new()
            }
        };

        let config: toml::Value = config_str.parse().unwrap_or_else(|err| {
            println!("\t- {}", &t!("error.settings.config", error = err));
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
        let gui_notification_timeout = get_u32(
            &gui_section,
            "notification_timeout",
            settings.gui_notification_timeout,
        );
        let gui_log = get_bool(&gui_section, "gui_log", settings.gui_log);
        let gui_log_level = get_str(&gui_section, "log_level", settings.gui_log_level);

        println!("\t- Save last window size: {:?}", gui_save_size);
        println!("\t- GUI width: {:?}", gui_last_width);
        println!("\t- GUI height: {:?}", gui_last_height);
        println!("\t- Maximized: {:?}", gui_maximized);
        println!("\t- Theme: {:?}", gui_theme);
        println!("\t- Icons: {:?}", gui_icons);
        println!("\t- Language: {:?}", gui_language);
        println!("\t- Search: {:?}", gui_search);
        println!("\t- Notification timeout: {:?}", gui_notification_timeout);
        println!("\t- Log enabled: {:?}", gui_log);
        println!("\t- Log level: {:?}", gui_log_level);

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

        println!("\t- Entropy source: {:?}", wallet_entropy_source);
        println!("\t- Entropy length: {:?}", wallet_entropy_length);
        println!("\t- Mnemonic passphrase length: {:?}", wallet_mnemonic_length);
        println!("\t- BIP: {:?}", wallet_bip);
        println!("\t- Address count: {:?}", wallet_address_count);
        println!("\t- Hard address: {:?}", wallet_hardened_address);

        let anu_enabled = get_bool(&anu_section, "enabled", settings.anu_enabled);
        let anu_data_format = get_str(&anu_section, "data_format", settings.anu_data_format);
        let anu_array_length = get_u32(&anu_section, "array_length", settings.anu_array_length);
        let anu_hex_block_size =
            get_u32(&anu_section, "hex_block_size", settings.anu_hex_block_size);
        let anu_log = get_bool(&anu_section, "log", settings.anu_log);
        let anu_timeout = get_u32(&anu_section, "timeout", settings.anu_timeout);

        println!("\t- Use ANU: {:?}", anu_enabled);
        println!("\t- ANU data format: {:?}", anu_data_format);
        println!("\t- ANU array length: {:?}", anu_array_length);
        println!("\t- ANU hex block size: {:?}", anu_hex_block_size);
        println!("\t- ANU log: {:?}", anu_log);
        println!("\t- ANU timeout: {:?}", anu_timeout);

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

        println!("\t- Use proxy: {:?}", proxy_status);
        println!("\t- Proxy server address: {:?}", proxy_server_address);
        println!("\t- Proxy server port: {:?}", proxy_server_port);
        println!("\t- Use proxy PAC: {:?}", proxy_use_pac);
        println!("\t- Proxy script address: {:?}", proxy_script_address);
        println!("\t- Use proxy login credentials: {:?}",proxy_login_credentials);
        println!("\t- Proxy username: {:?}", proxy_login_username);
        println!("\t- Proxy password: {:?}", proxy_login_password);
        println!("\t- Use proxy SSL: {:?}", proxy_use_ssl);
        println!("\t- Proxy SSL certificate: {:?}", proxy_ssl_certificate);
        println!("\t- Proxy retry attempts: {:?}", proxy_retry_attempts);
        println!("\t- Proxy timeout: {:?}", proxy_timeout);

        let mut application_settings = APP_SETTINGS.write().unwrap();
        application_settings.wallet_entropy_source = wallet_entropy_source.clone();
        application_settings.wallet_entropy_length = wallet_entropy_length;
        application_settings.wallet_mnemonic_length = wallet_mnemonic_length;
        application_settings.wallet_bip = wallet_bip;
        application_settings.wallet_address_count = wallet_address_count;
        application_settings.wallet_hardened_address = wallet_hardened_address;

        application_settings.gui_save_size = gui_save_size;
        application_settings.gui_last_width = gui_last_width;
        application_settings.gui_last_height = gui_last_height;
        application_settings.gui_maximized = gui_maximized;
        application_settings.gui_theme = gui_theme.clone();
        application_settings.gui_icons = gui_icons.clone();
        application_settings.gui_language = gui_language.clone();
        application_settings.gui_search = gui_search.clone();
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
    }

    fn update_value(
        &mut self,
        key: &str,
        new_value: toml_edit::Item,
        gui_state: Option<std::rc::Rc<std::cell::RefCell<GuiState>>>,
    ) {
        // println!("[+] {}", &t!("log.app_settings.update_value").to_string());

        match key {
            "wallet_entropy_source" => {
                if let Some(value) = new_value.as_str() {
                    if Some(value.to_string()) != self.wallet_entropy_source {
                        self.wallet_entropy_source = Some(value.to_string());
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "wallet_entropy_length" => {
                if let Some(value) = new_value.as_integer() {
                    let value = value as u32;
                    if Some(value) != self.wallet_entropy_length {
                        self.wallet_entropy_length = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "wallet_mnemonic_length" => {
                if let Some(value) = new_value.as_integer() {
                    let value = value as u32;
                    if Some(value) != self.wallet_mnemonic_length {
                        self.wallet_mnemonic_length = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "wallet_bip" => {
                if let Some(value) = new_value.as_integer() {
                    let value = value as u32;
                    if Some(value) != self.wallet_bip {
                        self.wallet_bip = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "wallet_address_count" => {
                if let Some(value) = new_value.as_integer() {
                    let value = value as u32;
                    if Some(value) != self.wallet_address_count {
                        self.wallet_address_count = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "wallet_hardened_address" => {
                if let Some(value) = new_value.as_bool() {
                    if Some(value) != self.wallet_hardened_address {
                        self.wallet_hardened_address = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "gui_save_size" => {
                if let Some(value) = new_value.as_bool() {
                    if Some(value) != self.gui_save_size {
                        self.gui_save_size = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "gui_last_width" => {
                if let Some(value) = new_value.as_integer() {
                    let value = value as u32;
                    if Some(value) != self.gui_last_width {
                        self.gui_last_width = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "gui_last_height" => {
                if let Some(value) = new_value.as_integer() {
                    let value = value as u32;
                    if Some(value) != self.gui_last_height {
                        self.gui_last_height = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "gui_maximized" => {
                if let Some(value) = new_value.as_bool() {
                    if Some(value) != self.gui_maximized {
                        self.gui_maximized = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "gui_theme" => {
                if let Some(new_theme) = new_value.as_str() {
                    if Some(new_theme.to_string()) != self.gui_theme {
                        self.gui_theme = Some(new_theme.to_string());
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
                            println!("State in gui_theme is None");
                        }
                    }
                } else {
                    eprintln!("Received invalid value for gui_theme: {:?}", new_value);
                }
            }
            "gui_icons" => {
                if let Some(new_icons) = new_value.as_str() {
                    if Some(new_icons.to_string()) != self.gui_icons {
                        self.gui_icons = Some(new_icons.to_string());
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                        if let Some(state) = gui_state {
                            let mut state = state.borrow_mut();
                            state.gui_icon_theme = self.gui_icons.clone();
                            state.reload_gui_icons();
                        } else {
                            println!("State in gui_icons is None");
                        }
                    }
                } else {
                    eprintln!("Received invalid value for gui_icons: {:?}", new_value);
                }
            }
            "gui_language" => {
                if let Some(value) = new_value.as_str() {
                    if Some(value.to_string()) != self.gui_language {
                        self.gui_language = Some(value.to_string());
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "gui_search" => {
                if let Some(value) = new_value.as_str() {
                    if Some(value.to_string()) != self.gui_search {
                        self.gui_search = Some(value.to_string());
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "gui_notification_timeout" => {
                if let Some(value) = new_value.as_integer() {
                    let value = value as u32;
                    if Some(value) != self.gui_notification_timeout {
                        self.gui_notification_timeout = Some(value);
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
                            println!("State in gui_theme is None");
                        }

                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "gui_log_level" => {
                if let Some(value) = new_value.as_str() {
                    if Some(value.to_string()) != self.gui_log_level {
                        self.gui_log_level = Some(value.to_string());
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "anu_enabled" => {
                if let Some(value) = new_value.as_bool() {
                    if Some(value) != self.anu_enabled {
                        self.anu_enabled = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "anu_data_format" => {
                if let Some(value) = new_value.as_str() {
                    if Some(value.to_string()) != self.anu_data_format {
                        self.anu_data_format = Some(value.to_string());
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "anu_array_length" => {
                if let Some(value) = new_value.as_integer() {
                    let value = value as u32;
                    if Some(value) != self.anu_array_length {
                        self.anu_array_length = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "anu_hex_block_size" => {
                if let Some(value) = new_value.as_integer() {
                    let value = value as u32;
                    if Some(value) != self.anu_hex_block_size {
                        self.anu_hex_block_size = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "anu_log" => {
                if let Some(value) = new_value.as_bool() {
                    if Some(value) != self.anu_log {
                        self.anu_log = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "anu_timeout" => {
                if let Some(value) = new_value.as_integer() {
                    let value = value as u32;
                    if Some(value) != self.anu_timeout {
                        self.anu_timeout = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "proxy_status" => {
                if let Some(value) = new_value.as_bool() {
                    if Some(value) != self.proxy_status {
                        self.proxy_status = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "proxy_server_address" => {
                if let Some(value) = new_value.as_str() {
                    if Some(value.to_string()) != self.proxy_server_address {
                        self.proxy_server_address = Some(value.to_string());
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "proxy_server_port" => {
                if let Some(value) = new_value.as_integer() {
                    let value = value as u32;
                    if Some(value) != self.proxy_server_port {
                        self.proxy_server_port = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "proxy_use_pac" => {
                if let Some(value) = new_value.as_bool() {
                    if Some(value) != self.proxy_use_pac {
                        self.proxy_use_pac = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "proxy_script_address" => {
                if let Some(value) = new_value.as_str() {
                    if Some(value.to_string()) != self.proxy_script_address {
                        self.proxy_script_address = Some(value.to_string());
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "proxy_login_credentials" => {
                if let Some(value) = new_value.as_bool() {
                    if Some(value) != self.proxy_login_credentials {
                        self.proxy_login_credentials = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "proxy_login_username" => {
                if let Some(value) = new_value.as_str() {
                    if Some(value.to_string()) != self.proxy_login_username {
                        self.proxy_login_username = Some(value.to_string());
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "proxy_login_password" => {
                if let Some(value) = new_value.as_str() {
                    if Some(value.to_string()) != self.proxy_login_password {
                        self.proxy_login_password = Some(value.to_string());
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "proxy_use_ssl" => {
                if let Some(value) = new_value.as_bool() {
                    if Some(value) != self.proxy_use_ssl {
                        self.proxy_use_ssl = Some(value);
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            "proxy_ssl_certificate" => {
                if let Some(value) = new_value.as_str() {
                    if Some(value.to_string()) != self.proxy_ssl_certificate {
                        self.proxy_ssl_certificate = Some(value.to_string());
                        println!("\t- Updating key  {:?} = {:?}", key, new_value);
                    }
                }
            }
            _ => {}
        }
    }

    fn save_settings(&self) {
        println!("[+] {}", &t!("log.app_settings.save_settings").to_string());
        
        let local_settings = os::LOCAL_SETTINGS.lock().unwrap();
        let local_config_file = local_settings.local_config_file.clone().unwrap();

        let config_str = fs::read_to_string(&local_config_file)
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("\t- Failed to read config file: {}", e),
                )
            })
            .expect("\t- Problem with local config file");

        let mut doc = config_str
            .parse::<toml_edit::DocumentMut>()
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("\t- Failed to parse config string: {}", e),
                )
            })
            .unwrap_or(toml_edit::DocumentMut::new());

        {
            let wallet_section = doc["wallet"].or_insert(toml_edit::Item::Table(Default::default()));
            if let toml_edit::Item::Table(wallet_table) = wallet_section {
                wallet_table["entropy_source"] = toml_edit::value(self.wallet_entropy_source.clone().unwrap());
                wallet_table["entropy_length"] = toml_edit::value(self.wallet_entropy_length.unwrap() as i64);
                wallet_table["mnemonic_length"] = toml_edit::value(self.wallet_mnemonic_length.unwrap() as i64);
                wallet_table["bip"] = toml_edit::value(self.wallet_bip.unwrap() as i64);
                wallet_table["address_count"] = toml_edit::value(self.wallet_address_count.unwrap() as i64);
                wallet_table["hardened_address"] = toml_edit::value(self.wallet_hardened_address.unwrap());
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
                gui_table["notification_timeout"] = toml_edit::value(self.gui_notification_timeout.unwrap() as i64);
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
                proxy_table["server_address"] = toml_edit::value(self.proxy_server_address.clone().unwrap());
                proxy_table["server_port"] = toml_edit::value(self.proxy_server_port.unwrap() as i64);
                proxy_table["use_pac"] = toml_edit::value(self.proxy_use_pac.unwrap());
                proxy_table["script_address"] = toml_edit::value(self.proxy_script_address.clone().unwrap());
                proxy_table["login_credentials"] = toml_edit::value(self.proxy_login_credentials.unwrap());
                proxy_table["login_username"] = toml_edit::value(self.proxy_login_username.clone().unwrap());
                proxy_table["login_password"] = toml_edit::value(self.proxy_login_password.clone().unwrap());
                proxy_table["use_ssl"] = toml_edit::value(self.proxy_use_ssl.unwrap());
                proxy_table["ssl_certificate"] = toml_edit::value(self.proxy_ssl_certificate.clone().unwrap());
            }
        }

        let toml_str = doc.to_string();

        let mut file = fs::File::create(&local_config_file)
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("\t- Failed to create config file: {}", e),
                )
            })
            .expect("\t- Problem with local config file");

        file.write_all(toml_str.as_bytes())
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("\t- Failed to write to config file: {}", e),
                )
            })
            .expect("\t- Can not write to local config file");
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
    master_xprv: Option<String>,
    master_xpub: Option<String>,
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
            master_xprv: None,
            master_xpub: None,
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
            message_queue: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::VecDeque::new(),
            )),
            processing: std::sync::Arc::new(std::sync::Mutex::new(false)),
        }
    }

    fn queue_message(&self, new_message: String, message_type: gtk::MessageType) {
        println!("[+] {}", &t!("log.app_messages.queue_message").to_string());

        let mut queue = self.message_queue.lock().unwrap();
        let last_message_in_queue = queue.get(queue.len().wrapping_sub(1));

        let some_message = match last_message_in_queue {
            Some(message) => message,
            None => &("".to_string(), gtk::MessageType::Info),
        };

        let last_message = some_message.0.clone();

        if new_message != last_message {
            queue.push_back((new_message, message_type));

            if !*self.processing.lock().unwrap() {
                self.start_message_processor();
            }
        }
    }

    fn start_message_processor(&self) {
        println!(
            "[+] {}",
            &t!("log.app_messages.start_message_processor").to_string()
        );

        let info_bar = match &self.gui_info_bar {
            Some(info_bar) => info_bar.clone(),
            None => {
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
                let mut queue_lock = queue.lock().unwrap();
                if let Some((message, message_type)) = queue_lock.pop_front() {
                    AppMessages::create_info_message(&info_bar, &message, message_type);

                    let lock_app_settings = APP_SETTINGS.read().unwrap();
                    let timeout = lock_app_settings.gui_notification_timeout.unwrap();

                    glib::timeout_add_local(std::time::Duration::from_secs(timeout as u64), {
                        let queue = queue.clone();
                        let info_bar = info_bar.clone();
                        let processing = processing.clone();

                        move || {
                            info_bar.set_reveal_child(false);

                            AppMessages::start_next_message(
                                &queue,
                                &info_bar,
                                &processing,
                                timeout,
                            );

                            glib::ControlFlow::Break
                        }
                    });

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

    fn create_info_message(
        revealer: &gtk::Revealer,
        message: &str,
        message_type: gtk::MessageType,
    ) {
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

        let close_button = gtk::Button::with_label(&t!("UI.element.button.close"));
        let gesture = gtk::GestureClick::new();

        gesture.connect_pressed(clone!(
            #[weak] revealer,
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
        gui_state: std::rc::Rc<std::cell::RefCell<GuiState>>,
    ) {
        let status = self.status.clone();
        let is_active = status.lock().unwrap();
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
                eprintln!("\t- Error: 'notif' texture not found in gui_button_images");
            }
        } else {
            eprintln!("\t- Error: gui_button_images is None");
        }

        println!("\t- Icon changed. Logging starts...");

        // IMPLEMENT: Show log messages
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

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

#[tokio::main]
async fn main() {
    let start_time = std::time::Instant::now();

    print_program_info();

    os::detect_os_and_user_dir();

    if let Err(err) = os::check_local_config() {
        eprintln!("\t- Error creating local config files: {}", err);
    } else {
        println!("\t- Config file ready");
    }

    AppSettings::load_settings();

    let application = adw::Application::builder()
        .application_id("wtf.r_o0_t.qr2m")
        .build();

    let gui_state = std::rc::Rc::new(std::cell::RefCell::new(GuiState::default_config()));

    application.connect_activate(clone!(
        #[strong] gui_state,
        move |app| {
            create_main_window(app.clone(), gui_state.clone(), Some(start_time));
        }
    ));

    application.run();
}

fn print_program_info() {
    let current_time = SystemTime::now();
    let timestamp = current_time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    println!(" ██████╗ ██████╗ ██████╗ ███╗   ███╗");
    println!("██╔═══██╗██╔══██╗╚════██╗████╗ ████║");
    println!("██║   ██║██████╔╝ █████╔╝██╔████╔██║");
    println!("██║▄▄ ██║██╔══██╗██╔═══╝ ██║╚██╔╝██║");
    println!("╚██████╔╝██║  ██║███████╗██║ ╚═╝ ██║");
    println!(" ╚══▀▀═╝ ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝");

    println!("{} {}", &APP_DESCRIPTION.unwrap(), &APP_VERSION.unwrap());
    println!("Start time (UNIX): {:?}", &timestamp.to_string());
    println!("-.-. --- .--. -.-- .-. .. --. .... - --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.");
}

fn setup_app_actions(
    application: adw::Application,
    gui_state: std::rc::Rc<std::cell::RefCell<GuiState>>,
    app_messages_state: std::rc::Rc<std::cell::RefCell<AppMessages>>,
) {
    println!("[+] {}", &t!("log.setup_app_actions").to_string());

    let new = gio::SimpleAction::new("new", None);
    let open = gio::SimpleAction::new("open", None);
    let save = gio::SimpleAction::new("save", None);
    let about = gio::SimpleAction::new("about", None);
    let settings = gio::SimpleAction::new("settings", None);
    let quit = gio::SimpleAction::new("quit", None);
    let log = gio::SimpleAction::new("log", None);
    let test = gio::SimpleAction::new("test", None);

    new.connect_activate(clone!(
        #[strong] application,
        #[strong] gui_state,
        move |_action, _parameter| {
            create_main_window(application.clone(), gui_state.clone(), None);
        }
    ));

    open.connect_activate(clone!(
        #[weak] app_messages_state,
        move |_action, _parameter| {
            open_wallet_from_file(&app_messages_state);
        }
    ));

    save.connect_activate(|_action, _parameter| {
        save_wallet_to_file();
    });

    about.connect_activate(move |_action, _parameter| {
        create_about_window();
    });

    log.connect_activate(clone!(
        #[strong] gui_state,
        move |_action, _parameter| {
            let log_window = create_log_window(gui_state.clone());
            log_window.present()
        }
    ));

    settings.connect_activate(clone!(
        #[strong] gui_state,
        #[weak] app_messages_state,
        move |_action, _parameter| {
            let settings_window = create_settings_window(gui_state.clone(), app_messages_state);
            settings_window.present();
        }
    ));

    quit.connect_activate(clone!(
        #[weak] application,
        move |_action, _parameter| {
            application.quit();
        }
    ));

    test.connect_activate(clone!(
        // #[strong] gui_state,
        // #[weak] app_messages_state,
        move |_action, _parameter| {
            let anu_window = dev::anu_window();
            anu_window.present();
        }
    ));

    application.set_accels_for_action("app.new", &["<Primary>N"]);
    application.set_accels_for_action("app.open", &["<Primary>O"]);
    application.set_accels_for_action("app.save", &["<Primary>S"]);
    application.set_accels_for_action("app.about", &["F1"]);
    application.set_accels_for_action("app.settings", &["F5"]);
    application.set_accels_for_action("app.quit", &["<Primary>Q"]);
    application.set_accels_for_action("app.test", &["<Primary>T"]);

    application.add_action(&new);
    application.add_action(&open);
    application.add_action(&save);
    application.add_action(&about);
    application.add_action(&settings);
    application.add_action(&quit);
    application.add_action(&test);
}

fn create_main_window(
    application: adw::Application,
    gui_state: std::rc::Rc<std::cell::RefCell<GuiState>>,
    start_time: Option<std::time::Instant>,
) {
    println!("[+] {}", &t!("log.create_main_window").to_string());

    let window = gtk::ApplicationWindow::builder()
        .application(&application)
        .title(format!(
            "{} {}",
            APP_DESCRIPTION.unwrap(),
            APP_VERSION.unwrap()
        ))
        .show_menubar(true)
        .decorated(true)
        .build();

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
    
    os::switch_locale(&gui_language);

    qr2m_lib::setup_css();

    let header_bar = gtk::HeaderBar::new();
    let info_bar = gtk::Revealer::new();
    info_bar.set_transition_type(gtk::RevealerTransitionType::SlideDown);
    info_bar.set_hexpand(true);
    info_bar.add_css_class("info-bar");
    window.set_titlebar(Some(&header_bar));

    let button_names = ["new", "open", "save", "about", "settings", "log", "random"];
    let mut buttons = std::collections::HashMap::new();

    for &name in &button_names {
        let button = gtk::Button::new();
        buttons.insert(name.to_string(), std::rc::Rc::new(button));
    }

    let button_tooltips = [
        ("new", "Ctrl+N"),
        ("open", "Ctrl+O"),
        ("save", "Ctrl+S"),
        ("about", "F1"),
        ("settings", "F5"),
        ("log", "F11"),
        ("random", ""),
    ];

    for (name, shortcut) in button_tooltips {
        if let Some(button) = buttons.get(name) {
            button.set_tooltip_text(Some(
                &t!(format!("UI.main.tooltips.{}", name), value = shortcut),
            ));
        }
    }

    let gui_theme = lock_app_settings.gui_theme.clone().unwrap();
    let gui_icons = lock_app_settings.gui_icons.clone().unwrap();
    let app_log_status = lock_app_settings.gui_log.unwrap();

    {
        let mut lock_gui_state = gui_state.borrow_mut();
        lock_gui_state.gui_language = Some(gui_language);
        lock_gui_state.gui_theme = Some(gui_theme);
        lock_gui_state.gui_icon_theme = Some(gui_icons);
        lock_gui_state.gui_log_status = Some(app_log_status);
    
        for (name, button) in &buttons {
            lock_gui_state.register_button(name.clone(), button.clone());
        }
    
        lock_gui_state.reload_gui_theme();
        lock_gui_state.reload_gui_icons();
    }

    {
        let settings = gtk::Settings::default().expect("Failed to get GtkSettings");
        settings.connect_gtk_application_prefer_dark_theme_notify(clone!(
            #[strong] gui_state,
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

    let app_messages_state = std::rc::Rc::new(std::cell::RefCell::new(AppMessages::new(Some(
        info_bar.clone(),
    ))));

    setup_app_actions(
        application.clone(),
        gui_state.clone(),
        app_messages_state.clone(),
    );

    header_bar.pack_start(&*buttons["new"]);
    header_bar.pack_start(&*buttons["open"]);
    header_bar.pack_start(&*buttons["save"]);
    header_bar.pack_end(&*buttons["settings"]);
    header_bar.pack_end(&*buttons["about"]);
    header_bar.pack_end(&*buttons["log"]);

    // JUMP: Action: Settings button action
    buttons["settings"].connect_clicked(clone!(
        #[strong] gui_state,
        #[strong] app_messages_state,
        move |_| {
            let settings_window =
                create_settings_window(gui_state.clone(), app_messages_state.clone());
            settings_window.show();
        }
    ));

    buttons["about"].connect_clicked(move |_| {
        create_about_window();
    });

    buttons["log"].connect_clicked(clone!(
        #[strong] gui_state,
        move |_| {
            let log_window = create_log_window(gui_state.clone());
            log_window.show();
        }
    ));

    buttons["new"].connect_clicked(clone!(
        #[strong] application,
        #[strong] gui_state,
        move |_| {
            create_main_window(application.clone(), gui_state.clone(), None);
        }
    ));

    buttons["save"].connect_clicked(move |_| {
        save_wallet_to_file();
    });

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

    let anu_enabled = lock_app_settings.anu_enabled.unwrap_or(false);
    let valid_entropy_sources: Vec<&str> = if anu_enabled {
        VALID_ENTROPY_SOURCES.to_vec()
    } else {
        VALID_ENTROPY_SOURCES
            .iter()
            .filter(|&&x| x != "QRNG")
            .cloned()
            .collect()
    };

    let valid_entropy_source_as_strings: Vec<String> = valid_entropy_sources
        .iter()
        .map(|&x| x.to_string())
        .collect();

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
        8.0 * 2.0,
        8.0 * 128.0 * 4.0,
        1.0,
        100.0,
        0.0
    );

    let mnemonic_passphrase_scale = gtk::Scale::new(gtk::Orientation::Horizontal,Some(&mnemonic_passphrase_adjustment));
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

    let generate_entropy_button = gtk::Button::new();
    generate_entropy_button.set_width_request(200);
    generate_entropy_button.set_label(&t!("UI.main.seed.generate"));

    let delete_entropy_button = gtk::Button::new();
    delete_entropy_button.set_width_request(200);
    delete_entropy_button.set_label(&t!("UI.main.seed.delete"));

    let entropy_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let entropy_frame = gtk::Frame::new(Some(&t!("UI.main.seed.entropy")));
    let entropy_text = gtk::TextView::new();
    entropy_text.set_vexpand(true);
    entropy_text.set_hexpand(true);
    entropy_text.set_wrap_mode(gtk::WrapMode::Char);
    entropy_text.set_editable(false);
    entropy_text.set_left_margin(5);
    entropy_text.set_top_margin(5);
    entropy_frame.set_child(Some(&entropy_text));
    entropy_box.append(&entropy_frame);

    let mnemonic_words_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let mnemonic_words_frame = gtk::Frame::new(Some(&t!("UI.main.seed.mnemonic.words")));
    let mnemonic_words_text = gtk::TextView::new();
    mnemonic_words_text.set_vexpand(true);
    mnemonic_words_text.set_hexpand(true);
    mnemonic_words_text.set_wrap_mode(gtk::WrapMode::Word);
    mnemonic_words_text.set_editable(false);
    mnemonic_words_text.set_left_margin(5);
    mnemonic_words_text.set_top_margin(5);
    mnemonic_words_frame.set_child(Some(&mnemonic_words_text));
    mnemonic_words_box.append(&mnemonic_words_frame);

    let seed_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let seed_frame = gtk::Frame::new(Some(&t!("UI.main.seed")));
    let seed_text = gtk::TextView::new();
    seed_text.set_vexpand(true);
    seed_text.set_hexpand(true);
    seed_text.set_wrap_mode(gtk::WrapMode::Char);
    seed_text.set_editable(false);
    seed_text.set_left_margin(5);
    seed_text.set_top_margin(5);
    seed_frame.set_child(Some(&seed_text));
    seed_box.append(&seed_frame);
    

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
    sidebar_seed_header_mnemonic_options.append(&mnemonic_passphrase_length_box);
    






    sidebar_seed_header_box.append(&sidebar_seed_header_entropy_options);
    sidebar_seed_header_box.append(&sidebar_seed_header_mnemonic_options);
    
    sidebar_seed_button_box.append(&generate_entropy_button);
    sidebar_seed_button_box.append(&delete_entropy_button);

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
    let filter_verified_coins_button = gtk::Button::with_label(
        &t!(
            "UI.main.coin.filter.status.verified",
            value = coin_db::COIN_STATUS_VERIFIED
        )
    );
    filter_verified_coins_button_box.append(&filter_verified_coins_button);
    coin_filter_content_box.append(&filter_verified_coins_button_box);
    filter_verified_coins_button_box.set_hexpand(true);

    let filter_not_verified_coins_button_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let filter_not_verified_coins_button = gtk::Button::with_label(
        &t!(
            "UI.main.coin.filter.status.not_verified",
            value = coin_db::COIN_STATUS_NOT_VERIFIED
        )
    );
    filter_not_verified_coins_button_box.append(&filter_not_verified_coins_button);
    coin_filter_content_box.append(&filter_not_verified_coins_button_box);
    filter_not_verified_coins_button_box.set_hexpand(true);

    let filter_in_plan_coins_button_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let filter_in_plan_coins_button = gtk::Button::with_label(
        &t!(
            "UI.main.coin.filter.status.future",
            value = coin_db::COIN_STATUS_IN_PLAN
        )
    );
    filter_in_plan_coins_button_box.append(&filter_in_plan_coins_button);
    coin_filter_content_box.append(&filter_in_plan_coins_button_box);
    filter_in_plan_coins_button_box.set_hexpand(true);

    let filter_not_supported_coins_button_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let filter_not_supported_coins_button = gtk::Button::with_label(
        &t!(
            "UI.main.coin.filter.status.not_supported",
            value = coin_db::COIN_STATUS_NOT_SUPPORTED
        )
    );
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
    coin_search.set_placeholder_text(Some(
        &t!(
            "UI.main.coin.search.text",
            value = valid_coin_search_filter_as_strings[default_coin_search_filter]
        )
    ));

    // Coin treeview
    let scrolled_window = gtk::ScrolledWindow::new();
    let coin_frame = gtk::Frame::new(Some(&t!("UI.main.coin")));

    coin_db::create_coin_completion_model();

    let coin_store = coin_db::create_coin_store();
    let coin_store = std::rc::Rc::new(std::cell::RefCell::new(coin_store));
    let coin_tree_store = gtk4::TreeStore::new(&[glib::Type::STRING; 14]);
    let coin_tree_store = std::rc::Rc::new(std::cell::RefCell::new(coin_tree_store));
    let coin_treeview = gtk::TreeView::new();
    let coin_treeview = std::rc::Rc::new(std::cell::RefCell::new(coin_treeview));

    coin_treeview.borrow().set_vexpand(true);
    coin_treeview.borrow().set_headers_visible(true);

    let columns = [
        &t!("UI.main.database.column.status").to_string(),
        &t!("UI.main.database.column.index").to_string(),
        &t!("UI.main.database.column.symbol").to_string(),
        &t!("UI.main.database.column.coin").to_string(),
        &t!("UI.main.database.column.key_derivation").to_string(),
        &t!("UI.main.database.column.hash").to_string(),
        &t!("UI.main.database.column.priv_header").to_string(),
        &t!("UI.main.database.column.pub_header").to_string(),
        &t!("UI.main.database.column.pub_hash").to_string(),
        &t!("UI.main.database.column.script").to_string(),
        &t!("UI.main.database.column.wif").to_string(),
        &t!("UI.main.database.column.evm").to_string(),
        &t!("UI.main.database.column.UCID").to_string(),
        &t!("UI.main.database.column.cmc").to_string(),
    ];

    for (i, column_title) in columns.iter().enumerate() {
        let column = gtk::TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();

        column.set_title(column_title);
        column.pack_start(&cell, true);
        column.add_attribute(&cell, "text", i as i32);
        coin_treeview.borrow().append_column(&column);
    }

    let full_store = coin_store.borrow();
    let verified_coins = coin_db::fetch_coins_from_database("Cmc_top", &full_store, "100");
    let filtered_store = coin_tree_store.borrow_mut();

    for found_coin in verified_coins {
        // Check if the coin's status is verified
        if found_coin.status == "Verified" {
            let iter = filtered_store.append(None);

            filtered_store.set(
                &iter,
                &[
                    (0, &found_coin.status),
                    (1, &found_coin.coin_index.to_string()),
                    (2, &found_coin.coin_symbol),
                    (3, &found_coin.coin_name),
                    (4, &found_coin.key_derivation),
                    (5, &found_coin.hash),
                    (6, &found_coin.private_header),
                    (7, &found_coin.public_header),
                    (8, &found_coin.public_key_hash),
                    (9, &found_coin.script_hash),
                    (10, &found_coin.wallet_import_format),
                    (11, &found_coin.evm),
                    (12, &found_coin.ucid),
                    (13, &found_coin.cmc_top),
                ],
            );
        }
    }

    coin_treeview.borrow().set_model(Some(&*filtered_store));
    scrolled_window.set_child(Some(&*coin_treeview.borrow()));
    coin_frame.set_child(Some(&scrolled_window));
    coin_main_content_box.append(&coin_frame);

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
    let master_xpub_frame = gtk::Frame::new(Some(&t!("UI.main.coin.keys.pub")));
    let master_private_key_text = gtk::TextView::new();
    let master_public_key_text = gtk::TextView::new();

    master_private_key_text.set_editable(false);
    master_public_key_text.set_editable(false);
    master_private_key_text.set_wrap_mode(gtk::WrapMode::Char);
    master_private_key_text.set_editable(false);
    master_private_key_text.set_left_margin(5);
    master_private_key_text.set_top_margin(5);
    master_public_key_text.set_wrap_mode(gtk::WrapMode::Char);
    master_public_key_text.set_editable(false);
    master_public_key_text.set_left_margin(5);
    master_public_key_text.set_top_margin(5);
    master_xprv_frame.set_child(Some(&master_private_key_text));
    master_xpub_frame.set_child(Some(&master_public_key_text));
    master_keys_box.append(&master_xprv_frame);
    master_keys_box.append(&master_xpub_frame);
    coin_main_content_box.append(&master_keys_box);

    stack.add_titled(
        &coin_main_box,
        Some("sidebar-coin"),
        &t!("UI.main.coin"),
    );

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
    let main_address_frame =
        gtk::Frame::new(Some(&t!("UI.main.address.derivation.address")));
    let main_purpose_frame =
        gtk::Frame::new(Some(&t!("UI.main.address.derivation.purpose")));

    main_bip_frame.set_hexpand(true);
    main_coin_frame.set_hexpand(true);
    main_address_frame.set_hexpand(true);
    main_purpose_frame.set_hexpand(true);

    let bip_hardened_frame =
        gtk::Frame::new(Some(&t!("UI.main.address.derivation.hard")));
    let coin_hardened_frame =
        gtk::Frame::new(Some(&t!("UI.main.address.derivation.hard")));
    let address_hardened_frame =
        gtk::Frame::new(Some(&t!("UI.main.address.derivation.hard")));

    let valid_bip_as_string: Vec<String> = VALID_BIP_DERIVATIONS
        .iter()
        .map(|&x| x.to_string())
        .collect();
    let valid_bip_as_ref: Vec<&str> = valid_bip_as_string.iter().map(|s| s.as_ref()).collect();
    let bip_dropdown = gtk::DropDown::from_strings(&valid_bip_as_ref);

    let wallet_bip = lock_app_settings.wallet_bip.unwrap();
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
    let derivation_label_frame =
        gtk::Frame::new(Some(&t!("UI.main.address.derivation")));
    derivation_label_frame.set_hexpand(true);

    let default_bip_label = if wallet_bip == 32 {
        main_purpose_frame.set_visible(false);
        format!("m/{}'/0'/0'", wallet_bip)
    } else {
        main_purpose_frame.set_visible(true);
        format!("m/{}'/0'/0'/0", wallet_bip)
    };

    let derivation_label_text = gtk4::Label::builder()
        .label(&default_bip_label)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .css_classes(["large-title"])
        .build();

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
        )>,
    ));
    let stop_addresses_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let stop_address_generation_button = gtk::Button::with_label(&t!("UI.main.address.generate.stop"));
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

    let address_store = gtk::ListStore::new(&[
        gtk4::glib::Type::STRING, // Coin
        gtk4::glib::Type::STRING, // Derivation Path
        gtk4::glib::Type::STRING, // Address
        gtk4::glib::Type::STRING, // Public Key
        gtk4::glib::Type::STRING, // Private Key
    ]);

    let sorted_model = gtk::TreeModelSort::with_model(&address_store);
    sorted_model.set_sort_column_id(gtk4::SortColumn::Index(0), gtk::SortType::Ascending);
    sorted_model.set_sort_column_id(gtk4::SortColumn::Index(1), gtk::SortType::Ascending);

    let address_treeview = gtk::TreeView::new();
    address_treeview.set_model(Some(&sorted_model));

    address_treeview.set_headers_visible(true);
    let columns = [
        &t!("UI.main.address.table.coin"),
        &t!("UI.main.address.table.path"),
        &t!("UI.main.address.table.address"),
        &t!("UI.main.address.table.pub"),
        &t!("UI.main.address.table.priv"),
    ];

    for (i, column_title) in columns.iter().enumerate() {
        let column = gtk::TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();
        column.set_title(column_title);
        column.pack_start(&cell, true);
        column.add_attribute(&cell, "text", i as i32);
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
    let address_options_hardened_address_frame = gtk::Frame::new(Some(&t!("UI.main.address.options.hardened")));
    let address_options_hardened_address_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let address_options_hardened_address_checkbox = gtk::CheckButton::new();
    let wallet_hardened_address = lock_app_settings.wallet_hardened_address;

    address_options_hardened_address_checkbox.set_active(wallet_hardened_address.unwrap());
    address_options_hardened_address_box.set_halign(gtk4::Align::Center);
    address_options_hardened_address_frame.set_child(Some(&address_options_hardened_address_box));
    address_options_hardened_address_box.append(&address_options_hardened_address_checkbox);

    // Address progress box
    let address_generation_progress_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let address_generation_progress_bar = gtk::ProgressBar::new();
    address_generation_progress_bar.set_hexpand(true);
    address_generation_progress_box.append(&address_generation_progress_bar);

    stop_address_generation_button.connect_clicked(clone!(
        #[strong] generator_handler,
        #[weak] delete_addresses_button_box,
        #[weak] stop_addresses_button_box,
        move |_| {
            if let Some((handle, cancel_tx)) = generator_handler.lock().unwrap().take() {
                cancel_tx.send(true).ok();
                handle.abort();
                println!("Address generation aborted");
                delete_addresses_button_box.set_visible(true);
                stop_addresses_button_box.set_visible(false);
            } else {
                eprintln!("No handle!");
            }
        }
    ));

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
        #[strong] app_messages_state,
        #[weak] entropy_text,
        #[weak] mnemonic_passphrase_text,
        #[weak] mnemonic_words_text,
        #[weak] seed_text,
        move |_| {
            let (entropy, passphrase) = open_wallet_from_file(&app_messages_state);

            if !entropy.is_empty() {
                println!(" ↳ Wallet entropy: {:?}", entropy);
                entropy_text.buffer().set_text(&entropy);

                match passphrase {
                    Some(pass) => {
                        println!("\t- Mnemonic passphrase: {:?}", pass);
                        mnemonic_passphrase_text.buffer().set_text(&pass);
                    }
                    None => {
                        println!("\t- No Mnemonic passphrase available");
                    }
                }

                let buffer = entropy_text.buffer();
                let start_iter = buffer.start_iter();
                let end_iter = buffer.end_iter();
                let full_entropy = buffer.text(&start_iter, &end_iter, false);

                if full_entropy != "" {
                    let mnemonic_words = keys::generate_mnemonic_words(&full_entropy);
                    mnemonic_words_text.buffer().set_text(&mnemonic_words);

                    let (entropy_len, _checksum_len) = match full_entropy.len() {
                        132 => (128, 4),
                        165 => (160, 5),
                        198 => (192, 6),
                        231 => (224, 7),
                        264 => (256, 8),
                        _ => (0, 0),
                    };

                    let (pre_entropy, _checksum) = full_entropy.split_at(entropy_len);

                    let seed = keys::generate_bip39_seed(
                        pre_entropy,
                        &mnemonic_passphrase_text.buffer().text(),
                    );
                    let seed_hex = hex::encode(&seed[..]);
                    seed_text.buffer().set_text(&seed_hex.to_string());

                    println!("\t- Seed (hex): {:?}", seed_hex);
                }
            }
        }
    ));

    // JUMP: Action: Generate Seed button
    generate_entropy_button.connect_clicked(clone!(
        #[strong] app_messages_state,
        #[weak] entropy_source_dropdown,
        #[weak] entropy_text,
        #[weak] entropy_length_dropdown,
        #[weak] mnemonic_words_text,
        #[weak] mnemonic_passphrase_text,
        #[weak] master_private_key_text,
        #[weak] master_public_key_text,
        #[weak] seed_text,
        move |_| {
            let selected_entropy_source_index = entropy_source_dropdown.selected() as usize;
            let selected_entropy_length_index = entropy_length_dropdown.selected() as usize;
            let selected_entropy_source_value =
                VALID_ENTROPY_SOURCES.get(selected_entropy_source_index);
            let selected_entropy_length_value =
                VALID_ENTROPY_LENGTHS.get(selected_entropy_length_index);
            let source = selected_entropy_source_value.unwrap().to_string();
            let entropy_length = selected_entropy_length_value.unwrap();

            let pre_entropy = keys::generate_entropy(&source, *entropy_length as u64);

            if !pre_entropy.is_empty() {
                let checksum =
                    qr2m_lib::calculate_checksum_for_entropy(&pre_entropy, entropy_length);
                println!("\t- Entropy checksum: {:?}", checksum);

                let full_entropy = format!("{}{}", &pre_entropy, &checksum);

                println!("\t- Final entropy: {:?}", full_entropy);
                entropy_text.buffer().set_text(&full_entropy);

                let mnemonic_words = keys::generate_mnemonic_words(&full_entropy);
                mnemonic_words_text.buffer().set_text(&mnemonic_words);

                let passphrase_text = mnemonic_passphrase_text.text().to_string();

                let seed = keys::generate_bip39_seed(&pre_entropy, &passphrase_text);
                let seed_hex = hex::encode(&seed[..]);
                seed_text.buffer().set_text(&seed_hex.to_string());

                println!("\t- Seed (hex): {:?}", seed_hex);

                let mut wallet_settings = WALLET_SETTINGS.lock().unwrap();
                wallet_settings.entropy_checksum = Some(checksum.clone());
                wallet_settings.entropy_string = Some(full_entropy.clone());
                wallet_settings.mnemonic_passphrase = Some(passphrase_text.clone());
                wallet_settings.mnemonic_words = Some(mnemonic_words.clone());
                wallet_settings.seed = Some(seed_hex.clone());

                master_private_key_text.buffer().set_text("");
                master_public_key_text.buffer().set_text("");
            } else {
                eprintln!("\t- {}", &t!("error.entropy.empty"));
                let lock_app_messages = app_messages_state.borrow();
                lock_app_messages.queue_message(
                    t!("error.entropy.empty").to_string(),
                    gtk::MessageType::Warning,
                );
            }
        }
    ));

    delete_entropy_button.connect_clicked(clone!(
        #[weak] entropy_text,
        #[weak] mnemonic_words_text,
        #[weak] mnemonic_passphrase_text,
        #[weak] seed_text,
        #[weak] master_private_key_text,
        #[weak] master_public_key_text,
        #[weak] address_store,
        move |_| {
            mnemonic_passphrase_text.buffer().set_text("");
            entropy_text.buffer().set_text("");
            mnemonic_words_text.buffer().set_text("");
            seed_text.buffer().set_text("");
            master_private_key_text.buffer().set_text("");
            master_public_key_text.buffer().set_text("");
            address_store.clear();
        }
    ));

    buttons["random"].connect_clicked(clone!(
        #[weak] mnemonic_passphrase_text,
        #[weak] mnemonic_passphrase_scale,
        move |_| {
            let scale_value = mnemonic_passphrase_scale.value() as u32;

            let mnemonic_rng_string: String = (0..scale_value)
                .map(|_| char::from(rand::rng().random_range(32..127)))
                .collect();
            println!("\t- RNG Mnemonic Passphrase: {:?}", mnemonic_rng_string);
            mnemonic_passphrase_text.set_text(&mnemonic_rng_string);
        }
    ));

    delete_master_keys_button.connect_clicked(clone!(
        #[weak] master_private_key_text,
        #[weak] master_public_key_text,
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
        #[strong] coin_entry,
        #[strong] app_messages_state,
        #[weak] seed_text,
        #[weak] coin_treeview,
        #[weak] master_private_key_text,
        #[weak] master_public_key_text,
        move |_| {
            let buffer = seed_text.buffer();
            let start_iter = buffer.start_iter();
            let end_iter = buffer.end_iter();
            let text = buffer.text(&start_iter, &end_iter, false);

            if !text.is_empty() {
                if let Some((model, iter)) = coin_treeview.borrow().selection().selected() {
                    let status = model.get_value(&iter, 0);
                    let coin_index = model.get_value(&iter, 1);
                    let coin_symbol = model.get_value(&iter, 2);
                    let coin_name = model.get_value(&iter, 3);
                    let key_derivation = model.get_value(&iter, 4);
                    let hash = model.get_value(&iter, 5);
                    let private_header = model.get_value(&iter, 6);
                    let public_header = model.get_value(&iter, 7);
                    let public_key_hash = model.get_value(&iter, 8);
                    let script_hash = model.get_value(&iter, 9);
                    let wallet_import_format = model.get_value(&iter, 10);
                    let evm = model.get_value(&iter, 11);
                    let ucid = model.get_value(&iter, 12);
                    let cmc_top = model.get_value(&iter, 13);

                    if let (
                        Ok(status),
                        Ok(coin_index),
                        Ok(coin_symbol),
                        Ok(coin_name),
                        Ok(key_derivation),
                        Ok(hash),
                        Ok(private_header),
                        Ok(public_header),
                        Ok(public_key_hash),
                        Ok(script_hash),
                        Ok(wallet_import_format),
                        Ok(evm),
                        Ok(ucid),
                        Ok(cmc_top),
                    ) = (
                        status.get::<String>(),
                        coin_index.get::<String>(),
                        coin_symbol.get::<String>(),
                        coin_name.get::<String>(),
                        key_derivation.get::<String>(),
                        hash.get::<String>(),
                        private_header.get::<String>(),
                        public_header.get::<String>(),
                        public_key_hash.get::<String>(),
                        script_hash.get::<String>(),
                        wallet_import_format.get::<String>(),
                        evm.get::<String>(),
                        ucid.get::<String>(),
                        cmc_top.get::<String>(),
                    ) {
                        master_private_key_text.buffer().set_text("");
                        master_public_key_text.buffer().set_text("");

                        println!("\n#### Coin info ####");
                        println!("\t- status: {}", status);
                        println!("\t- index: {}", coin_index);
                        println!("\t- coin_symbol: {}", coin_symbol);
                        println!("\t- coin_name: {}", coin_name);
                        println!("\t- key_derivation: {}", key_derivation);
                        println!("\t- hash: {}", hash);
                        println!("\t- private_header: {}", private_header);
                        println!("\t- public_header: {}", public_header);
                        println!("\t- public_key_hash: {}", public_key_hash);
                        println!("\t- script_hash: {}", script_hash);
                        println!("\t- wallet_import_format: {}", wallet_import_format);
                        println!("\t- EVM: {}", evm);
                        println!("\t- UCID: {}", ucid);
                        println!("\t- cmc_top: {}", cmc_top);

                        let buffer = seed_text.buffer();
                        let start_iter = buffer.start_iter();
                        let end_iter = buffer.end_iter();
                        let seed_string = buffer.text(&start_iter, &end_iter, true);

                        match keys::generate_master_keys(
                            &seed_string,
                            &private_header,
                            &public_header,
                        ) {
                            Ok(xprv) => {
                                master_private_key_text.buffer().set_text(&xprv.0);
                                master_public_key_text.buffer().set_text(&xprv.1);
                            }
                            Err(err) => {
                                {
                                    let lock_gui_state = app_messages_state.borrow();
                                    lock_gui_state.queue_message(
                                        t!("error.master.create").to_string(),
                                        gtk::MessageType::Warning,
                                    );
                                }
                                eprintln!("\t- {}: {}", &t!("error.master.create"), err)
                            }
                        }

                        coin_entry.set_text(&coin_index);

                        let mut wallet_settings = WALLET_SETTINGS.lock().unwrap();
                        wallet_settings.public_key_hash = Some(public_key_hash.clone());
                        wallet_settings.wallet_import_format =
                            Some(wallet_import_format.to_string());
                        wallet_settings.key_derivation = Some(key_derivation.to_string());
                        wallet_settings.hash = Some(hash.to_string());
                        wallet_settings.coin_index = Some(coin_index.parse().unwrap());
                        wallet_settings.coin_name = Some(coin_name.parse().unwrap());
                    }
                }
            } else {
                {
                    let app_messages_state = app_messages_state.borrow();
                    app_messages_state.queue_message(
                        t!("error.entropy.seed").to_string(),
                        gtk::MessageType::Warning,
                    );
                }
                // let lock_state = gui_state.lock().unwrap();
                // lock_state.show_message(t!("error.entropy.seed").to_string(), gtk::MessageType::Warning);

                // {
                //     if let Ok(mut log_lock) = app_log.lock() {
                //         log_lock.initialize_app_log(log_button.clone(), resources.clone());
                //     }
                // }
            }
        }
    ));

    entropy_source_dropdown.connect_selected_notify(clone!(
        #[weak] generate_entropy_button,
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
                generate_entropy_button.set_label(&t!("UI.main.seed.generate.file"));
            } else {
                generate_entropy_button.set_label(&t!("UI.main.seed.generate"));
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
        #[weak] coin_search,
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
        #[weak] entropy_text,
        #[weak] mnemonic_words_text,
        #[weak] seed_text,
        move |mnemonic_passphrase_text| {
            let entropy_buffer = entropy_text.buffer();
            let start_iter = entropy_buffer.start_iter();
            let end_iter = entropy_buffer.end_iter();
            let entropy_text = entropy_buffer.text(&start_iter, &end_iter, false);

            if entropy_text != "" {
                let entropy_length = entropy_text.len();
                let cut_entropy = entropy_length / 32;
                let new_pre_entropy = entropy_text[0..entropy_length - cut_entropy].to_string();

                let seed = keys::generate_bip39_seed(
                    &new_pre_entropy,
                    &mnemonic_passphrase_text.buffer().text(),
                );
                let seed_hex = hex::encode(&seed[..]);
                seed_text.buffer().set_text(&seed_hex.to_string());

                let final_entropy = entropy_text.clone().to_string();
                let mnemonic_words_buffer = mnemonic_words_text.buffer();
                let start_iter = mnemonic_words_buffer.start_iter();
                let end_iter = mnemonic_words_buffer.end_iter();
                let final_mnemonic_words = mnemonic_words_buffer
                    .text(&start_iter, &end_iter, false)
                    .to_string();
                let final_mnemonic_passphrase =
                    mnemonic_passphrase_text.buffer().text().to_string();

                let mut wallet_settings = WALLET_SETTINGS.lock().unwrap();
                wallet_settings.entropy_string = Some(final_entropy);
                wallet_settings.mnemonic_words = Some(final_mnemonic_words);
                wallet_settings.mnemonic_passphrase = Some(final_mnemonic_passphrase);
                wallet_settings.seed = Some(seed_hex.clone());
            }
        }
    ));

    mnemonic_passphrase_scale.connect_value_changed(clone!(
        #[weak] mnemonic_passphrase_length_info,
        #[weak(rename_to = random_mnemonic_passphrase_button)]
        buttons["random"],
        move |mnemonic_passphrase_scale| {
            let scale_value = mnemonic_passphrase_scale.value() as u32;
            mnemonic_passphrase_length_info.set_text(&scale_value.to_string());
            random_mnemonic_passphrase_button.emit_by_name::<()>("clicked", &[]);
        }
    ));

    coin_search.connect_search_changed({
        let coin_tree_store = std::rc::Rc::clone(&coin_tree_store);
        let coin_store = std::rc::Rc::clone(&coin_store);
        let coin_treeview = std::rc::Rc::clone(&coin_treeview);

        move |coin_search| {
            let search_text = coin_search.text().to_lowercase();
            coin_tree_store.borrow_mut().clear();

            let selected = coin_search_filter_dropdown.selected() as usize;
            let selected_search_parameter =
                VALID_COIN_SEARCH_PARAMETER.get(selected).unwrap_or(&"Name");
            let min_search_length = if selected_search_parameter == &"Index" {
                1
            } else {
                2
            };

            if search_text.len() >= min_search_length {
                let store = coin_store.borrow();
                let matching_coins = coin_db::fetch_coins_from_database(
                    selected_search_parameter,
                    &store,
                    &search_text,
                );

                if !matching_coins.is_empty() {
                    let store = coin_tree_store.borrow_mut();
                    store.clear();

                    for found_coin in matching_coins {
                        let iter = store.append(None);
                        store.set(
                            &iter,
                            &[
                                (0, &found_coin.status),
                                (1, &found_coin.coin_index.to_string()),
                                (2, &found_coin.coin_symbol),
                                (3, &found_coin.coin_name),
                                (4, &found_coin.key_derivation),
                                (5, &found_coin.hash),
                                (6, &found_coin.private_header),
                                (7, &found_coin.public_header),
                                (8, &found_coin.public_key_hash),
                                (9, &found_coin.script_hash),
                                (10, &found_coin.wallet_import_format),
                                (11, &found_coin.evm),
                                (12, &found_coin.ucid),
                                (13, &found_coin.cmc_top),
                            ],
                        );
                    }
                    coin_treeview.borrow().set_model(Some(&*store));
                } else {
                    coin_tree_store.borrow_mut().clear();
                }
            } else {
                coin_tree_store.borrow_mut().clear();
            }
        }
    });

    filter_top10_coins_button.connect_clicked({
        let coin_tree_store = std::rc::Rc::clone(&coin_tree_store);
        let coin_store = std::rc::Rc::clone(&coin_store);
        let coin_treeview = std::rc::Rc::clone(&coin_treeview);

        move |_| {
            let search_text = "10";
            let search_parameter = "Cmc_top";
            let store = coin_store.borrow();
            let matching_coins =
                coin_db::fetch_coins_from_database(search_parameter, &store, search_text);

            let store = coin_tree_store.borrow_mut();
            store.clear();

            if !matching_coins.is_empty() {
                for found_coin in matching_coins {
                    let iter = store.append(None);
                    store.set(
                        &iter,
                        &[
                            (0, &found_coin.status),
                            (1, &found_coin.coin_index.to_string()),
                            (2, &found_coin.coin_symbol),
                            (3, &found_coin.coin_name),
                            (4, &found_coin.key_derivation),
                            (5, &found_coin.hash),
                            (6, &found_coin.private_header),
                            (7, &found_coin.public_header),
                            (8, &found_coin.public_key_hash),
                            (9, &found_coin.script_hash),
                            (10, &found_coin.wallet_import_format),
                            (11, &found_coin.evm),
                            (12, &found_coin.ucid),
                            (13, &found_coin.cmc_top),
                        ],
                    );
                }
                coin_treeview.borrow().set_model(Some(&*store));
            } else {
                store.clear();
            }
        }
    });

    filter_top100_coins_button.connect_clicked({
        let coin_tree_store = std::rc::Rc::clone(&coin_tree_store);
        let coin_store = std::rc::Rc::clone(&coin_store);
        let coin_treeview = std::rc::Rc::clone(&coin_treeview);

        move |_| {
            let search_text = "100";
            let search_parameter = "Cmc_top";
            let store = coin_store.borrow();
            let matching_coins =
                coin_db::fetch_coins_from_database(search_parameter, &store, search_text);

            let store = coin_tree_store.borrow_mut();
            store.clear();

            if !matching_coins.is_empty() {
                for found_coin in matching_coins {
                    let iter = store.append(None);
                    store.set(
                        &iter,
                        &[
                            (0, &found_coin.status),
                            (1, &found_coin.coin_index.to_string()),
                            (2, &found_coin.coin_symbol),
                            (3, &found_coin.coin_name),
                            (4, &found_coin.key_derivation),
                            (5, &found_coin.hash),
                            (6, &found_coin.private_header),
                            (7, &found_coin.public_header),
                            (8, &found_coin.public_key_hash),
                            (9, &found_coin.script_hash),
                            (10, &found_coin.wallet_import_format),
                            (11, &found_coin.evm),
                            (12, &found_coin.ucid),
                            (13, &found_coin.cmc_top),
                        ],
                    );
                }
                coin_treeview.borrow().set_model(Some(&*store));
            } else {
                store.clear();
            }
        }
    });

    filter_verified_coins_button.connect_clicked({
        let coin_tree_store = std::rc::Rc::clone(&coin_tree_store);
        let coin_store = std::rc::Rc::clone(&coin_store);
        let coin_treeview = std::rc::Rc::clone(&coin_treeview);

        move |_| {
            let search_text = coin_db::VALID_COIN_STATUS_NAME[1];
            let search_parameter = "Status";
            let store = coin_store.borrow();
            let matching_coins =
                coin_db::fetch_coins_from_database(search_parameter, &store, search_text);

            let store = coin_tree_store.borrow_mut();
            store.clear();

            if !matching_coins.is_empty() {
                for found_coin in matching_coins {
                    let iter = store.append(None);
                    store.set(
                        &iter,
                        &[
                            (0, &found_coin.status),
                            (1, &found_coin.coin_index.to_string()),
                            (2, &found_coin.coin_symbol),
                            (3, &found_coin.coin_name),
                            (4, &found_coin.key_derivation),
                            (5, &found_coin.hash),
                            (6, &found_coin.private_header),
                            (7, &found_coin.public_header),
                            (8, &found_coin.public_key_hash),
                            (9, &found_coin.script_hash),
                            (10, &found_coin.wallet_import_format),
                            (11, &found_coin.evm),
                            (12, &found_coin.ucid),
                            (13, &found_coin.cmc_top),
                        ],
                    );
                }
                coin_treeview.borrow().set_model(Some(&*store));
            } else {
                store.clear();
            }
        }
    });

    filter_not_verified_coins_button.connect_clicked({
        let coin_tree_store = std::rc::Rc::clone(&coin_tree_store);
        let coin_store = std::rc::Rc::clone(&coin_store);
        let coin_treeview = std::rc::Rc::clone(&coin_treeview);

        move |_| {
            let search_text = coin_db::VALID_COIN_STATUS_NAME[2];
            let search_parameter = "Status";
            let store = coin_store.borrow();
            let matching_coins =
                coin_db::fetch_coins_from_database(search_parameter, &store, search_text);

            let store = coin_tree_store.borrow_mut();
            store.clear();

            if !matching_coins.is_empty() {
                for found_coin in matching_coins {
                    let iter = store.append(None);
                    store.set(
                        &iter,
                        &[
                            (0, &found_coin.status),
                            (1, &found_coin.coin_index.to_string()),
                            (2, &found_coin.coin_symbol),
                            (3, &found_coin.coin_name),
                            (4, &found_coin.key_derivation),
                            (5, &found_coin.hash),
                            (6, &found_coin.private_header),
                            (7, &found_coin.public_header),
                            (8, &found_coin.public_key_hash),
                            (9, &found_coin.script_hash),
                            (10, &found_coin.wallet_import_format),
                            (11, &found_coin.evm),
                            (12, &found_coin.ucid),
                            (13, &found_coin.cmc_top),
                        ],
                    );
                }
                coin_treeview.borrow().set_model(Some(&*store));
            } else {
                store.clear();
            }
        }
    });

    filter_in_plan_coins_button.connect_clicked({
        let coin_tree_store = std::rc::Rc::clone(&coin_tree_store);
        let coin_store = std::rc::Rc::clone(&coin_store);
        let coin_treeview = std::rc::Rc::clone(&coin_treeview);

        move |_| {
            let search_text = coin_db::VALID_COIN_STATUS_NAME[3];
            let search_parameter = "Status";
            let store = coin_store.borrow();
            let matching_coins =
                coin_db::fetch_coins_from_database(search_parameter, &store, search_text);

            let store = coin_tree_store.borrow_mut();
            store.clear();

            if !matching_coins.is_empty() {
                for found_coin in matching_coins {
                    let iter = store.append(None);
                    store.set(
                        &iter,
                        &[
                            (0, &found_coin.status),
                            (1, &found_coin.coin_index.to_string()),
                            (2, &found_coin.coin_symbol),
                            (3, &found_coin.coin_name),
                            (4, &found_coin.key_derivation),
                            (5, &found_coin.hash),
                            (6, &found_coin.private_header),
                            (7, &found_coin.public_header),
                            (8, &found_coin.public_key_hash),
                            (9, &found_coin.script_hash),
                            (10, &found_coin.wallet_import_format),
                            (11, &found_coin.evm),
                            (12, &found_coin.ucid),
                            (13, &found_coin.cmc_top),
                        ],
                    );
                }
                coin_treeview.borrow().set_model(Some(&*store));
            } else {
                store.clear();
            }
        }
    });

    filter_not_supported_coins_button.connect_clicked({
        let coin_tree_store = std::rc::Rc::clone(&coin_tree_store);
        let coin_store = std::rc::Rc::clone(&coin_store);
        let coin_treeview = std::rc::Rc::clone(&coin_treeview);

        move |_| {
            let search_text = coin_db::VALID_COIN_STATUS_NAME[0];
            let search_parameter = "Status";
            let store = coin_store.borrow();
            let matching_coins =
                coin_db::fetch_coins_from_database(search_parameter, &store, search_text);

            let store = coin_tree_store.borrow_mut();
            store.clear();

            if !matching_coins.is_empty() {
                for found_coin in matching_coins {
                    let iter = store.append(None);
                    store.set(
                        &iter,
                        &[
                            (0, &found_coin.status),
                            (1, &found_coin.coin_index.to_string()),
                            (2, &found_coin.coin_symbol),
                            (3, &found_coin.coin_name),
                            (4, &found_coin.key_derivation),
                            (5, &found_coin.hash),
                            (6, &found_coin.private_header),
                            (7, &found_coin.public_header),
                            (8, &found_coin.public_key_hash),
                            (9, &found_coin.script_hash),
                            (10, &found_coin.wallet_import_format),
                            (11, &found_coin.evm),
                            (12, &found_coin.ucid),
                            (13, &found_coin.cmc_top),
                        ],
                    );
                }
                coin_treeview.borrow().set_model(Some(&*store));
            } else {
                store.clear();
            }
        }
    });

    bip_dropdown.connect_selected_notify(clone!(
        #[weak] derivation_label_text,
        move |bip_dropdown| {
            let value = bip_dropdown.selected() as usize;
            let selected_entropy_source_value = VALID_BIP_DERIVATIONS.get(value);
            let bip = selected_entropy_source_value.unwrap();

            if *bip == 32 {
                main_purpose_frame.set_visible(false);
                bip_hardened_frame.set_visible(false);
            } else {
                main_purpose_frame.set_visible(true);
                bip_hardened_frame.set_visible(true);
            }

            let mut dp = DERIVATION_PATH.write().unwrap();
            dp.update_field("bip", Some(FieldValue::U32(*bip)));
            update_derivation_label(*dp, derivation_label_text)
        }
    ));

    bip_hardened_checkbox.connect_active_notify(clone!(
        #[weak] derivation_label_text,
        #[weak] bip_hardened_checkbox,
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
        #[weak] derivation_label_text,
        #[weak] coin_hardened_checkbox,
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
        #[weak] derivation_label_text,
        #[weak] address_hardened_checkbox,
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
        #[weak] derivation_label_text,
        #[weak] purpose_dropdown,
        move |_| {
            let purpose = purpose_dropdown.selected();
            let mut dp = DERIVATION_PATH.write().unwrap();

            dp.update_field("purpose", Some(FieldValue::U32(purpose)));
            update_derivation_label(*dp, derivation_label_text);
        }
    ));

    coin_entry.connect_changed(clone!(
        #[weak] derivation_label_text,
        #[strong] coin_entry,
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
        #[weak] derivation_label_text,
        #[weak] address_spinbutton,
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
        #[weak] address_start_spinbutton,
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

    // JUMP: Generate Addresses button
    // // Working version
    generate_addresses_button.connect_clicked(clone!(
        #[strong] address_store,
        #[strong] stop_addresses_button_box,
        #[strong] generator_handler,
        #[strong] app_messages_state,
        #[weak] derivation_label_text,
        #[weak] master_private_key_text,
        #[weak] address_start_spinbutton,
        #[weak] address_count_spinbutton,
        #[weak] address_options_hardened_address_checkbox,
        #[weak] address_generation_progress_bar,
        #[weak] delete_addresses_button_box,
        move |_| {
            let buffer = master_private_key_text.buffer();
            let start_iter = buffer.start_iter();
            let end_iter = buffer.end_iter();
            let master_private_key_string = buffer.text(&start_iter, &end_iter, true);

            if master_private_key_string.is_empty() {
                let lock_app_messages = app_messages_state.borrow();
                lock_app_messages.queue_message(t!("error.address.master").to_string(), gtk::MessageType::Warning);
                return;
            }

            let wallet_settings = {
                let lock = WALLET_SETTINGS.lock().unwrap();
                lock.clone()
            };

            address_generation_progress_bar.set_fraction(0.0);
            address_generation_progress_bar.set_show_text(true);
            stop_addresses_button_box.set_visible(true);
            delete_addresses_button_box.set_visible(false);

            let coin_name = wallet_settings.coin_name.clone().unwrap_or_default();
            let derivation_path = derivation_label_text.text();
            let hardened_address = address_options_hardened_address_checkbox.is_active();
            let address_start_point = address_start_spinbutton.text();
            let address_start_point_int = address_start_point.parse::<usize>().unwrap_or(0);
            let address_count = address_count_spinbutton.text();
            let address_count_int = address_count.parse::<usize>().unwrap_or(1);
            
            let (tx, rx) = std::sync::mpsc::channel();
            let (tp, rp) = std::sync::mpsc::channel();
            let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);


            let cpu_threads = num_cpus::get();
            dbg!(cpu_threads);
            let addresses_per_thread = address_count_int / cpu_threads;
            let extra_addresses = address_count_int % cpu_threads;
            let existing_addresses: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<String>>> = std::sync::Arc::new(
                std::sync::Mutex::new(
                    CRYPTO_ADDRESS
                        .iter()
                        .filter_map(|addr| addr.derivation_path.clone())
                        .collect()
                )
            );

            let generated_addresses = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let progress_status = std::sync::Arc::new(std::sync::Mutex::new(0.0));
            let start_time = std::time::Instant::now();

            let address_loop = tokio::spawn(async move {
                let mut handles = vec![];
                let mut current_start = address_start_point_int;
                let cancel_rx = std::sync::Arc::new(tokio::sync::Mutex::new(cancel_rx));
    
                for thread_id in 0..cpu_threads {
                    let num_addresses = if thread_id < extra_addresses {
                        addresses_per_thread + 1
                    } else {
                        addresses_per_thread
                    };
    
                    if num_addresses == 0 {
                        continue;
                    }
    
                    let tx = tx.clone();
                    let tp = tp.clone();
                    let existing_addresses = existing_addresses.clone();
                    let cancel_rx = cancel_rx.clone();
                    let wallet_settings = wallet_settings.clone();
                    let derivation_path = derivation_path.clone();
                    let coin_name = coin_name.clone();
                    let start_index = current_start;
                    let generated_addresses = generated_addresses.clone();
                    let progress_status = progress_status.clone();

                    let handle = tokio::spawn(async move {
                        let mut generated_count = 0;
                        let mut current_index = start_index;
                        let total_to_generate = num_addresses;
    
                        while generated_count < total_to_generate {
                            let cancel_rx = cancel_rx.lock().await;
                            if *cancel_rx.borrow() {
                                println!("Address generation aborted (thread {})", thread_id);
                                let _ = tp.send(1.0);
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
    
                            {
                                let existing = existing_addresses.lock().unwrap();
                                if existing.contains(&derivation_path) {
                                    current_index += 1;
                                    continue;
                                }
                            }
    
                            let magic_ingredients = keys::AddressHocusPokus {
                                coin_index: wallet_settings.coin_index.unwrap_or_default(),
                                derivation_path: derivation_path.clone(),
                                master_private_key_bytes: wallet_settings.master_private_key_bytes.clone().unwrap_or_default(),
                                master_chain_code_bytes: wallet_settings.master_chain_code_bytes.clone().unwrap_or_default(),
                                public_key_hash: wallet_settings.public_key_hash.clone().unwrap_or_default(),
                                key_derivation: wallet_settings.key_derivation.clone().unwrap_or_default(),
                                wallet_import_format: wallet_settings.wallet_import_format.clone().unwrap_or_default(),
                                hash: wallet_settings.hash.clone().unwrap_or_default(),
                            };
    
                            if let Ok((address, public_key, private_key)) = keys::generate_address(magic_ingredients) {
                                {
                                    let mut existing = existing_addresses.lock().unwrap();
                                    existing.insert(derivation_path.clone());
                                }
    
                                let new_entry = CryptoAddresses {
                                    coin_name: Some(coin_name.clone()),
                                    derivation_path: Some(derivation_path.clone()),
                                    address: Some(address.clone()),
                                    public_key: Some(public_key.clone()),
                                    private_key: Some(private_key.clone()),
                                };
    
                                CRYPTO_ADDRESS.insert(current_index as u32, new_entry.clone());
    
                                if tx.send(new_entry).is_err() {
                                    break;
                                }
    
                                generated_count += 1;
                                let current_total = generated_addresses.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;

                                let new_progress = if address_count_int > 0 {
                                    (current_total as f64) / (address_count_int as f64)
                                } else {
                                    0.0
                                };
    
                                let mut last = progress_status.lock().unwrap();
                                if new_progress > *last + 0.01 || new_progress >= 1.0 {
                                    *last = new_progress;
                                    let _ = tp.send(new_progress);
                                }
                            }
                            current_index += 1;
                        }
                    });
    
                    handles.push(handle);
                    current_start += num_addresses;
                }
    
                for handle in handles {
                    handle.await.unwrap();
                }
                let _ = tp.send(1.0);
            });

            *generator_handler.lock().unwrap() = Some((address_loop, cancel_tx));

            glib::idle_add_local(clone!(
                #[strong] address_store,
                #[strong] app_messages_state,
                #[strong] address_generation_progress_bar,
                #[strong] stop_addresses_button_box,
                #[strong] delete_addresses_button_box,
                move || {
                    while let Ok(new_entry) = rx.try_recv() {
                        let iter = address_store.append();
                        address_store.set(
                            &iter,
                            &[
                                (0, &new_entry.coin_name.clone().unwrap_or_default()),
                                (1, &new_entry.derivation_path.clone().unwrap_or_default()),
                                (2, &new_entry.address.clone().unwrap_or_default()),
                                (3, &new_entry.public_key.clone().unwrap_or_default()),
                                (4, &new_entry.private_key.clone().unwrap_or_default()),
                            ],
                        );
                    }
            
                    while let Ok(progress) = rp.try_recv() {
                        address_generation_progress_bar.set_fraction(progress);
            
                        if progress >= 1.0 {
                            {
                                let duration = start_time.elapsed();
                                let message = format!("Address generation completed in {:.2?}", duration);

                                println!("{}", message);

                                let lock_app_messages = app_messages_state.borrow();
                                lock_app_messages.queue_message(message.to_string(), gtk::MessageType::Info);

                                stop_addresses_button_box.set_visible(false);
                                delete_addresses_button_box.set_visible(true);
                                
                            }
                            return glib::ControlFlow::Break;
                        }
                    }
            
                    glib::ControlFlow::Continue
                }
            ));
            
        }
    ));

    delete_addresses_button.connect_clicked(clone!(
        #[weak] address_store,
        #[weak] address_start_spinbutton,
        #[weak] address_generation_progress_bar,
        #[strong] delete_addresses_button_box,
        move |_| {
            address_store.clear();
            CRYPTO_ADDRESS.clear();
            address_start_spinbutton.set_text("0");
            address_generation_progress_bar.set_show_text(false);
            address_generation_progress_bar.set_fraction(0.0);
            delete_addresses_button_box.set_visible(false);
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
        lock_app_messages.queue_message(t!("hello").to_string(), gtk::MessageType::Info);
    }

    window.set_child(Some(&main_window_box));

    window.connect_close_request(clone!(
        #[strong] window,
        move |_| {
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
                
                glib::Propagation::Proceed
            } else {
                glib::Propagation::Proceed
            }
        }
    ));

    window.present();

    if let Some(value) = start_time {
        let elapsed = value.elapsed();
        println!("Application startup time: {:.2?}", elapsed);
    };
}

fn create_log_window(
    gui_state: std::rc::Rc<std::cell::RefCell<GuiState>>,
    // resources: std::sync::Arc<std::sync::Mutex<GuiResources>>,
    // log: std::sync::Arc<std::sync::Mutex<AppLog>>,
) -> gtk::ApplicationWindow {
    println!("[+] {}", &t!("log.create_log_window").to_string());

    let log_window = gtk::ApplicationWindow::builder()
        .title(t!("UI.main.log").to_string())
        // .default_width(WINDOW_SETTINGS_DEFAULT_WIDTH.try_into().unwrap())
        // .default_height(WINDOW_SETTINGS_DEFAULT_HEIGHT.try_into().unwrap())
        .resizable(true)
        .modal(false)
        .build();

    let lock_gui_state = gui_state.borrow_mut();
    let new_log_button = std::rc::Rc::new(gtk::Button::new());
    lock_gui_state.register_button("log".to_string(), new_log_button);


    log_window
}

fn create_settings_window(
    gui_state: std::rc::Rc<std::cell::RefCell<GuiState>>,
    app_messages_state: std::rc::Rc<std::cell::RefCell<AppMessages>>,
) -> gtk::ApplicationWindow {
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
    let default_gui_theme_color_label =
        gtk::Label::new(Some(&t!("UI.settings.general.theme")));
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
    let default_gui_icons_label =
        gtk::Label::new(Some(&t!("UI.settings.general.icons")));
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
    let default_gui_language_label =
        gtk::Label::new(Some(&t!("UI.settings.general.language")));
    let valid_gui_languages_as_strings: Vec<String> =
        APP_LANGUAGE.iter().map(|&x| x.to_string()).collect();
    let valid_gui_languages_as_str_refs: Vec<&str> = valid_gui_languages_as_strings
        .iter()
        .map(|s| s.as_ref())
        .collect();
    let default_gui_language_dropdown =
        gtk::DropDown::from_strings(&valid_gui_languages_as_str_refs);
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
    let save_window_size_label =
        gtk::Label::new(Some(&t!("UI.settings.general.save_window")));
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
    let default_search_parameter_label =
        gtk::Label::new(Some(&t!("UI.settings.general.search")));
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

    // Notification timeout
    let notification_timeout_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let notification_timeout_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let notification_timeout_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let notification_timeout_label = gtk::Label::new(Some(
        &t!("UI.settings.wallet.notification_timeout"),
    ));
    let notification_timeout = lock_app_settings.gui_notification_timeout.unwrap() as f64;
    let notification_timeout_adjustment =
        gtk::Adjustment::new(notification_timeout, 1.0, 120.0, 1.0, 10.0, 0.0);
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
    content_general_box.append(&notification_timeout_box);

    

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
    let default_gui_log_level_label =
        gtk::Label::new(Some(&t!("UI.settings.general.log_level")));
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
    let qrng_enabled = lock_app_settings.anu_enabled.unwrap();
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
    let default_entropy_source_label = gtk::Label::new(Some(&t!("UI.settings.wallet.entropy.source")));
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
    let mnemonic_length_label =
        gtk::Label::new(Some(&t!("UI.settings.wallet.mnemonic_length")));
    let mnemonic_length = lock_app_settings.wallet_mnemonic_length.unwrap() as f64;
    let mnemonic_length_adjustment =
        gtk::Adjustment::new(mnemonic_length, 8.0 * 2.0, 8.0 * 128.0, 1.0, 100.0, 0.0);
    let mnemonic_length_spinbutton =
        gtk::SpinButton::new(Some(&mnemonic_length_adjustment), 1.0, 0);

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

    // Default BIP
    let default_bip_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let default_bip_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_bip_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_bip_label = gtk::Label::new(Some(&t!("UI.settings.wallet.bip")));
    let valid_bips_as_strings: Vec<String> = VALID_BIP_DERIVATIONS
        .iter()
        .map(|&x| x.to_string())
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
    let default_address_count_label =
        gtk::Label::new(Some(&t!("UI.settings.wallet.address_count")));
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
    let hardened_addresses_label =
        gtk::Label::new(Some(&t!("UI.settings.wallet.hardened")));
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
    let use_anu_api_checkbox = gtk::CheckButton::new();
    let is_checked = lock_app_settings.anu_enabled.unwrap();

    use_anu_api_checkbox.set_active(is_checked);
    use_anu_api_label_box.set_hexpand(true);
    use_anu_api_item_box.set_hexpand(true);
    use_anu_api_item_box.set_margin_end(20);
    use_anu_api_item_box.set_halign(gtk::Align::End);

    use_anu_api_label_box.append(&use_anu_api_label);
    use_anu_api_item_box.append(&use_anu_api_checkbox);
    use_anu_api_box.append(&use_anu_api_label_box);
    use_anu_api_box.append(&use_anu_api_item_box);
    content_anu_box.append(&use_anu_api_box);

    // Log ANU QRNG API
    let log_anu_api_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
    let log_anu_api_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let log_anu_api_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let log_anu_api_label = gtk::Label::new(Some(&t!("UI.settings.anu.log")));
    let log_anu_api_checkbox = gtk::CheckButton::new();

    log_anu_api_checkbox.set_active(lock_app_settings.anu_log.unwrap());
    log_anu_api_label_box.set_hexpand(true);
    log_anu_api_item_box.set_hexpand(true);
    log_anu_api_item_box.set_margin_end(20);
    log_anu_api_item_box.set_halign(gtk::Align::End);

    log_anu_api_label_box.append(&log_anu_api_label);
    log_anu_api_item_box.append(&log_anu_api_checkbox);
    log_anu_api_box.append(&log_anu_api_label_box);
    log_anu_api_box.append(&log_anu_api_item_box);
    content_anu_box.append(&log_anu_api_box);

    // ANU API data type
    let default_api_data_format_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let default_api_data_format_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_api_data_format_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_api_data_format_label =
        gtk::Label::new(Some(&t!("UI.settings.anu.data.type")));
    let valid_api_data_formats_as_strings: Vec<String> = VALID_ANU_API_DATA_FORMAT
        .iter()
        .map(|&x| x.to_string())
        .collect();
    let valid_api_data_formats_as_str_refs: Vec<&str> = valid_api_data_formats_as_strings
        .iter()
        .map(|s| s.as_ref())
        .collect();
    let anu_data_format_dropdown = gtk::DropDown::from_strings(&valid_api_data_formats_as_str_refs);
    let default_api_data_format = valid_api_data_formats_as_strings
        .iter()
        .position(|x| {
            x.parse::<String>().unwrap() == lock_app_settings.anu_data_format.clone().unwrap()
        })
        .unwrap_or(0);

    anu_data_format_dropdown.set_selected(default_api_data_format.try_into().unwrap());
    anu_data_format_dropdown.set_size_request(200, 10);
    default_api_data_format_box.set_hexpand(true);
    default_api_data_format_item_box.set_hexpand(true);
    default_api_data_format_item_box.set_margin_end(20);
    default_api_data_format_item_box.set_halign(gtk::Align::End);

    default_api_data_format_label_box.append(&default_api_data_format_label);
    default_api_data_format_item_box.append(&anu_data_format_dropdown);
    default_api_data_format_box.append(&default_api_data_format_label_box);
    default_api_data_format_box.append(&default_api_data_format_item_box);
    content_anu_box.append(&default_api_data_format_box);

    // ANU array length
    let default_anu_array_length_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let default_anu_array_length_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_anu_array_length_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_anu_array_length_label =
        gtk::Label::new(Some(&t!("UI.settings.anu.data.array")));
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
    let default_anu_array_length_spinbutton =
        gtk::SpinButton::new(Some(&array_length_adjustment), 1.0, 0);

    default_anu_array_length_label_box.set_hexpand(true);
    default_anu_array_length_item_box.set_hexpand(true);
    default_anu_array_length_item_box.set_margin_end(20);
    default_anu_array_length_item_box.set_halign(gtk::Align::End);
    default_anu_array_length_spinbutton.set_size_request(200, 10);

    default_anu_array_length_label_box.append(&default_anu_array_length_label);
    default_anu_array_length_item_box.append(&default_anu_array_length_spinbutton);
    default_anu_array_length_box.append(&default_anu_array_length_label_box);
    default_anu_array_length_box.append(&default_anu_array_length_item_box);
    content_anu_box.append(&default_anu_array_length_box);

    // ANU hex block size
    let default_anu_hex_length_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let default_anu_hex_length_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_anu_hex_length_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_anu_hex_length_label =
        gtk::Label::new(Some(&t!("UI.settings.anu.data.hex")));

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
    let default_anu_hex_length_spinbutton =
        gtk::SpinButton::new(Some(&hex_block_size_adjustment), 1.0, 0);

    default_anu_hex_length_label_box.set_hexpand(true);
    default_anu_hex_length_item_box.set_hexpand(true);
    default_anu_hex_length_item_box.set_margin_end(20);
    default_anu_hex_length_item_box.set_halign(gtk::Align::End);
    default_anu_hex_length_spinbutton.set_size_request(200, 10);

    default_anu_hex_length_label_box.append(&default_anu_hex_length_label);
    default_anu_hex_length_item_box.append(&default_anu_hex_length_spinbutton);
    default_anu_hex_length_box.append(&default_anu_hex_length_label_box);
    default_anu_hex_length_box.append(&default_anu_hex_length_item_box);
    content_anu_box.append(&default_anu_hex_length_box);

    if anu_data_format_dropdown.selected() == 2 {
        default_anu_hex_length_box.set_visible(true);
    } else {
        default_anu_hex_length_box.set_visible(false);
    };

    if use_anu_api_checkbox.is_active() {
        default_api_data_format_box.set_visible(true);
        log_anu_api_box.set_visible(true);
        default_anu_array_length_box.set_visible(true);
        if anu_data_format_dropdown.selected() as usize == 2 {
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
    let anu_connection_timeout_label =
        gtk::Label::new(Some(&t!("UI.settings.anu.timeout")));

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
    let anu_connection_timeout_spinbutton =
        gtk::SpinButton::new(Some(&anu_connection_timeout_adjustment), 1.0, 0);

    anu_connection_timeout_spinbutton.set_size_request(200, 10);
    anu_connection_timeout_label_box.set_hexpand(true);
    anu_connection_timeout_item_box.set_hexpand(true);
    anu_connection_timeout_item_box.set_margin_end(20);
    anu_connection_timeout_item_box.set_halign(gtk::Align::End);

    anu_connection_timeout_label_box.append(&anu_connection_timeout_label);
    anu_connection_timeout_item_box.append(&anu_connection_timeout_spinbutton);
    anu_connection_timeout_box.append(&anu_connection_timeout_label_box);
    anu_connection_timeout_box.append(&anu_connection_timeout_item_box);
    content_anu_box.append(&anu_connection_timeout_box);

    // Actions
    let default_anu_hex_length_box_clone = default_anu_hex_length_box.clone();
    let anu_data_format_dropdown_clone = anu_data_format_dropdown.clone();

    use_anu_api_checkbox.connect_toggled(move |checkbox| {
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

    anu_data_format_dropdown.connect_selected_notify(clone!(
        #[weak]
        default_anu_hex_length_box,
        // #[weak] anu_data_format_dropdown,
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
    // scrolled_window.set_margin_bottom(10);
    scrolled_window.set_child(Some(&proxy_settings_box));

    // Use proxy settings
    let use_proxy_settings_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
    let use_proxy_settings_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let use_proxy_settings_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let use_proxy_settings_label = gtk::Label::new(Some(&t!("UI.settings.proxy.use")));
    // let valid_proxy_settings_as_strings: Vec<String> = VALID_PROXY_STATUS.iter().map(|&x| x.to_string()).collect();
    // let valid_proxy_settings_as_str_refs: Vec<&str> = valid_proxy_settings_as_strings.iter().map(|s| s.as_ref()).collect();
    let use_proxy_settings_checkbox = gtk::CheckButton::new();

    let proxy_status = lock_app_settings.proxy_status.unwrap();

    use_proxy_settings_checkbox.set_active(proxy_status);

    // let default_proxy_settings_format = valid_proxy_settings_as_strings
    //     .iter()
    //     .position(|x| x.parse::<String>().unwrap() == proxy_status.clone())
    //     .unwrap_or(1);  // Default proxy: auto

    // use_proxy_settings_dropdown.set_selected(default_proxy_settings_format.try_into().unwrap());
    // use_proxy_settings_dropdown.set_size_request(200, 10);
    use_proxy_settings_label_box.set_hexpand(true);
    use_proxy_settings_item_box.set_hexpand(true);
    use_proxy_settings_item_box.set_margin_end(20);
    use_proxy_settings_item_box.set_halign(gtk::Align::End);

    use_proxy_settings_label_box.append(&use_proxy_settings_label);
    use_proxy_settings_item_box.append(&use_proxy_settings_checkbox);
    use_proxy_settings_box.append(&use_proxy_settings_label_box);
    use_proxy_settings_box.append(&use_proxy_settings_item_box);
    content_proxy_box.append(&use_proxy_settings_box);

    // Proxy manual settings
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
    let proxy_server_address_label =
        gtk::Label::new(Some(&t!("UI.settings.proxy.address")));
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
    let use_proxy_credentials_label =
        gtk::Label::new(Some(&t!("UI.settings.proxy.creds")));
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

    // Actions
    use_proxy_settings_checkbox.connect_active_notify(clone!(
        #[weak]
        proxy_manual_settings_box,
        move |dd| {
            let proxy_status = dd.is_active();
            // let selected_proxy_settings_value = VALID_PROXY_STATUS.get(value);
            // let settings = value;

            if proxy_status {
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

    let save_button = gtk::Button::with_label(&t!("UI.element.button.save"));
    let cancel_button = gtk::Button::with_label(&t!("UI.element.button.cancel"));
    let default_button = gtk::Button::with_label(&t!("UI.element.button.default"));

    // JUMP: Save settings button
    save_button.connect_clicked(clone!(
        #[weak] settings_window,
        #[strong] gui_state,
        #[weak] app_messages_state,
        move |_| {
            let mut settings = APP_SETTINGS.write().unwrap();

            let updates = [
                (
                    "wallet_entropy_source",
                    toml_edit::value(
                        VALID_ENTROPY_SOURCES[entropy_source_dropdown.selected() as usize],
                    ),
                ),
                (
                    "wallet_entropy_length",
                    toml_edit::value(
                        VALID_ENTROPY_LENGTHS[entropy_length_dropdown.selected() as usize] as i64,
                    ),
                ),
                (
                    "wallet_bip",
                    toml_edit::value(
                        VALID_BIP_DERIVATIONS[bip_dropdown.selected() as usize] as i64,
                    ),
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
                    toml_edit::value(
                        APP_LANGUAGE[default_gui_language_dropdown.selected() as usize],
                    ),
                ),
                (
                    "gui_search",
                    toml_edit::value(
                        VALID_COIN_SEARCH_PARAMETER
                            [default_search_parameter_dropdown.selected() as usize],
                    ),
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
                    toml_edit::value(
                        APP_LOG_LEVEL[default_gui_log_level_dropdown.selected() as usize],
                    ),
                ),
                (
                    "anu_enabled",
                    toml_edit::value(use_anu_api_checkbox.is_active()),
                ),
                (
                    "anu_log",
                    toml_edit::value(log_anu_api_checkbox.is_active()),
                ),
                (
                    "anu_timeout",
                    toml_edit::value(anu_connection_timeout_spinbutton.value_as_int() as i64),
                ),
                (
                    "anu_data_format",
                    toml_edit::value(
                        VALID_ANU_API_DATA_FORMAT[anu_data_format_dropdown.selected() as usize],
                    ),
                ),
                (
                    "anu_array_length",
                    toml_edit::value(default_anu_array_length_spinbutton.value_as_int() as i64),
                ),
                (
                    "anu_hex_block_size",
                    toml_edit::value(default_anu_hex_length_spinbutton.value_as_int() as i64),
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

            updates.iter().for_each(|(key, value)| {
                let gui_related = matches!(*key, "gui_theme" | "gui_log" | "gui_icons");
                settings.update_value(key, value.clone(), gui_related.then(|| gui_state.clone()));
            });

            AppSettings::save_settings(&settings);

            {
                let lock_app_messages = app_messages_state.borrow();
                lock_app_messages.queue_message(
                    t!("UI.messages.dialog.settings_saved").to_string(),
                    gtk::MessageType::Info,
                );
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
            let dialog = gtk::MessageDialog::builder()
                .modal(true)
                .text("Do you really want to reset config?")
                .build();

            dialog.add_button("Yes", gtk::ResponseType::Yes);
            dialog.add_button("No", gtk::ResponseType::No);

            dialog.connect_response(clone!(
                #[weak]
                settings_window,
                #[weak]
                app_messages_state,
                #[strong]
                gui_state,
                move |dialog, response| {
                    match response {
                        gtk::ResponseType::Yes => {
                            let lock_app_messages = app_messages_state.borrow();

                            match reset_user_settings().unwrap().as_str() {
                                "OK" => {
                                    dialog.close();
                                    settings_window.close();

                                    AppSettings::load_settings();
                                    adw::StyleManager::default()
                                        .set_color_scheme(adw::ColorScheme::PreferLight);

                                    let new_gui_state = std::rc::Rc::new(std::cell::RefCell::new(
                                        GuiState::default_config(),
                                    ));
                                    let mut lock_new_gui_state = new_gui_state.borrow_mut();
                                    let mut lock_gui_state = gui_state.borrow_mut();

                                    lock_gui_state.reload_gui_icons();
                                    lock_new_gui_state.gui_main_buttons =
                                        lock_gui_state.gui_main_buttons.clone();
                                    lock_new_gui_state.reload_gui_icons();
                                    lock_new_gui_state.apply_language();

                                    lock_app_messages.queue_message(
                                        t!("UI.messages.dialog.settings_reset").to_string(),
                                        gtk::MessageType::Info,
                                    );
                                }
                                _ => {
                                    lock_app_messages.queue_message(
                                        t!("error.settings.reset").to_string(),
                                        gtk::MessageType::Error,
                                    );
                                }
                            }
                        }
                        _ => {
                            dialog.close();
                        }
                    }
                }
            ));

            dialog.show();
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

    settings_window
}

fn reset_user_settings() -> Result<String, String> {
    println!("[+] {}", &t!("log.reset_user_settings").to_string());

    {
        let local_settings = os::LOCAL_SETTINGS.lock().unwrap();
        let local_config_file = local_settings.local_config_file.clone().unwrap();

        println!("\t- Local config file: {:?}", local_config_file);

        match std::fs::remove_file(local_config_file) {
            Ok(_) => {
                println!("\t- Local config file deleted");
            }
            Err(err) => {
                eprintln!("\t- Local config file NOT deleted \n Error: {}", err);
            }
        };
    }

    match os::check_local_config() {
        Ok(_) => {
            println!("\t- New config file created");
            Ok("OK".to_string())
        }
        Err(err) => {
            eprintln!("\t- New config file NOT created \n {}", err);
            Err(err.to_string())
        }
    }
}

fn create_about_window() {
    println!("[+] {}", &t!("log.create_about_window").to_string());

    let pixy: gtk4::gdk::Texture = qr2m_lib::get_texture_from_resource(&format!("logo/logo.{}", GUI_IMAGE_EXTENSION));
    let logo_picture = gtk::Picture::new();
    logo_picture.set_paintable(Some(&pixy));

    let my_license = std::path::Path::new("licenses").join("LICENSE.txt");
    let app_license = qr2m_lib::get_text_from_resources(my_license.to_str().unwrap());

    let their_license = std::path::Path::new("licenses").join("LICENSE-LGPL-2.1.txt");
    let lgpl_license = qr2m_lib::get_text_from_resources(their_license.to_str().unwrap());

    let licenses = format!("{}\n\n---\n\n{}", app_license, lgpl_license);

    let they_forced_me = [
        "This application uses GTK4 for its GUI.",
        "GTK4 is licensed under the GNU Lesser General Public License (LGPL) version 2.1 or later.",
        "For more details on the LGPL-2.1 license and your rights under this license, please refer to the License tab.",
    ];
    let comments = they_forced_me.join(" ");

    let help_window = gtk::AboutDialog::builder()
        .modal(true)
        // .default_width(600)
        .default_height(400)
        .program_name(APP_DESCRIPTION.unwrap())
        .version(APP_VERSION.unwrap())
        .website("https://www.github.com/control-owl/qr2m")
        .website_label("GitHub project")
        .authors([APP_AUTHOR.unwrap()])
        .copyright("Copyright [2023-2025] Control Owl")
        .license(licenses)
        .wrap_license(true)
        .comments(t!("UI.about.description").to_string())
        .logo(
            &logo_picture
                .paintable()
                .unwrap_or(gtk::gdk::Paintable::new_empty(32, 32)),
        )
        .comments(comments)
        .build();

    help_window.show();
}

fn open_wallet_from_file(
    app_messages_state: &std::rc::Rc<std::cell::RefCell<AppMessages>>,
) -> (String, Option<String>) {
    println!("[+] {}", &t!("log.open_wallet_from_file").to_string());

    let open_context = glib::MainContext::default();
    let open_loop = glib::MainLoop::new(Some(&open_context), false);
    let (tx, rx) = std::sync::mpsc::channel();

    let open_window = gtk::Window::new();
    let open_dialog = gtk::FileChooserNative::new(
        Some("Open Wallet File"),
        Some(&open_window),
        gtk::FileChooserAction::Open,
        Some("Open"),
        Some("Cancel"),
    );

    let filter = gtk::FileFilter::new();
    filter.add_pattern("*.qr2m");
    filter.set_name(Some("Wallet file (*.qr2m)"));
    open_dialog.add_filter(&filter);

    let all_files_filter = gtk::FileFilter::new();
    all_files_filter.add_pattern("*");
    all_files_filter.set_name(Some("All files"));
    open_dialog.add_filter(&all_files_filter);

    open_dialog.connect_response(clone!(
        #[strong] open_loop,
        #[strong] app_messages_state,
        move |open_dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file) = open_dialog.file() {
                    if let Some(path) = file.path() {
                        let file_path = path.to_string_lossy().to_string();
                        println!("\t- Wallet file chosen: {:?}", file_path);

                        let result = process_wallet_file_from_path(&file_path);
                        let lock_state = app_messages_state.borrow();

                        match result {
                            Ok((version, entropy, password)) => {
                                let passphrase = password.unwrap_or_default();

                                let file_entropy_string =
                                    format!("{}\n{}\n{}", version, entropy, passphrase);

                                if let Err(err) = tx.send(file_entropy_string) {
                                    lock_state.queue_message(
                                        format!("{} : {}", t!("error.wallet.send"), err),
                                        gtk::MessageType::Error,
                                    );
                                } else {
                                    lock_state.queue_message(
                                        t!("UI.messages.wallet.open").to_string(),
                                        gtk::MessageType::Info,
                                    );
                                    open_loop.quit();
                                }
                            }
                            Err(err) => {
                                lock_state.queue_message(
                                    format!("{} : {}", t!("error.wallet.process"), err),
                                    gtk::MessageType::Error,
                                );

                                open_loop.quit();
                            }
                        }
                    }
                }
            }
            open_dialog.destroy();
            open_loop.quit();
        }
    ));

    open_dialog.show();
    open_loop.run();

    match rx.recv() {
        Ok(file_entropy_string) => {
            let parts: Vec<&str> = file_entropy_string.split("\n").collect();
            let entropy = parts.get(1).map(|s| s.to_string());
            let passphrase = parts.get(2).map(|s| s.to_string());

            // IMPLEMENT: Check entropy before importing

            (entropy.unwrap_or_default(), passphrase)
        }
        Err(err) => {
            let lock_state = app_messages_state.borrow();
            lock_state.queue_message(
                format!("{} : {}", t!("error.wallet.open"), err),
                gtk::MessageType::Error,
            );
            (String::new(), None)
        }
    }
}

fn save_wallet_to_file() {
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
    let save_dialog = gtk::FileChooserNative::new(
        Some(t!("UI.dialog.save").to_string().as_str()),
        Some(&save_window),
        gtk::FileChooserAction::Save,
        Some(&t!("UI.element.button.save")),
        Some(&t!("UI.element.button.cancel")),
    );

    let filter = gtk::FileFilter::new();
    filter.add_pattern(&format!("*.{}", WALLET_DEFAULT_EXTENSION));
    filter.set_name(Some(&format!(
        "Wallet file (*.{})",
        WALLET_DEFAULT_EXTENSION
    )));
    save_dialog.add_filter(&filter);

    let all_files_filter = gtk::FileFilter::new();
    all_files_filter.add_pattern("*");
    all_files_filter.set_name(Some("All files"));
    save_dialog.add_filter(&all_files_filter);

    save_dialog.connect_response(clone!(
        #[strong]
        save_loop,
        move |save_dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file) = save_dialog.file() {
                    if let Some(path) = file.path() {
                        // TODO: Get data from WalletSettings struct
                        let wallet_data = format!(
                            "version = {}\n{}\n{}",
                            WALLET_CURRENT_VERSION, entropy_string, mnemonic_passphrase
                        );

                        std::fs::write(path, wallet_data).expect("Unable to write file");
                        save_loop.quit();
                    }
                }
            }
            save_dialog.destroy();
            save_loop.quit();
        }
    ));
    save_dialog.show();
    save_loop.run();
}

fn update_derivation_label(dp: DerivationPath, label: gtk::Label) {
    println!("[+] {}", &t!("log.update_derivation_label").to_string());

    println!("\t- Derivation Path: {:?}", dp);

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

    println!("\t- Derivation path: {:?}", &path);

    label.set_text(&path);
}

fn process_wallet_file_from_path(file_path: &str) -> Result<(u8, String, Option<String>), String> {
    println!(
        "[+] {}",
        &t!("log.process_wallet_file_from_path").to_string()
    );

    let file =
        File::open(file_path).map_err(|_| "Error: Could not open wallet file".to_string())?;
    let mut lines = std::io::BufReader::new(file).lines();

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
