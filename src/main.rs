// Crates
use std::{io::{self, Read, Seek, Write}, fs::{self, File}, path::Path, vec, str::FromStr, ops::Index};
use glib::{value::ValueType, PropertyGet, PropertySet};
use structopt::StructOpt;
use hex;
use rand::{Rng, RngCore};
use sha2::{Digest, Sha256};
use bitcoin::{self, hashes::sha256, Script};
use bip39;
use csv::ReaderBuilder;
use gtk4 as gtk;
use gtk::{
        ffi::{GtkCenterLayoutClass, GtkLabel}, 
        gdk::BUTTON_MIDDLE, 
        gio::{self, MenuItem, Menu}, 
        glib::{self, clone}, 
        prelude::*, 
        Stack, 
        StackSidebar, 
        StackTransitionType,
        EntryCompletion, ListStore, Label
    };

// Project files
mod error_handler;
use error_handler::CustomError;
mod converter;


// Global variables
const ENTROPY_FILE: &str = "entropy/test.qrn";
// const ENTROPY_FILE: &str = "./entropy/binary.qrn";
const WORDLIST_FILE: &str = "./lib/bip39-english.txt";
const COINLIST_FILE: &str = "./lib/bip44-coin_type.csv";
const VALID_ENTROPY_LENGTHS: [u32; 5] = [128, 160, 192, 224, 256];
const VALID_BIP_DERIVATIONS: [u32; 2] = [32, 44];
const VALID_MNEMONIC_WORD_COUNT: [u32; 5] = [12, 15, 18, 21, 24];
const VALID_ENTROPY_SOURCES: &'static [&'static str] = &["rng", "file"];
const APP_DESCRIPTION: Option<&str> = option_env!("CARGO_PKG_DESCRIPTION");
const APP_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
const APP_AUTHOR: Option<&str> = option_env!("CARGO_PKG_AUTHORS");


fn print_program_info() {
    println!(" ██████╗ ██████╗ ██████╗ ███╗   ███╗");
    println!("██╔═══██╗██╔══██╗╚════██╗████╗ ████║");
    println!("██║   ██║██████╔╝ █████╔╝██╔████╔██║");
    println!("██║▄▄ ██║██╔══██╗██╔═══╝ ██║╚██╔╝██║");
    println!("╚██████╔╝██║  ██║███████╗██║ ╚═╝ ██║");
    println!(" ╚══▀▀═╝ ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝");
    println!("{} ({})\n{}\n", &APP_DESCRIPTION.unwrap(), &APP_VERSION.unwrap(), &APP_AUTHOR.unwrap());
}


#[derive(Debug)]
struct CoinType {
    index: u32,
    path: u32,
    symbol: String,
    coin: String,
}

fn generate_entropy_from_rng(length: &u32) -> String {
    let mut rng = rand::thread_rng();
    let binary_string: String = (0..*length)
        .map(|_| rng.gen_range(0..=1))
        .map(|bit| char::from_digit(bit, 10).unwrap())
        .collect();

    binary_string
}

fn generate_entropy_from_file(file_path: &str, entropy_length: usize) -> String {
    let file = File::open(file_path);
    let mut reader = match file {
        Ok(file) => io::BufReader::new(file),
        Err(err) => {
            let error_msg = format!("Can not read entropy file: {}", err);
            // D3BUG!(error, "{}", error_msg);
            return String::new(); // Return default value or handle as appropriate
        }
    };

    let file_length = reader.seek(io::SeekFrom::End(0));
    let file_length = match file_length {
        Ok(length) => length,
        Err(err) => {
            let error_msg = format!("Error getting file length: {}", err);
            // D3BUG!(error, "{}", error_msg);
            return String::new(); // Return default value or handle as appropriate
        }
    };

    if file_length < entropy_length as u64 {
        let error_msg = format!("File too small for requested entropy length: {}", entropy_length);
        // D3BUG!(error, "{}", error_msg);
        return String::new(); // Return default value or handle as appropriate
    }

    let max_start = file_length.saturating_sub(entropy_length as u64);
    let start_point = rand::thread_rng().gen_range(0..=max_start);

    match reader.seek(io::SeekFrom::Start(start_point)) {
        Ok(_) => (),
        Err(err) => {
            let error_msg = format!("Error seeking in file: {}", err);
            // D3BUG!(error, "{}", error_msg);
            return String::new(); // Return default value or handle as appropriate
        }
    }

    let mut entropy_raw_binary = String::new();
    match reader.take(entropy_length as u64).read_to_string(&mut entropy_raw_binary) {
        Ok(_) => (),
        Err(err) => {
            let error_msg = format!("Error reading from file: {}", err);
            return String::new(); // Return default value or handle as appropriate
        }
    }

    entropy_raw_binary
}

