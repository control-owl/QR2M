// Crates
use std::{io::{self, Read, Seek, Write}, fs::{self, File}, path::Path, vec, str::FromStr, ops::Index};
use glib::{value::ValueType, PropertyGet};
use structopt::StructOpt;
use hex;
use rand::{Rng, RngCore};
use sha2::{Digest, Sha256};
use bitcoin::{self, hashes::sha256, Script};
use bip39;
use csv::ReaderBuilder;


// Project files
mod error_handler;
use error_handler::CustomError;
mod converter;


// Global variables
const ENTROPY_FILE: &str = "entropy/test.qrn";
// const ENTROPY_FILE: &str = "./entropy/binary.qrn";

const WORDLIST_FILE: &str = "./lib/bip39-english.txt";
const VALID_ENTROPY_LENGTHS: [u32; 5] = [128, 160, 192, 224, 256];
const VALID_BIP_DERIVATIONS: [u32; 2] = [32, 44];
const VALID_MNEMONIC_WORD_COUNT: [u32; 5] = [12, 15, 18, 21, 24];
const VALID_ENTROPY_SOURCES: &'static [&'static str] = &["rng", "file"];

const APP_DESCRIPTION: Option<&str> = option_env!("CARGO_PKG_DESCRIPTION");
const APP_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
const APP_AUTHOR: Option<&str> = option_env!("CARGO_PKG_AUTHORS");

// Debugging log
macro_rules! D3BUG {
    (info, $($arg:tt)*) => (
        if Cli::from_args().app_verbosity >= 3 {
            println!("\x1b[1;31m{}\x1b[0m", format_args!($($arg)*));
        }
    );
    (log, $($arg:tt)*) => (
        if Cli::from_args().app_verbosity >= 2 {
            println!("[LOG] {}", format_args!($($arg)*));
        }
    );
    (warning, $($arg:tt)*) => (
        if Cli::from_args().app_verbosity >= 1 {
            eprintln!("ε(๏_๏)з Gobbledygook ٩(͡๏̯͡๏)۶\n{}", format_args!($($arg)*));
        }
    );
    (error, $($arg:tt)*) => (
            eprintln!("\x1b[1;31m ,_,\n(O,O)\n(   )\n-\"-\"---ERROR-\n\n{}\x1b[0m", format_args!($($arg)*))
    );
    (output, $($arg:tt)*) => (
            println!("{}", format_args!($($arg)*))
    );
}


#[derive(Debug)]
struct CoinType {
    index: u32,
    path: u32,
    symbol: String,
    coin: String,
}


// CLI Arguments
#[derive(StructOpt)]
struct Cli {
    #[structopt(short = "a", long = "address-count", default_value = "1")]
    address_count: u32,

    #[structopt(short = "b", long = "bip", default_value = "44")]
    derivation_path: u32,

    #[structopt(short = "c", long = "coin", default_value = "BTC")]
    coin_symbol: String,

    #[structopt(short = "e", long = "entropy-source", default_value = "rng")]
    entropy_source: String,

    #[structopt(short = "h", long = "hardened-address")]
    hardened_address: bool,

    #[structopt(short = "l", long = "entropy-length", default_value = "256")]
    entropy_length: u32,

    #[structopt(short = "p", long = "passphrase", default_value = "")]
    seed_passphrase: String,
    
    #[structopt(short = "m", long = "import-mnemonic", default_value = "")]
    imported_mnemonic: String,
    
    #[structopt(short = "v", long = "verbosity", default_value = "0")]
    app_verbosity: u32,

    #[structopt(short = "s", long = "import-seed", default_value = "")]
    imported_seed: String,

}

fn process_arguments(cli_args: &Cli) {
    D3BUG!(info, "Program arguments:");

    D3BUG!(log, "App verbosity: {}", cli_args.app_verbosity);
    D3BUG!(log, "Entropy source: {}", cli_args.entropy_source);
    D3BUG!(log, "Entropy length: {}", cli_args.entropy_length);
    D3BUG!(log, "Seed passphrase: {}", cli_args.seed_passphrase);
    D3BUG!(log, "Coin Symbol: {}", cli_args.coin_symbol);
    D3BUG!(log, "Derivation Path: {}", cli_args.derivation_path);
    D3BUG!(log, "Address Count: {}", cli_args.address_count);
    D3BUG!(log, "Imported mnemonic: {}", cli_args.imported_mnemonic);
    D3BUG!(log, "Imported seed: {}", cli_args.imported_seed);
    // D3BUG!(log, ": {}", cli_args.);
}

fn print_program_info() {

    
    
    println!(" ██████╗ ██████╗ ██████╗ ███╗   ███╗");
    println!("██╔═══██╗██╔══██╗╚════██╗████╗ ████║");
    println!("██║   ██║██████╔╝ █████╔╝██╔████╔██║");
    println!("██║▄▄ ██║██╔══██╗██╔═══╝ ██║╚██╔╝██║");
    println!("╚██████╔╝██║  ██║███████╗██║ ╚═╝ ██║");
    println!(" ╚══▀▀═╝ ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝");
    println!("{} ({})\n{}\n", &APP_DESCRIPTION.unwrap(), &APP_VERSION.unwrap(), &APP_AUTHOR.unwrap());
}

