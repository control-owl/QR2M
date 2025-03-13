// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// module = "Cryptographic keys"
// copyright = "Copyright © 2023-2025 Control Owl"
// version = "2025-03-13"


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


use num_bigint::BigUint;
use sha3::Keccak256;
use sha2::{Digest, Sha256};
use rand::Rng;
use gtk4 as gtk;
use libadwaita as adw;
use adw::prelude::*;
use gtk::glib::clone;
use std::{
    fs::File, 
    io::Read
};

use crate::APP_SETTINGS;

pub type DerivationResult = Option<([u8; 32], [u8; 32], Vec<u8>)>;
type MasterPrivateKey = (String, String, Vec<u8>, Vec<u8>, Vec<u8>);

pub struct AddressHocusPokus {
    pub coin_index: u32,
    pub derivation_path: String,
    pub master_private_key_bytes: Vec<u8>,
    pub master_chain_code_bytes: Vec<u8>,
    pub public_key_hash: String,
    pub key_derivation: String,
    pub wallet_import_format: String,
    pub hash: String,
}


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


pub enum CryptoPublicKey {
    Secp256k1(secp256k1::PublicKey),
    Ed25519(ed25519_dalek::VerifyingKey),
}

pub fn derive_child_key_secp256k1(
    parent_key: &[u8],
    parent_chain_code: &[u8],
    index: u32,
    hardened: bool,
) -> DerivationResult {
    println!("[+] {}", &t!("log.derive_child_key").to_string());
    
    println!("parent_key {:?}", parent_key);
    println!("parent_chain_code {:?}", parent_chain_code);
    println!("index {:?}", index);
    println!("hardened {:?}", hardened);
    
    if index & 0x80000000 != 0 && !hardened {
        return None;
    }

    let secp = secp256k1::Secp256k1::new();
    let mut data = Vec::with_capacity(37);

    if hardened {
        data.push(0x00);
        data.extend_from_slice(parent_key);
    } else {
        let parent_secret_key = secp256k1::SecretKey::from_slice(parent_key).ok()?;
        let parent_pubkey = secp256k1::PublicKey::from_secret_key(&secp, &parent_secret_key);
        data.extend_from_slice(&parent_pubkey.serialize()[..]);
    }

    let index_bytes = if hardened {
        let index = index + crate::WALLET_MAX_ADDRESSES + 1;
        index.to_be_bytes()
    } else {
        index.to_be_bytes()
    };

    data.extend_from_slice(&index_bytes);

    println!("data_for_hmac_sha512 {:?}", data);
    
    let result = qr2m_lib::calculate_hmac_sha512_hash(parent_chain_code, &data);
    
    let child_private_key_bytes: [u8; 32] = result[..32].try_into().expect("Slice with incorrect length");
    let child_chain_code_bytes: [u8; 32] = result[32..].try_into().expect("Slice with incorrect length");

    let child_key_int = BigUint::from_bytes_be(&child_private_key_bytes);
    let parent_key_int = BigUint::from_bytes_be(parent_key);
    let curve_order = BigUint::from_bytes_be(&secp256k1::constants::CURVE_ORDER);
    let combined_int = (parent_key_int + child_key_int) % &curve_order;
    let combined_bytes = combined_int.to_bytes_be();
    let combined_bytes_padded = {
        let mut padded = [0u8; 32];
        let offset = 32 - combined_bytes.len();
        padded[offset..].copy_from_slice(&combined_bytes);
        padded
    };
    let child_secret_key = secp256k1::SecretKey::from_slice(&combined_bytes_padded).ok()?;
    let child_secret_key_bytes = child_secret_key.secret_bytes();
    let child_pubkey = secp256k1::PublicKey::from_secret_key(&secp, &child_secret_key);
    let child_public_key_bytes = child_pubkey.serialize().to_vec();

    println!("child_private_key_bytes {:?}", child_secret_key_bytes);
    println!("child_chain_code_bytes {:?}", child_chain_code_bytes);
    println!("child_public_key_bytes {:?}", child_public_key_bytes);

    Some((child_secret_key_bytes, child_chain_code_bytes, child_public_key_bytes))
}