fn calculate_checksum(entropy: &str, entropy_length: &u32) -> String {
    let entropy_binary = converter::convert_string_to_binary(&entropy);
    let hash_raw_binary: String = converter::convert_binary_to_string(&Sha256::digest(&entropy_binary));
    let checksum_lenght = entropy_length / 32;
    let checksum_raw_binary: String = hash_raw_binary.chars().take(checksum_lenght.try_into().unwrap()).collect();

    checksum_raw_binary
}

fn get_mnemonic_from_entropy(final_entropy_binary: &str) -> String {
    let chunks: Vec<String> = final_entropy_binary.chars().collect::<Vec<char>>().chunks(11).map(|chunk| chunk.iter().collect()).collect();
    let mnemonic_decimal: Vec<u32> = chunks.iter().map(|chunk| u32::from_str_radix(chunk, 2).unwrap()).collect();
    let mnemonic_file_content = fs::read_to_string(WORDLIST_FILE).expect("Can not read entropy file");
    let mnemonic_words: Vec<&str> = mnemonic_file_content.lines().collect();
    let mnemonic_words: Vec<&str> = mnemonic_decimal.iter().map(|&decimal| {
        if (decimal as usize) < mnemonic_words.len() {
            mnemonic_words[decimal as usize]
        } else {
            "INVALID_WORD"
        }
    }).collect();
    let final_mnemonic = mnemonic_words.join(" ");

    final_mnemonic
}

fn create_bip39_seed_from_entropy(entropy: &str, passphrase: &str) -> String {
    let entropy_vector = converter::convert_string_to_binary(&entropy);
    let mnemonic_result = bip39::Mnemonic::from_entropy(&entropy_vector).expect("Can not create mnemomic words");
    let mnemonic = mnemonic_result;
    let seed = bip39::Mnemonic::to_seed(&mnemonic, passphrase);
    let seed_hex = hex::encode(&seed[..]);

    seed_hex.to_string()
}

fn create_bip39_seed_from_mnemonic(mnemonic: &String, passphrase: &str) -> Result<String, CustomError> {
    let mnemonic_result = bip39::Mnemonic::from_str(&mnemonic);
    let mnemonic = mnemonic_result?;
    let seed = bip39::Mnemonic::to_seed(&mnemonic, passphrase);
    let seed_hex = hex::encode(&seed[..]);

    Ok(seed_hex)
}

fn create_master_private_key(seed_hex: String) -> String {
    let seed = hex::decode(seed_hex).expect("Failed to decode seed hex");
    let master_key = bitcoin::bip32::Xpriv::new_master(bitcoin::Network::Bitcoin, &seed).expect("Failed to derive master key");

    master_key.to_string()
}

fn create_coin_store() -> Vec<CoinType> {
    let file = File::open(&COINLIST_FILE).expect("can not open bip44 coin file");

    // Create a CSV reader
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    // Create a vector to store CoinType instances
    let mut coin_store = Vec::new();

    // Iterate over the CSV records and populate CoinType
    for result in rdr.records() {
        // Extract values from the CSV record
        let record = result.expect("error reading CSV record");
        let index = record[0].parse().expect("error parsing index");
        let path = u32::from_str_radix(&record[1][2..], 16).expect("error parsing path");
        let symbol = record[2].to_string();
        let coin = record[3].to_string();

        // Create a CoinType instance
        let coin_type = CoinType { index, path, symbol, coin };

        // Use the coin_type as needed (print or store in a collection, etc.)
        println!("{:?}", coin_type);

        // Add the CoinType to the vector
        coin_store.push(coin_type);
    }

    // Return the vector containing CoinType instances
    coin_store
}

fn search_coin_in_store<'a>(coin_store: &'a Vec<CoinType>, target_symbol: &'a str) -> Option<&'a CoinType> {
    for coin_type in coin_store {
        if coin_type.symbol == target_symbol {
            return Some(coin_type);
        }
    }
    None
}

