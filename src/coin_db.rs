// authors = ["Control Owl <qr2m[at]r-o0-t[dot]wtf>"]
// Extended Crypto-asset DataBase (ECDB)
// license = "CC-BY-NC-ND-4.0  [2023-2025]  Control Owl"

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

use super::{FunctionOutput, d3bug};
use csv::ReaderBuilder;
use glib::prelude::*;
use gtk4 as gtk;

const COINLIST_FILE: &str = "ECDB.csv";
pub const COIN_STATUS_NOT_SUPPORTED: u32 = 911; // ECDB Status: 0
pub const COIN_STATUS_VERIFIED: u32 = 257; // ECDB Status: 1
pub const COIN_STATUS_NOT_VERIFIED: u32 = 7; // ECDB Status: 2
pub const VALID_COIN_STATUS_NAME: &[&str] = &[
  // Coin status 2024-11-16
  "Not supported",
  "Verified",
  "Not verified",
  // "Not yet",
];

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

#[derive(Debug, Clone, Default)]
pub struct CryptoCoin {
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
  pub ucid: String,
  pub cmc_top: String,
}

mod implementation {
  use glib::{
    ParamSpecBuilderExt,
    prelude::*,
    subclass::{object::ObjectImpl, types::ObjectSubclass},
  };

  use super::CryptoCoin;

  #[derive(Default)]
  pub struct CoinDatabase {
    pub data: std::cell::RefCell<CryptoCoin>,
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
          glib::ParamSpecString::builder("status")
            .blurb("Status")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecUInt::builder("coin-index")
            .blurb("Coin Index")
            .minimum(0)
            .maximum(u32::MAX)
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("coin-symbol")
            .blurb("Coin Symbol")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("coin-name")
            .blurb("Coin Name")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("key-derivation")
            .blurb("Key Derivation")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("hash")
            .blurb("Hash")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("private-header")
            .blurb("Private Header")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("public-header")
            .blurb("Public Header")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("public-key-hash")
            .blurb("Public Key Hash")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("script-hash")
            .blurb("Script Hash")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("wallet-import-format")
            .blurb("Wallet Import Format")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("evm")
            .blurb("EVM")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("ucid")
            .blurb("UCID")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
          glib::ParamSpecString::builder("cmc-top")
            .blurb("CMC Top")
            .flags(glib::ParamFlags::READWRITE)
            .build(),
        ]
      })
    }

    fn set_property(&self, _id: usize, value: &glib::Value, specification: &glib::ParamSpec) {
      match specification.name() {
        "status" => self.data.borrow_mut().status = value.get().unwrap_or_default(),
        "coin-index" => self.data.borrow_mut().coin_index = value.get().unwrap_or_default(),
        "coin-symbol" => self.data.borrow_mut().coin_symbol = value.get().unwrap_or_default(),
        "coin-name" => self.data.borrow_mut().coin_name = value.get().unwrap_or_default(),
        "key-derivation" => self.data.borrow_mut().key_derivation = value.get().unwrap_or_default(),
        "hash" => self.data.borrow_mut().hash = value.get().unwrap_or_default(),
        "private-header" => self.data.borrow_mut().private_header = value.get().unwrap_or_default(),
        "public-header" => self.data.borrow_mut().public_header = value.get().unwrap_or_default(),
        "public-key-hash" => {
          self.data.borrow_mut().public_key_hash = value.get().unwrap_or_default()
        }
        "script-hash" => self.data.borrow_mut().script_hash = value.get().unwrap_or_default(),
        "wallet-import-format" => {
          self.data.borrow_mut().wallet_import_format = value.get().unwrap_or_default()
        }
        "evm" => self.data.borrow_mut().evm = value.get().unwrap_or_default(),
        "ucid" => self.data.borrow_mut().ucid = value.get().unwrap_or_default(),
        "cmc-top" => self.data.borrow_mut().cmc_top = value.get().unwrap_or_default(),
        _ => eprintln!("Unknown property"),
      }
    }

    fn property(&self, _id: usize, specification: &glib::ParamSpec) -> glib::Value {
      match specification.name() {
        "status" => self.data.borrow_mut().status.to_value(),
        "coin-index" => self.data.borrow_mut().coin_index.to_value(),
        "coin-symbol" => self.data.borrow_mut().coin_symbol.to_value(),
        "coin-name" => self.data.borrow_mut().coin_name.to_value(),
        "key-derivation" => self.data.borrow_mut().key_derivation.to_value(),
        "hash" => self.data.borrow_mut().hash.to_value(),
        "private-header" => self.data.borrow_mut().private_header.to_value(),
        "public-header" => self.data.borrow_mut().public_header.to_value(),
        "public-key-hash" => self.data.borrow_mut().public_key_hash.to_value(),
        "script-hash" => self.data.borrow_mut().script_hash.to_value(),
        "wallet-import-format" => self.data.borrow_mut().wallet_import_format.to_value(),
        "evm" => self.data.borrow_mut().evm.to_value(),
        "ucid" => self.data.borrow_mut().ucid.to_value(),
        "cmc-top" => self.data.borrow_mut().cmc_top.to_value(),
        _ => unimplemented!(),
      }
    }
  }
}

