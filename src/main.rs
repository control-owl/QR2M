#![allow(non_snake_case)]

// Crates
use rand::Rng;
use sha2::{Digest, Sha256};
use std::{io::{self, Read, Seek}, fs::{self, File}};
use bip39;
use hex;
use structopt::StructOpt;

mod converter;
use converter::{convert_binary_to_string, convert_string_to_binary};


// Arguments
#[derive(StructOpt)]
struct Cli {
    #[structopt(short = "e", long = "esize", default_value = "256")]
    entropy_length: usize,

    #[structopt(short = "d", long = "debug")]
    debug: bool,

    #[structopt(short = "p", long = "password", default_value = "")]
    password: String,
}


// Debugging
macro_rules! D3BUG {
    ($($arg:tt)*) => (
        if Cli::from_args().debug {
            println!($($arg)*);
        }
    );
}


// Global variables
const ENTROPY_FILE: &str = "entropy/binary.qrn";
const WORDLIST_FILE: &str = "lib/bip39-english.txt";
const VALID_ENTROPY_LENGTHS: [u32; 5] = [128, 160, 192, 224, 256];


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

fn main() -> Result<(), std::io::Error> {
    print_program_info();

    // Parse command-line arguments
    let cli_args = Cli::from_args();
    
    // Check if the provided entropy length is valid
    let entropy_length = check_entropy_length(cli_args.entropy_length.try_into().unwrap())?;
    let entropy = read_entropy_from_file(ENTROPY_FILE, cli_args.entropy_length)?;

    // let checksum_lenght = calculate_checksum_length(cli_args.entropy_length);
    let checksum = calculate_checksum(&entropy, &entropy_length)?;
    
    let full_entropy = get_full_entropy(&entropy, &checksum)?;

    let _mnemonic_words = get_mnemonic_from_full_entropy(&full_entropy)?;

    let _seed = create_bip39_seed(&entropy, &cli_args.password);

    Ok(())
}

fn read_entropy_from_file(file_path: &str, entropy_length: usize) -> Result<String, std::io::Error> {
    D3BUG!("----------[Entropy]----------");

    // Open the entropy file
    let file = File::open(file_path)?;
    let mut reader = io::BufReader::new(file);
    D3BUG!("Entropy file: {:?}", file_path);
    
    // Get the file length
    let file_length = reader.seek(io::SeekFrom::End(0))?;
    D3BUG!("Entropy file length: {:?}", file_length);
    
    // Check if file_length is less than entropy_length
    if file_length < entropy_length as u64 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Entropy file too small, or empty",
        ));
    }

    // Randomize reading start point
    let start_point: u64 = if file_length > entropy_length as u64 {
        let max_start = file_length.saturating_sub(entropy_length as u64);
        rand::thread_rng().gen_range(0..max_start)
    } else {
        0
    };
    reader.seek(io::SeekFrom::Start(start_point))?;
    D3BUG!("Random start point: {:?}", start_point);

    // Read entropy from file
    let mut entropy_raw_binary = String::new();
    reader.take(entropy_length as u64).read_to_string(&mut entropy_raw_binary)?;
    println!("Entropy: {:?}", entropy_raw_binary);

    Ok(entropy_raw_binary)
}

fn calculate_checksum(entropy: &String, entropy_length: &u32) -> Result<String, std::io::Error> {
    D3BUG!("----------[Checksum]----------");

    let entropy_binary = convert_string_to_binary(&entropy);
    let hash_raw_binary: String = convert_binary_to_string(&Sha256::digest(&entropy_binary));
    D3BUG!("sha256(entropy): {:?}", hash_raw_binary);

    let checksum_lenght = entropy_length / 32;
    D3BUG!("Checksum length: {:?}", checksum_lenght);

    // Take 1 bit for every 32 bits of the hash 
    let checksum_raw_binary: String = hash_raw_binary.chars().take(checksum_lenght.try_into().unwrap()).collect();
    println!("Checksum: {:?}", checksum_raw_binary);

    Ok(checksum_raw_binary)
}

fn get_full_entropy(entropy: &String, checksum: &String) -> Result<String, std::io::Error> {
    D3BUG!("----------[Final Entropy]----------");

    let full_entropy = format!("{}{}", entropy, checksum);
    D3BUG!("Final entropy: {:?}", full_entropy);

    Ok(full_entropy)
}

fn get_mnemonic_from_full_entropy(final_entropy_binary: &String) -> Result<String, std::io::Error> {
    D3BUG!("----------[Mnemonic]----------");

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
    D3BUG!("Mnemonic numbers: {:?}", mnemonic_decimal);

    let final_mnemonic = mnemonic_words.join(" ");
    println!("Mnemonic words: {:?}", final_mnemonic);
    

    Ok(final_mnemonic)

}

fn create_bip39_seed(entropy: &String, passphrase: &str) -> Result<String, bip39::Error> {
    D3BUG!("----------[Seed]----------");

    // Parse the mnemonic phrase
    let entropy_vector = convert_string_to_binary(&entropy);
    let mnemonic_result = bip39::Mnemonic::from_entropy(&entropy_vector);
    
    // Check if the conversion was successful
    let mnemonic = mnemonic_result?;
    
    // Now you can use the mnemonic to generate the seed
    let seed = bip39::Mnemonic::to_seed(&mnemonic, passphrase);

    // Convert the seed to hexadecimal
    let seed_hex = hex::encode(&seed[..]);
    println!("BIP39 Seed: {:?}", seed_hex);

    Ok(seed_hex)
}

fn check_entropy_length(entropy_length: u32) -> Result<u32, std::io::Error> {
    if !VALID_ENTROPY_LENGTHS.contains(&entropy_length) {
        eprintln!("Error: Invalid entropy_length. Allowed values are: {:?}", VALID_ENTROPY_LENGTHS);
        std::process::exit(2); // or any other non-zero exit code
    }

    Ok(entropy_length)
}
