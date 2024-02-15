// #![allow(dead_code)]
// #![allow(unused_variables)]
// #![allow(unused_imports)]
#![allow(non_snake_case)]


// Crates
use std::{
    fs::{
        self, 
        File
    }, io::{
        self, 
        Read, 
        Seek
    }
};
use hex;
use rand::Rng;
use sha2::{Digest, Sha256, Sha512};
use bip39;
use csv::ReaderBuilder;
use gtk4 as gtk;
use gtk::{
        gio, 
        glib::clone, 
        prelude::*, 
        Stack, 
        StackSidebar, 
    };

// Project files
// mod converter;


// Global variables
// const ENTROPY_FILE: &str = "./entropy/binary.qrn";
// const COINLIST_FILE: &str = "./lib/bip44-coin_type.csv";
const ENTROPY_FILE: &str = "./entropy/test.qrn";
const WORDLIST_FILE: &str = "./lib/bip39-mnemonic-words-english.txt";
const COINLIST_FILE: &str = "./lib/bip44-extended-coin-list.csv";

const VALID_ENTROPY_LENGTHS: [u32; 5] = [128, 160, 192, 224, 256];
const VALID_BIP_DERIVATIONS: [u32; 2] = [32, 44];
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
struct CoinDatabase {
    index: u32,
    path: u32,
    symbol: String,
    name: String,
    key_derivation: String,
    private_version: String,
    public_version: String,
    pubKeyHash: String,
    scriptHash: String,
    wif: String,
    comment: String,
}



fn convert_binary_to_string(input_value: &[u8]) -> String {
    input_value
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |i| ((byte >> i) & 1).to_string()))
        .collect()
}

fn convert_string_to_binary(input_value: &str) -> Vec<u8> {
    input_value
        .chars()
        .collect::<Vec<char>>()
        .chunks(8)
        .map(|chunk| {
            chunk.iter().fold(0, |acc, &bit| (acc << 1) | (bit as u8 - '0' as u8))
        })
        .collect()
}



fn generate_entropy(source: &str, length: u64, file_name: Option<&str>) -> String {
    match source {
        "rng" => {
            let mut rng = rand::thread_rng();
            let binary_string: String = (0..length)
                .map(|_| rng.gen_range(0..=1))
                .map(|bit| char::from_digit(bit, 10).unwrap())
                .collect();

            binary_string
        },
        "file" => {
            let file = File::open(&file_name.unwrap()).expect("Can not open entropy file.");
            let mut reader = io::BufReader::new(file);
            
            let file_length = reader.seek(io::SeekFrom::End(0)).unwrap_or_else(|err| {
                eprintln!("Error getting file length: {}", err);
                panic!();
            });
            
            if file_length < length {
                eprintln!("File too small for requested entropy length: {}", length);
            }

            let max_start = file_length.saturating_sub(length as u64);
            let start_point = rand::thread_rng().gen_range(0..=max_start);

            if let Err(err) = reader.seek(io::SeekFrom::Start(start_point)) {
                eprintln!("Error seeking in file: {}", err);
            }

            let mut entropy_raw_binary = String::new();
            if let Err(err) = reader.take(length as u64).read_to_string(&mut entropy_raw_binary) {
                eprintln!("Error reading from file: {}", err);
            }
            entropy_raw_binary
        },
        _ => panic!("Invalid source specified"), // Handle any other cases
    }
}

fn generate_checksum(entropy: &str, entropy_length: &u32) -> String {
    let entropy_binary = convert_string_to_binary(&entropy);
    let hash_raw_binary: String = convert_binary_to_string(&Sha256::digest(&entropy_binary));
    let checksum_lenght = entropy_length / 32;
    let checksum: String = hash_raw_binary.chars().take(checksum_lenght.try_into().unwrap()).collect();

    checksum
}

