#![allow(non_snake_case)]

use rand::Rng;
use sha2::{Digest, Sha256};
use std::io::{self, Read, Seek};
use std::fs::{self, File};
use bip39;
use hex;


// 
// Global variables
// 
const ENTROPY_FILE: &str = "entropy/test.txt";
// const ENTROPY_FILE: &str = "entropy/binary.qrn";
const WORDLIST_FILE: &str = "lib/bip39-english.txt";


// 
// Main code
// 
fn main() -> Result<(), std::io::Error> {
    println!("\n-----------------------------");
    println!("------------[QR2M]-----------");
    println!("-----------------------------\n");

    // Working
    let entropy = select_entropy_from_file(ENTROPY_FILE)?;
    let checksum = calculate_checksum(&entropy)?;
    let entropy_final = get_full_entropy(&entropy, &checksum)?;
    let _mnemonic_words = get_mnemonic_from_full_entropy(&entropy_final)?;

    let _seed = create_bip39_seed(&entropy, "");

    Ok(())
}

fn convert_binary_to_string(binary: &[u8]) -> String {
    binary
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |i| ((byte >> i) & 1).to_string()))
        .collect()
}

fn select_entropy_from_file(file_path: &str) -> Result<Vec<u8>, std::io::Error> {
    println!("----------[Entropy]----------");
    
    // Open the entropy file
    let file = File::open(file_path)?;
    let mut reader = io::BufReader::new(file);
    println!("Entropy file: {:?}", file_path);
    
    // Get the file length
    let file_length = reader.seek(io::SeekFrom::End(0))?;
    println!("Entropy file length: {:?}", file_length);
    
    // Adjust the range based on file length
    let start_point: u64 = if file_length > 256 {
        let max_start = file_length.saturating_sub(256);
        rand::thread_rng().gen_range(0..max_start)
    } else {
        if file_length < 256 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Entropy file too small, or empty"));
        } else {
            0
        }
    };

    println!("Random start point: {:?}", start_point);
    
    // Seek to the random start point
    reader.seek(io::SeekFrom::Start(start_point))?;
    
    // Read only 256 characters
    let mut entropy_raw_binary = String::new();
    reader.take(256).read_to_string(&mut entropy_raw_binary)?;
    println!("Entropy raw binary: {:?}", entropy_raw_binary);

    // Convert the binary string to a Vec<u8>
    let entropy_binary: Vec<u8> = entropy_raw_binary
        .chars()
        .collect::<Vec<char>>()
        .chunks(8)
        .map(|chunk| chunk.iter().fold(0, |acc, &bit| (acc << 1) | (bit as u8 - '0' as u8)))
        .collect();
    println!("Entropy binary: {:?}", entropy_binary);

    Ok(entropy_binary)
}

fn calculate_checksum(entropy_binary: &Vec<u8>) -> Result<Vec<u8>, std::io::Error> {
    println!("----------[Checksum]----------");

    // Calculate SHA256 hash of entropy_binary directly
    let hash = Sha256::digest(&entropy_binary);
    println!("Entropy binary sha256 hash: {:?}", hash);
    
    // Convert the entire hash to string using '1' and '0'
    let hash_raw_binary: String = hash
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |i| ((byte >> i) & 1).to_string()))
        .collect();
    println!("Entropy hash raw binary: {:?}", hash_raw_binary);

    // Take the first 8 bits of the hash
    let checksum_raw_binary: String = hash_raw_binary.chars().take(8).collect();
    println!("Checksum raw binary: {:?}", checksum_raw_binary);


    let checksum_binary: Vec<u8> = checksum_raw_binary
        .chars()
        .collect::<Vec<char>>()
        .chunks(8)
        .map(|chunk| chunk.iter().fold(0, |acc, &bit| (acc << 1) | (bit as u8 - '0' as u8)))
        .collect();
    println!("checksum_binary binary: {:?}", checksum_binary);


    Ok(checksum_binary)
}

fn get_full_entropy(entropy: &Vec<u8>, checksum: &Vec<u8>) -> Result<Vec<u8>, std::io::Error> {
    println!("----------[Final Entropy]----------");

    // Concatenate entropy and checksum
    let mut final_entropy_binary = Vec::with_capacity(entropy.len() + checksum.len());
    final_entropy_binary.extend_from_slice(entropy);
    final_entropy_binary.extend_from_slice(checksum);
    println!("Final entropy binary: {:?}", final_entropy_binary);
    
    // Display vector size in bits
    let vector_size_bits = final_entropy_binary.len() * 8;
    println!("Final entropy binary size: {:?} bits", vector_size_bits);

    // Convert the entire entropy to raw format using '1' and '0'
    let final_entropy_raw: String = convert_binary_to_string(&final_entropy_binary);
    println!("Final entropy raw: {:?}", final_entropy_raw);


    Ok(final_entropy_binary)
}

fn get_mnemonic_from_full_entropy(final_entropy_binary: &Vec<u8>) -> Result<String, std::io::Error> {
    println!("----------[Mnemonic]----------");

    let final_entropy_raw: String = convert_binary_to_string(&final_entropy_binary);

    // Split the final entropy into groups of 11 bits
    let chunks: Vec<String> = final_entropy_raw.chars().collect::<Vec<char>>().chunks(11).map(|chunk| chunk.iter().collect()).collect();

    // Convert each chunk to decimal numbers
    let mnemonic_decimal: Vec<u32> = chunks.iter().map(|chunk| u32::from_str_radix(chunk, 2).unwrap()).collect();

    // Read the file containing mnemonic words
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

    let final_mnemonic = mnemonic_words.join(" ");

    println!("Mnemonic numbers: {:?}", mnemonic_decimal);
    println!("Mnemonic words: {:?}", final_mnemonic);
    

    Ok(final_mnemonic)

}

fn create_bip39_seed(entropy: &Vec<u8>, passphrase: &str) -> Result<String, bip39::Error> {
    // Parse the mnemonic phrase
    let mnemonic_result = bip39::Mnemonic::from_entropy(&entropy);
    
    // Check if the conversion was successful
    let mnemonic = mnemonic_result?;
    
    // Now you can use the mnemonic to generate the seed
    let seed = bip39::Mnemonic::to_seed(&mnemonic, passphrase);

    // Convert the seed to hexadecimal
    let seed_hex = hex::encode(&seed[..]);
    println!("BIP39 Seed: {:?}", seed_hex);
    Ok(seed_hex)
}

// WORKING