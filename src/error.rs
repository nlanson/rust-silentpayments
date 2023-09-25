use std::fmt;

#[derive(Debug)]
pub enum Error {
    GenericError(String),
    InvalidLabel(String),
    InvalidAddress(String),
    InvalidSharedSecret(String),
    Secp256k1Error(bitcoin::secp256k1::Error),
    OutOfRangeError(bitcoin::secp256k1::scalar::OutOfRangeError),
    IOError(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self{
            Error::GenericError(msg) => write!(f, "{}", msg),
            Error::InvalidLabel(msg) => write!(f, "{}", msg),
            Error::InvalidAddress(msg) => write!(f, "{}", msg),
            Error::InvalidSharedSecret(msg) => write!(f, "{}", msg),
            Error::Secp256k1Error(e) => e.fmt(f),
            Error::OutOfRangeError(e) => e.fmt(f),
            Error::IOError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error  {}

impl From<hex::FromHexError> for Error {
    fn from(e: hex::FromHexError) -> Self {
        Error::InvalidLabel(e.to_string())
    }
}

impl From<bitcoin::bech32::Error> for Error {
    fn from(e: bitcoin::bech32::Error) -> Self {
        Error::InvalidAddress(e.to_string())
    }
}

impl From<bitcoin::secp256k1::Error> for Error {
    fn from(e: bitcoin::secp256k1::Error) -> Self {
        Error::Secp256k1Error(e)
    }
}

impl From<bitcoin::secp256k1::scalar::OutOfRangeError> for Error {
    fn from(e: bitcoin::secp256k1::scalar::OutOfRangeError) -> Self {
        Error::OutOfRangeError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e)
    }
}