fn generate_mnemonic_words(final_entropy_binary: &str) -> String {
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

fn generate_bip39_seed(entropy: &str, passphrase: &str) -> [u8; 64] {
    let entropy_vector = convert_string_to_binary(&entropy);
    let mnemonic_result = bip39::Mnemonic::from_entropy(&entropy_vector).expect("Can not create mnemomic words");
    let mnemonic = mnemonic_result;
    let seed = bip39::Mnemonic::to_seed(&mnemonic, passphrase);

    seed
}

fn create_coin_store() -> Vec<CoinDatabase> {
    let file = File::open(&COINLIST_FILE).expect("can not open bip44 coin file");
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
    let mut coin_store = Vec::new();

    for result in rdr.records() {
        let record = result.expect("error reading CSV record");

        let index: u32 = record[0].parse().expect("error parsing index");
        let path: u32 = u32::from_str_radix(&record[1][2..], 16).expect("error parsing path");
        let symbol: String = if record[2].is_empty() {"".to_string()} else {record[2].to_string()};
        let name: String = if record[3].is_empty() {"".to_string()} else {record[3].to_string()};
        let key_derivation:String = if record[4].is_empty() {"".to_string()} else {record[4].to_string()};
        let private_version: String = if record[5].is_empty() {"".to_string()} else {record[5].to_string()};
        let public_version: String = if record[6].is_empty() {"".to_string()} else {record[6].to_string()};
        let pubKeyHash: String = if record[7].is_empty() {"".to_string()} else {record[7].to_string()};
        let scriptHash: String = if record[8].is_empty() {"".to_string()} else {record[8].to_string()};
        let wif: String = if record[9].is_empty() {"".to_string()} else {record[9].to_string()};
        let comment: String = if record[10].is_empty() {"".to_string()} else {record[10].to_string()};
        
        
        let coin_type = CoinDatabase { index, path, symbol, name, key_derivation, private_version, public_version, pubKeyHash, scriptHash, wif, comment };

        coin_store.push(coin_type);
    }

    coin_store
}


fn create_coin_completion_model() -> gtk::ListStore {
    let valid_coin_symbols = create_coin_database(COINLIST_FILE);

    let store = gtk::ListStore::new(&[
        glib::Type::U32, // Index
        glib::Type::U32, // Path
        glib::Type::STRING, // Symbol
        glib::Type::STRING, // Name
        glib::Type::STRING, // key_derivation
        glib::Type::STRING, // private_version
        glib::Type::STRING, // public_version
        glib::Type::STRING, // pubKeyHash
        glib::Type::STRING, // scriptHash
        glib::Type::STRING, // Wif
        glib::Type::STRING, // Comment
    ]);

    for coin_symbol in valid_coin_symbols.iter() {
        let iter = store.append();
        store.set(&iter, &[
            (0, &coin_symbol.index), 
            (1, &coin_symbol.path), 
            (2, &coin_symbol.symbol), 
            (3, &coin_symbol.name),
            (4, &coin_symbol.key_derivation),
            (5, &coin_symbol.private_version),
            (6, &coin_symbol.public_version),
            (7, &coin_symbol.pubKeyHash),
            (8, &coin_symbol.scriptHash),
            (9, &coin_symbol.wif),
            (10, &coin_symbol.comment),
        ]);
    }

    store
}

fn create_gui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title(&format!("{} {}", APP_DESCRIPTION.unwrap(), APP_VERSION.unwrap()))
        .default_width(800)
        .default_height(600)
        .show_menubar(true)
        .icon_name("help-about-symbolic")
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
    let generate_seed_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let generate_seed_button = gtk::Button::new();
    generate_seed_button.set_label("Generate seed");
    generate_seed_box.set_halign(gtk::Align::Center);

    // Connections
    entropy_source_frame.set_child(Some(&entropy_source_dropdown));
    entropy_length_frame.set_child(Some(&entropy_length_dropdown));

    generate_seed_box.append(&generate_seed_button);
    entropy_source_box.append(&entropy_source_frame);
    entropy_length_box.append(&entropy_length_frame);
    entropy_header_first_box.append(&entropy_source_box);
    entropy_header_first_box.append(&entropy_length_box);
    entropy_header_second_box.append(&mnemonic_passphrase_box);
    entropy_header_box.append(&entropy_header_first_box);
    entropy_header_box.append(&entropy_header_second_box);
    entropy_header_box.append(&generate_seed_box);

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
    
    generate_seed_button.connect_clicked(clone!(
        @strong entropy_source_dropdown,
        @strong entropy_length_dropdown,
        @strong mnemonic_words_text,
        @strong seed_text => move |_| {
            let selected_entropy_source_index = entropy_source_dropdown.selected() as usize;
            let selected_entropy_length_index = entropy_length_dropdown.selected() as usize;
            let selected_entropy_source_value = VALID_ENTROPY_SOURCES.get(selected_entropy_source_index);
            let selected_entropy_length_value = VALID_ENTROPY_LENGTHS.get(selected_entropy_length_index);
            let source = selected_entropy_source_value.unwrap().to_string();
            let length = selected_entropy_length_value.unwrap();
            
            println!("Entropy source: {:?}", source);
            println!("Entropy length: {:?}", length);

            let entropy_length = selected_entropy_length_value;

            let pre_entropy = generate_entropy(
                &source,
                *length as u64,
                Some(&ENTROPY_FILE)
            );
            
            let checksum = generate_checksum(&pre_entropy, &entropy_length.unwrap());
            println!("Entropy: {:?}", &pre_entropy);
            println!("Checksum: {:?}", &checksum);
            let full_entropy = format!("{}{}", &pre_entropy, &checksum);
            entropy_text.buffer().set_text(&full_entropy);
            
            let mnemonic_words = generate_mnemonic_words(&full_entropy);
            mnemonic_words_text.buffer().set_text(&mnemonic_words);
            println!("Mnemonic words: {:?}", mnemonic_words);

            let passphrase_text = mnemonic_passphrase_text.text().to_string();
            println!("Mnemonic passphrase: {:?}", &passphrase_text);
            
            let seed = generate_bip39_seed(&pre_entropy, &passphrase_text);
            let seed_hex = hex::encode(&seed[..]);
            seed_text.buffer().set_text(&seed_hex.to_string());
            println!("Seed: {:?}", &seed_hex.to_string());
            
        }
    ));

    // Start Seed sidebar
    stack.add_titled(&entropy_main_box, Some("sidebar-seed"), "Seed");
 
    // SIDEBAR 2
    // Sidebar Coin
    let coin_main_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let coin_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let coin_frame = gtk::Frame::new(Some("Coin"));
    coin_main_box.set_margin_top(10);
    coin_main_box.set_margin_start(10);
    coin_main_box.set_margin_end(10);
    coin_main_box.set_margin_bottom(10);

    // Create scrolled window
    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_max_content_height(400); // Set maximum height

    // Coin treeview
    create_coin_completion_model();
    let coins = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let coin_treeview = gtk::TreeView::new();
    coin_treeview.set_vexpand(true);
    coin_treeview.set_headers_visible(true);

    let columns = ["Index", "Path", "Symbol", "Name", "Key derivation", "Private version", "Public version", "pubKeyHash", "scriptHash", "Wif", "Comment"];
    for (i, column_title) in columns.iter().enumerate() {
        let column = gtk::TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();
        column.set_title(column_title);
        column.pack_start(&cell, true);
        column.add_attribute(&cell, "text", i as i32);
        coin_treeview.append_column(&column);
    }

    // Coin search
    let coin_search = gtk::SearchEntry::new();
    coin_search.set_placeholder_text(Some("Find a coin by entering its symbol (BTC, LTC, ETH...)"));

    coins.append(&coin_search);
    scrolled_window.set_child(Some(&coin_treeview));
    coins.append(&scrolled_window);
    coin_frame.set_child(Some(&coins));
    coin_box.append(&coin_frame);

    // Derivation path
    let main_derivation_box = gtk::Box::new(gtk::Orientation::Vertical, 20);

    // BIP
    let bip_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let bip_frame = gtk::Frame::new(Some("BIP"));
    let valid_bip_as_string: Vec<String> = VALID_BIP_DERIVATIONS.iter().map(|&x| x.to_string()).collect();
    let valid_bip_as_string_refs: Vec<&str> = valid_bip_as_string.iter().map(|s| s.as_ref()).collect();
    let bip_dropdown = gtk::DropDown::from_strings(&valid_bip_as_string_refs);
    bip_frame.set_child(Some(&bip_dropdown));
    bip_frame.set_hexpand(true);
    bip_dropdown.set_selected(1);
    
    // Hardened path
    let hardened_frame = gtk::Frame::new(Some("Hardened path"));
    let hardened_checkbox = gtk4::CheckButton::new();
    hardened_checkbox.set_active(true);
    hardened_checkbox.set_margin_start(10);
    hardened_frame.set_child(Some(&hardened_checkbox));
    hardened_frame.set_hexpand(true);

    // Derivation label
    let derivation_label_frame = gtk::Frame::new(Some("Derivation path"));
    let derivation_label_text = gtk4::Label::builder()
        .label("m/44'/")
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .css_classes(["large-title"])
        .build();

    derivation_label_frame.set_child(Some(&derivation_label_text));

    // Generate extended keys button
    let generate_extended_keys_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let generate_extended_keys_button = gtk::Button::new();
    generate_extended_keys_button.set_label("Generate extended keys");
    generate_extended_keys_button_box.append(&generate_extended_keys_button);
    generate_extended_keys_button_box.set_halign(gtk::Align::Center);
    // generate_extended_keys_button.set_sensitive(false);

    // Master private key
    let extended_keys_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let master_xprv_frame = gtk::Frame::new(Some("Extended master private key"));
    let master_xpub_frame = gtk::Frame::new(Some("Extended master public key"));
    
    let extened_master_private_key_text = gtk::TextView::new();
    let extened_master_public_key_text = gtk::TextView::new();

    extened_master_private_key_text.set_editable(false);
    extened_master_public_key_text.set_editable(false);
    
    master_xprv_frame.set_child(Some(&extened_master_private_key_text));
    master_xpub_frame.set_child(Some(&extened_master_public_key_text));

    extened_master_private_key_text.set_wrap_mode(gtk::WrapMode::Char);
    extened_master_private_key_text.set_editable(false);
    extened_master_private_key_text.set_left_margin(5);
    extened_master_private_key_text.set_top_margin(5);

    extened_master_public_key_text.set_wrap_mode(gtk::WrapMode::Char);
    extened_master_public_key_text.set_editable(false);
    extened_master_public_key_text.set_left_margin(5);
    extened_master_public_key_text.set_top_margin(5);

    // Connections 
    main_derivation_box.append(&bip_box);
    main_derivation_box.append(&derivation_label_frame);
    bip_box.append(&bip_frame);
    bip_box.append(&hardened_frame);
    extended_keys_box.append(&master_xprv_frame);
    extended_keys_box.append(&master_xpub_frame);
    coin_main_box.append(&coin_box);
    coin_main_box.append(&main_derivation_box);
    coin_main_box.append(&generate_extended_keys_button_box);
    coin_main_box.append(&extended_keys_box);
    
    // Actions
    let coin_store = create_coin_store();
    let treestore = gtk4::TreeStore::new(&[glib::Type::STRING; 11]);

    let bip_dropdown_clone = bip_dropdown.clone();
    let derivation_label_text_clone = derivation_label_text.clone();
    let hardened_checkbox_clone = hardened_checkbox.clone();
    let coin_treeview_clone = coin_treeview.clone();
    
    fn update_derivation_label_text(
        bip_dropdown_clone: &gtk::DropDown,
        hardened_checkbox_clone: &gtk::CheckButton,
        derivation_label_text_clone: &gtk::Label,
        tree_coin: &gtk::TreeView,
    ) {
        let selected_bip_index = bip_dropdown_clone.selected() as usize;
        let selected_bip_value = VALID_BIP_DERIVATIONS.get(selected_bip_index);
        let bip = selected_bip_value.unwrap().to_string();
        let hard_path = hardened_checkbox_clone.is_active();
        let mut coin = "".to_string();
        let mut path = "".to_string();

        if let Some((model,  iter)) = tree_coin.selection().selected() {
            coin = model.get_value(&iter, 0).get::<String>().expect("problem with treeview");
        };
        
        if hard_path == true {
            path = "'".to_string();
        }
        
        let derivation = format!("m/{}{}/{}{}", bip, path, coin, path);
        
        println!("derivation: {}", derivation);
        derivation_label_text_clone.set_text(&derivation);
    }

    let bip_dropdown_clone2 = bip_dropdown_clone.clone();
    let derivation_label_text_clone2 = derivation_label_text_clone.clone();
    let hardened_checkbox_clone2 = hardened_checkbox_clone.clone();
    let coin_treeview_clone2 = coin_treeview_clone.clone();
    
    bip_dropdown.connect_selected_notify(move |_| {    
        update_derivation_label_text(
            &bip_dropdown_clone2,
            &hardened_checkbox_clone2,
            &derivation_label_text_clone2,
            &coin_treeview_clone2
        );
    });
    
    let bip_dropdown_clone3 = bip_dropdown_clone.clone();
    let derivation_label_text_clone3 = derivation_label_text_clone.clone();
    let hardened_checkbox_clone3 = hardened_checkbox_clone.clone();
    let coin_treeview_clone3 = coin_treeview_clone.clone();
    
    hardened_checkbox.connect_active_notify(move |_| {
        update_derivation_label_text(
            &bip_dropdown_clone3,
            &hardened_checkbox_clone3,
            &derivation_label_text_clone3,
            &coin_treeview_clone3
        );
    });

    let bip_dropdown_clone4 = bip_dropdown_clone.clone();
    let derivation_label_text_clone4 = derivation_label_text_clone.clone();
    let hardened_checkbox_clone4 = hardened_checkbox_clone.clone();
    let coin_treeview_clone4 = coin_treeview_clone.clone();

    coin_treeview.connect_cursor_changed(move |tree_view| {
        if let Some((model, iter)) = tree_view.selection().selected() {
            let coin = model.get_value(&iter, 0);
            let header = model.get_value(&iter, 1);
            let symbol = model.get_value(&iter, 2);
            let name = model.get_value(&iter, 3);
            let key_derivation = model.get_value(&iter, 4);
            let private_version = model.get_value(&iter, 5);
            let public_version = model.get_value(&iter, 6);
            let pubKeyHash = model.get_value(&iter, 7);
            let scriptHash = model.get_value(&iter, 8);
            let wif = model.get_value(&iter, 9);
            let comment = model.get_value(&iter, 10);

            if let (
                Ok(coin_type),
                Ok(coin_header),
                Ok(coin_symbol),
                Ok(coin_name),
                Ok(key_derivation),
                Ok(private_version),
                Ok(public_version),
                Ok(pubKeyHash),
                Ok(scriptHash),
                Ok(wif),
                Ok(comment),
            ) = (
                coin.get::<String>(), 
                header.get::<String>(), 
                symbol.get::<String>(), 
                name.get::<String>(),
                key_derivation.get::<String>(),
                private_version.get::<String>(),
                public_version.get::<String>(),
                pubKeyHash.get::<String>(),
                scriptHash.get::<String>(),
                wif.get::<String>(),
                comment.get::<String>(),
            ) 
                {
                    println!("coin_type: {}", coin_type);
                    println!("coin_header: {}", coin_header);
                    println!("coin_symbol: {}", coin_symbol);
                    println!("coin_name: {}", coin_name);
                    println!("key_derivation: {}", key_derivation);
                    println!("private_version: {}", private_version);
                    println!("public_version: {}", public_version);
                    println!("pubKeyHash: {}", pubKeyHash);
                    println!("scriptHash: {}", scriptHash);
                    println!("wif: {}", wif);
                    println!("comment: {}", comment);
                    
                    println!("Starting deriving keys:");
                }

                update_derivation_label_text(
                &bip_dropdown_clone4,
                &hardened_checkbox_clone4,
                &derivation_label_text_clone4,
                &coin_treeview_clone4
            );
        }
    });


    coin_search.connect_search_changed(move|coin_search| {
        let search_text = coin_search.text().to_uppercase();
    
        if search_text.len() >= 2 {
            let matching_coins = get_coins_starting_with(&coin_store, &search_text);
            
            if !matching_coins.is_empty() {
                treestore.clear();
                
                for found_coin in matching_coins {
                    let iter = treestore.append(None);
                    treestore.set(&iter, &[
                        (0, &found_coin.index.to_string()),
                        (1, &format!("0x{:X}", found_coin.path)),
                        (2, &found_coin.symbol),
                        (3, &found_coin.name),
                        (4, &found_coin.key_derivation),
                        (5, &found_coin.private_version),
                        (6, &found_coin.public_version),
                        (7, &found_coin.pubKeyHash),
                        (8, &found_coin.scriptHash),
                        (9, &found_coin.wif),
                        (10, &found_coin.comment),
                    ]);
                }
                coin_treeview.set_model(Some(&treestore));
            } else {
                treestore.clear();
            }
        } else {
            treestore.clear();
        }
    });

    derivation_label_text.connect_label_notify(move |label| {
        let re = regex::Regex::new(r"m/(\d{1,2}|[1-9]\d')/(\d{1,8}|[1-9]\d')").unwrap();
        if re.is_match(&label.text()) {
            let coin = "BTC";
            match derive_xprv(&seed_text.text().to_string(), coin, false) {
                Ok(xprv) => extened_master_private_key_text.buffer().set_text(&xprv),
                Err(err) => println!("Error: {}", err),
            }
        } else {
            println!("Label text does not match the expected format.");
        }
    });


    // Start: Coins
    stack.add_titled(&coin_main_box, Some("sidebar-coin"), "Coin");
    let main_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    main_content_box.append(&stack_sidebar);
    main_content_box.append(&stack);
    window.set_child(Some(&main_content_box));

    window.present();
}

