use crate::auth::cipher::CipherError;
use crate::auth::pass::PassError;
use crate::auth::twofa::TwoFAError;

pub type IdType = u32;

pub const FIELD_SEP: char = '|';
pub const SECRET_SEP: char = '#';
pub const CREDS_SEP: char = '%';
pub const STORE_SEP: char = '&';

#[derive(Debug)]
pub enum DataError {
    Parse(String),
    ChecksumMismatch,
    Cipher(CipherError),
    LoadError(String),
    SaveError(String),
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataError::Parse(s) => write!(f, "parse failed: {s}"),
            DataError::ChecksumMismatch => write!(f, "checksum mismatch"),
            DataError::Cipher(e) => write!(f, "{e}"),
            DataError::LoadError(s) => write!(f, "load failed: {s}"),
            DataError::SaveError(s) => write!(f, "save failed: {s}"),
        }
    }
}

impl std::error::Error for DataError {}

impl From<CipherError> for DataError {
    fn from(e: CipherError) -> Self {
        DataError::Cipher(e)
    }
}

impl From<PassError> for DataError {
    fn from(e: PassError) -> Self {
        DataError::Parse(e.to_string())
    }
}

impl From<TwoFAError> for DataError {
    fn from(e: TwoFAError) -> Self {
        DataError::Parse(e.to_string())
    }
}
