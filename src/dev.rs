// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// module = "Development playground"
// copyright = "Copyright © 2023-2024 D3BUG"


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


use sha2::{Digest, Sha256};
use ed25519_dalek::{SigningKey, VerifyingKey};


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


pub fn derive_from_path_ed25519(
    master_key: &[u8],
    master_chain_code: &[u8],
    path: &str,
) -> crate::DerivationResult {
    println!("Deriving from path for ed25519: {}", path);

    println!("master_key: {:?}", &master_key);
    println!("master_chain_code: {:?}", &master_chain_code);
    println!("path: {:?}", &path);

    let private_key = master_key.to_vec();
    let chain_code = master_chain_code.to_vec();
    let mut public_key = Vec::new();
    let mut private_key_array = [0; 32];

    for part in path.split('/') {
        if part == "m" {
            continue;
        }

        let hardened = part.ends_with("'");
        let index: u32 = match part.trim_end_matches("'").parse() {
            Ok(index) => {
                index
            },
            Err(_) => {
                eprintln!("Error: Unable to parse index from path part: {}", part);
                return None;
            }
        };
        let derived = derive_child_key_ed25519(
            &private_key, 
            &chain_code, 
            index, 
            hardened
        ).unwrap_or_default();

        private_key.clone().copy_from_slice(&derived.0);
        private_key_array = derived.0.clone().try_into().expect("Incorrect private key length");

        chain_code.clone().copy_from_slice(&derived.1);
        public_key = derived.2.clone();
    }

    let chain_code_array: [u8; 32] = chain_code.try_into().expect("Slice with incorrect length");
    Some((private_key_array, chain_code_array, public_key))
}

fn derive_child_key_ed25519(
    parent_key: &[u8],
    parent_chain_code: &[u8],
    index: u32,
    hardened: bool,
) -> crate::DerivationResult {
    println!("Deriving ed25519 child key");
    
    println!("parent_key: {:?}", &parent_key);
    println!("parent_chain_code: {:?}", &parent_chain_code);
    println!("index: {:?}", &index);
    println!("hardened: {:?}", &hardened);
    
    let mut hasher = Sha256::new();
    hasher.update(parent_key);
    hasher.update(&index.to_be_bytes());
    if hardened {
        hasher.update(&[1u8; 1]);
    }
    let result = hasher.finalize();
    
    if result.len() != 64 {
        return None;
    }
    
    let mut child_private_key_bytes: [u8; 32] = [0; 32];
    let mut child_chain_code_bytes: [u8; 32] = [0; 32];
    
    child_private_key_bytes.copy_from_slice(&result[..32]);
    child_chain_code_bytes.copy_from_slice(&result[32..]);

    let secret_key = SigningKey::from_bytes(&child_private_key_bytes).to_bytes();
    let public_key = VerifyingKey::from_bytes(&secret_key).unwrap_or_default().to_bytes().to_vec();
    
    println!("child_private_key_bytes: {:?}", &secret_key);
    println!("child_chain_code_bytes: {:?}", &child_chain_code_bytes);
    println!("child_public_key_bytes: {:?}", &public_key);

    Some((
        secret_key,
        child_chain_code_bytes,
        public_key,
    ))
}

pub fn generate_ed25519_address(public_key: &crate::CryptoPublicKey) -> String {
    let public_key_bytes = match public_key {
        crate::CryptoPublicKey::Ed25519(key) => key.to_bytes().to_vec(),
        _ => {
            eprintln!("generate_ed25519_address called with non-ed25519 key");
            Vec::new()
        }
    };
    
    let hash = Sha256::digest(&public_key_bytes);
    bs58::encode(hash).with_alphabet(bs58::Alphabet::BITCOIN).into_string()
}


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