pub fn create_private_key_for_address(
    private_key: Option<&secp256k1::SecretKey>, 
    compressed: Option<bool>,
    wif: Option<&str>,
    hash: &str,
) -> Result<String, String> {
    println!("Private key to WIF");

    let wallet_import_format = match wif {
        Some(w) => {
            if w.is_empty() {
                "80"
            } else {
                w.trim_start_matches("0x")
            }
        },
        None => "80",
    };

    let compressed = compressed.unwrap_or(true);
    
    let wallet_import_format_bytes = match hex::decode(wallet_import_format) {
        Ok(bytes) => bytes,
        Err(_) => return Err("Invalid WIF format".to_string()),
    };

    match hash {
        "sha256" => {
            let mut extended_key = Vec::with_capacity(34);
            extended_key.extend_from_slice(&wallet_import_format_bytes);

            if let Some(private_key) = private_key {
                extended_key.extend_from_slice(&private_key.secret_bytes());

                if compressed {
                    extended_key.push(0x01);
                }
            } else {
                return Err("Private key must be provided".to_string());
            }

            let checksum = qr2m_lib::calculate_double_sha256_hash(&extended_key);
            let address_checksum = &checksum[0..4];
            extended_key.extend_from_slice(address_checksum);

            Ok(bs58::encode(extended_key).into_string())
        },
        "keccak256" => {
            if let Some(private_key) = private_key {
                Ok(format!("0x{}", hex::encode(private_key.secret_bytes())))
            } else {
                Err("Private key must be provided".to_string())
            }
        },
        "sha256+ripemd160" => {
            match private_key {
                Some(key) => {
                    let private_key_hex = hex::encode(key.secret_bytes());
                    println!("Private key hex: {}", private_key_hex);
                    Ok(private_key_hex)
                },
                None => {
                    println!("Private key must be provided");
                    Err("Private key must be provided".to_string())
                },
            }
        },
        _ => Err(format!("Unsupported hash method: {}", hash)),
    }
}

pub fn derive_from_path_secp256k1(
    master_key: &[u8],
    master_chain_code: &[u8],
    path: &str,
) -> DerivationResult {
    println!("[+] {}", &t!("log.derive_from_path_secp256k1").to_string());

    println!("Derivation path {:?}", path);

    let mut private_key = master_key.to_vec();
    let mut chain_code = master_chain_code.to_vec();
    let mut public_key = Vec::new();


    for part in path.split('/') {
        if part == "m" {
            continue;
        }

        let hardened = part.ends_with("'");
        let index: u32 = match part.trim_end_matches("'").parse() {
            Ok(index) => {
                println!("Index: {:?}", &index);
                index
            },
            Err(_) => {
                eprintln!("Error: Unable to parse index from path part: {}", part);
                return None;
            }
        };
        
        let derived = derive_child_key_secp256k1(
            &private_key, 
            &chain_code, 
            index, 
            hardened
        ).unwrap_or_default();
        
        private_key = derived.0.to_vec();
        chain_code = derived.1.to_vec();
        public_key = derived.2;
    }
    
    let secret_key = match secp256k1::SecretKey::from_slice(&private_key) {
        Ok(sk) => sk,
        Err(e) => {
            eprintln!("Error: Unable to create SecretKey from key slice: {}", e);
            return None;
        }
    };

    if chain_code.len() != 32 {
        eprintln!("Error: Invalid chain code length");
        return None;
    }

    let mut chain_code_array = [0u8; 32];
    chain_code_array.copy_from_slice(&chain_code);

    let mut public_key_array = [0u8; 33];
    public_key_array.copy_from_slice(&public_key);

    Some((secret_key.secret_bytes(), chain_code_array, public_key_array.to_vec()))
}

