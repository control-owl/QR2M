// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// license = "CC-BY-NC-ND-4.0  [2023-2025]  Control Owl"

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

use gtk::{gdk_pixbuf, glib, prelude::*};
use gtk4 as gtk;
use include_dir::{Dir, include_dir};
use sha2::{Digest, Sha256, Sha512};
use std::io::{self, Write};

const APP_DEFAULT_BUTTON_HEIGHT: u8 = 24;
const APP_DEFAULT_BUTTON_WIDTH: u8 = 24;
const APP_IMAGE_BITS: u8 = 8;
const APP_IMAGE_HAS_ALPHA: bool = true;

pub static RES_DIR: Dir<'_> = include_dir!("res");

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
    .map(|chunk| {
      chunk
        .iter()
        .fold(0, |acc, &bit| (acc << 1) | (bit as u8 - b'0'))
    })
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
  hasher.update(first_hash);

  hasher.finalize().to_vec()
}

pub fn calculate_sha256_and_ripemd160_hash(input: &[u8]) -> Vec<u8> {
  let mut hasher = Sha256::new();
  hasher.update(input);
  let hash = hasher.finalize();

  use ripemd::Digest;
  let mut ripemd = ripemd::Ripemd160::new();
  ripemd.update(hash);

  ripemd.finalize().to_vec()
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
  hasher.update(inner_hash);
  let final_hash = hasher.finalize().to_vec();

  assert_eq!(final_hash.len(), HASH_SIZE, "Final hash length mismatch");

  final_hash
}

pub fn calculate_checksum_for_master_keys(data: &[u8]) -> [u8; 4] {
  let hash = Sha256::digest(data);
  let double_hash = Sha256::digest(hash);
  let mut checksum = [0u8; 4];
  checksum.copy_from_slice(&double_hash[..4]);
  checksum
}

pub fn calculate_checksum_for_entropy(entropy: &str) -> String {
  let entropy_binary = convert_string_to_binary(entropy);
  let hash_raw_binary: String = convert_binary_to_string(&Sha256::digest(&entropy_binary));

  let checksum_length = match entropy.len() {
    128 => 4,
    160 => 5,
    192 => 6,
    224 => 7,
    256 => 8,
    _ => {
      eprintln!("Wrong entropy length! Checksum not done");
      0
    }
  };

  hash_raw_binary
    .chars()
    .take(checksum_length.try_into().unwrap_or_default())
    .collect()
}

pub fn is_valid_entropy(full_entropy: &str) -> bool {
  let (entropy_len, checksum_len) = match full_entropy.len() {
    132 => (128, 4),
    165 => (160, 5),
    198 => (192, 6),
    231 => (224, 7),
    264 => (256, 8),
    _ => return false,
  };

  let (entropy, checksum) = full_entropy.split_at(entropy_len);

  let calculated_checksum = calculate_checksum_for_entropy(entropy);

  if calculated_checksum != checksum {
    return false;
  }

  entropy.len() == entropy_len
    && checksum.len() == checksum_len
    && full_entropy.chars().all(|c| c == '0' || c == '1')
}

pub fn is_valid_seed(seed: &str) -> bool {
  if seed.len() != 128 {
    return false;
  }

  if seed.is_empty() {
    return false;
  }

  seed.chars().all(|c| c.is_ascii_hexdigit())
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

pub fn get_text_from_resources(file_name: &str) -> String {
  match RES_DIR.get_file(file_name) {
    Some(file) => match std::str::from_utf8(file.contents()) {
      Ok(text) => {
        // #[cfg(debug_assertions)]
        // println!("text: {:?}", text);

        text.to_string()
      }
      Err(err) => {
        eprintln!("Failed to read {file_name} as UTF-8: {err}");
        String::new()
      }
    },
    None => {
      eprintln!("Failed to get {file_name} from embedded resources");
      String::new()
    }
  }
}

pub fn get_picture_from_resources(image_name: &str) -> gtk::Picture {
  match RES_DIR.get_file(image_name) {
    Some(file) => {
      let image_data = file.contents();
      let image_bytes = glib::Bytes::from_static(image_data);
      let loader = gdk_pixbuf::PixbufLoader::new();

      if loader.write(&image_bytes).is_ok() {
        match loader.close() {
          Ok(_) => {}
          Err(error) => eprintln!("\t- ERROR problem with loader:\n\t{error}"),
        };

        let texture = gtk::gdk::Texture::from_bytes(&image_bytes)
          .map_err(|err| format!("Failed to create texture: {err}"))
          .unwrap();

        let picture = gtk::Picture::for_paintable(&texture);

        picture.set_size_request(
          APP_DEFAULT_BUTTON_WIDTH as i32,
          APP_DEFAULT_BUTTON_HEIGHT as i32,
        );

        return picture;
      }
      generate_empty_picture()
    }
    None => {
      eprintln!("Failed to get {image_name} from embedded resources");
      generate_empty_picture()
    }
  }
}

pub fn get_texture_from_resource(image_name: &str) -> gtk::gdk::Texture {
  match RES_DIR.get_file(image_name) {
    Some(file) => {
      let image_data = file.contents();
      let image_bytes = glib::Bytes::from_static(image_data);
      let loader = gdk_pixbuf::PixbufLoader::new();

      if loader.write(&image_bytes).is_ok() {
        match loader.close() {
          Ok(_) => {}
          Err(err) => eprintln!(" - [!] ERROR problem with loading SVG icons:\n\t{err:?}"),
        };

        if let Some(pixbuf) = loader.pixbuf() {
          return gtk::gdk::Texture::for_pixbuf(&pixbuf);
        }
      }
      generate_empty_texture()
    }
    None => {
      eprintln!("Failed to get {image_name} from embedded resources");
      generate_empty_texture()
    }
  }
}

pub fn get_file_from_resources(file_name: &str) -> Result<&'static include_dir::File, String> {
  RES_DIR
    .get_file(file_name)
    .ok_or_else(|| format!("File '{file_name}' not found in resources"))
}

