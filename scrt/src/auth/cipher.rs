use std::string::FromUtf8Error;

use base64::{Engine, prelude::BASE64_STANDARD};

use chacha20poly1305::{
    ChaCha20Poly1305, ChaChaPoly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};

use argon2::Argon2;

#[derive(Clone, Debug)]
pub struct EncryptedData {
    salt: [u8; 16],
    nonce: [u8; 12],
    txt: String,
}

pub trait Serialized {
    type Error: std::error::Error + Send + Sync + 'static;
    fn dumps(&self) -> String;
    fn parse(data: &str) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

#[derive(Debug)]
pub enum CipherError {
    Parse(String),
    Serialize(String),
    Decrypt(chacha20poly1305::aead::Error),
    Utf8(FromUtf8Error),
}

impl std::error::Error for CipherError {}

impl std::fmt::Display for CipherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CipherError::Parse(s) => write!(f, "parse failed: {s}"),
            CipherError::Serialize(s) => write!(f, "serialize failed: {s}"),
            CipherError::Decrypt(e) => write!(f, "decrypt failed: {e}"),
            CipherError::Utf8(e) => write!(f, "utf8 failed: {e}"),
        }
    }
}

impl From<chacha20poly1305::aead::Error> for CipherError {
    fn from(e: chacha20poly1305::aead::Error) -> Self {
        CipherError::Decrypt(e)
    }
}

impl From<FromUtf8Error> for CipherError {
    fn from(e: FromUtf8Error) -> Self {
        CipherError::Utf8(e)
    }
}

pub trait Encrypted: Serialized {
    fn encrypt(&self, password: &str) -> EncryptedData {
        EncryptedData::encrypt(password, self.dumps().as_bytes())
    }

    fn decrypt(password: &str, packet: &EncryptedData) -> Result<Self, Self::Error>
    where
        Self: Sized,
        Self::Error: From<CipherError>,
    {
        let s: String = packet.decrypt(password).map_err(Into::into)?;
        Self::parse(&s)
    }
}

/// Derive a 32-byte key from a password and salt using Argon2.
pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key: [u8; 32] = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .expect("Argon2 key derivation failed");
    key
}

/// Generate array of random bytes
#[macro_export]
macro_rules! rand_arr {
    ($size:expr) => {{
        let mut arr = [0u8; $size];
        rand::fill(&mut arr[..]);
        arr
    }};
}

impl EncryptedData {
    pub fn decrypt(&self, password: &str) -> Result<String, CipherError> {
        let dek: [u8; 32] = derive_key(password, &self.salt[..]);
        let ciphertext: Vec<u8> = BASE64_STANDARD
            .decode(self.txt.as_str())
            .map_err(|e| CipherError::Parse(format!("ciphertext base64: {e}")))?;
        let cipher: ChaCha20Poly1305 = ChaCha20Poly1305::new(Key::from_slice(&dek));
        let data: Vec<u8> =
            cipher.decrypt(Nonce::from_slice(&self.nonce), ciphertext.as_slice())?;
        let s: String = String::from_utf8(data)?;
        Ok(s)
    }

    pub fn encrypt(password: &str, plaintxt: &[u8]) -> Self {
        let salt = rand_arr!(16);
        let nonce = rand_arr!(12);

        let key = derive_key(password, &salt);

        let cipher: ChaCha20Poly1305 = ChaChaPoly1305::new(Key::from_slice(&key));
        let ciphertxt = cipher
            .encrypt(Nonce::from_slice(&nonce), plaintxt)
            .expect("encrypt");

        Self {
            salt,
            nonce,
            txt: BASE64_STANDARD.encode(&ciphertxt[..]),
        }
    }
}

const DELIM: char = '|';

impl Serialized for EncryptedData {
    type Error = CipherError;

    fn dumps(&self) -> String {
        let nonce_b64: String = BASE64_STANDARD.encode(self.nonce.as_slice());
        let salt_b64: String = BASE64_STANDARD.encode(self.salt.as_slice());
        format!("{nonce_b64}{DELIM}{salt_b64}{DELIM}{}", self.txt)
    }

    fn parse(data: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let parts: Vec<&str> = data.splitn(3, DELIM).collect();
        if parts.len() != 3 {
            return Err(CipherError::Parse(format!(
                "expected 3 fields [nonce|salt|txt], got {}",
                parts.len()
            )));
        }
        let nonce_dec: Vec<u8> = BASE64_STANDARD
            .decode(parts[0])
            .map_err(|e| CipherError::Parse(format!("nonce base64: {e}")))?;
        if nonce_dec.len() != 12 {
            return Err(CipherError::Parse(format!(
                "nonce must be 12 bytes, got {}",
                nonce_dec.len()
            )));
        }
        let salt_dec: Vec<u8> = BASE64_STANDARD
            .decode(parts[1])
            .map_err(|e| CipherError::Parse(format!("salt base64: {e}")))?;
        if salt_dec.len() != 16 {
            return Err(CipherError::Parse(format!(
                "salt must be 16 bytes, got {}",
                salt_dec.len()
            )));
        }
        let mut nonce: [u8; 12] = [0u8; 12];
        nonce.copy_from_slice(&nonce_dec[..12]);
        let mut salt: [u8; 16] = [0u8; 16];
        salt.copy_from_slice(&salt_dec[..16]);
        let txt: String = parts[2].to_string();
        Ok(Self { salt, nonce, txt })
    }
}

#[cfg(test)]
mod tests {
    use super::{EncryptedData, Serialized};

    #[test]
    fn encrypt_decrypt_round_trip() {
        let password: &str = "secret123";
        let plaintext: &[u8] = b"hello world";
        let enc: EncryptedData = EncryptedData::encrypt(password, plaintext);
        let dec: String = enc.decrypt(password).expect("decrypt should succeed");
        assert_eq!(dec.as_bytes(), plaintext);
    }

    #[test]
    fn encrypt_decrypt_wrong_password() {
        let password: &str = "correct_password";
        let plaintext: &[u8] = b"sensitive data";
        let enc: EncryptedData = EncryptedData::encrypt(password, plaintext);
        let result: Result<String, _> = enc.decrypt("wrong_password");
        assert!(result.is_err(), "decrypt with wrong password must fail");
    }

    #[test]
    fn serialize_deserialize_encrypted_data() {
        let password: &str = "pass";
        let plaintext: &[u8] = b"payload";
        let enc: EncryptedData = EncryptedData::encrypt(password, plaintext);
        let s: String = enc.dumps();
        // 6t+jckTmH84oApFo|R1OOJZBJeL9goUBYGDPAHw==|E8jy7ErWXAMjpa5RtO+I2lGCFmyt/cM=
        println!("{s}");
        let enc2: EncryptedData = EncryptedData::parse(&s).expect("parse should succeed");
        let dec1: String = enc.decrypt(password).expect("decrypt 1");
        let dec2: String = enc2.decrypt(password).expect("decrypt 2");
        assert_eq!(dec1, dec2);
        assert_eq!(dec1.as_bytes(), plaintext);
    }
    #[test]
    fn deserialize_bad_data_fails() {
        let bad_input = "thisisnot:valid";
        let result = super::EncryptedData::parse(bad_input);
        assert!(result.is_err(), "should fail to parse bad input");
    }
}
