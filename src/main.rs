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
    io::{self, BufRead, BufReader, Read, Write}, 
    net::{TcpStream,ToSocketAddrs}, 
    path::Path, 
    time::{Duration, SystemTime}
};
use hex;
use rand::Rng;
use sha2::{Digest, Sha256, Sha512};
use bip39;
use csv::ReaderBuilder;
use gtk4 as gtk;
use libadwaita as adw;
use adw::prelude::*;
use gtk::{gio, glib::clone, Stack, StackSidebar};
use qr2m_converters::{convert_binary_to_string, convert_string_to_binary};
use rust_i18n::t;


// Multi-language support
#[macro_use] extern crate rust_i18n;
i18n!("locale", fallback = "en");

// Default settings
const APP_DESCRIPTION: Option<&str> = option_env!("CARGO_PKG_DESCRIPTION");
const APP_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
const APP_AUTHOR: Option<&str> = option_env!("CARGO_PKG_AUTHORS");
const APP_LANGUAGE: &'static [&'static str] = &[
    "English", 
    "Deutsch",
    "Hrvatski",
];
const WORDLIST_FILE: &str = "lib/bip39-mnemonic-words-english.txt";
const COINLIST_FILE: &str = "lib/bip44-extended-coin-list.csv";
const LOG_FILE: &str = "log/wallet/qrng.log";
const LOG_OUTPUT: &'static [&'static str] = &[
    "Default", 
    "File",
    "None",
];
const VALID_ENTROPY_LENGTHS: [u32; 5] = [128, 160, 192, 224, 256];
const VALID_BIP_DERIVATIONS: [u32; 5] = [32, 44, 49, 84, 86];
const VALID_ENTROPY_SOURCES: &'static [&'static str] = &[
    "RNG", 
    "QRNG",
    "File",
];
const VALID_WALLET_PURPOSE: &'static [&'static str] = &[
    "Internal", 
    "External", 
];
const ANU_TIMESTAMP_FILE: &str = "tmp/anu.timestamp";
const ANU_LOG_DIRECTORY: &str = "log/anu/anu";
const ANU_API_URL: &str = "qrng.anu.edu.au:80";
const VALID_ANU_API_DATA_FORMAT: &'static [&'static str] = &[
    "uint8", 
    "uint16", 
    "hex16",
];
const ANU_DEFAULT_ARRAY_LENGTH: u32 = 1024;
const ANU_MINIMUM_ARRAY_LENGTH: u32 = 32;
const ANU_MAXIMUM_ARRAY_LENGTH: u32 = 1024;
const ANU_DEFAULT_HEX_BLOCK_SIZE: u32 = 16;
const TCP_REQUEST_TIMEOUT_SECONDS: u64 = 60;
const ANU_REQUEST_INTERVAL_SECONDS: i64 = 120;
const WINDOW_MAIN_DEFAULT_WIDTH: u32 = 1000;
const WINDOW_MAIN_DEFAULT_HEIGHT: u32 = 800;
const WINDOW_SETTINGS_DEFAULT_WIDTH: u32 = 700;
const WINDOW_SETTINGS_DEFAULT_HEIGHT: u32 = 500;
const VALID_PROXY_STATUS: &'static [&'static str] = &[
    "Off", 
    "Auto", 
    "Manual",
];
// TODO: Translate
const VALID_GUI_THEMES: &'static [&'static str] = &[
    "System", 
    "Light", 
    "Dark",
];



// BASIC -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

fn print_program_info() {
    let current_time = SystemTime::now();
    let timestamp = current_time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let sig = "-.-. --- .--. -.-- .-. .. --. .... - --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.";

    println!(" ██████╗ ██████╗ ██████╗ ███╗   ███╗");
    println!("██╔═══██╗██╔══██╗╚════██╗████╗ ████║");
    println!("██║   ██║██████╔╝ █████╔╝██╔████╔██║");
    println!("██║▄▄ ██║██╔══██╗██╔═══╝ ██║╚██╔╝██║");
    println!("╚██████╔╝██║  ██║███████╗██║ ╚═╝ ██║");
    println!(" ╚══▀▀═╝ ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝");

    fancy_print(Some(&format!("{} {}", &APP_DESCRIPTION.unwrap(), &APP_VERSION.unwrap())), None);
    fancy_print(Some(&format!("Start time {}", &timestamp.to_string())), None);
    fancy_print(Some(sig), None);

}

// fancy_print(Some("entropy_source"), Some(&format!("{:?}", source)));
// fancy_print(Some("ERROR"), Some(&t!("error.entropy.create.file")));
fn fancy_print(value: Option<&str>, msg: Option<&str>) {
    let settings = AppSettings::load_settings()
                .expect(&t!("error.settings.read"));

    let log_output = match settings.get_value("log_output") {
        Some(format) => format.parse::<String>().unwrap_or_else(|_| {
            fancy_print(Some("ERROR"), Some(&t!("error.settings.wrong", element = "log_output", value = "String")));
            String::from(*&LOG_OUTPUT[0])
        }),
        None => {
            fancy_print(Some("ERROR"), Some(&t!("error.settings.read", part = "log_output")));
            String::from(*&LOG_OUTPUT[0])
        }
    };

    let mut formatted_output = String::new();

    if let Some(value) = value {
        if let Some(msg) = msg {
            formatted_output.push_str(&format!("\t{} = {}", value, msg));
        } else {
            formatted_output.push_str(&format!("{}", value));
        }
    }

    match value.unwrap() {
        "ERROR" => {
            create_message_window(value.unwrap(), &msg.unwrap(), None, None);
        },
        _ => {}
    }
    match log_output.as_str() {
        "Default" => {
            println!("{}", formatted_output);
        },
        "File" => {
            println!("{}", formatted_output);

            if let Some(parent) = Path::new(LOG_FILE).parent() {
                match fs::create_dir_all(parent) {
                    Ok(_) => {
                        let mut file = match std::fs::OpenOptions::new().create(true).append(true).open(&LOG_FILE) {
                            Ok(file) => file,
                            Err(e) => {
                                eprintln!("Error creating file: {}", e);
                                return;
                            }
                        };
        
                        formatted_output.push_str("\n");
                        file.write_all(formatted_output.as_bytes())
                                .expect(&t!("error.file.write", value = &LOG_FILE).to_string());
                            
                    }
                    Err(err) => {
                        eprintln!("Error creating directory {}: {}", parent.display(), err);
                    }
                }
            }
        },
        _ => {},
    }
}

/// Generates entropy based on the specified source and length.
///
/// # Arguments
///
/// * `source` - A reference to a string specifying the entropy source. Supported values are:
///   * `"RNG"`: Generates entropy using the local random number generator.
///   * `"QRNG"`: Retrieves entropy from the ANU Quantum Random Number Generator.
///   * `"File"`: Reads entropy from a selected file.
///
/// * `entropy_length` - The length of the entropy to generate.
///
/// # Returns
///
/// A string containing the generated entropy.
///
/// # Examples
///
/// ```
/// let rng_entropy = generate_entropy("RNG", 256);
/// println!("Random entropy: {}", rng_entropy);
/// ```
fn generate_entropy(source: &str, entropy_length: u64) -> String {
    fancy_print(Some(&t!("log.generate_entropy").to_string()), None);

    fancy_print(Some("entropy_source"), Some(&format!("{:?}", source)));
    fancy_print(Some("entropy_length"), Some(&format!("{:?}", entropy_length)));

    match source {
        "RNG" => {
            let mut rng = rand::thread_rng();
            let rng_entropy_string: String = (0..entropy_length)
                .map(|_| rng.gen_range(0..=1))
                .map(|bit| char::from_digit(bit, 10).unwrap())
                .collect();

            ADDRESS_DATA.with(|data| {
                let mut data = data.borrow_mut();
                fancy_print(Some("entropy_initial"), Some(&format!("{:?}", rng_entropy_string)));
                data.entropy_string = Some(rng_entropy_string.clone());
            });

            rng_entropy_string
        },
        "QRNG" => {
            let settings = AppSettings::load_settings()
                .expect(&t!("error.settings.read"));

            let anu_format = match settings.get_value("anu_data_format") {
                Some(format) => format.parse::<String>().unwrap_or_else(|_| {
                    fancy_print(Some("ERROR"), Some(&t!("error.settings.wrong", element = "anu_data_format", value = "String")));
                    String::from("uint8")
                }),
                None => {
                    fancy_print(Some("ERROR"), Some(&t!("error.settings.read", value = "anu_data_format")));
                    String::from("uint8")
                }
            };
            
            let array_length = match settings.get_value("anu_array_length") {
                Some(array_length) => array_length.parse::<u32>().unwrap_or_else(|_| {
                    fancy_print(Some("ERROR"), Some(&t!("error.settings.wrong", element = "anu_array_length", value = "String")));
                    ANU_DEFAULT_ARRAY_LENGTH
                }),
                None => {
                    fancy_print(Some("ERROR"), Some(&t!("error.settings.read", value = "anu_array_length")));
                    ANU_DEFAULT_ARRAY_LENGTH
                }
            };
            
            let hex_block_size = match settings.get_value("anu_hex_block_size") {
                Some(hex_block_size) => hex_block_size.parse::<u32>().unwrap_or_else(|_| {
                    fancy_print(Some("ERROR"), Some(&t!("error.settings.wrong", element = "hex_block_size", value = "u32")));
                    ANU_DEFAULT_HEX_BLOCK_SIZE
                }),
                None => {
                    fancy_print(Some("ERROR"), Some(&t!("error.settings.read", value = "hex_block_size")));
                    ANU_DEFAULT_HEX_BLOCK_SIZE
                }
            };

            fancy_print(Some("anu_data_format"), Some(&format!("{:?}", anu_format)));
            fancy_print(Some("anu_array_length"), Some(&format!("{:?}", array_length)));
            fancy_print(Some("anu_hex_block_size"), Some(&format!("{:?}", hex_block_size)));
            
            let qrng_entropy_string = get_entropy_from_anu(
                entropy_length.try_into().unwrap(),
                &anu_format, 
                array_length, 
                Some(hex_block_size)
            );

            ADDRESS_DATA.with(|data| {
                let mut data = data.borrow_mut();
                fancy_print(Some("entropy_initial"), Some(&format!("{:?}", qrng_entropy_string)));
                data.entropy_string = Some(qrng_entropy_string.clone());
            });

            qrng_entropy_string
        },
        "File" => {
            let main_context = glib::MainContext::default();
            let main_loop = glib::MainLoop::new(Some(&main_context), false);
            let (tx, rx) = std::sync::mpsc::channel();
            
            let window = gtk::Window::new();

            let dialog = gtk::FileChooserDialog::new(
            Some(t!("UI.dialog.select").to_string()),
            Some(&window),
            gtk::FileChooserAction::Open,
            &[(&t!("UI.element.button.open").to_string(), gtk::ResponseType::Accept), (&t!("UI.element.button.cancel").to_string(), gtk::ResponseType::Cancel)],
            );

            let main_loop_clone = main_loop.clone();

            dialog.connect_response(move |dialog, response| {
                if response == gtk::ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let file_path = path.to_string_lossy().to_string();
                            fancy_print(Some("entropy_file_name"), Some(&format!("{:?}", file_path)));
                            
                            let file_entropy_string = generate_entropy_from_file(&file_path, entropy_length);
                            
                            if let Err(err) = tx.send(file_entropy_string) {
                                fancy_print(Some("ERROR"), Some(&t!("error.mpsc.send", value = err)));

                            } else {
                                main_loop.quit();
                            }
                        }
                    }
                }
                dialog.close();
            });
            
            dialog.show();
            main_loop_clone.run();
            
            match rx.recv() {
                Ok(received_file_entropy_string) => {
                    ADDRESS_DATA.with(|data| {
                        let mut data = data.borrow_mut();
                        data.entropy_string = Some(received_file_entropy_string.clone());
                    });

                    received_file_entropy_string
                },
                Err(_) => {
                    fancy_print(Some("ERROR"), Some(&t!("error.entropy.create.file")));
                    String::new()
                }
            }
        },
        _ => {
            fancy_print(Some("ERROR"), Some(&t!("error.entropy.create.source")));
            return String::new()
        }
    }
}

