use crate::auth::cipher::{CipherError, EncryptedData, Serialized};

#[derive(Debug)]
pub enum PassError {
    MasterKeyNotFound,
    Cipher(CipherError),
}

impl std::fmt::Display for PassError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PassError::MasterKeyNotFound => write!(f, "master key not found"),
            PassError::Cipher(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for PassError {}

impl From<CipherError> for PassError {
    fn from(e: CipherError) -> Self {
        PassError::Cipher(e)
    }
}

#[derive(Default)]
pub struct PassRing {
    hashes: Vec<EncryptedData>,
}

fn decrypt_key(password: &str, secret: &str) -> String {
    format!("{password}{secret}")
}

impl PassRing {
    /// Try each entry with (password + secret) as decrypt key; return master key on first success.
    pub fn get_master_key(&self, password: &str, secret: &str) -> Result<String, PassError> {
        let key: String = decrypt_key(password, secret);
        for enc in &self.hashes {
            if let Ok(master_key) = enc.decrypt(&key) {
                return Ok(master_key);
            }
        }
        Err(PassError::MasterKeyNotFound)
    }

    /// Encrypt master_key with (password + secret) and append to hashes.
    pub fn add_password(&mut self, master_key: &str, password: &str, secret: &str) {
        let key: String = decrypt_key(password, secret);
        self.hashes
            .push(EncryptedData::encrypt(&key, master_key.as_bytes()));
    }
}

const DELIM: char = ';';

impl Serialized for PassRing {
    type Error = CipherError;

    fn dumps(&self) -> String {
        self.hashes
            .iter()
            .map(|h| h.dumps())
            .collect::<Vec<_>>()
            .join(&DELIM.to_string())
    }

    fn parse(data: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if data.trim().is_empty() {
            return Err(CipherError::Parse(
                "PassRing parse: input is empty".to_string(),
            ));
        }
        let hashes: Result<Vec<EncryptedData>, CipherError> =
            data.split(DELIM).map(EncryptedData::parse).collect();
        Ok(PassRing { hashes: hashes? })
    }
}

#[cfg(test)]
mod tests {
    use super::{PassError, PassRing};
    use crate::auth::cipher::Serialized;

    #[test]
    fn get_master_key_ok_with_correct_password_and_secret() {
        let master_key: &str = "the_master_key";
        let password: &str = "my_password";
        let key1: &str = "key_a";
        let key2: &str = "key_b";

        let mut ring: PassRing = PassRing::default();
        ring.add_password(master_key, password, key1);
        ring.add_password(master_key, password, key2);

        let got: String = ring
            .get_master_key(password, key1)
            .expect("get_master_key with key1 must succeed");
        assert_eq!(got, master_key);

        let got2: String = ring
            .get_master_key(password, key2)
            .expect("get_master_key with key2 must succeed");
        assert_eq!(got2, master_key);

        let err: Result<String, PassError> = ring.get_master_key(password, "key_x");
        assert!(err.is_err());
        assert!(matches!(err.unwrap_err(), PassError::MasterKeyNotFound));
    }

    #[test]
    fn get_master_key_wrong_password_or_secret() {
        let master_key: &str = "secret_master";
        let mut ring: PassRing = PassRing::default();
        ring.add_password(master_key, "correct_pass", "correct_key");

        assert!(ring.get_master_key("correct_pass", "correct_key").is_ok());
        assert!(matches!(
            ring.get_master_key("wrong_pass", "correct_key")
                .unwrap_err(),
            PassError::MasterKeyNotFound
        ));
        assert!(matches!(
            ring.get_master_key("correct_pass", "wrong_key")
                .unwrap_err(),
            PassError::MasterKeyNotFound
        ));
    }

    #[test]
    fn passring_serialization() {
        let master_key: &str = "super_master";
        let password: &str = "super_secret_password";
        let keys: [&str; 3] = ["key_alpha", "key_beta", "key_gamma"];

        let mut ring: PassRing = PassRing::default();
        for key in &keys {
            ring.add_password(master_key, password, key);
        }

        let serialized: String = ring.dumps();
        let parsed: PassRing = PassRing::parse(&serialized).expect("parse should succeed");

        for key in &keys {
            let got: String = parsed
                .get_master_key(password, key)
                .expect("get_master_key after round-trip");
            assert_eq!(got, master_key);
        }

        assert!(parsed.get_master_key(password, "key_delta").is_err());
    }
}
