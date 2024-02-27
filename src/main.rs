// #![allow(dead_code)]
// #![allow(unused_variables)]
// #![allow(unused_imports)]
// #![allow(non_snake_case)]


// Dependencies
use std::{
    fs::{
        self, 
        File
    }, 
    io::{
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
use qr2m_converters;


// Global variables
const WORDLIST_FILE: &str = "lib/bip39-mnemonic-words-english.txt";
const COINLIST_FILE: &str = "lib/bip44-extended-coin-list.csv";
const ENTROPY_FILE: &str = "entropy/test.qrn";
const VALID_ENTROPY_LENGTHS: [u32; 5] = [128, 160, 192, 224, 256];
const VALID_BIP_DERIVATIONS: [u32; 2] = [32, 44];
const VALID_ENTROPY_SOURCES: &'static [&'static str] = &[
    "RNG", 
    "File", 
    "ANU API",
];
const VALID_WALLET_PURPOSE: &'static [&'static str] = &[
    "Internal", 
    "External", 
];
const APP_DESCRIPTION: Option<&str> = option_env!("CARGO_PKG_DESCRIPTION");
const APP_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
const APP_AUTHOR: Option<&str> = option_env!("CARGO_PKG_AUTHORS");
const GUI_HEIGHT: i32 = 800;
const GUI_WIDTH: i32 = 1200;


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
    private_header: String,
    public_header: String,
    public_key_hash: String,
    script_hash: String,
    wif: String,
    comment: String,
}


fn generate_entropy(source: &str, length: u64, file_name: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    match source {
        "RNG" => {
            let mut rng = rand::thread_rng();
            let binary_string: String = (0..length)
                .map(|_| rng.gen_range(0..=1))
                .map(|bit| char::from_digit(bit, 10).unwrap())
                .collect();

            Ok(binary_string)
        },
        "File" => {
            let file = File::open(file_name.unwrap())?;
            let mut reader = io::BufReader::new(file);
            
            let file_length = reader.seek(io::SeekFrom::End(0))?;
            
            if file_length < length {
                eprintln!("Entropy file too small for requested entropy length: {}", length);
                return Err("Insufficient entropy in file".into());
            }

            let max_start = file_length.saturating_sub(length);
            let start_point = rand::thread_rng().gen_range(0..=max_start);

            reader.seek(io::SeekFrom::Start(start_point))?;

            let mut entropy_raw_binary = String::new();
            reader.take(length).read_to_string(&mut entropy_raw_binary)?;

            Ok(entropy_raw_binary)
        },
        _ => Err("Invalid entropy source specified".into()),
    }
}

fn generate_checksum(entropy: &str, entropy_length: &u32) -> String {
    let entropy_binary = qr2m_converters::convert_string_to_binary(&entropy);
    let hash_raw_binary: String = qr2m_converters::convert_binary_to_string(&Sha256::digest(&entropy_binary));
    let checksum_lenght = entropy_length / 32;
    let checksum: String = hash_raw_binary.chars().take(checksum_lenght.try_into().unwrap()).collect();

    checksum
}

fn generate_mnemonic_words(final_entropy_binary: &str) -> String {
    let chunks: Vec<String> = final_entropy_binary.chars().collect::<Vec<char>>().chunks(11).map(|chunk| chunk.iter().collect()).collect();
    let mnemonic_decimal: Vec<u32> = chunks.iter().map(|chunk| u32::from_str_radix(chunk, 2).unwrap()).collect();
    
    let mnemonic_file_content = match fs::read_to_string(WORDLIST_FILE) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading wordlist file: {}", err);
            return String::new();
        }
    };
    
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
    let entropy_vector = qr2m_converters::convert_string_to_binary(&entropy);
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
        let symbol: String = if record[2].is_empty()            {"".to_string()} else {record[2].to_string()};
        let name: String = if record[3].is_empty()              {"".to_string()} else {record[3].to_string()};
        let key_derivation:String = if record[4].is_empty()     {"".to_string()} else {record[4].to_string()};
        let private_header: String = if record[5].is_empty()    {"".to_string()} else {record[5].to_string()};
        let public_header: String = if record[6].is_empty()     {"".to_string()} else {record[6].to_string()};
        let public_key_hash: String = if record[7].is_empty()   {"".to_string()} else {record[7].to_string()};
        let script_hash: String = if record[8].is_empty()       {"".to_string()} else {record[8].to_string()};
        let wif: String = if record[9].is_empty()               {"".to_string()} else {record[9].to_string()};
        let comment: String = if record[10].is_empty()          {"".to_string()} else {record[10].to_string()};
        
        let coin_type = CoinDatabase { 
            index, 
            path, 
            symbol, 
            name, 
            key_derivation, 
            private_header, 
            public_header, 
            public_key_hash, 
            script_hash, 
            wif, 
            comment 
        };

        coin_store.push(coin_type);
    }

    coin_store
}

