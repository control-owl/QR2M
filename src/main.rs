// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// copyright = "Copyright © 2023-2024 D3BUG"


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

// #![windows_subsystem = "windows"]
#![allow(non_snake_case)]
// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(unused_assignments)]
// #![allow(dead_code)]
// #![allow(unused_mut)]


// REQUIREMENTS -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


// Crates
use std::{
    fs::{self, File}, 
    io::{self, BufRead, Write}, 
    time::SystemTime
};
use hex;
use rand::Rng;

use gtk4 as gtk;
use libadwaita as adw;
use adw::prelude::*;
use gtk::{gio, glib::clone, Stack, StackSidebar};


// Mods
mod anu;
mod coin_db;
mod dev;
mod os;
mod test_vectors;
mod keys;

// Multi-language support
#[macro_use]
extern crate rust_i18n;
i18n!("res/locale", fallback = "en");

// Default settings
const APP_NAME: Option<&str> = option_env!("CARGO_PKG_NAME");
const APP_DESCRIPTION: Option<&str> = option_env!("CARGO_PKG_DESCRIPTION");
const APP_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
const APP_AUTHOR: Option<&str> = option_env!("CARGO_PKG_AUTHORS");
const APP_LANGUAGE: &'static [&'static str] = &[
    "English", 
    "Deutsch",
    "Hrvatski",
];
const DEFAULT_NOTIFICATION_TIMEOUT: u32 = 10;
const DEFAULT_MNEMONIC_LENGTH: u32 = 10;
const WORDLIST_FILE: &str = "bip39-mnemonic-words-english.txt";
const APP_DEFAULT_CONFIG_FILE: &str = "default.conf";
const VALID_ENTROPY_LENGTHS: [u32; 5] = [128, 160, 192, 224, 256];
const VALID_BIP_DERIVATIONS: [u32; 5] = [32, 44, 49, 84, 86];
const VALID_ENTROPY_SOURCES: &'static [&'static str] = &[
    "RNG", 
    "RNG+", 
    "File",
    "QRNG",
];
const VALID_WALLET_PURPOSE: &'static [&'static str] = &[
    "Internal", 
    "External", 
];
const VALID_ANU_API_DATA_FORMAT: &'static [&'static str] = &[
    "uint8", 
    "uint16", 
    "hex16",
];
const WALLET_DEFAULT_ADDRESS_COUNT: u32 = 10;
const WALLET_DEFAULT_HARDENED_ADDRESS: bool = false;
const WALLET_DEFAULT_EXTENSION: &str = "qr2m";
const WALLET_CURRENT_VERSION: u32 = 1;
const ANU_DEFAULT_ARRAY_LENGTH: u32 = 1024;
const ANU_MINIMUM_ARRAY_LENGTH: u32 = 32;
const ANU_MAXIMUM_ARRAY_LENGTH: u32 = 1024;
const ANU_DEFAULT_HEX_BLOCK_SIZE: u32 = 16;
const DEFAULT_PROXY_PORT: u32 = 8080;
const PROXY_DEFAULT_RETRY_ATTEMPTS: u32 = 3;
const PROXY_DEFAULT_TIMEOUT: u32 = 5000;
const WINDOW_MAIN_DEFAULT_WIDTH: u32 = 1000;
const WINDOW_MAIN_DEFAULT_HEIGHT: u32 = 800;
const WINDOW_SETTINGS_DEFAULT_WIDTH: u32 = 700;
const WINDOW_SETTINGS_DEFAULT_HEIGHT: u32 = 500;
const VALID_PROXY_STATUS: &'static [&'static str] = &[
    "Off", 
    "Auto", 
    "Manual",
];
const VALID_GUI_THEMES: &'static [&'static str] = &[
    "System", 
    "Light", 
    "Dark",
];
const VALID_COIN_SEARCH_PARAMETER: &'static [&'static str] = &[
    "Name", 
    "Symbol", 
    "Index",
];

lazy_static::lazy_static! {
    static ref WALLET_SETTINGS: std::sync::Arc<std::sync::Mutex<WalletSettings>> = std::sync::Arc::new(std::sync::Mutex::new(WalletSettings::default()));
    static ref APPLICATION_SETTINGS: std::sync::Arc<std::sync::Mutex<AppSettings>> = std::sync::Arc::new(std::sync::Mutex::new(AppSettings::default()));
}

struct AppState {
    language: Option<String>,
    theme: Option<String>,
    app_messages: Option<std::sync::Arc<std::sync::Mutex<AppMessages>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            language: Some("en".to_string()),
            theme: Some("System".to_string()),
            app_messages: None,
        }
    }

    fn initialize_app_messages(&mut self, info_bar: gtk::Revealer) {
        self.app_messages = Some(std::sync::Arc::new(std::sync::Mutex::new(AppMessages::new(info_bar))));
    }

    fn show_message(&self, message: String, message_type: gtk::MessageType) {
        if let Some(app_messages) = &self.app_messages {
            let app_messages = app_messages.lock().unwrap();
            app_messages.queue_message(message, message_type);
        }
    }

    fn apply_theme(&mut self) {
        if let Some(theme) = &self.theme {
            let preferred_theme = match theme.as_str() {
                "System" => adw::ColorScheme::PreferLight,
                "Light" => adw::ColorScheme::ForceLight,
                "Dark" => adw::ColorScheme::PreferDark,
                _ => {
                    self.show_message(
                        t!("error.settings.parse", element = "gui_theme", value = theme).to_string(),
                        gtk::MessageType::Error
                    );
                    adw::ColorScheme::PreferLight
                }
            };
            // self.show_message("Theme changed.".to_string(), gtk::MessageType::Info);

            adw::StyleManager::default().set_color_scheme(preferred_theme);
        } else {
            self.show_message("Theme is not set. Using default color scheme.".to_string(), gtk::MessageType::Warning);
            adw::StyleManager::default().set_color_scheme(adw::ColorScheme::PreferLight);
        }
    }

    fn apply_language(&mut self) {
        if let Some(language) = &self.language {
            let language_code = match language.as_str() {
                "Deutsch" => "de",
                "Hrvatski" => "hr",
                "English" => "en",
                _ => "en",
            };
        
            rust_i18n::set_locale(language_code);
            self.show_message(t!("UI.messages.change-language").to_string(), gtk::MessageType::Info);
            
        } else {
            rust_i18n::set_locale("en");
            self.show_message("Language is not set. Falling back to English.".to_string(), gtk::MessageType::Warning);
        }
    }
}

struct AppMessages {
    info_bar: Option<gtk::Revealer>,
    message_queue: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<(String, gtk::MessageType)>>>,
    processing: std::sync::Arc<std::sync::Mutex<bool>>,
}