/// Generates a checksum for the provided entropy.
///
/// # Arguments
///
/// * `entropy` - The entropy for which the checksum is generated.
/// * `entropy_length` - The length of the entropy in bits.
///
/// # Returns
///
/// The generated checksum as a string.
///
/// # Examples
///
/// ```rust
/// let checksum = generate_entropy_checksum("0101010101", &10);
/// assert_eq!(checksum.len(), 1);
/// ```
fn generate_entropy_checksum(entropy: &str, entropy_length: &u32) -> String {
    fancy_print(Some(&t!("log.generate_entropy_checksum").to_string()), None);

    let entropy_binary = convert_string_to_binary(&entropy);
    fancy_print(Some("entropy_as_binary"), Some(&format!("{:?}", entropy_binary)));
    
    let hash_raw_binary: String = convert_binary_to_string(&Sha256::digest(&entropy_binary));
    fancy_print(Some("entropy_sha256_hash"), Some(&format!("{:?}", hash_raw_binary)));
    
    let checksum_length = entropy_length / 32;
    fancy_print(Some("entropy_checksum_length"), Some(&format!("{:?}", checksum_length)));
    
    let entropy_checksum: String = hash_raw_binary.chars().take(checksum_length.try_into().unwrap()).collect();
    fancy_print(Some("entropy_checksum"), Some(&format!("{:?}", entropy_checksum)));
    
    ADDRESS_DATA.with(|data| {
        let mut data = data.borrow_mut();
        data.entropy_checksum = Some(entropy_checksum.clone());
    });
    
    entropy_checksum
}

/// Generates mnemonic words from the final entropy binary.
///
/// # Arguments
///
/// * `final_entropy_binary` - The final entropy in binary format.
///
/// # Returns
///
/// The generated mnemonic words as a string.
///
/// # Examples
///
/// ```rust
/// let mnemonic = generate_mnemonic_words("01010101010101010101");
/// assert!(!mnemonic.is_empty());
/// ```
fn generate_mnemonic_words(final_entropy_binary: &str) -> String {
    fancy_print(Some(&t!("log.generate_mnemonic_words").to_string()), None);

    let chunks: Vec<String> = final_entropy_binary.chars()
        .collect::<Vec<char>>()
        .chunks(11)
        .map(|chunk| chunk.iter().collect())
        .collect();
    fancy_print(Some("entropy_final_chunks"), Some(&format!("{:?}", chunks)));


    let mnemonic_decimal: Vec<u32> = chunks.iter()
        .map(|chunk| u32::from_str_radix(chunk, 2).unwrap())
        .collect();
    fancy_print(Some("mnemonic_as_decimal"), Some(&format!("{:?}", mnemonic_decimal)));

    let mnemonic_file_content = match fs::read_to_string(WORDLIST_FILE) {
        Ok(content) => content,
        Err(err) => {
            fancy_print(Some("ERROR"), Some(&t!("error.wordlist.read", value = err)));
            return String::new();
        }
    };

    let bad_word = t!("error.wordlist.word").to_string();
    let mnemonic_words_vector: Vec<&str> = mnemonic_file_content.lines().collect();
    let mnemonic_words_vector: Vec<&str> = mnemonic_decimal.iter().map(|&decimal| {
        if (decimal as usize) < mnemonic_words_vector.len() {
            mnemonic_words_vector[decimal as usize]
        } else {
            &bad_word
        }
    }).collect();

    let mnemonic_words_as_string = mnemonic_words_vector.join(" ");

    ADDRESS_DATA.with(|data| {
        let mut data = data.borrow_mut();
        fancy_print(Some("mnemonic_words"), Some(&format!("{:?}", mnemonic_words_as_string)));
        data.mnemonic_words = Some(mnemonic_words_as_string.clone());
    });

    mnemonic_words_as_string
}

/// Generates a BIP39 seed from the provided entropy and passphrase.
///
/// # Arguments
///
/// * `entropy` - The entropy used for seed generation.
/// * `passphrase` - The passphrase used for seed generation.
///
/// # Returns
///
/// The generated BIP39 seed as a fixed-size array of bytes.
///
/// # Examples
///
/// ```rust
/// let seed = generate_bip39_seed("0101010101", "passphrase");
/// assert_eq!(seed.len(), 64);
/// ```
fn generate_bip39_seed(entropy: &str, passphrase: &str) -> [u8; 64] {
    fancy_print(Some(&t!("log.generate_bip39_seed").to_string()), None);

    let entropy_vector = convert_string_to_binary(&entropy);
    let mnemonic = match bip39::Mnemonic::from_entropy(&entropy_vector) {
        Ok(mnemonic) => mnemonic,
        Err(err) => {
            fancy_print(Some("ERROR"), Some(&t!("error.bip.mnemonic", error = err)));
            return [0; 64];
        },
    };
    let seed = bip39::Mnemonic::to_seed(&mnemonic, passphrase);
    let seed_hex = hex::encode(&seed[..]);
    
    ADDRESS_DATA.with(|data| {
        fancy_print(Some("seed_as_hex"), Some(&format!("{:?}", seed_hex)));
        let mut data = data.borrow_mut();
        data.seed = Some(seed_hex);
    });
    
    seed
}

/// Reads the contents of a file located at the specified path and generates entropy based on it.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the path to the file.
/// * `entropy_length` - An unsigned 64-bit integer specifying the length of entropy to be generated.
///
/// # Examples
///
/// ```
/// let entropy = generate_entropy_from_file("example.txt", 256);
/// println!("{}", entropy);
/// ```
fn generate_entropy_from_file(file_path: &str, entropy_length: u64) -> String {
    fancy_print(Some(&t!("log.generate_entropy_from_file").to_string()), None);

    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(err) => {
            fancy_print(Some("ERROR"), Some(&t!("error.file.open", value = file_path, error = err)));
            return String::new()
        },
    };
    
    let mut buffer = Vec::new();
    
    match file.read_to_end(&mut buffer) {
        Ok(_) => {},
        Err(err) => {
            fancy_print(Some("ERROR"), Some(&t!("error.file.read", value = file_path, error = err)));
        },
    };

    let hash = sha256_hash(&["qr2m".as_bytes(), &buffer].concat());
    fancy_print(Some("entropy_sha256_hash"), Some(&format!("{:?}", hash)));

    let mut entropy = String::new();
    for byte in hash {
        entropy.push_str(&format!("{:08b}", byte));
    }

    entropy = entropy.chars().take(entropy_length as usize).collect();

    fancy_print(Some("entropy_initial"), Some(&format!("{:?}", entropy)));
    entropy
}

/// Derives master private and public keys from a seed using the provided headers.
///
/// # Arguments
///
/// * `seed` - A hexadecimal string representing the seed.
/// * `private_header` - A hexadecimal string representing the private header.
/// * `public_header` - A hexadecimal string representing the public header.
///
/// # Returns
///
/// A tuple containing the derived master private and public keys as strings, or an error message string.
///
/// # Errors
///
/// Returns an error if parsing headers or creating keys fails.
///
/// # Example
///
/// ```
/// let seed = "12ab34cd56ef";
/// let private_header = "0x0488ADE4";
/// let public_header = "0x0488B21E";
/// match derive_master_keys(seed, private_header, public_header) {
///     Ok((master_xprv, master_xpub)) => {
///         println!("Master Private Key: {}", master_xprv);
///         println!("Master Public Key: {}", master_xpub);
///     },
///     Err(err) => println!("Error: {}", err),
/// }
/// ```
fn derive_master_keys(seed: &str, mut private_header: &str, mut public_header: &str) -> Result<(String, String), String> {
    fancy_print(Some(&t!("log.derive_master_keys").to_string()), None);

    // Reverting to Bitcoin in case that coin is undefined
    if private_header.is_empty() {
        private_header = "0x0488ADE4";
    }
    if public_header.is_empty() {
        public_header = "0x0488B21E";
    }

    // Default message for all blockchains ? Why ?
    let message = "Bitcoin seed";

    println!("Private header: {}", private_header);
    println!("Public header: {}", public_header);

    let private_header = u32::from_str_radix(private_header.trim_start_matches("0x"), 16)
        .expect(&t!("error.master.parse.header", value = "private").to_string());
    let public_header = u32::from_str_radix(public_header.trim_start_matches("0x"), 16)
        .expect(&t!("error.master.parse.header", value = "public").to_string());

    println!("Private header (parsed): {}", private_header);
    println!("Public header (parsed): {}", public_header);

    let seed_bytes = hex::decode(seed).expect(&t!("error.seed.decode").to_string());
    let hmac_result = hmac_sha512(message.as_bytes(), &seed_bytes);
    let (master_private_key_bytes, master_chain_code_bytes) = hmac_result.split_at(32);

    println!("Seed (bytes): {:?}", seed_bytes);
    println!("HMAC result: {:?}", hmac_result);
    println!("Master Private Key (bytes): {:?}", master_private_key_bytes);
    println!("Chain Code (bytes): {:?}", master_chain_code_bytes);

    // Private construct
    let mut master_private_key = Vec::new();

    master_private_key.extend_from_slice(&u32::to_be_bytes(private_header));                  // Version        4 bytes
    master_private_key.push(0x00);                                                                 // Depth          1 byte
    master_private_key.extend([0x00; 4].iter());                                                   // Parent finger  4 bytes
    master_private_key.extend([0x00; 4].iter());                                                   // Index/child    4 bytes
    master_private_key.extend_from_slice(master_chain_code_bytes);                                 // Chain code     32 bytes
    master_private_key.push(0x00);                                                                 // Key prefix     1 byte
    master_private_key.extend_from_slice(master_private_key_bytes);                                // Key            32 bytes

    println!("Master Private Key: {:?}", master_private_key);

    let checksum: [u8; 4] = calculate_checksum(&master_private_key);                         // Checksum       4 bytes
    master_private_key.extend_from_slice(&checksum);

    println!("Master Private Key (with checksum): {:?}", master_private_key);

    let master_xprv = bs58::encode(&master_private_key).into_string();              // Total      82 bytes

    println!("Master XPRV: {}", master_xprv);

    // Public construct
    let secp = secp256k1::Secp256k1::new();
    let master_secret_key = secp256k1::SecretKey::from_slice(&master_private_key_bytes)
        .expect(&t!("error.master.create").to_string());
    let master_public_key_bytes = secp256k1::PublicKey::from_secret_key(&secp, &master_secret_key).serialize();

    println!("Master Secret Key: {:?}", master_secret_key);
    println!("Master Public Key (bytes): {:?}", master_public_key_bytes);

    let mut master_public_key = Vec::new();

    master_public_key.extend_from_slice(&u32::to_be_bytes(public_header));                    // Version        4 bytes
    master_public_key.push(0x00);                                                                   // Depth          1 byte
    master_public_key.extend([0x00; 4].iter());                                                     // Parent finger  4 bytes
    master_public_key.extend([0x00; 4].iter());                                                     // Index/child    4 bytes
    master_public_key.extend_from_slice(master_chain_code_bytes);                                   // Chain code     32 bytes
    master_public_key.extend_from_slice(&master_public_key_bytes);                                  // Key            33 bytes (compressed)

    println!("Master Public Key: {:?}", master_public_key);

    let checksum: [u8; 4] = calculate_checksum(&master_public_key);                           // Checksum       4 bytes
    master_public_key.extend_from_slice(&checksum);

    println!("Master Public Key (with checksum): {:?}", master_public_key);

    let master_xpub = bs58::encode(&master_public_key).into_string();                // Total      82 bytes

    println!("Master XPUB: {}", master_xpub);

    ADDRESS_DATA.with(|data| {
        let mut data = data.borrow_mut();
        println!("Master Private key bytes: {:?}", &master_private_key_bytes);
        println!("Master Chain code bytes: {:?}", &master_chain_code_bytes);

        data.master_private_key_bytes = Some(master_private_key_bytes.to_vec());
        data.master_chain_code_bytes = Some(master_chain_code_bytes.to_vec());
    });

    Ok((master_xprv, master_xpub))
}