pub fn generate_address_sha256(
    public_key: &CryptoPublicKey,
    public_key_hash: &[u8],
) -> String {
    println!("[+] {}", &t!("log.generate_address_sha256").to_string());

    let public_key_bytes = match public_key {
        CryptoPublicKey::Secp256k1(key) => key.serialize().to_vec(),
        CryptoPublicKey::Ed25519(key) => key.to_bytes().to_vec(),
    };
    
    println!("Public key bytes: {:?}", &public_key_bytes);

    let hash160 = qr2m_lib::calculate_sha256_and_ripemd160_hash(&public_key_bytes);

    let mut payload = Vec::with_capacity(public_key_hash.len() + hash160.len());
    payload.extend_from_slice(public_key_hash);
    payload.extend_from_slice(&hash160);
    println!("Extended sha256_and_ripemd160 payload: {:?}", &payload);

    let checksum = qr2m_lib::calculate_double_sha256_hash(&payload);

    let address_checksum = &checksum[0..4];
    println!("Address checksum: {:?}", address_checksum);

    let mut address_payload = payload;
    address_payload.extend_from_slice(address_checksum);
    println!("Extended Address payload: {:?}", address_payload);

    bs58::encode(address_payload).into_string()
}

pub fn generate_address_keccak256(
    public_key: &CryptoPublicKey,
    _public_key_hash: &[u8],
) -> String {
    let public_key_bytes = match public_key {
        CryptoPublicKey::Secp256k1(key) => key.serialize_uncompressed().to_vec(),
        CryptoPublicKey::Ed25519(key) => key.to_bytes().to_vec(),
    };
    println!("Public key bytes: {:?}", &public_key_bytes);

    let public_key_slice = match public_key {
        CryptoPublicKey::Secp256k1(_) => &public_key_bytes[1..],
        CryptoPublicKey::Ed25519(_) => &public_key_bytes[..],
    };

    let mut keccak = Keccak256::new();
    keccak.update(public_key_slice);
    let keccak_result = keccak.finalize();
    println!("Keccak256 hash result: {:?}", &keccak_result);

    let address_bytes = &keccak_result[12..];
    println!("Address bytes: {:?}", address_bytes);

    let address = format!("0x{}", hex::encode(address_bytes));
    println!("Generated Ethereum address: {:?}", address);

    address
}

pub fn generate_sha256_ripemd160_address(
    coin_index: u32,
    public_key: &CryptoPublicKey,
    public_key_hash: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    let public_key_bytes = match public_key {
        CryptoPublicKey::Secp256k1(key) => key.serialize().to_vec(),
        CryptoPublicKey::Ed25519(key) => key.to_bytes().to_vec(),
    };
    println!("Public key bytes: {:?}", &public_key_bytes);

    let hash = qr2m_lib::calculate_sha256_and_ripemd160_hash(&public_key_bytes);
    let mut address_bytes = Vec::new();

    address_bytes.extend_from_slice(public_key_hash);
    address_bytes.extend(&hash);

    let checksum = Sha256::digest(Sha256::digest(&address_bytes));
    let checksum = &checksum[0..4];

    let mut full_address_bytes = address_bytes.clone();
    full_address_bytes.extend(checksum);

    let alphabet = match coin_index {
        144 => bs58::Alphabet::RIPPLE,
        _ => bs58::Alphabet::DEFAULT,
    };

    let encoded_address = bs58::encode(full_address_bytes).with_alphabet(alphabet).into_string();
    println!("Base58 encoded address: {}", encoded_address);

    Ok(encoded_address)
}


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