fn get_coins_starting_with<'a>(coin_store: &'a Vec<CoinDatabase>, target_prefix: &'a str) -> Vec<&'a CoinDatabase> {
    coin_store
        .iter()
        .filter(|&coin_type| coin_type.symbol.starts_with(target_prefix))
        .collect()
}

fn create_coin_database(file_path: &str) -> Vec<CoinDatabase> {
    let file = File::open(&file_path).expect("can not read file");
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

    let coin_types: Vec<CoinDatabase> = rdr
        .records()
        .filter_map(|record| record.ok())
        .enumerate()
        .map(|(index, record)| {
            
            let path: u32 = index as u32;
            let index: u32 = index.try_into().expect("Conversion from usize to u32 failed");
            let symbol: String = record.get(2).unwrap_or_default().to_string();
            let name: String = record.get(3).unwrap_or_default().to_string();
            let key_derivation: String = record.get(4).unwrap_or_default().to_string();
            let private_version: String = record.get(5).unwrap_or_default().to_string();
            let public_version: String = record.get(6).unwrap_or_default().to_string();
            let pubKeyHash: String = record.get(7).unwrap_or_default().to_string();
            let scriptHash: String = record.get(8).unwrap_or_default().to_string();
            let wif: String = record.get(9).unwrap_or_default().to_string();
            let comment: String = record.get(10).unwrap_or_default().to_string();

            CoinDatabase { index, path, symbol, name, key_derivation, private_version, public_version, pubKeyHash, scriptHash, wif, comment }
            }
        )
        .collect();

    coin_types
}
        

