// Crates
use std::{io::{self, Read, Seek, Write}, fs::{self, File}, path::Path, vec, str::FromStr, ops::Index};
use structopt::StructOpt;
use hex;
use rand::{Rng, RngCore};
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
const ENTROPY_FILE: &str = "./entropy/binary.qrn";
const WORDLIST_FILE: &str = "./lib/bip39-english.txt";
const VALID_ENTROPY_LENGTHS: [u32; 5] = [128, 160, 192, 224, 256];
const VALID_BIP_DERIVATIONS: [u32; 3] = [32, 44, 84];
// const VALID_BIP_DERIVATIONS: [u32; 5] = [32, 44, 49, 84, 341];
const VALID_MNEMONIC_WORD_COUNT: [u32; 5] = [12, 15, 18, 21, 24];
const VALID_ENTROPY_SOURCES: &'static [&'static str] = &["rng", "file"];


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
    address_count: String,

    #[structopt(short = "b", long = "bip", default_value = "44")]
    derivation_path: u32,

    #[structopt(short = "c", long = "coin", default_value = "BTC")]
    coin_symbol: String,

    #[structopt(short = "e", long = "entropy-source", default_value = "rng")]
    entropy_source: String,

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

fn main() -> Result<(), CustomError> {
    print_program_info();

    // CLI arguments
    let cli_args = Cli::from_args();

    // Pre check
    let entropy_length = check_entropy_length(cli_args.entropy_length.try_into().unwrap())?;
    let bip_derivation = check_bip_entry(cli_args.derivation_path.try_into().unwrap())?;
    let address_count = check_address_count_input(&cli_args.address_count)?;
    

    // Preparing
    let mut seed  = "".to_string();
    let mut mnemonic_words  = "".to_string();
    let mut entropy  = "".to_string();

    if !&cli_args.imported_seed.is_empty() {
        seed = import_seed(&cli_args.imported_seed.to_string())?; 
    } else {
        // Import Mnemonic
        if !&cli_args.imported_mnemonic.is_empty() {
            mnemonic_words = import_mnemonic_words(&cli_args.imported_mnemonic, &WORDLIST_FILE)?;
            // seed = create_bip39_seed_from_mnemonic(&mnemonic_words, &cli_args.passphrase)?;
        } else {
            
            let _entropy_source = check_source_entry(&cli_args.entropy_source)?;

            // Generate entropy
            match cli_args.entropy_source.as_str() {
                "file" => {
                    entropy = read_entropy_from_file(
                        ENTROPY_FILE,
                        cli_args.entropy_length.try_into().unwrap(),

                    )?;
                },
                "rng" => {
                    entropy = generate_entropy_from_rng(cli_args.entropy_length)?;
                },
                _ => println!("error"),
            }
            
            // Create full entropy
            let checksum = calculate_checksum(&entropy, &entropy_length)?;
            let full_entropy = get_full_entropy(&entropy, &checksum)?;
            
            // Mnemonic and seed
            mnemonic_words = get_mnemonic_from_entropy(&full_entropy)?;
            seed = create_bip39_seed_from_entropy(&entropy, &cli_args.seed_passphrase)?;
        }
    }


    // Master key
    let master_key = create_master_private_key(&seed)?;

    // Coin
    let coin_type = check_coin_type(&cli_args.coin_symbol)?;
    D3BUG!(log, "Coin index: {:?}", &coin_type);
    
    // Extended private keys

    let derivation_path = create_derivation_path(
        bip_derivation,
        coin_type,
        Some(0),
        Some(0),
    )?;

    let xprivkey = create_extended_private_key(&master_key, &derivation_path)?;

    let xpubkey = create_extended_public_key(&xprivkey, None)?;

    let address = create_p2pkh_address(xprivkey, &derivation_path, &address_count)?;
    Ok(())
}

fn generate_entropy_from_rng(length: u32) -> Result<String, CustomError> {
    D3BUG!(info, "Entropy (RNG):");
    
    let mut rng = rand::thread_rng();
    let binary_string: String = (0..length)
        .map(|_| rng.gen_range(0..=1))
        .map(|bit| char::from_digit(bit, 10).unwrap())
        .collect();

    D3BUG!(output, "RNG entropy: {:?}", binary_string.to_string());
    Ok(binary_string)
}

