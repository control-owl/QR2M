#![allow(non_snake_case)]
#![allow(unused_imports)]
// #![allow(unused_variables)]


// REQUIREMENTS
// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

// Crates
use std::{
    fs::{self, File, OpenOptions}, 
    io::{self, Read, Seek, prelude::*, BufReader, BufRead},
    net::{TcpStream,ToSocketAddrs},
    path::Path,time::{Duration, SystemTime}
};
use hex;
use rand::Rng;
use sha2::{Digest, Sha256, Sha512};
use bip39;
use csv::ReaderBuilder;
use gtk4 as gtk;
use libadwaita as adw;
use adw::prelude::*;
use gtk::{gio, glib::clone, prelude::*, BaselinePosition, Stack, StackSidebar};
use toml::Value;
use qr2m_converters::{convert_binary_to_string, convert_string_to_binary};

// Default settings
// TODO: Replace with new settings
const WORDLIST_FILE: &str = "lib/bip39-mnemonic-words-english.txt";
const COINLIST_FILE: &str = "lib/bip44-extended-coin-list.csv";
const VALID_ENTROPY_LENGTHS: [u32; 5] = [128, 160, 192, 224, 256];
const VALID_BIP_DERIVATIONS: [u32; 5] = [32, 44, 49, 84, 86];
const VALID_ENTROPY_SOURCES: &'static [&'static str] = &[
    "RNG", 
    "QRNG",
    "Test",
];
const VALID_WALLET_PURPOSE: &'static [&'static str] = &[
    "Internal", 
    "External", 
];
const APP_DESCRIPTION: Option<&str> = option_env!("CARGO_PKG_DESCRIPTION");
const APP_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
const APP_AUTHOR: Option<&str> = option_env!("CARGO_PKG_AUTHORS");



// BASIC
// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

fn print_program_info() {
    println!(" ██████╗ ██████╗ ██████╗ ███╗   ███╗");
    println!("██╔═══██╗██╔══██╗╚════██╗████╗ ████║");
    println!("██║   ██║██████╔╝ █████╔╝██╔████╔██║");
    println!("██║▄▄ ██║██╔══██╗██╔═══╝ ██║╚██╔╝██║");
    println!("╚██████╔╝██║  ██║███████╗██║ ╚═╝ ██║");
    println!(" ╚══▀▀═╝ ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝");
    println!("{} ({})\n{}", &APP_DESCRIPTION.unwrap(), &APP_VERSION.unwrap(), &APP_AUTHOR.unwrap());
    println!("-.-. --- .--. -.-- .-. .. --. .... -\n")
}

fn generate_entropy(source: &str, length: u64, _file_name: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    match source {
        "RNG" => {
            let mut rng = rand::thread_rng();
            let binary_string: String = (0..length)
                .map(|_| rng.gen_range(0..=1))
                .map(|bit| char::from_digit(bit, 10).unwrap())
                .collect();

            Ok(binary_string)
        },
        "QRNG" => {
            // ANU API Options
            // TODO: Get velues from settings
            // let anu_format = "uint8"; // uint8, uint16, hex16
            // let array_length = 1024;       // 1-1024
            // let hex_block_size = 16;   // 1-1024

            // TODO: Re-create ANU
            // let qrng = get_anu_response(
            //     anu_format, 
            //     array_length, 
            //     hex_block_size
            // );
            let qrng = String::from("1000010101111101011101001100000011111101100111000011110010001100111011111111001110111100110011011101011000101100101001001000011101001000001010100010111111100111100101110001100001011110000110001001101110100000111100101111101111110101111111011111100100011111");
            
            Ok(qrng)
        },
        "Test" => {
            // let file = File::open(file_name.unwrap())?;
            let file = File::open("entropy/test.qrn").expect("Can not open test entropy file");
            let mut reader = io::BufReader::new(file);
            
            let file_length = reader.seek(io::SeekFrom::End(0))?;
            
            if file_length < length {
                eprintln!("Entropy file too small for requested entropy length: {}", length);
                return Err("Insufficient entropy in file".into());
            }

            let max_start = file_length.saturating_sub(length);
            let start_point = rand::thread_rng().gen_range(0..=max_start);

            reader.seek(io::SeekFrom::Start(start_point))?;

            let mut entropy_raw_binary = String::new();
            reader.take(length).read_to_string(&mut entropy_raw_binary)?;

            Ok(entropy_raw_binary)
        },
        "File" => {
            todo!() // Create shannon entropy and support any file as entropy source

        },
        _ => Err("Invalid entropy source specified".into()),
    }
}

fn generate_checksum(entropy: &str, entropy_length: &u32) -> String {
    let entropy_binary = convert_string_to_binary(&entropy);
    let hash_raw_binary: String = convert_binary_to_string(&Sha256::digest(&entropy_binary));
    let checksum_lenght = entropy_length / 32;
    let checksum: String = hash_raw_binary.chars().take(checksum_lenght.try_into().unwrap()).collect();

    checksum
}

fn calculate_checksum(data: &[u8]) -> [u8; 4] {
    let hash1 = Sha256::digest(data);
    let hash2 = Sha256::digest(&hash1);
    let checksum = &hash2[..4];
    let mut result = [0u8; 4];
    result.copy_from_slice(checksum);
    result
}

fn generate_mnemonic_words(final_entropy_binary: &str) -> String {
    let chunks: Vec<String> = final_entropy_binary.chars().collect::<Vec<char>>().chunks(11).map(|chunk| chunk.iter().collect()).collect();
    let mnemonic_decimal: Vec<u32> = chunks.iter().map(|chunk| u32::from_str_radix(chunk, 2).unwrap()).collect();
    
    let mnemonic_file_content = match fs::read_to_string(WORDLIST_FILE) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading wordlist file: {}", err);
            return String::new();
        }
    };
    
    let mnemonic_words: Vec<&str> = mnemonic_file_content.lines().collect();
    let mnemonic_words: Vec<&str> = mnemonic_decimal.iter().map(|&decimal| {
        if (decimal as usize) < mnemonic_words.len() {
            mnemonic_words[decimal as usize]
        } else {
            "INVALID_WORD"
        }
    }).collect();

    let final_mnemonic = mnemonic_words.join(" ");

    final_mnemonic
}

fn generate_bip39_seed(entropy: &str, passphrase: &str) -> [u8; 64] {
    let entropy_vector = convert_string_to_binary(&entropy);
    let mnemonic = bip39::Mnemonic::from_entropy(&entropy_vector).expect("Can not create mnemomic words");
    let seed = bip39::Mnemonic::to_seed(&mnemonic, passphrase);

    seed
}

