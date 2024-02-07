// #![allow(dead_code)]
// #![allow(unused_variables)]
// #![allow(unused_imports)]
#![allow(non_snake_case)]


// Crates
use std::{
    fs::{
        self, 
        File
    }, 
    // hash::Hasher,
    io::{
        self, 
        Read, 
        Seek
    }, vec
};
use hex;
use rand::Rng;
use sha2::{Digest, Sha256};
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
mod converter;


// Global variables
const ENTROPY_FILE: &str = "entropy/test.qrn";
// const ENTROPY_FILE: &str = "./entropy/binary.qrn";
const WORDLIST_FILE: &str = "./lib/bip39-english.txt";
const COINLIST_FILE: &str = "./lib/bip44-coin_type.csv";
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
    coin: String,
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
    let entropy_binary = converter::convert_string_to_binary(&entropy);
    let hash_raw_binary: String = converter::convert_binary_to_string(&Sha256::digest(&entropy_binary));
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
    let entropy_vector = converter::convert_string_to_binary(&entropy);
    let mnemonic_result = bip39::Mnemonic::from_entropy(&entropy_vector).expect("Can not create mnemomic words");
    let mnemonic = mnemonic_result;
    let seed = bip39::Mnemonic::to_seed(&mnemonic, passphrase);

    seed
}

fn create_coin_store() -> Vec<CoinDatabase> {
    let file = File::open(&COINLIST_FILE).expect("can not open bip44 coin file");
    let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);
    let mut coin_store = Vec::new();

    // Iterate over the CSV records and populate CoinDatabase
    for result in rdr.records() {
        let record = result.expect("error reading CSV record");
        let index = record[0].parse().expect("error parsing index");
        let path = u32::from_str_radix(&record[1][2..], 16).expect("error parsing path");
        let symbol = record[2].to_string();
        let coin = record[3].to_string();
        let coin_type = CoinDatabase { index, path, symbol, coin };

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
        glib::Type::STRING, // Coin
    ]);

    for coin_symbol in valid_coin_symbols.iter() {
        let iter = store.append();
        store.set(&iter, &[(0, &coin_symbol.index), (1, &coin_symbol.path), (2, &coin_symbol.symbol), (3, &coin_symbol.coin)]);
    }

    store
}

