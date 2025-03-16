// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// module = "Extended Crypto-asset DataBase (ECDB)"
// copyright = "Copyright © 2023-2025 Control Owl"
// version = "2025-03-16"


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


use csv::ReaderBuilder;
use gtk4 as gtk;
use glib::prelude::*;

const COINLIST_FILE: &str = "ECDB.csv";
pub const COIN_STATUS_NOT_SUPPORTED: u32 = 899;     // ECDB Status: 0
pub const COIN_STATUS_VERIFIED: u32 = 254;          // ECDB Status: 1
pub const COIN_STATUS_NOT_VERIFIED: u32 = 10;       // ECDB Status: 2
pub const COIN_STATUS_IN_PLAN: u32 = 12;            // ECDB Status: 3
pub const VALID_COIN_STATUS_NAME: &[&str] = &[
    // Coin status 2024-11-16
    "Not supported", 
    "Verified", 
    "Not verified",
    "Not yet",
];


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


mod implementation {
    use glib::{
        prelude::*,
        subclass::{
            object::ObjectImpl, 
            types::ObjectSubclass
        },
        ParamSpecBuilderExt,
    };

    #[derive(Default)]
    pub struct CoinDatabase {
        pub status: std::cell::RefCell<String>,
        pub coin_index: std::cell::RefCell<u32>,
        pub coin_symbol: std::cell::RefCell<String>,
        pub coin_name: std::cell::RefCell<String>,
        pub key_derivation: std::cell::RefCell<String>,
        pub hash: std::cell::RefCell<String>,
        pub private_header: std::cell::RefCell<String>,
        pub public_header: std::cell::RefCell<String>,
        pub public_key_hash: std::cell::RefCell<String>,
        pub script_hash: std::cell::RefCell<String>,
        pub wallet_import_format: std::cell::RefCell<String>,
        pub evm: std::cell::RefCell<String>,
        pub ucid: std::cell::RefCell<String>,
        pub cmc_top: std::cell::RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CoinDatabase {
        const NAME: &'static str = "CoinDatabase";

        type Type = super::CoinDatabase;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for CoinDatabase {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: std::sync::OnceLock<Vec<glib::ParamSpec>> = std::sync::OnceLock::new();
            
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecString::builder("status").blurb("Status").flags(glib::ParamFlags::READWRITE).build(),
                    glib::ParamSpecUInt::builder("coin-index").blurb("Coin Index").minimum(0).maximum(u32::MAX).flags(glib::ParamFlags::READWRITE).build(),
                    glib::ParamSpecString::builder("coin-symbol").blurb("Coin Symbol").flags(glib::ParamFlags::READWRITE).build(),
                    glib::ParamSpecString::builder("coin-name").blurb("Coin Name").flags(glib::ParamFlags::READWRITE).build(),
                    glib::ParamSpecString::builder("key-derivation").blurb("Key Derivation").flags(glib::ParamFlags::READWRITE).build(),
                    glib::ParamSpecString::builder("hash").blurb("Hash").flags(glib::ParamFlags::READWRITE).build(),
                    glib::ParamSpecString::builder("private-header").blurb("Private Header").flags(glib::ParamFlags::READWRITE).build(),
                    glib::ParamSpecString::builder("public-header").blurb("Public Header").flags(glib::ParamFlags::READWRITE).build(),
                    glib::ParamSpecString::builder("public-key-hash").blurb("Public Key Hash").flags(glib::ParamFlags::READWRITE).build(),
                    glib::ParamSpecString::builder("script-hash").blurb("Script Hash").flags(glib::ParamFlags::READWRITE).build(),
                    glib::ParamSpecString::builder("wallet-import-format").blurb("Wallet Import Format").flags(glib::ParamFlags::READWRITE).build(),
                    glib::ParamSpecString::builder("evm").blurb("EVM").flags(glib::ParamFlags::READWRITE).build(),
                    glib::ParamSpecString::builder("ucid").blurb("UCID").flags(glib::ParamFlags::READWRITE).build(),
                    glib::ParamSpecString::builder("cmc-top").blurb("CMC Top").flags(glib::ParamFlags::READWRITE).build(),
                ]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, specification: &glib::ParamSpec) {
            match specification.name() {
                "status" => *self.status.borrow_mut() = value.get().unwrap_or_default(),
                "coin-index" => *self.coin_index.borrow_mut() = value.get().unwrap_or_default(),
                "coin-symbol" => *self.coin_symbol.borrow_mut() = value.get().unwrap_or_default(),
                "coin-name" => *self.coin_name.borrow_mut() = value.get().unwrap_or_default(),
                "key-derivation" => *self.key_derivation.borrow_mut() = value.get().unwrap_or_default(),
                "hash" => *self.hash.borrow_mut() = value.get().unwrap_or_default(),
                "private-header" => *self.private_header.borrow_mut() = value.get().unwrap_or_default(),
                "public-header" => *self.public_header.borrow_mut() = value.get().unwrap_or_default(),
                "public-key-hash" => *self.public_key_hash.borrow_mut() = value.get().unwrap_or_default(),
                "script-hash" => *self.script_hash.borrow_mut() = value.get().unwrap_or_default(),
                "wallet-import-format" => *self.wallet_import_format.borrow_mut() = value.get().unwrap_or_default(),
                "evm" => *self.evm.borrow_mut() = value.get().unwrap_or_default(),
                "ucid" => *self.ucid.borrow_mut() = value.get().unwrap_or_default(),
                "cmc-top" => *self.cmc_top.borrow_mut() = value.get().unwrap_or_default(),
                _ => eprintln!("Unknown property"),
            }
        }

