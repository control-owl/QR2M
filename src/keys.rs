// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// license = "CC-BY-NC-ND-4.0  [2023-2025]  Control Owl"

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

use crate::{AppError, FunctionOutput};
use adw::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;
use num_bigint::BigUint;
use rand::Rng;
use sha2::{Digest, Sha256};
use sha3::Keccak256;
use std::{fs::File, io::Read};

pub type DerivationResult = Option<([u8; 32], [u8; 32], Vec<u8>)>;
pub type AddressResult = Option<Address>;

#[derive(Debug)]
pub struct AddressHocusPokus {
  pub coin_index: u32,
  pub derivation_path: String,
  pub master_private_key_bytes: Vec<u8>,
  pub master_chain_code_bytes: Vec<u8>,
  pub public_key_hash: String,
  pub key_derivation: String,
  pub wallet_import_format: String,
  pub hash: String,
  // pub seed: String,
}

#[derive(Debug)]
pub struct Address {
  pub address: String,
  pub public_key: String,
  pub private_key: String,
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

#[derive(Debug)]
pub enum CryptoPublicKey {
  Secp256k1(secp256k1::PublicKey),
  #[cfg(feature = "dev")]
  Ed25519(ed25519_dalek::VerifyingKey),
}

pub fn derive_child_key_secp256k1(
  parent_key: &[u8],
  parent_chain_code: &[u8],
  index: u32,
  hardened: bool,
) -> DerivationResult {
  #[cfg(debug_assertions)]
  {
    println!("[+] {}", &t!("log.derive_child_key").to_string());

    println!("parent_key {:?}", parent_key);
    println!("parent_chain_code {:?}", parent_chain_code);
    println!("index {:?}", index);
    println!("hardened {:?}", hardened);
  }

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

  #[cfg(debug_assertions)]
  println!("data_for_hmac_sha512 {:?}", data);

  let result = qr2m_lib::calculate_hmac_sha512_hash(parent_chain_code, &data);

  let child_private_key_bytes: [u8; 32] = result[..32]
    .try_into()
    .expect("Slice with incorrect length");
  let child_chain_code_bytes: [u8; 32] = result[32..]
    .try_into()
    .expect("Slice with incorrect length");

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

  #[cfg(debug_assertions)]
  {
    println!("child_private_key_bytes {:?}", child_secret_key_bytes);
    println!("child_chain_code_bytes {:?}", child_chain_code_bytes);
    println!("child_public_key_bytes {:?}", child_public_key_bytes);
  }

  Some((
    child_secret_key_bytes,
    child_chain_code_bytes,
    child_public_key_bytes,
  ))
}

pub fn create_private_key_for_address(
  private_key: Option<&secp256k1::SecretKey>,
  compressed: Option<bool>,
  wif: Option<&str>,
  hash: &str,
  coin_index: u32,
) -> Result<String, String> {
  #[cfg(debug_assertions)]
  println!(
    "[+] {}",
    &t!("log.create_private_key_for_address").to_string()
  );

  let wallet_import_format = match wif {
    Some(w) => {
      if w.is_empty() {
        "80"
      } else {
        w.trim_start_matches("0x")
      }
    }
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
    }
    "keccak256" => {
      if let Some(private_key) = private_key {
        if coin_index == 195 {
          Ok(hex::encode(private_key.secret_bytes()))
        } else {
          Ok(format!("0x{}", hex::encode(private_key.secret_bytes())))
        }
      } else {
        Err("Private key must be provided".to_string())
      }
    }
    "sha256+ripemd160" => match private_key {
      Some(key) => {
        let private_key_hex = hex::encode(key.secret_bytes());
        #[cfg(debug_assertions)]
        println!("Private key hex: {}", private_key_hex);
        Ok(private_key_hex)
      }
      None => {
        println!("Private key must be provided");
        Err("Private key must be provided".to_string())
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
  #[cfg(debug_assertions)]
  {
    println!("[+] {}", &t!("log.derive_from_path_secp256k1").to_string());
    println!("Derivation path {:?}", path);
  }

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
        #[cfg(debug_assertions)]
        println!("Index: {:?}", &index);

        index
      }
      Err(_) => {
        eprintln!("Error: Unable to parse index from path part: {}", part);
        return None;
      }
    };

    let derived =
      derive_child_key_secp256k1(&private_key, &chain_code, index, hardened).unwrap_or_default();

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

  Some((
    secret_key.secret_bytes(),
    chain_code_array,
    public_key_array.to_vec(),
  ))
}

pub fn generate_address_sha256(public_key: &CryptoPublicKey, public_key_hash: &[u8]) -> String {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.generate_address_sha256").to_string());

  let public_key_bytes = match public_key {
    CryptoPublicKey::Secp256k1(key) => key.serialize().to_vec(),
    #[cfg(feature = "dev")]
    CryptoPublicKey::Ed25519(key) => key.to_bytes().to_vec(),
  };

  #[cfg(debug_assertions)]
  println!("Public key bytes: {:?}", &public_key_bytes);

  let hash160 = qr2m_lib::calculate_sha256_and_ripemd160_hash(&public_key_bytes);

  let mut payload = Vec::with_capacity(public_key_hash.len() + hash160.len());
  payload.extend_from_slice(public_key_hash);
  payload.extend_from_slice(&hash160);

  #[cfg(debug_assertions)]
  println!("Extended sha256_and_ripemd160 payload: {:?}", &payload);

  let checksum = qr2m_lib::calculate_double_sha256_hash(&payload);
  let address_checksum = &checksum[0..4];

  #[cfg(debug_assertions)]
  println!("Address checksum: {:?}", address_checksum);

  let mut address_payload = payload;
  address_payload.extend_from_slice(address_checksum);

  #[cfg(debug_assertions)]
  println!("Extended Address payload: {:?}", address_payload);

  bs58::encode(address_payload).into_string()
}

pub fn generate_address_keccak256(
  public_key: &CryptoPublicKey,
  public_key_hash: &[u8],
  coin_index: u32,
) -> String {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.generate_address_keccak256").to_string());

  let public_key_bytes = match public_key {
    CryptoPublicKey::Secp256k1(key) => key.serialize_uncompressed().to_vec(),
    #[cfg(feature = "dev")]
    CryptoPublicKey::Ed25519(key) => key.to_bytes().to_vec(),
  };

  #[cfg(debug_assertions)]
  println!("Public key bytes: {:?}", &public_key_bytes);

  let public_key_slice = match public_key {
    CryptoPublicKey::Secp256k1(_) => &public_key_bytes[1..],
    #[cfg(feature = "dev")]
    CryptoPublicKey::Ed25519(_) => &public_key_bytes[..],
  };

  let mut keccak = Keccak256::new();
  keccak.update(public_key_slice);
  let keccak_result = keccak.finalize();

  #[cfg(debug_assertions)]
  println!("Keccak256 hash result: {:?}", &keccak_result);

  let address_bytes = &keccak_result[12..];

  #[cfg(debug_assertions)]
  println!("Address bytes: {:?}", address_bytes);

  let address = match coin_index {
    195 => {
      let mut tron_prefixed = public_key_hash.to_vec();
      tron_prefixed.extend_from_slice(address_bytes);

      let checksum = {
        let hash = Sha256::digest(&tron_prefixed);
        let hash2 = Sha256::digest(hash);
        hash2[..4].to_vec()
      };

      let mut full_payload = tron_prefixed.clone();
      full_payload.extend_from_slice(&checksum);

      bs58::encode(full_payload).into_string()
    }
    _ => {
      format!("0x{}", hex::encode(address_bytes))
    }
  };

  #[cfg(debug_assertions)]
  println!("Generated address: {}", address);

  address
}

pub fn generate_sha256_ripemd160_address(
  coin_index: u32,
  public_key: &CryptoPublicKey,
  public_key_hash: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
  #[cfg(debug_assertions)]
  println!(
    "[+] {}",
    &t!("log.generate_sha256_ripemd160_address").to_string()
  );

  let public_key_bytes = match public_key {
    CryptoPublicKey::Secp256k1(key) => key.serialize().to_vec(),
    #[cfg(feature = "dev")]
    CryptoPublicKey::Ed25519(key) => key.to_bytes().to_vec(),
  };

  #[cfg(debug_assertions)]
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

  let encoded_address = bs58::encode(full_address_bytes)
    .with_alphabet(alphabet)
    .into_string();

  #[cfg(debug_assertions)]
  println!("Base58 encoded address: {}", encoded_address);

  Ok(encoded_address)
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

pub fn generate_entropy(
  source: &str,
  entropy_length: u64,
  // state: Option<std::sync::Arc<std::sync::Mutex<AppState>>>,
) -> FunctionOutput<String> {
  #[cfg(debug_assertions)]
  {
    println!("[+] {}", &t!("log.generate_entropy").to_string());

    println!(" - Entropy source: {:?}", source);
    println!(" - Entropy length: {:?}", entropy_length);
  }

  match source {
    "RNG" | "RNG+" => {
      let mut rng = rand::rng();
      let rng_entropy: Result<String, AppError> = (0..entropy_length)
        .map(|_| rng.random_range(0..=1))
        .map(|bit| {
          char::from_digit(bit, 10)
            .ok_or_else(|| AppError::Custom(format!("Problem with RNG string for bit: {}", bit)))
        })
        .collect();

      let rng_entropy_string = rng_entropy?;

      #[cfg(debug_assertions)]
      println!(" - RNG Entropy: {:?}", rng_entropy_string);

      let mut wallet_settings = crate::WALLET_SETTINGS.lock().unwrap();
      wallet_settings.entropy_string = Some(rng_entropy_string.clone());

      Ok(rng_entropy_string)
    }
    #[cfg(feature = "full")]
    "QRNG" => {
      let (anu_data_format, array_length, hex_block_size, anu_log, entropy_length) = {
        let lock_app_settings = crate::APP_SETTINGS.read().unwrap();
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

      std::thread::spawn(gtk::glib::clone!(
        #[strong]
        open_loop,
        move || {
          let qrng_entropy_string = match crate::anu::get_entropy_from_anu(
            entropy_length as usize,
            &anu_data_format,
            array_length,
            hex_block_size,
            anu_log,
          ) {
            Ok(data) => data,
            Err(err) => {
              return Err(AppError::Custom(format!(
                "Anu response was empty: {:?}",
                err
              )));
            }
          };

          if let Err(err) = tx.send(qrng_entropy_string.clone()) {
            eprintln!("Error sending data back: {}", err);
          }

          open_loop.quit();

          Ok(qrng_entropy_string)
        }
      ));

      open_loop.run();

      match rx.recv() {
        Ok(data) => Ok(data),
        Err(err) => Err(AppError::Custom(format!(
          "Problem with generating QRNG: {:?}",
          err
        ))),
      }
    }
    "File" => {
      let open_context = glib::MainContext::default();
      let open_loop = glib::MainLoop::new(Some(&open_context), false);
      let (tx, rx) = std::sync::mpsc::channel::<String>();
      let open_dialog = gtk::FileDialog::builder().title("Select file").build();
      let loop_clone = open_loop.clone();

      open_dialog.open(
        None::<&gtk::Window>,
        None::<&gtk::gio::Cancellable>,
        move |result| match result {
          Ok(file) => {
            if let Some(path) = file.path() {
              let file_path = path.to_string_lossy().to_string();

              #[cfg(debug_assertions)]
              println!(" - Entropy file name: {:?}", file_path);

              let file_entropy_string = generate_entropy_from_file(&file_path, entropy_length);

              let mut wallet_settings = crate::WALLET_SETTINGS.lock().unwrap();
              wallet_settings.entropy_string = Some(file_entropy_string.clone());

              if let Err(err) = tx.send(file_entropy_string) {
                eprintln!("{}", &t!("error.mpsc.send", value = err));
              } else {
                #[cfg(debug_assertions)]
                println!("Sent");

                loop_clone.quit();
              }
            }
          }
          Err(err) => {
            eprintln!("{}", &t!("error.entropy.create.file"));
            eprintln!("File dialog error: {}", err);
            loop_clone.quit();
          }
        },
      );

      open_loop.run();

      match rx.recv() {
        Ok(received_file_entropy_string) => {
          #[cfg(debug_assertions)]
          println!("Received entropy: {}", received_file_entropy_string);

          Ok(received_file_entropy_string)
        }
        Err(err) => Err(AppError::Custom(format!(
          "Problem with generating entropy from a file: {:?}",
          err
        ))),
      }
    }
    _ => Err(AppError::Custom(
      "Problem with generating entropy".to_string(),
    )),
  }
}

pub fn generate_mnemonic_words(final_entropy_binary: &str, dictionary: Option<&str>) -> String {
  #[cfg(debug_assertions)]
  {
    println!("[+] {}", &t!("log.generate_mnemonic_words").to_string());
    println!(" - Entropy: {:?}", final_entropy_binary);
  }

  let chunks: Vec<String> = final_entropy_binary
    .chars()
    .collect::<Vec<char>>()
    .chunks(11)
    .map(|chunk| chunk.iter().collect())
    .collect();

  let mnemonic_decimal: Vec<u32> = chunks
    .iter()
    .map(|chunk| u32::from_str_radix(chunk, 2).unwrap())
    .collect();

  let dictionary_file = match dictionary.unwrap_or_default() {
    "Czech" => "czech.txt",
    "French" => "french.txt",
    "Italian" => "italian.txt",
    "Portuguese" => "portuguese.txt",
    "Spanish" => "spanish.txt",
    "Chinese simplified" => "chinese_simplified.txt",
    "Chinese traditional" => "chinese_traditional.txt",
    "Japanese" => "japanese.txt",
    "Korean" => "korean.txt",
    _ => "english.txt",
  };

  let wordlist_path = std::path::Path::new("wordlists").join(dictionary_file);
  let wordlist = qr2m_lib::get_text_from_resources(wordlist_path.to_str().unwrap());

  let bad_word = t!("error.wordlist.word").to_string();
  let mnemonic_words_vector: Vec<&str> = wordlist.lines().collect();
  let mnemonic_words_vector: Vec<&str> = mnemonic_decimal
    .iter()
    .map(|&decimal| {
      if (decimal as usize) < mnemonic_words_vector.len() {
        mnemonic_words_vector[decimal as usize]
      } else {
        &bad_word
      }
    })
    .collect();

  let mnemonic_words_as_string = mnemonic_words_vector.join(" ");

  #[cfg(debug_assertions)]
  {
    println!(" - Entropy chunks: {:?}", chunks);
    println!(" - Decimal mnemonic: {:?}", mnemonic_decimal);
    println!(" - Mnemonic words: {:?}", mnemonic_words_vector);
  }

  let mut wallet_settings = crate::WALLET_SETTINGS.lock().unwrap();
  wallet_settings.mnemonic_words = Some(mnemonic_words_as_string.clone());

  mnemonic_words_as_string
}

pub fn generate_seed_from_mnemonic(mnemonic: &str, passphrase: &str) -> [u8; 64] {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.generate_seed_from_mnemonic").to_string());

  let salt = format!("mnemonic{}", passphrase);
  let mut seed = [0u8; 64];
  ring::pbkdf2::derive(
    ring::pbkdf2::PBKDF2_HMAC_SHA512,
    std::num::NonZeroU32::new(2048).unwrap(),
    salt.as_bytes(),
    mnemonic.as_bytes(),
    &mut seed,
  );
  seed
}

pub fn generate_entropy_from_file(file_path: &str, entropy_length: u64) -> String {
  #[cfg(debug_assertions)]
  {
    println!("[+] {}", &t!("log.generate_entropy_from_file").to_string());
    println!(" - File: {:?}", file_path);
    println!(" - Entropy length: {:?}", entropy_length);
  }

  let mut file = match File::open(file_path) {
    Ok(file) => file,
    Err(err) => {
      eprintln!("{}", &t!("error.file.open", value = file_path, error = err));
      return String::new();
    }
  };

  let mut buffer = Vec::new();

  match file.read_to_end(&mut buffer) {
    Ok(_) => {}
    Err(err) => {
      eprintln!("{}", &t!("error.file.read", value = file_path, error = err));
    }
  };

  let hash = qr2m_lib::calculate_sha256_hash(&["qr2m".as_bytes(), &buffer].concat());
  let mut entropy = String::new();

  for byte in &hash {
    entropy.push_str(&format!("{:08b}", byte));
  }

  entropy = entropy.chars().take(entropy_length as usize).collect();

  #[cfg(debug_assertions)]
  {
    println!(" - File entropy hash: {:?}", hash);
    println!(" - File entropy: {:?}", entropy);
  }

  entropy
}

pub fn generate_master_keys_secp256k1(
  seed: &str,
  mut private_header: &str,
  mut public_header: &str,
) {
  #[cfg(debug_assertions)]
  {
    println!("[+] {}", &t!("log.derive_master_keys").to_string());
    println!(" - Private header: {:?}", private_header);
    println!(" - Public header: {:?}", public_header);
  }

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

  let master_private_key_encoded = bs58::encode(&master_private_key).into_string();
  let secp = secp256k1::Secp256k1::new();
  let master_secret_key =
    secp256k1::SecretKey::from_slice(master_private_key_bytes).expect(&t!("error.master.create"));
  let master_public_key_bytes =
    secp256k1::PublicKey::from_secret_key(&secp, &master_secret_key).serialize();
  let mut master_public_key = Vec::new();

  master_public_key.extend_from_slice(&u32::to_be_bytes(public_header));
  master_public_key.push(0x00);
  master_public_key.extend([0x00; 4].iter());
  master_public_key.extend([0x00; 4].iter());
  master_public_key.extend_from_slice(master_chain_code_bytes);
  master_public_key.extend_from_slice(&master_public_key_bytes);

  let checksum: [u8; 4] = qr2m_lib::calculate_checksum_for_master_keys(&master_public_key);

  master_public_key.extend_from_slice(&checksum);

  let master_public_key_encoded = bs58::encode(&master_public_key).into_string();

  #[cfg(debug_assertions)]
  {
    println!(" - Parsed private header {:?}", private_header);
    println!(" - Parsed public header {:?}", public_header);
    println!(" - Seed: {:?}", seed_bytes);
    println!(" - Hmac sha512 hash: {:?}", hmac_result);
    println!(
      " - Master key private bytes: {:?}",
      master_private_key_bytes
    );
    println!(" - Master key chain code: {:?}", master_chain_code_bytes);
    println!(" - Master private key: {:?}", master_private_key_encoded);
    println!(" - Master secret key {:?}", master_secret_key);
    println!(" - Master public key {:?}", master_public_key_bytes);
    println!(" - Master public key: {:?}", master_public_key_encoded);
  }

  let mut wallet_settings = crate::WALLET_SETTINGS.lock().unwrap();
  wallet_settings.master_private_key = Some(master_private_key_encoded.clone());
  wallet_settings.master_public_key = Some(master_public_key_encoded.clone());
  wallet_settings.master_private_key_bytes = Some(master_private_key_bytes.to_vec());
  wallet_settings.master_chain_code_bytes = Some(master_chain_code_bytes.to_vec());
  wallet_settings.master_public_key_bytes = Some(master_public_key_bytes.to_vec());
}

pub fn generate_address(ingredients: AddressHocusPokus) -> Result<AddressResult, String> {
  #[cfg(debug_assertions)]
  {
    println!("[+] {}", &t!("log.generate_address").to_string());
    println!("\t- ingredients: {:?}", ingredients);
  }

  let secp = secp256k1::Secp256k1::new();

  let public_key_hash_vec = if ingredients.key_derivation != "ed25519" {
    let trimmed_public_key_hash = ingredients
      .public_key_hash
      .strip_prefix("0x")
      .unwrap_or(&ingredients.public_key_hash);

    hex::decode(trimmed_public_key_hash)
      .map_err(|e| format!("Problem with decoding public_key_hash_vec: {:?}", e))?
  } else {
    Vec::new()
  };

  #[cfg(debug_assertions)]
  dbg!(&public_key_hash_vec);

  let derived_child_keys = match ingredients.key_derivation.as_str() {
    "secp256k1" => derive_from_path_secp256k1(
      &ingredients.master_private_key_bytes,
      &ingredients.master_chain_code_bytes,
      &ingredients.derivation_path,
    ),
    #[cfg(feature = "dev")]
    "ed25519" => crate::dev::derive_from_path_ed25519(
      &ingredients.master_private_key_bytes,
      &ingredients.master_chain_code_bytes,
      &ingredients.derivation_path,
      // &ingredients.seed,
    )?,
    _ => {
      return Err(format!(
        "Unsupported key derivation method: {:?}",
        ingredients.key_derivation
      ));
    }
  }
  .expect("Can not derive child key");

  #[cfg(debug_assertions)]
  dbg!(&derived_child_keys);

  let public_key = match ingredients.key_derivation.as_str() {
    "secp256k1" => {
      let secp_pub_key = secp256k1::PublicKey::from_secret_key(
        &secp,
        &secp256k1::SecretKey::from_slice(&derived_child_keys.0).expect("Invalid secret key"),
      );
      CryptoPublicKey::Secp256k1(secp_pub_key)
    }
    #[cfg(feature = "dev")]
    "ed25519" => {
      let sign_key = ed25519_dalek::SigningKey::from_bytes(&derived_child_keys.0);
      // let secret_key = sign_key.to_bytes();
      let pub_key = sign_key.verifying_key();
      CryptoPublicKey::Ed25519(pub_key)
    }
    _ => {
      return Err(format!(
        "Unsupported key derivation method: {:?}",
        ingredients.key_derivation
      ));
    }
  };

  #[cfg(debug_assertions)]
  dbg!(&public_key);

  let public_key_encoded = match ingredients.hash.as_str() {
    "sha256" | "sha256+ripemd160" => match &public_key {
      CryptoPublicKey::Secp256k1(public_key) => hex::encode(public_key.serialize()),
      #[cfg(feature = "dev")]
      _ => String::new(),
    },
    "keccak256" => match &public_key {
      CryptoPublicKey::Secp256k1(public_key) => {
        if ingredients.coin_index == 195 {
          hex::encode(public_key.serialize())
        } else {
          format!("0x{}", hex::encode(public_key.serialize()))
        }
      }
      #[cfg(feature = "dev")]
      _ => String::new(),
    },
    #[cfg(feature = "dev")]
    "ed25519" => match &public_key {
      CryptoPublicKey::Ed25519(public_key) => bs58::encode(public_key.to_bytes())
        .with_alphabet(bs58::Alphabet::DEFAULT)
        .into_string(),
      _ => String::new(),
    },
    _ => {
      return Err(format!("Unsupported hash method: {:?}", ingredients.hash));
    }
  };

  #[cfg(debug_assertions)]
  dbg!(&public_key_encoded);

  let address = match ingredients.hash.as_str() {
    "sha256" => generate_address_sha256(&public_key, &public_key_hash_vec),
    "keccak256" => {
      generate_address_keccak256(&public_key, &public_key_hash_vec, ingredients.coin_index)
    }
    "sha256+ripemd160" => match generate_sha256_ripemd160_address(
      ingredients.coin_index,
      &public_key,
      &public_key_hash_vec,
    ) {
      Ok(addr) => addr,
      Err(e) => {
        return Err(format!("Error generating address: {}", e));
      }
    },
    #[cfg(feature = "dev")]
    "ed25519" => crate::dev::generate_ed25519_address(&public_key),
    _ => {
      return Err(format!("Unsupported hash method: {:?}", ingredients.hash));
    }
  };

  #[cfg(debug_assertions)]
  dbg!(&address);

  let priv_key_wif = if ingredients.key_derivation == "ed25519" {
    bs58::encode(&derived_child_keys.0)
      .with_alphabet(bs58::Alphabet::DEFAULT)
      .into_string()
  } else {
    let compressed = true;
    create_private_key_for_address(
      Some(
        &secp256k1::SecretKey::from_slice(&derived_child_keys.0)
          .map_err(|e| format!("Invalid secret key: {:?}", e))?,
      ),
      Some(compressed),
      Some(&ingredients.wallet_import_format),
      &ingredients.hash,
      ingredients.coin_index,
    )
    .map_err(|e| format!("Failed to convert private key to WIF: {:?}", e))?
  };

  #[cfg(debug_assertions)]
  dbg!(&priv_key_wif);

  Ok(Some(Address {
    address: address.clone(),
    public_key: public_key_encoded.clone(),
    private_key: priv_key_wif.clone(),
  }))
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

pub fn convert_seed_to_mnemonic(seed: &[u8]) -> String {
  #[cfg(debug_assertions)]
  println!("[+] {}", &t!("log.convert_seed_to_mnemonic").to_string());

  let mut hex = String::with_capacity(128);

  for byte in seed.iter() {
    hex.push_str(&format!("{:02x}", byte));
  }

  hex
}