fn create_gui(application: &gtk::Application) {
    let title = format!("{} {}", APP_DESCRIPTION.unwrap(), APP_VERSION.unwrap());

    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title(&title)
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
    let generate_wallet_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let generate_seed_button = gtk::Button::new();
    generate_seed_button.set_label("Generate");
    generate_wallet_box.set_halign(gtk::Align::Center);

    // Connections
    entropy_source_frame.set_child(Some(&entropy_source_dropdown));
    entropy_length_frame.set_child(Some(&entropy_length_dropdown));

    generate_wallet_box.append(&generate_seed_button);
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

    // Coin treeview
    create_coin_completion_model();
    let coins = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let coin_treeview = gtk::TreeView::new();
    coin_treeview.set_vexpand(true);
    coin_treeview.set_headers_visible(true);

    let columns = ["Index", "Path", "Symbol", "Coin"];
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
    coins.append(&coin_treeview);
    coin_frame.set_child(Some(&coins));
    
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
        .label("m/44'")
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .css_classes(["large-title"])
        .build();

    derivation_label_frame.set_child(Some(&derivation_label_text));

    // Generate button
    let generate_master_pk_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let generate_master_pk_button = gtk::Button::new();
    generate_master_pk_button.set_label("Generate");
    generate_master_pk_button_box.append(&generate_master_pk_button);
    generate_master_pk_button_box.set_halign(gtk::Align::Center);

    // Master private key
    let master_private_key_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let master_private_key_frame = gtk::Frame::new(Some("Master private key"));
    let master_private_key_text = gtk::TextView::new();
    master_private_key_text.set_editable(false);
    master_private_key_frame.set_child(Some(&master_private_key_text));

    // Connections 
    coin_box.append(&coin_frame);
    main_derivation_box.append(&bip_box);
    main_derivation_box.append(&derivation_label_frame);
    bip_box.append(&bip_frame);
    bip_box.append(&hardened_frame);
    master_private_key_box.append(&master_private_key_frame);
    coin_main_box.append(&coin_box);
    coin_main_box.append(&main_derivation_box);
    coin_main_box.append(&generate_master_pk_button_box);
    coin_main_box.append(&master_private_key_box);
    
    // Actions
    let coin_store = create_coin_store();
    let treestore = gtk4::TreeStore::new(&[glib::Type::STRING; 4]);
    let bip_dropdown_clone = bip_dropdown.clone();
    let hardened_checkbox_clone = hardened_checkbox.clone();
    let derivation_label_text_clone = derivation_label_text.clone();

    bip_dropdown.connect_selected_notify(move |_| {
        let selected_index = bip_dropdown_clone.selected();
        // if let selected_index = selected_index {
            if let Some(value_from_array) = VALID_BIP_DERIVATIONS.get(selected_index as usize) {
                let hard = if hardened_checkbox_clone.is_active() { "'" } else { "" };
                let text = format!("m/{}{}", value_from_array, hard);
                derivation_label_text.set_text(&text);
            // }
        }
    });
    
    let hardened_checkbox_clone2 = hardened_checkbox.clone();
    let derivation_label_text_clone2 = derivation_label_text_clone.clone();

    hardened_checkbox.connect_active_notify(move |_| {
        let bip_dropdown_clone = bip_dropdown.clone(); // Clone again here

        let selected_index = bip_dropdown_clone.selected();
        // if let selected_index = selected_index {
            if let Some(value_from_array) = VALID_BIP_DERIVATIONS.get(selected_index as usize) {
                let hard = if hardened_checkbox_clone2.is_active() { "'" } else { "" };
                let text = format!("m/{}{}", value_from_array, hard);
            derivation_label_text_clone2.set_text(&text);
            }
        // }
    });

    coin_treeview.connect_cursor_changed(move |tree_view| {
        if let Some((model, iter)) = tree_view.selection().selected() {
            // Get the values from the selected row
            
            let coin = model.get_value(&iter, 0);
            let header = model.get_value(&iter, 1);
            let symbol = model.get_value(&iter, 2);
            let name = model.get_value(&iter, 3);

            if let (
                Ok(coin_type),
                Ok(coin_header),
                Ok(coin_symbol),
                Ok(coin_name)
            ) = (
                coin.get::<String>(), 
                header.get::<String>(), 
                symbol.get::<String>(), 
                name.get::<String>()) {
                    println!("coin_type: {}", coin_type);
                    println!("coin_header: {}", coin_header);
                    println!("coin_symbol: {}", coin_symbol);
                    println!("coin_name: {}", coin_name);
                    println!("Starting deriving keys:");
                }
        }
    });

    coin_search.connect_search_changed(move|coin_search| {
        let search_text = coin_search.text().to_uppercase();
        master_private_key_text.buffer().set_text("");
    
        if search_text.len() >= 2 {
            let matching_coins = get_coins_starting_with(&coin_store, &search_text);
    
            if !matching_coins.is_empty() {
                // let text = format!("{} found", &search_text);
    
                // let master_priv = create_master_private_key(cloned_seed_text.text().to_string());
                // master_private_key_text.buffer().set_text(&master_priv.to_string());
    
                treestore.clear();
    
                for found_coin in matching_coins {
                    let iter = treestore.append(None);
                    treestore.set(&iter, &[
                        (0, &found_coin.index.to_string()),
                        (1, &format!("0x{:X}", found_coin.path)), // Format as hexadecimal
                        (2, &found_coin.symbol),
                        (3, &found_coin.coin),
                    ]);
                }
    
                coin_treeview.set_model(Some(&treestore));
            } else {
                // let msg = format!("No coins found starting with {}.", search_text);
                treestore.clear();
            }
        } else {
            // master_private_key_text.buffer().set_text("");
            treestore.clear();
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
            let path = index as u32;
            let index = index.try_into().expect("Conversion from usize to u32 failed");
            let symbol = record.get(2).unwrap_or_default().to_string();
            let coin = record.get(3).unwrap_or_default().to_string();
            CoinDatabase { index, path, symbol, coin }
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
    quit.connect_activate(
        glib::clone!(@weak application => move |_action, _parameter| {
            application.quit();
        }),
    );
    
    application.set_accels_for_action("app.quit", &["<Primary>Q"]);
    application.add_action(&quit);

    application.run();
}


























// TESTING BEYOND
// ------------------------------------------------------------------------------------
// Generating master private key


fn create_derivation_path(
    purpose: u32,
    coin_type: u32,
    account: u32,
    change: u32, 
    address_index: u32, 
    hardened: bool
) -> Vec<u8> {

    let mut path_bytes = vec![];
    let purpose = purpose | (1 << 31);
    let coin_type = coin_type | (if hardened { 1 << 31 } else { 0 });
    let account = account | (if hardened { 1 << 31 } else { 0 });
    let change = change;
    let address_index = address_index | (if hardened { 1 << 31 } else { 0 });
    
    path_bytes.extend_from_slice(&purpose.to_be_bytes());
    path_bytes.extend_from_slice(&coin_type.to_be_bytes());
    path_bytes.extend_from_slice(&account.to_be_bytes());
    path_bytes.extend_from_slice(&change.to_be_bytes());
    path_bytes.extend_from_slice(&address_index.to_be_bytes());
    
    println!("Derivation path: {:?}", &path_bytes);
    path_bytes

}







// Working bellow, but wrong output

// fn generate_xprv_from_seed(
//     coin_type: u32,
//     derivation_path: Vec<u32>,
//     seed: [u8; 64],
// ) -> String {
//     // Parse coin_header from hexadecimal string
//     // let parsed_coin_header = u32::from_str_radix(&coin_header[2..], 16).expect("Invalid coin_header");
    
//     // Concatenate the coin header and coin type into a 32-byte array
//     // let coin_header_bytes: [u8; 4] = parsed_coin_header.to_be_bytes();
//     // let mut extended_key_header = [0u8; 32];
//     extended_key_header[..4].copy_from_slice(&coin_header_bytes);
//     extended_key_header[4..8].copy_from_slice(&coin_type.to_be_bytes());
    
//     // Compute HMAC-SHA512 hash
//     let mut mac = Hmac::<Sha512>::new_from_slice(&seed).expect("HMAC can take key of any size");
//     mac.update(&extended_key_header);
//     let hmac_result = mac.finalize().into_bytes();

//     let seed_left = &hmac_result[..32]; // Take the first 32 bytes
//     let seed_right = &hmac_result[32..]; // Take the last 32 bytes
    
//     // Construct secret key from left part
//     let secret_key = SecretKey::from_slice(seed_left)
//         .expect("Invalid secret key bytes");

//     // Construct public key from secret key
//     let secp = Secp256k1::new();
//     let public_key = PublicKey::from_secret_key(&secp, &secret_key);

//     // Construct the extended private key
//     let mut extended_private_key = Vec::new();
//     extended_private_key.extend_from_slice(seed_left);
//     extended_private_key.extend_from_slice(&public_key.serialize());

//     // Derive extended private key for each derivation path element
//     for index in derivation_path {
//         let index_bytes = if index >= 0x80000000 {
//             let mut index_bytes = index.to_be_bytes();
//             index_bytes[0] |= 0x80; // Set the first bit to indicate a hardened path
//             index_bytes
//         } else {
//             index.to_be_bytes()
//         };

//         // HMAC the extended private key with the index
//         let mut mac = Hmac::<Sha512>::new_from_slice(seed_right).expect("HMAC can take key of any size");
//         mac.update(&extended_private_key);
//         mac.update(&index_bytes);
//         let hmac_result = mac.finalize().into_bytes();

//         let hmac_left = &hmac_result[..32]; // Take the first 32 bytes

//         // Update extended private key with left part of HMAC result
//         extended_private_key.clear();
//         extended_private_key.extend_from_slice(hmac_left);
//         extended_private_key.extend_from_slice(&public_key.serialize());
//     }
    
//     // Convert extended private key to base58
//     let xprv = base58_encode(&extended_private_key);
//     println!("XPRV: {:?}", xprv);
//     xprv
// }



// use sha2::Sha512;
// use hmac::{Hmac, Mac};
// use num_bigint::BigUint;
// use num_traits::{Zero, Num, ToPrimitive};

// fn base58_encode(data: &[u8]) -> String {
//     const BASE58_ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
//     const BASE: usize = 58;

//     // Convert the input data to a big integer
//     let num = BigUint::from_bytes_be(data);

//     let mut result = Vec::new();
//     let mut num = num.clone(); // Create a mutable copy of num

//     while !num.is_zero() {
//         let (div, rem) = (num.clone() / BASE as u64, (num % BASE as u64).to_u8().unwrap());
//         result.push(BASE58_ALPHABET[rem as usize]);
//         num = div;
//     }

//     // Add leading '1's for each leading zero byte in the original data
//     for byte in data.iter().take_while(|&&b| b == 0) {
//         result.push(BASE58_ALPHABET[0]);
//     }

//     result.reverse();
//     String::from_utf8(result).unwrap()
// }