pub fn generate_entropy(
    source: &str, 
    entropy_length: u64, 
    // state: Option<std::sync::Arc<std::sync::Mutex<AppState>>>,
) -> String {
    println!("[+] {}", &t!("log.generate_entropy").to_string());

    println!(" - Entropy source: {:?}", source);
    println!(" - Entropy length: {:?}", entropy_length);

    match source {
        "RNG" | "RNG+" => {
            let mut rng = rand::rng();
            let rng_entropy_string: String = (0..entropy_length)
                .map(|_| rng.random_range(0..=1))
                .map(|bit| char::from_digit(bit, 10).unwrap())
                .collect();

            println!(" - RNG Entropy: {:?}", rng_entropy_string);

            let mut wallet_settings = crate::WALLET_SETTINGS.lock().unwrap(); // This locks the Mutex
            wallet_settings.entropy_string = Some(rng_entropy_string.clone());

            rng_entropy_string
        },
        "QRNG" => {
            // if let Some(state) = &state {
            //     let mut state = state.lock().unwrap();
            //     let info_bar = state.info_bar.clone();

            //     state.update_infobar_message(
            //         "Requesting QRNG from ANU API ...".to_string(),
            //         // info_bar.unwrap(),
            //         gtk::MessageType::Info,
            //     );
            // }

            let (
                anu_data_format, 
                array_length, 
                hex_block_size, 
                anu_log,
                entropy_length,
            ) = {
                let lock_app_settings = APP_SETTINGS.read().unwrap();
                (
                    lock_app_settings.anu_data_format.clone().unwrap(),
                    lock_app_settings.anu_array_length.unwrap(),
                    lock_app_settings.anu_hex_block_size.unwrap(),
                    lock_app_settings.anu_log.unwrap(),
                    lock_app_settings.wallet_entropy_length.unwrap(),
                )
            };

            let open_context = glib::MainContext::default();
            let open_loop = glib::MainLoop::new(Some(&open_context), false);
            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(clone!(
                #[strong] open_loop,
                move || {

                    // data_format: &str, 
                    // array_length: u32, 
                    // hex_block_size: Option<u32>,
                    // log: Option<bool>,

                    let qrng_entropy_string = crate::anu::get_entropy_from_anu(
                        entropy_length as usize,
                        &anu_data_format,
                        array_length,
                        hex_block_size,
                        anu_log,
                    );

                    if let Err(err) = tx.send(qrng_entropy_string) {
                        println!("Error sending data back: {}", err);
                    }

                    open_loop.quit();
                }
            ));

            open_loop.run();

            match rx.recv() {
                Ok(received_qrng_entropy_string) => {
                    // if let Some(state) = &state {
                    //     let mut state = state.lock().unwrap();
                    //     let info_bar = state.info_bar.clone();
                    //     state.update_infobar_message(
                    //         format!("QRNG Data received"),
                    //         // info_bar.unwrap(),
                    //         gtk::MessageType::Info,
                    //     );
                    // }

                    received_qrng_entropy_string
                },
                Err(_) => {
                    println!("Error retrieving entropy from ANU API.");
                    String::new()
                }
            }
        },
        "File" => {
            let open_context = glib::MainContext::default();
            let open_loop = glib::MainLoop::new(Some(&open_context), false);
            let (tx, rx) = std::sync::mpsc::channel();
            
            let open_window = gtk::Window::new();          
            let open_dialog = gtk::FileChooserNative::new(
                Some(t!("UI.dialog.open").to_string().as_str()),
                Some(&open_window),
                gtk::FileChooserAction::Open,
                Some(&t!("UI.element.button.open")),
                Some(&t!("UI.element.button.cancel"))
            );
    
            open_dialog.connect_response(clone!(
                #[strong] open_loop,
                move |open_dialog, response| {
                    if response == gtk::ResponseType::Accept {
                        if let Some(file) = open_dialog.file() {
                            if let Some(path) = file.path() {
                                let file_path = path.to_string_lossy().to_string();
                                println!(" - Entropy file name: {:?}", file_path);
                                
                                let file_entropy_string = generate_entropy_from_file(&file_path, entropy_length);
                                
                                if let Err(err) = tx.send(file_entropy_string) {
                                    println!("{}", &t!("error.mpsc.send", value = err));
                                } else {
                                    open_loop.quit();
                                }
                            }
                        }
                    }
                    open_dialog.destroy();
                    open_loop.quit();
                }
            ));
            
            open_dialog.show();
            open_loop.run();
            
            match rx.recv() {
                Ok(received_file_entropy_string) => {
                    let mut wallet_settings = crate::WALLET_SETTINGS.lock().unwrap();
                    wallet_settings.entropy_string = Some(received_file_entropy_string.clone());

                    received_file_entropy_string
                },
                Err(_) => {
                    println!("{}", &t!("error.entropy.create.file"));
                    String::new()
                }
            }
        },
        _ => {
            println!("{}", &t!("error.entropy.create.source"));
            String::new()
        }
    }
}

