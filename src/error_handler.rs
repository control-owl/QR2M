use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorHandler {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Bip39 error: {0}")]
    Bip39Error(#[from] bip39::Error),

    #[error("Bip32 error: {0}")]
    Bip32Error(#[from] bitcoin::bip32::Error),

    #[error("Custom error: {0}")]
    CustomError(String),

    #[error("Invalid entropy length. Allowed values are: {0}")]
    InvalidEntropyLength(String),

    #[error("Provied mnemonic has invalid word(s): {0}")]
    InvalidMnemonicWord(String),

    #[error("Problem with wordlist file")]
    WordlistReadError(),

    #[error("Provided file is too small compared with what is needed")]
    FileTooSmall(),

    #[error("Provided mnemonic is invalid")]
    InvalidMnemonic(),
}

impl From<&str> for ErrorHandler {
    fn from(message: &str) -> Self {
        ErrorHandler::CustomError(message.to_string())
    }
}

impl From<&str> for CustomError {
    fn from(message: &str) -> Self {
        CustomError(message.to_string())
    }
}

#[derive(Debug)]
struct CustomError(String);


// Example usage:
// let error: ErrorHandler = "This is a custom error message".into();
// println!("{:?}", error);