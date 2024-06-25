// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// module = "QRNG Library"
// copyright = "Copyright © 2023-2024 D3BUG"
// version = "2024-06-16"


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


use sha2::{Digest, Sha256, Sha512};


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


pub fn convert_binary_to_string(input_value: &[u8]) -> String {
    input_value
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |i| ((byte >> i) & 1).to_string()))
        .collect()
}

pub fn convert_string_to_binary(input_value: &str) -> Vec<u8> {
    input_value
        .chars()
        .collect::<Vec<char>>()
        .chunks(8)
        .map(|chunk| chunk.iter().fold(0, |acc, &bit| (acc << 1) | (bit as u8 - '0' as u8)))
        .collect()
}


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


pub fn calculate_sha256_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();

    hasher.update(data);
    hasher.finalize().iter().cloned().collect()
}

pub fn calculate_double_sha256_hash(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let first_hash = hasher.finalize();
    
    let mut hasher = Sha256::new();
    hasher.update(&first_hash);
    let final_hash = hasher.finalize().to_vec();

    final_hash
}

pub fn calculate_sha256_and_ripemd160_hash(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let hash = hasher.finalize();
    
    let mut ripemd = ripemd::Ripemd160::new();
    ripemd.update(&hash);
    let final_hash = ripemd.finalize().to_vec();

    final_hash
}

pub fn calculate_hmac_sha512_hash(key: &[u8], data: &[u8]) -> Vec<u8> {
    const BLOCK_SIZE: usize = 128;
    const HASH_SIZE: usize = 64;

    let padded_key = if key.len() > BLOCK_SIZE {
        let mut hasher = Sha512::new();
        hasher.update(key);
        let mut hashed_key = vec![0u8; HASH_SIZE];
        hashed_key.copy_from_slice(&hasher.finalize());
        hashed_key.resize(BLOCK_SIZE, 0x00);
        hashed_key
    } else {
        let mut padded_key = vec![0x00; BLOCK_SIZE];
        padded_key[..key.len()].copy_from_slice(key);
        padded_key
    };

    assert_eq!(padded_key.len(), BLOCK_SIZE, "Padded key length mismatch");

    let mut inner_pad = vec![0x36; BLOCK_SIZE];
    let mut outer_pad = vec![0x5c; BLOCK_SIZE];
    for (i, &b) in padded_key.iter().enumerate() {
        inner_pad[i] ^= b;
        outer_pad[i] ^= b;
    }

    let mut hasher = Sha512::new();
    hasher.update(&inner_pad);
    hasher.update(data);
    let inner_hash = hasher.finalize();
    let mut hasher = Sha512::new();
    hasher.update(&outer_pad);
    hasher.update(&inner_hash);
    let final_hash = hasher.finalize().to_vec();

    assert_eq!(final_hash.len(), HASH_SIZE, "Final hash length mismatch");

    final_hash
}

pub fn calculate_checksum_for_master_keys(data: &[u8]) -> [u8; 4] {
    let hash = Sha256::digest(data);
    let double_hash = Sha256::digest(&hash);
    let mut checksum = [0u8; 4];
    checksum.copy_from_slice(&double_hash[..4]);
    checksum
}

pub fn calculate_checksum_for_entropy(entropy: &str, entropy_length: &u32) -> String {
    let entropy_binary = convert_string_to_binary(&entropy);
    let hash_raw_binary: String = convert_binary_to_string(&Sha256::digest(&entropy_binary));
    let checksum_length = entropy_length / 32;
    let entropy_checksum: String = hash_raw_binary.chars().take(checksum_length.try_into().unwrap()).collect();
    entropy_checksum
}


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