fn derive_master_keys(seed: &str, mut private_header: &str, mut public_header: &str,) -> Result<(String, String), String> {
    // Reverting to Bitcoin in case that coin is undefined
    if private_header.is_empty() {private_header = "0x0488ADE4";}
    if public_header.is_empty() {public_header = "0x0488B21E";}
    // Default message for all blockchains
    let message = "Bitcoin seed";

    let private_header = u32::from_str_radix(private_header.trim_start_matches("0x"), 16)
        .expect("Can not parse private header");
    let public_header = u32::from_str_radix(public_header.trim_start_matches("0x"), 16)
        .expect("Can not parse public header");

    let seed_bytes = hex::decode(seed).expect("Can not decode seed");
    let hmac_result = hmac_sha512(message.as_bytes(), &seed_bytes);
    let (master_private_key_bytes, chain_code_bytes) = hmac_result.split_at(32);

    // Private construct
    let mut master_private_key = Vec::new();
    master_private_key.extend_from_slice(&u32::to_be_bytes(private_header));     // Version        4 bytes
    master_private_key.push(0x00);                                                    // Depth          1 byte
    master_private_key.extend([0x00; 4].iter());                                      // Parent finger  4 bytes
    master_private_key.extend([0x00; 4].iter());                                      // Index/child    4 bytes
    master_private_key.extend_from_slice(chain_code_bytes);                           // Chain code     32 bytes
    master_private_key.push(0x00);                                                    // Key prefix     1 byte
    master_private_key.extend_from_slice(master_private_key_bytes);                   // Key            32 bytes
    let checksum: [u8; 4] = calculate_checksum(&master_private_key);            // Checksum       4 bytes
    master_private_key.extend_from_slice(&checksum);
    
    let master_xprv = bs58::encode(&master_private_key).into_string(); // Total      82 bytes
    println!("Master private key: {}", master_xprv);

    // Public construct
    let secp = secp256k1::Secp256k1::new();
    let master_secret_key = secp256k1::SecretKey::from_slice(&master_private_key_bytes)
        .map_err(|e| format!("Error creating private key: {:?}", e))?;
    let master_public_key_bytes = secp256k1::PublicKey::from_secret_key(&secp, &master_secret_key).serialize();

    let mut master_public_key = Vec::new();
    master_public_key.extend_from_slice(&u32::to_be_bytes(public_header));      // Version        4 bytes
    master_public_key.push(0x00);                                                     // Depth          1 byte
    master_public_key.extend([0x00; 4].iter());                                       // Parent finger  4 bytes
    master_public_key.extend([0x00; 4].iter());                                       // Index/child    4 bytes
    master_public_key.extend_from_slice(chain_code_bytes);                            // Chain code     32 bytes
    master_public_key.extend_from_slice(&master_public_key_bytes);                    // Key            33 bytes (compressed)
    let checksum: [u8; 4] = calculate_checksum(&master_public_key);              // Checksum       4 bytes
    master_public_key.extend_from_slice(&checksum);
    
    let master_xpub = bs58::encode(&master_public_key).into_string();   // Total      82 bytes
    println!("Master public key: {}", master_xpub);

    Ok((master_xprv, master_xpub))
}

fn hmac_sha512(key: &[u8], data: &[u8]) -> Vec<u8> {
    const BLOCK_SIZE: usize = 128;
    const HASH_SIZE: usize = 64;

    let mut padded_key = [0x00; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        let mut hasher = Sha512::new();
        hasher.update(key);
        padded_key[..HASH_SIZE].copy_from_slice(&hasher.finalize());
    } else {
        padded_key[..key.len()].copy_from_slice(key);
    }

    let mut ipad = padded_key.clone();
    let mut opad = padded_key.clone();

    // XOR key with ipad and opad
    ipad.iter_mut().for_each(|byte| *byte ^= 0x36);
    opad.iter_mut().for_each(|byte| *byte ^= 0x5c);

    // Append data to ipad
    let mut ipad_data = vec![0x00; BLOCK_SIZE + data.len()];
    ipad_data[..BLOCK_SIZE].copy_from_slice(&ipad);
    ipad_data[BLOCK_SIZE..].copy_from_slice(&data);

    // Calculate inner hash
    let inner_hash = Sha512::digest(&ipad_data);

    // Append inner hash to opad
    let mut opad_inner = vec![0x00; BLOCK_SIZE + HASH_SIZE];
    opad_inner[..BLOCK_SIZE].copy_from_slice(&opad);
    opad_inner[BLOCK_SIZE..].copy_from_slice(&inner_hash);
    // println!("opad_inner length: {}", opad_inner.len());
    // println!("inner_hash length: {}", inner_hash.len());
    // Calculate outer hash
    Sha512::digest(&opad_inner).to_vec() 
}



// COINS
// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
struct CoinDatabase {
    index: u32,
    path: u32,
    symbol: String,
    name: String,
    key_derivation: String,
    private_header: String,
    public_header: String,
    public_key_hash: String,
    script_hash: String,
    wif: String,
    comment: String,
}

fn create_coin_store() -> Vec<CoinDatabase> {
    let file = File::open(&COINLIST_FILE).expect("can not open bip44 coin file");
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
    let mut coin_store = Vec::new();

    for result in rdr.records() {
        let record = result.expect("error reading CSV record");

        let index: u32 = record[0].parse().expect("error parsing index");
        let path: u32 = u32::from_str_radix(&record[1][2..], 16).expect("error parsing path");
        let symbol: String = if record[2].is_empty()            {"".to_string()} else {record[2].to_string()};
        let name: String = if record[3].is_empty()              {"".to_string()} else {record[3].to_string()};
        let key_derivation:String = if record[4].is_empty()     {"".to_string()} else {record[4].to_string()};
        let private_header: String = if record[5].is_empty()    {"".to_string()} else {record[5].to_string()};
        let public_header: String = if record[6].is_empty()     {"".to_string()} else {record[6].to_string()};
        let public_key_hash: String = if record[7].is_empty()   {"".to_string()} else {record[7].to_string()};
        let script_hash: String = if record[8].is_empty()       {"".to_string()} else {record[8].to_string()};
        let wif: String = if record[9].is_empty()               {"".to_string()} else {record[9].to_string()};
        let comment: String = if record[10].is_empty()          {"".to_string()} else {record[10].to_string()};
        
        let coin_type = CoinDatabase { 
            index, 
            path, 
            symbol, 
            name, 
            key_derivation, 
            private_header, 
            public_header, 
            public_key_hash, 
            script_hash, 
            wif, 
            comment 
        };

        coin_store.push(coin_type);
    }

    coin_store
}

fn create_coin_completion_model() -> gtk::ListStore {
    let valid_coin_symbols = create_coin_database(COINLIST_FILE);

    let store = gtk::ListStore::new(&[
        glib::Type::U32,    // Index
        glib::Type::U32,    // Path
        glib::Type::STRING, // Symbol
        glib::Type::STRING, // Name
        glib::Type::STRING, // key_derivation
        glib::Type::STRING, // private_header
        glib::Type::STRING, // public_header
        glib::Type::STRING, // public_key_hash
        glib::Type::STRING, // script_hash
        glib::Type::STRING, // Wif
        glib::Type::STRING, // Comment
    ]);

    for coin_symbol in valid_coin_symbols.iter() {
        let iter = store.append();
        store.set(&iter, &[
            (0, &coin_symbol.index), 
            (1, &coin_symbol.path), 
            (2, &coin_symbol.symbol), 
            (3, &coin_symbol.name),
            (4, &coin_symbol.key_derivation),
            (5, &coin_symbol.private_header),
            (6, &coin_symbol.public_header),
            (7, &coin_symbol.public_key_hash),
            (8, &coin_symbol.script_hash),
            (9, &coin_symbol.wif),
            (10, &coin_symbol.comment),
        ]);
    }

    store
}

fn get_coins_starting_with<'a>(coin_store: &'a Vec<CoinDatabase>, target_prefix: &'a str) -> Vec<&'a CoinDatabase> {
    coin_store
        .iter()
        .filter(|&coin_type| coin_type.symbol.starts_with(target_prefix))
        .collect()
}

fn create_coin_database(file_path: &str) -> Vec<CoinDatabase> {
    let file = File::open(&file_path).expect("can not read file");
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

    let coin_types: Vec<CoinDatabase> = rdr
        .records()
        .filter_map(|record| record.ok())
        .enumerate()
        .map(|(index, record)| {
            
            let path: u32 = index as u32;
            let index: u32 = index.try_into().expect("Conversion from usize to u32 failed");
            let symbol: String = record.get(2).unwrap_or_default().to_string();
            let name: String = record.get(3).unwrap_or_default().to_string();
            let key_derivation: String = record.get(4).unwrap_or_default().to_string();
            let private_header: String = record.get(5).unwrap_or_default().to_string();
            let public_header: String = record.get(6).unwrap_or_default().to_string();
            let public_key_hash: String = record.get(7).unwrap_or_default().to_string();
            let script_hash: String = record.get(8).unwrap_or_default().to_string();
            let wif: String = record.get(9).unwrap_or_default().to_string();
            let comment: String = record.get(10).unwrap_or_default().to_string();

            CoinDatabase { index, path, symbol, name, key_derivation, private_header, public_header, public_key_hash, script_hash, wif, comment }
            }
        )
        .collect();

    coin_types
}