pub fn generate_mnemonic_words(final_entropy_binary: &str) -> String {
    println!("[+] {}", &t!("log.generate_mnemonic_words").to_string());
    println!(" - Final entropy: {:?}", final_entropy_binary);
    
    let chunks: Vec<String> = final_entropy_binary.chars()
        .collect::<Vec<char>>()
        .chunks(11)
        .map(|chunk| chunk.iter().collect())
        .collect();

    let mnemonic_decimal: Vec<u32> = chunks.iter()
        .map(|chunk| u32::from_str_radix(chunk, 2).unwrap())
        .collect();
    
    let wordlist_path = std::path::Path::new("coin").join(crate::WORDLIST_FILE);
    let wordlist = qr2m_lib::get_text_from_resources(wordlist_path.to_str().unwrap());

    let bad_word = t!("error.wordlist.word").to_string();
    let mnemonic_words_vector: Vec<&str> = wordlist.lines().collect();
    let mnemonic_words_vector: Vec<&str> = mnemonic_decimal.iter().map(|&decimal| {
        if (decimal as usize) < mnemonic_words_vector.len() {
            mnemonic_words_vector[decimal as usize]
        } else {
            &bad_word
        }
    }).collect();

    let mnemonic_words_as_string = mnemonic_words_vector.join(" ");
    
    println!(" - Entropy chunks: {:?}", chunks);
    println!(" - Decimal mnemonic: {:?}", mnemonic_decimal);
    println!(" - Mnemonic words: {:?}", mnemonic_words_vector);

    let mut wallet_settings = crate::WALLET_SETTINGS.lock().unwrap();
    wallet_settings.mnemonic_words = Some(mnemonic_words_as_string.clone());
    
    mnemonic_words_as_string
}

pub fn generate_bip39_seed(entropy: &str, passphrase: &str) -> [u8; 64] {
    println!("[+] {}", &t!("log.generate_bip39_seed").to_string());
    println!(" - Entropy: {:?}", entropy);
    println!(" - Passphrase: {:?}", passphrase);

    let entropy_vector = qr2m_lib::convert_string_to_binary(entropy);
    let mnemonic = match bip39::Mnemonic::from_entropy(&entropy_vector) {
        Ok(mnemonic) => mnemonic,
        Err(err) => {
            println!("{}", &t!("error.bip.mnemonic", error = err));
            return [0; 64];
        },
    };
    let seed = bip39::Mnemonic::to_seed(&mnemonic, passphrase);

    println!(" - Seed: {:?}", seed);
    
    seed
}

pub fn generate_entropy_from_file(file_path: &str, entropy_length: u64) -> String {
    println!("[+] {}", &t!("log.generate_entropy_from_file").to_string());
    println!(" - File: {:?}", file_path);
    println!(" - Entropy length: {:?}", entropy_length);

    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(err) => {
            println!("{}", &t!("error.file.open", value = file_path, error = err));
            return String::new()
        },
    };
    
    let mut buffer = Vec::new();
    
    match file.read_to_end(&mut buffer) {
        Ok(_) => {},
        Err(err) => {
            println!("{}", &t!("error.file.read", value = file_path, error = err));
        },
    };

    let hash = qr2m_lib::calculate_sha256_hash(&["qr2m".as_bytes(), &buffer].concat());
    let mut entropy = String::new();

    for byte in &hash {
        entropy.push_str(&format!("{:08b}", byte));
    }

    entropy = entropy.chars().take(entropy_length as usize).collect();
    
    println!(" - File entropy hash: {:?}", hash);
    println!(" - File entropy: {:?}", entropy);

    entropy
}

