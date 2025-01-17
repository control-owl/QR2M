// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// module = "Extended Crypto-asset DataBase (ECDB)"
// copyright = "Copyright © 2023-2025 Control Owl"
// version = "2024-11-16"


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


use csv::ReaderBuilder;
use gtk4 as gtk;

const COINLIST_FILE: &str = "ECDB.csv";

// Coin status 2024-11-16
pub const VALID_COIN_STATUS_NAME: &'static [&'static str] = &[
    "Not supported", 
    "Verified", 
    "Not verified",
    "Not yet",
];
pub const COIN_STATUS_NOT_SUPPORTED: u32 = 899;     // ECDB Status: 0
pub const COIN_STATUS_VERIFIED: u32 = 254;          // ECDB Status: 1
pub const COIN_STATUS_NOT_VERIFIED: u32 = 10;       // ECDB Status: 2
pub const COIN_STATUS_IN_PLAN: u32 = 12;            // ECDB Status: 3


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.


#[derive(Clone)]
pub struct CoinDatabase {
    pub status: String,
    pub coin_index: u32,
    pub coin_symbol: String,
    pub coin_name: String,
    pub key_derivation: String,
    pub hash: String,
    pub private_header: String,
    pub public_header: String,
    pub public_key_hash: String,
    pub script_hash: String,
    pub wallet_import_format: String,
    pub evm: String,
    pub UCID: String,
    pub cmc_top: String,
}

pub fn create_coin_store() -> Vec<CoinDatabase> {
    let resource_path = std::path::Path::new("coin").join(COINLIST_FILE);
    let path_str = resource_path.to_str().expect("Failed to convert path to string");
    let csv_content = qr2m_lib::get_text_from_resources(path_str);

    if csv_content.is_empty() {
        eprintln!("Failed to retrieve CSV from embedded resources");
        return Vec::new();
    }

    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(std::io::Cursor::new(csv_content));
    let mut coin_store = Vec::new();

    for result in rdr.records() {
        let record = result.expect(&t!("error.csv.read").to_string());
        
        let number_status = record[0].to_string();
        let status = match number_status.as_str() {
            "0" => VALID_COIN_STATUS_NAME[0],
            "1" => VALID_COIN_STATUS_NAME[1],
            "2" => VALID_COIN_STATUS_NAME[2],
            "3" => VALID_COIN_STATUS_NAME[3],
            _ => "Unknown status",
        }.to_string();


        let coin_index: u32 = record[1].parse().expect(&t!("error.csv.parse", value = "coin_index").to_string());
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
        let UCID = record[12].to_string();
        let cmc_top = record[13].to_string();

        
        let coin_type = CoinDatabase { 
            status, 
            coin_index, 
            coin_symbol, 
            coin_name, 
            key_derivation, 
            hash,
            private_header, 
            public_header, 
            public_key_hash, 
            script_hash, 
            wallet_import_format,
            evm,
            UCID,
            cmc_top, 
        };

        coin_store.push(coin_type);
    }

    coin_store
}

pub fn create_coin_completion_model() -> gtk::ListStore {
    let valid_coin_symbols = create_coin_database();

    let store = gtk::ListStore::new(&[
        glib::Type::STRING, // status
        glib::Type::U32,    // index
        glib::Type::STRING, // coin_symbol
        glib::Type::STRING, // coin_name
        glib::Type::STRING, // key_derivation
        glib::Type::STRING, // hash
        glib::Type::STRING, // private_header
        glib::Type::STRING, // public_header
        glib::Type::STRING, // public_key_hash
        glib::Type::STRING, // script_hash
        glib::Type::STRING, // wallet_import_format
        glib::Type::STRING, // evm
        glib::Type::STRING, // UCID
        glib::Type::STRING, // cmc_top
    ]);

    for coin_symbol in valid_coin_symbols.iter() {
        let iter = store.append();
        store.set(&iter, &[
            (0, &coin_symbol.status),
            (1, &coin_symbol.coin_index), 
            (2, &coin_symbol.coin_symbol), 
            (3, &coin_symbol.coin_name),
            (4, &coin_symbol.key_derivation),
            (5, &coin_symbol.hash),
            (6, &coin_symbol.private_header),
            (7, &coin_symbol.public_header),
            (8, &coin_symbol.public_key_hash),
            (9, &coin_symbol.script_hash),
            (10, &coin_symbol.wallet_import_format),
            (11, &coin_symbol.evm),
            (12, &coin_symbol.UCID),
            (13, &coin_symbol.cmc_top),
        ]);
    }

    store
}

pub fn fetch_coins_from_database<'a>(
    part: &'a str,
    coin_store: &'a Vec<CoinDatabase>, 
    target_value: &'a str
) -> Vec<&'a CoinDatabase> {

    let mut result: Vec<&'a CoinDatabase> = match part {
        "Symbol" => {
            coin_store
                .iter()
                .filter(|&coin_type| coin_type.coin_symbol.to_lowercase().contains(target_value))
                .collect()
        },
        "Cmc_top" => {
            match target_value {
                "10" => {
                    coin_store
                        .iter()
                        .filter(|&coin| coin.cmc_top.to_lowercase() == "10")
                        .collect()
                },
                "100" => {
                    coin_store
                        .iter()
                        .filter(|&coin| coin.cmc_top.to_lowercase() == "10" || coin.cmc_top.to_lowercase() == "100")
                        .collect()
                },
                _ => {
                    coin_store
                        .iter()
                        .filter(|&coin| coin.cmc_top.to_lowercase() == target_value.to_lowercase())
                        .collect()
                }
            }
        },
        "Status" => {
            coin_store
                .iter()
                .filter(|&coin| coin.status.to_lowercase() == target_value.to_lowercase())
                .collect()
        },
        "Index" => {
            coin_store
                .iter()
                .filter(|&coin_type| coin_type.coin_index == target_value.parse().unwrap_or(0))
                .collect()
        },
        "Name" | _ => {
            coin_store
                .iter()
                .filter(|&coin_type| coin_type.coin_name.to_lowercase().contains(target_value))
                .collect()
        },
    };

    // Sort the result by cmc_top in ascending order
    result.sort_by_key(|coin| coin.cmc_top.parse::<usize>().unwrap_or(usize::MAX));

    result
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
        .enumerate()
        .map(|(_index, record)| {
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
            let UCID: String = record.get(12).unwrap_or_default().to_string();
            let cmc_top: String = record.get(13).unwrap_or_default().to_string();

            CoinDatabase {
                status,
                coin_index,
                coin_symbol,
                coin_name,
                key_derivation,
                hash,
                private_header,
                public_header,
                public_key_hash,
                script_hash,
                wallet_import_format,
                evm,
                UCID,
                cmc_top,
            }
        }).collect();

    coin_types
}


// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