// GUI
// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
struct AppSettings {
    entropy_source: String,
    entropy_length: u32,
    bip: u32,
    gui_save_window_size: bool,
    gui_last_width: u32,
    gui_last_height: u32,
    anu_data_format: String,
    anu_array_length: u32,
    anu_hex_block_size: u32,

}

impl AppSettings {
    fn load_settings() -> io::Result<Self> {
        let config_file = "config/custom.conf";
        let default_config_file = "config/default.conf";

        if !Path::new(config_file).exists() {
            fs::copy(default_config_file, config_file)?;
        }

        let config_str = fs::read_to_string(config_file)?;
        let config: toml::Value = config_str.parse().expect("Failed to parse config file");

        // APP settings
        let app_section = config.get("app").expect("Missing 'app' section");
        let gui_save_window_size = app_section.get("save_window")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let gui_last_width = app_section.get("window_width")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(800);

        let gui_last_height = app_section.get("window_height")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(1024);

        // Wallet settings
        let wallet_section = config.get("wallet").expect("Missing 'wallet' section");
        let entropy_source = wallet_section.get("entropy_source")
            .and_then(|v| v.as_str())
            .unwrap_or("rng")
            .to_string();

        let entropy_length = wallet_section.get("entropy_length")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(256);

        let bip = wallet_section.get("bip")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(44);

        
        // ANU settings
        let anu_section = config.get("anu").expect("Missing 'anu' section");
        let anu_data_format = anu_section.get("data_format")
            .and_then(|v| v.as_str())
            .unwrap_or("uint8")
            .to_string();

        let anu_array_length = anu_section.get("array_length")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(1024);
        
        let anu_hex_block_size = anu_section.get("hex_block_size")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(24);


        // Create and return AppSettings instance
        Ok(AppSettings {
            entropy_source,
            entropy_length,
            bip,
            gui_save_window_size,
            gui_last_width,
            gui_last_height,
            anu_data_format,
            anu_array_length,
            anu_hex_block_size,
        })
    }

    fn get_value(&self, name: &str) -> Option<String> {
        match name {
            "entropy_source" => Some(self.entropy_source.clone()),
            "entropy_length" => Some(self.entropy_length.to_string()),
            "bip" => Some(self.bip.to_string()),
            "gui_save_window_size" => Some(self.gui_save_window_size.to_string()),
            "gui_last_width" => Some(self.gui_last_width.to_string()),
            "gui_last_height" => Some(self.gui_last_height.to_string()),
            "anu_data_format" => Some(self.anu_data_format.clone()),
            "anu_array_length" => Some(self.anu_array_length.to_string()),
            "anu_hex_block_size" => Some(self.anu_hex_block_size.to_string()),
            _ => None,
        }
    }
}

fn create_settings_window() {
    let settings = AppSettings::load_settings().expect("Can not read settings");

    let settings_window = gtk::ApplicationWindow::builder()
        .title("Settings")
        .default_width(600)
        .default_height(400)
        .resizable(false)
        .build();

    // TODO: Create settings icon
    // settings_window.set_icon_name(Some("org.gnome.Settings"));

    let stack = Stack::new();
    let stack_sidebar = StackSidebar::new();
    stack_sidebar.set_stack(&stack);
    
    // Sidebar 1: General settings
    let general_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let content_general_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let general_settings_frame = gtk::Frame::new(Some(" App settings"));

    general_settings_box.set_margin_bottom(10);
    general_settings_box.set_margin_top(10);
    general_settings_box.set_margin_start(10);
    general_settings_box.set_margin_end(10);
    content_general_box.set_margin_start(20);
    general_settings_box.append(&general_settings_frame);
    general_settings_frame.set_child(Some(&content_general_box));
    general_settings_frame.set_hexpand(true);
    general_settings_frame.set_vexpand(true);

    stack.add_titled(&general_settings_box, Some("sidebar-settings-general"), "General");
 

    // Sidebar 2: Wallet settings
    let wallet_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let content_wallet_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let wallet_settings_frame = gtk::Frame::new(Some(" Wallet settings"));
    
    wallet_settings_box.set_margin_bottom(10);
    wallet_settings_box.set_margin_top(10);
    wallet_settings_box.set_margin_start(10);
    wallet_settings_box.set_margin_end(10);
    content_wallet_box.set_margin_start(20);
    wallet_settings_box.append(&wallet_settings_frame);
    wallet_settings_frame.set_child(Some(&content_wallet_box));
    wallet_settings_frame.set_hexpand(true);
    wallet_settings_frame.set_vexpand(true);

    // Default entropy source
    let default_entropy_source_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let default_entropy_source_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_entropy_source_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_entropy_source_label = gtk::Label::new(Some("Entropy source:"));
    let valid_entropy_source_as_strings: Vec<String> = VALID_ENTROPY_SOURCES.iter().map(|&x| x.to_string()).collect();
    let valid_entropy_source_as_str_refs: Vec<&str> = valid_entropy_source_as_strings.iter().map(|s| s.as_ref()).collect();
    let entropy_source_dropdown = gtk::DropDown::from_strings(&valid_entropy_source_as_str_refs);
    let default_entropy_source = valid_entropy_source_as_strings
        .iter()
        .position(|s| *s == settings.entropy_source) 
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
    let default_entropy_length_label = gtk::Label::new(Some("Entropy length:"));
    let valid_entropy_lengths_as_strings: Vec<String> = VALID_ENTROPY_LENGTHS.iter().map(|&x| x.to_string()).collect();
    let valid_entropy_lengths_as_str_refs: Vec<&str> = valid_entropy_lengths_as_strings.iter().map(|s| s.as_ref()).collect();
    let entropy_length_dropdown = gtk::DropDown::from_strings(&valid_entropy_lengths_as_str_refs);
    let default_entropy_length = valid_entropy_lengths_as_strings
        .iter()
        .position(|x| x.parse::<u32>().unwrap() == settings.entropy_length)
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
    let default_bip_label = gtk::Label::new(Some("BIP:"));
    let valid_bips_as_strings: Vec<String> = VALID_BIP_DERIVATIONS.iter().map(|&x| x.to_string()).collect();
    let valid_bips_as_str_refs: Vec<&str> = valid_bips_as_strings.iter().map(|s| s.as_ref()).collect();
    let bip_dropdown = gtk::DropDown::from_strings(&valid_bips_as_str_refs);
    let default_bip = valid_bips_as_strings
        .iter()
        .position(|x| x.parse::<u32>().unwrap() == settings.bip)
        .unwrap_or(0);

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

    stack.add_titled(&wallet_settings_box, Some("sidebar-settings-wallet"), "Wallet");


    // Sidebar 3: ANU settings
    let anu_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let content_anu_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let anu_settings_frame = gtk::Frame::new(Some(" App settings"));

    anu_settings_box.set_margin_bottom(10);
    anu_settings_box.set_margin_top(10);
    anu_settings_box.set_margin_start(10);
    anu_settings_box.set_margin_end(10);
    content_anu_box.set_margin_start(20);
    anu_settings_box.append(&anu_settings_frame);
    anu_settings_frame.set_child(Some(&content_anu_box));
    anu_settings_frame.set_hexpand(true);
    anu_settings_frame.set_vexpand(true);

    stack.add_titled(&anu_settings_box, Some("sidebar-settings-anu"), "ANU");



    // Compose settings window
    let main_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let main_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    main_content_box.append(&stack_sidebar);
    main_content_box.append(&stack);
    
    // Buttons
    let buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let save_button = gtk::Button::with_label("Save");
    let cancel_button = gtk::Button::with_label("Cancel");

    cancel_button.connect_clicked(clone!(
        @weak settings_window => move |_| {
            settings_window.close()
        }
    ));

    buttons_box.append(&save_button);
    buttons_box.append(&cancel_button);
    buttons_box.set_margin_bottom(10);
    buttons_box.set_margin_top(10);
    buttons_box.set_margin_start(10);
    buttons_box.set_margin_end(10);
    buttons_box.set_direction(gtk::TextDirection::Rtl);
    main_settings_box.append(&main_content_box);
    main_settings_box.append(&buttons_box);
    settings_window.set_child(Some(&main_settings_box));

    settings_window.show();
}

