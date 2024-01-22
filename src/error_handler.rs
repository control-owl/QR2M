use std::fmt;
use std::io;
use bip39;
use bitcoin::bip32;


#[derive(Debug)]
pub enum CustomError {
    CSVError(csv::Error),
    ParseError(std::num::ParseIntError),
    IOError(io::Error),
    Bip39Error(bip39::Error),
    Bip32Error(bip32::Error),
    BitcoinAddressError(bitcoin::address::Error),
    FileTooSmall(String),
    InvalidEntropyLength(String),
    InvalidBipEntry(String),
    InvalidMnemonicWord(String),
    WordlistReadError,
    InvalidMnemonicWordCount(String),
    InvalidSourceEntry(String),
    InvalidCoinSymbol(String),
    InvalidSeed(String),
    DecodingError(String),
    InputNotValidNumber(String),
    New(String),
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomError::CSVError(err) => write!(f, "CSV Error: {}", err),
            CustomError::ParseError(err) => write!(f, "Parse Error: {}", err),
            CustomError::IOError(err) => write!(f, "IO Error: {}", err),
            CustomError::Bip39Error(err) => write!(f, "Bip39 Error: {}", err),
            CustomError::Bip32Error(err) => write!(f, "Bip32 Error: {}", err),
            CustomError::BitcoinAddressError(err) => write!(f, "Bitcoin Address Error: {}", err),

            CustomError::WordlistReadError => write!(f, "There was a problem with reading wordlist file"),
            CustomError::FileTooSmall(value) => write!(f, "The provided file is too small: {}", value),
            CustomError::InvalidEntropyLength(value) => write!(f, "Invalid entropy length.\nAllowed values are: {}", value),
            CustomError::InvalidBipEntry(value) => write!(f, "The provided BIP is invalid.\nAllowed values are: {}", value),
            CustomError::InvalidMnemonicWord(value) => write!(f, "The provided mnemonic has invalid word: {}", value),
            CustomError::InvalidMnemonicWordCount(value) => write!(f, "Unfortunately, the entered mnemonic is not valid. \nThis program supports only specific word counts: {}", value),
            CustomError::InvalidSourceEntry(value) => write!(f, "Source for entropy invalid. \nThis program supports only: {} as arguments", value),
            CustomError::InvalidCoinSymbol(value) => write!(f, "The provided coin is not supported: {}", value),
            CustomError::InvalidSeed(value) => write!(f, "The provided seed is not valid.\nstring contains non hexadecimal characters: {}", value),
            CustomError::DecodingError(value) => write!(f, "The provided seed could not be decoded: {}", value),
            CustomError::InputNotValidNumber(value) => write!(f, "This is not a valid number.\nPlease provide a whole number greater than or equal to 1.\nYour input was: {}", value),
            CustomError::New(value) => write!(f, "NEW ERROR: {}", value),
        }
    }
}

impl From<io::Error> for CustomError {
    fn from(err: io::Error) -> Self {
        CustomError::IOError(err)
    }
}

impl From<bip39::Error> for CustomError {
    fn from(err: bip39::Error) -> Self {
        CustomError::Bip39Error(err)
    }
}

impl From<bip32::Error> for CustomError {
    fn from(err: bip32::Error) -> Self {
        CustomError::Bip32Error(err)
    }
}

impl From<bitcoin::address::Error> for CustomError {
    fn from(err: bitcoin::address::Error) -> Self {
        CustomError::BitcoinAddressError(err)
    }
}

impl From<csv::Error> for CustomError {
    fn from(err: csv::Error) -> Self {
        CustomError::CSVError(err)
    }
}

impl From<std::num::ParseIntError> for CustomError {
    fn from(err: std::num::ParseIntError) -> Self {
        CustomError::ParseError(err)
    }
}
