extern crate rand;
extern crate sha2;

use rand::Rng;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};

const ENTROPY_FILE: &str = "entropy/binary.qrn";
const WORDLIST_FILE: &str = "lib/bip39-english.txt";


fn generate_entropy() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..32).map(|_| rng.gen()).collect()
}

fn calculate_checksum(entropy: &[u8]) -> Vec<u8> {
    let hash = Sha256::digest(entropy);

    // Take 1 bit of the hash for every 32 bits of entropy
    let num_bits = entropy.len() * 8;
    let num_bits_to_take = num_bits / 32;

    // Extract the relevant bits from the hash
    let checksum: Vec<u8> = hash
        .iter()
        .take(num_bits_to_take)
        .enumerate()
        .filter_map(|(i, &byte)| if i % 8 == 0 { Some(byte) } else { None })
        .collect();

    checksum
}

fn bytes_to_binary_string(data: &[u8]) -> String {
    data.iter().map(|&byte| format!("{:08b}", byte)).collect()
}

fn binary_string_to_decimal(binary_string: &str) -> u64 {
    u64::from_str_radix(binary_string, 2).unwrap()
}

fn decimal_words_to_bip39(decimal_value: u64, wordlist: &Vec<String>) -> &str {
    &wordlist[decimal_value as usize]
}

fn read_wordlist_from_file(file_path: &str) -> Vec<String> {
    fs::read_to_string(file_path)
        .expect("Failed to read wordlist file")
        .lines()
        .map(String::from)
        .collect()
}

fn decimal_words<'a>(final_entropy: &'a [u8], wordlist: &'a Vec<String>) -> Vec<&'a str> {
    bytes_to_binary_string(final_entropy)
        .chars()
        .collect::<Vec<char>>()
        .chunks(11)
        .map(|chunk| {
            let decimal_value = binary_string_to_decimal(&chunk.iter().collect::<String>());
            decimal_words_to_bip39(decimal_value, wordlist)
        })
        .collect()
}


fn main() {
    let entropy = generate_entropy();
    let checksum = calculate_checksum(&entropy);
    let final_entropy: Vec<u8> = entropy.iter().cloned().chain(checksum.iter().cloned()).collect();

    let wordlist = read_wordlist_from_file(WORDLIST_FILE);

    let bip39_words = decimal_words(&final_entropy, &wordlist);

    println!("Entropy: {}", bytes_to_binary_string(&entropy));
    println!("Checksum: {}", bytes_to_binary_string(&checksum));
    println!("final_entropy: {}", bytes_to_binary_string(&final_entropy));
    println!("BIP39 Words: {:?}", bip39_words);
}