fn create_about_window() {
    let logo = gtk::gdk::Texture::from_file(&gio::File::for_path("lib/logo.svg")).expect("msg");
    let license = fs::read_to_string("LICENSE.txt").unwrap();

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
        .license(license)
        .wrap_license(true)
        .comments("(Q)RNG crypto key generator")
        .logo(&logo)
        .build();

    help_window.show();

}

fn create_main_window(application: &adw::Application) {
    let settings = AppSettings::load_settings().expect("Can not read settings");

    let window_width = match settings.get_value("gui_last_width") {
        Some(width_str) => width_str.parse::<i32>().unwrap_or_else(|_| {
            eprintln!("Failed to parse default window width value: {}", width_str);
            1200
        }),
        None => {
            eprintln!("'gui_last_width' not found in settings");
            1200
        }
    };

    let window_height = match settings.get_value("gui_last_height") {
        Some(height_str) => height_str.parse::<i32>().unwrap_or_else(|_| {
            eprintln!("Failed to parse default window height value: {}", height_str);
            800
        }),
        None => {
            eprintln!("'gui_last_height' not found in settings");
            800
        }
    };

    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title(&format!("{} {}", APP_DESCRIPTION.unwrap(), APP_VERSION.unwrap()))
        .default_width(window_width)
        .default_height(window_height)
        .show_menubar(true)
        .build();

    // TODO: Set main window icon
    window.set_icon_name(Some("org.gnome.Settings"));

    let header_bar = gtk::HeaderBar::new();
    window.set_titlebar(Some(&header_bar));

    let new_wallet_button = gtk::Button::new();
    let open_wallet_button = gtk::Button::new();
    let save_wallet_button = gtk::Button::new();
    let settings_button = gtk::Button::new();
    let about_button = gtk::Button::new();

    new_wallet_button.set_icon_name("tab-new-symbolic");
    open_wallet_button.set_icon_name("document-open-symbolic");
    save_wallet_button.set_icon_name("document-save-symbolic");
    settings_button.set_icon_name("org.gnome.Settings-symbolic");
    about_button.set_icon_name("help-about-symbolic");
    
    header_bar.pack_start(&new_wallet_button);
    header_bar.pack_start(&open_wallet_button);
    header_bar.pack_start(&save_wallet_button);
    header_bar.pack_end(&settings_button);
    header_bar.pack_end(&about_button);

    settings_button.connect_clicked(move |_| {
        create_settings_window();
    });

    about_button.connect_clicked(move |_| {
        create_about_window();
    });

    // New wallet (window) CTRL+N
    let new_window = application.clone();
    new_wallet_button.connect_clicked(move |_| {
        create_main_window(&new_window);
    });

    // Headerbar button tooltips
    new_wallet_button.set_tooltip_text(Some("New wallet (Ctrl+N)"));
    open_wallet_button.set_tooltip_text(Some("Open wallet (Ctrl+O)"));
    save_wallet_button.set_tooltip_text(Some("Save wallet (Ctrl+S)"));
    settings_button.set_tooltip_text(Some("Settings (F5)"));
    about_button.set_tooltip_text(Some("About (F1)"));

    let stack = Stack::new();
    let stack_sidebar = StackSidebar::new();
    stack_sidebar.set_stack(&stack);


    // -.-. --- .--. -.-- .-. .. --. .... -
    // Sidebar 1: Seed
    // -.-. --- .--. -.-- .-. .. --. .... -
    let entropy_main_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    entropy_main_box.set_margin_top(10);
    entropy_main_box.set_margin_start(10);
    entropy_main_box.set_margin_end(10);
    entropy_main_box.set_margin_bottom(10);

    // Header
    let entropy_header_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let entropy_header_first_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let entropy_header_second_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);

    // Entropy source
    let entropy_source_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let entropy_source_frame = gtk::Frame::new(Some("Entropy source"));
    let valid_entropy_source_as_strings: Vec<String> = VALID_ENTROPY_SOURCES.iter().map(|&x| x.to_string()).collect();
    let valid_entropy_source_as_str_refs: Vec<&str> = valid_entropy_source_as_strings.iter().map(|s| s.as_ref()).collect();
    let entropy_source_dropdown = gtk::DropDown::from_strings(&valid_entropy_source_as_str_refs);
    let default_entropy_source = valid_entropy_source_as_strings
        .iter()
        .position(|s| *s == settings.entropy_source) 
        .unwrap_or(0);

    entropy_source_dropdown.set_selected(default_entropy_source.try_into().unwrap());
    entropy_source_box.set_hexpand(true);
    entropy_source_frame.set_hexpand(true);
    
    // Entropy length
    let entropy_length_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let entropy_length_frame = gtk::Frame::new(Some("Entropy length"));
    let valid_entropy_lengths_as_strings: Vec<String> = VALID_ENTROPY_LENGTHS.iter().map(|&x| x.to_string()).collect();
    let valid_entropy_lengths_as_str_refs: Vec<&str> = valid_entropy_lengths_as_strings.iter().map(|s| s.as_ref()).collect();
    let entropy_length_dropdown = gtk::DropDown::from_strings(&valid_entropy_lengths_as_str_refs);
    let default_entropy_length = valid_entropy_lengths_as_strings
        .iter()
        .position(|x| x.parse::<u32>().unwrap() == settings.entropy_length)
        .unwrap_or(0);

    entropy_length_dropdown.set_selected(default_entropy_length.try_into().unwrap());
    entropy_length_box.set_hexpand(true);
    entropy_length_frame.set_hexpand(true);

    // Mnemonic passphrase
    let mnemonic_passphrase_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let mnemonic_passphrase_frame = gtk::Frame::new(Some("Mnemonic passphrase"));
    let mnemonic_passphrase_text = gtk::Entry::new();
    mnemonic_passphrase_box.set_hexpand(true);
    mnemonic_passphrase_text.set_hexpand(true);
    
    // Generate button
    let generate_seed_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let generate_seed_button = gtk::Button::new();
    generate_seed_button.set_label("Generate seed");
    generate_seed_box.set_halign(gtk::Align::Center);

    // Connections
    entropy_source_frame.set_child(Some(&entropy_source_dropdown));
    entropy_length_frame.set_child(Some(&entropy_length_dropdown));

    generate_seed_box.append(&generate_seed_button);
    entropy_source_box.append(&entropy_source_frame);
    entropy_length_box.append(&entropy_length_frame);
    entropy_header_first_box.append(&entropy_source_box);
    entropy_header_first_box.append(&entropy_length_box);
    entropy_header_second_box.append(&mnemonic_passphrase_box);
    entropy_header_box.append(&entropy_header_first_box);
    entropy_header_box.append(&entropy_header_second_box);
    entropy_header_box.append(&generate_seed_box);

    // Body
    let body_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    
    // Entropy string
    let entropy_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let entropy_frame = gtk::Frame::new(Some("Entropy"));
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
    let mnemonic_words_frame = gtk::Frame::new(Some("Mnemonic words"));
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
    let seed_frame = gtk::Frame::new(Some("Seed"));
    let seed_text = gtk::TextView::new();
    seed_box.set_hexpand(true);
    seed_text.set_editable(false);
    seed_text.set_vexpand(true);
    seed_text.set_hexpand(true);
    seed_text.set_left_margin(5);
    seed_text.set_top_margin(5);
    seed_text.set_wrap_mode(gtk::WrapMode::Char);

    // Connections
    mnemonic_words_frame.set_child(Some(&mnemonic_words_text));
    mnemonic_passphrase_frame.set_child(Some(&mnemonic_passphrase_text));
    seed_frame.set_child(Some(&seed_text));
    mnemonic_words_box.append(&mnemonic_words_frame);
    mnemonic_passphrase_box.append(&mnemonic_passphrase_frame);
    seed_box.append(&seed_frame);
    body_box.append(&entropy_box);
    body_box.append(&mnemonic_words_box);
    body_box.append(&seed_box);
    entropy_main_box.append(&entropy_header_box);
    entropy_main_box.append(&body_box);
    
    generate_seed_button.connect_clicked(clone!(
        @weak entropy_source_dropdown,
        @weak entropy_length_dropdown,
        @weak mnemonic_words_text,
        @weak seed_text => move |_| {
            let selected_entropy_source_index = entropy_source_dropdown.selected() as usize;
            let selected_entropy_length_index = entropy_length_dropdown.selected() as usize;
            let selected_entropy_source_value = VALID_ENTROPY_SOURCES.get(selected_entropy_source_index);
            let selected_entropy_length_value = VALID_ENTROPY_LENGTHS.get(selected_entropy_length_index);
            let source = selected_entropy_source_value.unwrap().to_string();
            let length = selected_entropy_length_value.unwrap();
            
            println!("Entropy source: {:?}", source);
            println!("Entropy length: {:?}", length);

            let entropy_length = selected_entropy_length_value;
            
            let pre_entropy = generate_entropy(
                &source,
                *length as u64,
                None // Some(&ENTROPY_FILE)
            ).unwrap();
            
            let checksum = generate_checksum(&pre_entropy, entropy_length.unwrap());
            println!("Entropy: {:?}", &pre_entropy);
            println!("Checksum: {:?}", &checksum);
            let full_entropy = format!("{}{}", &pre_entropy, &checksum);
            entropy_text.buffer().set_text(&full_entropy);
            
            let mnemonic_words = generate_mnemonic_words(&full_entropy);
            mnemonic_words_text.buffer().set_text(&mnemonic_words);
            println!("Mnemonic words: {:?}", mnemonic_words);

            let passphrase_text = mnemonic_passphrase_text.text().to_string();
            println!("Mnemonic passphrase: {:?}", &passphrase_text);
            
            let seed = generate_bip39_seed(&pre_entropy, &passphrase_text);
            let seed_hex = hex::encode(&seed[..]);
            seed_text.buffer().set_text(&seed_hex.to_string());
            println!("Seed: {:?}", &seed_hex.to_string());
            
        }
    ));

    // Start Seed sidebar
    stack.add_titled(&entropy_main_box, Some("sidebar-seed"), "Seed");


    // -.-. --- .--. -.-- .-. .. --. .... -
    // Sidebar 2: Coin
    // -.-. --- .--. -.-- .-. .. --. .... -
    let coin_main_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let coin_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let coin_frame = gtk::Frame::new(Some("Coin"));
    coin_main_box.set_margin_top(10);
    coin_main_box.set_margin_start(10);
    coin_main_box.set_margin_end(10);
    coin_main_box.set_margin_bottom(10);

    // Create scrolled window
    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_max_content_height(400); // Set maximum height

    // Coin treeview
    create_coin_completion_model();
    let coins = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let coin_treeview = gtk::TreeView::new();
    coin_treeview.set_vexpand(true);
    coin_treeview.set_headers_visible(true);

    let columns = ["Index", "Path", "Symbol", "Name", "Key derivation", "Private header", "Public header", "public_key_hash", "script_hash", "Wif", "Comment"];
    for (i, column_title) in columns.iter().enumerate() {
        let column = gtk::TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();
        column.set_title(column_title);
        column.pack_start(&cell, true);
        column.add_attribute(&cell, "text", i as i32);
        coin_treeview.append_column(&column);
    }

    // Coin search
    let coin_search = gtk::SearchEntry::new();
    coin_search.set_placeholder_text(Some("Find a coin by entering its symbol (BTC, LTC, ETH...)"));

    coins.append(&coin_search);
    scrolled_window.set_child(Some(&coin_treeview));
    coins.append(&scrolled_window);
    coin_frame.set_child(Some(&coins));
    coin_box.append(&coin_frame);

    // Master private key
    let master_keys_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let master_xprv_frame = gtk::Frame::new(Some("Master private key"));
    let master_xpub_frame = gtk::Frame::new(Some("Master public key"));
    
    let master_private_key_text = gtk::TextView::new();
    let master_public_key_text = gtk::TextView::new();

    master_private_key_text.set_editable(false);
    master_public_key_text.set_editable(false);
    
    master_xprv_frame.set_child(Some(&master_private_key_text));
    master_xpub_frame.set_child(Some(&master_public_key_text));

    master_private_key_text.set_wrap_mode(gtk::WrapMode::Char);
    master_private_key_text.set_editable(false);
    master_private_key_text.set_left_margin(5);
    master_private_key_text.set_top_margin(5);

    master_public_key_text.set_wrap_mode(gtk::WrapMode::Char);
    master_public_key_text.set_editable(false);
    master_public_key_text.set_left_margin(5);
    master_public_key_text.set_top_margin(5);

    // Connections 
    master_keys_box.append(&master_xprv_frame);
    master_keys_box.append(&master_xpub_frame);
    coin_main_box.append(&coin_box);
    coin_main_box.append(&master_keys_box);
    
    // Actions
    let coin_store = create_coin_store();
    let treestore = gtk4::TreeStore::new(&[glib::Type::STRING; 11]);
    let coin_treeview_clone = coin_treeview.clone();

    coin_treeview.connect_cursor_changed(move |_| {
        if let Some((model, iter)) = coin_treeview_clone.selection().selected() {
            let coin = model.get_value(&iter, 0);
            let header = model.get_value(&iter, 1);
            let symbol = model.get_value(&iter, 2);
            let name = model.get_value(&iter, 3);
            let key_derivation = model.get_value(&iter, 4);
            let private_header = model.get_value(&iter, 5);
            let public_header = model.get_value(&iter, 6);
            let public_key_hash = model.get_value(&iter, 7);
            let script_hash = model.get_value(&iter, 8);
            let wif = model.get_value(&iter, 9);
            let comment = model.get_value(&iter, 10);

            if let (
                Ok(coin_type),
                Ok(coin_header),
                Ok(coin_symbol),
                Ok(coin_name),
                Ok(key_derivation),
                Ok(private_header),
                Ok(public_header),
                Ok(public_key_hash),
                Ok(script_hash),
                Ok(wif),
                Ok(comment),
            ) = (
                coin.get::<String>(), 
                header.get::<String>(), 
                symbol.get::<String>(), 
                name.get::<String>(),
                key_derivation.get::<String>(),
                private_header.get::<String>(),
                public_header.get::<String>(),
                public_key_hash.get::<String>(),
                script_hash.get::<String>(),
                wif.get::<String>(),
                comment.get::<String>(),
            ) 
                {
                    println!("coin_type: {}", coin_type);
                    println!("coin_header: {}", coin_header);
                    println!("coin_symbol: {}", coin_symbol);
                    println!("coin_name: {}", coin_name);
                    println!("key_derivation: {}", key_derivation);
                    println!("private_header: {}", private_header);
                    println!("public_header: {}", public_header);
                    println!("public_key_hash: {}", public_key_hash);
                    println!("script_hash: {}", script_hash);
                    println!("wif: {}", wif);
                    println!("comment: {}", comment);
                    let buffer = seed_text.buffer();
                    let start_iter = buffer.start_iter();
                    let end_iter = buffer.end_iter();
                    let seed_string = buffer.text(&start_iter, &end_iter, true);
                    println!("Seed: {}", seed_string);
                    
                    match derive_master_keys(
                        &seed_string, 
                        // &coin_symbol,
                        &private_header,
                        &public_header,
                        // false,
                    ) {
                        Ok(xprv) => {
                            master_private_key_text.buffer().set_text(&xprv.0);
                            master_public_key_text.buffer().set_text(&xprv.1);
                        },
                        Err(err) => println!("Can not derive master keys: {}", err),
                    }
                }
                
        }
    });
    
    coin_search.connect_search_changed(move|coin_search| {
        let search_text = coin_search.text().to_uppercase();
        treestore.clear();
    
        if search_text.len() >= 2 {
            let matching_coins = get_coins_starting_with(&coin_store, &search_text);
            
            if !matching_coins.is_empty() {
                treestore.clear();
                
                for found_coin in matching_coins {
                    let iter = treestore.append(None);
                    treestore.set(&iter, &[
                        (0, &found_coin.index.to_string()),
                        (1, &format!("0x{:X}", found_coin.path)),
                        (2, &found_coin.symbol),
                        (3, &found_coin.name),
                        (4, &found_coin.key_derivation),
                        (5, &found_coin.private_header),
                        (6, &found_coin.public_header),
                        (7, &found_coin.public_key_hash),
                        (8, &found_coin.script_hash),
                        (9, &found_coin.wif),
                        (10, &found_coin.comment),
                    ]);
                }
                coin_treeview.set_model(Some(&treestore));
            } else {
                treestore.clear();
            }
        } else {
            treestore.clear();
        }
    });

    stack.add_titled(&coin_main_box, Some("sidebar-coin"), "Coin");


    // -.-. --- .--. -.-- .-. .. --. .... -
    // Sidebar 3 
    // -.-. --- .--. -.-- .-. .. --. .... -
    let main_address_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    main_address_box.set_hexpand(true);
    main_address_box.set_vexpand(true);
    main_address_box.set_margin_top(10);
    main_address_box.set_margin_start(10);
    main_address_box.set_margin_end(10);
    main_address_box.set_margin_bottom(10);

    let derivation_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    
    let main_bip_frame = gtk::Frame::new(Some("BIP"));
    main_bip_frame.set_hexpand(true);
    // main_bip_frame.set_vexpand(true);
    
    let main_coin_frame = gtk::Frame::new(Some("Coin"));
    main_coin_frame.set_hexpand(true);
    // main_coin_frame.set_vexpand(true);

    let main_address_frame = gtk::Frame::new(Some("Address"));
    main_address_frame.set_hexpand(true);
    // main_address_frame.set_vexpand(true);
    
    let main_purpose_frame = gtk::Frame::new(Some("Purpose"));
    main_purpose_frame.set_hexpand(true);
    // main_purpose_frame.set_vexpand(true);

    let bip_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let coin_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let address_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let purpose_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    purpose_box.set_hexpand(true);
    // purpose_box.set_vexpand(true);

    let bip_frame = gtk::Frame::new(Some("BIP"));
    bip_frame.set_hexpand(true);
    // bip_frame.set_vexpand(true);

    let bip_hardened_frame = gtk::Frame::new(Some("Hardened?"));
    bip_hardened_frame.set_hexpand(true);
    // bip_hardened_frame.set_vexpand(true);
    
    let coin_frame = gtk::Frame::new(Some("Coin"));
    coin_frame.set_hexpand(true);
    // coin_frame.set_vexpand(true);
    
    let coin_hardened_frame = gtk::Frame::new(Some("Hardened?"));
    coin_hardened_frame.set_hexpand(true);
    // coin_hardened_frame.set_vexpand(true);
    
    let address_frame = gtk::Frame::new(Some("Address"));
    address_frame.set_hexpand(true);
    // address_frame.set_vexpand(true);

    let address_hardened_frame = gtk::Frame::new(Some("Hardened?"));
    address_hardened_frame.set_hexpand(true);
    // address_hardened_frame.set_vexpand(true);

    let purpose_frame = gtk::Frame::new(Some("Purpose"));
    purpose_frame.set_hexpand(true);
    // purpose_frame.set_vexpand(true);


    let valid_bip_as_string: Vec<String> = VALID_BIP_DERIVATIONS.iter().map(|&x| x.to_string()).collect();
    let valid_bip_as_ref: Vec<&str> = valid_bip_as_string.iter().map(|s| s.as_ref()).collect();
    let bip_dropdown = gtk::DropDown::from_strings(&valid_bip_as_ref);
    bip_dropdown.set_selected(1); // BIP44

    let bip_hardened_checkbox = gtk::CheckButton::new();
    bip_hardened_checkbox.set_active(true);
    
    let coin_entry = gtk::Entry::new();
    coin_entry.set_editable(false);
    
    let coin_hardened_checkbox = gtk::CheckButton::new();
    coin_hardened_checkbox.set_active(true);
    
    let adjustment = gtk::Adjustment::new(
        0.0, // initial value
        0.0, // minimum value
        2147483647.0, // maximum value
        1.0, // step increment
        100.0, // page increment
        0.0, // page size
    );
    let address_spinbutton = gtk::SpinButton::new(Some(&adjustment), 1.0, 0);
    
    let address_hardened_checkbox = gtk::CheckButton::new();
    address_hardened_checkbox.set_active(true);
    
    let valid_wallet_pupose_as_strings: Vec<String> = VALID_WALLET_PURPOSE.iter().map(|&x| x.to_string()).collect();
    let valid_wallet_pupose_as_ref: Vec<&str> = valid_wallet_pupose_as_strings.iter().map(|s| s.as_ref()).collect();
    let purpose_dropbox = gtk::DropDown::from_strings(&valid_wallet_pupose_as_ref);
    purpose_dropbox.set_selected(1); // External

    bip_frame.set_child(Some(&bip_dropdown));
    bip_hardened_frame.set_child(Some(&bip_hardened_checkbox));
    coin_frame.set_child(Some(&coin_entry));
    coin_hardened_frame.set_child(Some(&coin_hardened_checkbox));
    address_frame.set_child(Some(&address_spinbutton));
    address_hardened_frame.set_child(Some(&address_hardened_checkbox));
    purpose_frame.set_child(Some(&purpose_dropbox));

    bip_box.append(&bip_frame);
    bip_box.append(&bip_hardened_frame);
    coin_box.append(&coin_frame);
    coin_box.append(&coin_hardened_frame);
    address_box.append(&address_frame);
    address_box.append(&address_hardened_frame);
    purpose_box.append(&purpose_frame);

    main_bip_frame.set_child(Some(&bip_box));
    main_coin_frame.set_child(Some(&coin_box));
    main_address_frame.set_child(Some(&address_box));
    main_purpose_frame.set_child(Some(&purpose_box));

    derivation_box.append(&main_bip_frame);
    derivation_box.append(&main_coin_frame);
    derivation_box.append(&main_address_frame);
    derivation_box.append(&main_purpose_frame);


    let derivation_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let derivation_label_frame = gtk::Frame::new(Some("Derivation path"));
    derivation_label_frame.set_hexpand(true);
    // derivation_label_frame.set_vexpand(true);
    
    
    let derivation_label_text = gtk4::Label::builder()
        .label("m/44'/0'/0")            // TODO: get value from config file
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .css_classes(["large-title"])
        .build();

    derivation_label_box.append(&derivation_label_frame);
    derivation_label_frame.set_child(Some(&derivation_label_text));

    let address_treeview_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let address_treeview_frame = gtk::Frame::new(Some("Addresses"));
    address_treeview_frame.set_hexpand(true);
    address_treeview_frame.set_vexpand(true);

    let address_treeview = gtk::TreeView::new();
    address_treeview.set_headers_visible(true);
    let columns = ["Path", "Address", "Public key", "Private key"];
    for (i, column_title) in columns.iter().enumerate() {
        let column = gtk::TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();
        column.set_title(column_title);
        column.pack_start(&cell, true);
        column.add_attribute(&cell, "text", i as i32);
        address_treeview.append_column(&column);
    }

    address_treeview_frame.set_child(Some(&address_treeview));
    address_treeview_box.append(&address_treeview_frame);
    
    
    main_address_box.append(&derivation_box);
    main_address_box.append(&derivation_label_box);
    main_address_box.append(&address_treeview_box);
    
    stack.add_titled(&main_address_box, Some("sidebar-address"), "Address");
 

    let main_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    main_content_box.append(&stack_sidebar);
    main_content_box.append(&stack);
    window.set_child(Some(&main_content_box));

    window.present();
}