fn create_coin_completion_model() -> gtk::ListStore {
    let valid_coin_symbols = create_coin_database(COINLIST_FILE);

    let store = gtk::ListStore::new(&[
        glib::Type::U32,    // Index
        glib::Type::U32,    // Path
        glib::Type::STRING, // Symbol
        glib::Type::STRING, // Name
        glib::Type::STRING, // key_derivation
        glib::Type::STRING, // private_header
        glib::Type::STRING, // public_header
        glib::Type::STRING, // public_key_hash
        glib::Type::STRING, // script_hash
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
            (5, &coin_symbol.private_header),
            (6, &coin_symbol.public_header),
            (7, &coin_symbol.public_key_hash),
            (8, &coin_symbol.script_hash),
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
        .default_width(GUI_WIDTH)
        .default_height(GUI_HEIGHT)
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
    let seed_text = gtk::TextView::new();
    seed_box.set_hexpand(true);
    seed_text.set_editable(false);
    seed_text.set_vexpand(true);
    seed_text.set_hexpand(true);
    seed_text.set_left_margin(5);
    seed_text.set_top_margin(5);
    seed_text.set_wrap_mode(gtk::WrapMode::Char);

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
            ).unwrap();
            
            let checksum = generate_checksum(&pre_entropy, entropy_length.unwrap());
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

    // 
    //  
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

    let columns = ["Index", "Path", "Symbol", "Name", "Key derivation", "Private header", "Public header", "public_key_hash", "script_hash", "Wif", "Comment"];
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

    
    // Master private key
    let master_keys_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    let master_xprv_frame = gtk::Frame::new(Some("Master private key"));
    let master_xpub_frame = gtk::Frame::new(Some("Master public key"));
    
    let master_private_key_text = gtk::TextView::new();
    let master_public_key_text = gtk::TextView::new();

    master_private_key_text.set_editable(false);
    master_public_key_text.set_editable(false);
    
    master_xprv_frame.set_child(Some(&master_private_key_text));
    master_xpub_frame.set_child(Some(&master_public_key_text));

    master_private_key_text.set_wrap_mode(gtk::WrapMode::Char);
    master_private_key_text.set_editable(false);
    master_private_key_text.set_left_margin(5);
    master_private_key_text.set_top_margin(5);

    master_public_key_text.set_wrap_mode(gtk::WrapMode::Char);
    master_public_key_text.set_editable(false);
    master_public_key_text.set_left_margin(5);
    master_public_key_text.set_top_margin(5);

    // Connections 
    master_keys_box.append(&master_xprv_frame);
    master_keys_box.append(&master_xpub_frame);
    coin_main_box.append(&coin_box);
    coin_main_box.append(&master_keys_box);
    
    // Actions
    let coin_store = create_coin_store();
    let treestore = gtk4::TreeStore::new(&[glib::Type::STRING; 11]);
    let coin_treeview_clone = coin_treeview.clone();

    coin_treeview.connect_cursor_changed(move |_| {
        if let Some((model, iter)) = coin_treeview_clone.selection().selected() {
            let coin = model.get_value(&iter, 0);
            let header = model.get_value(&iter, 1);
            let symbol = model.get_value(&iter, 2);
            let name = model.get_value(&iter, 3);
            let key_derivation = model.get_value(&iter, 4);
            let private_header = model.get_value(&iter, 5);
            let public_header = model.get_value(&iter, 6);
            let public_key_hash = model.get_value(&iter, 7);
            let script_hash = model.get_value(&iter, 8);
            let wif = model.get_value(&iter, 9);
            let comment = model.get_value(&iter, 10);

            if let (
                Ok(coin_type),
                Ok(coin_header),
                Ok(coin_symbol),
                Ok(coin_name),
                Ok(key_derivation),
                Ok(private_header),
                Ok(public_header),
                Ok(public_key_hash),
                Ok(script_hash),
                Ok(wif),
                Ok(comment),
            ) = (
                coin.get::<String>(), 
                header.get::<String>(), 
                symbol.get::<String>(), 
                name.get::<String>(),
                key_derivation.get::<String>(),
                private_header.get::<String>(),
                public_header.get::<String>(),
                public_key_hash.get::<String>(),
                script_hash.get::<String>(),
                wif.get::<String>(),
                comment.get::<String>(),
            ) 
                {
                    println!("coin_type: {}", coin_type);
                    println!("coin_header: {}", coin_header);
                    println!("coin_symbol: {}", coin_symbol);
                    println!("coin_name: {}", coin_name);
                    println!("key_derivation: {}", key_derivation);
                    println!("private_header: {}", private_header);
                    println!("public_header: {}", public_header);
                    println!("public_key_hash: {}", public_key_hash);
                    println!("script_hash: {}", script_hash);
                    println!("wif: {}", wif);
                    println!("comment: {}", comment);
                    let buffer = seed_text.buffer();
                    let start_iter = buffer.start_iter();
                    let end_iter = buffer.end_iter();
                    let seed_string = buffer.text(&start_iter, &end_iter, true);
                    println!("Seed: {}", seed_string);
                    
                    match derive_master_keys(
                        &seed_string, 
                        // &coin_symbol,
                        &private_header,
                        &public_header,
                        // false,
                    ) {
                        Ok(xprv) => {
                            master_private_key_text.buffer().set_text(&xprv.0);
                            master_public_key_text.buffer().set_text(&xprv.1);
                        },
                        Err(err) => println!("Can not derive master keys: {}", err),
                    }
                }
                
        }
    });
    
    coin_search.connect_search_changed(move|coin_search| {
        let search_text = coin_search.text().to_uppercase();
        treestore.clear();
    
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
                        (5, &found_coin.private_header),
                        (6, &found_coin.public_header),
                        (7, &found_coin.public_key_hash),
                        (8, &found_coin.script_hash),
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

    stack.add_titled(&coin_main_box, Some("sidebar-coin"), "Coin");

    








    // ----------------------------------------------------------------------
    //  S I D E B A R   3   -   A D D R E S S 
    //
    let main_address_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    main_address_box.set_hexpand(true);
    main_address_box.set_vexpand(true);

    let derivation_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    
    let main_bip_frame = gtk::Frame::new(Some("BIP"));
    main_bip_frame.set_hexpand(true);
    // main_bip_frame.set_vexpand(true);
    
    let main_coin_frame = gtk::Frame::new(Some("Coin"));
    main_coin_frame.set_hexpand(true);
    // main_coin_frame.set_vexpand(true);

    let main_address_frame = gtk::Frame::new(Some("Address"));
    main_address_frame.set_hexpand(true);
    // main_address_frame.set_vexpand(true);
    
    let main_purpose_frame = gtk::Frame::new(Some("Purpose"));
    main_purpose_frame.set_hexpand(true);
    // main_purpose_frame.set_vexpand(true);

    let bip_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let coin_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let address_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let purpose_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    purpose_box.set_hexpand(true);
    // purpose_box.set_vexpand(true);

    let bip_frame = gtk::Frame::new(Some("BIP"));
    bip_frame.set_hexpand(true);
    // bip_frame.set_vexpand(true);

    let bip_hardened_frame = gtk::Frame::new(Some("Hardened?"));
    bip_hardened_frame.set_hexpand(true);
    // bip_hardened_frame.set_vexpand(true);
    
    let coin_frame = gtk::Frame::new(Some("Coin"));
    coin_frame.set_hexpand(true);
    // coin_frame.set_vexpand(true);
    
    let coin_hardened_frame = gtk::Frame::new(Some("Hardened?"));
    coin_hardened_frame.set_hexpand(true);
    // coin_hardened_frame.set_vexpand(true);
    
    let address_frame = gtk::Frame::new(Some("Address"));
    address_frame.set_hexpand(true);
    // address_frame.set_vexpand(true);

    let address_hardened_frame = gtk::Frame::new(Some("Hardened?"));
    address_hardened_frame.set_hexpand(true);
    // address_hardened_frame.set_vexpand(true);

    let purpose_frame = gtk::Frame::new(Some("Purpose"));
    purpose_frame.set_hexpand(true);
    // purpose_frame.set_vexpand(true);


    let valid_bip_as_string: Vec<String> = VALID_BIP_DERIVATIONS.iter().map(|&x| x.to_string()).collect();
    let valid_bip_as_ref: Vec<&str> = valid_bip_as_string.iter().map(|s| s.as_ref()).collect();
    let bip_dropdown = gtk::DropDown::from_strings(&valid_bip_as_ref);
    bip_dropdown.set_selected(1); // BIP44

    let bip_hardened_checkbox = gtk::CheckButton::new();
    bip_hardened_checkbox.set_active(true);
    
    let coin_entry = gtk::Entry::new();
    coin_entry.set_editable(false);
    
    let coin_hardened_checkbox = gtk::CheckButton::new();
    coin_hardened_checkbox.set_active(true);
    
    let adjustment = gtk::Adjustment::new(
        0.0, // initial value
        0.0, // minimum value
        100.0, // maximum value
        1.0, // step increment
        10.0, // page increment
        0.0, // page size
    );
    let address_spinbutton = gtk::SpinButton::new(Some(&adjustment), 1.0, 0);
    
    let address_hardened_checkbox = gtk::CheckButton::new();
    address_hardened_checkbox.set_active(true);
    
    let valid_wallet_pupose_as_strings: Vec<String> = VALID_WALLET_PURPOSE.iter().map(|&x| x.to_string()).collect();
    let valid_wallet_pupose_as_ref: Vec<&str> = valid_wallet_pupose_as_strings.iter().map(|s| s.as_ref()).collect();
    let purpose_dropbox = gtk::DropDown::from_strings(&valid_wallet_pupose_as_ref);
    purpose_dropbox.set_selected(1); // External

    bip_frame.set_child(Some(&bip_dropdown));
    bip_hardened_frame.set_child(Some(&bip_hardened_checkbox));
    coin_frame.set_child(Some(&coin_entry));
    coin_hardened_frame.set_child(Some(&coin_hardened_checkbox));
    address_frame.set_child(Some(&address_spinbutton));
    address_hardened_frame.set_child(Some(&address_hardened_checkbox));
    purpose_frame.set_child(Some(&purpose_dropbox));

    bip_box.append(&bip_frame);
    bip_box.append(&bip_hardened_frame);

    coin_box.append(&coin_frame);
    coin_box.append(&coin_hardened_frame);

    address_box.append(&address_frame);
    address_box.append(&address_hardened_frame);
    
    purpose_box.append(&purpose_frame);


    main_bip_frame.set_child(Some(&bip_box));
    main_coin_frame.set_child(Some(&coin_box));
    main_address_frame.set_child(Some(&address_box));
    main_purpose_frame.set_child(Some(&purpose_box));

    derivation_box.append(&main_bip_frame);
    derivation_box.append(&main_coin_frame);
    derivation_box.append(&main_address_frame);
    derivation_box.append(&main_purpose_frame);


    // ----------------------------------------------------------------------
    //  D e r i v a t i o n   l a b e l
    // 
    let derivation_label_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let derivation_label_frame = gtk::Frame::new(Some("Derivation path"));
    derivation_label_frame.set_hexpand(true);
    // derivation_label_frame.set_vexpand(true);
    
    let derivation_label_text = gtk4::Label::builder()
    .label("m/44'/")
    .halign(gtk::Align::Center)
    .valign(gtk::Align::Center)
    .css_classes(["large-title"])
    .build();

    derivation_label_box.append(&derivation_label_frame);
    derivation_label_frame.set_child(Some(&derivation_label_text));

    // 
    // ----------------------------------------------------------------------
    //  A d d r e s s   t r e e v i e w 
    let address_treeview_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let address_treeview_frame = gtk::Frame::new(Some("Addresses"));
    address_treeview_frame.set_hexpand(true);
    address_treeview_frame.set_vexpand(true);

    let address_treeview = gtk::TreeView::new();
    address_treeview.set_headers_visible(true);
    let columns = ["Path", "Address", "Public key", "Private key"];
    for (i, column_title) in columns.iter().enumerate() {
        let column = gtk::TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();
        column.set_title(column_title);
        column.pack_start(&cell, true);
        column.add_attribute(&cell, "text", i as i32);
        address_treeview.append_column(&column);
    }

    address_treeview_frame.set_child(Some(&address_treeview));
    address_treeview_box.append(&address_treeview_frame);
    // 
    // ----------------------------------------------------------------------
    
    
    main_address_box.append(&derivation_box);
    main_address_box.append(&derivation_label_box);
    main_address_box.append(&address_treeview_box);
    
    
    
    
    
    
    
    // derivation_frame.set_child(Some(&derivation_box));


    // let address_main_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
    // let main_derivation_box = gtk::Box::new(gtk::Orientation::Vertical, 20);

    // // Derivation
    // let derivation_box_input = gtk::Box::new(gtk::Orientation::Vertical, 20);
    // let derivation_box_bip = gtk::Box::new(gtk::Orientation::Vertical, 20);
    




    // // BIP
    // let purpose_frame = gtk::Frame::new(Some("Purpose"));
    // let main_bip_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    // let bip_frame = gtk::Frame::new(Some("BIP"));
    // let hard_bip_frame = gtk::Frame::new(Some("Hardened"));
    // main_bip_box.append(&bip_frame);
    // main_bip_box.append(&hard_bip_frame);
    // let valid_bip_as_string: Vec<String> = _VALID_BIP_DERIVATIONS.iter().map(|&x| x.to_string()).collect();
    // let valid_bip_as_ref: Vec<&str> = valid_bip_as_string.iter().map(|s| s.as_ref()).collect();
    // let bip_dropdown = gtk::DropDown::from_strings(&valid_bip_as_ref);
    // bip_dropdown.set_selected(1); // BIP44    
    // bip_frame.set_child(Some(&bip_dropdown));
    // let hard_bip_checkbox = gtk::CheckButton::new();
    // hard_bip_frame.set_child(Some(&hard_bip_checkbox));
    // purpose_frame.set_child(Some(&main_bip_box));
    
    // derivation_box_bip.append(&purpose_frame);
    





    // let coin_main_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    // let coin_main_frame = gtk::Frame::new(Some("Coin"));

    // let coin_main_index_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    // let main_coin_index_frame = gtk::Frame::new(Some("Coin index"));
    
    // let coin_main_index_hardened_box = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    // let main_coin_index_hardened_frame = gtk::Frame::new(Some("Hardened"));

    

    // coin_box.append(&main_coin_frame);
    // coin_hard_box.append(&main_coin_hardened_frame);
    // // main_coin_frame.set_child(Some(&))
    
    // // main_coin_box.append(&main_coin_frame);








    // let derivation_box_coin = gtk::Box::new(gtk::Orientation::Vertical, 20);
    // let derivation_box_account = gtk::Box::new(gtk::Orientation::Vertical, 20);
    // let derivation_box_internal = gtk::Box::new(gtk::Orientation::Vertical, 20);
    
    // derivation_box_input.append(&derivation_box_bip);
    // derivation_box_input.append(&derivation_box_coin);
    // derivation_box_input.append(&derivation_box_account);
    // derivation_box_input.append(&derivation_box_internal);
    












    // // Derivation label
    // let derivation_box_label = gtk::Box::new(gtk::Orientation::Vertical, 20);
    // let derivation_label_frame = gtk::Frame::new(Some("Derivation path"));
    // let derivation_label_text = gtk4::Label::builder()
    //     .label("m/44'/")
    //     .halign(gtk::Align::Center)
    //     .valign(gtk::Align::Center)
    //     .css_classes(["large-title"])
    //     .build();
    // derivation_label_frame.set_child(Some(&derivation_label_text));
    // derivation_box_label.append(&derivation_label_frame);

    // main_derivation_box.append(&derivation_box_input);
    // main_derivation_box.append(&derivation_box_label);

    // address_main_box.append(&main_derivation_box);

    // // // Hardened path
    // // let hardened_frame = gtk::Frame::new(Some("Hardened path"));
    // // let hardened_checkbox = gtk4::CheckButton::new();
    // // hardened_checkbox.set_active(true);
    // // hardened_checkbox.set_margin_start(10);
    // // hardened_frame.set_child(Some(&hardened_checkbox));
    // // hardened_frame.set_hexpand(true);
    
    // Start Seed sidebar
    stack.add_titled(&main_address_box, Some("sidebar-address"), "Address");
 
    
    let main_content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    main_content_box.append(&stack_sidebar);
    main_content_box.append(&stack);
    window.set_child(Some(&main_content_box));

    window.present();
}

fn get_coins_starting_with<'a>(
    coin_store: &'a Vec<CoinDatabase>, 
    target_prefix: &'a str
) -> Vec<&'a CoinDatabase> {
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
            let private_header: String = record.get(5).unwrap_or_default().to_string();
            let public_header: String = record.get(6).unwrap_or_default().to_string();
            let public_key_hash: String = record.get(7).unwrap_or_default().to_string();
            let script_hash: String = record.get(8).unwrap_or_default().to_string();
            let wif: String = record.get(9).unwrap_or_default().to_string();
            let comment: String = record.get(10).unwrap_or_default().to_string();

            CoinDatabase { index, path, symbol, name, key_derivation, private_header, public_header, public_key_hash, script_hash, wif, comment }
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
    // println!("opad_inner length: {}", opad_inner.len());
    // println!("inner_hash length: {}", inner_hash.len());
    // Calculate outer hash
    Sha512::digest(&opad_inner).to_vec() 
}

fn derive_master_keys(
    seed: &str, 
    mut private_header: &str,
    mut public_header: &str,
) -> Result<(String, String), String> {
    
    // Reverting to Bitcoin in case that coin is undefined
    if private_header.is_empty() {private_header = "0x0488ADE4";}
    if public_header.is_empty() {public_header = "0x0488B21E";}
    // Default message for all blockchains
    let message = "Bitcoin seed";

    let private_header = u32::from_str_radix(private_header.trim_start_matches("0x"), 16)
        .expect("Can not parse private header");
    let public_header = u32::from_str_radix(public_header.trim_start_matches("0x"), 16)
        .expect("Can not parse public header");

    let seed_bytes = hex::decode(seed).expect("Can not decode seed");
    let hmac_result = hmac_sha512(message.as_bytes(), &seed_bytes);
    let (master_private_key_bytes, chain_code_bytes) = hmac_result.split_at(32);

    // Private construct
    let mut master_private_key = Vec::new();
    master_private_key.extend_from_slice(&u32::to_be_bytes(private_header));     // Version        4 bytes
    master_private_key.push(0x00);                                                    // Depth          1 byte
    master_private_key.extend([0x00; 4].iter());                                      // Parent finger  4 bytes
    master_private_key.extend([0x00; 4].iter());                                      // Index/child    4 bytes
    master_private_key.extend_from_slice(chain_code_bytes);                           // Chain code     32 bytes
    master_private_key.push(0x00);                                                    // Key prefix     1 byte
    master_private_key.extend_from_slice(master_private_key_bytes);                   // Key            32 bytes
    let checksum: [u8; 4] = calculate_checksum(&master_private_key);            // Checksum       4 bytes
    master_private_key.extend_from_slice(&checksum);
    
    let master_xprv = bs58::encode(&master_private_key).into_string(); // Total      82 bytes
    println!("Master private key: {}", master_xprv);

    // Public construct
    let secp = secp256k1::Secp256k1::new();
    let master_secret_key = secp256k1::SecretKey::from_slice(&master_private_key_bytes)
        .map_err(|e| format!("Error creating private key: {:?}", e))?;
    let master_public_key_bytes = secp256k1::PublicKey::from_secret_key(&secp, &master_secret_key).serialize();

    let mut master_public_key = Vec::new();
    master_public_key.extend_from_slice(&u32::to_be_bytes(public_header));      // Version        4 bytes
    master_public_key.push(0x00);                                                     // Depth          1 byte
    master_public_key.extend([0x00; 4].iter());                                       // Parent finger  4 bytes
    master_public_key.extend([0x00; 4].iter());                                       // Index/child    4 bytes
    master_public_key.extend_from_slice(chain_code_bytes);                            // Chain code     32 bytes
    master_public_key.extend_from_slice(&master_public_key_bytes);                    // Key            33 bytes (compressed)
    let checksum: [u8; 4] = calculate_checksum(&master_public_key);              // Checksum       4 bytes
    master_public_key.extend_from_slice(&checksum);
    
    let master_xpub = bs58::encode(&master_public_key).into_string();   // Total      82 bytes
    println!("Master public key: {}", master_xpub);

    Ok((master_xprv, master_xpub))
}

fn calculate_checksum(data: &[u8]) -> [u8; 4] {
    let hash1 = Sha256::digest(data);
    let hash2 = Sha256::digest(&hash1);
    let checksum = &hash2[..4];
    let mut result = [0u8; 4];
    result.copy_from_slice(checksum);
    result
}