fn create_derivation_path(
    cli_bip: u32,
    coin_type: u32,
    account: Option<u32>,
    change: Option<u32>,
    // index: Option<u32>,
) -> Result<Vec<bitcoin::bip32::ChildNumber>, bitcoin::bip32::Error> {
    let purpose = bitcoin::bip32::ChildNumber::from_hardened_idx(cli_bip)?;
    let coin_type = bitcoin::bip32::ChildNumber::from_hardened_idx(coin_type)?;
    let account = bitcoin::bip32::ChildNumber::from_hardened_idx(account.unwrap_or(0)).expect("Invalid child number");
    let change = bitcoin::bip32::ChildNumber::from_normal_idx(change.unwrap_or(0)).expect("Invalid child number");
    
    let derivation = match cli_bip {
        32 => vec![account, change],
        // 44 => vec![purpose, coin_type, account],
        44 => vec![purpose, coin_type, account, change],
        _ => vec![], // You may want to handle the case where bip is None
    };
    
    let mut path = String::from("m");
    
    for child_number in &derivation {
        path.push_str(&format!("/{}", child_number.to_string()));
    }

    Ok(derivation)
}

fn create_extended_private_key(
    master: &bitcoin::bip32::Xpriv,
    derivation: &Vec<bitcoin::bip32::ChildNumber>,
) -> Result<bitcoin::bip32::Xpriv, bitcoin::bip32::Error> {
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let bip32_xprv = master
        .derive_priv(&secp, &derivation)
        .expect("Failed to derive derivation private key");
    let mut modified_derivation = derivation.clone();
    modified_derivation.pop();

    let extended_key = master
        .derive_priv(&secp, &modified_derivation)
        .expect("Failed to derive extended private key");

    Ok(bip32_xprv)
}

fn create_extended_public_key(xprv: &bitcoin::bip32::Xpriv, index: Option<&u32>) -> Result<bitcoin::bip32::Xpub, CustomError> {
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let xpubkey = bitcoin::bip32::Xpub::from_priv(&secp, &xprv);
    
    Ok(xpubkey)
}

fn create_bitcoin_address(
    xprv: bitcoin::bip32::Xpriv,
    derivation: &Vec<bitcoin::bip32::ChildNumber>,
    count: &u32,
) -> Result<Vec<bitcoin::Address>, CustomError> {
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let mut addresses = Vec::new();

    for &format in ["p2pkh", "p2wpkh"].iter() {
        for index in 0..*count {
            let child = bitcoin::bip32::ChildNumber::from_hardened_idx(index)?;
            let child_xprv = xprv.derive_priv(&secp, &child)?;
            let child_pubkey = child_xprv.to_priv().public_key(&secp);

            let address = match format {
                "p2pkh" => bitcoin::Address::p2pkh(&child_pubkey, bitcoin::Network::Bitcoin),
                "p2wpkh" => bitcoin::Address::p2wpkh(&child_pubkey, bitcoin::Network::Bitcoin)?,
                "p2tr" => {
                    // Create a Secp256k1 context
                    let mut rng = rand::thread_rng();
                    let seckey_bytes: [u8; 32] = rng.gen();
                    let seckey =
                        secp256k1::SecretKey::from_slice(&seckey_bytes).expect("Invalid secret key");
                    let keypair = seckey.keypair(&secp);
                    let (xonly_public_key, parity) = bitcoin::XOnlyPublicKey::from_keypair(&keypair);

                    bitcoin::Address::p2tr(
                        &secp,
                        xonly_public_key,
                        None,
                        bitcoin::Network::Bitcoin,
                    )
                }
                _ => return Err(CustomError::New(format.to_string())),
            };

            let mut path = String::from("m");
            for child_number in derivation {
                path.push_str(&format!("/{}", child_number.to_string()));
            }

            addresses.push(address);
        }
    }

    Ok(addresses)
}

fn create_coin_completion_model() -> gtk::ListStore {
    let valid_coin_symbols = read_csv("lib/bip44-coin_type.csv");

    let store = gtk::ListStore::new(&[
        glib::Type::U32, // Index
        glib::Type::U32, // Path
        glib::Type::STRING, // Symbol
        glib::Type::STRING, // Coin
    ]);

    for coin_symbol in valid_coin_symbols.iter() {
        let iter = store.append();
        store.set(&iter, &[(0, &coin_symbol.index), (1, &coin_symbol.path), (2, &coin_symbol.symbol), (3, &coin_symbol.coin)]);
    }

    store
}