fn main() {
    print_program_info();

    let application = adw::Application::builder()
        .application_id("com.github.qr2m")
        .build();

    application.connect_activate(|app| {
        create_main_window(app);
    });

    let quit = gio::SimpleAction::new("quit", None);
    let new = gio::SimpleAction::new("new", None);
    let open = gio::SimpleAction::new("open", None);
    let save = gio::SimpleAction::new("save", None);
    let settings = gio::SimpleAction::new("settings", None);
    let about = gio::SimpleAction::new("about", None);
    
    quit.connect_activate(
        glib::clone!(@weak application => move |_action, _parameter| {
            application.quit();
        }),
    );

    
    let new_window = application.clone();
    new.connect_activate(move |_action, _parameter| {
        create_main_window(&new_window);
    });

    settings.connect_activate(move |_action, _parameter| {
        create_settings_window();
    });

    about.connect_activate(move |_action, _parameter| {
        create_about_window();
    });

    open.connect_activate(|_action, _parameter| {
        todo!() // Open wallet action activated
    });

    save.connect_activate(|_action, _parameter| {
        todo!() // Save wallet action activated
    });


    application.set_accels_for_action("app.quit", &["<Primary>Q"]);
    application.add_action(&quit);

    application.set_accels_for_action("app.new", &["<Primary>N"]);
    application.add_action(&new);

    application.set_accels_for_action("app.open", &["<Primary>O"]);
    application.add_action(&open);

    application.set_accels_for_action("app.save", &["<Primary>S"]);
    application.add_action(&save);

    application.set_accels_for_action("app.settings", &["F5"]);
    application.add_action(&settings);

    application.set_accels_for_action("app.about", &["F1"]);
    application.add_action(&about);

    application.run();
}










