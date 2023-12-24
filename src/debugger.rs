use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorHandler {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Bip39 error: {0}")]
    Bip39Error(#[from] bip39::Error),

    #[error("{0}")]
    CustomError(String),
}

impl ErrorHandler {
    pub fn custom(message: &str) -> Self {
        ErrorHandler::CustomError(message.to_string())
    }
}

#[derive(Debug)]
struct CustomError(String);