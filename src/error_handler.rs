use std::fmt;
use std::io;
use bip39;
use bitcoin::bip32;


#[derive(Debug)]
pub enum CustomError {
    IOError(io::Error),
    Bip39Error(bip39::Error),
    Bip32Error(bip32::Error),
    FileTooSmall(String),
    InvalidEntropyLength(String),
    InvalidBipEntry(String),
    InvalidMnemonicWord(String),
    WordlistReadError,
    InvalidMnemonicWordCount(String),
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomError::IOError(err) => write!(f, "IO Error: {}", err),
            CustomError::Bip39Error(err) => write!(f, "Bip39 Error: {}", err),
            CustomError::Bip32Error(err) => write!(f, "Bip32 Error: {}", err),

            CustomError::WordlistReadError => write!(f, "There was a problem with reading wordlist file"),
            CustomError::FileTooSmall(value) => write!(f, "The provided file is too small: {}", value),
            CustomError::InvalidEntropyLength(value) => write!(f, "Invalid entropy length.\nAllowed values are: {}", value),
            CustomError::InvalidBipEntry(value) => write!(f, "The provided BIP is invalid.\nAllowed values are: {}", value),
            CustomError::InvalidMnemonicWord(value) => write!(f, "The provided mnemonic has invalid word: {}", value),
            CustomError::InvalidMnemonicWordCount(value) => write!(f, "Unfortunately, the entered mnemonic is not valid. \nThis program supports only specific word counts: {}", value),
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