// T E S T I N G   Z O N E
// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
// 
// 
// const ANU_VALID_DATA_FORMAT: &'static [&'static str] = &[
//     "uint8", 
//     "uint16", 
//     "hex16",
// ];
// const ANU_TIMESTAMP_FILE: &str = "tmp/anu.timestamp";
// const ANU_QRNG_FILE: &str = "tmp/anu";
// const ANU_LOG_FILE: &str = "log/anu-session";
// const ANU_API_URL: &str = "qrng.anu.edu.au:80";
// const TCP_REQUEST_TIMEOUT_SECONDS: u64 = 60;
// const TCP_REQUEST_INTERVAL_SECONDS: i64 = 120;
// 
// 
// fn get_anu_response(anu_format: &str, array_length: u32, hex_block_size: u32) -> String {
//     println!("Connecting to ANU API...");
//     let mut socket_addr = ANU_API_URL
//         .to_socket_addrs()
//         .map_err(|e| format!("Socket address parsing error: {}", e))
//         .unwrap();
// 
//     let socket_addr = socket_addr
//         .next()
//         .ok_or("No socket addresses found for ANU API URL")
//         .unwrap();
// 
//     let mut stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(TCP_REQUEST_TIMEOUT_SECONDS))
//         .map_err(|e| format!("Connection error: {}", e))
//         .unwrap();
// 
//     let anu_request = format!(
//         "GET /API/jsonI.php?type={}&length={}&size={} HTTP/1.1\r\nHost: qrng.anu.edu.au\r\nConnection: close\r\n\r\n",
//             anu_format, 
//             array_length, 
//             hex_block_size
//     )
//     .into_bytes();
// 
//     stream.write_all(&anu_request)
//         .map_err(|e| format!("Write error: {}", e))
//         .unwrap();
// 
//     stream.flush()
//         .map_err(|e| format!("Flush error: {}", e))
//         .unwrap();
// 
//     let mut response = String::new();
//     let mut buffer = [0; 256];
//     let mut chunks = Vec::new(); // Store received chunks
// 
//     loop {
//         match stream.read(&mut buffer) {
//             Ok(bytes_read) if bytes_read > 0 => {
//                 let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);
//                 print!("{}", chunk);
//                 response.push_str(&chunk);
//                 chunks.push(chunk.to_string());
// 
//                 if chunk.ends_with("\r\n\r\n") {
//                     break; // End of response
//                 }
//             }
//             Ok(_) | Err(_) => break, // No more data or error occurred, break the loop
//         }
//     }
// 
//     println!("ANU API response done");
// 
//     // Combine chunks into single string
//     let combined_response = chunks.concat();
// 
//     // Create log
//     write_api_response_to_log(&combined_response, ANU_LOG_FILE);
// 
//     // Remove chunked encoding
//     let response = combined_response.replace("\r\n", "");
// 
//     response
// }
// 
// 
// 
// fn write_api_response_to_log(response: &str, log_file: &str) {
//     let current_time = SystemTime::now();
//     let timestamp = current_time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
//     let log_file = format!("{}-{}", log_file, timestamp);
// 
//     if let Some(parent) = Path::new(log_file.as_str()).parent() {
//         match fs::create_dir_all(parent) {
//             Ok(_) => {
//                 let mut file = match File::create(&log_file) {
//                     Ok(file) => file,
//                     Err(e) => {
//                         eprintln!("Error creating file: {}", e);
//                         return;
//                     }
//                 };
//                 if let Err(e) = file.write_all(response.as_bytes()) {
//                     eprintln!("Error writing to file: {}", e);
//                 }
//             }
//             Err(err) => {
//                 eprintln!("Error creating directory {}: {}", parent.display(), err);
//             }
//         }
//     }
// }
// 
// 
// 
// fn get_entropy_from_anu(
//     entropy_length: usize,
//     data_format: &str,
//     array_length: u32,
//     hex_block_size: Option<u32>
// ) -> Result<String, Box<dyn std::error::Error>> {
// 
//         let anu_data = fetch_anu_qrng_data(data_format, array_length, hex_block_size.unwrap());
//         let mut entropy_raw_binary: String = String::new();
// 
//         match data_format {
//             "uint8" => {
//                 let bytes = anu_data
//                     .split_whitespace()
//                     .map(|byte_str| byte_str.parse::<u8>().unwrap())
//                     .collect::<Vec<u8>>();
//              
//                 // Convert bytes to binary strings
//                 for byte in bytes {
//                     entropy_raw_binary.push_str(&format!("{:08b}", byte));
//                 }
//             },
//             "uint16" => {
//                 todo!()// Create parsing for uin16
//             },
//             "hex16" => {
//                 let hex_strings = extract_hex_strings(
//                     &anu_data, 
//                     hex_block_size.unwrap().try_into().unwrap()
//                 );
// 
//                 let mut anu_qrng_binary = String::new();
//  
//                 for hex_string in hex_strings {
//                     // println!("Hex string: {}", hex_string);
//                     let bytes = hex::decode(hex_string).expect("Failed to decode hex string");
//                     let binary_string: String = bytes.iter()
//                         .map(|byte| format!("{:08b}", byte))
//                         .collect();
// 
//                     // println!("Binary string: {:?}", binary_string);
//                     anu_qrng_binary.push_str(&binary_string);
//                 }
// 
//                 // Write all binary strings to a file
//                 let qrng_file = format!("{}.binary", ANU_QRNG_FILE);
//                 let mut file = File::create(&qrng_file)?;
//                 file.write_all(anu_qrng_binary.as_bytes())?;
// 
//                 if anu_qrng_binary.len() < entropy_length {
//                     return Err(format!(
//                         "Entropy string too short for requested entropy length: {}",
//                         entropy_length
//                     ).into());
//                 }
// 
//                 let max_start = anu_qrng_binary.len() - entropy_length;
//                 let start_point = rand::thread_rng().gen_range(0..=max_start);
// 
//                 entropy_raw_binary = anu_qrng_binary
//                     .chars()
//                     .skip(start_point)
//                     .take(entropy_length)
//                     .collect();
// 
//                 println!("Final entropy string: {}", entropy_raw_binary);
//             },
//             &_ => eprint!("Unknown ANU format")
//         }
// 
//     Ok(entropy_raw_binary)
// }
// 
// fn fetch_anu_qrng_data(anu_data: &str, array_length: u32, block_size: u32) -> String {
//     let current_time = SystemTime::now();
//     // println!("New request: {:?}", current_time);
// 
//     let last_request_time = load_last_anu_request().unwrap();
//     // println!("Last request: {:?}", last_request_time);
// 
//     let elapsed = current_time.duration_since(last_request_time).unwrap_or(Duration::from_secs(0));
//     let wait_duration = Duration::from_secs(TCP_REQUEST_INTERVAL_SECONDS as u64);
//     if elapsed < wait_duration {
//         let remaining_seconds = wait_duration.as_secs() - elapsed.as_secs();
//         eprintln!("One request per 2 minutes. You have to wait {} seconds more", remaining_seconds);
//         return String::new();
//     }
// 
//     let response = generate_anu_qrng(anu_data, array_length, block_size);
// 
//     if let Err(err) = create_anu_timestamp(current_time) {
//         eprintln!("Error saving last request time: {}", err);
//     }
// 
//     let file = format!("{}.{}", ANU_QRNG_FILE, anu_data);
// 
//     append_to_file(&file, &response)
//         .map_err(|e| format!("Error appending to file: {}", e))
//         .map(|_| response)
//         .unwrap_or_else(|e| {
//             eprintln!("Can not append to a file: {}", e);
//             String::new()
//         })
// }
// 
// 
// 
// fn generate_anu_qrng(anu_data: &str, array_length: u32, block_size: u32) -> String {
//     println!("Connecting to ANU API...");
//     let mut socket_addr = ANU_API_URL
//         .to_socket_addrs()
//         .map_err(|e| format!("Socket address parsing error: {}", e))
//         .unwrap();
// 
//     let socket_addr = socket_addr
//         .next()
//         .ok_or("No socket addresses found for ANU API URL")
//         .unwrap();
// 
//     let mut stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(TCP_REQUEST_TIMEOUT_SECONDS))
//         .map_err(|e| format!("Connection error: {}", e))
//         .unwrap();
// 
//     let anu_request = format!(
//         "GET /API/jsonI.php?type={}&length={}&size={} HTTP/1.1\r\nHost: qrng.anu.edu.au\r\nConnection: close\r\n\r\n",
//         anu_data, array_length, block_size
//     )
//     .into_bytes();
// 
//     stream.write_all(&anu_request)
//         .map_err(|e| format!("Write error: {}", e))
//         .unwrap();
// 
//     stream.flush()
//         .map_err(|e| format!("Flush error: {}", e))
//         .unwrap();
// 
//     let mut response = String::new();
//     let mut buffer = [0; 256];
//     let mut chunks = Vec::new(); // Store received chunks
// 
//     loop {
//         match stream.read(&mut buffer) {
//             Ok(bytes_read) if bytes_read > 0 => {
//                 let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);
//                 print!("{}", chunk);
//                 response.push_str(&chunk);
//                 chunks.push(chunk.to_string());
// 
//                 if chunk.ends_with("\r\n\r\n") {
//                     break; // End of response
//                 }
//             }
//             Ok(_) | Err(_) => break, // No more data or error occurred, break the loop
//         }
//     }
// 
//     println!("ANU API response done");
// 
//     // Combine chunks into single string
//     let combined_response = chunks.concat();
// 
//     // Create log
//     write_api_response_to_log(&combined_response, ANU_LOG_FILE);
// 
//     // Remove chunked encoding
//     let response = combined_response.replace("\r\n", "");
// 
//     response
// }
// 
// 
// 
// 
// 
// fn load_last_anu_request() -> Option<SystemTime> {
//     let path = Path::new(ANU_TIMESTAMP_FILE);
//     if path.exists() {
//         if let Ok(file) = File::open(path) {
//             let reader = BufReader::new(file);
//             if let Some(Ok(timestamp_str)) = reader.lines().next() {
//                 if let Ok(timestamp) = timestamp_str.trim().parse::<i64>() {
//                     return Some(SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp as u64));
//                 }
//             }
//         }
//     }
//     Some(SystemTime::UNIX_EPOCH)
// }
// 
// fn create_anu_timestamp(time: SystemTime) -> Result<(), io::Error> {
//     let timestamp = time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string();
// 
//     if let Some(parent) = Path::new(ANU_TIMESTAMP_FILE).parent() {
//         fs::create_dir_all(parent)?;
//     }
// 
//     let mut file = File::create(ANU_TIMESTAMP_FILE)?;
// 
//     file.write_all(timestamp.as_bytes())?;
// 
//     Ok(())
// }
// 
// fn append_to_file(file_path: &str, data: &str) -> std::io::Result<()> {
//     let mut file = OpenOptions::new()
//         .write(true)
//         .create(true)
//         .append(true)
//         .open(file_path)?;
// 
//     file.write_all(data.as_bytes())?;
// 
//     Ok(())
// }
// 
// fn extract_hex_strings(response: &str, hex_block_size: usize) -> Vec<String> {
//     let hex_block_size = hex_block_size * 2; // Adjust for byte format for ANU
//     let mut hex_strings = Vec::new();
//     let mut current_string = String::new();
//     let mut in_hex_string = false;
// 
//     for c in response.chars() {
//         if !in_hex_string {
//             if c == '"' {
//                 // Start of a potential hex string
//                 in_hex_string = true;
//                 current_string.clear();
//             }
//         } else {
//             if c == '"' {
//                 // End of hex string found, check if it's of expected length and contains valid hex characters
//                 if current_string.len() == hex_block_size && current_string.chars().all(|c| c.is_ascii_hexdigit()) {
//                     hex_strings.push(current_string.clone());
//                 }
//                 current_string.clear();
//                 in_hex_string = false;
//             } else if c == '\r' || c == '\n' || c == '\t' {
//                 // Ignore control characters within the hex string
//                 current_string.clear();
//                 in_hex_string = false;
//             } else {
//                 // Character is part of hex string, add to current string
//                 current_string.push(c);
//             }
//         }
//     }
// 
//     hex_strings
// }
// 
// 
// 
// // WORKING but uint8 can be done better
// // 

