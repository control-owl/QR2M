// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// module = "Development playground"
// copyright = "Copyright © 2023-2025 Control Owl"
// version = "2025-01-17"


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


use sha2::{Digest, Sha256};
use ed25519_dalek::{SigningKey, VerifyingKey};
use gtk4 as gtk;
use libadwaita as adw;
use adw::prelude::*;

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

// SOLANA

pub fn derive_from_path_ed25519(
    master_key: &[u8],
    master_chain_code: &[u8],
    path: &str,
) -> crate::keys::DerivationResult {
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
) -> crate::keys::DerivationResult {
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
        eprintln!("len is not 64, it is: {}", result.len());
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

pub fn generate_ed25519_address(public_key: &crate::keys::CryptoPublicKey) -> String {
    let public_key_bytes = match public_key {
        crate::keys::CryptoPublicKey::Ed25519(key) => key.to_bytes().to_vec(),
        _ => {
            eprintln!("generate_ed25519_address called with non-ed25519 key");
            Vec::new()
        }
    };
    
    let hash = Sha256::digest(&public_key_bytes);
    bs58::encode(hash).with_alphabet(bs58::Alphabet::BITCOIN).into_string()
}


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

// NEW ANU LOGIC

const QRNG_KEY_LEVEL: usize = 14;
const QRNG_MAGIC_NUMBER: usize = 1024 * QRNG_KEY_LEVEL;


fn get_qrng() -> String {
    use rand::{Rng, rng};

    let mut rng = rng();
    let length = rng.random_range(2..=QRNG_MAGIC_NUMBER);

    let hex_chars: String = (0..length)
        .map(|_| {
            let random_char = rng.random_range(0..16);
            match random_char {
                0..=9 => (b'0' + random_char as u8) as char,
                10..=15 => (b'A' + (random_char - 10) as u8) as char, // A-F
                _ => unreachable!(),
            }
        })
        .collect();

    println!("Generated String: {}", hex_chars);
    hex_chars
}

pub fn anu_window() {
    let app = gtk::Application::builder()
        .application_id("wtf.r_o0_t.qr2m.qrng_checker")
        .build();

    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("QRNG Checker")
            .default_width(1024)
            .default_height(768)
            .build();

        let main_grid_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let scroll_window = gtk::ScrolledWindow::new();
        scroll_window.set_hexpand(true);
        scroll_window.set_vexpand(true);

        let grid = gtk::Grid::builder()
            .column_spacing(3)
            .row_spacing(3)
            .margin_start(5)
            .margin_end(5)
            .margin_top(5)
            .margin_bottom(5)
            .build();

        scroll_window.set_child(Some(&grid));
        main_grid_box.append(&scroll_window);

        let main_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let ok_button = gtk::Button::with_label("OK");
        let cancel_button = gtk::Button::with_label("Cancel");
        let new_button = gtk::Button::with_label("New QRNG");

        main_button_box.append(&ok_button);
        main_button_box.append(&new_button);
        main_button_box.append(&cancel_button);

        main_button_box.set_margin_bottom(4);
        main_button_box.set_margin_top(4);
        main_button_box.set_margin_start(4);
        main_button_box.set_margin_end(4);

        main_grid_box.append(&main_button_box);

        let mut boxes = Vec::new();

        for i in 0..QRNG_MAGIC_NUMBER {
            let small_box = gtk::Box::builder()
                .width_request(7)
                .height_request(7)
                .build();

            small_box.set_css_classes(&["empty-box"]);
            // IMPLEMENT: Get window size, then calculate maximum boxes per row
            grid.attach(&small_box, (i % 150) as i32, (i / 150) as i32, 1, 1);
            boxes.push(small_box);
        }

        let recolor_boxes = std::rc::Rc::new(std::cell::RefCell::new({
            let boxes = boxes.clone();
            move || {
                let qrng_string = get_qrng();
                let qrng_length = qrng_string.len();
                println!("New QRNG Length: {}", qrng_length);

                for (i, small_box) in boxes.iter().enumerate() {
                    if i < qrng_length {
                        small_box.set_css_classes(&["green-box"]);
                    } else {
                        small_box.set_css_classes(&["empty-box"]);
                    }
                }

                if qrng_length == QRNG_MAGIC_NUMBER {
                    println!("Done");
                } else {
                    println!("Not enough");
                }
            }
        }));

        {
            let recolor_boxes = recolor_boxes.clone();
            new_button.connect_clicked(move |_| {
                recolor_boxes.borrow_mut()();
            });
        }

        window.set_child(Some(&main_grid_box));
        window.show();
    });

    app.run();
}