// fn create_tree_view() -> (gtk::TreeView, gtk::ListStore) {
//     let store = create_coin_completion_model();

//     let tree_view = gtk::TreeView::with_model(&store);
//     tree_view.set_vexpand(true);
//     tree_view.set_headers_visible(true);

//     // Add columns to the TreeView
//     let columns = ["Index", "Path", "Symbol", "Coin"];
//     for (i, column_title) in columns.iter().enumerate() {
//         let column = gtk::TreeViewColumn::new();
//         let cell = gtk::CellRendererText::new();
//         column.set_title(column_title);
//         column.pack_start(&cell, true);
//         column.add_attribute(&cell, "text", i as i32);
//         tree_view.append_column(&column);
//     }

//     (tree_view, store)
// }

fn gtk4_create_main_menu(app: &gtk::Application) {
    let about = gio::ActionEntry::builder("about")
        .activate(|_, _, _| println!("About was pressed"))
        .build();

    let quit = gio::ActionEntry::builder("quit")
        .activate(|app: &gtk::Application, _, _| app.quit())
        .build();

    app.add_action_entries([about, quit]);

    let menubar = {
        let wallet_menu = {
            let open_menu_item = MenuItem::new(Some("Open"), Some("app.open"));
            let save_menu_item = MenuItem::new(Some("Save"), Some("app.save"));
            let quit_menu_item = MenuItem::new(Some("Quit"), Some("app.quit"));
            
            let wallet_menu = Menu::new();
            wallet_menu.append_item(&open_menu_item);
            wallet_menu.append_item(&save_menu_item);
            wallet_menu.append_item(&quit_menu_item);
            wallet_menu
        };

        let entropy_menu = {
            let new_menu_item = MenuItem::new(Some("New"), Some("app.new_entropy"));

            let entropy_menu = Menu::new();
            entropy_menu.append_item(&new_menu_item);
            entropy_menu
        };

        let help_menu = {
            let about_menu_item = MenuItem::new(Some("About"), Some("app.about"));

            let help_menu = Menu::new();
            help_menu.append_item(&about_menu_item);
            help_menu
        };


        let menubar = Menu::new();
        menubar.append_submenu(Some("Wallet"), &wallet_menu);
        menubar.append_submenu(Some("Entropy"), &entropy_menu);
        menubar.append_submenu(Some("Help"), &help_menu);

        menubar
    };

    app.set_menubar(Some(&menubar));
}