pub fn generate_master_keys(seed: &str, mut private_header: &str, mut public_header: &str) -> Result<MasterPrivateKey, String> {
    println!("[+] {}", &t!("log.derive_master_keys").to_string());
    println!(" - Private header: {:?}", private_header);
    println!(" - Public header: {:?}", public_header);

    if private_header.is_empty() {
        private_header = "0x0488ADE4";
    }
    if public_header.is_empty() {
        public_header = "0x0488B21E";
    }
    
    let private_header = u32::from_str_radix(private_header.trim_start_matches("0x"), 16)
        .expect(&t!("error.master.parse.header", value = "private"));
    let public_header = u32::from_str_radix(public_header.trim_start_matches("0x"), 16)
        .expect(&t!("error.master.parse.header", value = "public"));

    let seed_bytes = hex::decode(seed).expect(&t!("error.seed.decode"));
    let message = "Bitcoin seed";
    let hmac_result = qr2m_lib::calculate_hmac_sha512_hash(message.as_bytes(), &seed_bytes);
    let (master_private_key_bytes, master_chain_code_bytes) = hmac_result.split_at(32);
    let mut master_private_key = Vec::new();

    master_private_key.extend_from_slice(&u32::to_be_bytes(private_header));
    master_private_key.push(0x00);
    master_private_key.extend([0x00; 4].iter());
    master_private_key.extend([0x00; 4].iter());
    master_private_key.extend_from_slice(master_chain_code_bytes);
    master_private_key.push(0x00);
    master_private_key.extend_from_slice(master_private_key_bytes);
    
    let checksum: [u8; 4] = qr2m_lib::calculate_checksum_for_master_keys(&master_private_key);
    
    master_private_key.extend_from_slice(&checksum);
    
    let master_xprv = bs58::encode(&master_private_key).into_string();
    let secp = secp256k1::Secp256k1::new();
    let master_secret_key = secp256k1::SecretKey::from_slice(master_private_key_bytes)
        .expect(&t!("error.master.create"));
    let master_public_key_bytes = secp256k1::PublicKey::from_secret_key(&secp, &master_secret_key).serialize();
    let mut master_public_key = Vec::new();

    master_public_key.extend_from_slice(&u32::to_be_bytes(public_header));
    master_public_key.push(0x00);
    master_public_key.extend([0x00; 4].iter());
    master_public_key.extend([0x00; 4].iter());
    master_public_key.extend_from_slice(master_chain_code_bytes);
    master_public_key.extend_from_slice(&master_public_key_bytes);
    
    let checksum: [u8; 4] = qr2m_lib::calculate_checksum_for_master_keys(&master_public_key);
    
    master_public_key.extend_from_slice(&checksum);
    
    let master_xpub = bs58::encode(&master_public_key).into_string();
    
    println!(" - Parsed private header {:?}", private_header);
    println!(" - Parsed public header {:?}", public_header);
    println!(" - Seed: {:?}", seed_bytes);
    println!(" - Hmac sha512 hash: {:?}", hmac_result);
    println!(" - Master key private bytes: {:?}", master_private_key_bytes);
    println!(" - Master key chain code: {:?}", master_chain_code_bytes);
    println!(" - Master private key (xprv): {:?}", master_xprv);
    println!(" - Master secret key {:?}", master_secret_key);
    println!(" - Master public key {:?}", master_public_key_bytes);
    println!(" - Master public key (xpub): {:?}", master_xpub);

    let mut wallet_settings = crate::WALLET_SETTINGS.lock().unwrap();
    wallet_settings.master_xprv = Some(master_xprv.clone());
    wallet_settings.master_xpub = Some(master_xpub.clone());
    wallet_settings.master_private_key_bytes = Some(master_private_key_bytes.to_vec());
    wallet_settings.master_chain_code_bytes = Some(master_chain_code_bytes.to_vec());
    wallet_settings.master_public_key_bytes = Some(master_public_key_bytes.to_vec());

    Ok((
        master_xprv, 
        master_xpub,
        master_private_key_bytes.to_vec(), 
        master_chain_code_bytes.to_vec(), 
        master_public_key_bytes.to_vec(), 
    ))
}