glib::wrapper! {
    pub struct CoinDatabase(ObjectSubclass<implementation::CoinDatabase>);
}

impl CoinDatabase {
  pub fn new(crypto_coin: CryptoCoin) -> Self {
    let builder = glib::Object::builder::<CoinDatabase>()
      .property("status", crypto_coin.status)
      .property("coin-index", crypto_coin.coin_index)
      .property("coin-symbol", crypto_coin.coin_symbol)
      .property("coin-name", crypto_coin.coin_name)
      .property("key-derivation", crypto_coin.key_derivation)
      .property("hash", crypto_coin.hash)
      .property("private-header", crypto_coin.private_header)
      .property("public-header", crypto_coin.public_header)
      .property("public-key-hash", crypto_coin.public_key_hash)
      .property("script-hash", crypto_coin.script_hash)
      .property("wallet-import-format", crypto_coin.wallet_import_format)
      .property("evm", crypto_coin.evm)
      .property("ucid", crypto_coin.ucid)
      .property("cmc-top", crypto_coin.cmc_top);

    builder.build()
  }
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.

pub fn create_coin_store() -> FunctionOutput<gtk::gio::ListStore> {
  d3bug(">>> create_coin_store", "debug");

  let resource_path = std::path::Path::new("coin").join(COINLIST_FILE);

  let coin_store_path = resource_path
    .to_str()
    .ok_or_else(|| crate::AppError::Custom("Failed to find COINLIST_FILE".to_string()))?;

  d3bug(&format!("Coin store path: {:?}", coin_store_path), "debug");

  let csv_content = qr2m_lib::get_text_from_resources(coin_store_path);

  if csv_content.is_empty() {
    return Err(crate::AppError::Custom(
      "Failed to retrieve CSV from COINLIST_FILE. CSV empty.".to_string(),
    ));
  } else {
    d3bug("Coin store loaded", "debug");
  }

  let mut store_reader = ReaderBuilder::new()
    .has_headers(true)
    .from_reader(std::io::Cursor::new(csv_content));
  let store = gtk::gio::ListStore::new::<CoinDatabase>();

  for result in store_reader.records() {
    let record =
      result.map_err(|err| crate::AppError::Custom(format!("Error reading CSV record: {err}")))?;
    let number_status = record[0].to_string();
    let status = match number_status.as_str() {
      "0" => VALID_COIN_STATUS_NAME[0],
      "1" => VALID_COIN_STATUS_NAME[1],
      "2" => VALID_COIN_STATUS_NAME[2],
      // "3" => VALID_COIN_STATUS_NAME[3],
      _ => "Unknown status",
    }
    .to_string();

    let coin_index: u32 = record[1]
      .parse()
      .map_err(|err| crate::AppError::Custom(format!("Error parsing coin_index: {err}")))?;

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

    let crypto_coin = CryptoCoin {
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
      ucid,
      cmc_top,
    };

    let coin = CoinDatabase::new(crypto_coin);

    store.append(&coin);
  }

  Ok(store)
}

pub fn create_coin_completion_model() -> FunctionOutput<gtk::gio::ListStore> {
  d3bug(">>> create_coin_completion_model", "debug");

  let crypto_coins = create_coin_database()?;
  let store = gtk::gio::ListStore::new::<CoinDatabase>();

  for item in crypto_coins.iter() {
    store.append(item);
  }

  Ok(store)
}

pub fn create_coin_store_filters(
  part: &str,
  target_value: &str,
) -> FunctionOutput<gtk::CustomFilter> {
  d3bug(">>> create_coin_store_filters", "debug");

  let part = part.to_string();
  let target_value = target_value.to_string();

  let filter_func = move |item: &glib::Object| {
    let coin = match item.downcast_ref::<CoinDatabase>() {
      Some(c) => c,
      None => {
        d3bug("Failed to downcast to CoinDatabase", "error");
        return false;
      }
    };

    match part.as_str() {
      "Cmc_top" => match target_value.as_str() {
        "10" => coin.property::<String>("cmc-top").to_lowercase() == "10",
        "100" => {
          let cmc_top = coin.property::<String>("cmc-top").to_lowercase();
          cmc_top == "10" || cmc_top == "100"
        }
        _ => coin.property::<String>("cmc-top").to_lowercase() == target_value.to_lowercase(),
      },
      "Status" => coin.property::<String>("status").to_lowercase() == target_value.to_lowercase(),
      "Index" => coin.property::<u32>("coin-index").to_string() == target_value,
      "Symbol" => coin
        .property::<String>("coin-symbol")
        .to_lowercase()
        .contains(&target_value.to_lowercase()),
      _ => coin
        .property::<String>("coin-name")
        .to_lowercase()
        .contains(&target_value.to_lowercase()),
    }
  };

  Ok(gtk::CustomFilter::new(filter_func))
}

pub fn create_sorter() -> FunctionOutput<gtk::CustomSorter> {
  fn extract_cmc_top(item: &glib::Object) -> FunctionOutput<usize> {
    item
      .downcast_ref::<CoinDatabase>()
      .map(|coin| {
        coin
          .property::<String>("cmc-top")
          .parse::<usize>()
          .map_err(|e| {
            crate::AppError::Custom(format!("Failed to parse cmc-top as usize: {:?}", e))
          })
      })
      .ok_or_else(|| crate::AppError::Custom("local_config_file not set".to_string()))?
  }

  let sorter = move |item1: &glib::Object, item2: &glib::Object| -> gtk::Ordering {
    let cmc_top1 = extract_cmc_top(item1);
    let cmc_top2 = extract_cmc_top(item2);

    match (cmc_top1, cmc_top2) {
      (Ok(value1), Ok(value2)) => value1.cmp(&value2).into(),
      (Err(err), _) => {
        d3bug(
          &format!("Failed to extract cmc-top for item1: {err}"),
          "error",
        );
        gtk::Ordering::Larger
      }
      (_, Err(err)) => {
        d3bug(
          &format!("Failed to extract cmc-top for item2: {err}"),
          "error",
        );
        gtk::Ordering::Smaller
      }
    }
  };

  Ok(gtk::CustomSorter::new(sorter))
}

fn create_coin_database() -> FunctionOutput<Vec<CoinDatabase>> {
  let resource_path = std::path::Path::new("coin").join("ECDB.csv");
  let resource_path_str = resource_path.to_str().unwrap_or_default();
  let csv_content = qr2m_lib::get_text_from_resources(resource_path_str);

  if csv_content.is_empty() {
    return Err(crate::AppError::Custom(
      "Failed to retrieve CSV content from resources".to_string(),
    ));
  }

  let mut rdr = ReaderBuilder::new()
    .has_headers(true)
    .from_reader(csv_content.as_bytes());

  let coin_types: Vec<CoinDatabase> = rdr
    .records()
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

      let crypto_coin = CryptoCoin {
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
        ucid,
        cmc_top,
      };

      CoinDatabase::new(crypto_coin)
    })
    .collect();

  Ok(coin_types)
}

// -.-. --- .--. -.-- .-. .. --. .... - / --.- .-. ..--- -- .- - .-. --- ----- - -.. --- - .-- - ..-.