/// Computes the HMAC-SHA512 hash of the given key and data.
///
/// # Arguments
///
/// * `key` - A reference to a byte slice containing the key.
/// * `data` - A reference to a byte slice containing the data.
///
/// # Returns
///
/// A vector of bytes representing the HMAC-SHA512 hash.
///
/// # Example
///
/// ```
/// let key = b"secret";
/// let data = b"hello world";
/// let hmac = hmac_sha512(key, data);
/// println!("HMAC-SHA512 Hash: {:?}", hmac);
/// ```
fn hmac_sha512(key: &[u8], data: &[u8]) -> Vec<u8> {
    fancy_print(Some(&t!("log.hmac_sha512").to_string()), None);

    const BLOCK_SIZE: usize = 128;
    const HASH_SIZE: usize = 64;

    // Step 1: Create the padded key
    let padded_key = if key.len() > BLOCK_SIZE {
        println!("Key length is greater than BLOCK_SIZE. Hashing the key.");
        let mut hasher = Sha512::new();
        hasher.update(key);
        let mut hashed_key = vec![0u8; HASH_SIZE];
        hashed_key.copy_from_slice(&hasher.finalize());
        hashed_key.resize(BLOCK_SIZE, 0x00);
        println!("Hashed key: {:?}", hashed_key);
        hashed_key
    } else {
        println!("Key length ({}) is less than or equal to BLOCK_SIZE ({}). Padding the key.", key.len(), BLOCK_SIZE);
        let mut padded_key = vec![0x00; BLOCK_SIZE];
        padded_key[..key.len()].copy_from_slice(key);
        println!("Padded key: {:?}", padded_key);
        padded_key
    };

    // Verify the padded key length
    assert_eq!(padded_key.len(), BLOCK_SIZE, "Padded key length mismatch");

    // Step 2: Create inner and outer paddings
    let mut inner_pad = vec![0x36; BLOCK_SIZE];
    let mut outer_pad = vec![0x5c; BLOCK_SIZE];
    for (i, &b) in padded_key.iter().enumerate() {
        inner_pad[i] ^= b;
        outer_pad[i] ^= b;
    }

    println!("Inner padding (inner_pad): {:?}", inner_pad);
    println!("Outer padding (outer_pad): {:?}", outer_pad);

    // Step 3: Perform inner hash
    let mut hasher = Sha512::new();
    hasher.update(&inner_pad);
    hasher.update(data);
    let inner_hash = hasher.finalize();

    println!("Inner hash result: {:?}", inner_hash);

    // Step 4: Perform outer hash
    let mut hasher = Sha512::new();
    hasher.update(&outer_pad);
    hasher.update(&inner_hash);
    let final_hash = hasher.finalize().to_vec();

    // Verify the final hash length
    assert_eq!(final_hash.len(), HASH_SIZE, "Final hash length mismatch");

    println!("Final HMAC result: {:?}", final_hash);

    final_hash
}

/// Computes the SHA-256 hash of the given byte slice.
///
/// # Arguments
///
/// * `data` - A reference to a byte slice containing the data to be hashed.
///
/// # Returns
///
/// A vector of bytes representing the SHA-256 hash.
///
/// # Example
///
/// ```
/// let data = b"hello world";
/// let hash = sha256_hash(data);
/// println!("SHA-256 Hash: {:?}", hash);
/// ```
fn sha256_hash(data: &[u8]) -> Vec<u8> {
    fancy_print(Some(&t!("log.sha256_hash").to_string()), None);

    let mut hasher = Sha256::new();

    hasher.update(data);
    hasher.finalize().iter().cloned().collect()
}

/// Calculates the checksum of the given data using SHA-256.
///
/// # Arguments
///
/// * `data` - A reference to a byte slice containing the data.
///
/// # Returns
///
/// An array of 4 bytes representing the checksum.
///
/// # Example
///
/// ```
/// let data = b"hello world";
/// let checksum = calculate_checksum(data);
/// println!("Checksum: {:?}", checksum);
/// ```
fn calculate_checksum(data: &[u8]) -> [u8; 4] {
    fancy_print(Some(&t!("log.calculate_checksum").to_string()), None);
    
    let hash = Sha256::digest(data);
    let double_hash = Sha256::digest(&hash);
    let mut checksum = [0u8; 4];
    checksum.copy_from_slice(&double_hash[..4]);
    checksum
}



// COINS -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

/// This struct holds information about a particular coin.
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
    evm: String,
    comment: String,
}

/// Creates a vector of `CoinDatabase` from a CSV file.
///
/// # Returns
///
/// Returns a vector containing `CoinDatabase` entries read from the CSV file.
fn create_coin_store() -> Vec<CoinDatabase> {
    fancy_print(Some(&t!("log.create_coin_store").to_string()), None);

    let file = File::open(&COINLIST_FILE).expect("can not open bip44 coin file");
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
    let mut coin_store = Vec::new();

    for result in rdr.records() {
        let record = result
            .expect(&t!("error.csv.read").to_string());
        
        let index: u32 = record[0].parse()
            .expect(&t!("error.csv.parse", value = "index").to_string());
        
        let path: u32 = u32::from_str_radix(&record[1][2..], 16)
            .expect(&t!("error.csv.parse", value = "path").to_string());

        let symbol: String = if record[2].is_empty()            {"".to_string()} else {record[2].to_string()};
        let name: String = if record[3].is_empty()              {"".to_string()} else {record[3].to_string()};
        let key_derivation:String = if record[4].is_empty()     {"".to_string()} else {record[4].to_string()};
        let private_header: String = if record[5].is_empty()    {"".to_string()} else {record[5].to_string()};
        let public_header: String = if record[6].is_empty()     {"".to_string()} else {record[6].to_string()};
        let public_key_hash: String = if record[7].is_empty()   {"".to_string()} else {record[7].to_string()};
        let script_hash: String = if record[8].is_empty()       {"".to_string()} else {record[8].to_string()};
        let wif: String = if record[9].is_empty()               {"".to_string()} else {record[9].to_string()};
        let evm: String = if record[10].is_empty()              {"".to_string()} else {record[10].to_string()};
        let comment: String = if record[11].is_empty()          {"".to_string()} else {record[11].to_string()};
        
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
            evm,
            comment 
        };

        coin_store.push(coin_type);
    }

    coin_store
}

/// Creates a `gtk::ListStore` for displaying coin information in a GTK application.
///
/// This function populates the list store with coin information retrieved from the CSV file.
///
/// # Returns
///
/// Returns a `gtk::ListStore` containing coin information.
fn create_coin_completion_model() -> gtk::ListStore {
    fancy_print(Some(&t!("log.create_coin_completion_model").to_string()), None);

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
        glib::Type::STRING, // EVM
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
            (10, &coin_symbol.evm),
            (11, &coin_symbol.comment),
        ]);
    }

    store
}

/// Retrieves coins starting with the specified prefix from the coin store.
///
/// # Arguments
///
/// * `coin_store` - A reference to a vector of `CoinDatabase`.
/// * `target_prefix` - The prefix to match with coin symbols.
///
/// # Returns
///
/// Returns a vector containing references to `CoinDatabase` entries whose symbols start with the specified prefix.
fn get_coins_starting_with<'a>(coin_store: &'a Vec<CoinDatabase>, target_prefix: &'a str) -> Vec<&'a CoinDatabase> {
    coin_store
        .iter()
        .filter(|&coin_type| coin_type.symbol.starts_with(target_prefix))
        .collect()
}

/// Creates a vector of `CoinDatabase` from a CSV file.
///
/// # Arguments
///
/// * `file_path` - The path to the CSV file containing coin information.
///
/// # Returns
///
/// Returns a vector containing `CoinDatabase` entries read from the CSV file.
fn create_coin_database(file_path: &str) -> Vec<CoinDatabase> {
    fancy_print(Some(&t!("log.create_coin_database").to_string()), None);

    let file = File::open(&file_path)
        .expect(&t!("error.file.read", value = file_path).to_string());

    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

    let coin_types: Vec<CoinDatabase> = rdr
        .records()
        .filter_map(|record| record.ok())
        .enumerate()
        .map(|(index, record)| {
            
            let path: u32 = index as u32;
            let index: u32 = index.try_into()
                .expect(
                    &t!(
                        "error.converter.IO", 
                        input = "usize",
                        output="u32").to_string());

            let symbol: String = record.get(2).unwrap_or_default().to_string();
            let name: String = record.get(3).unwrap_or_default().to_string();
            let key_derivation: String = record.get(4).unwrap_or_default().to_string();
            let private_header: String = record.get(5).unwrap_or_default().to_string();
            let public_header: String = record.get(6).unwrap_or_default().to_string();
            let public_key_hash: String = record.get(7).unwrap_or_default().to_string();
            let script_hash: String = record.get(8).unwrap_or_default().to_string();
            let wif: String = record.get(9).unwrap_or_default().to_string();
            let evm: String = record.get(10).unwrap_or_default().to_string();
            let comment: String = record.get(11).unwrap_or_default().to_string();

            CoinDatabase {
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
                evm, 
                comment
            }
        }
        )
        .collect();

    coin_types
}



// GUI -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

/// Struct to hold application settings.
///
/// # Fields
///
/// * `entropy_source`: The source of entropy used for wallet generation.
/// * `entropy_length`: The length of entropy used for wallet generation.
/// * `bip`: The BIP (Bitcoin Improvement Proposal) version.
/// * `gui_save_window_size`: A flag indicating whether to save window size in GUI.
/// * `gui_last_width`: The last width of the GUI window.
/// * `gui_last_height`: The last height of the GUI window.
/// * `anu_enabled`: A flag indicating whether ANU (Australian National University) data is enabled.
/// * `anu_data_format`: The format of ANU data.
/// * `anu_array_length`: The length of the ANU array.
/// * `anu_hex_block_size`: The size of hex blocks in ANU data.
/// * `anu_log`: A flag indicating whether to log ANU data.
///
/// # Examples
///
/// ```
/// use std::io;
/// use std::fs;
/// use std::path::Path;
/// use toml;
///
/// struct AppSettings {
///     // fields...
/// }
///
/// impl AppSettings {
///     fn load_settings() -> io::Result<Self> {
///         // implementation...
///     }
///
///     fn get_value(&self, name: &str) -> Option<String> {
///         // implementation...
///     }
/// }
/// ```
struct AppSettings {
    wallet_entropy_source: String,
    wallet_entropy_length: u32,
    wallet_bip: u32,
    gui_save_window_size: bool,
    gui_window_width: u32,
    gui_window_height: u32,
    gui_window_maximized: bool,
    gui_theme: String,
    gui_language: String,
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
    log_output: String,
}

impl AppSettings {
    // FEATURE: create verify_settings function

    /// Loads application settings from a configuration file.
    ///
    /// The function reads settings from the specified configuration file. If the file
    /// doesn't exist, it copies settings from a default configuration file.
    ///
    /// # Errors
    ///
    /// Returns an error if there are any I/O errors or if the configuration file
    /// cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io;
    /// use my_app::AppSettings;
    ///
    /// fn main() -> io::Result<()> {
    ///     let settings = AppSettings::load_settings()?;
    ///     Ok(())
    /// }
    /// 
    /// ```
    fn load_settings() -> io::Result<Self> {
        // fancy_print(Some("Loading settings:"), None);

        // FEATURE: Create local ($HOME) settings
        let config_file = "config/custom.conf";
        let default_config_file = "config/default.conf";

        if !Path::new(config_file).exists() {
            fs::copy(default_config_file, config_file)?;
        }
        
        let config_str = match fs::read_to_string(config_file) {
            Ok(contents) => contents,
            Err(err) => {
                // IMPROVEMENT: ask if to load default config file
                // FEATURE: open dialog window, show visually error parameter
                eprintln!("{}: {}", &t!("error.file.read", value = config_file), err);
                String::new()
            }
        };
        
        // BUG: If one parameter has typo, whole AppSetting is empty ???
        let config: toml::Value = match config_str.parse() {
            Ok(value) => {
                value
            },
            Err(err) => {
                fancy_print(Some("ERROR"), Some(&t!("error.settings.config", error = err)));
                toml::Value::Table(toml::value::Table::new())
            }
        };

        // FEATURE: make a config's version compatibility check
        // let config_version = match config.get("version").and_then(|v| v.as_integer()) {
        //     Some(v) => v as u32,
        //     None => 0
        // };
        // println!("config_version: {}", config_version);

        let empty_value = toml::Value::String("".to_string());

        // GUI setting
        let gui_section = match config.get("gui") {
            Some(section) => section,
            None => &empty_value    // IMPROVEMENT: replace empty_value with default 'gui' values
        };
            
        let gui_save_window_size = gui_section.get("save_window_size")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let gui_window_width = gui_section.get("window_width")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(WINDOW_MAIN_DEFAULT_WIDTH);

        let gui_window_height = gui_section.get("window_height")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(WINDOW_MAIN_DEFAULT_HEIGHT);
    
        let gui_window_maximized = gui_section.get("window_maximized")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let gui_theme = gui_section.get("theme")
            .and_then(|v| v.as_str())
            .unwrap_or(*&VALID_GUI_THEMES[0])
            .to_string();

        let gui_language = gui_section.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or(*&APP_LANGUAGE[0])
            .to_string();

        // Wallet settings
        let wallet_section = match config.get("wallet") {
            Some(section) => section,
            None => &empty_value
        };

        let wallet_entropy_source = wallet_section.get("entropy_source")
            .and_then(|v| v.as_str())
            .unwrap_or(*&VALID_ENTROPY_SOURCES[0])
            .to_string();

        let wallet_entropy_length = wallet_section.get("entropy_length")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(*VALID_ENTROPY_LENGTHS.last().unwrap_or(&0));

        let wallet_bip = wallet_section.get("bip")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(*VALID_BIP_DERIVATIONS.get(1).unwrap_or(&VALID_BIP_DERIVATIONS[0]));

        
        // ANU settings
        let anu_section = match config.get("anu") {
            Some(section) => section,
            None => &empty_value
        };

        let anu_enabled = anu_section.get("anu_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let anu_data_format = anu_section.get("data_format")
            .and_then(|v| v.as_str())
            .unwrap_or(*&VALID_ANU_API_DATA_FORMAT[0])
            .to_string();

        let anu_array_length = anu_section.get("array_length")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(ANU_DEFAULT_ARRAY_LENGTH);
        
        let anu_hex_block_size = anu_section.get("hex_block_size")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(ANU_DEFAULT_HEX_BLOCK_SIZE);

        let anu_log = anu_section.get("anu_log")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);


        // Proxy settings
        let proxy_section = match config.get("proxy") {
            Some(section) => section,
            None => &empty_value
        };

        let proxy_status = proxy_section.get("proxy_status")
            .and_then(|v| v.as_str())
            .unwrap_or(*&VALID_PROXY_STATUS[0])
            .to_string();

        let proxy_server_address = proxy_section.get("proxy_server_address")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let proxy_server_port = proxy_section.get("proxy_server_port")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(8080);

        let proxy_use_pac: bool = proxy_section.get("proxy_use_pac")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let proxy_script_address = proxy_section.get("proxy_script_address")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let proxy_login_credentials: bool = proxy_section.get("proxy_login_credentials")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let proxy_login_username = proxy_section.get("proxy_login_username")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let proxy_login_password = proxy_section.get("proxy_login_password")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let proxy_use_ssl: bool = proxy_section.get("proxy_use_ssl")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let proxy_ssl_certificate = proxy_section.get("proxy_ssl_certificate")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();


        // Log settings
        let log_section = match config.get("log") {
            Some(section) => section,
            None => &empty_value
        };

        let log_output = log_section.get("log_output")
            .and_then(|v| v.as_str())
            .unwrap_or(*&LOG_OUTPUT[0])
            .to_string();


        Ok(AppSettings {
            wallet_entropy_source,
            wallet_entropy_length,
            wallet_bip,
            gui_save_window_size,
            gui_window_width,
            gui_window_height,
            gui_window_maximized,
            gui_theme,
            gui_language,
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
            log_output,
        })
    }