impl AppMessages {
    fn new(info_bar: gtk::Revealer) -> Self {
        Self {
            info_bar: Some(info_bar),
            message_queue: std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new())),
            processing: std::sync::Arc::new(std::sync::Mutex::new(false)),
        }
    }

    fn queue_message(&self, message: String, message_type: gtk::MessageType) {
        let mut queue = self.message_queue.lock().unwrap();
        queue.push_back((message, message_type));

        if !*self.processing.lock().unwrap() {
            self.start_message_processor();
        }
    }

    fn start_message_processor(&self) {
        let info_bar = match &self.info_bar {
            Some(info_bar) => info_bar.clone(),
            None => {
                eprintln!("Error: info_bar is not initialized.");
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
    
                    // IMPLEMENT: Connect to value from setting 
                    glib::timeout_add_local(std::time::Duration::from_secs(3), {
                        let queue = queue.clone();
                        let info_bar = info_bar.clone();
                        let processing = processing.clone();
    
                        move || {
                            info_bar.set_reveal_child(false);
    
                            AppMessages::start_next_message(&queue, &info_bar, &processing);
    
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
        queue: &std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<(String, gtk::MessageType)>>>,
        info_bar: &gtk::Revealer,
        processing: &std::sync::Arc<std::sync::Mutex<bool>>,
    ) {
        let mut queue_lock = queue.lock().unwrap();
        if let Some((message, message_type)) = queue_lock.pop_front() {
            AppMessages::create_info_message(info_bar, &message, message_type);
    
            let queue = queue.clone();
            let info_bar = info_bar.clone();
            let processing = processing.clone();
    
            glib::timeout_add_local(std::time::Duration::from_secs(3), move || {
                info_bar.set_reveal_child(false);
    
                AppMessages::start_next_message(&queue, &info_bar, &processing);
    
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
        let message_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        let message_label = gtk::Label::new(Some(message));
        message_label.set_hexpand(true);

        match message_type {
            gtk::MessageType::Error => message_box.set_css_classes(&["error-message"]),
            gtk::MessageType::Warning => message_box.set_css_classes(&["warning-message"]),
            _ => message_box.set_css_classes(&["info-message"]),
        }

        let close_button = gtk::Button::with_label("Close");
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

#[derive(Debug, Default)]
struct AppSettings {
    wallet_entropy_source: String,
    wallet_entropy_length: u32,
    wallet_bip: u32,
    wallet_address_count: u32,
    wallet_hardened_address: bool,
    gui_save_size: bool,
    gui_last_width: u32,
    gui_last_height: u32,
    gui_maximized: bool,
    gui_theme: String,
    gui_language: String,
    gui_search: String,
    gui_notification_timeout: u32,
    gui_mnemonic_length: u32,
    anu_enabled: bool,
    anu_data_format: String,
    anu_array_length: u32,
    anu_hex_block_size: u32,
    anu_log: bool,
    proxy_status: String,
    proxy_server_address: String,
    proxy_server_port: u32,
    proxy_use_pac: bool,
    proxy_script_address: String,
    proxy_login_credentials: bool,
    proxy_login_username: String,
    proxy_login_password: String,
    proxy_use_ssl: bool,
    proxy_ssl_certificate: String,
    proxy_retry_attempts: u32,
    proxy_timeout: u32,
}

impl AppSettings {
    fn load_settings() -> io::Result<Self> {
        println!("[+] {}", &t!("log.load-settings").to_string());
        
        let local_settings = os::LOCAL_SETTINGS.lock().unwrap();
        let local_config_file = local_settings.local_config_file.clone().unwrap();
        
        println!("\t Settings file: {:?}", local_config_file);

        let config_str = match fs::read_to_string(&local_config_file) {
            Ok(contents) => contents,
            Err(err) => {
                eprintln!(
                    "Failed to read local config file: {:?} \n Error: {:?}",
                    local_config_file, err
                );
        
                let default_config = qr2m_lib::get_text_from_resources(APP_DEFAULT_CONFIG_FILE);
                if default_config.is_empty() {
                    eprintln!("Default configuration from embedded resources is empty.");
                } else {
                    println!("Loaded default configuration from embedded resources.");
                }
        
                default_config
            }
        };

        let config: toml::Value = config_str.parse().unwrap_or_else(|err| {
            println!("{}", &t!("error.settings.config", error = err));
            toml::Value::Table(toml::value::Table::new())
        });

        fn get_str<'a>(section: &'a toml::Value, key: &str, default: &str) -> String {
            section.get(key).and_then(|v| v.as_str()).unwrap_or(default).to_string()
        }

        fn get_u32<'a>(section: &'a toml::Value, key: &str, default: u32) -> u32 {
            section.get(key).and_then(|v| v.as_integer()).map(|v| v as u32).unwrap_or(default)
        }

        fn get_bool<'a>(section: &'a toml::Value, key: &str, default: bool) -> bool {
            section.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
        }

        fn load_section(config: &toml::Value, section_name: &str, default_config: &toml::Value) -> toml::Value {
            config.get(section_name)
                .cloned()
                .unwrap_or_else(|| default_config.get(section_name).cloned().unwrap_or(toml::Value::Table(toml::value::Table::new())))
        }
        
        let default_config_str = qr2m_lib::get_text_from_resources(APP_DEFAULT_CONFIG_FILE);
        let default_config: toml::Value = default_config_str.parse().unwrap_or_else(|_| toml::Value::Table(toml::value::Table::new()));
        
        // Sections
        let gui_section = load_section(&config, "gui", &default_config);
        let wallet_section = load_section(&config, "wallet", &default_config);
        let anu_section = load_section(&config, "anu", &default_config);
        let proxy_section = load_section(&config, "proxy", &default_config);
        
        // GUI settings
        let gui_save_size = get_bool(&gui_section, "save_size", true);
        let gui_last_width = get_u32(&gui_section, "last_width", WINDOW_MAIN_DEFAULT_WIDTH);
        let gui_last_height = get_u32(&gui_section, "last_height", WINDOW_MAIN_DEFAULT_HEIGHT);
        let gui_maximized = get_bool(&gui_section, "maximized", true);
        let gui_theme = get_str(&gui_section, "theme", &VALID_GUI_THEMES[0]);
        let gui_language = get_str(&gui_section, "language", &APP_LANGUAGE[0]);
        let gui_search = get_str(&gui_section, "search", &VALID_COIN_SEARCH_PARAMETER[0]);
        let gui_notification_timeout = get_u32(&gui_section, "notification_timeout", DEFAULT_NOTIFICATION_TIMEOUT);
        let gui_mnemonic_length = get_u32(&gui_section, "mnemonic_length", DEFAULT_MNEMONIC_LENGTH);

        println!("\t Save last window size: {:?}", gui_save_size);
        println!("\t GUI width: {:?}", gui_last_width);
        println!("\t GUI height: {:?}", gui_last_height);
        println!("\t Maximized: {:?}", gui_maximized);
        println!("\t Theme: {:?}", gui_theme);
        println!("\t Language: {:?}", gui_language);
        println!("\t Search: {:?}", gui_search);
        println!("\t Notification timeout: {:?}", gui_notification_timeout);
        println!("\t Mnemonic passphrase length: {:?}", gui_mnemonic_length);

        // Wallet settings
        let wallet_entropy_source = get_str(&wallet_section, "entropy_source", &VALID_ENTROPY_SOURCES[0]);
        let wallet_entropy_length = get_u32(&wallet_section, "entropy_length", *VALID_ENTROPY_LENGTHS.last().unwrap_or(&0));
        let wallet_bip = get_u32(&wallet_section, "bip", *VALID_BIP_DERIVATIONS.get(1).unwrap_or(&VALID_BIP_DERIVATIONS[0]));
        let wallet_address_count = get_u32(&wallet_section, "address_count", WALLET_DEFAULT_ADDRESS_COUNT);
        let wallet_hardened_address = get_bool(&wallet_section, "hardened_address", WALLET_DEFAULT_HARDENED_ADDRESS);

        println!("\t Entropy source: {:?}", wallet_entropy_source);
        println!("\t Entropy length: {:?}", wallet_entropy_length);
        println!("\t BIP: {:?}", wallet_bip);
        println!("\t Address count: {:?}", wallet_address_count);
        println!("\t Hard address: {:?}", wallet_hardened_address);

        // ANU settings
        let anu_enabled = get_bool(&anu_section, "enabled", true);
        let anu_data_format = get_str(&anu_section, "data_format", &VALID_ANU_API_DATA_FORMAT[0]);
        let anu_array_length = get_u32(&anu_section, "array_length", ANU_DEFAULT_ARRAY_LENGTH);
        let anu_hex_block_size = get_u32(&anu_section, "hex_block_size", ANU_DEFAULT_HEX_BLOCK_SIZE);
        let anu_log = get_bool(&anu_section, "log", true);

        println!("\t Use ANU: {:?}", anu_enabled);
        println!("\t ANU data format: {:?}", anu_data_format);
        println!("\t ANU array length: {:?}", anu_array_length);
        println!("\t ANU hex block size: {:?}", anu_hex_block_size);
        println!("\t ANU log: {:?}", anu_log);

        // Proxy settings
        let proxy_status = get_str(&proxy_section, "status", &VALID_PROXY_STATUS[0]);
        let proxy_server_address = get_str(&proxy_section, "server_address", "");
        let proxy_server_port = get_u32(&proxy_section, "server_port", DEFAULT_PROXY_PORT);
        let proxy_use_pac = get_bool(&proxy_section, "use_pac", false);
        let proxy_script_address = get_str(&proxy_section, "script_address", "");
        let proxy_login_credentials = get_bool(&proxy_section, "login_credentials", false);
        let proxy_login_username = get_str(&proxy_section, "login_username", "");
        let proxy_login_password = get_str(&proxy_section, "login_password", "");
        let proxy_use_ssl = get_bool(&proxy_section, "use_ssl", false);
        let proxy_ssl_certificate = get_str(&proxy_section, "ssl_certificate", "");
        let proxy_retry_attempts = get_u32(&proxy_section, "retry_attempts", PROXY_DEFAULT_RETRY_ATTEMPTS);
        let proxy_timeout = get_u32(&proxy_section, "timeout", PROXY_DEFAULT_TIMEOUT);
        
        println!("\t Use proxy: {:?}", proxy_status);
        println!("\t Proxy server address: {:?}", proxy_server_address);
        println!("\t Proxy server port: {:?}", proxy_server_port);
        println!("\t Use proxy PAC: {:?}", proxy_use_pac);
        println!("\t Proxy script address: {:?}", proxy_script_address);
        println!("\t Use proxy login credentials: {:?}", proxy_login_credentials);
        println!("\t Proxy username: {:?}", proxy_login_username);
        println!("\t Proxy password: {:?}", proxy_login_password);
        println!("\t Use proxy SSL: {:?}", proxy_use_ssl);
        println!("\t Proxy SSL certificate: {:?}", proxy_ssl_certificate);
        println!("\t Proxy retry attempts: {:?}", proxy_retry_attempts);
        println!("\t Proxy timeout: {:?}", proxy_timeout);

        let mut application_settings = APPLICATION_SETTINGS.lock().unwrap();
        application_settings.wallet_entropy_source = wallet_entropy_source.clone();
        application_settings.wallet_entropy_length = wallet_entropy_length.clone();
        application_settings.wallet_bip = wallet_bip.clone();
        application_settings.wallet_address_count = wallet_address_count.clone();
        application_settings.wallet_hardened_address = wallet_hardened_address.clone();

        application_settings.gui_save_size = gui_save_size.clone();
        application_settings.gui_last_width = gui_last_width.clone();
        application_settings.gui_last_height = gui_last_height.clone();
        application_settings.gui_maximized = gui_maximized.clone();
        application_settings.gui_theme = gui_theme.clone();
        application_settings.gui_language = gui_language.clone();
        application_settings.gui_search = gui_search.clone();
        application_settings.gui_notification_timeout = gui_notification_timeout.clone();
        application_settings.gui_mnemonic_length = gui_mnemonic_length.clone();

        application_settings.anu_enabled = anu_enabled.clone();
        application_settings.anu_data_format = anu_data_format.clone();
        application_settings.anu_array_length = anu_array_length.clone();
        application_settings.anu_hex_block_size = anu_hex_block_size.clone();
        application_settings.anu_log = anu_log.clone();

        application_settings.proxy_status = proxy_status.clone();
        application_settings.proxy_server_address = proxy_server_address.clone();
        application_settings.proxy_server_port = proxy_server_port.clone();
        application_settings.proxy_use_pac = proxy_use_pac.clone();
        application_settings.proxy_script_address = proxy_script_address.clone();
        application_settings.proxy_login_credentials = proxy_login_credentials.clone();
        application_settings.proxy_login_username = proxy_login_username.clone();
        application_settings.proxy_login_password = proxy_login_password.clone();
        application_settings.proxy_use_ssl = proxy_use_ssl.clone();
        application_settings.proxy_ssl_certificate = proxy_ssl_certificate.clone();
        application_settings.proxy_retry_attempts = proxy_retry_attempts.clone();
        application_settings.proxy_timeout = proxy_timeout.clone();

        Ok(AppSettings {
            wallet_entropy_source,
            wallet_entropy_length,
            wallet_bip,
            wallet_address_count,
            wallet_hardened_address,
            gui_save_size,
            gui_last_width,
            gui_last_height,
            gui_maximized,
            gui_theme,
            gui_language,
            gui_search,
            gui_notification_timeout,
            gui_mnemonic_length,
            anu_enabled,
            anu_data_format,
            anu_array_length,
            anu_hex_block_size,
            anu_log,
            proxy_status,
            proxy_server_address,
            proxy_server_port,
            proxy_use_pac,
            proxy_script_address,
            proxy_login_credentials,
            proxy_login_username,
            proxy_login_password,
            proxy_use_ssl,
            proxy_ssl_certificate,
            proxy_retry_attempts,
            proxy_timeout,
        })
    }

    fn update_value(
        &mut self,
        key: &str,
        new_value: toml_edit::Item,
        state: Option<std::sync::Arc<std::sync::Mutex<AppState>>>,
    ) {
        match key {
            "wallet_entropy_source" => {
                if new_value.as_str().map(|v| v != self.wallet_entropy_source).unwrap_or(false) {
                    self.wallet_entropy_source = new_value.as_str().unwrap().to_string();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "wallet_entropy_length" => {
                if new_value.as_integer().map(|v| v as u32 != self.wallet_entropy_length).unwrap_or(false) {
                    self.wallet_entropy_length = new_value.as_integer().unwrap() as u32;
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "wallet_bip" => {
                if new_value.as_integer().map(|v| v as u32 != self.wallet_bip).unwrap_or(false) {
                    self.wallet_bip = new_value.as_integer().unwrap() as u32;
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "wallet_address_count" => {
                if new_value.as_integer().map(|v| v as u32 != self.wallet_address_count).unwrap_or(false) {
                    self.wallet_address_count = new_value.as_integer().unwrap() as u32;
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "wallet_hardened_address" => {
                if new_value.as_bool().map(|v| v != self.wallet_hardened_address).unwrap_or(false) {
                    self.wallet_hardened_address = new_value.as_bool().unwrap();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "gui_save_size" => {
                if new_value.as_bool().map(|v| v != self.gui_save_size).unwrap_or(false) {
                    self.gui_save_size = new_value.as_bool().unwrap();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "gui_last_width" => {
                if new_value.as_integer().map(|v| v as u32 != self.gui_last_width).unwrap_or(false) {
                    self.gui_last_width = new_value.as_integer().unwrap() as u32;
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "gui_last_height" => {
                if new_value.as_integer().map(|v| v as u32 != self.gui_last_height).unwrap_or(false) {
                    self.gui_last_height = new_value.as_integer().unwrap() as u32;
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "gui_maximized" => {
                if new_value.as_bool().map(|v| v != self.gui_maximized).unwrap_or(false) {
                    self.gui_maximized = new_value.as_bool().unwrap();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "gui_theme" => {
                if let Some(new_theme) = new_value.as_str() {
                    if Some(new_theme) != Some(&self.gui_theme) {
                        self.gui_theme = new_theme.to_string();
                        println!("Updating key {:?} = {:?}", key, new_value);
                        
                        if let Some(state) = state {
                            let mut state = state.lock().unwrap();
                            state.theme = Some(self.gui_theme.clone());
                            state.apply_theme();
                        } else {
                            println!("State in gui_theme is None");
                        }

                        self.save_settings();
                    }
                } else {
                    eprintln!("Received invalid value for gui_theme: {:?}", new_value);
                }
            },
            "gui_language" => {
                if let Some(new_language) = new_value.as_str() {
                    if Some(new_language) != Some(&self.gui_language) {
                        self.gui_language = new_language.to_string();
                        println!("Updating key {:?} = {:?}", key, new_value);
                        
                        if let Some(state) = state {
                            let mut state = state.lock().unwrap();
                            state.language = Some(self.gui_language.clone());
                            state.apply_language();
                        } else {
                            eprintln!("State in gui_language is None");
                        }

                        self.save_settings();
                    }
                } else {
                    eprintln!("Received invalid value for gui_language: {:?}", new_value);
                }
            },
            "gui_search" => {
                if new_value.as_str().map(|v| v != self.gui_search).unwrap_or(false) {
                    self.gui_search = new_value.as_str().unwrap().to_string();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "gui_notification_timeout" => {
                if new_value.as_integer().map(|v| v as u32 != self.gui_notification_timeout).unwrap_or(false) {
                    self.gui_notification_timeout = new_value.as_integer().unwrap() as u32;
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "gui_mnemonic_length" => {
                if new_value.as_integer().map(|v| v as u32 != self.gui_mnemonic_length).unwrap_or(false) {
                    self.gui_mnemonic_length = new_value.as_integer().unwrap() as u32;
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "anu_enabled" => {
                if new_value.as_bool().map(|v| v != self.anu_enabled).unwrap_or(false) {
                    self.anu_enabled = new_value.as_bool().unwrap();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "anu_data_format" => {
                if new_value.as_str().map(|v| v != self.anu_data_format).unwrap_or(false) {
                    self.anu_data_format = new_value.as_str().unwrap().to_string();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "anu_array_length" => {
                if new_value.as_integer().map(|v| v as u32 != self.anu_array_length).unwrap_or(false) {
                    self.anu_array_length = new_value.as_integer().unwrap() as u32;
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "anu_hex_block_size" => {
                if new_value.as_integer().map(|v| v as u32 != self.anu_hex_block_size).unwrap_or(false) {
                    self.anu_hex_block_size = new_value.as_integer().unwrap() as u32;
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "anu_log" => {
                if new_value.as_bool().map(|v| v != self.anu_log).unwrap_or(false) {
                    self.anu_log = new_value.as_bool().unwrap();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "proxy_status" => {
                if new_value.as_str().map(|v| v != self.proxy_status).unwrap_or(false) {
                    self.proxy_status = new_value.as_str().unwrap().to_string();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "proxy_server_address" => {
                if new_value.as_str().map(|v| v != self.proxy_server_address).unwrap_or(false) {
                    self.proxy_server_address = new_value.as_str().unwrap().to_string();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "proxy_server_port" => {
                if new_value.as_integer()
                .map(|v| (v as u32 != self.proxy_server_port) && (v != 0))
                .unwrap_or(false) 
                {
                    self.proxy_server_port = new_value.as_integer().unwrap() as u32;
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "proxy_use_pac" => {
                if new_value.as_bool().map(|v| v != self.proxy_use_pac).unwrap_or(false) {
                    self.proxy_use_pac = new_value.as_bool().unwrap();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "proxy_script_address" => {
                if new_value.as_str().map(|v| v != self.proxy_script_address).unwrap_or(false) {
                    self.proxy_script_address = new_value.as_str().unwrap().to_string();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "proxy_login_credentials" => {
                if new_value.as_bool().map(|v| v != self.proxy_login_credentials).unwrap_or(false) {
                    self.proxy_login_credentials = new_value.as_bool().unwrap();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "proxy_login_username" => {
                if new_value.as_str().map(|v| v != self.proxy_login_username).unwrap_or(false) {
                    self.proxy_login_username = new_value.as_str().unwrap().to_string();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "proxy_login_password" => {
                if new_value.as_str().map(|v| v != self.proxy_login_password).unwrap_or(false) {
                    self.proxy_login_password = new_value.as_str().unwrap().to_string();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "proxy_use_ssl" => {
                if new_value.as_bool().map(|v| v != self.proxy_use_ssl).unwrap_or(false) {
                    self.proxy_use_ssl = new_value.as_bool().unwrap();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            "proxy_ssl_certificate" => {
                if new_value.as_str().map(|v| v != self.proxy_ssl_certificate).unwrap_or(false) {
                    self.proxy_ssl_certificate = new_value.as_str().unwrap().to_string();
                    println!("Updating key {:?} = {:?}", key, new_value);
                    self.save_settings();
                }
            },
            _ => {}
        }
    }

    fn save_settings(&self) {
        let local_settings = os::LOCAL_SETTINGS.lock().unwrap();
        let local_config_file = local_settings.local_config_file.clone().unwrap();

        let config_str = fs::read_to_string(&local_config_file)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to read config file: {}", e)))
            .expect("Problem with local config file");
        
        let mut doc = config_str.parse::<toml_edit::DocumentMut>()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to parse config string: {}", e)))
            .expect("Problem with mut doc");
    
        {
            let wallet_section = doc["wallet"].or_insert(toml_edit::Item::Table(Default::default()));
                if let toml_edit::Item::Table(wallet_table) = wallet_section {
                wallet_table["entropy_source"] = toml_edit::value(self.wallet_entropy_source.clone());
                wallet_table["entropy_length"] = toml_edit::value(self.wallet_entropy_length as i64);
                wallet_table["bip"] = toml_edit::value(self.wallet_bip as i64);
                wallet_table["address_count"] = toml_edit::value(self.wallet_address_count as i64);
                wallet_table["hardened_address"] = toml_edit::value(self.wallet_hardened_address);
            }
        }
    
        {
            let gui_section = doc["gui"].or_insert(toml_edit::Item::Table(Default::default()));
            if let toml_edit::Item::Table(gui_table) = gui_section {
                gui_table["save_size"] = toml_edit::value(self.gui_save_size);
                gui_table["last_width"] = toml_edit::value(self.gui_last_width as i64);
                gui_table["last_height"] = toml_edit::value(self.gui_last_height as i64);
                gui_table["maximized"] = toml_edit::value(self.gui_maximized);
                gui_table["theme"] = toml_edit::value(self.gui_theme.clone());
                gui_table["language"] = toml_edit::value(self.gui_language.clone());
                gui_table["search"] = toml_edit::value(self.gui_search.clone());
                gui_table["notification_timeout"] = toml_edit::value(self.gui_notification_timeout as i64);
                gui_table["mnemonic_length"] = toml_edit::value(self.gui_mnemonic_length as i64);
            }
        }
    
        {
            let anu_section = doc["anu"].or_insert(toml_edit::Item::Table(Default::default()));
            if let toml_edit::Item::Table(anu_table) = anu_section {
                anu_table["enabled"] = toml_edit::value(self.anu_enabled);
                anu_table["data_format"] = toml_edit::value(self.anu_data_format.clone());
                anu_table["array_length"] = toml_edit::value(self.anu_array_length as i64);
                anu_table["hex_block_size"] = toml_edit::value(self.anu_hex_block_size as i64);
                anu_table["log"] = toml_edit::value(self.anu_log);
            }
        }
    
        {
            let proxy_section = doc["proxy"].or_insert(toml_edit::Item::Table(Default::default()));
            if let toml_edit::Item::Table(proxy_table) = proxy_section {
                proxy_table["status"] = toml_edit::value(self.proxy_status.clone());
                proxy_table["server_address"] = toml_edit::value(self.proxy_server_address.clone());
                proxy_table["server_port"] = toml_edit::value(self.proxy_server_port as i64);
                proxy_table["use_pac"] = toml_edit::value(self.proxy_use_pac);
                proxy_table["script_address"] = toml_edit::value(self.proxy_script_address.clone());
                proxy_table["login_credentials"] = toml_edit::value(self.proxy_login_credentials);
                proxy_table["login_username"] = toml_edit::value(self.proxy_login_username.clone());
                proxy_table["login_password"] = toml_edit::value(self.proxy_login_password.clone());
                proxy_table["use_ssl"] = toml_edit::value(self.proxy_use_ssl);
                proxy_table["ssl_certificate"] = toml_edit::value(self.proxy_ssl_certificate.clone());
            }
        }
    
        let toml_str = doc.to_string();
    
        let mut file = fs::File::create(&local_config_file)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to create config file: {}", e)))
            .expect("Problem with local config file");
        
        file.write_all(toml_str.as_bytes())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to write to config file: {}", e)))
            .expect("can not write to local config file");
        
    }

}

#[derive(Debug, Default)]
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

impl DerivationPath {
    fn default() -> Self {
        Self {
            bip: Some(44),
            hardened_bip: Some(true),
            coin: Some(0),
            hardened_coin: Some(true),
            address: Some(0),
            hardened_address: Some(true),
            purpose: Some(0),
        }
    }

    fn update_field(&mut self, field: &str, value: Option<FieldValue>) {
        match field {
            "bip" => self.bip = value.and_then(|v| v.into_u32()),
            "hardened_bip" => self.hardened_bip = value.and_then(|v| v.into_bool()),
            "coin" => self.coin = value.and_then(|v| v.into_u32()),
            "hardened_coin" => self.hardened_coin = value.and_then(|v| v.into_bool()),
            "address" => self.address = value.and_then(|v| v.into_u32()),
            "hardened_address" => self.hardened_address = value.and_then(|v| v.into_bool()),
            "purpose" => self.purpose = value.and_then(|v| v.into_u32()),
            _ => eprintln!("{}", &t!("error.DP.read")),
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

struct CryptoAddresses {
    coin_name: String,
    derivation_path: String,
    address: String,
    public_key: String,
    private_key: String,
}


// BASIC -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


fn print_program_info() {
    let current_time = SystemTime::now();
    let timestamp = current_time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

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

// GUI -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


fn main() {
    print_program_info();

    os::detect_os_and_user_dir();

    if let Err(err) = os::create_local_files() {
        eprintln!("Error creating local config files: {}", err);
    }

    let application = adw::Application::builder()
        .application_id("wtf.r_o0_t.qr2m")
        .build();

    let state = std::sync::Arc::new(std::sync::Mutex::new(AppState::new()));

    application.connect_activate(clone!(
        #[strong] state,
        move |app| {
            create_main_window(&app, state.clone());
        }
    ));

    setup_app_actions(&application, state.clone());

    application.run();
}

fn setup_app_actions(
    application: &adw::Application, 
    state: std::sync::Arc<std::sync::Mutex<AppState>>,
) {
    let new = gio::SimpleAction::new("new", None);
    let open = gio::SimpleAction::new("open", None);
    let save = gio::SimpleAction::new("save", None);
    let about = gio::SimpleAction::new("about", None);
    let settings = gio::SimpleAction::new("settings", None);
    let quit = gio::SimpleAction::new("quit", None);
    let test = gio::SimpleAction::new("test", None);
    
    new.connect_activate(clone!(
        #[weak] application,
        move |_action, _parameter| {
            let state = std::sync::Arc::new(std::sync::Mutex::new(AppState::new()));

            create_main_window(&application, state.clone());
        }
    ));

    open.connect_activate(move |_action, _parameter| {
        open_wallet_from_file();
    });

    save.connect_activate(|_action, _parameter| {
        save_wallet_to_file();
    });

    about.connect_activate(move |_action, _parameter| {
        create_about_window();
    });

    settings.connect_activate(clone!(
        #[strong] state,
        move |_action, _parameter| {
            let settings_window = create_settings_window(state.clone());
            settings_window.connect_close_request(clone!(
                #[strong] state,
                move |_| {
                    if let Ok(mut state) = state.lock() {
                        state.apply_theme();
                        state.apply_language();
                    }
                    glib::Propagation::Proceed
                }
            ));
            settings_window.show();
        }
    ));

    quit.connect_activate(clone!(
        #[weak] application,
        move |_action, _parameter| {
            application.quit();
        }
    ));

    test.connect_activate(|_action, _parameter| {
        anu_window();
    });

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
    application: &adw::Application, 
    state: std::sync::Arc<std::sync::Mutex<AppState>>,
) {
    println!("[+] {}", &t!("log.create_main_window").to_string());

    match AppSettings::load_settings() {
        Ok(_) => println!("Settings loaded"),
        Err(err) => eprintln!("{} \t {}", t!("error.settings.read").to_string(), err)
    };

    let app_settings = APPLICATION_SETTINGS.lock().unwrap();
    let gui_language = app_settings.gui_language.clone();
    let gui_theme = app_settings.gui_theme.clone();
    let window_width = app_settings.gui_last_width;
    let window_height = app_settings.gui_last_height;
    let wallet_bip = app_settings.wallet_bip;
    let default_mnemonic_length = app_settings.gui_mnemonic_length;

    os::switch_locale(&gui_language);
    
    let preferred_theme = match String::from(&gui_theme).as_str() {
        "System" => adw::ColorScheme::PreferLight,
        "Light" => adw::ColorScheme::ForceLight,
        "Dark" => adw::ColorScheme::PreferDark,
        _ => {
            eprintln!("{}", &t!("error.settings.parse", element = "gui_theme", value = gui_theme));
            adw::ColorScheme::PreferLight
        },
    };
    
    application.style_manager().set_color_scheme(preferred_theme);

    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title(&format!("{} {}", APP_DESCRIPTION.unwrap(), APP_VERSION.unwrap()))
        .default_width(window_width as i32)
        .default_height(window_height as i32)
        .show_menubar(true)
        .decorated(true)
        .build();

    qr2m_lib::setup_css();

    let header_bar = gtk::HeaderBar::new();
    let info_bar = gtk::Revealer::new();
    info_bar.set_transition_type(gtk::RevealerTransitionType::SlideDown);
    info_bar.set_hexpand(true);
    info_bar.add_css_class("info-bar");
    window.set_titlebar(Some(&header_bar));
    
    {
        if let Ok(mut state_guard) = state.lock() {
            state_guard.language = Some(gui_language);
            state_guard.theme = Some(gui_theme);
            state_guard.initialize_app_messages(info_bar.clone());
        }
    }

    let new_wallet_button = gtk::Button::new();
    let open_wallet_button = gtk::Button::new();
    let save_wallet_button = gtk::Button::new();
    let about_button = gtk::Button::new();
    let settings_button = gtk::Button::new();

    let theme_images = get_window_theme_icons();
    new_wallet_button.set_child(Some(&theme_images[0]));
    open_wallet_button.set_child(Some(&theme_images[1]));
    save_wallet_button.set_child(Some(&theme_images[2]));
    about_button.set_child(Some(&theme_images[3]));
    settings_button.set_child(Some(&theme_images[4]));
    
    new_wallet_button.set_tooltip_text(Some(&t!("UI.main.headerbar.wallet.new", value = "Ctrl+N").to_string()));
    open_wallet_button.set_tooltip_text(Some(&t!("UI.main.headerbar.wallet.open", value = "Ctrl+O").to_string()));
    save_wallet_button.set_tooltip_text(Some(&t!("UI.main.headerbar.wallet.save", value = "Ctrl+S").to_string()));
    about_button.set_tooltip_text(Some(&t!("UI.main.headerbar.about", value = "F1").to_string()));
    settings_button.set_tooltip_text(Some(&t!("UI.main.headerbar.settings", value = "F5").to_string()));

    header_bar.pack_start(&new_wallet_button);
    header_bar.pack_start(&open_wallet_button);
    header_bar.pack_start(&save_wallet_button);
    header_bar.pack_end(&settings_button);
    header_bar.pack_end(&about_button);
    
    // JUMP: Main: Settings button action
    settings_button.connect_clicked(clone!(
        #[strong] state,
        move |_| {
            let settings_window = create_settings_window(state.clone());
            settings_window.show();
        }
    ));
    
    about_button.connect_clicked(move |_| {
        create_about_window();
    });

    new_wallet_button.connect_clicked(clone!(
        #[weak] application,
        // #[strong] state,
        move |_| {
            let state = std::sync::Arc::new(std::sync::Mutex::new(AppState::new()));

            create_main_window(&application, state.clone());
        }
    ));

    save_wallet_button.connect_clicked(move |_| {
        save_wallet_to_file();
    });
    

    let stack = Stack::new();
    let stack_sidebar = StackSidebar::new();
    stack_sidebar.set_stack(&stack);


    // -.-. --- .--. -.-- .-. .. --. .... -
    // Sidebar 1: Seed
    // -.-. --- .--. -.-- .-. .. --. .... -
    // JUMP: Main: Sidebar 1: Seed
    let entropy_main_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    entropy_main_box.set_margin_top(10);
    entropy_main_box.set_margin_start(10);
    entropy_main_box.set_margin_end(10);
    entropy_main_box.set_margin_bottom(10);

    // Header
    let entropy_header_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let entropy_header_first_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let entropy_header_second_box = gtk::Box::new(gtk::Orientation::Vertical, 10);

    // Entropy source
    let entropy_source_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let entropy_source_frame = gtk::Frame::new(Some(&t!("UI.main.seed.entropy.source").to_string()));
    
    let anu_enabled = app_settings.anu_enabled;

    let valid_entropy_sources: Vec<&str> = if anu_enabled {
        VALID_ENTROPY_SOURCES.iter().cloned().collect()
    } else {
        VALID_ENTROPY_SOURCES.iter().filter(|&&x| x != "QRNG").cloned().collect()
    };

    let valid_entropy_source_as_strings: Vec<String> = valid_entropy_sources.iter().map(|&x| x.to_string()).collect();
    let valid_entropy_source_as_str_refs: Vec<&str> = valid_entropy_source_as_strings.iter().map(|s| s.as_ref()).collect();
    let entropy_source_dropdown = gtk::DropDown::from_strings(&valid_entropy_source_as_str_refs);
    
    let wallet_entropy_source = app_settings.wallet_entropy_source.clone();
    
    let default_entropy_source = valid_entropy_source_as_strings
        .iter()
        .position(|s| *s == wallet_entropy_source) 
        .unwrap_or(0);

    entropy_source_dropdown.set_selected(default_entropy_source.try_into().unwrap());
    entropy_source_box.set_hexpand(true);
    entropy_source_frame.set_hexpand(true);
    
    // Entropy length
    let entropy_length_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let entropy_length_frame = gtk::Frame::new(Some(&t!("UI.main.seed.entropy.length").to_string()));
    let valid_entropy_lengths_as_strings: Vec<String> = VALID_ENTROPY_LENGTHS.iter().map(|&x| x.to_string()).collect();
    let valid_entropy_lengths_as_str_refs: Vec<&str> = valid_entropy_lengths_as_strings.iter().map(|s| s.as_ref()).collect();
    let entropy_length_dropdown = gtk::DropDown::from_strings(&valid_entropy_lengths_as_str_refs);
    
    let wallet_entropy_length = app_settings.wallet_entropy_length;
    
    let default_entropy_length = valid_entropy_lengths_as_strings
        .iter()
        .position(|x| x.parse::<u32>().unwrap() == wallet_entropy_length)
        .unwrap_or(0);

    entropy_length_dropdown.set_selected(default_entropy_length.try_into().unwrap());
    entropy_length_box.set_hexpand(true);
    entropy_length_frame.set_hexpand(true);

    // RNG mnemonic passphrase length
    let mnemonic_passphrase_length_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let mnemonic_passphrase_items_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let mnemonic_passphrase_scale_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let mnemonic_passphrase_info_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let mnemonic_passphrase_length_frame = gtk::Frame::new(Some(&t!("UI.main.seed.mnemonic.length").to_string()));
    let mnemonic_passphrase_adjustment = gtk::Adjustment::new(
        8.0 * 32.0,
        8.0 * 2.0,
        8.0 * 128.0,
        1.0,
        100.0,
        0.0,
    );
    let mnemonic_passphrase_scale = gtk::Scale::new(gtk::Orientation::Horizontal, Some(&mnemonic_passphrase_adjustment));
    let mnemonic_passphrase_length_info = gtk::Entry::new();
        
    mnemonic_passphrase_items_box.set_hexpand(true);
    mnemonic_passphrase_scale_box.set_hexpand(true);
    mnemonic_passphrase_length_box.set_hexpand(true);
    mnemonic_passphrase_length_frame.set_hexpand(true);

    let value = entropy_source_dropdown.selected() as usize;
    let selected_entropy_source_value = VALID_ENTROPY_SOURCES.get(value);
    let source = selected_entropy_source_value.unwrap();

    mnemonic_passphrase_length_info.set_editable(false);
    mnemonic_passphrase_length_info.set_width_request(50);
    mnemonic_passphrase_length_info.set_input_purpose(gtk::InputPurpose::Digits);
    mnemonic_passphrase_length_info.set_text(&default_mnemonic_length.to_string());
    
    // Mnemonic passphrase
    let mnemonic_passphrase_main_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let mnemonic_passphrase_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let mnemonic_passphrase_frame = gtk::Frame::new(Some(&t!("UI.main.seed.mnemonic.pass").to_string()));
    let mnemonic_passphrase_text = gtk::Entry::new();
    mnemonic_passphrase_main_box.set_hexpand(true);
    mnemonic_passphrase_text.set_hexpand(true);

    let pixy = qr2m_lib::get_image_from_resources("theme/basic/dark/random.svg");
    let image = gtk::Picture::for_pixbuf(&pixy);

    let random_mnemonic_passphrase_button = gtk::Button::new();
    random_mnemonic_passphrase_button.set_child(Some(&image));

    if *source == "RNG+" {
        mnemonic_passphrase_length_box.set_visible(true);
        random_mnemonic_passphrase_button.set_visible(true);
    } else {
        mnemonic_passphrase_length_box.set_visible(false);
        random_mnemonic_passphrase_button.set_visible(false);
    }


    let seed_buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    seed_buttons_box.set_halign(gtk::Align::Center);

    // Generate entropy button
    let generate_entropy_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let generate_entropy_button = gtk::Button::new();
    generate_entropy_button.set_label(&t!("UI.main.seed.generate").to_string());
    generate_entropy_box.set_halign(gtk::Align::Center);
    generate_entropy_box.set_margin_top(10);

    // Delete entropy button
    let delete_entropy_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let delete_entropy_button = gtk::Button::new();
    delete_entropy_button.set_label(&t!("UI.main.seed.delete").to_string());
    delete_entropy_box.set_halign(gtk::Align::Center);
    delete_entropy_box.set_margin_top(10);


    // Body
    let body_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    
    // Entropy string
    let entropy_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let entropy_frame = gtk::Frame::new(Some(&t!("UI.main.seed.entropy").to_string()));
    let entropy_text = gtk::TextView::new();
    entropy_text.set_vexpand(true);
    entropy_text.set_hexpand(true);
    entropy_text.set_wrap_mode(gtk::WrapMode::Char);
    entropy_frame.set_child(Some(&entropy_text));
    entropy_box.append(&entropy_frame);
    entropy_text.set_editable(false);
    entropy_text.set_left_margin(5);
    entropy_text.set_top_margin(5);

    // Mnemonic words
    let mnemonic_words_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let mnemonic_words_frame = gtk::Frame::new(Some(&t!("UI.main.seed.mnemonic.words").to_string()));
    let mnemonic_words_text = gtk::TextView::new();
    mnemonic_words_box.set_hexpand(true);
    mnemonic_words_text.set_vexpand(true);
    mnemonic_words_text.set_hexpand(true);
    mnemonic_words_text.set_editable(false);
    mnemonic_words_text.set_left_margin(5);
    mnemonic_words_text.set_top_margin(5);
    mnemonic_words_text.set_wrap_mode(gtk::WrapMode::Word);
    
    // Seed
    let seed_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let seed_frame = gtk::Frame::new(Some(&t!("UI.main.seed").to_string()));
    let seed_text = gtk::TextView::new();
    seed_box.set_hexpand(true);
    seed_text.set_editable(false);
    seed_text.set_vexpand(true);
    seed_text.set_hexpand(true);
    seed_text.set_left_margin(5);
    seed_text.set_top_margin(5);
    seed_text.set_wrap_mode(gtk::WrapMode::Char);

    // Connections
    entropy_source_frame.set_child(Some(&entropy_source_dropdown));
    entropy_length_frame.set_child(Some(&entropy_length_dropdown));
    mnemonic_passphrase_length_frame.set_child(Some(&mnemonic_passphrase_items_box));
    mnemonic_passphrase_length_box.append(&mnemonic_passphrase_length_frame);
    mnemonic_passphrase_items_box.append(&mnemonic_passphrase_scale_box);
    mnemonic_passphrase_items_box.append(&mnemonic_passphrase_info_box);


    mnemonic_passphrase_scale_box.append(&mnemonic_passphrase_scale);
    mnemonic_passphrase_info_box.append(&mnemonic_passphrase_length_info);

    generate_entropy_box.append(&generate_entropy_button);
    delete_entropy_box.append(&delete_entropy_button);
    entropy_source_box.append(&entropy_source_frame);
    entropy_length_box.append(&entropy_length_frame);
    entropy_header_first_box.append(&entropy_source_box);
    entropy_header_first_box.append(&entropy_length_box);
    entropy_header_second_box.append(&mnemonic_passphrase_main_box);
    entropy_header_second_box.append(&mnemonic_passphrase_length_box);
    entropy_header_box.append(&entropy_header_first_box);
    entropy_header_box.append(&entropy_header_second_box);

    
    seed_buttons_box.append(&generate_entropy_box);
    seed_buttons_box.append(&delete_entropy_box);
    
    
    
    entropy_header_box.append(&seed_buttons_box);
    mnemonic_words_frame.set_child(Some(&mnemonic_words_text));
    mnemonic_passphrase_frame.set_child(Some(&mnemonic_passphrase_content_box));
    mnemonic_passphrase_content_box.append(&mnemonic_passphrase_text);
    mnemonic_passphrase_content_box.append(&random_mnemonic_passphrase_button);
    seed_frame.set_child(Some(&seed_text));
    mnemonic_words_box.append(&mnemonic_words_frame);
    mnemonic_passphrase_main_box.append(&mnemonic_passphrase_frame);
    seed_box.append(&seed_frame);
    body_box.append(&entropy_box);
    body_box.append(&mnemonic_words_box);
    body_box.append(&seed_box);
    entropy_main_box.append(&entropy_header_box);
    entropy_main_box.append(&body_box);
    
    // Start Seed sidebar
    stack.add_titled(
        &entropy_main_box,
        Some("sidebar-seed"), 
        &t!("UI.main.seed").to_string());


    // -.-. --- .--. -.-- .-. .. --. .... -
    // Sidebar 2: Coin
    // -.-. --- .--. -.-- .-. .. --. .... -
    // JUMP: Main: Sidebar 2: Coin
    let coin_main_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let coin_main_content_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    // coin_frame.set_child(Some(&coin_main_content_box));
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
    let coin_filter_main_frame = gtk::Frame::new(Some(&t!("UI.main.coin.filter").to_string()));
    let coin_filter_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    coin_filter_main_frame.set_child(Some(&coin_filter_content_box));
    coin_filter_main_box.append(&coin_filter_main_frame);
    coin_filter_main_box.set_hexpand(true);
    coin_filter_main_frame.set_hexpand(true);
    coin_filter_content_box.set_hexpand(true);

    let filter_top10_coins_button_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let filter_top10_coins_button = gtk::Button::with_label(&t!("UI.main.coin.filter.status.top", value = 10).to_string());
    filter_top10_coins_button_box.append(&filter_top10_coins_button);
    coin_filter_content_box.append(&filter_top10_coins_button_box);
    filter_top10_coins_button_box.set_hexpand(true);
    
    let filter_top100_coins_button_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let filter_top100_coins_button = gtk::Button::with_label(&t!("UI.main.coin.filter.status.top", value = 100).to_string());
    filter_top100_coins_button_box.append(&filter_top100_coins_button);
    coin_filter_content_box.append(&filter_top100_coins_button_box);
    filter_top100_coins_button_box.set_hexpand(true);
    
    let filter_verified_coins_button_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let filter_verified_coins_button = gtk::Button::with_label(&t!("UI.main.coin.filter.status.verified", value = coin_db::COIN_STATUS_VERIFIED).to_string());
    filter_verified_coins_button_box.append(&filter_verified_coins_button);
    coin_filter_content_box.append(&filter_verified_coins_button_box);
    filter_verified_coins_button_box.set_hexpand(true);
    
    let filter_not_verified_coins_button_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let filter_not_verified_coins_button = gtk::Button::with_label(&t!("UI.main.coin.filter.status.not-verified", value = coin_db::COIN_STATUS_NOT_VERIFIED).to_string());
    filter_not_verified_coins_button_box.append(&filter_not_verified_coins_button);
    coin_filter_content_box.append(&filter_not_verified_coins_button_box);
    filter_not_verified_coins_button_box.set_hexpand(true);
    
    let filter_in_plan_coins_button_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let filter_in_plan_coins_button = gtk::Button::with_label(&t!("UI.main.coin.filter.status.future", value = coin_db::COIN_STATUS_IN_PLAN).to_string());
    filter_in_plan_coins_button_box.append(&filter_in_plan_coins_button);
    coin_filter_content_box.append(&filter_in_plan_coins_button_box);
    filter_in_plan_coins_button_box.set_hexpand(true);
    
    let filter_not_supported_coins_button_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let filter_not_supported_coins_button = gtk::Button::with_label(&t!("UI.main.coin.filter.status.not-supported", value = coin_db::COIN_STATUS_NOT_SUPPORTED).to_string());
    filter_not_supported_coins_button_box.append(&filter_not_supported_coins_button);
    coin_filter_content_box.append(&filter_not_supported_coins_button_box);
    filter_not_supported_coins_button_box.set_hexpand(true);
    coin_main_content_box.append(&coin_filter_main_box);


    // Coin search
    let coin_filter_main_frame = gtk::Frame::new(Some(&t!("UI.main.coin.search").to_string()));
    let search_coin_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    coin_filter_main_frame.set_child(Some(&search_coin_content_box));
    
    // Search entry
    let coin_search = gtk::SearchEntry::new();
    search_coin_content_box.append(&coin_search);
    coin_search.set_hexpand(true);

    // Advance search
    let advance_search_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let advance_search_label = gtk::Label::new(Some(&t!("UI.main.coin.search.advance").to_string()));
    let advance_search_checkbox = gtk::CheckButton::new();

    advance_search_content_box.append(&advance_search_label);
    advance_search_content_box.append(&advance_search_checkbox);

    search_coin_content_box.append(&advance_search_content_box);
    
    // Search filter
    let advance_search_filter_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let valid_coin_search_filter_as_strings: Vec<String> = VALID_COIN_SEARCH_PARAMETER.iter().map(|&x| x.to_string()).collect();
    let valid_coin_search_filter_as_str_refs: Vec<&str> = valid_coin_search_filter_as_strings.iter().map(|s| s.as_ref()).collect();
    let coin_search_filter_dropdown = gtk::DropDown::from_strings(&valid_coin_search_filter_as_str_refs);
    
    let gui_search = app_settings.gui_search.clone();

    let default_coin_search_filter = valid_coin_search_filter_as_strings
        .iter()
        .position(|s| *s == gui_search) 
        .unwrap_or(0);
    if let Ok(index) = default_coin_search_filter.try_into() {
        coin_search_filter_dropdown.set_selected(index);
    } else {
        eprintln!("Invalid index for coin_search_filter_dropdown");
        coin_search_filter_dropdown.set_selected(0);
    }    
    // coin_search_filter_dropdown.set_selected(default_coin_search_filter.try_into().unwrap());
    advance_search_filter_box.set_visible(false);
    advance_search_filter_box.append(&coin_search_filter_dropdown);
    search_coin_content_box.append(&advance_search_filter_box);
    coin_main_content_box.append(&coin_filter_main_frame);
    coin_search.set_placeholder_text(Some(&t!("UI.main.coin.search.text", value = valid_coin_search_filter_as_strings[default_coin_search_filter]).to_string()));


    // Coin treeview
    let scrolled_window = gtk::ScrolledWindow::new();
    let coin_frame = gtk::Frame::new(Some(&t!("UI.main.coin").to_string()));

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
            filtered_store.set(&iter, &[
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
                (12, &found_coin.UCID),
                (13, &found_coin.cmc_top),
            ]);
        }
    }

    coin_treeview.borrow().set_model(Some(&*filtered_store));
    scrolled_window.set_child(Some(&*coin_treeview.borrow()));
    coin_frame.set_child(Some(&scrolled_window));
    coin_main_content_box.append(&coin_frame);

    // Generate master keys button
    let generate_master_keys_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let generate_master_keys_button = gtk::Button::new();
    generate_master_keys_button.set_label(&t!("UI.main.coin.generate").to_string());
    generate_master_keys_box.set_halign(gtk::Align::Center);
    generate_master_keys_box.append(&generate_master_keys_button);
    coin_main_content_box.append(&generate_master_keys_box);

    // Master private keys entries
    let master_keys_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let master_xprv_frame = gtk::Frame::new(Some(&t!("UI.main.coin.keys.priv").to_string()));
    let master_xpub_frame = gtk::Frame::new(Some(&t!("UI.main.coin.keys.pub").to_string()));
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
        &t!("UI.main.coin").to_string()
    );


    // -.-. --- .--. -.-- .-. .. --. .... -
    // Sidebar 3: Address
    // -.-. --- .--. -.-- .-. .. --. .... -
    // JUMP: Main: Sidebar 3: Address
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
    let main_bip_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation.bip").to_string()));
    let main_coin_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation.coin").to_string()));
    let main_address_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation.address").to_string()));
    let main_purpose_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation.purpose").to_string()));

    main_bip_frame.set_hexpand(true);
    main_coin_frame.set_hexpand(true);
    main_address_frame.set_hexpand(true);
    main_purpose_frame.set_hexpand(true);
    
    let bip_hardened_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation.hard").to_string()));
    let coin_hardened_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation.hard").to_string()));
    let address_hardened_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation.hard").to_string()));

    let valid_bip_as_string: Vec<String> = VALID_BIP_DERIVATIONS.iter().map(|&x| x.to_string()).collect();
    let valid_bip_as_ref: Vec<&str> = valid_bip_as_string.iter().map(|s| s.as_ref()).collect();
    let bip_dropdown = gtk::DropDown::from_strings(&valid_bip_as_ref);
    
    let default_index = VALID_BIP_DERIVATIONS.iter().position(|&x| x == wallet_bip).unwrap_or_else(|| {
        eprintln!("{}", &t!("error.bip.value", value = wallet_bip));
        1 // BIP44
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
    
    let adjustment = gtk::Adjustment::new(
        0.0,
        0.0,
        2147483647.0,
        1.0,
        100.0,
        0.0,
    );
    
    let address_spinbutton = gtk::SpinButton::new(Some(&adjustment), 1.0, 0);
    address_spinbutton.set_hexpand(true);
    
    let address_hardened_checkbox = gtk::CheckButton::new();
    address_hardened_checkbox.set_active(true);
    address_hardened_checkbox.set_halign(gtk::Align::Center);
    
    let valid_wallet_purpose_as_strings: Vec<String> = VALID_WALLET_PURPOSE.iter().map(|&x| x.to_string()).collect();
    let valid_wallet_purpose_as_ref: Vec<&str> = valid_wallet_purpose_as_strings.iter().map(|s| s.as_ref()).collect();
    let purpose_dropdown = gtk::DropDown::from_strings(&valid_wallet_purpose_as_ref);
    purpose_dropdown.set_selected(0); // Internal
    purpose_dropdown.set_hexpand(true);

    bip_hardened_frame.set_child(Some(&bip_hardened_checkbox));
    coin_hardened_frame.set_child(Some(&coin_hardened_checkbox));
    address_hardened_frame.set_child(Some(&address_hardened_checkbox));


    // Derivation label
    let derivation_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let derivation_label_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation").to_string()));
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

    
    // Generate address button
    let generate_addresses_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let generate_addresses_button = gtk::Button::with_label(&t!("UI.main.address.generate").to_string());

    generate_addresses_button_box.append(&generate_addresses_button);
    generate_addresses_button_box.set_halign(gtk::Align::Center);


    // Address tree
    let address_scrolled_window = gtk::ScrolledWindow::new();
    let address_treeview_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let address_treeview_frame = gtk::Frame::new(Some(&t!("UI.main.address").to_string()));
    address_treeview_frame.set_hexpand(true);
    address_treeview_frame.set_vexpand(true);

    let address_store = gtk::ListStore::new(&[
        gtk4::glib::Type::STRING, // Coin
        gtk4::glib::Type::STRING, // Derivation Path
        gtk4::glib::Type::STRING, // Address
        gtk4::glib::Type::STRING, // Public Key
        gtk4::glib::Type::STRING, // Private Key
    ]);

    let address_treeview = gtk::TreeView::new();
    address_treeview.set_headers_visible(true);
    let columns = [
        &t!("UI.main.address.table.coin").to_string(), 
        &t!("UI.main.address.table.path").to_string(), 
        &t!("UI.main.address.table.address").to_string(), 
        &t!("UI.main.address.table.pub").to_string(), 
        &t!("UI.main.address.table.priv").to_string()
    ];

    for (i, column_title) in columns.iter().enumerate() {
        let column = gtk::TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();
        column.set_title(column_title);
        column.pack_start(&cell, true);
        column.add_attribute(&cell, "text", i as i32);
        address_treeview.append_column(&column);
    }

    address_treeview.set_model(Some(&address_store));

    // Address options main box
    let address_options_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let address_options_content = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    address_options_box.append(&address_options_content);
    
    // Address count
    let wallet_address_count = app_settings.wallet_address_count;

    let address_options_frame = gtk::Frame::new(Some(&t!("UI.main.address.options.count").to_string()));
    let address_options_address_count_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let address_options_adjustment = gtk::Adjustment::new(
        wallet_address_count as f64,
        1.0,
        2147483647.0,
        1.0,
        10.0,
        0.0,
    );
    let address_options_spinbutton = gtk::SpinButton::new(Some(&address_options_adjustment), 1.0, 0);
    
    address_options_frame.set_child(Some(&address_options_address_count_box));
    address_options_address_count_box.append(&address_options_spinbutton);


    // Hardened address
    let address_options_hardened_address_frame = gtk::Frame::new(Some(&t!("UI.main.address.options.hardened").to_string()));
    let address_options_hardened_address_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let address_options_hardened_address_checkbox = gtk::CheckButton::new();
    let wallet_hardened_address = app_settings.wallet_hardened_address;

    address_options_hardened_address_checkbox.set_active(wallet_hardened_address);
    address_options_hardened_address_box.set_halign(gtk4::Align::Center);
    address_options_hardened_address_frame.set_child(Some(&address_options_hardened_address_box));
    address_options_hardened_address_box.append(&address_options_hardened_address_checkbox);
    
    // Clear address
    // let address_options_clear_addresses_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let address_options_clear_addresses_button = gtk::Button::with_label(&t!("UI.main.address.options.clean").to_string());
    // address_options_clear_addresses_box.set_halign(gtk4::Align::Center);
    // address_options_clear_addresses_box.append(&address_options_clear_addresses_button);
    address_options_clear_addresses_button.set_size_request(200, 2);
    
    // Connections
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
    address_options_content.append(&address_options_hardened_address_frame);
    address_options_content.append(&address_options_clear_addresses_button);
    main_address_box.append(&derivation_box);
    main_address_box.append(&derivation_label_box);
    main_address_box.append(&generate_addresses_button_box);
    main_address_box.append(&address_treeview_box);
    main_address_box.append(&address_options_box);
    
    stack.add_titled(
        &main_address_box,
        Some("sidebar-address"), 
        &t!("UI.main.address").to_string()
    );
    
    

    // JUMP: Main: ACTIONS

    // JUMP: Main: Open Wallet
    open_wallet_button.connect_clicked(clone!(
        #[weak] entropy_text,
        #[weak] mnemonic_passphrase_text,
        move |_| {
            let (entropy, passphrase) = open_wallet_from_file();

            if !entropy.is_empty() {
                println!("\t Wallet entropy: {:?}", entropy);
                entropy_text.buffer().set_text(&entropy);
            }

            match passphrase {
                Some(pass) => {
                    println!("\t Mnemonic passphrase: {:?}", pass);
                    mnemonic_passphrase_text.buffer().set_text(&pass);
                },
                None => {
                    println!("\t No Mnemonic passphrase available");
                },
            }
        }
    ));


    // JUMP: Generate Seed button
    generate_entropy_button.connect_clicked(clone!(
        #[weak] entropy_source_dropdown,
        #[weak] entropy_text,
        #[weak] entropy_length_dropdown,
        #[weak] mnemonic_words_text,
        #[weak] mnemonic_passphrase_text,
        #[weak] seed_text,
        #[strong] state,
        move |_| {
            let selected_entropy_source_index = entropy_source_dropdown.selected() as usize;
            let selected_entropy_length_index = entropy_length_dropdown.selected() as usize;
            let selected_entropy_source_value = VALID_ENTROPY_SOURCES.get(selected_entropy_source_index);
            let selected_entropy_length_value = VALID_ENTROPY_LENGTHS.get(selected_entropy_length_index);
            let source = selected_entropy_source_value.unwrap().to_string();
            let entropy_length = selected_entropy_length_value.unwrap(); 

            mnemonic_words_text.buffer().set_text("");
            entropy_text.buffer().set_text("");
            seed_text.buffer().set_text("");

            let pre_entropy = keys::generate_entropy(
                &source,
                *entropy_length as u64,
                // Some(std::sync::Arc::clone(&state)),
            );
                
            if !pre_entropy.is_empty() {
                let checksum = qr2m_lib::calculate_checksum_for_entropy(&pre_entropy, entropy_length);
                println!("\t Entropy checksum: {:?}", checksum);

                let full_entropy = format!("{}{}", &pre_entropy, &checksum);

                println!("\t Final entropy: {:?}", full_entropy);
                entropy_text.buffer().set_text(&full_entropy);
                
                let mnemonic_words = keys::generate_mnemonic_words(&full_entropy);
                mnemonic_words_text.buffer().set_text(&mnemonic_words);
                
                let passphrase_text = mnemonic_passphrase_text.text().to_string();

                let seed = keys::generate_bip39_seed(&pre_entropy, &passphrase_text);
                let seed_hex = hex::encode(&seed[..]);
                seed_text.buffer().set_text(&seed_hex.to_string());
                
                println!("\t Seed (hex): {:?}", seed_hex);

                let mut wallet_settings = WALLET_SETTINGS.lock().unwrap();
                wallet_settings.entropy_checksum = Some(checksum.clone());
                wallet_settings.entropy_string = Some(full_entropy.clone());
                wallet_settings.mnemonic_passphrase = Some(passphrase_text.clone());
                wallet_settings.mnemonic_words = Some(mnemonic_words.clone());
                wallet_settings.seed = Some(seed_hex.clone());

            } else {
                eprintln!("{}", &t!("error.entropy.empty"));
                let lock_state = state.lock().unwrap();
                AppState::show_message(&lock_state, t!("error.entropy.empty").to_string(), gtk::MessageType::Warning);
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
        // #[weak] save_wallet_button,
        move |_| {
            mnemonic_passphrase_text.buffer().set_text("");
            entropy_text.buffer().set_text("");
            mnemonic_words_text.buffer().set_text("");
            seed_text.buffer().set_text("");
            master_private_key_text.buffer().set_text("");
            master_public_key_text.buffer().set_text("");
            address_store.clear();
    
            // save_wallet_button.set_sensitive(false);
    }));
    
    random_mnemonic_passphrase_button.connect_clicked(clone!(
        #[weak] mnemonic_passphrase_text,
        #[weak] mnemonic_passphrase_scale,
        move |_| {
            let scale_value = mnemonic_passphrase_scale.value() as u32;

            let mnemonic_rng_string: String = (0..scale_value)
                        .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
                        .collect();
            println!("\t RNG Mnemonic Passphrase: {:?}", mnemonic_rng_string);
            mnemonic_passphrase_text.set_text(&mnemonic_rng_string);
        }
    ));


    // JUMP: Generate Master Keys button
    generate_master_keys_button.connect_clicked(clone!(
        #[strong] coin_entry,
        #[weak] seed_text,
        #[weak] coin_treeview,
        #[weak] master_private_key_text,
        #[strong] state,
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
                    let UCID = model.get_value(&iter, 12);
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
                        Ok(UCID),
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
                        UCID.get::<String>(),
                        cmc_top.get::<String>(),
                    ) {
                        master_private_key_text.buffer().set_text("");
                        master_public_key_text.buffer().set_text("");
    
                        println!("\n#### Coin info ####");
                        println!("status: {}", status);
                        println!("index: {}", coin_index);
                        println!("coin_symbol: {}", coin_symbol);
                        println!("coin_name: {}", coin_name);
                        println!("key_derivation: {}", key_derivation);
                        println!("hash: {}", hash);
                        println!("private_header: {}", private_header);
                        println!("public_header: {}", public_header);
                        println!("public_key_hash: {}", public_key_hash);
                        println!("script_hash: {}", script_hash);
                        println!("wallet_import_format: {}", wallet_import_format);
                        println!("EVM: {}", evm);
                        println!("UCID: {}", UCID);
                        println!("cmc_top: {}", cmc_top);

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
                            },
                            Err(err) => {
                                
                                let lock_state = state.lock().unwrap();
                                AppState::show_message(&lock_state, t!("error.master.create").to_string(), gtk::MessageType::Warning);
                                eprintln!("{}: {}", &t!("error.master.create"), err)
                            },
                        }
    
                        coin_entry.set_text(&coin_index);

                        let mut wallet_settings = WALLET_SETTINGS.lock().unwrap();
                        wallet_settings.public_key_hash = Some(public_key_hash.clone());
                        wallet_settings.wallet_import_format = Some(wallet_import_format.to_string());
                        wallet_settings.key_derivation = Some(key_derivation.to_string());
                        wallet_settings.hash = Some(hash.to_string());
                        wallet_settings.coin_index = Some(coin_index.parse().unwrap());
                        wallet_settings.coin_name = Some(coin_name.parse().unwrap());
                    }  
                }
            } else {
                let lock_state = state.lock().unwrap();
                AppState::show_message(&lock_state, t!("error.entropy.seed").to_string(), gtk::MessageType::Warning);
            }
        }
    ));

    entropy_source_dropdown.connect_selected_notify(clone!(
        #[weak] generate_entropy_button,
        #[weak] random_mnemonic_passphrase_button,
        move |entropy_source_dropdown| {
            let value = entropy_source_dropdown.selected() as usize;
            let selected_entropy_source_value = VALID_ENTROPY_SOURCES.get(value);
            let source = selected_entropy_source_value.unwrap();
    
            if *source == "RNG+" {
                mnemonic_passphrase_length_box.set_visible(true);
                random_mnemonic_passphrase_button.set_visible(true);
            } else {
                mnemonic_passphrase_length_box.set_visible(false);
                random_mnemonic_passphrase_button.set_visible(false);
            }

            if *source == "File" {
                generate_entropy_button.set_label(&t!("UI.main.seed.generate.file").to_string());
            } else {
                generate_entropy_button.set_label(&t!("UI.main.seed.generate").to_string());
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
    }));

    coin_search_filter_dropdown.connect_selected_notify(clone!(
        #[weak] coin_search,
        move |dropdown| {
            let selected: usize = dropdown.selected().try_into().unwrap_or(0);
            coin_search.set_placeholder_text(Some(&t!("UI.main.coin.search.text", value = VALID_COIN_SEARCH_PARAMETER[selected])));
            coin_search.set_text("");
    }));
    
    mnemonic_passphrase_text.connect_changed(clone!(
        // #[weak] generate_entropy_button,
        #[weak] entropy_text,
        #[weak] mnemonic_words_text,
        #[weak] seed_text,
        // #[weak] mnemonic_passphrase_length_info,
        move |mnemonic_passphrase_text| {
            let entropy_buffer = entropy_text.buffer();
            let start_iter = entropy_buffer.start_iter();
            let end_iter = entropy_buffer.end_iter();
            let entropy_text = entropy_buffer.text(&start_iter, &end_iter, false);

            if entropy_text != "" {
                let entropy_length = entropy_text.len();
                let cut_entropy = entropy_length / 32;
                let new_pre_entropy = entropy_text[0..entropy_length - cut_entropy].to_string();

                let seed = keys::generate_bip39_seed(&new_pre_entropy, &mnemonic_passphrase_text.buffer().text());
                let seed_hex = hex::encode(&seed[..]);
                seed_text.buffer().set_text(&seed_hex.to_string());

                let final_entropy = entropy_text.clone().to_string();
                let mnemonic_words_buffer = mnemonic_words_text.buffer();
                let start_iter = mnemonic_words_buffer.start_iter();
                let end_iter = mnemonic_words_buffer.end_iter();
                let final_mnemonic_words = mnemonic_words_buffer.text(&start_iter, &end_iter, false).to_string();
                let final_mnemonic_passphrase = mnemonic_passphrase_text.buffer().text().to_string();

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
        #[weak] random_mnemonic_passphrase_button,
        move |mnemonic_passphrase_scale| {
            let scale_value = mnemonic_passphrase_scale.value() as u32;
            mnemonic_passphrase_length_info.set_text(&scale_value.to_string());
            random_mnemonic_passphrase_button.emit_by_name::<()>("clicked", &[]);
        }
    ));

    // JUMP: Main: Entropy change by import
    let buffer = entropy_text.buffer();
    buffer.connect_changed(clone!(
        #[weak] entropy_text,
        #[weak] mnemonic_passphrase_text,
        move |_| {

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

                let seed = keys::generate_bip39_seed(&pre_entropy, &mnemonic_passphrase_text.buffer().text());
                let seed_hex = hex::encode(&seed[..]);
                seed_text.buffer().set_text(&seed_hex.to_string());
                
                println!("\t Seed (hex): {:?}", seed_hex);
            }
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
            let selected_search_parameter = VALID_COIN_SEARCH_PARAMETER.get(selected).unwrap_or(&"Name");
            let min_search_length = if selected_search_parameter == &"Index" { 1 } else { 2 };

            if search_text.len() >= min_search_length {
                let store = coin_store.borrow();
                let matching_coins = coin_db::fetch_coins_from_database(selected_search_parameter, &store, &search_text);

                if !matching_coins.is_empty() {
                    let store = coin_tree_store.borrow_mut();
                    store.clear();

                    for found_coin in matching_coins {
                        let iter = store.append(None);
                        store.set(&iter, &[
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
                            (12, &found_coin.UCID),
                            (13, &found_coin.cmc_top),
                        ]);
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
            let matching_coins = coin_db::fetch_coins_from_database(search_parameter, &store, &search_text);

            let store = coin_tree_store.borrow_mut();
            store.clear();

            if !matching_coins.is_empty() {
                for found_coin in matching_coins {
                    let iter = store.append(None);
                    store.set(&iter, &[
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
                        (12, &found_coin.UCID),
                        (13, &found_coin.cmc_top),
                    ]);
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
            let matching_coins = coin_db::fetch_coins_from_database(search_parameter, &store, &search_text);

            let store = coin_tree_store.borrow_mut();
            store.clear();

            if !matching_coins.is_empty() {
                for found_coin in matching_coins {
                    let iter = store.append(None);
                    store.set(&iter, &[
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
                        (12, &found_coin.UCID),
                        (13, &found_coin.cmc_top),
                    ]);
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
            let matching_coins = coin_db::fetch_coins_from_database(search_parameter, &store, &search_text);

            let store = coin_tree_store.borrow_mut();
            store.clear();

            if !matching_coins.is_empty() {
                for found_coin in matching_coins {
                    let iter = store.append(None);
                    store.set(&iter, &[
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
                        (12, &found_coin.UCID),
                        (13, &found_coin.cmc_top),
                    ]);
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
            let matching_coins = coin_db::fetch_coins_from_database(search_parameter, &store, &search_text);

            let store = coin_tree_store.borrow_mut();
            store.clear();

            if !matching_coins.is_empty() {
                for found_coin in matching_coins {
                    let iter = store.append(None);
                    store.set(&iter, &[
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
                        (12, &found_coin.UCID),
                        (13, &found_coin.cmc_top),
                    ]);
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
            let matching_coins = coin_db::fetch_coins_from_database(search_parameter, &store, &search_text);

            let store = coin_tree_store.borrow_mut();
            store.clear();

            if !matching_coins.is_empty() {
                for found_coin in matching_coins {
                    let iter = store.append(None);
                    store.set(&iter, &[
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
                        (12, &found_coin.UCID),
                        (13, &found_coin.cmc_top),
                    ]);
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
            let matching_coins = coin_db::fetch_coins_from_database(search_parameter, &store, &search_text);

            let store = coin_tree_store.borrow_mut();
            store.clear();

            if !matching_coins.is_empty() {
                for found_coin in matching_coins {
                    let iter = store.append(None);
                    store.set(&iter, &[
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
                        (12, &found_coin.UCID),
                        (13, &found_coin.cmc_top),
                    ]);
                }
                coin_treeview.borrow().set_model(Some(&*store));
            } else {
                store.clear();
            }
        }
    });

    let derivation_path = std::rc::Rc::new(std::cell::RefCell::new(DerivationPath::default()));
    let dp_clone = std::rc::Rc::clone(&derivation_path);

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
    
            dp_clone.borrow_mut().update_field("bip", Some(FieldValue::U32(*bip)));
            update_derivation_label(*dp_clone.borrow(), derivation_label_text)
        }
    ));
        
    let dp_clone = std::rc::Rc::clone(&derivation_path);
    
    bip_hardened_checkbox.connect_active_notify(clone!(
        #[weak] derivation_label_text,
        #[weak] bip_hardened_checkbox,
        move |_| {
            dp_clone.borrow_mut().update_field("hardened_bip", Some(FieldValue::Bool(bip_hardened_checkbox.is_active())));
            update_derivation_label(*dp_clone.borrow(), derivation_label_text)
        }
    ));
        
    let dp_clone2 = std::rc::Rc::clone(&derivation_path);
    
    coin_hardened_checkbox.connect_active_notify(clone!(
        #[weak] derivation_label_text,
        #[weak] coin_hardened_checkbox,
        move |_| {
            dp_clone2.borrow_mut().update_field("hardened_coin", Some(FieldValue::Bool(coin_hardened_checkbox.is_active())));
            update_derivation_label(*dp_clone2.borrow(), derivation_label_text)
        }
    ));

    let dp_clone3 = std::rc::Rc::clone(&derivation_path);
    
    address_hardened_checkbox.connect_active_notify(clone!(
        #[weak] derivation_label_text,
        #[weak] address_hardened_checkbox,
        move |_| {
            dp_clone3.borrow_mut().update_field("hardened_address", Some(FieldValue::Bool(address_hardened_checkbox.is_active())));
            update_derivation_label(*dp_clone3.borrow(), derivation_label_text)
        }
    ));
        
    let dp_clone4 = std::rc::Rc::clone(&derivation_path);
    
    purpose_dropdown.connect_selected_notify(clone!(
        #[weak] derivation_label_text,
        #[weak] purpose_dropdown,
        move |_| {
            let purpose = purpose_dropdown.selected();

            dp_clone4.borrow_mut().update_field("purpose", Some(FieldValue::U32(purpose)));
            update_derivation_label(*dp_clone4.borrow(), derivation_label_text);
        }
    ));

    let dp_clone5 = std::rc::Rc::clone(&derivation_path);

    coin_entry.connect_changed(clone!(
        #[weak] derivation_label_text,
        #[strong] coin_entry,
        move |_| {
            let coin_number = coin_entry.buffer().text();
            let ff = coin_number.as_str();
            let my_int = ff.parse::<u32>();

            if my_int.is_ok() {
                dp_clone5.borrow_mut().update_field("coin", Some(FieldValue::U32(my_int.unwrap())));
                update_derivation_label(*dp_clone5.borrow(), derivation_label_text);
            }
        }
    ));

    let dp_clone6 = std::rc::Rc::clone(&derivation_path);

    address_spinbutton.connect_changed(clone!(
        #[weak] derivation_label_text,
        #[weak] address_spinbutton,
        move |_| {
            let address_number = address_spinbutton.text();
            let ff = address_number.as_str();
            let my_int = ff.parse::<u32>();

            if my_int.is_ok() {
                dp_clone6.borrow_mut().update_field("address", Some(FieldValue::U32(my_int.unwrap())));
                update_derivation_label(*dp_clone6.borrow(), derivation_label_text);
            }
        }
    ));

    // JUMP: Generate Addresses button
    generate_addresses_button.connect_clicked(clone!(
        #[weak] address_store,
        #[weak] derivation_label_text,
        #[weak] master_private_key_text,
        #[strong] state,
        move |_| {
            // IMPLEMENT: Generating info_bar message

            let buffer = master_private_key_text.buffer();
            let start_iter = buffer.start_iter();
            let end_iter = buffer.end_iter();
            let master_private_key_string = buffer.text(&start_iter, &end_iter, true);
            
            if master_private_key_string == "" {
                let lock_state = state.lock().unwrap();
                AppState::show_message(&lock_state, t!("error.address.master").to_string(), gtk::MessageType::Warning);
            } else {
                println!("\n#### Generating addresses button ####");
    
                let wallet_settings = WALLET_SETTINGS.lock().unwrap();
                let master_private_key_bytes = wallet_settings.master_private_key_bytes.clone().unwrap_or_default();
                let master_chain_code_bytes = wallet_settings.master_chain_code_bytes.clone().unwrap_or_default();
                let key_derivation = wallet_settings.key_derivation.clone().unwrap_or_default();
                let hash = wallet_settings.hash.clone().unwrap_or_default();
                let wallet_import_format = wallet_settings.wallet_import_format.clone().unwrap_or_default();
                let public_key_hash = wallet_settings.public_key_hash.clone().unwrap_or_default();
                let coin_index = wallet_settings.coin_index.clone().unwrap_or_default();
                let coin_name = wallet_settings.coin_name.clone().unwrap_or_default();
      
                // IMPLEMENT: Save last address number, and next time generate from there afterwards
                let DP = derivation_label_text.text();
                let path = DP.to_string();
            
                let address_count = address_options_spinbutton.text();
                let address_count_str = address_count.as_str();
                let address_count_int = address_count_str.parse::<u32>().unwrap_or(WALLET_DEFAULT_ADDRESS_COUNT);
                
                let hardened = address_options_hardened_address_checkbox.is_active();
            
                let secp = secp256k1::Secp256k1::new();
            
                let trimmed_public_key_hash = if public_key_hash.starts_with("0x") {
                    &public_key_hash[2..]
                } else {
                    &public_key_hash
                };
            
                let public_key_hash_vec = match hex::decode(trimmed_public_key_hash) {
                    Ok(vec) => vec,
                    Err(e) => {
                        println!("Failed to convert: {}", e);
                        return;
                    },
                };
    
                let mut addresses = Vec::new();
            
                for i in 0..address_count_int {
                    let full_path = if hardened {
                        format!("{}/{}'", path, i)
                    } else {
                        format!("{}/{}", path, i)
                    };
            
                    let derived_child_keys = match key_derivation.as_str() {
                        "secp256k1" => keys::derive_from_path_secp256k1(&master_private_key_bytes, &master_chain_code_bytes, &full_path),
                        "ed25519" => dev::derive_from_path_ed25519(&master_private_key_bytes, &master_chain_code_bytes, &full_path),
                        "N/A" | _ => {
                            println!("Unsupported key derivation method: {:?}", key_derivation);
                            return
                        }
                    }.expect("Failed to derive key from path");
    
                    let public_key = match key_derivation.as_str() {
                        "secp256k1" => {
                            let secp_pub_key = secp256k1::PublicKey::from_secret_key(
                                &secp,
                                &secp256k1::SecretKey::from_slice(&derived_child_keys.0).expect("Invalid secret key")
                            );
                            keys::CryptoPublicKey::Secp256k1(secp_pub_key)
                        },
                        "ed25519" => {
                            let secret_key = ed25519_dalek::SigningKey::from_bytes(&derived_child_keys.0);
                            let pub_key_bytes = ed25519_dalek::VerifyingKey::from(&secret_key);
                            keys::CryptoPublicKey::Ed25519(pub_key_bytes)
                        },
                        "N/A" | _ => {
                            println!("Unsupported key derivation method: {:?}", key_derivation);
                            return;
                        }
                    };
    
                    let public_key_encoded = match hash.as_str() {
                        "sha256" | "sha256+ripemd160" => match &public_key {
                            keys::CryptoPublicKey::Secp256k1(public_key) => hex::encode(public_key.serialize()),
                            keys::CryptoPublicKey::Ed25519(public_key) => hex::encode(public_key.to_bytes()),
                        },
                        "keccak256" => match &public_key {
                            keys::CryptoPublicKey::Secp256k1(public_key) => format!("0x{}", hex::encode(public_key.serialize())),
                            keys::CryptoPublicKey::Ed25519(public_key) => format!("0x{}", hex::encode(public_key.to_bytes())),
                        },
                        "N/A" | _ => {
                            println!("Unsupported hash method: {:?}", hash);
                            return;
                        }
                    };
                    
                    let address = match hash.as_str() {
                        "sha256" => keys::generate_address_sha256(&public_key, &public_key_hash_vec),
                        "keccak256" => keys::generate_address_keccak256(&public_key, &public_key_hash_vec),
                        "sha256+ripemd160" => match keys::generate_sha256_ripemd160_address(
                            coin_index, 
                            &public_key, 
                            &public_key_hash_vec
                        ) {
                            Ok(addr) => addr,
                            Err(e) => {
                                println!("Error generating address: {}", e);
                                return;
                            }
                        },
                        "ed25519" => dev::generate_ed25519_address(&public_key),
                        "N/A" | _ => {
                            println!("Unsupported hash method: {:?}", hash);
                            return;
                        }
                    };
    
                    println!("Crypto address: {:?}", address);
    
                    // IMPROVEMENT: remove hard-coding
                    let compressed = true;
                    
                    let priv_key_wif = keys::create_private_key_for_address(
                        Some(&secp256k1::SecretKey::from_slice(&derived_child_keys.0).expect("Invalid secret key")),
                        Some(compressed),
                        Some(&wallet_import_format),
                        &hash,
                    ).expect("Failed to convert private key to WIF");
                    
                    addresses.push(CryptoAddresses {
                        coin_name: coin_name.clone(),
                        derivation_path: full_path,
                        address: address.clone(),
                        public_key: public_key_encoded.clone(),
                        private_key: priv_key_wif.clone(),
                    });
                }
            
                for address in addresses {
                    let iter = address_store.append();
                    address_store.set(&iter, &[
                        (0, &address.coin_name),
                        (1, &address.derivation_path),
                        (2, &address.address),
                        (3, &address.public_key),
                        (4, &address.private_key),
                    ]);
                }
            }

        }
    ));
    
    address_options_clear_addresses_button.connect_clicked(move |_| {
        address_store.clear();
    });

    // Main sidebar
    let main_window_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let main_sidebar_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let main_infobar_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    
    main_sidebar_box.append(&stack_sidebar);
    main_sidebar_box.append(&stack);
    main_infobar_box.append(&info_bar);
    main_window_box.set_hexpand(true);
    main_infobar_box.set_hexpand(true);
    main_window_box.append(&main_sidebar_box);
    main_window_box.append(&main_infobar_box);
    
    let app_messages = AppMessages::new(info_bar.clone());
    app_messages.queue_message(t!("hello").to_string(), gtk::MessageType::Info);
    
    window.set_child(Some(&main_window_box));
    window.present();
}

fn create_settings_window(
    state: std::sync::Arc<std::sync::Mutex<AppState>>,
) -> gtk::ApplicationWindow { 
    println!("[+] {}", &t!("log.create_settings_window").to_string());

    let settings = AppSettings::load_settings()
        .expect(&t!("error.settings.read").to_string());

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
    let general_settings_frame = gtk::Frame::new(Some(&t!("UI.settings.general").to_string()));
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
    let default_gui_theme_color_label = gtk::Label::new(Some(&t!("UI.settings.general.theme").to_string()));
    let valid_gui_themes_as_strings: Vec<String> = VALID_GUI_THEMES.iter().map(|&x| x.to_string()).collect();
    let valid_gui_themes_as_str_refs: Vec<&str> = valid_gui_themes_as_strings.iter().map(|s| s.as_ref()).collect();
    let gui_theme_dropdown = gtk::DropDown::from_strings(&valid_gui_themes_as_str_refs);
    let default_gui_theme = valid_gui_themes_as_strings
        .iter()
        .position(|s| *s == settings.gui_theme) 
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

    // GUI language
    let default_gui_language_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let default_gui_language_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_gui_language_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_gui_language_label = gtk::Label::new(Some(&t!("UI.settings.general.language").to_string()));
    let valid_gui_languages_as_strings: Vec<String> = APP_LANGUAGE.iter().map(|&x| x.to_string()).collect();
    let valid_gui_languages_as_str_refs: Vec<&str> = valid_gui_languages_as_strings.iter().map(|s| s.as_ref()).collect();
    let default_gui_language_dropdown = gtk::DropDown::from_strings(&valid_gui_languages_as_str_refs);
    let default_gui_language = valid_gui_languages_as_strings
        .iter()
        .position(|s| *s == settings.gui_language) 
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
    let save_window_size_label = gtk::Label::new(Some(&t!("UI.settings.general.save_window").to_string()));
    let save_window_size_checkbox = gtk::CheckButton::new();
    let is_checked = settings.gui_save_size;

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
    let default_search_parameter_label = gtk::Label::new(Some(&t!("UI.settings.general.search").to_string()));
    let valid_search_parameters_as_strings: Vec<String> = VALID_COIN_SEARCH_PARAMETER.iter().map(|&x| x.to_string()).collect();
    let valid_search_parameters_as_str_refs: Vec<&str> = valid_search_parameters_as_strings.iter().map(|s| s.as_ref()).collect();
    let default_search_parameter_dropdown = gtk::DropDown::from_strings(&valid_search_parameters_as_str_refs);
    let default_search_parameter = valid_search_parameters_as_strings
        .iter()
        .position(|s| *s == settings.gui_search) 
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
    let notification_timeout_label = gtk::Label::new(Some(&t!("UI.settings.wallet.notification-timeout").to_string()));
    let notification_timeout = settings.gui_notification_timeout as f64;
    let notification_timeout_adjustment = gtk::Adjustment::new(
        notification_timeout,
        1.0,
        120.0,
        1.0,
        10.0,
        0.0,
    );
    let notification_timeout_spinbutton = gtk::SpinButton::new(Some(&notification_timeout_adjustment), 1.0, 0);

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

    // Default mnemonic passphrase length
    let mnemonic_length_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let mnemonic_length_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let mnemonic_length_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let mnemonic_length_label = gtk::Label::new(Some(&t!("UI.settings.wallet.mnemonic-length").to_string()));
    let mnemonic_length = settings.gui_mnemonic_length as f64;
    let mnemonic_length_adjustment = gtk::Adjustment::new(
        mnemonic_length,
        8.0 * 2.0,
        8.0 * 128.0,
        1.0,
        100.0,
        0.0,
    );
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
    content_general_box.append(&mnemonic_length_box);


    stack.add_titled(
        &general_settings_box,
        Some("sidebar-settings-general"),
        &t!("UI.settings.sidebar.general").to_string()
    );

    // -.-. --- .--. -.-- .-. .. --. .... -
    // JUMP: Settings: Sidebar 2: Wallet settings
    // -.-. --- .--. -.-- .-. .. --. .... -
    let wallet_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let wallet_settings_frame = gtk::Frame::new(Some(&t!("UI.settings.wallet").to_string()));
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
    let qrng_enabled = settings.anu_enabled;
    let valid_entropy_sources: Vec<&str> = if qrng_enabled {
        VALID_ENTROPY_SOURCES.iter().cloned().collect()
    } else {
        VALID_ENTROPY_SOURCES.iter().filter(|&&x| x != "QRNG").cloned().collect()
    };

    let default_entropy_source_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let default_entropy_source_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_entropy_source_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_entropy_source_label = gtk::Label::new(Some(&t!("UI.settings.wallet.entropy.source").to_string()));
    let valid_entropy_source_as_strings: Vec<String> = valid_entropy_sources.iter().map(|&x| x.to_string()).collect();
    let valid_entropy_source_as_str_refs: Vec<&str> = valid_entropy_source_as_strings.iter().map(|s| s.as_ref()).collect();
    let entropy_source_dropdown = gtk::DropDown::from_strings(&valid_entropy_source_as_str_refs);
    let default_entropy_source = valid_entropy_source_as_strings
        .iter()
        .position(|s| *s == settings.wallet_entropy_source) 
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
    let default_entropy_length_label = gtk::Label::new(Some(&t!("UI.settings.wallet.entropy.length").to_string()));
    let valid_entropy_lengths_as_strings: Vec<String> = VALID_ENTROPY_LENGTHS.iter().map(|&x| x.to_string()).collect();
    let valid_entropy_lengths_as_str_refs: Vec<&str> = valid_entropy_lengths_as_strings.iter().map(|s| s.as_ref()).collect();
    let entropy_length_dropdown = gtk::DropDown::from_strings(&valid_entropy_lengths_as_str_refs);
    let default_entropy_length = valid_entropy_lengths_as_strings
        .iter()
        .position(|x| x.parse::<u32>().unwrap() == settings.wallet_entropy_length)
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

    // Default BIP
    let default_bip_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let default_bip_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_bip_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_bip_label = gtk::Label::new(Some(&t!("UI.settings.wallet.bip").to_string()));
    let valid_bips_as_strings: Vec<String> = VALID_BIP_DERIVATIONS.iter().map(|&x| x.to_string()).collect();
    let valid_bips_as_str_refs: Vec<&str> = valid_bips_as_strings.iter().map(|s| s.as_ref()).collect();
    let bip_dropdown = gtk::DropDown::from_strings(&valid_bips_as_str_refs);
    let default_bip = valid_bips_as_strings
        .iter()
        .position(|x| x.parse::<u32>().unwrap() == settings.wallet_bip)
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
    let default_address_count_label = gtk::Label::new(Some(&t!("UI.settings.wallet.address-count").to_string()));
    let default_address_count = settings.wallet_address_count as f64;
    let address_count_adjustment = gtk::Adjustment::new(
        default_address_count,
        1.0,
        2147483647.0,
        1.0,
        10.0,
        0.0,
    );
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
    let hardened_addresses_label = gtk::Label::new(Some(&t!("UI.settings.wallet.hardened").to_string()));
    let hardened_addresses_checkbox = gtk::CheckButton::new();
    let is_checked = settings.wallet_hardened_address;

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
        &t!("UI.settings.sidebar.wallet").to_string()
    );


    // -.-. --- .--. -.-- .-. .. --. .... -
    // JUMP: Settings: Sidebar 3: ANU settings
    // -.-. --- .--. -.-- .-. .. --. .... -
    let anu_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let anu_settings_frame = gtk::Frame::new(Some(&t!("UI.settings.anu").to_string()));
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
    let use_anu_api_label = gtk::Label::new(Some(&t!("UI.settings.anu.use_anu").to_string()));
    let use_anu_api_checkbox = gtk::CheckButton::new();
    let is_checked = settings.anu_enabled;

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
    let log_anu_api_label = gtk::Label::new(Some(&t!("UI.settings.anu.log").to_string()));
    let log_anu_api_checkbox = gtk::CheckButton::new();

    log_anu_api_checkbox.set_active(settings.anu_log);
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
    let default_api_data_format_label = gtk::Label::new(Some(&t!("UI.settings.anu.data.type").to_string()));
    let valid_api_data_formats_as_strings: Vec<String> = VALID_ANU_API_DATA_FORMAT.iter().map(|&x| x.to_string()).collect();
    let valid_api_data_formats_as_str_refs: Vec<&str> = valid_api_data_formats_as_strings.iter().map(|s| s.as_ref()).collect();
    let anu_data_format_dropdown = gtk::DropDown::from_strings(&valid_api_data_formats_as_str_refs);
    let default_api_data_format = valid_api_data_formats_as_strings
        .iter()
        .position(|x| x.parse::<String>().unwrap() == settings.anu_data_format)
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
    let default_anu_array_length_label = gtk::Label::new(Some(&t!("UI.settings.anu.data.array").to_string()));
    let mut default_array_length = settings.anu_array_length;
    default_array_length = std::cmp::max(ANU_MINIMUM_ARRAY_LENGTH, default_array_length);
    default_array_length = std::cmp::min(ANU_MAXIMUM_ARRAY_LENGTH, default_array_length);

    let array_length_adjustment = gtk::Adjustment::new(
        default_array_length as f64,       
        ANU_MINIMUM_ARRAY_LENGTH as f64,   
        ANU_MAXIMUM_ARRAY_LENGTH as f64,   
        1.0,
        10.0,
        0.0,
    );
    let default_anu_array_length_spinbutton = gtk::SpinButton::new(Some(&array_length_adjustment), 1.0, 0);

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
    let default_anu_hex_length_label = gtk::Label::new(Some(&t!("UI.settings.anu.data.hex").to_string()));
    
    let mut default_hex_size = settings.anu_hex_block_size;
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
    let default_anu_hex_length_spinbutton = gtk::SpinButton::new(Some(&hex_block_size_adjustment), 1.0, 0);

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
    } ;

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
    }
    
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
        #[weak] default_anu_hex_length_box,
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
        &t!("UI.settings.sidebar.anu").to_string()
    );


    // -.-. --- .--. -.-- .-. .. --. .... -
    // JUMP: Settings: Sidebar 4: Proxy settings
    // -.-. --- .--. -.-- .-. .. --. .... -
    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_max_content_height(400);
    
    let proxy_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let proxy_settings_frame = gtk::Frame::new(Some(&t!("UI.settings.proxy").to_string()));
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
    let use_proxy_settings_label = gtk::Label::new(Some(&t!("UI.settings.proxy.use").to_string()));
    let valid_proxy_settings_as_strings: Vec<String> = VALID_PROXY_STATUS.iter().map(|&x| x.to_string()).collect();
    let valid_proxy_settings_as_str_refs: Vec<&str> = valid_proxy_settings_as_strings.iter().map(|s| s.as_ref()).collect();
    let use_proxy_settings_dropdown = gtk::DropDown::from_strings(&valid_proxy_settings_as_str_refs);
    let default_proxy_settings_format = valid_proxy_settings_as_strings
        .iter()
        .position(|x| x.parse::<String>().unwrap() == settings.proxy_status)
        .unwrap_or(1);  // Default proxy: auto

    use_proxy_settings_dropdown.set_selected(default_proxy_settings_format.try_into().unwrap());
    use_proxy_settings_dropdown.set_size_request(200, 10);
    use_proxy_settings_label_box.set_hexpand(true);
    use_proxy_settings_item_box.set_hexpand(true);
    use_proxy_settings_item_box.set_margin_end(20);
    use_proxy_settings_item_box.set_halign(gtk::Align::End);

    use_proxy_settings_label_box.append(&use_proxy_settings_label);
    use_proxy_settings_item_box.append(&use_proxy_settings_dropdown);
    use_proxy_settings_box.append(&use_proxy_settings_label_box);
    use_proxy_settings_box.append(&use_proxy_settings_item_box);
    content_proxy_box.append(&use_proxy_settings_box);

    // Proxy manual settings
    let proxy_manual_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    
    // IMPLEMENT non-case sensitive input Manual/manual

    if settings.proxy_status == "Manual" {
        proxy_manual_settings_box.set_visible(true);
    } else {
        proxy_manual_settings_box.set_visible(false);
    }

    // Proxy server address
    let proxy_server_address_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
    let proxy_server_address_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let proxy_server_address_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let proxy_server_address_label = gtk::Label::new(Some(&t!("UI.settings.proxy.address").to_string()));
    let proxy_server_address_entry = gtk::Entry::new();

    proxy_server_address_entry.set_size_request(200, 10);
    proxy_server_address_label_box.set_hexpand(true);
    proxy_server_address_item_box.set_hexpand(true);
    proxy_server_address_item_box.set_margin_end(20);
    proxy_server_address_item_box.set_halign(gtk::Align::End);
    proxy_server_address_entry.set_text(&settings.proxy_server_address);

    proxy_server_address_label_box.append(&proxy_server_address_label);
    proxy_server_address_item_box.append(&proxy_server_address_entry);
    proxy_server_address_box.append(&proxy_server_address_label_box);
    proxy_server_address_box.append(&proxy_server_address_item_box);
    proxy_manual_settings_box.append(&proxy_server_address_box);


    // Proxy server port
    let proxy_server_port_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
    let proxy_server_port_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let proxy_server_port_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let proxy_server_port_label = gtk::Label::new(Some(&t!("UI.settings.proxy.port").to_string()));
    let proxy_server_port_entry = gtk::Entry::new();

    proxy_server_port_entry.set_size_request(200, 10);
    proxy_server_port_label_box.set_hexpand(true);
    proxy_server_port_item_box.set_hexpand(true);
    proxy_server_port_item_box.set_margin_end(20);
    proxy_server_port_item_box.set_halign(gtk::Align::End);

    if settings.proxy_server_port == 0 {
        proxy_server_port_entry.set_text(&DEFAULT_PROXY_PORT.to_string());
    } else {
        proxy_server_port_entry.set_text(&settings.proxy_server_port.to_string());
    }

    proxy_server_port_label_box.append(&proxy_server_port_label);
    proxy_server_port_item_box.append(&proxy_server_port_entry);
    proxy_server_port_box.append(&proxy_server_port_label_box);
    proxy_server_port_box.append(&proxy_server_port_item_box);
    proxy_manual_settings_box.append(&proxy_server_port_box);
    
    // Use proxy credentials
    let use_proxy_credentials_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
    let use_proxy_credentials_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let use_proxy_credentials_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let use_proxy_credentials_label = gtk::Label::new(Some(&t!("UI.settings.proxy.creds").to_string()));
    let use_proxy_credentials_checkbox = gtk::CheckButton::new();
    let is_checked = settings.proxy_login_credentials;
    
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
    
    if settings.proxy_login_credentials == true {
        use_proxy_credentials_content_box.set_visible(true);
    } else {
        use_proxy_credentials_content_box.set_visible(false);
    }

    // Proxy username
    let proxy_username_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
    let proxy_username_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let proxy_username_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let proxy_username_label = gtk::Label::new(Some(&t!("UI.settings.proxy.username").to_string()));
    let proxy_username_entry = gtk::Entry::new();

    proxy_username_entry.set_size_request(200, 10);
    proxy_username_label_box.set_hexpand(true);
    proxy_username_item_box.set_hexpand(true);
    proxy_username_item_box.set_margin_end(20);
    proxy_username_item_box.set_halign(gtk::Align::End);
    proxy_username_entry.set_text(&settings.proxy_login_username);

    proxy_username_label_box.append(&proxy_username_label);
    proxy_username_item_box.append(&proxy_username_entry);
    proxy_username_box.append(&proxy_username_label_box);
    proxy_username_box.append(&proxy_username_item_box);
    use_proxy_credentials_content_box.append(&proxy_username_box);

    // Proxy password
    let proxy_password_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
    let proxy_password_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let proxy_password_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let proxy_password_label = gtk::Label::new(Some(&t!("UI.settings.proxy.password").to_string()));
    let proxy_password_entry = gtk::PasswordEntry::new();

    proxy_password_entry.set_size_request(200, 10);
    proxy_password_label_box.set_hexpand(true);
    proxy_password_item_box.set_hexpand(true);
    proxy_password_item_box.set_margin_end(20);
    proxy_password_item_box.set_halign(gtk::Align::End);

    proxy_password_entry.set_show_peek_icon(true);
    proxy_password_entry.set_text(&settings.proxy_login_password);

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
    let use_proxy_pac_label = gtk::Label::new(Some(&t!("UI.settings.proxy.pac").to_string()));
    let use_proxy_pac_checkbox = gtk::CheckButton::new();
    let is_checked = settings.proxy_use_pac;
    
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
    
    if settings.proxy_use_pac == true {
        use_proxy_pac_content_box.set_visible(true);
    } else {
        use_proxy_pac_content_box.set_visible(false);
    }

    // Proxy PAC path
    let proxy_pac_path_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
    let proxy_pac_path_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let proxy_pac_path_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let proxy_pac_path_label = gtk::Label::new(Some(&t!("UI.settings.proxy.pac.path").to_string()));
    let proxy_pac_path_entry = gtk::Entry::new();

    proxy_pac_path_entry.set_size_request(200, 10);
    proxy_pac_path_label_box.set_hexpand(true);
    proxy_pac_path_item_box.set_hexpand(true);
    proxy_pac_path_item_box.set_margin_end(20);
    proxy_pac_path_item_box.set_halign(gtk::Align::End);
    proxy_pac_path_entry.set_text(&settings.proxy_script_address);

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
    let use_proxy_ssl_label = gtk::Label::new(Some(&t!("UI.settings.proxy.ssl").to_string()));
    let use_proxy_ssl_checkbox = gtk::CheckButton::new();
    let is_checked = settings.proxy_use_ssl;
    
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
    
    if settings.proxy_use_ssl == true {
        use_proxy_ssl_certificate_content_box.set_visible(true);
    } else {
        use_proxy_ssl_certificate_content_box.set_visible(false);
    }

    // Proxy SSL certificate path
    let proxy_ssl_certificate_path_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
    let proxy_ssl_certificate_path_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let proxy_ssl_certificate_path_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let proxy_ssl_certificate_path_label = gtk::Label::new(Some(&t!("UI.settings.proxy.ssl.path").to_string()));
    let proxy_ssl_certificate_path_entry = gtk::Entry::new();

    proxy_ssl_certificate_path_entry.set_size_request(200, 10);
    proxy_ssl_certificate_path_label_box.set_hexpand(true);
    proxy_ssl_certificate_path_item_box.set_hexpand(true);
    proxy_ssl_certificate_path_item_box.set_margin_end(20);
    proxy_ssl_certificate_path_item_box.set_halign(gtk::Align::End);
    proxy_ssl_certificate_path_entry.set_text(&settings.proxy_ssl_certificate);

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
        &t!("UI.settings.sidebar.proxy").to_string()
    );

    // Actions
    use_proxy_settings_dropdown.connect_selected_notify(clone!(
        #[weak] proxy_manual_settings_box,
        move |dd| {
            let value = dd.selected() as usize;
            let selected_proxy_settings_value = VALID_PROXY_STATUS.get(value);
            let settings = selected_proxy_settings_value.unwrap();
            
            if *settings == "Manual" {
                proxy_manual_settings_box.set_visible(true);
            } else {
                proxy_manual_settings_box.set_visible(false);
            }
        }
    ));

    use_proxy_credentials_checkbox.connect_active_notify(clone!(
        #[weak] use_proxy_credentials_content_box,
        move |cb| {
            use_proxy_credentials_content_box.set_visible(cb.is_active());
        }
    ));

    use_proxy_pac_checkbox.connect_active_notify(clone!(
        #[weak] use_proxy_pac_content_box,
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

    let save_button = gtk::Button::with_label(&t!("UI.element.button.save").to_string());
    let cancel_button = gtk::Button::with_label(&t!("UI.element.button.cancel").to_string());
    let default_button = gtk::Button::with_label(&t!("UI.element.button.default").to_string());


    // JUMP: Save settings button
    save_button.connect_clicked(clone!(
        #[weak] settings_window,
        #[strong] state,
        move |_| {
            let mut settings = APPLICATION_SETTINGS.lock().unwrap();
    
            let updates = vec![
                ("wallet_entropy_source", toml_edit::value(VALID_ENTROPY_SOURCES[entropy_source_dropdown.selected() as usize])),
                ("wallet_entropy_length", toml_edit::value(VALID_ENTROPY_LENGTHS[entropy_length_dropdown.selected() as usize] as i64)),
                ("wallet_bip", toml_edit::value(VALID_BIP_DERIVATIONS[bip_dropdown.selected() as usize] as i64)),
                ("wallet_address_count", toml_edit::value(address_count_spinbutton.value_as_int() as i64)),
                ("wallet_hardened_address", toml_edit::value(hardened_addresses_checkbox.is_active())),
                ("gui_save_size", toml_edit::value(save_window_size_checkbox.is_active())),
                ("gui_theme", toml_edit::value(VALID_GUI_THEMES[gui_theme_dropdown.selected() as usize])),
                ("gui_language", toml_edit::value(APP_LANGUAGE[default_gui_language_dropdown.selected() as usize])),
                ("gui_search", toml_edit::value(VALID_COIN_SEARCH_PARAMETER[default_search_parameter_dropdown.selected() as usize])),
                ("gui_notification_timeout", toml_edit::value(notification_timeout_spinbutton.value_as_int() as i64)),
                ("gui_mnemonic_length", toml_edit::value(mnemonic_length_spinbutton.value_as_int() as i64)),
                ("anu_enabled", toml_edit::value(use_anu_api_checkbox.is_active())),
                ("anu_log", toml_edit::value(log_anu_api_checkbox.is_active())),
                ("anu_data_format", toml_edit::value(VALID_ANU_API_DATA_FORMAT[anu_data_format_dropdown.selected() as usize])),
                ("anu_array_length", toml_edit::value(default_anu_array_length_spinbutton.value_as_int() as i64)),
                ("anu_hex_block_size", toml_edit::value(default_anu_hex_length_spinbutton.value_as_int() as i64)),
                ("proxy_status", toml_edit::value(VALID_PROXY_STATUS[use_proxy_settings_dropdown.selected() as usize])),
                ("proxy_server_address", toml_edit::value(proxy_server_address_entry.text().to_string())),
                ("proxy_server_port", toml_edit::value(proxy_server_port_entry.text().parse::<u32>().unwrap_or_default() as i64)),
                ("proxy_use_pac", toml_edit::value(use_proxy_ssl_checkbox.is_active())),
                ("proxy_script_address", toml_edit::value(proxy_pac_path_entry.text().to_string())),
                ("proxy_login_credentials", toml_edit::value(use_proxy_credentials_checkbox.is_active())),
                ("proxy_login_username", toml_edit::value(proxy_username_entry.text().to_string())),
                ("proxy_login_password", toml_edit::value(proxy_password_entry.text().to_string())),
                ("proxy_use_ssl", toml_edit::value(use_proxy_ssl_checkbox.is_active())),
                ("proxy_ssl_certificate", toml_edit::value(proxy_ssl_certificate_path_entry.text().to_string())),
            ];
    
            for (key, value) in updates {
                settings.update_value(key, value, Some(state.clone()));
            }
    
            AppSettings::save_settings(&*settings);
                       
            let mut state_lock = state.lock().unwrap();
            let selected_theme = VALID_GUI_THEMES[gui_theme_dropdown.selected() as usize].to_string();
            state_lock.theme = Some(selected_theme);

            if let Some(app_messages) = &state_lock.app_messages {
                let app_messages = app_messages.clone();
                let app_messages = app_messages.lock().unwrap();
                app_messages.queue_message(t!("UI.messages.dialog.settings-saved").to_string(), gtk::MessageType::Info);
            } else {
                eprintln!("Error: AppMessages not initialized.");
            }

            settings_window.close();
        }
    ));
    
    cancel_button.connect_clicked(clone!(
        #[weak] settings_window,
        move |_| {
            settings_window.close()
        }
    ));
    
    default_button.connect_clicked(clone!(
        #[weak] settings_window,
        move |_| {

            let dialog = gtk::MessageDialog::builder()
                .modal(true)
                .text("Do you really want to reset config?")
                .build();

            dialog.add_button("Yes", gtk::ResponseType::Yes);
            dialog.add_button("No", gtk::ResponseType::No);

            dialog.connect_response(clone!(
                #[weak] settings_window,
                #[weak] state,
                move |dialog, response| {
                    match response {
                        gtk::ResponseType::Yes => {
                            let new_state = std::sync::Arc::new(std::sync::Mutex::new(AppState::new()));
                            let mut lock_new_state = new_state.lock().unwrap();
                            let lock_state = state.lock().unwrap();
                            
                            match reset_user_settings().unwrap().as_str() {
                                "OK" => {
                                    dialog.close();
                                    settings_window.close();
                                    
                                    AppSettings::load_settings().expect(&t!("error.settings.read").to_string());

                                    AppState::apply_theme(&mut lock_new_state);
                                    AppState::show_message(&lock_state, t!("UI.messages.dialog.settings-reset").to_string(), gtk::MessageType::Info);
                                },
                                _ => {
                                    AppState::show_message(&lock_state, t!("error.settings.reset").to_string(), gtk::MessageType::Error);
                                },
                            }
                        },
                        _ => {
                            dialog.close();
                        },
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
    println!("[+] {}", &t!("log.reset-config").to_string());

    {
        let local_settings = os::LOCAL_SETTINGS.lock().unwrap();
        let local_config_file = local_settings.local_config_file.clone().unwrap();
        
        println!("\t Local config file: {:?}", local_config_file);
        
        match std::fs::remove_file(local_config_file) {
            Ok(_) => {
                println!("\t Local config file deleted");
            },
            Err(err) => {
                // TODO: AppMessages
                eprintln!("\t Local config file NOT deleted \n Error: {}", err);
            },
        };
    }
    
    match os::create_local_files() {
        Ok(_) => {
            println!("\t New config file created");
            Ok("OK".to_string())
        },
        Err(err) => {
            // TODO: AppMessages
            eprintln!("\t New config file NOT created \n {}", err);
            Err(err.to_string())
        },
    }
}

fn create_about_window() {
    println!("[+] {}", &t!("log.create_about_window").to_string());

    let pixy = qr2m_lib::get_image_from_resources("logo/logo.svg");
    let logo_picture = gtk::Picture::for_pixbuf(&pixy).paintable().unwrap();

    let my_license = std::path::Path::new("licenses").join("LICENSE.txt");
    let app_license = qr2m_lib::get_text_from_resources(&my_license.to_str().unwrap());
    
    let their_license = std::path::Path::new("licenses").join("LICENSE-LGPL-2.1.txt");
    let lgpl_license = qr2m_lib::get_text_from_resources(&their_license.to_str().unwrap());
    
    let licenses = format!("{}\n\n---\n\n{}", app_license, lgpl_license);

    let they_forced_me = [
        "This application uses GTK4 for its GUI.",
        "GTK4 is licensed under the GNU Lesser General Public License (LGPL) version 2.1 or later.",
        "For more details on the LGPL-2.1 license and your rights under this license, please refer to the License tab."
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
        .copyright("Copyright [2023-2024] Control Owl")
        .license(licenses)
        .wrap_license(true)
        .comments(&t!("UI.about.description").to_string())
        .logo(&logo_picture)
        .comments(comments)
        .build();

    help_window.show();

}

pub fn create_info_message(
    revealer: &gtk::Revealer,
    message: &str,
    message_type: gtk::MessageType,
) -> gtk::Box {
    let message_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let message_label = gtk::Label::new(Some(message));
    message_label.set_hexpand(true);

    match message_type {
        gtk::MessageType::Error => message_box.set_css_classes(&["error-message"]),
        gtk::MessageType::Warning => message_box.set_css_classes(&["warning-message"]),
        _ => message_box.set_css_classes(&["info-message"]),
    }

    let close_button = gtk::Button::with_label("Close");
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

    message_box
}

fn open_wallet_from_file() -> (String, Option<String>) {
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
        move |open_dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file) = open_dialog.file() {
                    if let Some(path) = file.path() {
                        let file_path = path.to_string_lossy().to_string();
                        println!("\t Wallet file chosen: {:?}", file_path);

                        let result = process_wallet_file_from_path(&file_path);

                        match result {
                            Ok((version, entropy, password)) => {
                                let passphrase = match password {
                                    Some(pass) => pass,
                                    None => String::new(),
                                };

                                let file_entropy_string = format!("{}\n{}\n{}", version, entropy, passphrase);
                                if let Err(err) = tx.send(file_entropy_string) {
                                    println!("Error sending data: {}", err);
                                } else {
                                    open_loop.quit();
                                }
                            },
                            Err(err) => {
                                println!("Error processing wallet file: {}", err);
                                let error_message = format!("Error: {}", err);
                                if let Err(err) = tx.send(error_message) {
                                    println!("Error sending data: {}", err);
                                }
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

            (entropy.unwrap_or_else(|| String::new()), passphrase)
        },
        Err(_) => {
            println!("Error: Failed to read wallet file.");
            (String::new(), None)
        }
    }
}

fn save_wallet_to_file() {
    // TODO: Check if wallet is created before proceeding
    let save_context = glib::MainContext::default();
    let save_loop = glib::MainLoop::new(Some(&save_context), false);
    
    let wallet_settings = WALLET_SETTINGS.lock().unwrap();
    let entropy_string = wallet_settings.entropy_string.clone().unwrap_or_default();
    let mnemonic_passphrase = wallet_settings.mnemonic_passphrase.clone().unwrap_or_default();

    let save_window = gtk::Window::new();
    let save_dialog = gtk::FileChooserNative::new(
        Some(t!("UI.dialog.save").to_string().as_str()),
        Some(&save_window),
        gtk::FileChooserAction::Save,
        Some(&t!("UI.element.button.save")),
        Some(&t!("UI.element.button.cancel"))
    );

    let filter = gtk::FileFilter::new();
    filter.add_pattern(&format!("*.{}",WALLET_DEFAULT_EXTENSION));
    filter.set_name(Some(&format!("Wallet file (*.{})",WALLET_DEFAULT_EXTENSION)));
    save_dialog.add_filter(&filter);
    
    let all_files_filter = gtk::FileFilter::new();
    all_files_filter.add_pattern("*");
    all_files_filter.set_name(Some("All files"));
    save_dialog.add_filter(&all_files_filter);

    save_dialog.connect_response(clone!(
        #[strong] save_loop,
        move |save_dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file) = save_dialog.file() {
                    if let Some(path) = file.path() {
                        // TODO: Get data from WalletSettings struct
                        let wallet_data = format!("version = {}\n{}\n{}", WALLET_CURRENT_VERSION, entropy_string, mnemonic_passphrase);

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

fn get_window_theme_icons() -> [gtk::Image; 5] {
    println!("[+] {}", &t!("log.get_window_theme_icons").to_string());

    let settings = gtk::Settings::default().unwrap();
    let theme_subdir = if settings.is_gtk_application_prefer_dark_theme() {
        "dark"
    } else {
        "light"
    };

    let theme_base_path = std::path::Path::new("theme").join("basic").join(theme_subdir);
    println!("\t Theme path: {:?}", theme_base_path);

    let icon_files = [
        "new-wallet.png",
        "open-wallet.png",
        "save-wallet.png",
        "about.png",
        "settings.png",
    ];

    let mut images = Vec::new();

    for icon_file in icon_files.iter() {
        let icon_path = theme_base_path.join(icon_file);

        let valid_icon = qr2m_lib::get_image_from_resources(icon_path.to_str().unwrap());

        images.push(valid_icon);
    }

    let final_images: [gtk::Image; 5] = core::array::from_fn(|i| gtk::Image::from_pixbuf(Some(&images[i])));

    final_images
}

fn update_derivation_label(DP: DerivationPath, label: gtk::Label, ) {
    println!("[+] {}", &t!("log.update_derivation_label").to_string());
    println!("\t Derivation Path: {:?}", DP);

    let mut path = String::new();
    path.push_str("m");

    if DP.bip.unwrap() == 32  {
        path.push_str(&format!("/{}", DP.coin.unwrap_or_default()));
        if DP.hardened_coin.unwrap_or_default() {
            path.push_str(&format!("'"));
        }

        path.push_str(&format!("/{}", DP.address.unwrap_or_default()));
        if DP.hardened_address.unwrap_or_default() {
            path.push_str(&format!("'"));
        }
    } else {
        path.push_str(&format!("/{}", DP.bip.unwrap_or_default()));
        if DP.hardened_bip.unwrap_or_default() {
            path.push_str(&format!("'"));
        }

        path.push_str(&format!("/{}", DP.coin.unwrap_or_default()));
        if DP.hardened_coin.unwrap_or_default() {
            path.push_str(&format!("'"));
        }

        path.push_str(&format!("/{}", DP.address.unwrap_or_default()));
        if DP.hardened_address.unwrap_or_default() {
            path.push_str(&format!("'"));
        }

        path.push_str(&format!("/{}", DP.purpose.unwrap_or_default()));
    }
    
    println!("\t Derivation path: {:?}", &path);

    label.set_text(&path);
}

fn process_wallet_file_from_path(file_path: &str) -> Result<(u8, String, Option<String>), String> {
    let file = File::open(file_path).map_err(|_| "Error: Could not open wallet file".to_string())?;
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
        },
        _ => Err(format!("Error: Unsupported wallet version '{}'", version)),
    }
}

fn parse_wallet_version(line: &str) -> Result<u8, String> {
    if line.starts_with("version = ") {
        match line["version = ".len()..].parse::<u8>() {
            Ok(version) => Ok(version),
            Err(_) => Err("Error: Invalid version format, expected an integer".to_string()),
        }
    } else {
        Err("Error: Version line is malformed, expected 'version = X' format".to_string())
    }
}


// TESTING -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


// I have a new idea how to still generate secure entropy even if source is corrupted/monitored
// It will take me time to implement it since I suck in Rust
// But idea is already tested in my head :D

fn get_qrng() -> String {
    use rand::{Rng, thread_rng};

    let mut rng = thread_rng();
    let length = rng.gen_range(2..=1024);

    let hex_chars: String = (0..length)
        .map(|_| {
            let random_char = rng.gen_range(0..16);
            match random_char {
                0..=9 => (b'0' + random_char as u8) as char,
                10..=15 => (b'A' + (random_char - 10) as u8) as char, // A-F
                _ => unreachable!(),
            }
        })
        .collect();

    println!("Generated String: {}", hex_chars);
    hex_chars
}

fn anu_window() {
    let app = gtk::Application::builder()
        .application_id("wtf.r_o0_t.qr2m.qrng_checker")
        .build();

    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("QRNG Checker")
            .default_width(600)
            .default_height(600)
            .build();

        let main_grid_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let grid = gtk::Grid::builder()
            .column_spacing(5)
            .row_spacing(5)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        main_grid_box.append(&grid);

        let main_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let ok_button = gtk::Button::with_label("OK");
        let cancel_button = gtk::Button::with_label("Cancel");
        let new_button = gtk::Button::with_label("New QRNG");

        main_button_box.append(&ok_button);
        main_button_box.append(&new_button);
        main_button_box.append(&cancel_button);

        main_button_box.set_margin_bottom(10);
        main_button_box.set_margin_top(10);
        main_button_box.set_margin_start(10);
        main_button_box.set_margin_end(10);

        main_grid_box.append(&main_button_box);

        let total_boxes = 1024;
        let mut boxes = Vec::new();

        for i in 0..total_boxes {
            let label = gtk::Label::builder()
                .label("  ")
                .width_request(20)
                .height_request(20)
                .build();

            label.set_css_classes(&["empty-box"]);
            grid.attach(&label, (i % 32) as i32, (i / 32) as i32, 1, 1);
            boxes.push(label);
        }

        let recolor_boxes = std::rc::Rc::new(std::cell::RefCell::new({
            let boxes = boxes.clone();
            move || {
                let qrng_string = get_qrng();
                let qrng_length = qrng_string.len();
                println!("New QRNG Length: {}", qrng_length);

                for (i, label) in boxes.iter().enumerate() {
                    if i < qrng_length {
                        label.set_css_classes(&["green-box"]);
                    } else {
                        label.set_css_classes(&["empty-box"]);
                    }
                }

                if qrng_length == total_boxes {
                    println!("Done");
                } else {
                    println!("Not enough");
                }
            }
        }));

        {
            let recolor_boxes = recolor_boxes.clone();
            new_button.connect_clicked(move |_| {
                recolor_boxes.borrow_mut()();
            });
        }

        window.set_child(Some(&main_grid_box));
        window.show();
    });

    app.run();
}