fn create_GUI(application: &gtk::Application) {
    let title = format!("{} {}", APP_DESCRIPTION.unwrap(), APP_VERSION.unwrap());

    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title(&title)
        .default_width(800)
        .default_height(600)
        .show_menubar(true)
        .build();

    let header_bar = gtk::HeaderBar::new();
    window.set_titlebar(Some(&header_bar));

    let new_wallet_button = gtk::Button::new();
    new_wallet_button.set_icon_name("tab-new-symbolic");
    header_bar.pack_start(&new_wallet_button);

    let open_wallet_button = gtk::Button::new();
    open_wallet_button.set_icon_name("document-open-symbolic");
    header_bar.pack_start(&open_wallet_button);

    let save_wallet_button = gtk::Button::new();
    save_wallet_button.set_icon_name("document-save-symbolic");
    header_bar.pack_start(&save_wallet_button);

    let settings_button = gtk::Button::new();
    settings_button.set_icon_name("org.gnome.Settings-symbolic");
    header_bar.pack_end(&settings_button);

    // Create a Stack and a StackSidebar
    let stack = Stack::new();
    let stack_sidebar = StackSidebar::new();
    stack_sidebar.set_stack(&stack);

    // SEED SIDEBAR
    let entropy_main_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    entropy_main_box.set_margin_top(10);
    entropy_main_box.set_margin_start(10);
    entropy_main_box.set_margin_end(10);
    entropy_main_box.set_margin_bottom(10);

    // Header
    let entropy_header_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let entropy_header_first_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let entropy_header_second_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);

    // Entropy source
    let entropy_source_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let entropy_source_frame = gtk::Frame::new(Some("Entropy source"));
    let valid_entropy_source_as_strings: Vec<String> = VALID_ENTROPY_SOURCES.iter().map(|&x| x.to_string()).collect();
    let valid_entropy_source_as_str_refs: Vec<&str> = valid_entropy_source_as_strings.iter().map(|s| s.as_ref()).collect();
    let entropy_source_dropdown = gtk::DropDown::from_strings(&valid_entropy_source_as_str_refs);
    entropy_source_box.set_hexpand(true);
    entropy_source_frame.set_hexpand(true);
    
    // Entropy length
    let entropy_length_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let entropy_length_frame = gtk::Frame::new(Some("Entropy length"));
    let valid_entropy_lengths_as_strings: Vec<String> = VALID_ENTROPY_LENGTHS.iter().map(|&x| x.to_string()).collect();
    let valid_entropy_lengths_as_str_refs: Vec<&str> = valid_entropy_lengths_as_strings.iter().map(|s| s.as_ref()).collect();
    let entropy_length_dropdown = gtk::DropDown::from_strings(&valid_entropy_lengths_as_str_refs);
    entropy_length_box.set_hexpand(true);
    entropy_length_frame.set_hexpand(true);
    entropy_length_dropdown.set_selected(4);

    // Mnemonic passphrase
    let mnemonic_passphrase_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let mnemonic_passphrase_frame = gtk::Frame::new(Some("Mnemonic passphrase"));
    let mnemonic_passphrase_text = gtk::Entry::new();
    mnemonic_passphrase_box.set_hexpand(true);
    mnemonic_passphrase_text.set_hexpand(true);
    
    // Generate button
    let generate_wallet_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let generate_wallet_button = gtk::Button::new();
    generate_wallet_button.set_label("Generate");
    generate_wallet_box.set_halign(gtk::Align::Center);

    // Connections
    entropy_source_frame.set_child(Some(&entropy_source_dropdown));
    entropy_length_frame.set_child(Some(&entropy_length_dropdown));

    generate_wallet_box.append(&generate_wallet_button);
    entropy_source_box.append(&entropy_source_frame);
    entropy_length_box.append(&entropy_length_frame);
    entropy_header_first_box.append(&entropy_source_box);
    entropy_header_first_box.append(&entropy_length_box);
    entropy_header_second_box.append(&mnemonic_passphrase_box);
    entropy_header_box.append(&entropy_header_first_box);
    entropy_header_box.append(&entropy_header_second_box);
    entropy_header_box.append(&generate_wallet_box);

    // Body
    let body_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    
    // Entropy string
    let entropy_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let entropy_frame = gtk::Frame::new(Some("Entropy"));
    let entropy_text = gtk::TextView::new();
    entropy_text.set_vexpand(true);
    entropy_text.set_hexpand(true);
    entropy_text.set_wrap_mode(gtk::WrapMode::Char);
    entropy_frame.set_child(Some(&entropy_text));
    entropy_box.append(&entropy_frame);
    entropy_text.set_editable(false);
    entropy_text.set_left_margin(5);
    entropy_text.set_top_margin(5);
    
    // Mnemonic words
    let mnemonic_words_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let mnemonic_words_frame = gtk::Frame::new(Some("Mnemonic words"));
    let mnemonic_words_text = gtk::TextView::new();
    mnemonic_words_box.set_hexpand(true);
    mnemonic_words_text.set_vexpand(true);
    mnemonic_words_text.set_hexpand(true);
    mnemonic_words_text.set_editable(false);
    mnemonic_words_text.set_left_margin(5);
    mnemonic_words_text.set_top_margin(5);
    mnemonic_words_text.set_wrap_mode(gtk::WrapMode::Word);

    // Seed
    let seed_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let seed_frame = gtk::Frame::new(Some("Seed"));
    let seed_text = gtk::Entry::new();
    seed_text.set_editable(false);

    // Connections
    mnemonic_words_frame.set_child(Some(&mnemonic_words_text));
    mnemonic_passphrase_frame.set_child(Some(&mnemonic_passphrase_text));
    seed_frame.set_child(Some(&seed_text));

    mnemonic_words_box.append(&mnemonic_words_frame);
    mnemonic_passphrase_box.append(&mnemonic_passphrase_frame);
    seed_box.append(&seed_frame);
    body_box.append(&entropy_box);
    body_box.append(&mnemonic_words_box);
    body_box.append(&seed_box);
    entropy_main_box.append(&entropy_header_box);
    entropy_main_box.append(&body_box);

    
    // + NEW IMPLEMENTATION
    // F5 is generate new seed

    // clone for coin tab
    let cloned_seed_text = seed_text.clone();
    
    // Generate seed button
    generate_wallet_button.connect_clicked(clone!(
        @strong entropy_source_dropdown,
        @strong entropy_length_dropdown,
        @strong mnemonic_words_text => move |_| {
            let selected_entropy_source_index = entropy_source_dropdown.selected() as usize;
            let selected_entropy_length_index = entropy_length_dropdown.selected() as usize;
            let selected_entropy_source_value = VALID_ENTROPY_SOURCES.get(selected_entropy_source_index);
            let selected_entropy_length_value = VALID_ENTROPY_LENGTHS.get(selected_entropy_length_index);


            println!("source: {}", selected_entropy_source_value.unwrap().to_string());
            println!("Length: {}", selected_entropy_length_value.unwrap());

            // let converted_value: usize = *selected_entropy_length_value as usize;

            let entropy_length = selected_entropy_length_value;
            let mut pre_entropy: String = "".to_string();
            let mut full_entropy: String = "".to_string();

            match selected_entropy_source_value {
                Some(selected_source) => {
                    match selected_source {
                        &"rng" => {
                            if let Some(length) = entropy_length {
                                pre_entropy = generate_entropy_from_rng(&length);
                                // let checksum = calculate_checksum(&result, length);
                            } else {
                                // Handle the case where entropy_length is None
                            }
                        }
                        &"file" => {
                            if let Some(length) = entropy_length {
                                pre_entropy = generate_entropy_from_file(ENTROPY_FILE, *length as usize);
                                // full_entropy = get_full_entropy(&result, &checksum);
                            } else {
                                // Handle the case where entropy_length is None
                            }
                        }
                        _ => {
                            // Handle other cases or do nothing
                        }
                    }
                }
                None => {
                    // Handle the case where selected_entropy_source_value is None
                }
            }
            
            let checksum = calculate_checksum(&pre_entropy, &entropy_length.unwrap());
            full_entropy = format!("{}{}", &pre_entropy, &checksum);
            entropy_text.buffer().set_text(&full_entropy);


            let mnemonic_words = get_mnemonic_from_entropy(&full_entropy);
            mnemonic_words_text.buffer().set_text(&mnemonic_words);


            let passphrase_text = mnemonic_passphrase_text.text().to_string();
            println!("pass: {}", &passphrase_text);

            let seed = create_bip39_seed_from_entropy(&pre_entropy, &passphrase_text);
            seed_text.buffer().set_text(&seed);
        }
    ));

    // Start Seed sidebar
    stack.add_titled(&entropy_main_box, Some("sidebar-seed"), "Seed");
    
    // Sidebar Coin
    let coin_main_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    coin_main_box.set_margin_top(10);
    coin_main_box.set_margin_start(10);
    coin_main_box.set_margin_end(10);
    coin_main_box.set_margin_bottom(10);

    
    let coin_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let coin_frame = gtk::Frame::new(Some("Coin"));
    
    // Coin section
    // let coin_tree_view = create_tree_view();
    let store = create_coin_completion_model();

    // let coin_treeview = gtk::TreeView::with_model(&store);
    let coin_treeview = gtk::TreeView::new();
    coin_treeview.set_vexpand(true);
    coin_treeview.set_headers_visible(true);

    // Add columns to the TreeView
    let columns = ["Index", "Path", "Symbol", "Coin"];
    for (i, column_title) in columns.iter().enumerate() {
        let column = gtk::TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();
        column.set_title(column_title);
        column.pack_start(&cell, true);
        column.add_attribute(&cell, "text", i as i32);
        coin_treeview.append_column(&column);
    }


    let coin_search = gtk::SearchEntry::new();
    let coin_label = gtk4::Label::builder()
        .label("Type coin symbol to start")
        // .vexpand(true)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .css_classes(["large-title"])
        .build();


    // Coins
    let coins = gtk::Box::new(gtk::Orientation::Vertical, 20);
    coins.append(&coin_search);
    coins.append(&coin_label);
    coins.append(&coin_treeview);
    coin_frame.set_child(Some(&coins));

    // Derivation path
    let derivation_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let derivation_frame = gtk::Frame::new(Some("Derivation path"));
    let valid_derivation_path_as_string: Vec<String> = VALID_BIP_DERIVATIONS.iter().map(|&x| x.to_string()).collect();
    let valid_derivation_path_as_str_refs: Vec<&str> = valid_derivation_path_as_string.iter().map(|s| s.as_ref()).collect();
    let derivation_dropdown = gtk::DropDown::from_strings(&valid_derivation_path_as_str_refs);
    derivation_frame.set_child(Some(&derivation_dropdown));
    derivation_frame.set_hexpand(true);
    derivation_dropdown.set_selected(1);
    
    
    // Hardened path
    let hardened_frame = gtk::Frame::new(Some("Hardened path"));
    let hardened_checkbox = gtk4::CheckButton::new();
    hardened_frame.set_child(Some(&hardened_checkbox));
    hardened_frame.set_hexpand(true);
    

    
    
    
    
    // Master private key
    let master_private_key_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let master_private_key_frame = gtk::Frame::new(Some("Master private key"));
    let master_private_key_text = gtk::TextView::new();
    master_private_key_text.set_editable(false);
    master_private_key_frame.set_child(Some(&master_private_key_text));


    // Connections 
    coin_box.append(&coin_frame);
    derivation_box.append(&derivation_frame);
    derivation_box.append(&hardened_frame);
    master_private_key_box.append(&master_private_key_frame);
    coin_main_box.append(&coin_box);
    coin_main_box.append(&derivation_box);
    coin_main_box.append(&master_private_key_box);
    
    let coin_store = create_coin_store();
    coin_search.connect_search_changed(clone!(@weak coin_label => move |coin_search| {
        if coin_search.text() != "" {

            let target_symbol = coin_search.text().to_uppercase().to_string();
        
            if let Some(found_coin) = search_coin_in_store(&coin_store, &target_symbol) {
                println!("Coin found: {:?}", found_coin);
                coin_label.set_text("Coin found");

                let master_priv = create_master_private_key(cloned_seed_text.text().to_string());
                master_private_key_text.buffer().set_text(&master_priv.to_string());
                
                // Refresh the TreeView
                let treestore = gtk4::TreeStore::new(&[glib::Type::STRING; 4]);
                coin_treeview.set_model(Some(&treestore));
            
                // Add some sample data to the TreeStore
                let data = vec![
                    ("11","0x8000000b","NSR","NuShares"),
                ];
            
                for item in data {
                    let iter = treestore.append(None);
                    treestore.set(&iter, &[(0, &item.0), (1, &item.1), (2, &item.2), (3, &item.3)]);
                }

            } else {
                let msg = format!("Coin with symbol {} not found.", target_symbol);
                coin_label.set_text(&msg);
                master_private_key_text.buffer().set_text("");
                // treestore.set
            }
        } else {
            coin_label.set_text("Search for a coin symbol");
        }
    }));
    
    
    
    
    // Start: Coins
    stack.add_titled(&coin_main_box, Some("sidebar-coin"), "Coin");



    // Create a Box to hold the main content and sidebar
    let main_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    main_content_box.append(&stack_sidebar);
    main_content_box.append(&stack);
    window.set_child(Some(&main_content_box));

    window.present();
}


