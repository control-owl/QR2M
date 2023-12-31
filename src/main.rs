// Crates
use std::{io::{self, Read, Seek, Write}, fs::{self, File}, path::Path, vec, str::FromStr};
use structopt::StructOpt;
use hex;
use rand::Rng;
use sha2::{Digest, Sha256};
use bitcoin;
use bip39;
use csv::ReaderBuilder;


// Project files
mod error_handler;
use error_handler::CustomError;
mod converter;


// Global variables
// const ENTROPY_FILE: &str = "entropy/test.qrn";
const ENTROPY_FILE: &str = "entropy/binary.qrn";
const WORDLIST_FILE: &str = "lib/bip39-english.txt";
const VALID_ENTROPY_LENGTHS: [u32; 5] = [128, 160, 192, 224, 256];
const VALID_BIP_DERIVATIONS: [u32; 2] = [32, 44];
// const VALID_BIP_DERIVATIONS: [u32; 5] = [32, 44, 49, 84, 341];
const VALID_MNEMONIC_WORD_COUNT: [u32; 5] = [12, 15, 18, 21, 24];
const VALID_ENTROPY_SOURCES: &'static [&'static str] = &["rng", "file"];

// Debugging log
macro_rules! D3BUG {
    (info, $($arg:tt)*) => (
        if Cli::from_args().verbosity >= 3 {
            println!("\x1b[1;31m{}\x1b[0m", format_args!($($arg)*));
        }
    );
    (log, $($arg:tt)*) => (
        if Cli::from_args().verbosity >= 2 {
            println!("[LOG] {}", format_args!($($arg)*));
        }
    );
    (warning, $($arg:tt)*) => (
        if Cli::from_args().verbosity >= 1 {
            eprintln!("[! WARNING !] {}", format_args!($($arg)*));
        }
    );
    (error, $($arg:tt)*) => (
            eprintln!("\x1b[1;31m[!! ERROR !!] {}\x1b[0m", format_args!($($arg)*))
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
    #[structopt(short = "b", long = "bip", default_value = "44")]
    derivation_path: u32,

    #[structopt(short = "c", long = "coin", default_value = "BTC")]
    coin_symbol: String,

    #[structopt(short = "e", long = "esize", default_value = "256")]
    entropy_length: u32,

    #[structopt(short = "p", long = "passphrase", default_value = "")]
    passphrase: String,
    
    #[structopt(short = "m", long = "import-mnemonic", default_value = "")]
    imported_mnemonic: String,
    
    #[structopt(short = "s", long = "entropy-source", default_value = "rng")]
    entropy_source: String,

    #[structopt(short = "v", long = "verbosity", default_value = "0")]
    verbosity: u32,

}


fn print_program_info() {
    let description = option_env!("CARGO_PKG_DESCRIPTION").unwrap_or_default();
    let version = option_env!("CARGO_PKG_VERSION").unwrap_or_default();
    let authors = option_env!("CARGO_PKG_AUTHORS").unwrap_or_default();
    
    
    println!(" ██████╗ ██████╗ ██████╗ ███╗   ███╗");
    println!("██╔═══██╗██╔══██╗╚════██╗████╗ ████║");
    println!("██║   ██║██████╔╝ █████╔╝██╔████╔██║");
    println!("██║▄▄ ██║██╔══██╗██╔═══╝ ██║╚██╔╝██║");
    println!("╚██████╔╝██║  ██║███████╗██║ ╚═╝ ██║");
    println!(" ╚══▀▀═╝ ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝");
    println!("{} ({})\n{}\n", description, version, authors);
}


// use std::env;

fn main() -> Result<(), CustomError> {
    print_program_info();

    // Parse CLI arguments
    let cli_args = Cli::from_args();
    // Check provided arguments
    let entropy_length = check_entropy_length(cli_args.entropy_length.try_into().unwrap())?;
    let _bip_derivation = check_bip_entry(cli_args.derivation_path.try_into().unwrap())?;
    
    let mut seed  = "".to_string();
    let mut mnemonic_words  = "".to_string();
    let mut entropy  = "".to_string();

    // Import mnemonic
    if !&cli_args.imported_mnemonic.is_empty() {
        mnemonic_words = import_mnemonic_words(&cli_args.imported_mnemonic, &WORDLIST_FILE)?;
        seed = create_bip39_seed_from_mnemonic(&mnemonic_words, &cli_args.passphrase)?;
    } else {
        match cli_args.entropy_source.as_str() {
            "file" => {
                entropy = read_entropy_from_file(ENTROPY_FILE, cli_args.entropy_length.try_into().unwrap())?;
            },
            "rng" => {
                entropy = generate_entropy_from_rng(cli_args.entropy_length)
            },
            _ => println!("error"),
        }
        
        // Get entropy
        let checksum = calculate_checksum(&entropy, &entropy_length)?;
        let full_entropy = get_full_entropy(&entropy, &checksum)?;
        
        // Mnemonic
        mnemonic_words = get_mnemonic_from_entropy(&full_entropy)?;
        seed = create_bip39_seed_from_entropy(&entropy, &cli_args.passphrase)?;
    }

    // Master key
    let master_key = create_master_private_key(&seed)?;

    // Coin
    let mut coin_type: u32 = 0;
    match check_coin_type(&cli_args.coin_symbol) {
        Ok(index) => {
            coin_type = index;
            D3BUG!(log, "Coin type ID: \"{}\"", coin_type)
        },
        Err(err) => D3BUG!(error, "{:?}", err),
    }

    // Childrens
    let derivation_path = create_derivation_path(Some(cli_args.derivation_path), Some(coin_type), Some(0), None, None)?;
    let _child_key = create_account_master_key(&master_key, &derivation_path);

    Ok(())
}

fn generate_entropy_from_rng(length: u32) -> String {
    let mut rng = rand::thread_rng();

    let binary_string: String = (0..length)
        .map(|_| rng.gen_range(0..=1))
        .map(|bit| char::from_digit(bit, 10).unwrap())
        .collect();
    D3BUG!(output, "RNG entropy: {:?}", binary_string.to_string());
    binary_string
}

fn read_entropy_from_file(file_path: &str, entropy_length: usize) -> Result<String, CustomError> {
    D3BUG!(info, "Entropy:");

    // Open the entropy file
    let file = File::open(file_path)?;
    let mut reader = io::BufReader::new(file);
    D3BUG!(log, "Entropy file: {:?}", file_path);
    
    // Get the file length
    let file_length = reader.seek(io::SeekFrom::End(0))?;
    D3BUG!(log, "Entropy file length: \"{:?}\"", file_length);
    
    // Check if file_length is less than entropy_length
    if file_length < entropy_length as u64 {
        let error_msg: CustomError = CustomError::FileTooSmall(entropy_length.to_string());
        D3BUG!(error, "{}", error_msg);
        return Err(CustomError::InvalidEntropyLength(file_length.to_string()))
    }

    // Randomize reading start point
    let start_point: u64 = if file_length > entropy_length as u64 {
        let max_start = file_length.saturating_sub(entropy_length as u64);
        rand::thread_rng().gen_range(0..max_start)
    } else {
        0
    };
    reader.seek(io::SeekFrom::Start(start_point))?;
    D3BUG!(log, "Random start point: \"{:?}\"", start_point);

    // Read entropy from file
    let mut entropy_raw_binary = String::new();
    reader.take(entropy_length as u64).read_to_string(&mut entropy_raw_binary)?;
    D3BUG!(output, "Entropy: {:?}", entropy_raw_binary);

    Ok(entropy_raw_binary)
}

fn calculate_checksum(entropy: &String, entropy_length: &u32) -> Result<String, CustomError> {
    D3BUG!(info, "Checksum:");

    let entropy_binary = converter::convert_string_to_binary(&entropy);
    let hash_raw_binary: String = converter::convert_binary_to_string(&Sha256::digest(&entropy_binary));
    D3BUG!(log, "sha256(entropy): {:?}", hash_raw_binary);

    let checksum_lenght = entropy_length / 32;
    D3BUG!(log, "Checksum length: \"{:?}\"", checksum_lenght);

    // Take 1 bit for every 32 bits of the hash 
    let checksum_raw_binary: String = hash_raw_binary.chars().take(checksum_lenght.try_into().unwrap()).collect();
    D3BUG!(output, "Checksum: {:?}", checksum_raw_binary);

    Ok(checksum_raw_binary)
}

fn get_full_entropy(entropy: &String, checksum: &String) -> Result<String, CustomError> {
    D3BUG!(info, "Final Entropy:");

    let full_entropy = format!("{}{}", entropy, checksum);
    D3BUG!(output, "Final entropy: {:?}", full_entropy);

    Ok(full_entropy)
}

fn get_mnemonic_from_entropy(final_entropy_binary: &String) -> Result<String, CustomError> {
    D3BUG!(info, "Mnemonic:");

    // Split the final entropy into groups of 11 bits
    let chunks: Vec<String> = final_entropy_binary.chars().collect::<Vec<char>>().chunks(11).map(|chunk| chunk.iter().collect()).collect();

    // Convert each chunk to decimal numbers
    let mnemonic_decimal: Vec<u32> = chunks.iter().map(|chunk| u32::from_str_radix(chunk, 2).unwrap()).collect();

    // Read the file containing mnemonic words
    let mnemonic_file_content = fs::read_to_string(WORDLIST_FILE)?;
    let mnemonic_words: Vec<&str> = mnemonic_file_content.lines().collect();

    // Map decimal numbers to Bitcoin mnemonic words
    let mnemonic_words: Vec<&str> = mnemonic_decimal.iter().map(|&decimal| {
        // Ensure the decimal number is withilengthn the valid range
        if (decimal as usize) < mnemonic_words.len() {
            mnemonic_words[decimal as usize]
        } else {
            // Handle the case where the decimal number is out of range
            "INVALID_WORD"
        }
    }).collect();
    D3BUG!(log, "Mnemonic numbers: {:?}", mnemonic_decimal);

    let final_mnemonic = mnemonic_words.join(" ");
    D3BUG!(output, "Mnemonic words: {:?}", final_mnemonic);
    

    Ok(final_mnemonic)

}

fn create_bip39_seed_from_entropy(entropy: &String, passphrase: &str) -> Result<String, CustomError> {
    D3BUG!(info, "Seed:");
    
    // Parse the mnemonic phrase
    let entropy_vector = converter::convert_string_to_binary(&entropy);
    let mnemonic_result = bip39::Mnemonic::from_entropy(&entropy_vector);
    
    // Check if the conversion was successful
    let mnemonic = mnemonic_result?;
    
    // Now you can use the mnemonic to generate the seed
    let seed = bip39::Mnemonic::to_seed(&mnemonic, passphrase);

    // Convert the seed to hexadecimal
    let seed_hex = hex::encode(&seed[..]);
    D3BUG!(output, "BIP39 Passphrase: {:?}", passphrase);
    D3BUG!(output, "BIP39 Seed (hex): {:?}", seed_hex);

    Ok(seed_hex)
}

fn create_bip39_seed_from_mnemonic(mnemonic: &String, passphrase: &str) -> Result<String, CustomError> {
    D3BUG!(info, "BIP39 Seed:");

    let mnemonic_result = bip39::Mnemonic::from_str(&mnemonic);
    let mnemonic = mnemonic_result?;
    let seed = bip39::Mnemonic::to_seed(&mnemonic, passphrase);

    let seed_hex = hex::encode(&seed[..]);

    D3BUG!(output, "BIP39 Passphrase: {:?}", passphrase);
    D3BUG!(output, "BIP39 Seed (hex): {:?}", seed_hex);

    Ok(seed_hex)
}

fn check_entropy_length(entropy_length: u32) -> Result<u32, CustomError> {
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

    Ok(entropy_length)
}

fn check_bip_entry(bip_entry: u32) -> Result<u32, CustomError> {
    if !VALID_BIP_DERIVATIONS.contains(&bip_entry) {
        // D3BUG!(error, "Invalid BIP. Allowed values are: {:?}", VALID_BIP_DERIVATIONS);
        // std::process::exit(2); // or any other non-zero exit code

        let allowed_values = VALID_BIP_DERIVATIONS
            .iter()
            .map(|&x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        let error_msg: CustomError = CustomError::InvalidBipEntry(allowed_values);
        D3BUG!(error, "{}", error_msg);
        return Err(CustomError::InvalidBipEntry(bip_entry.to_string()))
    }

    Ok(bip_entry)
}

fn create_master_private_key(seed_hex: &str) -> Result<bitcoin::bip32::Xpriv, CustomError> {
    D3BUG!(info, "Master Key:");

    // Convert hex seed to binary
    let seed = hex::decode(&seed_hex).expect("Failed to decode seed hex");

    // Create a master key from the seed
    let master_key = bitcoin::bip32::Xpriv::new_master(bitcoin::Network::Bitcoin, &seed).expect("Failed to derive master key");
    D3BUG!(output, "BIP32 Master Private Key (xpriv): \"{}\"", master_key); 

    Ok(master_key)
}

fn check_coin_type(coin_symbol: &str) -> Result<u32, Box<dyn std::error::Error>> {
    D3BUG!(info, "Coin:");

    D3BUG!(output, "Desired coin: {:?}", &coin_symbol);
    let path = Path::new("lib/bip44-coin_type.csv");
    let file = File::open(path)?;
    
    let mut rdr = ReaderBuilder::new().from_reader(file);
    let mut matching_entries: Vec<CoinType> = Vec::new();
    
    // Iterate over CSV records and check for the coin symbol
    for record in rdr.records() {
        let record = record?;
        let index: u32 = record[0].parse()?;
        let path: u32 = u32::from_str_radix(record[1].trim_start_matches("0x"), 16)?;
        let symbol = record[2].to_string();
        let coin_name = record[3].to_string();
        
        if symbol == coin_symbol {
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
        return Ok(0);
    
    }
    
    // If there are multiple matching entries, prompt the user to choose
    if matching_entries.len() > 1 {
        println!("Multiple entries found for symbol {:?}. Please choose one:", coin_symbol);
        for (i, entry) in matching_entries.iter().enumerate() {
            D3BUG!(output, "{}: {:?}", i + 1, entry);
        }
        
        print!("Enter the index of the desired entry: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let chosen_index: usize = input.trim().parse()?;
        if chosen_index > 0 && chosen_index <= matching_entries.len() {
            return Ok(matching_entries[chosen_index - 1].index);
        } else {
            return Ok(0);
        }
    } else {
        D3BUG!(log, "Coin {:?} found", &coin_symbol);
    }

    // If there is only one matching entry, return its index
    Ok(matching_entries[0].index)
}

fn create_derivation_path(
    purpose: Option<u32>,
    coin_type: Option<u32>,
    account: Option<u32>,
    change: Option<u32>,
    address_index: Option<u32>,
) -> Result<Vec<bitcoin::bip32::ChildNumber>, bitcoin::bip32::Error> {
    D3BUG!(info, "Derivation Path:");
    

    let bip = purpose;

    let purpose = bitcoin::bip32::ChildNumber::from_hardened_idx(purpose.unwrap()).expect("Invalid child number");
    let coin_type = bitcoin::bip32::ChildNumber::from_hardened_idx(coin_type.unwrap()).expect("Invalid child number"); // 1 for Bitcoin, 0 for Bitcoin Testnet
    let account = bitcoin::bip32::ChildNumber::from_hardened_idx(account.unwrap_or(0)).expect("Invalid child number");
    let change = bitcoin::bip32::ChildNumber::from_hardened_idx(change.unwrap_or(0)).expect("Invalid child number"); // 0 for external addresses (receive), 1 for internal addresses (change)
    let _address_index = bitcoin::bip32::ChildNumber::from_normal_idx(address_index.unwrap_or(0)).expect("Invalid child number");
    
    let derivation = match bip {
        Some(32) => vec![account, change],
        Some(44) => vec![purpose, coin_type, account],
        // Some(49) => vec![purpose, coin_type, account],
        _ => {
            vec![]
        }
    };

    let mut path = String::from("m");

    for child_number in &derivation {
        path.push_str(&format!("/{}", child_number.to_string()));
    }
    D3BUG!(output, "Derivation Path: {:?}", &path);


    Ok(derivation)
}

fn create_account_master_key(master: &bitcoin::bip32::Xpriv, derivation: &Vec<bitcoin::bip32::ChildNumber>) -> Result<bitcoin::bip32::Xpriv, bitcoin::bip32::Error> {
    D3BUG!(info, "Child Key:");
    
    let secp = bitcoin::secp256k1::Secp256k1::new();
    
    let child_key = master
    .derive_priv(&secp, &derivation)
    .expect("Failed to derive account key");

D3BUG!(output, "Account Extended Private Key: \"{}\"", &child_key);
Ok(child_key)

}

fn import_mnemonic_words(mnemonic: &str, wordlist_path: &str) -> Result<String, CustomError> {
    D3BUG!(info, "Importing mnemonic:");

    let mnemonic_words: Vec<&str> = mnemonic.split_whitespace().collect();

    // Check if the number of words is valid
    if !VALID_MNEMONIC_WORD_COUNT.contains(&(mnemonic_words.len() as u32)) {
        let allowed_values = VALID_MNEMONIC_WORD_COUNT
            .iter()
            .map(|&x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        let error_msg: CustomError = CustomError::InvalidMnemonicWordCount(allowed_values);
        D3BUG!(error, "{}", error_msg);
        return Err(CustomError::InvalidMnemonicWordCount(mnemonic_words.len().to_string()))
        // call_error(error_msg, mnemonic_words.len().to_string())?;

    } else {
        D3BUG!(log, "Imported mnemonic word count {:?}", &mnemonic_words.len());
    }

    // Load the wordlist file
    let wordlist_content = match fs::read_to_string(wordlist_path) {
        Ok(content) => content,
        Err(_) => {
            let error_msg: CustomError = CustomError::WordlistReadError;
            D3BUG!(error, "{}", error_msg);
            return Err(CustomError::WordlistReadError)
        }
    };

    // Check if every word in the mnemonic is in the wordlist
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
use rand::RngCore;
fn get_entropy_from_rng(entropy_size: &u32) -> Result<String, CustomError>{

    // Create a vector to store random bytes
    let mut entropy_bytes = vec![0; *entropy_size as usize];

    // Generate random bytes
    rand::thread_rng().fill_bytes(&mut entropy_bytes);

    // Convert bytes to a hexadecimal string
    let entropy_string = entropy_bytes
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>();

    Ok(entropy_string)

}

fn call_error<T>(error_type: CustomError, _value: T) -> Result<(), CustomError>
where
    T: std::fmt::Display,
{
    D3BUG!(error, "{}", error_type);
    Err(error_type)
}