    /// Retrieves the value of a specific setting.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the setting to retrieve.
    ///
    /// # Returns
    ///
    /// Returns the value of the specified setting as a `String`, or `None` if the
    /// setting does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_app::AppSettings;
    ///
    /// let settings = AppSettings::load_settings().unwrap();
    /// let entropy_source = settings.get_value("entropy_source");
    /// ```
    fn get_value(&self, name: &str) -> Option<String> {
        match name {
            "wallet_entropy_source" => Some(self.wallet_entropy_source.clone()),
            "wallet_entropy_length" => Some(self.wallet_entropy_length.to_string()),
            "wallet_bip" => Some(self.wallet_bip.to_string()),

            "gui_save_window_size" => Some(self.gui_save_window_size.to_string()),
            "gui_last_width" => Some(self.gui_window_width.to_string()),
            "gui_last_height" => Some(self.gui_window_height.to_string()),
            "gui_window_maximized" => Some(self.gui_window_maximized.to_string()),
            "gui_theme" => Some(self.gui_theme.to_string()),
            "gui_language" => Some(self.gui_language.to_string()),

            "anu_enabled" => Some(self.anu_enabled.to_string()),
            "anu_data_format" => Some(self.anu_data_format.clone()),
            "anu_array_length" => Some(self.anu_array_length.to_string()),
            "anu_hex_block_size" => Some(self.anu_hex_block_size.to_string()),
            "anu_log" => Some(self.anu_log.to_string()),

            "proxy_status" => Some(self.proxy_status.clone()),
            "proxy_server_address" => Some(self.proxy_server_address.clone()),
            "proxy_server_port" => Some(self.proxy_server_port.to_string()),
            "proxy_script_address" => Some(self.proxy_script_address.clone()),
            "proxy_login_credentials" => Some(self.proxy_login_credentials.to_string()),
            "proxy_login_username" => Some(self.proxy_login_username.clone()),
            "proxy_login_password" => Some(self.proxy_login_password.clone()),
            "proxy_use_ssl" => Some(self.proxy_use_ssl.to_string()),
            "proxy_ssl_certificate" => Some(self.proxy_ssl_certificate.clone()),

            "log_output" => Some(self.log_output.clone()),
            _ => None,
        }
    }
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

/// Loads the contents of a file located at the specified path and returns them as a byte vector.
///
/// # Arguments
///
/// * `path` - The path to the file to be loaded.
///
/// # Returns
///
/// A vector containing the bytes of the file's contents.
///
/// # Errors
///
/// If the file cannot be opened or read, the function will panic with an error message indicating the failure.
///
/// # Examples
///
/// ```rust
/// let icon_bytes = load_icon_bytes("/path/to/icon.png");
/// assert!(!icon_bytes.is_empty());
/// ```
fn load_icon_bytes(path: &str) -> Vec<u8> {
    fancy_print(Some(&t!("log.load_icon_bytes").to_string()), Some(path));

    let mut file = std::fs::File::open(path).expect(&t!("error.file.open", value = path).to_string());
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect(&t!("error.file.read", value = path).to_string());
    buffer
}

/// Retrieves window theme icons based on the system theme color.
///
/// This function detects the system's current theme color (light or dark) using GTK settings
/// and retrieves corresponding icons for the application window.
///
/// # Returns
///
/// An array containing GTK images representing different window theme icons.
///
/// # Examples
///
/// ```rust
/// let window_icons = get_window_theme_icons();
/// // Use the retrieved icons to set up the application window.
/// ```
fn get_window_theme_icons() -> [gtk::Image; 5] {
    fancy_print(Some(&t!("log.get_window_theme_icons").to_string()), None);

    // IMPLEMENT: auto detect system theme color switch, change my icons also
    let settings = gtk::Settings::default().unwrap();
    let mut _theme_path: String = String::new();

    if settings.is_gtk_application_prefer_dark_theme() {
        _theme_path = "res/theme/basic/dark".to_string();
    } else {
        _theme_path = "res/theme/basic/light".to_string();
    }

    // BUG: SVG is not working on Windows, revert to PNG icons
    // IMPLEMENT: Check if svg can be loaded, if not, revert to png
    let default_image_extension = "png";
    
    let icon_new_wallet_bytes = load_icon_bytes(&format!("{}/new-wallet.{}", _theme_path, default_image_extension));
    let icon_open_wallet_bytes = load_icon_bytes(&format!("{}/open-wallet.{}", _theme_path, default_image_extension));
    let icon_save_wallet_bytes = load_icon_bytes(&format!("{}/save-wallet.{}", _theme_path, default_image_extension));
    let icon_about_bytes = load_icon_bytes(&format!("{}/about.{}", _theme_path, default_image_extension));
    let icon_settings_bytes = load_icon_bytes(&format!("{}/settings.{}", _theme_path, default_image_extension));
    
    let icon_new_wallet = gtk::Image::builder()
        .gicon(&gio::BytesIcon::new(&glib::Bytes::from(&icon_new_wallet_bytes)))
        .build();

    let icon_open_wallet = gtk::Image::builder()
        .gicon(&gio::BytesIcon::new(&glib::Bytes::from(&icon_open_wallet_bytes)))
        .build();
    
    let icon_save_wallet = gtk::Image::builder()
        .gicon(&gio::BytesIcon::new(&glib::Bytes::from(&icon_save_wallet_bytes)))
        .build();
    
    let icon_about = gtk::Image::builder()
        .gicon(&gio::BytesIcon::new(&glib::Bytes::from(&icon_about_bytes)))
        .build();
    
    let icon_settings = gtk::Image::builder()
        .gicon(&gio::BytesIcon::new(&glib::Bytes::from(&icon_settings_bytes)))
        .build();

    let images: [gtk::Image; 5] = [
        icon_new_wallet,
        icon_open_wallet,
        icon_save_wallet,
        icon_about,
        icon_settings,
    ];

    images
    
}

/// Creates the settings window.
///
/// This function initializes and displays the settings window with various sections
/// for different types of settings such as general, wallet, and ANU settings.
/// Users can modify the settings and save or cancel their changes.
///
/// # Examples
///
/// ```
/// use my_app::create_settings_window;
///
/// create_settings_window();
/// ```
fn create_settings_window() {
    fancy_print(Some(&t!("log.create_settings_window").to_string()), None);

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
    
    // Sidebar 1: General settings
    let general_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let general_settings_frame = gtk::Frame::new(Some(&t!("UI.settings.general").to_string()));
    let content_general_box = gtk::Box::new(gtk::Orientation::Vertical, 20);

    general_settings_box.set_margin_bottom(10);
    general_settings_box.set_margin_top(10);
    general_settings_box.set_margin_start(10);
    general_settings_box.set_margin_end(10);
    content_general_box.set_margin_start(20);
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
    let valid_gui_themes_as_strings: Vec<String> = APP_LANGUAGE.iter().map(|&x| x.to_string()).collect();
    let valid_gui_themes_as_str_refs: Vec<&str> = valid_gui_themes_as_strings.iter().map(|s| s.as_ref()).collect();
    let gui_theme_dropdown = gtk::DropDown::from_strings(&valid_gui_themes_as_str_refs);
    let default_gui_theme = valid_gui_themes_as_strings
        .iter()
        .position(|s| *s == settings.gui_theme) 
        .unwrap_or(0);

    gui_theme_dropdown.set_selected(default_gui_theme.try_into().unwrap());
    gui_theme_dropdown.set_size_request(200, 10);
    default_gui_language_box.set_hexpand(true);
    default_gui_language_item_box.set_hexpand(true);
    default_gui_language_item_box.set_margin_end(20);
    default_gui_language_item_box.set_halign(gtk::Align::End);
    
    default_gui_language_label_box.append(&default_gui_language_label);
    default_gui_language_item_box.append(&gui_theme_dropdown);
    default_gui_language_box.append(&default_gui_language_label_box);
    default_gui_language_box.append(&default_gui_language_item_box);
    content_general_box.append(&default_gui_language_box);

    // GUI: Save last window size
    let window_save_box = gtk::Box::new(gtk::Orientation::Horizontal, 50);
    let window_save_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let window_save_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let save_window_size_label = gtk::Label::new(Some(&t!("UI.settings.general.save_window").to_string()));
    let save_window_size_checkbox = gtk::CheckButton::new();
    let is_checked = settings.gui_save_window_size;

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

    stack.add_titled(
        &general_settings_box,
        Some("sidebar-settings-general"),
        &t!("UI.settings.sidebar.general").to_string()
    );
 
 
    // Sidebar 2: Wallet settings
    let wallet_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let wallet_settings_frame = gtk::Frame::new(Some(&t!("UI.settings.wallet").to_string()));
    let content_wallet_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    
    wallet_settings_box.set_margin_bottom(10);
    wallet_settings_box.set_margin_top(10);
    wallet_settings_box.set_margin_start(10);
    wallet_settings_box.set_margin_end(10);
    content_wallet_box.set_margin_start(20);
    wallet_settings_frame.set_hexpand(true);
    wallet_settings_frame.set_vexpand(true);
    wallet_settings_box.append(&wallet_settings_frame);
    wallet_settings_frame.set_child(Some(&content_wallet_box));

    // Default entropy source
    let default_entropy_source_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let default_entropy_source_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_entropy_source_item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let default_entropy_source_label = gtk::Label::new(Some(&t!("UI.settings.wallet.entropy.source").to_string()));
    let valid_entropy_source_as_strings: Vec<String> = VALID_ENTROPY_SOURCES.iter().map(|&x| x.to_string()).collect();
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

    stack.add_titled(
        &wallet_settings_box, 
        Some("sidebar-settings-wallet"), 
        &t!("UI.settings.sidebar.wallet").to_string()
    );


    // -.-. --- .--. -.-- .-. .. --. .... -
    // Sidebar 3: ANU settings
    // -.-. --- .--. -.-- .-. .. --. .... -
    let anu_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let anu_settings_frame = gtk::Frame::new(Some(&t!("UI.settings.anu").to_string()));
    let content_anu_box = gtk::Box::new(gtk::Orientation::Vertical, 20);

    anu_settings_box.set_margin_bottom(0);
    anu_settings_box.set_margin_top(10);
    anu_settings_box.set_margin_start(10);
    anu_settings_box.set_margin_end(10);
    content_anu_box.set_margin_start(20);
    content_anu_box.set_margin_top(10);
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
    
    // Min/Max check
    let mut default_array_length = settings.anu_array_length;
    default_array_length = std::cmp::max(ANU_MINIMUM_ARRAY_LENGTH, default_array_length);
    default_array_length = std::cmp::min(ANU_MAXIMUM_ARRAY_LENGTH, default_array_length);

    let array_length_adjustment = gtk::Adjustment::new(
        default_array_length as f64,        // initial value
        ANU_MINIMUM_ARRAY_LENGTH as f64,    // minimum value
        ANU_MAXIMUM_ARRAY_LENGTH as f64,    // maximum value
        1.0, // step increment
        10.0, // page increment
        0.0, // page size
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
        default_hex_size as f64,            // initial value
        1.0,                                // minimum value
        ANU_MAXIMUM_ARRAY_LENGTH as f64,    // maximum value
        1.0, // step increment
        10.0, // page increment
        0.0, // page size
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

    // Actions
    let default_anu_hex_length_box_clone = default_anu_hex_length_box.clone();
    let anu_data_format_dropdown_clone = anu_data_format_dropdown.clone();

    use_anu_api_checkbox.connect_toggled(move |checkbox| {
        if checkbox.is_active() {
            default_api_data_format_box.set_visible(true);
            default_anu_array_length_box.set_visible(true);
            if anu_data_format_dropdown_clone.selected() as usize == 2 {
                default_anu_hex_length_box_clone.set_visible(true);
            } else {
                default_anu_hex_length_box_clone.set_visible(false);
            }
        } else {
            default_api_data_format_box.set_visible(false);
            default_anu_array_length_box.set_visible(false);
            default_anu_hex_length_box_clone.set_visible(false);
        }
    });
    

    anu_data_format_dropdown.connect_selected_notify(clone!(
        @weak default_anu_hex_length_box,
        @weak anu_data_format_dropdown => move |dd| {
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
    // Sidebar 4: Proxy settings
    // -.-. --- .--. -.-- .-. .. --. .... -
    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_max_content_height(400);
    
    let proxy_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let proxy_settings_frame = gtk::Frame::new(Some(&t!("UI.settings.proxy").to_string()));
    let content_proxy_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    
    proxy_settings_box.set_margin_bottom(0);
    proxy_settings_box.set_margin_top(10);
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
    
    if settings.proxy_status == "manual" {
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
    proxy_server_port_entry.set_text(&settings.proxy_server_port.to_string());

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
    // IMPLEMENT: Translate tooltip to another languages
    // FEATURE: Make tooltip for every object
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
        @weak proxy_manual_settings_box => move |dd| {
            let value = dd.selected() as usize;
            let selected_proxy_settings_value = VALID_PROXY_STATUS.get(value);
            let settings = selected_proxy_settings_value.unwrap();
            
            if *settings == "manual" {
                proxy_manual_settings_box.set_visible(true);
            } else {
                proxy_manual_settings_box.set_visible(false);
            }
        }
    ));

    use_proxy_credentials_checkbox.connect_active_notify(clone!(
        @weak use_proxy_credentials_content_box => move |cb| {
            use_proxy_credentials_content_box.set_visible(cb.is_active());
        }
    ));

    use_proxy_pac_checkbox.connect_active_notify(clone!(
        @weak use_proxy_pac_content_box => move |cb| {
            use_proxy_pac_content_box.set_visible(cb.is_active());
        }
    ));

    use_proxy_ssl_checkbox.connect_active_notify(clone!(
        @weak use_proxy_ssl_checkbox => move |cb| {
            use_proxy_ssl_certificate_content_box.set_visible(cb.is_active());
        }
    ));

    // Compose settings window
    let main_settings_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let main_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    main_content_box.append(&stack_sidebar);
    main_content_box.append(&stack);
    
    // Buttons
    let buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let save_button = gtk::Button::with_label(&t!("UI.settings.button.save").to_string());
    let cancel_button = gtk::Button::with_label(&t!("UI.settings.button.cancel").to_string());
    // IMPLEMENT: apply button

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

/// Creates the settings window.
///
/// This function initializes and displays the settings window with various sections
/// for different types of settings such as general, wallet, and ANU settings.
/// Users can modify the settings and save or cancel their changes.
///
/// # Examples
///
/// ```
/// use my_app::create_settings_window;
///
/// create_settings_window();
/// ```
fn create_about_window() {
    fancy_print(Some(&t!("log.create_about_window").to_string()), None);

    let logo = gtk::gdk::Texture::from_file(&gio::File::for_path("lib/logo.png")).expect("msg");
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
        .comments(&t!("UI.about.description").to_string())
        .logo(&logo)
        .build();

    help_window.show();

}

fn update_derivation_label(DP: DerivationPath, label: gtk::Label, ) {
    fancy_print(Some(&t!("log.update_derivation_label").to_string()), None);

    // println!("New derivation_path: {:?}", DP);

    let mut path = String::new();

    if DP.bip.unwrap() == 32  {
        // BIP      m/32[']
        path.push_str(&format!("m/{}", DP.bip.unwrap_or_default()));
        if DP.hardened_bip.unwrap_or_default() {
            path.push_str(&format!("'"));
        }
        // COIN     m/32[']/0[']
        path.push_str(&format!("/{}", DP.coin.unwrap_or_default()));
        if DP.hardened_coin.unwrap_or_default() {
            path.push_str(&format!("'"));
        }
        // ADDRESS  m/32[']/0[']/0[']
        path.push_str(&format!("/{}", DP.address.unwrap_or_default()));
        if DP.hardened_address.unwrap_or_default() {
            path.push_str(&format!("'"));
        }
    } else {
        // BIP      m/!32[']
        path.push_str(&format!("m/{}", DP.bip.unwrap_or_default()));
        if DP.hardened_bip.unwrap_or_default() {
            path.push_str(&format!("'"));
        }
        // COIN     m/!32[']/0[']
        path.push_str(&format!("/{}", DP.coin.unwrap_or_default()));
        if DP.hardened_coin.unwrap_or_default() {
            path.push_str(&format!("'"));
        }
        // ADDRESS  m/!32[']/0[']/0[']
        path.push_str(&format!("/{}", DP.address.unwrap_or_default()));
        if DP.hardened_address.unwrap_or_default() {
            path.push_str(&format!("'"));
        }
        // PURPOSE  m/!32[']/0[']/0[']/[0,1]
        path.push_str(&format!("/{}", DP.purpose.unwrap_or_default()));

    }
    
    label.set_text(&path);
}

/// Creates the main application window.
///
/// This function initializes and configures the main application window, including its
/// dimensions, title, header bar, sidebar, and content area.
///
/// # Arguments
///
/// * `application` - The reference to the application instance.
///
/// # Examples
///
/// ```
/// use my_app::create_main_window;
///
/// let application = adw::Application::new(None, Default::default()).expect("Initialization failed");
/// create_main_window(&application);
/// ```
fn create_main_window(application: &adw::Application) {
    fancy_print(Some(&t!("log.create_main_window").to_string()), None);

    let settings = AppSettings::load_settings()
        .expect(&t!("error.file.read").to_string());
    
    let window_width = match settings.get_value("gui_last_width") {
        Some(width_str) => width_str.parse::<i32>().unwrap_or_else(|_| {
            eprintln!("{}", &t!("error.settings.parse", element = "gui_last_width", value = width_str));
            WINDOW_MAIN_DEFAULT_WIDTH.try_into().unwrap()
        }),
        None => {
            eprintln!("{}", &t!("error.settings.not_found", value = "gui_last_width"));
            WINDOW_MAIN_DEFAULT_WIDTH.try_into().unwrap()
        }
    };
    
    let window_height = match settings.get_value("gui_last_height") {
        Some(height_str) => height_str.parse::<i32>().unwrap_or_else(|_| {
            eprintln!("{}", &t!("error.settings.parse", value = "gui_last_height"));
            WINDOW_MAIN_DEFAULT_HEIGHT.try_into().unwrap()
        }),
        None => {
            eprintln!("{}", &t!("error.settings.not_found", value = "gui_last_height"));
            WINDOW_MAIN_DEFAULT_HEIGHT.try_into().unwrap()
        }
    };

    let preferred_theme = match settings.get_value("gui_theme") {
        Some(value) => {
            // let theme = String::from(value);
            // println!("theme {}", theme);

            match String::from(&value).as_str() {
                "System" => adw::ColorScheme::PreferLight,
                "Light" => adw::ColorScheme::ForceLight,
                "Dark" => adw::ColorScheme::PreferDark,
                _ => {
                    eprintln!("{}", &t!("error.settings.parse", element = "gui_theme", value = value));
                    adw::ColorScheme::PreferLight
                },
            }
        },
        None => {
            eprintln!("{}", &t!("error.settings.not_found", value = "gui_theme"));
            adw::ColorScheme::PreferLight
        }
        

    };

    // println!("preferred_theme: {:?}", preferred_theme);
    application.style_manager().set_color_scheme(preferred_theme);

    // MAIN WINDOW
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title(&format!("{} {}", APP_DESCRIPTION.unwrap(), APP_VERSION.unwrap()))
        .default_width(window_width)
        .default_height(window_height)
        .show_menubar(true)
        .decorated(true)
        .build();

    // Main menu (HeaderBar)
    let header_bar = gtk::HeaderBar::new();
    window.set_titlebar(Some(&header_bar));
    
    // HeaderBar buttons
    let new_wallet_button = gtk::Button::new();
    let open_wallet_button = gtk::Button::new();
    let save_wallet_button = gtk::Button::new();
    let about_button = gtk::Button::new();
    let settings_button = gtk::Button::new();

    // HeaderBar Icons
    // FEATURE: make my own menu icons
    let theme_images = get_window_theme_icons();

    new_wallet_button.set_icon_name("tab-new-symbolic");
    new_wallet_button.set_child(Some(&theme_images[0]));
    open_wallet_button.set_icon_name("document-open-symbolic");
    open_wallet_button.set_child(Some(&theme_images[1]));
    save_wallet_button.set_icon_name("document-save-symbolic");
    save_wallet_button.set_child(Some(&theme_images[2]));
    about_button.set_icon_name("help-about-symbolic");
    about_button.set_child(Some(&theme_images[3]));
    settings_button.set_icon_name("org.gnome.Settings-symbolic");
    settings_button.set_child(Some(&theme_images[4]));
    
    // HeaderBar Tooltips
    new_wallet_button.set_tooltip_text(Some(&t!("UI.main.headerbar.wallet.new", value = "Ctrl+N").to_string()));
    open_wallet_button.set_tooltip_text(Some(&t!("UI.main.headerbar.wallet.open", value = "Ctrl+O").to_string()));
    save_wallet_button.set_tooltip_text(Some(&t!("UI.main.headerbar.wallet.save", value = "Ctrl+S").to_string()));
    about_button.set_tooltip_text(Some(&t!("UI.main.headerbar.about", value = "F1").to_string()));
    settings_button.set_tooltip_text(Some(&t!("UI.main.headerbar.settings", value = "F5").to_string()));

    // Connections
    header_bar.pack_start(&new_wallet_button);
    header_bar.pack_start(&open_wallet_button);
    header_bar.pack_start(&save_wallet_button);
    header_bar.pack_end(&settings_button);
    header_bar.pack_end(&about_button);

    // Actions
    settings_button.connect_clicked(move |_| {
        create_settings_window();
    });

    // open_wallet_button.connect_clicked(move |_| {
    //     create_message_window("msg", None, None);
    // });

    about_button.connect_clicked(move |_| {
        create_about_window();
    });

    // New wallet (window) CTRL+N
    let new_window = application.clone();
    new_wallet_button.connect_clicked(move |_| {
        create_main_window(&new_window);
    });

    // Main stack
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
    let entropy_source_frame = gtk::Frame::new(Some(&t!("UI.main.seed.entropy.source").to_string()));
    let valid_entropy_source_as_strings: Vec<String> = VALID_ENTROPY_SOURCES.iter().map(|&x| x.to_string()).collect();
    let valid_entropy_source_as_str_refs: Vec<&str> = valid_entropy_source_as_strings.iter().map(|s| s.as_ref()).collect();
    let entropy_source_dropdown = gtk::DropDown::from_strings(&valid_entropy_source_as_str_refs);
    let default_entropy_source = valid_entropy_source_as_strings
        .iter()
        .position(|s| *s == settings.wallet_entropy_source) 
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
    let default_entropy_length = valid_entropy_lengths_as_strings
        .iter()
        .position(|x| x.parse::<u32>().unwrap() == settings.wallet_entropy_length)
        .unwrap_or(0);

    entropy_length_dropdown.set_selected(default_entropy_length.try_into().unwrap());
    entropy_length_box.set_hexpand(true);
    entropy_length_frame.set_hexpand(true);

    // Mnemonic passphrase
    let mnemonic_passphrase_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let mnemonic_passphrase_frame = gtk::Frame::new(Some(&t!("UI.main.seed.mnemonic.pass").to_string()));
    let mnemonic_passphrase_text = gtk::Entry::new();
    mnemonic_passphrase_box.set_hexpand(true);
    mnemonic_passphrase_text.set_hexpand(true);
    
    // Generate seed button
    let generate_seed_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let generate_seed_button = gtk::Button::new();
    generate_seed_button.set_label(&t!("UI.main.seed.generate").to_string());
    generate_seed_box.set_halign(gtk::Align::Center);
    generate_seed_box.set_margin_top(10);

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
    generate_seed_box.append(&generate_seed_button);
    entropy_source_box.append(&entropy_source_frame);
    entropy_length_box.append(&entropy_length_frame);
    entropy_header_first_box.append(&entropy_source_box);
    entropy_header_first_box.append(&entropy_length_box);
    entropy_header_second_box.append(&mnemonic_passphrase_box);
    entropy_header_box.append(&entropy_header_first_box);
    entropy_header_box.append(&entropy_header_second_box);
    entropy_header_box.append(&generate_seed_box);
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
    
    // Start Seed sidebar
    stack.add_titled(
        &entropy_main_box,
        Some("sidebar-seed"), 
        &t!("UI.main.seed").to_string());


    // -.-. --- .--. -.-- .-. .. --. .... -
    // Sidebar 2: Coin
    // -.-. --- .--. -.-- .-. .. --. .... -
    let coin_main_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let coin_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let coin_frame = gtk::Frame::new(Some(&t!("UI.main.coin").to_string()));
    coin_main_box.set_margin_top(10);
    coin_main_box.set_margin_start(10);
    coin_main_box.set_margin_end(10);
    coin_main_box.set_margin_bottom(10);

    // Create scrolled window
    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_max_content_height(400);

    // Coin treeview
    create_coin_completion_model();
    let coin_store = create_coin_store();
    let tree_store = gtk4::TreeStore::new(&[glib::Type::STRING; 12]);
    let coins = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let coin_treeview = gtk::TreeView::new();
    coin_treeview.set_vexpand(true);
    coin_treeview.set_headers_visible(true);

    let columns = [
        &t!("UI.main.database.column.index").to_string(),
        &t!("UI.main.database.column.path").to_string(),
        &t!("UI.main.database.column.symbol").to_string(),
        &t!("UI.main.database.column.name").to_string(),
        &t!("UI.main.database.column.deriv").to_string(),
        &t!("UI.main.database.column.priv_header").to_string(),
        &t!("UI.main.database.column.pub_header").to_string(),
        &t!("UI.main.database.column.pub_hash").to_string(),
        &t!("UI.main.database.column.script").to_string(),
        &t!("UI.main.database.column.wif").to_string(),
        &t!("UI.main.database.column.evm").to_string(),
        &t!("UI.main.database.column.comment").to_string(),
    ];

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
    coin_search.set_placeholder_text(Some(&t!("UI.main.coin.search").to_string()));

    // Generate master keys button
    let generate_master_keys_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let generate_master_keys_button = gtk::Button::new();
    generate_master_keys_button.set_label(&t!("UI.main.coin.generate").to_string());
    generate_master_keys_box.set_halign(gtk::Align::Center);
    generate_master_keys_box.append(&generate_master_keys_button);

    // Master private keys
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

    // Connections
    coins.append(&coin_search);
    scrolled_window.set_child(Some(&coin_treeview));
    coins.append(&scrolled_window);
    coin_frame.set_child(Some(&coins));
    coin_box.append(&coin_frame);
    master_xprv_frame.set_child(Some(&master_private_key_text));
    master_xpub_frame.set_child(Some(&master_public_key_text));
    master_keys_box.append(&master_xprv_frame);
    master_keys_box.append(&master_xpub_frame);
    coin_main_box.append(&coin_box);
    coin_main_box.append(&generate_master_keys_box);
    coin_main_box.append(&master_keys_box);
    
    stack.add_titled(
        &coin_main_box, 
        Some("sidebar-coin"), 
        &t!("UI.main.coin").to_string()
    );


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

    // Derivation labels
    // TODO: Show derivation boxes according to BIP number
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
    
    let bip_number = match settings.get_value("wallet_bip") {
        Some(bip_number) => {
            // TODO: parsed_bip_number can not be any u32 number. Make extra check of make new function: verify_settings function
            let parsed_bip_number = bip_number.parse::<u32>().unwrap_or_else(|_| {
                eprintln!("{}", &t!("error.settings.parse", element = "default BIP number", value = bip_number));
                44  // Default BIP44
            });
            
            let default_index = VALID_BIP_DERIVATIONS.iter().position(|&x| x == parsed_bip_number).unwrap_or_else(|| {
                eprintln!("{}", &t!("error.bip.value", value = parsed_bip_number));
                1 // BIP44
            });

            bip_dropdown.set_selected(default_index.try_into().unwrap());
            parsed_bip_number
        },
        None => {
            eprintln!("{}", &t!("error.settings.not_found", value = "bip"));
            
            let default_bip_number = 44;
            let default_index = VALID_BIP_DERIVATIONS.iter().position(|&x| x == default_bip_number).unwrap_or_else(|| {
                eprintln!("{}", &t!("error.bip.value", value = default_bip_number));
                1 // BIP44
            });

            bip_dropdown.set_selected(default_index.try_into().unwrap());
            default_bip_number
        }
    };

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
        0.0, // initial value
        0.0, // minimum value
        2147483647.0, // maximum value
        1.0, // step increment
        100.0, // page increment
        0.0, // page size
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

    let derivation_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let derivation_label_frame = gtk::Frame::new(Some(&t!("UI.main.address.derivation").to_string()));
    derivation_label_frame.set_hexpand(true);
    
    let default_bip_label = if bip_number == 32 {
        main_purpose_frame.set_visible(false);
        format!("m/{}'/0'/0'", bip_number)
    } else {
        main_purpose_frame.set_visible(true);
        format!("m/{}'/0'/0'/0", bip_number)
    };
    
    let derivation_label_text = gtk4::Label::builder()
        .label(&default_bip_label)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .css_classes(["large-title"])
        .build();

    let generate_addresses_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let generate_addresses_button = gtk::Button::with_label(&t!("UI.main.address.generate").to_string());

    generate_addresses_button_box.append(&generate_addresses_button);
    generate_addresses_button_box.set_halign(gtk::Align::Center);

    let address_treeview_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let address_treeview_frame = gtk::Frame::new(Some(&t!("UI.main.address").to_string()));
    address_treeview_frame.set_hexpand(true);
    address_treeview_frame.set_vexpand(true);

    let address_treeview = gtk::TreeView::new();
    address_treeview.set_headers_visible(true);
    let columns = [
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
    address_treeview_frame.set_child(Some(&address_treeview));
    address_treeview_box.append(&address_treeview_frame);
    main_address_box.append(&derivation_box);
    main_address_box.append(&derivation_label_box);
    main_address_box.append(&generate_addresses_button_box);
    main_address_box.append(&address_treeview_box);
    
    stack.add_titled(
        &main_address_box,
        Some("sidebar-address"), 
        &t!("UI.main.address").to_string()
    );


    // ACTIONS
    // JUMP: Generate Seed button
    generate_seed_button.connect_clicked(clone!(
        @weak entropy_source_dropdown,
        @weak entropy_length_dropdown,
        @weak mnemonic_words_text,
        @weak seed_text,
        @weak stack  => move |_| {
            let selected_entropy_source_index = entropy_source_dropdown.selected() as usize;
            let selected_entropy_length_index = entropy_length_dropdown.selected() as usize;
            let selected_entropy_source_value = VALID_ENTROPY_SOURCES.get(selected_entropy_source_index);
            let selected_entropy_length_value = VALID_ENTROPY_LENGTHS.get(selected_entropy_length_index);
            let source = selected_entropy_source_value.unwrap().to_string();
            let length = selected_entropy_length_value.unwrap();
            
            entropy_text.buffer().set_text("");
            mnemonic_words_text.buffer().set_text("");
            seed_text.buffer().set_text("");

            let entropy_length = selected_entropy_length_value;
            
            let pre_entropy = generate_entropy(
                &source,
                *length as u64,
            );
            
            if !pre_entropy.is_empty() {
                let checksum = generate_entropy_checksum(&pre_entropy, entropy_length.unwrap());
                let full_entropy = format!("{}{}", &pre_entropy, &checksum);

                fancy_print(Some("entropy_final"), Some(&format!("{:?}", full_entropy)));
                entropy_text.buffer().set_text(&full_entropy);
                
                let mnemonic_words = generate_mnemonic_words(&full_entropy);
                mnemonic_words_text.buffer().set_text(&mnemonic_words);
                
                let passphrase_text = mnemonic_passphrase_text.text().to_string();
                fancy_print(Some("mnemonic_passphrase"), Some(&format!("{:?}", passphrase_text)));
                
                let seed = generate_bip39_seed(&pre_entropy, &passphrase_text);
                let seed_hex = hex::encode(&seed[..]);
                seed_text.buffer().set_text(&seed_hex.to_string());
            } else {
                // TODO: If entropy is empty show error dialog
                eprintln!("{}", &t!("error.entropy.empty"))
            }
        }
    ));

    let coin_treeview_clone = coin_treeview.clone();
    let master_seed_text_clone = seed_text.clone();

    // JUMP: Generate Master Keys button
    generate_master_keys_button.connect_clicked(clone!(
        @strong coin_entry,
        @weak stack   => move |_| {
            // TODO: Check if seed is empty, show error dialog
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
                let evm = model.get_value(&iter, 10);
                let comment = model.get_value(&iter, 11);

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
                    Ok(evm),
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
                    evm.get::<String>(),
                    comment.get::<String>(),
                ) {
                    println!("\n#### Coin info ####");

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
                    println!("EVM: {}", evm);
                    println!("comment: {}", comment);
                    let buffer = master_seed_text_clone.buffer();
                    let start_iter = buffer.start_iter();
                    let end_iter = buffer.end_iter();
                    let seed_string = buffer.text(&start_iter, &end_iter, true);
                    

        


                    match derive_master_keys(
                        &seed_string, 
                        &private_header,
                        &public_header,
                    ) {
                        Ok(xprv) => {
                            master_private_key_text.buffer().set_text(&xprv.0);
                            master_public_key_text.buffer().set_text(&xprv.1);
                        },
                        Err(err) => eprintln!("{}: {}", &t!("error.master.create"), err),
                    }

                    coin_entry.set_text(&coin_type);
                }  
            }
        }
    ));

    coin_search.connect_search_changed(move|coin_search| {
        let search_text = coin_search.text().to_uppercase();
        tree_store.clear();
    
        if search_text.len() >= 2 {
            let matching_coins = get_coins_starting_with(&coin_store, &search_text);
            
            if !matching_coins.is_empty() {
                tree_store.clear();
                
                for found_coin in matching_coins {
                    let iter = tree_store.append(None);
                    tree_store.set(&iter, &[
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
                        (10, &found_coin.evm),
                        (11, &found_coin.comment),
                    ]);
                }
                coin_treeview.set_model(Some(&tree_store));
            } else {
                tree_store.clear();
            }
        } else {
            tree_store.clear();
        }
    });
    
    
    let derivation_path = std::rc::Rc::new(std::cell::RefCell::new(DerivationPath::default()));
    let dp_clone = std::rc::Rc::clone(&derivation_path);

    bip_dropdown.connect_selected_notify(clone!(
        @weak derivation_label_text,
        @weak bip_dropdown => move |_| {
            let value = bip_dropdown.selected() as usize;
            let selected_entropy_source_value = VALID_BIP_DERIVATIONS.get(value);
            let bip = selected_entropy_source_value.unwrap();
    
            if *bip == 32 {
                main_purpose_frame.set_visible(false);
            } else {
                main_purpose_frame.set_visible(true);
            }
    
            dp_clone.borrow_mut().update_field("bip", Some(FieldValue::U32(*bip)));
            // println!("new DP: {:?}", dp_clone.borrow());
            update_derivation_label(*dp_clone.borrow(), derivation_label_text)
        }
    ));
        
    let dp_clone = std::rc::Rc::clone(&derivation_path);
    
    bip_hardened_checkbox.connect_active_notify(clone!(
        @weak derivation_label_text,
        @weak bip_hardened_checkbox => move |_| {
            dp_clone.borrow_mut().update_field("hardened_bip", Some(FieldValue::Bool(bip_hardened_checkbox.is_active())));
            // println!("new DP: {:?}", dp_clone.borrow());
            update_derivation_label(*dp_clone.borrow(), derivation_label_text)
        }
    ));
        
    let dp_clone2 = std::rc::Rc::clone(&derivation_path);
    
    coin_hardened_checkbox.connect_active_notify(clone!(
        @weak derivation_label_text,
        @weak coin_hardened_checkbox => move |_| {
            dp_clone2.borrow_mut().update_field("hardened_coin", Some(FieldValue::Bool(coin_hardened_checkbox.is_active())));
            // println!("new DP: {:?}", dp_clone2.borrow());
            update_derivation_label(*dp_clone2.borrow(), derivation_label_text)
        }
    ));

    let dp_clone3 = std::rc::Rc::clone(&derivation_path);
    
    address_hardened_checkbox.connect_active_notify(clone!(
        @weak derivation_label_text,
        @weak address_hardened_checkbox => move |_| {
            dp_clone3.borrow_mut().update_field("hardened_address", Some(FieldValue::Bool(address_hardened_checkbox.is_active())));
            // println!("new DP: {:?}", dp_clone3.borrow());
            update_derivation_label(*dp_clone3.borrow(), derivation_label_text)
        }
    ));
        
    let dp_clone4 = std::rc::Rc::clone(&derivation_path);
    
    purpose_dropdown.connect_selected_notify(clone!(
        @weak derivation_label_text,
        @weak purpose_dropdown => move |_| {
            let purpose = purpose_dropdown.selected();

            dp_clone4.borrow_mut().update_field("purpose", Some(FieldValue::U32(purpose)));
            // println!("new Purpose: {:?}", dp_clone4.borrow());
            update_derivation_label(*dp_clone4.borrow(), derivation_label_text);
        }
    ));

    let dp_clone5 = std::rc::Rc::clone(&derivation_path);

    coin_entry.connect_changed(clone!(
        @weak derivation_label_text,
        @strong coin_entry => move |_| {
            let coin_number = coin_entry.buffer().text();
            let ff = coin_number.as_str();
            let my_int = ff.parse::<u32>();

            if my_int.is_ok() {
                dp_clone5.borrow_mut().update_field("coin", Some(FieldValue::U32(my_int.unwrap())));
                // println!("new Coin: {:?}", dp_clone5.borrow());
                update_derivation_label(*dp_clone5.borrow(), derivation_label_text);
            }
        }
    ));

    let dp_clone6 = std::rc::Rc::clone(&derivation_path);

    address_spinbutton.connect_changed(clone!(
        @weak derivation_label_text,
        @weak address_spinbutton => move |_| {
            let address_number = address_spinbutton.text();
            let ff = address_number.as_str();
            let my_int = ff.parse::<u32>();

            if my_int.is_ok() {
                dp_clone6.borrow_mut().update_field("address", Some(FieldValue::U32(my_int.unwrap())));
                // println!("new Address: {:?}", dp_clone6.borrow());
                update_derivation_label(*dp_clone6.borrow(), derivation_label_text);
            }
        }
    ));

    // JUMP: Generate Addresses button
    generate_addresses_button.connect_clicked(move |_| {
        println!("\n#### Generating addresses button ####");
    
        let master_private_key_bytes = ADDRESS_DATA.with(|data| {
            let data = data.borrow();
            // BUG: called `Option::unwrap()` on a `None` value
            data.master_private_key_bytes.clone().unwrap_or_default()
        });
        println!("Received Master private bytes: {:?}", master_private_key_bytes);


        let master_chain_code_bytes = ADDRESS_DATA.with(|data| {
            let data = data.borrow();
            data.master_chain_code_bytes.clone().unwrap_or_default()
        });
        println!("Received Master chain code bytes: {:?}", master_chain_code_bytes);


        let path = "m/0'/0";

        let derived = derive_from_path(&master_private_key_bytes, &master_chain_code_bytes, path).expect("Failed to derive key from path");
        let secp = secp256k1::Secp256k1::new();
        let public_key = secp256k1::PublicKey::from_secret_key(&secp, &derived.0);
        let address = generate_address(&public_key);
        println!("Generated address: {}", address);





        // Bitcoin crate
        let master_private_key = bitcoin::PrivateKey::from_slice(&derived.0.secret_bytes(), bitcoin::Network::Bitcoin).unwrap();
        let master_public_key = bitcoin::PublicKey::from_private_key(&secp, &master_private_key);
        let bitcoin_address = bitcoin::Address::p2pkh(master_public_key.pubkey_hash(), bitcoin::Network::Bitcoin);
        println!("Generated Bitcoin address: {}", bitcoin_address);

    });
    
    
    // Main sidebar
    let main_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    main_content_box.append(&stack_sidebar);
    main_content_box.append(&stack);
    window.set_child(Some(&main_content_box));

    window.present();
}

fn create_message_window(title: &str, msg: &str, progress_active: Option<bool>, wait_time: Option<u32>) {
    fancy_print(Some(&t!("log.create_message_window").to_string()), None);
        
    let message_window = gtk::MessageDialog::builder()
        .title(title)
        .resizable(false)
        .modal(true)
        .build();

    let dialog_main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    dialog_main_box.set_margin_bottom(20);
    dialog_main_box.set_margin_top(20);
    dialog_main_box.set_margin_start(50);
    dialog_main_box.set_margin_end(50);
    
    let message_label_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    message_label_box.set_margin_bottom(10);
    
    let message_label = gtk::Label::new(Some(&msg));
    message_label.set_justify(gtk::Justification::Center);
    
    message_label_box.append(&message_label);
    dialog_main_box.append(&message_label_box);
    
    // Progress box
    if progress_active.unwrap_or(false) {
        let progress_main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        progress_main_box.set_margin_top(10);
        progress_main_box.set_margin_bottom(10);

        let level_bar = gtk::LevelBar::new();
        level_bar.set_max_value(100.0);

        progress_main_box.append(&level_bar);
        dialog_main_box.append(&progress_main_box);

        let wait_time = wait_time.unwrap_or(10).min(ANU_REQUEST_INTERVAL_SECONDS as u32);
        let level_bar_clone = level_bar.clone();
        let message_window_clone = message_window.clone();

        let mut progress = 0.0;
        progress += 100.0 / wait_time as f64;
        level_bar_clone.set_value(progress);
        
        glib::timeout_add_seconds_local(1, move || {
            progress += 100.0 / wait_time as f64;
            level_bar_clone.set_value(progress);
            if progress >= 100.0 {
                message_window_clone.close();
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    // Message Box
    let message_main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // Do not show
    let do_not_show_main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    do_not_show_main_box.set_margin_top(10);

    let do_not_show_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let do_not_show_label = gtk::Label::new(Some("Do not show any more"));
    let do_not_show_checkbox = gtk::CheckButton::new();

    do_not_show_content_box.append(&do_not_show_label);
    do_not_show_content_box.append(&do_not_show_checkbox);
    do_not_show_content_box.set_halign(gtk::Align::Center);

    do_not_show_main_box.append(&do_not_show_content_box);

    // Connections
    dialog_main_box.append(&message_main_box);
    dialog_main_box.append(&do_not_show_main_box);

    message_window.set_child(Some(&dialog_main_box));
    message_window.show();
}

fn main() {
    print_program_info();

    println!("{}", t!("hello"));

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
    let test = gio::SimpleAction::new("test", None);
    
    quit.connect_activate(
        glib::clone!(@weak application => move |_action, _parameter| {
            application.quit();
        }),
    );
    
    // Keyboard shortcuts
    let new_window = application.clone();
    new.connect_activate(move |_action, _parameter| {
        create_main_window(&new_window);
    });

    open.connect_activate(move |_action, _parameter| {
        todo!() // Open wallet action activated
    });
    
    save.connect_activate(|_action, _parameter| {
        todo!() // Save wallet action activated
    });

    settings.connect_activate(move |_action, _parameter| {
        create_settings_window();
    });

    about.connect_activate(move |_action, _parameter| {
        create_about_window();
    });

    test.connect_activate(move |_action, _parameter| {
        create_message_window(
            "Test title dialog", 
            "One request every 10 seconds.", 
            Some(true), 
            Some(10)
        );
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

    // Only to start testing window
    application.set_accels_for_action("app.test", &["<Primary>T"]);
    application.add_action(&test);

    application.run();
}



// ANU QRNG -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

/// Fetch entropy data from ANU Quantum Random Number Generator (QRNG) API.
///
/// This function fetches entropy data from the ANU QRNG API based on the specified parameters.
///
/// # Arguments
///
/// * `entropy_length` - The length of the entropy string to fetch.
/// * `data_format` - The format of the data to fetch (e.g., "uint8", "uint16", "hex16").
/// * `array_length` - The length of the array of random numbers to fetch.
/// * `hex_block_size` - The block size for hex data format (optional).
///
/// # Returns
///
/// A string containing the fetched entropy data, or an empty string if the fetch fails.
fn get_entropy_from_anu(entropy_length: usize, data_format: &str, array_length: u32,hex_block_size: Option<u32>) -> String {
    fancy_print(Some(&t!("log.get_entropy_from_anu").to_string()), None);

    let start_time = SystemTime::now();

    let anu_data = fetch_anu_qrng_data(
        data_format, 
        array_length, 
        hex_block_size.unwrap()
    );

    if !&anu_data.as_ref().unwrap().is_empty() {
        create_anu_timestamp(start_time);
        // TODO: Check if global log is enabled, then save
        write_api_response_to_log(&anu_data);
    } else {
        return String::new()
    }

    let entropy = match data_format {
        "uint8" =>  {
            let uint8 = extract_uint8_data(&anu_data);

            process_uint8_data(&uint8)
        },
        "uint16" =>  {
            todo!() // Create uint16 ANU extraction
        },
        "hex16" =>  {
            todo!() // Create hex16 ANU extraction
            // let hex_strings = extract_hex_strings(
            //         &anu_data, 
            //         hex_block_size.unwrap().try_into().unwrap()
            //     );
            //     let mut anu_qrng_binary = String::new();
            //     for hex_string in hex_strings {
            //         // println!("Hex string: {}", hex_string);
            //         let bytes = hex::decode(hex_string).expect("Failed to decode hex string");
            //         let binary_string: String = bytes.iter()
            //             .map(|byte| format!("{:08b}", byte))
            //             .collect();
            //         // println!("Binary string: {:?}", binary_string);
            //         anu_qrng_binary.push_str(&binary_string);
            //     }
            //     // Write all binary strings to a file
            //     let qrng_file = format!("{}.binary", ANU_QRNG_FILE);
            //     let mut file = File::create(&qrng_file).expect("Can not read file");
            //     file.write_all(anu_qrng_binary.as_bytes()).expect("can not write to file");
            //     if anu_qrng_binary.len() < entropy_length {
            //         return Err(format!(
            //             "Entropy string too short for requested entropy length: {}",
            //             entropy_length
            //         ).into());
            //     }
            //     let max_start = anu_qrng_binary.len() - entropy_length;
            //     let start_point = rand::thread_rng().gen_range(0..=max_start);
            //     entropy_raw_binary = anu_qrng_binary
            //         .chars()
            //         .skip(start_point)
            //         .take(entropy_length)
            //         .collect();
            //     println!("Final entropy string: {}", entropy_raw_binary);
        },
        _ => {
            eprintln!("{}", &t!("error.anu.format"));
            return String::new()
        }
    };

    if entropy.len() > entropy_length {
        let original_len = entropy.len();
        let mut rng = rand::thread_rng();
        let start_index = rng.gen_range(0..original_len);

        let trimmed_entropy = if start_index + entropy_length < original_len {
            entropy[start_index..start_index + entropy_length].to_string()
        } else {
            entropy[start_index..].to_string()
        };

        return trimmed_entropy;
    } else if entropy.len() == entropy_length {
        return entropy.to_string();
    } else {
        eprintln!("{}", &t!("error.anu.short"));
        return String::new();
    }
}

/// Fetch data from ANU Quantum Random Number Generator (QRNG) API.
///
/// This function fetches data from the ANU QRNG API based on the specified parameters.
///
/// # Arguments
///
/// * `data_format` - The format of the data to fetch (e.g., "uint8", "uint16", "hex16").
/// * `array_length` - The length of the array of random numbers to fetch.
/// * `block_size` - The block size for hex data format.
///
/// # Returns
///
/// An optional string containing the fetched data, or None if the fetch fails.
fn fetch_anu_qrng_data(data_format: &str, array_length: u32, block_size: u32) -> Option<String> {
    fancy_print(Some(&t!("log.fetch_anu_qrng_data").to_string()), None);

    let current_time = SystemTime::now();
    let last_request_time = load_last_anu_request().unwrap();

    let elapsed = current_time.duration_since(last_request_time).unwrap_or(Duration::from_secs(0));
    let wait_duration = Duration::from_secs(ANU_REQUEST_INTERVAL_SECONDS as u64);

    if elapsed < wait_duration {
        let remaining_seconds = wait_duration.as_secs() - elapsed.as_secs();
        create_message_window(
            "ANU API Timeout", 
            &t!("error.anu.timeout", value = remaining_seconds), 
            Some(true), 
            Some(remaining_seconds as u32)
        );
        eprintln!("{}", &t!("error.anu.timeout", value = remaining_seconds));
        return Some(String::new());
    }
    
    create_message_window(
        "ANU QRNG API", 
        &t!("UI.main.anu.download"), 
        Some(true), 
        Some(5)
    );


    let mut socket_addr = ANU_API_URL
        .to_socket_addrs()
        .map_err(|e| format!("Socket address parsing error: {}", e))
        .unwrap();
    
    let socket_addr = socket_addr
        .next()
        .ok_or("No socket addresses found for ANU API URL")
        .unwrap();

    let mut stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(TCP_REQUEST_TIMEOUT_SECONDS))
        .map_err(|e| format!("Connection error: {}", e))
        .unwrap();

    let anu_request = format!(
        "GET /API/jsonI.php?type={}&length={}&size={} HTTP/1.1\r\nHost: qrng.anu.edu.au\r\nConnection: close\r\n\r\n",
        data_format, array_length, block_size
    )
    .into_bytes();

    stream.write_all(&anu_request)
        .map_err(|e| format!("Write error: {}", e))
        .unwrap();

    stream.flush()
        .map_err(|e| format!("Flush error: {}", e))
        .unwrap();

    let mut response = String::new();
    let mut buffer = [0; 256];
    let mut chunks = Vec::new();

    loop {
        // print!(".");
        match stream.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);
                // print!("{}", chunk);
                response.push_str(&chunk);
                chunks.push(chunk.to_string());

                if chunk.ends_with("\r\n\r\n") {
                    break;
                }
            }
            Ok(_) | Err(_) => break,
        }
    }

    // print!("done\n");

    let combined_response = chunks.concat();

    Some(combined_response)
}

/// Load the timestamp of the last ANU QRNG request from a file.
///
/// This function loads the timestamp of the last ANU QRNG request from a file.
///
/// # Returns
///
/// An optional `SystemTime` representing the timestamp of the last request, or None if the file does not exist.
fn load_last_anu_request() -> Option<SystemTime> {
    fancy_print(Some(&t!("log.load_last_anu_request").to_string()), None);

    let path = Path::new(ANU_TIMESTAMP_FILE);
    if path.exists() {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            if let Some(Ok(timestamp_str)) = reader.lines().next() {
                if let Ok(timestamp) = timestamp_str.trim().parse::<i64>() {
                    return Some(SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp as u64));
                }
            }
        }
    }
    Some(SystemTime::UNIX_EPOCH)
}

/// Create a timestamp file for the ANU request.
///
/// This function creates a timestamp file for the ANU request to track the last request time.
///
/// # Arguments
///
/// * `time` - The current time for the ANU request.
fn create_anu_timestamp(time: SystemTime) {
    fancy_print(Some(&t!("log.create_anu_timestamp").to_string()), None);

    let timestamp = time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string();

    if let Some(parent) = Path::new(ANU_TIMESTAMP_FILE).parent() {
        fs::create_dir_all(parent).expect("Can not create log directory");
    }

    let mut file = File::create(ANU_TIMESTAMP_FILE).expect("Can not create ANU timestamp file");

    file.write_all(timestamp.as_bytes()).expect("Can not write to ANU timestamp file");

    println!("ANU timestamp: {}",timestamp);
}

/// Write the ANU API response to a log file.
///
/// This function writes the ANU API response to a log file.
///
/// # Arguments
///
/// * `response` - The ANU API response.
fn write_api_response_to_log(response: &Option<String>) {
    fancy_print(Some(&t!("log.write_api_response_to_log").to_string()), None);

    // IMPLEMENT: Check if log is enabled before writing it
    let current_time = SystemTime::now();
    let timestamp = current_time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let log_file = format!("{}-{}.log", ANU_LOG_DIRECTORY, timestamp);

    if let Some(parent) = Path::new(log_file.as_str()).parent() {
        match fs::create_dir_all(parent) {
            Ok(_) => {
                let mut file = match File::create(&log_file) {
                    Ok(file) => file,
                    Err(e) => {
                        eprintln!("Error creating file: {}", e);
                        return;
                    }
                };

                if let Some(data) = &response {
                    let bytes = data.as_bytes();
                    if let Err(e) = file.write_all(bytes) {
                        eprintln!("Can not write ANU response to log file: {}", e);
                    }
                    println!("ANU response written to log");
                } else {
                    eprintln!("ANU response is empty");
                }
            }
            Err(err) => {
                eprintln!("Error creating directory {}: {}", parent.display(), err);
            }
        }
    }
}

/// Extract uint8 data from the ANU API response.
///
/// This function extracts uint8 data from the ANU API response.
///
/// # Arguments
///
/// * `api_response` - The ANU API response.
///
/// # Returns
///
/// An optional `Vec<u8>` containing the extracted uint8 data, or None if extraction fails.
fn extract_uint8_data(api_response: &Option<String>) -> Option<Vec<u8>> {
    fancy_print(Some(&t!("log.extract_uint8_data").to_string()), None);

    // Check if the API response is present
    let api_response = match api_response {
        Some(response) => response,
        None => {
            println!("ANU response is None.");
            return None;
        }
    };

    // Find the index where the JSON data starts
    let json_start_index = match api_response.find('{') {
        Some(index) => index,
        None => {
            println!("JSON data not found in the response.");
            return None;
        }
    };

    // Find the index where the JSON data ends
    let json_end_index = match api_response.rfind('}') {
        Some(index) => index,
        None => {
            println!("JSON data end not found in the response.");
            return None;
        }
    };

    // Extract the JSON data
    let json_str = &api_response[json_start_index..=json_end_index];

    // Parse JSON
    let parsed_json: Result<serde_json::Value, _> = serde_json::from_str(json_str);
    let parsed_json = match parsed_json {
        Ok(value) => value,
        Err(err) => {
            println!("Failed to parse JSON: {}", err);
            return None;
        }
    };

    // Extract uint8 data
    let data_array = parsed_json["data"].as_array();
    let data_array = match data_array {
        Some(arr) => arr,
        None => {
            println!("No data array found.");
            return None;
        }
    };

    let mut uint8_data = Vec::new();

    for data_item in data_array {
        if let Some(byte_val) = data_item.as_u64() {
            if byte_val <= u8::MAX as u64 {
                uint8_data.push(byte_val as u8);
            } else {
                eprintln!("Error parsing byte: number too large to fit in target type");
            }
        } else {
            eprintln!("Invalid byte value: {:?}", data_item);
        }
    }

    Some(uint8_data)
}

/// Process uint8 data into a binary string.
///
/// This function processes uint8 data into a binary string.
///
/// # Arguments
///
/// * `data` - The uint8 data to process.
///
/// # Returns
///
/// A string containing the processed binary data.
fn process_uint8_data(data: &Option<Vec<u8>>) -> String {
    fancy_print(Some(&t!("log.process_uint8_data").to_string()), None);

    let data = match data {
        Some(data) => data,
        None => {
            eprintln!("ANU response was empty.");
            return String::new();
        }
    };

    let binary_string = data
        .iter()
        .flat_map(|byte| {
            format!("{:08b}", byte)
                .chars()
                .collect::<Vec<_>>()
        })
        .collect::<String>();

    // println!("ANU entropy: {}", &binary_string);

    binary_string
}






// TESTING -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

thread_local! {
    static ADDRESS_DATA: std::cell::RefCell<WalletSettings> = std::cell::RefCell::new(WalletSettings::default());
}

#[derive(Debug, Default)]
struct WalletSettings {
    entropy_string: Option<String>,
    entropy_checksum: Option<String>,
    mnemonic_words: Option<String>,
    seed: Option<String>,
    master_private_key_bytes: Option<Vec<u8>>,
    master_chain_code_bytes: Option<Vec<u8>>,

    // SEND:
    // ADDRESS_DATA.with(|data| {
    //     let mut data = data.borrow_mut();
    //     println!("RNG entropy (string): {}", &rng_entropy_string);
    //     data.entropy = Some(rng_entropy_string.clone());
    // });
    // 
    // GET:
    // let master_private_key_bytes = ADDRESS_DATA.with(|data| {
    //     let data = data.borrow();
    //     data.master_private_key_bytes.clone().unwrap()
    // });
}

fn derive_child_key(parent_key: &[u8], parent_chain_code: &[u8], index: u32, hardened: bool) -> Option<(Vec<u8>, Vec<u8>)> {
    fancy_print(Some(&t!("log.derive_child_key").to_string()), None);

    let secp = secp256k1::Secp256k1::new();
    let mut data = vec![];

    if hardened {
        data.push(0x00);
        data.extend_from_slice(parent_key);
    } else {
        let parent_key = secp256k1::SecretKey::from_slice(parent_key).expect("32 bytes, within curve order");
        let parent_pubkey = secp256k1::PublicKey::from_secret_key(&secp, &parent_key);
        data.extend_from_slice(&parent_pubkey.serialize());
    }
    
    let index_bytes = index.to_be_bytes();
    data.extend_from_slice(&index_bytes);

    let result = hmac_sha512(parent_chain_code, &data);

    let child_key = secp256k1::SecretKey::from_slice(&result[0..32]).ok()?;
    let tweak = secp256k1::Scalar::from_be_bytes(parent_key.try_into().ok()?).ok()?;
    child_key.add_tweak(&tweak).ok()?;
    let child_chain_code = result[32..64].to_vec();

    Some((child_key.secret_bytes().to_vec(), child_chain_code))
}

fn derive_from_path(master_key: &[u8], master_chain_code: &[u8], path: &str) -> Option<(secp256k1::SecretKey, [u8; 32])> {
    fancy_print(Some(&t!("log.derive_from_path").to_string()), None);

    println!("Derivation path: {:?}", path);

    let mut key = master_key.to_vec();
    let mut chain_code = master_chain_code.to_vec();

    for part in path.split('/') {
        if part == "m" {
            continue;
        }

        let hardened = part.ends_with("'");
        let index: u32 = match part.trim_end_matches("'").parse() {
            Ok(index) => index,
            Err(_) => {
                eprintln!("Error: Unable to parse index from path part: {}", part);
                return None;
            }
        };

        let derived = derive_child_key(&key, &chain_code, index, hardened)?;
        key = derived.0;
        chain_code = derived.1;
    }

    let secret_key = secp256k1::SecretKey::from_slice(&key)
        .expect("Error: Unable to create SecretKey from key slice");

    let mut chain_code_array = [0u8; 32];
    chain_code_array.copy_from_slice(&chain_code);

    Some((secret_key, chain_code_array))
}

fn generate_address(pk: &secp256k1::PublicKey) -> String {
    fancy_print(Some(&t!("log.generate_address").to_string()), None);

    let pk_bytes = pk.serialize();
    println!("Public key bytes: {:?}", &pk_bytes);
    
    let hash160 = sha256_and_ripemd160(&pk_bytes);
    
    let mut payload = vec![0x00];
    payload.extend(&hash160);
    println!("Extended sha256_and_ripemd160 payload: {:?}", &payload);
    
    let checksum = double_sha256(&payload);
    
    let address_checksum = &checksum[0..4];
    println!("Address checksum: {:?}", address_checksum);
    
    let mut address_payload = payload;
    address_payload.extend_from_slice(address_checksum);
    println!("Extended Address payload: {:?}", address_payload);
    
    bs58::encode(address_payload).into_string()
}

fn sha256_and_ripemd160(input: &[u8]) -> Vec<u8> {
    fancy_print(Some(&t!("log.sha256_and_ripemd160").to_string()), None);

    println!("\n#### sha256_and_ripemd160 ####");
    println!("Received data: {:?}", input);

    let mut hasher = Sha256::new();
    hasher.update(input);
    let hash = hasher.finalize();
    println!("Sha256 hash: {:?}", hash);
    
    let mut ripemd = ripemd::Ripemd160::new();
    ripemd.update(&hash);
    let final_hash = ripemd.finalize().to_vec();
    println!("sha256_and_ripemd160 output: {:?}", final_hash);

    final_hash
}

fn double_sha256(input: &[u8]) -> Vec<u8> {
    fancy_print(Some(&t!("log.double_sha256").to_string()), None);

    println!("Received data: {:?}", input);

    let mut hasher = Sha256::new();
    hasher.update(input);
    let first_hash = hasher.finalize();
    println!("First hash: {:?}", first_hash);
    
    let mut hasher = Sha256::new();
    hasher.update(&first_hash);
    let final_hash = hasher.finalize().to_vec();
    println!("doubleSha256 output: {:?}", final_hash);

    final_hash
}



















// OLD CODE
// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

// ANU extract hex16
// TODO: recheck if hex16 code is still working
// fn extract_hex_strings(response: &str, hex_block_size: usize) -> Vec<String> {
//     let hex_block_size = hex_block_size * 2; // Adjust for byte format for ANU
//     let mut hex_strings = Vec::new();
//     let mut current_string = String::new();
//     let mut in_hex_string = false;
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
//     hex_strings
// }