pub fn generate_empty_picture() -> gtk::Picture {
  let empty_pixbuf = gdk_pixbuf::Pixbuf::new(
    gdk_pixbuf::Colorspace::Rgb,
    APP_IMAGE_HAS_ALPHA,
    APP_IMAGE_BITS as i32,
    APP_DEFAULT_BUTTON_WIDTH as i32,
    APP_DEFAULT_BUTTON_HEIGHT as i32,
  )
  .expect("Failed to create empty pixbuf");

  empty_pixbuf.fill(0x070410FF);

  let picture = gtk::Picture::new();

  picture.set_size_request(
    APP_DEFAULT_BUTTON_WIDTH as i32,
    APP_DEFAULT_BUTTON_HEIGHT as i32,
  );

  picture.add_css_class("empty-image");
  picture
}

pub fn generate_empty_texture() -> gtk::gdk::Texture {
  let empty_pixbuf = gdk_pixbuf::Pixbuf::new(
    gdk_pixbuf::Colorspace::Rgb,
    APP_IMAGE_HAS_ALPHA,
    APP_IMAGE_BITS as i32,
    APP_DEFAULT_BUTTON_WIDTH as i32,
    APP_DEFAULT_BUTTON_HEIGHT as i32,
  )
  .expect("Failed to create empty pixbuf");

  empty_pixbuf.fill(0x070410FF);

  gtk::gdk::Texture::for_pixbuf(&empty_pixbuf)
}

pub fn setup_css() {
  let provider = gtk::CssProvider::new();

  let css_theme = match RES_DIR.get_file(std::path::Path::new("theme").join("style.css")) {
    Some(css_file) => css_file.contents_utf8().unwrap_or_default(),
    None => {
      eprintln!("CSS theme file not found");
      ""
    }
  };

  provider.load_from_string(css_theme);

  gtk::style_context_add_provider_for_display(
    &gtk::gdk::Display::default().expect("Error initializing display"),
    &provider,
    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
  );
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

pub fn save_config_to_file(
  local_config_file: &std::path::PathBuf,
  toml_str: &str,
) -> io::Result<()> {
  let mut file = std::fs::File::create(local_config_file).map_err(|_err| {
    #[cfg(debug_assertions)]
    eprintln!("\t- Failed to create config file: {_err}");
    io::Error::other("Failed to create config file")
  })?;

  file.write_all(toml_str.as_bytes()).map_err(|_err| {
    #[cfg(debug_assertions)]
    eprintln!("\t- Failed to write to config file: {_err}");

    io::Error::other("Failed to write to config file")
  })?;

  #[cfg(debug_assertions)]
  println!("\t- Config file written successfully: {local_config_file:?}");

  Ok(())
}

pub fn read_config_from_file(local_config_file: &std::path::PathBuf) -> io::Result<String> {
  std::fs::read_to_string(local_config_file).map_err(|_err| {
    #[cfg(debug_assertions)]
    eprintln!("\t- Failed to read config file: {_err}");

    io::Error::new(io::ErrorKind::NotFound, "Failed to read config file")
  })
}

pub fn get_active_app_feature() -> &'static str {
  if cfg!(feature = "dev") {
    "dev"
  } else if cfg!(feature = "full") {
    "full"
  } else {
    "offline"
  }
}