fn check_address_count_input(count: &str) -> Result<u32, CustomError> {
    D3BUG!(info, "Checking address count input:");

    match count.parse::<u32>() {
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

fn read_entropy_from_file(file_path: &str, entropy_length: usize) -> Result<String, CustomError> {
    D3BUG!(info, "Entropy from file:");
    
    let file = File::open(file_path)?;
    let mut reader = io::BufReader::new(file);
    D3BUG!(log, "Entropy file: {:?}", file_path);
    
    let file_length = reader.seek(io::SeekFrom::End(0))?;
    D3BUG!(log, "Entropy file length: \"{:?}\"", file_length);
    
    if file_length < entropy_length as u64 {
        let error_msg: CustomError = CustomError::FileTooSmall(entropy_length.to_string());
        D3BUG!(error, "{}", error_msg);
        return Err(CustomError::InvalidEntropyLength(file_length.to_string()))
    }
    
    let mut start_point: u64 = 0;

    if file_length > entropy_length as u64 {
        let max_start = file_length.saturating_sub(entropy_length as u64);
        start_point = rand::thread_rng().gen_range(0..=max_start);
    }

    reader.seek(io::SeekFrom::Start(start_point))?;
    D3BUG!(log, "Random start point: \"{:?}\"", start_point);

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
    Ok(final_mnemonic)

}

fn create_bip39_seed_from_entropy(entropy: &String, passphrase: &str) -> Result<String, CustomError> {
    D3BUG!(info, "Seed:");
    
    let entropy_vector = converter::convert_string_to_binary(&entropy);
    let mnemonic_result = bip39::Mnemonic::from_entropy(&entropy_vector);
    let mnemonic = mnemonic_result?;
    let seed = bip39::Mnemonic::to_seed(&mnemonic, passphrase);
    let seed_hex = hex::encode(&seed[..]);

    D3BUG!(output, "BIP39 passphrase: {:?}", passphrase);
    D3BUG!(output, "BIP39 seed: {:?}", seed_hex);

    Ok(seed_hex)
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

fn create_master_private_key(seed_hex: &str) -> Result<bitcoin::bip32::Xpriv, CustomError> {
    D3BUG!(info, "Master key:");

    let seed = hex::decode(&seed_hex).expect("Failed to decode seed hex");
    let master_key = bitcoin::bip32::Xpriv::new_master(bitcoin::Network::Bitcoin, &seed).expect("Failed to derive master key");

    D3BUG!(output, "BIP32 Master private key: \"{}\"", master_key); 
    Ok(master_key)
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
        D3BUG!(error, "{}", error_msg);
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
) -> Result<Vec<bitcoin::bip32::ChildNumber>, bitcoin::bip32::Error> {
    D3BUG!(info, "Derivation path:");
    
    let purpose = bitcoin::bip32::ChildNumber::from_hardened_idx(cli_bip)?;
    let coin_type = bitcoin::bip32::ChildNumber::from_hardened_idx(coin_type)?;
    let account = bitcoin::bip32::ChildNumber::from_hardened_idx(account.unwrap_or(0)).expect("Invalid child number");
    let change = bitcoin::bip32::ChildNumber::from_hardened_idx(change.unwrap_or(0)).expect("Invalid child number");
    
    let derivation = match cli_bip {
        32 => vec![account, change],
        44 => vec![purpose, coin_type, account, change],
        84 => vec![purpose, coin_type, account, change],
        // Some(49) => vec![purpose, coin_type, account],
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
    D3BUG!(info, "Extended private key");

    let secp = bitcoin::secp256k1::Secp256k1::new();
    let extended_key = master
        .derive_priv(&secp, derivation)
        .expect("Failed to derive extended private key");

    D3BUG!(output, "Extended private key: \"{}\"", extended_key);
    Ok(extended_key)
}

fn create_extended_public_key(
    xprv: &bitcoin::bip32::Xpriv,
    index: Option<&u32>
) -> Result<bitcoin::bip32::Xpub, CustomError> {
    D3BUG!(info, "Extended public key");

    let secp = bitcoin::secp256k1::Secp256k1::new();
    let xpubkey = bitcoin::bip32::Xpub::from_priv(&secp, &xprv);
    
    D3BUG!(output, "Extended public key: {:?}", xpubkey.to_string());

    
    Ok(xpubkey)
}


fn create_p2pkh_address(
    xprv: bitcoin::bip32::Xpriv,
    derivation: &Vec<bitcoin::bip32::ChildNumber>,
    count: &u32
) -> Result<Vec<bitcoin::Address>, CustomError> {
    D3BUG!(info, "p2pkh addresses:");

    let secp = bitcoin::secp256k1::Secp256k1::new();

    let mut addresses = Vec::new();

    for index in 0..*count {
        let child = bitcoin::bip32::ChildNumber::from_hardened_idx(index)?;

        let child_xprv = xprv.derive_priv(&secp, &child)?;
        let child_pubkey = child_xprv.to_priv().public_key(&secp);

        let address = bitcoin::Address::p2pkh(&child_pubkey, bitcoin::Network::Bitcoin);

        let mut path = String::from("m");
        for child_number in derivation {
            path.push_str(&format!("/{}", child_number.to_string()));
        }
        
        D3BUG!(output, "{}/{}: {:?}", &path, child, address.to_string());
        
        addresses.push(address);
    }

    Ok(addresses)
}