fn main() {
    print_program_info();
    
    let application = gtk::Application::builder()
        .application_id("com.github.qr2m")
        .build();
    application.connect_activate(create_gui);

    let quit = gio::SimpleAction::new("quit", None);
    let new = gio::SimpleAction::new("new", None);
    
    
    quit.connect_activate(
        glib::clone!(@weak application => move |_action, _parameter| {
            application.quit();
        }),
    );

    // new.connect_activate(
    //     glib::clone!(@weak application => move |_action, _parameter| {
    //         let clone = application.clone();
    //     }),
    // );


    application.set_accels_for_action("app.quit", &["<Primary>Q"]);
    application.add_action(&quit);

    application.set_accels_for_action("app.new", &["<Primary>N"]);
    application.add_action(&new);

    application.run();
}

fn hmac_sha512(key: &[u8], data: &[u8]) -> Vec<u8> {
    const BLOCK_SIZE: usize = 128;
    const HASH_SIZE: usize = 64;

    let mut padded_key = [0x00; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        let mut hasher = Sha512::new();
        hasher.update(key);
        padded_key[..HASH_SIZE].copy_from_slice(&hasher.finalize());
    } else {
        padded_key[..key.len()].copy_from_slice(key);
    }

    let mut ipad = padded_key.clone();
    let mut opad = padded_key.clone();

    // XOR key with ipad and opad
    ipad.iter_mut().for_each(|byte| *byte ^= 0x36);
    opad.iter_mut().for_each(|byte| *byte ^= 0x5c);

    // Append data to ipad
    let mut ipad_data = vec![0x00; BLOCK_SIZE + data.len()];
    ipad_data[..BLOCK_SIZE].copy_from_slice(&ipad);
    ipad_data[BLOCK_SIZE..].copy_from_slice(&data);

    // Calculate inner hash
    let inner_hash = Sha512::digest(&ipad_data);

    // Append inner hash to opad
    let mut opad_inner = vec![0x00; BLOCK_SIZE + HASH_SIZE];
    opad_inner[..BLOCK_SIZE].copy_from_slice(&opad);
    opad_inner[BLOCK_SIZE..].copy_from_slice(&inner_hash);
    println!("opad_inner length: {}", opad_inner.len());
    println!("inner_hash length: {}", inner_hash.len());
    // Calculate outer hash
    Sha512::digest(&opad_inner).to_vec()
}