pub fn generate_address(ingredients: AddressHocusPokus) -> Result<(String, String, String), String> {
    println!("[+] {}", &t!("log.generate_address").to_string());

    
    println!("\t- derivation_path: {:?}", ingredients.derivation_path);

    let secp = secp256k1::Secp256k1::new();

    let trimmed_public_key_hash = if let Some(stripped) = ingredients.public_key_hash.strip_prefix("0x") {
        stripped
    } else {
        &ingredients.public_key_hash
    };

    let public_key_hash_vec = match hex::decode(trimmed_public_key_hash) {
        Ok(vec) => vec,
        Err(e) => {
            return Err(format!("Problem with decoding public_key_hash_vec: {:?}", e).to_string());
        },
    };

    let derived_child_keys = match ingredients.key_derivation.as_str() {
        "secp256k1" => derive_from_path_secp256k1(&ingredients.master_private_key_bytes, &ingredients.master_chain_code_bytes, &ingredients.derivation_path),
        "ed25519" => crate::dev::derive_from_path_ed25519(&ingredients.master_private_key_bytes, &ingredients.master_chain_code_bytes, &ingredients.derivation_path),
        _ => {
            return Err(format!("Unsupported key derivation method: {:?}", ingredients.key_derivation))
        }
    }.expect("Can not derive child key");

    let public_key = match ingredients.key_derivation.as_str() {
        "secp256k1" => {
            let secp_pub_key = secp256k1::PublicKey::from_secret_key(
                &secp,
                &secp256k1::SecretKey::from_slice(&derived_child_keys.0).expect("Invalid secret key")
            );
            CryptoPublicKey::Secp256k1(secp_pub_key)
        },
        "ed25519" => {
            let secret_key = ed25519_dalek::SigningKey::from_bytes(&derived_child_keys.0);
            let pub_key_bytes = ed25519_dalek::VerifyingKey::from(&secret_key);
            CryptoPublicKey::Ed25519(pub_key_bytes)
        },
        _ => {
            return Err(format!("Unsupported key derivation method: {:?}", ingredients.key_derivation));
        }
    };

    let public_key_encoded = match ingredients.hash.as_str() {
        "sha256" | "sha256+ripemd160" => match &public_key {
            CryptoPublicKey::Secp256k1(public_key) => hex::encode(public_key.serialize()),
            CryptoPublicKey::Ed25519(public_key) => hex::encode(public_key.to_bytes()),
        },
        "keccak256" => match &public_key {
            CryptoPublicKey::Secp256k1(public_key) => format!("0x{}", hex::encode(public_key.serialize())),
            CryptoPublicKey::Ed25519(public_key) => format!("0x{}", hex::encode(public_key.to_bytes())),
        },
        _ => {
            return Err(format!("Unsupported hash method: {:?}", ingredients.hash));
        }
    };

    let address = match ingredients.hash.as_str() {
        "sha256" => generate_address_sha256(&public_key, &public_key_hash_vec),
        "keccak256" => generate_address_keccak256(&public_key, &public_key_hash_vec),
        "sha256+ripemd160" => match generate_sha256_ripemd160_address(
            ingredients.coin_index, 
            &public_key, 
            &public_key_hash_vec
        ) {
            Ok(addr) => addr,
            Err(e) => {
                return Err(format!("Error generating address: {}", e));
            }
        },
        "ed25519" => crate::dev::generate_ed25519_address(&public_key),
        _ => {
            return Err(format!("Unsupported hash method: {:?}", ingredients.hash));
        }
    };

    // IMPROVEMENT: remove hard-coding, add this option to UI
    let compressed = true;

    let priv_key_wif = create_private_key_for_address(
        Some(&secp256k1::SecretKey::from_slice(&derived_child_keys.0).expect("Invalid secret key")),
        Some(compressed),
        Some(&ingredients.wallet_import_format),
        &ingredients.hash,
    ).expect("Failed to convert private key to WIF");

    Ok((address.clone(), public_key_encoded.clone(), priv_key_wif.clone()))
}


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

