use std::collections::HashMap;

use argon2::Argon2;

use base64::{Engine, prelude::BASE64_STANDARD};

const SALT_LEN: usize = 32;
const KEY_LEN: usize = 32;

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum TwoFAMethod {
    PassPhrase,
    BiometricKey,
    PassFile,
    PassKey,
}

#[derive(Debug)]
pub enum TwoFAError {
    MethodNotFound,
    Parse(String),
}

impl std::fmt::Display for TwoFAError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TwoFAError::MethodNotFound => write!(f, "method not found"),
            TwoFAError::Parse(s) => write!(f, "parse failed: {s}"),
        }
    }
}

impl std::error::Error for TwoFAError {}

impl TwoFAMethod {
    pub fn key(&self) -> &'static str {
        match self {
            TwoFAMethod::PassPhrase => "PassPhrase",
            TwoFAMethod::BiometricKey => "BiometricKey",
            TwoFAMethod::PassFile => "PassFile",
            TwoFAMethod::PassKey => "PassKey",
        }
    }

    pub fn from_key(s: &str) -> Option<Self> {
        match s {
            "PassPhrase" => Some(TwoFAMethod::PassPhrase),
            "BiometricKey" => Some(TwoFAMethod::BiometricKey),
            "PassFile" => Some(TwoFAMethod::PassFile),
            "PassKey" => Some(TwoFAMethod::PassKey),
            _ => None,
        }
    }

    /// KDF(verification_data, salt).
    pub fn get_key(verification_data: &str, salt: &[u8]) -> String {
        let mut key_out: [u8; KEY_LEN] = [0u8; KEY_LEN];
        Argon2::default()
            .hash_password_into(verification_data.as_bytes(), salt, &mut key_out)
            .expect("argon2 key derivation");
        key_out
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    }
}

pub struct TwoFAData {
    data: HashMap<TwoFAMethod, [u8; SALT_LEN]>,
}

const DELIM: char = ';';

impl TwoFAData {
    pub fn get_key(
        &self,
        method: TwoFAMethod,
        verification_data: &str,
    ) -> Result<String, TwoFAError> {
        let salt: &[u8; SALT_LEN] = self.data.get(&method).ok_or(TwoFAError::MethodNotFound)?;
        Ok(TwoFAMethod::get_key(verification_data, &salt[..]))
    }
}

impl crate::auth::cipher::Serialized for TwoFAData {
    type Error = TwoFAError;

    fn dumps(&self) -> String {
        let mut keys: Vec<&TwoFAMethod> = self.data.keys().collect();
        keys.sort_by(|a, b| a.key().cmp(b.key()));
        keys.iter()
            .map(|k| {
                let salt: &[u8; SALT_LEN] = self.data.get(*k).unwrap();
                let salt_b64: String = BASE64_STANDARD.encode(salt.as_slice());
                format!("{}:{}", k.key(), salt_b64)
            })
            .collect::<Vec<String>>()
            .join(&format!("{DELIM}"))
    }

    fn parse(data: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut data_map: HashMap<TwoFAMethod, [u8; SALT_LEN]> = HashMap::new();
        if data.is_empty() {
            return Ok(Self { data: data_map });
        }
        for part in data.split(DELIM) {
            let colon: Option<usize> = part.find(':');
            let (key_s, value): (&str, &str) = match colon {
                Some(i) => (&part[..i], &part[i + 1..]),
                None => {
                    return Err(TwoFAError::Parse("expected method:salt_b64".to_string()));
                }
            };
            let method: TwoFAMethod = TwoFAMethod::from_key(key_s)
                .ok_or_else(|| TwoFAError::Parse(format!("unknown method: {key_s}")))?;
            let salt_bytes: Vec<u8> = BASE64_STANDARD
                .decode(value)
                .map_err(|e| TwoFAError::Parse(format!("salt base64: {e}")))?;
            if salt_bytes.len() != SALT_LEN {
                return Err(TwoFAError::Parse(format!(
                    "salt must be {} bytes, got {}",
                    SALT_LEN,
                    salt_bytes.len()
                )));
            }
            let mut salt: [u8; SALT_LEN] = [0u8; SALT_LEN];
            salt.copy_from_slice(&salt_bytes[..SALT_LEN]);
            data_map.insert(method, salt);
        }
        Ok(Self { data: data_map })
    }
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;

    use super::{SALT_LEN, TwoFAData, TwoFAError, TwoFAMethod};
    use crate::auth::cipher::Serialized;

    fn make_two_fa_data(methods: &[TwoFAMethod]) -> TwoFAData {
        let mut parts: Vec<String> = methods
            .iter()
            .map(|method| {
                let salt: [u8; SALT_LEN] = crate::rand_arr!(SALT_LEN);
                let salt_b64: String = BASE64_STANDARD.encode(salt.as_slice());
                format!("{}:{}", method.key(), salt_b64)
            })
            .collect();
        parts.sort();
        let s: String = parts.join(";");
        TwoFAData::parse(&s).expect("parse test data")
    }

    #[test]
    fn get_key_same_inputs_same_key_different_method_different_key() {
        let verification: &str = "v1";
        let two_fa: TwoFAData = make_two_fa_data(&[TwoFAMethod::PassPhrase, TwoFAMethod::PassFile]);

        let k1: String = two_fa
            .get_key(TwoFAMethod::PassPhrase, verification)
            .expect("get_key PassPhrase");
        let k2: String = two_fa
            .get_key(TwoFAMethod::PassPhrase, verification)
            .expect("get_key PassPhrase again");
        assert_eq!(k1, k2, "same method/verification must yield same key");

        let k3: String = two_fa
            .get_key(TwoFAMethod::PassFile, verification)
            .expect("get_key PassFile");
        assert_ne!(k1, k3, "different method must yield different key");
    }

    #[test]
    fn get_key_same_args_same_key_different_verification_different_key() {
        let two_fa: TwoFAData = make_two_fa_data(&[TwoFAMethod::PassPhrase]);
        let verification: &str = "v1";

        let k1: String = two_fa
            .get_key(TwoFAMethod::PassPhrase, verification)
            .expect("get_key");
        let k2: String = two_fa
            .get_key(TwoFAMethod::PassPhrase, verification)
            .expect("get_key again");
        assert_eq!(k1, k2, "same arguments must return the same key");

        let k3: String = two_fa
            .get_key(TwoFAMethod::PassPhrase, "v2")
            .expect("get_key different verification");
        assert_ne!(
            k1, k3,
            "different verification_data must yield different key"
        );
    }

    #[test]
    fn get_key_method_not_found() {
        let two_fa: TwoFAData = make_two_fa_data(&[TwoFAMethod::PassPhrase]);
        let result: Result<String, TwoFAError> = two_fa.get_key(TwoFAMethod::BiometricKey, "v");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TwoFAError::MethodNotFound));
    }

    #[test]
    fn serialize_deserialize_round_trip() {
        let two_fa: TwoFAData = make_two_fa_data(&[TwoFAMethod::PassPhrase, TwoFAMethod::PassFile]);
        let s: String = two_fa.dumps();
        let restored: TwoFAData = TwoFAData::parse(&s).expect("parse should succeed");
        let verification: &str = "v";
        for method in [TwoFAMethod::PassPhrase, TwoFAMethod::PassFile] {
            let orig_key: String = two_fa
                .get_key(method.clone(), verification)
                .expect("get_key");
            let restored_key: String = restored
                .get_key(method, verification)
                .expect("get_key after round-trip");
            assert_eq!(orig_key, restored_key);
        }
    }
}