fn derive_xprv(seed: &str, coin: &str, public: bool) -> Result<String, String> {
    // Define version bytes for different cryptocurrencies
    let header: (u32, u32) = match coin {
        // Coin      Private     Public
        "BTC" =>    (0x0488ade4, 0x0488b21e),
        "SHADOW" => (0xEE8031E8, 0xEE80286A),
        "DASH" =>   (0x0488ade4, 0x0488b21e),
        "LTC" =>    (0x0488ade4, 0x0488b21e),
        _ => return Err(String::from("Unsupported coin")),
    };

    let seed_bytes = hex::decode(seed).map_err(|_| String::from("Invalid seed hex"))?;
    let hmac_result = hmac_sha512(b"Bitcoin seed", &seed_bytes);
    let (master_private_key_bytes, chain_code_bytes) = hmac_result.split_at(32);

    // Construct the extended key
    let mut extended_key = Vec::new();

    // Choose the header based on whether it's public or private key derivation
    let header_bytes = if public { header.1 } else { header.0 };
    extended_key.extend_from_slice(&u32::to_be_bytes(header_bytes)); // Version        4 bytes
    extended_key.push(0x00);                                          // Depth          1 byte
    extended_key.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);        // Parent finger  4 bytes
    extended_key.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);        // Index/child    4 bytes
    extended_key.extend_from_slice(chain_code_bytes);                 // Chain code     32 bytes
    extended_key.push(0x00);                                          // Key prefix     1 byte

    if public {
        // Derive and serialize the public key
        let secp = secp256k1::Secp256k1::new();
        let sk = secp256k1::SecretKey::from_slice(&master_private_key_bytes)
            .map_err(|e| format!("Error converting private key: {:?}", e))?;
        let pk = secp256k1::PublicKey::from_secret_key(&secp, &sk);
        let pk_bytes = pk.serialize_uncompressed();
        extended_key.extend_from_slice(&pk_bytes);
    } else {
        // Append the master private key bytes
        extended_key.extend_from_slice(master_private_key_bytes);
    }

    // Calculate checksum and append
    let checksum: [u8; 4] = calculate_checksum(&extended_key);  // Checksum       4 bytes
    extended_key.extend_from_slice(&checksum);

    // Encode the extended key in Base58Check format
    let xprv_base58check = bs58::encode(&extended_key).into_string();

    Ok(xprv_base58check)
}



fn calculate_checksum(data: &[u8]) -> [u8; 4] {
    let hash1 = Sha256::digest(data);
    let hash2 = Sha256::digest(&hash1);
    let checksum = &hash2[..4];
    let mut result = [0u8; 4];
    result.copy_from_slice(checksum);
    result
}