        fn property(&self, _id: usize, specification: &glib::ParamSpec) -> glib::Value {
            match specification.name() {
                "status" => self.status.borrow().to_value(),
                "coin-index" => self.coin_index.borrow().to_value(),
                "coin-symbol" => self.coin_symbol.borrow().to_value(),
                "coin-name" => self.coin_name.borrow().to_value(),
                "key-derivation" => self.key_derivation.borrow().to_value(),
                "hash" => self.hash.borrow().to_value(),
                "private-header" => self.private_header.borrow().to_value(),
                "public-header" => self.public_header.borrow().to_value(),
                "public-key-hash" => self.public_key_hash.borrow().to_value(),
                "script-hash" => self.script_hash.borrow().to_value(),
                "wallet-import-format" => self.wallet_import_format.borrow().to_value(),
                "evm" => self.evm.borrow().to_value(),
                "ucid" => self.ucid.borrow().to_value(),
                "cmc-top" => self.cmc_top.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct CoinDatabase(ObjectSubclass<implementation::CoinDatabase>);
}

impl CoinDatabase {
    pub fn new(
        status: &str,
        coin_index: u32,
        coin_symbol: &str,
        coin_name: &str,
        key_derivation: &str,
        hash: &str,
        private_header: &str,
        public_header: &str,
        public_key_hash: &str,
        script_hash: &str,
        wallet_import_format: &str,
        evm: &str,
        ucid: &str,
        cmc_top: &str,
    ) -> Self {
        let builder = glib::Object::builder::<CoinDatabase>()
            .property("status", status)
            .property("coin-index", coin_index)
            .property("coin-symbol", coin_symbol)
            .property("coin-name", coin_name)
            .property("key-derivation", key_derivation)
            .property("hash", hash)
            .property("private-header", private_header)
            .property("public-header", public_header)
            .property("public-key-hash", public_key_hash)
            .property("script-hash", script_hash)
            .property("wallet-import-format", wallet_import_format)
            .property("evm", evm)
            .property("ucid", ucid)
            .property("cmc-top", cmc_top);

        builder.build()
    }
}

pub fn create_coin_store() -> gtk::gio::ListStore {
    let resource_path = std::path::Path::new("coin").join(COINLIST_FILE);
    let path_str = resource_path.to_str().expect("Failed to convert path to string");
    let csv_content = qr2m_lib::get_text_from_resources(path_str);

    if csv_content.is_empty() {
        eprintln!("Failed to retrieve CSV from embedded resources");
        return gtk::gio::ListStore::new::<CoinDatabase>();
    }

    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(std::io::Cursor::new(csv_content));
    let store = gtk::gio::ListStore::new::<CoinDatabase>();

    for result in rdr.records() {
        let record = result.expect(&t!("error.csv.read"));
        
        let number_status = record[0].to_string();
        let status = match number_status.as_str() {
            "0" => VALID_COIN_STATUS_NAME[0],
            "1" => VALID_COIN_STATUS_NAME[1],
            "2" => VALID_COIN_STATUS_NAME[2],
            "3" => VALID_COIN_STATUS_NAME[3],
            _ => "Unknown status",
        }.to_string();


        let coin_index: u32 = record[1].parse().expect(&t!("error.csv.parse", value = "coin_index"));
        let coin_symbol = record[2].to_string();
        let coin_name = record[3].to_string();
        let key_derivation = record[4].to_string();
        let hash = record[5].to_string();
        let private_header = record[6].to_string();
        let public_header = record[7].to_string();
        let public_key_hash = record[8].to_string();
        let script_hash = record[9].to_string();
        let wallet_import_format = record[10].to_string();
        let evm = record[11].to_string();
        let ucid = record[12].to_string();
        let cmc_top = record[13].to_string();

        
        let coin = CoinDatabase::new(
            &status,
            coin_index,
            &coin_symbol,
            &coin_name,
            &key_derivation,
            &hash,
            &private_header,
            &public_header,
            &public_key_hash,
            &script_hash,
            &wallet_import_format,
            &evm,
            &ucid,
            &cmc_top,
        );

        store.append(&coin);
    }

    store
}

pub fn create_coin_completion_model() -> gtk::gio::ListStore {
    let crypto_coins = create_coin_database();
    let store =  gtk::gio::ListStore::new::<CoinDatabase>();

    for item in crypto_coins.iter() {
        store.append(item);
    }

    store
}

pub fn create_filter(part: &str, target_value: &str) -> gtk::CustomFilter {
    let part = part.to_string();
    let target_value = target_value.to_string();

    let filter_func = move |item: &glib::Object| {
        let coin = item.downcast_ref::<CoinDatabase>().expect("Failed to downcast to CoinDatabase");
        match part.as_str() {
            "Cmc_top" => match target_value.as_str() {
                "10" => coin.property::<String>("cmc-top").to_lowercase() == "10",
                "100" => {
                    let cmc_top = coin.property::<String>("cmc-top").to_lowercase();
                    cmc_top == "10" || cmc_top == "100"
                },
                _ => coin.property::<String>("cmc-top").to_lowercase() == target_value.to_lowercase(),
            },
            "Status" => coin.property::<String>("status").to_lowercase() == target_value.to_lowercase(),
            "Index" => coin.property::<u32>("coin-index").to_string() == target_value,
            "Symbol" => coin.property::<String>("coin-symbol").to_lowercase().contains(&target_value.to_lowercase()),
            _ => coin.property::<String>("coin-name").to_lowercase().contains(&target_value.to_lowercase()),
        }
    };

    gtk::CustomFilter::new(filter_func)
}

pub fn create_sorter() -> gtk::CustomSorter {
    fn extract_cmc_top(item: &glib::Object) -> usize {
        item.downcast_ref::<CoinDatabase>()
            .and_then(|coin| coin.property::<String>("cmc-top").parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    }

    let sorter = move |item1: &glib::Object, item2: &glib::Object| -> gtk::Ordering {
        extract_cmc_top(item1).cmp(&extract_cmc_top(item2)).into()
    };

    gtk::CustomSorter::new(sorter)
}

fn create_coin_database() -> Vec<CoinDatabase> {
    let resource_path = std::path::Path::new("coin").join("ECDB.csv");
    let resource_path_str = resource_path.to_str().unwrap_or_default();
    let csv_content = qr2m_lib::get_text_from_resources(resource_path_str);
    
    if csv_content.is_empty() {
        eprintln!("Error: Failed to retrieve CSV file from resources.");
        return Vec::new();
    }

    let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_reader(csv_content.as_bytes());

    let coin_types: Vec<CoinDatabase> = rdr.records()
        .filter_map(|record| record.ok())
        .map(|record| {
            let status: String = record.get(0).unwrap_or_default().to_string();
            let coin_index: u32 = record.get(1).unwrap_or_default().parse().unwrap();
            let coin_symbol: String = record.get(2).unwrap_or_default().to_string();
            let coin_name: String = record.get(3).unwrap_or_default().to_string();
            let key_derivation: String = record.get(4).unwrap_or_default().to_string();
            let hash: String = record.get(5).unwrap_or_default().to_string();
            let private_header: String = record.get(6).unwrap_or_default().to_string();
            let public_header: String = record.get(7).unwrap_or_default().to_string();
            let public_key_hash: String = record.get(8).unwrap_or_default().to_string();
            let script_hash: String = record.get(9).unwrap_or_default().to_string();
            let wallet_import_format: String = record.get(10).unwrap_or_default().to_string();
            let evm: String = record.get(11).unwrap_or_default().to_string();
            let ucid: String = record.get(12).unwrap_or_default().to_string();
            let cmc_top: String = record.get(13).unwrap_or_default().to_string();

            CoinDatabase::new(
                &status,
                coin_index,
                &coin_symbol,
                &coin_name,
                &key_derivation,
                &hash,
                &private_header,
                &public_header,
                &public_key_hash,
                &script_hash,
                &wallet_import_format,
                &evm,
                &ucid,
                &cmc_top,
            )
        }).collect();

    coin_types
}


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

