use crate::auth::cipher::{Encrypted, EncryptedData, Serialized};
use crate::auth::pass::PassRing;
use crate::auth::twofa::{TwoFAData, TwoFAMethod};
use crate::data::utils::{CREDS_SEP, DataError, STORE_SEP};
use crate::data::vault::Vault;
use crate::rand_arr;

pub struct Credentials {
    keys: PassRing,
    v2fa: TwoFAData,
}

pub struct ScrtStore {
    pub checksum: String,
    pub creds: Credentials,
    pub data: EncryptedData,
}

fn random_master_key_hex() -> String {
    let bytes: [u8; 32] = rand_arr!(32);
    bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
}

impl Credentials {
    /// Build credentials and master key from password and pass phrase (2FA). Private.
    fn new(password: &str, pass_phrase: &str) -> (Self, String) {
        let master_key: String = random_master_key_hex();
        let mut two_fa: TwoFAData = TwoFAData::default();
        let secret_key: String = two_fa.add_method(TwoFAMethod::PassPhrase, pass_phrase);
        let mut pass_ring: PassRing = PassRing::default();
        pass_ring.add_password(&master_key, password, &secret_key);
        let creds: Credentials = Credentials {
            keys: pass_ring,
            v2fa: two_fa,
        };
        (creds, master_key)
    }
}

impl Serialized for Credentials {
    type Error = DataError;

    fn dumps(&self) -> String {
        format!("{}{CREDS_SEP}{}", self.keys.dumps(), self.v2fa.dumps())
    }

    fn parse(data: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = data.splitn(2, CREDS_SEP).collect();
        if parts.len() != 2 {
            return Err(DataError::Parse(
                "Credentials: expected keys%v2fa".to_string(),
            ));
        }
        let keys: PassRing = PassRing::parse(parts[0]).map_err(DataError::Cipher)?;
        let v2fa: TwoFAData =
            TwoFAData::parse(parts[1]).map_err(|e| DataError::Parse(format!("v2fa: {e}")))?;
        Ok(Self { keys, v2fa })
    }
}

impl Serialized for ScrtStore {
    type Error = DataError;

    fn dumps(&self) -> String {
        let payload: String = format!("{}{STORE_SEP}{}", self.creds.dumps(), self.data.dumps());
        let hash: blake3::Hash = blake3::hash(payload.as_bytes());
        let checksum: String = hash.to_hex().to_string();
        format!("{checksum}{STORE_SEP}{payload}")
    }

    fn parse(data: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = data.splitn(2, STORE_SEP).collect();
        if parts.len() != 2 {
            return Err(DataError::Parse(
                "ScrtStore: expected checksum&payload".to_string(),
            ));
        }
        let checksum_hex: &str = parts[0];
        let payload: &str = parts[1];
        let computed: blake3::Hash = blake3::hash(payload.as_bytes());
        if computed.to_hex().as_str() != checksum_hex {
            return Err(DataError::ChecksumMismatch);
        }
        let inner: Vec<&str> = payload.splitn(2, STORE_SEP).collect();
        if inner.len() != 2 {
            return Err(DataError::Parse(
                "ScrtStore payload: expected creds&data".to_string(),
            ));
        }
        let creds: Credentials = Credentials::parse(inner[0])?;
        let data: EncryptedData = EncryptedData::parse(inner[1]).map_err(DataError::Cipher)?;
        Ok(Self {
            checksum: checksum_hex.to_string(),
            creds,
            data,
        })
    }
}

impl ScrtStore {
    /// Create a new store with empty vault, keyed by password and pass phrase (2FA).
    pub fn new(password: String, pass_phrase: String) -> Self {
        let (creds, master_key): (Credentials, String) = Credentials::new(&password, &pass_phrase);
        let vault: Vault = Vault::default();
        let data: EncryptedData = EncryptedData::encrypt(&master_key, vault.dumps().as_bytes());
        ScrtStore {
            checksum: String::from("random bulshit go"),
            creds,
            data,
        }
    }

    pub fn unlock(
        &self,
        password: String,
        method: TwoFAMethod,
        verification_data: String,
    ) -> Result<Vault, DataError> {
        let secret: String = self.creds.v2fa.get_key(method, &verification_data)?;
        let master_key: String = self.creds.keys.get_master_key(&password, &secret)?;
        Vault::decrypt(&master_key, &self.data)
    }
}

#[cfg(test)]
mod tests {

    use super::{Credentials, DataError, ScrtStore, Serialized, TwoFAMethod, Vault};

    fn two_fa_sample() -> crate::auth::twofa::TwoFAData {
        use base64::Engine;
        use base64::prelude::BASE64_STANDARD;
        let s: String = format!(
            "{}:{}",
            TwoFAMethod::PassPhrase.key(),
            BASE64_STANDARD.encode([0u8; 32].as_slice())
        );
        crate::auth::twofa::TwoFAData::parse(&s).expect("parse twofa")
    }

    #[test]
    fn credentials_serialize_deserialize() {
        let mut ring: crate::auth::pass::PassRing = crate::auth::pass::PassRing::default();
        ring.add_password("master", "pass", "key1");
        let creds: Credentials = Credentials {
            keys: ring,
            v2fa: two_fa_sample(),
        };
        let out: String = creds.dumps();
        let parsed: Credentials = Credentials::parse(&out).expect("parse");
        let out2: String = parsed.dumps();
        assert_eq!(out, out2);
    }

    #[test]
    fn scrt_store_serialize_deserialize() {
        let store: ScrtStore = ScrtStore::new("p".to_string(), "pp".to_string());
        let out: String = store.dumps();
        let parsed: ScrtStore = ScrtStore::parse(&out).expect("parse");
        let out2: String = parsed.dumps();
        assert_eq!(out, out2, "round-trip must yield same serialized form");
    }

    #[test]
    fn scrt_store_bad_checksum_fails() {
        let store: ScrtStore = ScrtStore::new("p".to_string(), "pp".to_string());
        let out: String = store.dumps();
        let bad: String = format!(
            "wrong_checksum_hex{}{}",
            super::STORE_SEP,
            out.split_once(super::STORE_SEP).unwrap().1
        );
        let result: Result<ScrtStore, DataError> = ScrtStore::parse(&bad);
        assert!(matches!(result, Err(DataError::ChecksumMismatch)));
    }

    #[test]
    fn scrt_store_unlock() {
        let password: String = "user_password".to_string();
        let pass_phrase: String = "verif123".to_string();
        let store: ScrtStore = ScrtStore::new(password.clone(), pass_phrase.clone());
        let unlocked: Vault = store
            .unlock(password, TwoFAMethod::PassPhrase, pass_phrase)
            .expect("unlock");
        assert!(unlocked.metadata.is_empty());
    }
}