fn generate_entropy_from_rng(length: &u32) -> String {
    // D3BUG!(info, "Entropy (RNG):");
    
    let mut rng = rand::thread_rng();
    let binary_string: String = (0..*length)
        .map(|_| rng.gen_range(0..=1))
        .map(|bit| char::from_digit(bit, 10).unwrap())
        .collect();

    // D3BUG!(output, "RNG entropy: {:?}", binary_string.to_string());
    binary_string
}

fn check_address_count_input(count: &u32) -> Result<u32, CustomError> {
    D3BUG!(info, "Checking address count input:");

    match count.to_string().parse::<u32>() {
        Ok(parsed_value) => {
            D3BUG!(log, "Desired addresses: {:?}", &count);
            Ok(parsed_value)
        },
        Err(_) => {
            let error_msg: CustomError = CustomError::InputNotValidNumber(count.to_string());
            D3BUG!(error, "{}", error_msg);
            Err(CustomError::InputNotValidNumber(count.to_string()))
        }
    }
}

fn generate_entropy_from_file(file_path: &str, entropy_length: usize) -> String {
    // D3BUG!(info, "Entropy from file:");
    
    let file = File::open(file_path);
    let mut reader = match file {
        Ok(file) => io::BufReader::new(file),
        Err(err) => {
            let error_msg = format!("Can not read entropy file: {}", err);
            D3BUG!(error, "{}", error_msg);
            return String::new(); // Return default value or handle as appropriate
        }
    };

    // D3BUG!(log, "Entropy file: {:?}", file_path);
    
    let file_length = reader.seek(io::SeekFrom::End(0));
    let file_length = match file_length {
        Ok(length) => length,
        Err(err) => {
            let error_msg = format!("Error getting file length: {}", err);
            D3BUG!(error, "{}", error_msg);
            return String::new(); // Return default value or handle as appropriate
        }
    };
    // D3BUG!(log, "Entropy file length: \"{:?}\"", file_length);

    if file_length < entropy_length as u64 {
        let error_msg = format!("File too small for requested entropy length: {}", entropy_length);
        D3BUG!(error, "{}", error_msg);
        return String::new(); // Return default value or handle as appropriate
    }

    let max_start = file_length.saturating_sub(entropy_length as u64);
    let start_point = rand::thread_rng().gen_range(0..=max_start);

    match reader.seek(io::SeekFrom::Start(start_point)) {
        Ok(_) => (),
        Err(err) => {
            let error_msg = format!("Error seeking in file: {}", err);
            D3BUG!(error, "{}", error_msg);
            return String::new(); // Return default value or handle as appropriate
        }
    }

    let mut entropy_raw_binary = String::new();
    match reader.take(entropy_length as u64).read_to_string(&mut entropy_raw_binary) {
        Ok(_) => (),
        Err(err) => {
            let error_msg = format!("Error reading from file: {}", err);
            D3BUG!(error, "{}", error_msg);
            return String::new(); // Return default value or handle as appropriate
        }
    }

    D3BUG!(output, "Entropy: {:?}", entropy_raw_binary);
    entropy_raw_binary
}

fn calculate_checksum(entropy: &str, entropy_length: &u32) -> String {
    // D3BUG!(info, "Checksum:");

    let entropy_binary = converter::convert_string_to_binary(&entropy);
    let hash_raw_binary: String = converter::convert_binary_to_string(&Sha256::digest(&entropy_binary));
    // D3BUG!(log, "sha256(entropy): {:?}", hash_raw_binary);

    let checksum_lenght = entropy_length / 32;
    // D3BUG!(log, "Checksum length: \"{:?}\"", checksum_lenght);

    // Take 1 bit for every 32 bits of the hash 
    // let checksum_raw_binary: String = hash_raw_binary.chars().take(checksum_lenght.try_into().unwrap()).collect();
    let checksum_raw_binary: String = hash_raw_binary.chars().take(checksum_lenght.try_into().unwrap()).collect();

    // D3BUG!(output, "Checksum: {:?}", checksum_raw_binary);
    // Ok(checksum_raw_binary)
    checksum_raw_binary
}

fn get_full_entropy(entropy: &str, checksum: &str) -> String {
    // D3BUG!(info, "Final Entropy:");

    let full_entropy = format!("{}{}", entropy, checksum);
    
    // D3BUG!(output, "Final entropy: {:?}", full_entropy);
    // Ok(full_entropy)
    full_entropy
}

