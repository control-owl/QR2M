#![allow(non_snake_case)]

use rand::Rng;
use sha2::{Digest, Sha256};
use std::io::{self, Read, Seek};
use std::fs::{self, File};
use std::{error, fmt};


// 
// Extra error handling
// 
#[derive(Debug)]
struct ErrorHandler {
    message: String,
}

impl ErrorHandler {
    fn new(message: &str) -> Self {
        ErrorHandler {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ErrorHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for ErrorHandler {}

impl From<io::Error> for ErrorHandler {
    fn from(error: io::Error) -> Self {
        ErrorHandler::new(&format!("IO Error: {}", error))
    }
}


// 
// Global variables
// 
const ENTROPY_FILE: &str = "entropy/binary.qrn";
const WORDLIST_FILE: &str = "lib/bip39-english.txt";


// 
// Main code
// 
fn main() -> Result<(), ErrorHandler> {
    let entropy = select_entropy_from_file(ENTROPY_FILE)?;
    let checksum = calculate_checksum(&entropy)?;

    let entropy_final = get_full_entropy(&entropy, &checksum)?;
    let mnemonic_words = get_mnemonic_from_full_entropy(&entropy_final)?;

    Ok(())
}

fn select_entropy_from_file(file_path: &str) -> Result<Vec<u8>, ErrorHandler> {
    println!("----------[Entropy]----------");
    
    // Open the entropy file
    let file = File::open(file_path)?;
    let mut reader = io::BufReader::new(file);
    
    // Get the file length
    let file_length = reader.seek(io::SeekFrom::End(0))?;
    println!("Entropy file length: {}", file_length);
    
    // Set a random start point
    let mut rng = rand::thread_rng();
    let start_point: u64 = rng.gen_range(0..(file_length.saturating_sub(256)));
    println!("Random start point: {}", start_point);

    // Seek to the random start point
    reader.seek(io::SeekFrom::Start(start_point))?;
    
    // Read only 256 characters
    let mut entropy_ascii = String::new();
    reader.take(256).read_to_string(&mut entropy_ascii)?;

    // Convert the binary string to a Vec<u8>
    let byte_vec: Vec<u8> = entropy_ascii
    .chars()
    .collect::<Vec<char>>()
        .chunks(8)
        .map(|chunk| chunk.iter().fold(0, |acc, &bit| (acc << 1) | (bit as u8 - '0' as u8)))
        .collect();
        
    println!("Entropy binary: {}", entropy_ascii);
    println!("Entropy vector: {:?}", byte_vec);

    Ok(byte_vec)
}

fn calculate_checksum(entropy_ascii: &Vec<u8>) -> Result<String, ErrorHandler> {
    println!("----------[Checksum]----------");

    // Calculate SHA256 hash of entropy_ascii directly
    let hash = Sha256::digest(&entropy_ascii);
    
    // Convert the entire hash to UTF-8 format
    // let hash_utf8: String = hash.iter().map(|byte| format!("{:02x}", byte)).collect();
    // println!("Hash in UTF-8 format: {}", hash_utf8);
    
    // Convert the entire hash to ASCII format using '1' and '0'
    let hash_ascii: String = hash
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |i| ((byte >> i) & 1).to_string()))
        .collect();
        
    // Take the first 8 bits of the hash and convert each bit to ASCII
    let checksum_ascii: String = hash_ascii.chars().take(8).collect();
        
    println!("Entropy hash binary: {}", hash_ascii);
    println!("Entropy hash vector: {:?}", hash);
    println!("Checksum: {}", checksum_ascii);

    Ok(checksum_ascii)
}

fn get_full_entropy(entropy: &Vec<u8>, checksum_ascii: &String) -> Result<String, ErrorHandler> {
    println!("----------[Final Entropy]----------");

    // Convert the entire entropy to ASCII format using '1' and '0'
    let entropy_ascii: String = entropy
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |i| ((byte >> i) & 1).to_string()))
        .collect();

    // Concatenate entropy_ascii and checksum_ascii into final_entropy_ascii
    let final_entropy_ascii = format!("{}{}", entropy_ascii, checksum_ascii);

    println!("Final entropy binary: {}", final_entropy_ascii);

    Ok(final_entropy_ascii)
}

fn get_mnemonic_from_full_entropy(final_entropy_ascii: &str) -> Result<(), ErrorHandler> {
    println!("----------[Mnemonic]----------");

    // Split the final entropy into groups of 11 bits
    let chunks: Vec<String> = final_entropy_ascii.chars().collect::<Vec<char>>().chunks(11).map(|chunk| chunk.iter().collect()).collect();

    // Convert each chunk to decimal numbers
    let mnemonic_decimal: Vec<u32> = chunks.iter().map(|chunk| u32::from_str_radix(chunk, 2).unwrap()).collect();

    // Read the file containing all 2048 mnemonic words
    let mnemonic_file_content = fs::read_to_string(WORDLIST_FILE)?;
    let mnemonic_words: Vec<&str> = mnemonic_file_content.lines().collect();

    // Map decimal numbers to Bitcoin mnemonic words
    let mnemonic_words: Vec<&str> = mnemonic_decimal.iter().map(|&decimal| {
        // Ensure the decimal number is within the valid range
        if (decimal as usize) < mnemonic_words.len() {
            mnemonic_words[decimal as usize]
        } else {
            // Handle the case where the decimal number is out of range
            "INVALID_WORD"
        }
    }).collect();

    println!("Mnemonic numbers: {:?}", mnemonic_decimal);
    println!("Mnemonic words: {:?}", mnemonic_words.join(" "));

    Ok(())

}

// WORKING