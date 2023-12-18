use rand::Rng;
use sha2::{Digest, Sha256};
use std::io::{self, Read, Seek, SeekFrom};
use std::fs::File;

const ENTROPY_FILE: &str = "entropy/binary.qrn";

fn generate_entropy_from_file(file_path: &str) -> Result<Vec<u8>, io::Error> {
    // Open the file
    let file = File::open(file_path)?;
    let mut reader = io::BufReader::new(file);

    // Set a random start point
    let mut rng = rand::thread_rng();
    let start_point: usize = rng.gen_range(0..256);
    println!("Random Start Point: {}", start_point);

    // Seek to the random start point
    reader.seek(io::SeekFrom::Start(start_point as u64))?;

    // Read only 256 characters into a string called entropy_ascii
    let mut entropy_ascii = String::new();
    reader.take(256).read_to_string(&mut entropy_ascii)?;

    // Convert the binary string to a Vec<u8>
    let byte_vec: Vec<u8> = entropy_ascii
        .chars()
        .collect::<Vec<char>>()
        .chunks(8)
        .map(|chunk| chunk.iter().fold(0, |acc, &bit| (acc << 1) | (bit as u8 - '0' as u8)))
        .collect();
    
    println!("Entropy ASCII: {}", entropy_ascii);
    println!("Entropy Bytes: {:?}", byte_vec);

    Ok(byte_vec)
}

fn entropy_checksum(entropy_ascii: Vec<u8>) -> Result<String, io::Error> {
    // Calculate SHA256 hash of entropy_ascii directly
    let hash = Sha256::digest(&entropy_ascii);
    println!("Hash: {:?}", hash);

    // Convert the entire hash to UTF-8 format
    let hash_utf8: String = hash.iter().map(|byte| format!("{:02x}", byte)).collect();
    println!("Hash in UTF-8 format: {}", hash_utf8);
    
    // Convert the entire hash to ASCII format using '1' and '0'
    let hash_ascii: String = hash
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |i| ((byte >> i) & 1).to_string()))
        .collect();
    println!("Hash in ASCII format: {}", hash_ascii);

    // Take the first 8 bits of the hash and convert each bit to ASCII
    let ascii_chars: String = hash_ascii.chars().take(8).collect();
    println!("First 8 bits in ASCII format: {}", ascii_chars);

    Ok(ascii_chars)
}

fn main() -> Result<(), io::Error> {
    // Example usage
    let entropy = generate_entropy_from_file(ENTROPY_FILE)?;

    println!("Entropy: {:?}", entropy);

    let checksum = entropy_checksum(entropy)?;

    println!("Checksum: {:?}", checksum);

    Ok(())
}