impl std::fmt::Display for CoinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Implement how you want CoinType to be formatted when using {}
        write!(f, "{}, {}, {}, {}", self.index, self.path, self.symbol, self.coin)
    }
}

fn read_csv(file_path: &str) -> Vec<CoinType> {
    let file = File::open(&COINLIST_FILE).expect("can not read file");
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

    let coin_types: Vec<CoinType> = rdr
        .records()
        .filter_map(|record| record.ok())
        .enumerate()
        .map(|(index, record)| {
            let path = index as u32;
            let index = index.try_into().expect("Conversion from usize to u32 failed");
            let symbol = record.get(2).unwrap_or_default().to_string();
            let coin = record.get(3).unwrap_or_default().to_string();
            CoinType { index, path, symbol, coin }
            }
        )
        .collect();

    coin_types

}


fn main() {
    print_program_info();
    
    // D3BUG!(log, "Starting GUI");
    // Create a new application
    let application = gtk::Application::builder()
        .application_id("com.github.qr2m")
        .build();
    application.connect_activate(create_GUI);

    // Create quit
    let quit = gio::SimpleAction::new("quit", None);
    quit.connect_activate(
        glib::clone!(@weak application => move |_action, _parameter| {
            application.quit();
        }),
    );
    application.set_accels_for_action("app.quit", &["<Primary>Q"]);
    application.add_action(&quit);

    application.run();
}