fn get_mnemonic_from_entropy(final_entropy_binary: &str) -> String {
    D3BUG!(info, "Mnemonic:");

    // Split the final entropy into groups of 11 bits
    let chunks: Vec<String> = final_entropy_binary.chars().collect::<Vec<char>>().chunks(11).map(|chunk| chunk.iter().collect()).collect();

    // Convert each chunk to decimal numbers
    let mnemonic_decimal: Vec<u32> = chunks.iter().map(|chunk| u32::from_str_radix(chunk, 2).unwrap()).collect();

    // Read the file containing mnemonic words
    let mnemonic_file_content = fs::read_to_string(WORDLIST_FILE).expect("Can not read entropy file");
    let mnemonic_words: Vec<&str> = mnemonic_file_content.lines().collect();

    let mnemonic_words: Vec<&str> = mnemonic_decimal.iter().map(|&decimal| {
        if (decimal as usize) < mnemonic_words.len() {
            mnemonic_words[decimal as usize]
        } else {
            "INVALID_WORD"
        }
    }).collect();
    D3BUG!(log, "Mnemonic numbers: {:?}", mnemonic_decimal);

    let final_mnemonic = mnemonic_words.join(" ");
    
    D3BUG!(output, "Mnemonic words: {:?}", final_mnemonic);
    final_mnemonic

}

fn create_bip39_seed_from_entropy(entropy: &str, passphrase: &str) -> String {
    // D3BUG!(info, "Seed:");
    
    let entropy_vector = converter::convert_string_to_binary(&entropy);
    let mnemonic_result = bip39::Mnemonic::from_entropy(&entropy_vector).expect("Can not create mnemomic words");
    let mnemonic = mnemonic_result;
    let seed = bip39::Mnemonic::to_seed(&mnemonic, passphrase);
    let seed_hex = hex::encode(&seed[..]);

    // D3BUG!(output, "BIP39 passphrase: {:?}", passphrase);
    // D3BUG!(output, "BIP39 seed: {:?}", seed_hex);

    seed_hex.to_string()
}

fn create_bip39_seed_from_mnemonic(mnemonic: &String, passphrase: &str) -> Result<String, CustomError> {
    D3BUG!(info, "BIP39 seed:");

    let mnemonic_result = bip39::Mnemonic::from_str(&mnemonic);
    let mnemonic = mnemonic_result?;
    let seed = bip39::Mnemonic::to_seed(&mnemonic, passphrase);
    let seed_hex = hex::encode(&seed[..]);

    D3BUG!(output, "BIP39 Passphrase: {:?}", passphrase);
    D3BUG!(output, "BIP39 Seed (hex): {:?}", seed_hex);

    Ok(seed_hex)
}

fn check_entropy_length(entropy_length: u32) -> Result<u32, CustomError> {
    D3BUG!(info, "Checking entropy length input:");
    
    if !VALID_ENTROPY_LENGTHS.contains(&entropy_length) {
        let allowed_values = VALID_ENTROPY_LENGTHS
            .iter()
            .map(|&x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        let error_msg: CustomError = CustomError::InvalidEntropyLength(allowed_values);
        D3BUG!(error, "{}", error_msg);
        return Err(CustomError::InvalidEntropyLength(entropy_length.to_string()))
    }

    D3BUG!(log, "CLI argument: {:?}", &entropy_length);
    Ok(entropy_length)
}

fn check_bip_entry(bip_entry: u32) -> Result<u32, CustomError> {
    D3BUG!(info, "Checking BIP input:");
    
    if !VALID_BIP_DERIVATIONS.contains(&bip_entry) {
        let allowed_values = VALID_BIP_DERIVATIONS
            .iter()
            .map(|&x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        let error_msg: CustomError = CustomError::InvalidBipEntry(allowed_values);
        D3BUG!(error, "{}", error_msg);
        return Err(CustomError::InvalidBipEntry(bip_entry.to_string()))
    }
    
    D3BUG!(log, "CLI argument: {:?}", &bip_entry);
    Ok(bip_entry)
}

fn check_source_entry(source_entry: &str) -> Result<String, CustomError> {
    D3BUG!(info, "Checking entropy source input:");

    if !VALID_ENTROPY_SOURCES.contains(&source_entry) {
        let allowed_values = VALID_ENTROPY_SOURCES
            .iter()
            .map(|&x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        let error_msg: CustomError = CustomError::InvalidSourceEntry(allowed_values);
        D3BUG!(error, "{}", error_msg);
        return Err(CustomError::InvalidSourceEntry(source_entry.to_string()))
    }

    D3BUG!(log, "CLI argument: {:?}", &source_entry);
    Ok(source_entry.to_string())
}

// fn create_master_private_key(seed_hex: String) -> Result<bitcoin::bip32::Xpriv, CustomError> {
fn create_master_private_key(seed_hex: String) -> String {
    D3BUG!(info, "Master key:");

    let seed = hex::decode(seed_hex).expect("Failed to decode seed hex");
    let master_key = bitcoin::bip32::Xpriv::new_master(bitcoin::Network::Bitcoin, &seed).expect("Failed to derive master key");

    D3BUG!(output, "BIP32 Master private key: \"{}\"", master_key); 
    master_key.to_string()
}

fn check_coin_type(coin_symbol: &str) -> Result<u32, CustomError> {
    D3BUG!(info, "Coin:");
    D3BUG!(output, "Coin: {:?}", &coin_symbol.to_uppercase());

    let path = Path::new("lib/bip44-coin_type.csv");
    let file = File::open(path)?;
    
    let mut rdr = ReaderBuilder::new().from_reader(file);
    let mut matching_entries: Vec<CoinType> = Vec::new();
    
    for record in rdr.records() {
        let record = record?;
        let index: u32 = record[0].parse()?;
        let path: u32 = u32::from_str_radix(record[1].trim_start_matches("0x"), 16)?;
        let symbol = record[2].to_string();
        let coin_name = record[3].to_string();
        
        if symbol.to_lowercase() == coin_symbol.to_lowercase() {
            let coin_type = CoinType {
                index,
                path,
                symbol,
                coin: coin_name,
            };
            matching_entries.push(coin_type);
        }
    }

    if matching_entries.is_empty() {
        let error_msg: CustomError = CustomError::InvalidCoinSymbol(coin_symbol.to_string());
        D3BUG!(warning, "{}", error_msg);
        return Err(CustomError::InvalidCoinSymbol(coin_symbol.to_string()));
    } else {
        if matching_entries.len() > 1 {
            println!("Multiple entries found for symbol {:?}. Please choose one:", coin_symbol.to_uppercase());
            for (i, entry) in matching_entries.iter().enumerate() {
                D3BUG!(output, "{}: {:?}", i + 1, entry);
            }
            
            print!("Enter the index of the desired coin: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            let chosen_index: usize = input.trim().parse()?;
            if chosen_index > 0 && chosen_index <= matching_entries.len() {
                return Ok(matching_entries[chosen_index - 1].index);
            } else {
                return Ok(0);
            }
        }
    }
    
    Ok(matching_entries[0].index)
}

fn import_mnemonic_words(mnemonic: &str, wordlist_path: &str) -> Result<String, CustomError> {
    D3BUG!(info, "Importing mnemonic:");

    let mnemonic_words: Vec<&str> = mnemonic.split_whitespace().collect();

    if !VALID_MNEMONIC_WORD_COUNT.contains(&(mnemonic_words.len() as u32)) {
        let allowed_values = VALID_MNEMONIC_WORD_COUNT
            .iter()
            .map(|&x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        let error_msg: CustomError = CustomError::InvalidMnemonicWordCount(allowed_values);
        D3BUG!(error, "{}", error_msg);
        return Err(CustomError::InvalidMnemonicWordCount(mnemonic_words.len().to_string()))
    } else {
        D3BUG!(log, "Imported mnemonic word count {:?}", &mnemonic_words.len());
    }

    let wordlist_content = match fs::read_to_string(wordlist_path) {
        Ok(content) => content,
        Err(_) => {
            let error_msg: CustomError = CustomError::WordlistReadError;
            D3BUG!(error, "{}", error_msg);
            return Err(CustomError::WordlistReadError)
        }
    };
    
    for word in &mnemonic_words {
        if !wordlist_content.contains(word) {
            let error_msg: CustomError = CustomError::InvalidMnemonicWord(word.to_string());
            D3BUG!(error, "{}", error_msg);
            return Err(CustomError::InvalidMnemonicWord(word.to_string()))
        }
    }
    let words: String = mnemonic_words.join(" ");

    D3BUG!(log, "Imported mnemonic {:?}", &words);
    Ok(words)
}

fn import_seed(imported_seed: &str) -> Result<String, CustomError> {
    D3BUG!(info, "Importing seed:");

    if !imported_seed.chars().all(|c| c.is_ascii_hexdigit()) {
        let error_msg: CustomError = CustomError::InvalidSeed(imported_seed.to_string());
        D3BUG!(error, "{}", error_msg);
        return Err(error_msg);
    }

    match hex::decode(imported_seed) {
        Ok(decoded) => {
            D3BUG!(output, "Imported seed: {:?}", imported_seed);
            Ok(hex::encode(decoded))
        },
        Err(err) => {
            // eprintln!("Error decoding seed: {}", err);
            // Err(CustomError::DecodingError(err.to_string()))
            let error_msg: CustomError = CustomError::DecodingError(imported_seed.to_string());
            D3BUG!(error, "{}", error_msg);
            return Err(CustomError::DecodingError(err.to_string()))
        }
    }
}

fn create_derivation_path(
    cli_bip: u32,
    coin_type: u32,
    account: Option<u32>,
    change: Option<u32>,
    // index: Option<u32>,
) -> Result<Vec<bitcoin::bip32::ChildNumber>, bitcoin::bip32::Error> {
    D3BUG!(info, "Derivation path:");
    
    let purpose = bitcoin::bip32::ChildNumber::from_hardened_idx(cli_bip)?;
    let coin_type = bitcoin::bip32::ChildNumber::from_hardened_idx(coin_type)?;
    let account = bitcoin::bip32::ChildNumber::from_hardened_idx(account.unwrap_or(0)).expect("Invalid child number");
    let change = bitcoin::bip32::ChildNumber::from_normal_idx(change.unwrap_or(0)).expect("Invalid child number");
    // let index = bitcoin::bip32::ChildNumber::from_normal_idx(index.unwrap_or(0)).expect("Invalid child number");
    
    let derivation = match cli_bip {
        32 => vec![account, change],
        // 44 => vec![purpose, coin_type, account],
        44 => vec![purpose, coin_type, account, change],
        _ => vec![], // You may want to handle the case where bip is None
    };
    
    let mut path = String::from("m");
    
    for child_number in &derivation {
        path.push_str(&format!("/{}", child_number.to_string()));
    }

    D3BUG!(output, "BIP: {:?}", &cli_bip);
    D3BUG!(output, "Derivation path: {:?}", &path);
    Ok(derivation)
}

fn create_extended_private_key(
    master: &bitcoin::bip32::Xpriv,
    derivation: &Vec<bitcoin::bip32::ChildNumber>,
) -> Result<bitcoin::bip32::Xpriv, bitcoin::bip32::Error> {
    D3BUG!(info, "Extended private keys:");

    let secp = bitcoin::secp256k1::Secp256k1::new();
    
    let bip32_xprv = master
        .derive_priv(&secp, &derivation)
        .expect("Failed to derive derivation private key");
    D3BUG!(output, "BIP32 xprv: \"{}\"", bip32_xprv);


    let mut modified_derivation = derivation.clone();
    modified_derivation.pop();
    let extended_key = master
        .derive_priv(&secp, &modified_derivation)
        .expect("Failed to derive extended private key");
    D3BUG!(output, "Account xprv : \"{}\"", extended_key);


    Ok(bip32_xprv)
}

fn create_extended_public_key(
    xprv: &bitcoin::bip32::Xpriv,
    index: Option<&u32>
) -> Result<bitcoin::bip32::Xpub, CustomError> {
    D3BUG!(info, "Account extended public key");

    let secp = bitcoin::secp256k1::Secp256k1::new();
    let xpubkey = bitcoin::bip32::Xpub::from_priv(&secp, &xprv);
    
    D3BUG!(output, "xpub: {:?}", xpubkey.to_string());


    Ok(xpubkey)
}

fn create_bitcoin_address(
    xprv: bitcoin::bip32::Xpriv,
    derivation: &Vec<bitcoin::bip32::ChildNumber>,
    count: &u32,
) -> Result<Vec<bitcoin::Address>, CustomError> {
    D3BUG!(info, "Bitcoin addresses:");

    let secp = bitcoin::secp256k1::Secp256k1::new();
    let mut addresses = Vec::new();

    // for &format in ["p2pkh", "p2wpkh", "p2tr"].iter() {
    for &format in ["p2pkh", "p2wpkh"].iter() {
        for index in 0..*count {
            let child = bitcoin::bip32::ChildNumber::from_hardened_idx(index)?;
            let child_xprv = xprv.derive_priv(&secp, &child)?;
            let child_pubkey = child_xprv.to_priv().public_key(&secp);

            let address = match format {
                "p2pkh" => bitcoin::Address::p2pkh(&child_pubkey, bitcoin::Network::Bitcoin),
                "p2wpkh" => bitcoin::Address::p2wpkh(&child_pubkey, bitcoin::Network::Bitcoin)?,
                "p2tr" => {
                    // Create a Secp256k1 context
                    let mut rng = rand::thread_rng();
                    let seckey_bytes: [u8; 32] = rng.gen();
                    let seckey =
                        secp256k1::SecretKey::from_slice(&seckey_bytes).expect("Invalid secret key");
                    let keypair = seckey.keypair(&secp);
                    let (xonly_public_key, parity) = bitcoin::XOnlyPublicKey::from_keypair(&keypair);

                    bitcoin::Address::p2tr(
                        &secp,
                        xonly_public_key,
                        None,
                        bitcoin::Network::Bitcoin,
                    )
                }
                _ => return Err(CustomError::New(format.to_string())),
            };

            let mut path = String::from("m");
            for child_number in derivation {
                path.push_str(&format!("/{}", child_number.to_string()));
            }

            D3BUG!(output, "{}/{} [{}]: {:?}", &path, child, &format, address);

            addresses.push(address);
        }
    }

    Ok(addresses)
}









// -----------------------------------------------------------------------------------------

// GTK4
use gtk4 as gtk;
use gtk::{
        ffi::{GtkCenterLayoutClass, GtkLabel}, 
        gdk::BUTTON_MIDDLE, 
        gio::{self, MenuItem, Menu}, 
        glib::{self, clone}, 
        prelude::*, 
        Stack, 
        StackSidebar, 
        StackTransitionType,
        EntryCompletion, ListStore, Label
    };


fn create_coin_completion_model() -> gtk::ListStore {
    // Replace this with your actual function to read coin symbols from CSV
    let valid_coin_symbols = read_csv("lib/bip44-coin_type.csv");

    let store = gtk::ListStore::new(&[glib::Type::STRING]);
    for coin_symbol in valid_coin_symbols.iter() {
        store.set(&store.append(), &[(0, &coin_symbol)]);
    }
    store
}



fn gtk4_create_main_menu(app: &gtk::Application) {
    let about = gio::ActionEntry::builder("about")
        .activate(|_, _, _| println!("About was pressed"))
        .build();

    let quit = gio::ActionEntry::builder("quit")
        .activate(|app: &gtk::Application, _, _| app.quit())
        .build();

    app.add_action_entries([about, quit]);

    let menubar = {
        let wallet_menu = {
            let open_menu_item = MenuItem::new(Some("Open"), Some("app.open"));
            let save_menu_item = MenuItem::new(Some("Save"), Some("app.save"));
            let quit_menu_item = MenuItem::new(Some("Quit"), Some("app.quit"));
            
            let wallet_menu = Menu::new();
            wallet_menu.append_item(&open_menu_item);
            wallet_menu.append_item(&save_menu_item);
            wallet_menu.append_item(&quit_menu_item);
            wallet_menu
        };

        let entropy_menu = {
            let new_menu_item = MenuItem::new(Some("New"), Some("app.new_entropy"));

            let entropy_menu = Menu::new();
            entropy_menu.append_item(&new_menu_item);
            entropy_menu
        };

        let help_menu = {
            let about_menu_item = MenuItem::new(Some("About"), Some("app.about"));

            let help_menu = Menu::new();
            help_menu.append_item(&about_menu_item);
            help_menu
        };


        let menubar = Menu::new();
        menubar.append_submenu(Some("Wallet"), &wallet_menu);
        menubar.append_submenu(Some("Entropy"), &entropy_menu);
        menubar.append_submenu(Some("Help"), &help_menu);

        menubar
    };

    app.set_menubar(Some(&menubar));
}


fn create_GUI(application: &gtk::Application) {
    let title = format!("{} {}", APP_DESCRIPTION.unwrap(), APP_VERSION.unwrap());

    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title(&title)
        .default_width(800)
        .default_height(600)
        .show_menubar(true)
        .build();

    let header_bar = gtk::HeaderBar::new();
    window.set_titlebar(Some(&header_bar));

    let new_wallet_button = gtk::Button::new();
    new_wallet_button.set_icon_name("tab-new-symbolic");
    header_bar.pack_start(&new_wallet_button);

    let open_wallet_button = gtk::Button::new();
    open_wallet_button.set_icon_name("document-open-symbolic");
    header_bar.pack_start(&open_wallet_button);

    let save_wallet_button = gtk::Button::new();
    save_wallet_button.set_icon_name("document-save-symbolic");
    header_bar.pack_start(&save_wallet_button);

    let settings_button = gtk::Button::new();
    settings_button.set_icon_name("org.gnome.Settings-symbolic");
    header_bar.pack_end(&settings_button);

    // Create a Stack and a StackSidebar
    let stack = Stack::new();
    let stack_sidebar = StackSidebar::new();
    stack_sidebar.set_stack(&stack);

    // SEED SIDEBAR
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
    entropy_source_box.set_hexpand(true);
    entropy_source_frame.set_hexpand(true);
    
    // Entropy length
    let entropy_length_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let entropy_length_frame = gtk::Frame::new(Some("Entropy length"));
    let valid_entropy_lengths_as_strings: Vec<String> = VALID_ENTROPY_LENGTHS.iter().map(|&x| x.to_string()).collect();
    let valid_entropy_lengths_as_str_refs: Vec<&str> = valid_entropy_lengths_as_strings.iter().map(|s| s.as_ref()).collect();
    let entropy_length_dropdown = gtk::DropDown::from_strings(&valid_entropy_lengths_as_str_refs);
    entropy_length_box.set_hexpand(true);
    entropy_length_frame.set_hexpand(true);
    entropy_length_dropdown.set_selected(4);

    // Mnemonic passphrase
    let mnemonic_passphrase_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let mnemonic_passphrase_frame = gtk::Frame::new(Some("Mnemonic passphrase"));
    let mnemonic_passphrase_text = gtk::Entry::new();
    mnemonic_passphrase_box.set_hexpand(true);
    mnemonic_passphrase_text.set_hexpand(true);
    
    // Generate button
    let generate_wallet_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let generate_wallet_button = gtk::Button::new();
    generate_wallet_button.set_label("Generate");
    generate_wallet_box.set_halign(gtk::Align::Center);

    // Connections
    entropy_source_frame.set_child(Some(&entropy_source_dropdown));
    entropy_length_frame.set_child(Some(&entropy_length_dropdown));

    generate_wallet_box.append(&generate_wallet_button);
    entropy_source_box.append(&entropy_source_frame);
    entropy_length_box.append(&entropy_length_frame);
    entropy_header_first_box.append(&entropy_source_box);
    entropy_header_first_box.append(&entropy_length_box);
    entropy_header_second_box.append(&mnemonic_passphrase_box);
    entropy_header_box.append(&entropy_header_first_box);
    entropy_header_box.append(&entropy_header_second_box);
    entropy_header_box.append(&generate_wallet_box);

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
    let seed_text = gtk::Entry::new();
    seed_text.set_editable(false);

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

    
    // + NEW IMPLEMENTATION
    // F5 is generate new seed

    // clone for coin tab
    let cloned_seed_text = seed_text.clone();
    
    // Generate seed button
    generate_wallet_button.connect_clicked(clone!(
        @strong entropy_source_dropdown,
        @strong entropy_length_dropdown,
        @strong mnemonic_words_text => move |_| {
            let selected_entropy_source_index = entropy_source_dropdown.selected() as usize;
            let selected_entropy_length_index = entropy_length_dropdown.selected() as usize;
            let selected_entropy_source_value = VALID_ENTROPY_SOURCES.get(selected_entropy_source_index);
            let selected_entropy_length_value = VALID_ENTROPY_LENGTHS.get(selected_entropy_length_index);


            println!("source: {}", selected_entropy_source_value.unwrap().to_string());
            println!("Length: {}", selected_entropy_length_value.unwrap());

            // let converted_value: usize = *selected_entropy_length_value as usize;

            let entropy_length = selected_entropy_length_value;
            let mut pre_entropy: String = "".to_string();
            let mut full_entropy: String = "".to_string();

            match selected_entropy_source_value {
                Some(selected_source) => {
                    match selected_source {
                        &"rng" => {
                            if let Some(length) = entropy_length {
                                pre_entropy = generate_entropy_from_rng(&length);
                                // let checksum = calculate_checksum(&result, length);
                            } else {
                                // Handle the case where entropy_length is None
                            }
                        }
                        &"file" => {
                            if let Some(length) = entropy_length {
                                pre_entropy = generate_entropy_from_file(ENTROPY_FILE, *length as usize);
                                // full_entropy = get_full_entropy(&result, &checksum);
                            } else {
                                // Handle the case where entropy_length is None
                            }
                        }
                        _ => {
                            // Handle other cases or do nothing
                        }
                    }
                }
                None => {
                    // Handle the case where selected_entropy_source_value is None
                }
            }
            
            let checksum = calculate_checksum(&pre_entropy, &entropy_length.unwrap());
            full_entropy = get_full_entropy(&pre_entropy, &checksum);
            entropy_text.buffer().set_text(&full_entropy);


            let mnemonic_words = get_mnemonic_from_entropy(&full_entropy);
            mnemonic_words_text.buffer().set_text(&mnemonic_words);


            let passphrase_text = mnemonic_passphrase_text.text().to_string();
            println!("pass: {}", &passphrase_text);

            let seed = create_bip39_seed_from_entropy(&pre_entropy, &passphrase_text);
            seed_text.buffer().set_text(&seed);
        }
    ));

    // Start Seed sidebar
    stack.add_titled(&entropy_main_box, Some("sidebar-seed"), "Seed");
    
    // Sidebar Coin
    let coin_main_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    coin_main_box.set_margin_top(10);
    coin_main_box.set_margin_start(10);
    coin_main_box.set_margin_end(10);
    coin_main_box.set_margin_bottom(10);
    let coin_frame = gtk::Frame::new(Some("Coin"));
    let coin_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    
    // Coins
    let coin_search = gtk4::SearchEntry::new();
    let coin_label = gtk::Label::builder()
        .label("Type coin symbol to start")
        .vexpand(true)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .css_classes(["large-title"])
        .build();


    // NEW IMPLEMENT
    // Derivation path

    let derivation_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let derivation_frame = gtk::Frame::new(Some("Derivation path"));

    // let derivation_dropdown = gtk4::DropDown::new(model, expression)






    let valid_derivation_path_as_string: Vec<String> = VALID_ENTROPY_LENGTHS.iter().map(|&x| x.to_string()).collect();
    let valid_derivation_path_as_str_refs: Vec<&str> = valid_derivation_path_as_string.iter().map(|s| s.as_ref()).collect();
    let derivation_dropdown = gtk::DropDown::from_strings(&valid_derivation_path_as_str_refs);
    // derivation_dropdown.set_hexpand(true);
    // derivation_dropdown.set_hexpand(true);
    // entroderivation_dropdownpy_length_dropdown.set_selected(4);








    // // Entropy source
    // let entropy_source_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    // let entropy_source_frame = gtk::Frame::new(Some("Entropy source"));
    // let valid_entropy_source_as_strings: Vec<String> = VALID_ENTROPY_SOURCES.iter().map(|&x| x.to_string()).collect();
    // let valid_entropy_source_as_str_refs: Vec<&str> = valid_entropy_source_as_strings.iter().map(|s| s.as_ref()).collect();
    // let entropy_source_dropdown = gtk::DropDown::from_strings(&valid_entropy_source_as_str_refs);
    // entropy_source_box.set_hexpand(true);
    // entropy_source_frame.set_hexpand(true);







































    // Master private key
    let master_private_key_frame = gtk::Frame::new(Some("Master private key"));
    let master_private_key_text = gtk::TextView::new();
    master_private_key_text.set_editable(false);



    // Connections
    // Frames (elements)


    coin_frame.set_child(Some(&coin_box));
    coin_frame.set_child(Some(&derivation_box));

    // Box (frame)
    
    // Mainbox (box)
    
    
    
    
    // Box
    
    derivation_box.append(&derivation_frame);
    
    
    derivation_frame.set_child(Some(&derivation_dropdown));
    master_private_key_frame.set_child(Some(&master_private_key_text));
    
    
    coin_box.append(&coin_search);
    coin_box.append(&coin_label);
    coin_main_box.append(&coin_frame);
    coin_main_box.append(&derivation_frame);
    coin_main_box.append(&master_private_key_frame);
    

    // Start: Coins
    stack.add_titled(&coin_main_box, Some("sidebar-coin"), "Coin");



    coin_search.connect_search_changed(clone!(@weak coin_label => move |coin_search| {
        if coin_search.text() != "" {
            let result = check_coin_type(&coin_search.text());
            
            match result {
                Ok(valid_coin_type) => {
                    let label_text = format!("Coin found: {}", &valid_coin_type.to_string());
                    coin_label.set_text(&label_text);
                    // seed = &seed.to_string();
                    println!("seed: {}", cloned_seed_text.text());
                    let master_priv = create_master_private_key(cloned_seed_text.text().to_string());
                    master_private_key_text.buffer().set_text(&master_priv.to_string());
                }
                Err(error_message) => {
                    coin_label.set_text(&error_message.to_string());
                }
            }
        } else {
            coin_label.set_text("Search for a coin symbol");
        }
    }));
















    // // // Create and add sidebar 4
    // // stack.add_titled(&sidebar4, Some("sidebar4"), "Address");







        // CREATE HELP PAGE








    // Create a Box to hold the main content and sidebar
    let main_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    main_content_box.append(&stack_sidebar);
    main_content_box.append(&stack);
    window.set_child(Some(&main_content_box));

    window.present();
}

fn read_csv(file_path: &str) -> Vec<String> {
    let file = File::open(file_path).expect("can not read file");
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
    
    let symbols: Vec<String> = rdr
        .records()
        .filter_map(|record| record.ok())
        .filter_map(|record| record.get(2).map(|s| s.to_string()))
        .collect();

    symbols
}

fn main() {
    // Create a new application
    let application = gtk::Application::builder()
        .application_id("com.github.qr2m")
        .build();
    // application.connect_startup(gtk4_create_main_menu);
    application.connect_activate(create_GUI);

    // When activated, shuts down the application
    let quit = gio::SimpleAction::new("quit", None);
    quit.connect_activate(
        glib::clone!(@weak application => move |_action, _parameter| {
            application.quit();
        }),
    );
    application.set_accels_for_action("app.quit", &["<Primary>Q"]);
    application.add_action(&quit);


    application.run();
}








// OLD DO NOT DELETE


// fn main() -> Result<(), CustomError> {
//     print_program_info();
    
//     // CLI arguments
//     let cli_args = Cli::from_args();
//     process_arguments(&cli_args);

//     // Pre check
//     let entropy_length = check_entropy_length(cli_args.entropy_length.try_into().unwrap())?;
//     let bip_derivation = check_bip_entry(cli_args.derivation_path.try_into().unwrap())?;
//     let address_count = check_address_count_input(&cli_args.address_count.try_into().unwrap())?;
//     let mut seed  = "".to_string();
//     let mut mnemonic_words  = "".to_string();
//     let mut entropy  = "".to_string();

//     if !&cli_args.imported_seed.is_empty() {
//         seed = import_seed(&cli_args.imported_seed.to_string())?; 
//     } else {
//         // Import Mnemonic
//         if !&cli_args.imported_mnemonic.is_empty() {
//             mnemonic_words = import_mnemonic_words(&cli_args.imported_mnemonic, &WORDLIST_FILE)?;
//             // seed = create_bip39_seed_from_mnemonic(&mnemonic_words, &cli_args.passphrase)?;
//         } else {
//             let _entropy_source = check_source_entry(&cli_args.entropy_source)?;

//             // Generate entropy
//             match cli_args.entropy_source.as_str() {
//                 "file" => {
//                     entropy = generate_entropy_from_file(
//                         ENTROPY_FILE,
//                         cli_args.entropy_length.try_into().unwrap(),
//                     )?;
//                 },
//                 "rng" => {
//                     entropy = generate_entropy_from_rng(cli_args.entropy_length)?;
//                 },
//                 _ => println!("error"),
//             }
            
//             // Create full entropy
//             let checksum = calculate_checksum(&entropy, &entropy_length)?;
//             let full_entropy = get_full_entropy(&entropy, &checksum)?;
            
//             // Mnemonic and seed
//             mnemonic_words = get_mnemonic_from_entropy(&full_entropy)?;
//             seed = create_bip39_seed_from_entropy(&entropy, &cli_args.seed_passphrase)?;
//         }
//     }

//     // Master key
//     let master_key = create_master_private_key(&seed)?;

//     // Coin
//     let coin_type = check_coin_type(&cli_args.coin_symbol)?;
//     D3BUG!(log, "Coin index: {:?}", &coin_type);
    
    
//     let derivation_path = create_derivation_path(
//         bip_derivation,
//         coin_type,
//         None,
//         None,
//         // Some(0),
//     )?;
    


//     // Extended private keys
//     let xprivkey = create_extended_private_key(&master_key, &derivation_path)?;
//     let xpubkey = create_extended_public_key(&xprivkey, None)?;

//     let address = create_bitcoin_address(xprivkey, &derivation_path, &address_count)?;
//     // derive_key(&xprivkey, &derivation_path);

//     Ok(())
// }

