use std::collections::HashMap;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;

use crate::auth::cipher::{CipherError, Encrypted, EncryptedData, Serialized};
use crate::auth::pass::{PassError, PassRing};
use crate::auth::twofa::TwoFAError;
use crate::auth::twofa::{TwoFAData, TwoFAMethod};
use crate::rand_arr;

type IdType = u32;

const FIELD: char = '|';
const CREDS_SEP: &str = "%";
const STORE_SEP: &str = "&";
const SECRET_SEP: &str = "#";

#[derive(Debug)]
pub enum DataError {
    Parse(String),
    ChecksumMismatch,
    Cipher(CipherError),
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataError::Parse(s) => write!(f, "parse failed: {s}"),
            DataError::ChecksumMismatch => write!(f, "checksum mismatch"),
            DataError::Cipher(e) => write!(f, "{e}"),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataEntry {
    pub name: String,
    pub info: Option<String>,
    pub key: [u8; 12],
    pub group: Option<String>,
    pub tags: Vec<String>,
    pub id: IdType,
}

impl Serialized for MetadataEntry {
    type Error = DataError;

    fn dumps(&self) -> String {
        let info_b64: String = self
            .info
            .as_ref()
            .map(|s| BASE64_STANDARD.encode(s.as_bytes()))
            .unwrap_or_default();
        let group_b64: String = self
            .group
            .as_ref()
            .map(|s| BASE64_STANDARD.encode(s.as_bytes()))
            .unwrap_or_default();
        let tags_b64: String = BASE64_STANDARD.encode(self.tags.join(",").as_bytes());
        format!(
            "{}{FIELD}{}{FIELD}{}{FIELD}{}{FIELD}{}{FIELD}{}",
            BASE64_STANDARD.encode(self.name.as_bytes()),
            info_b64,
            BASE64_STANDARD.encode(self.key.as_slice()),
            group_b64,
            tags_b64,
            self.id
        )
    }

    fn parse(data: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = data.splitn(6, FIELD).collect();
        if parts.len() != 6 {
            return Err(DataError::Parse(format!(
                "MetadataEntry: expected 6 fields, got {}",
                parts.len()
            )));
        }
        let name_bytes: Vec<u8> = BASE64_STANDARD
            .decode(parts[0])
            .map_err(|e| DataError::Parse(format!("name base64: {e}")))?;
        let name: String = String::from_utf8(name_bytes)
            .map_err(|e| DataError::Parse(format!("name utf8: {e}")))?;
        let info: Option<String> = if parts[1].is_empty() {
            None
        } else {
            let b: Vec<u8> = BASE64_STANDARD
                .decode(parts[1])
                .map_err(|e| DataError::Parse(format!("info base64: {e}")))?;
            Some(String::from_utf8(b).map_err(|e| DataError::Parse(format!("info utf8: {e}")))?)
        };
        let key_bytes: Vec<u8> = BASE64_STANDARD
            .decode(parts[2])
            .map_err(|e| DataError::Parse(format!("key base64: {e}")))?;
        if key_bytes.len() != 12 {
            return Err(DataError::Parse(format!(
                "key must be 12 bytes, got {}",
                key_bytes.len()
            )));
        }
        let mut key: [u8; 12] = [0u8; 12];
        key.copy_from_slice(&key_bytes[..12]);
        let group: Option<String> = if parts[3].is_empty() {
            None
        } else {
            let b: Vec<u8> = BASE64_STANDARD
                .decode(parts[3])
                .map_err(|e| DataError::Parse(format!("group base64: {e}")))?;
            Some(String::from_utf8(b).map_err(|e| DataError::Parse(format!("group utf8: {e}")))?)
        };
        let tags_bytes: Vec<u8> = BASE64_STANDARD
            .decode(parts[4])
            .map_err(|e| DataError::Parse(format!("tags base64: {e}")))?;
        let tags_str: String = String::from_utf8(tags_bytes)
            .map_err(|e| DataError::Parse(format!("tags utf8: {e}")))?;
        let tags: Vec<String> = if tags_str.is_empty() {
            Vec::new()
        } else {
            tags_str.split(',').map(String::from).collect()
        };
        let id: IdType = parts[5]
            .parse()
            .map_err(|_| DataError::Parse("id must be u32".to_string()))?;
        Ok(Self {
            name,
            info,
            key,
            group,
            tags,
            id,
        })
    }
}

const STYPE_TAG_HIDDEN: &str = "Hidden";
const STYPE_TAG_PARTIAL: &str = "Partial";
const STYPE_TAG_VIEW: &str = "View";
const STYPE_TAG_PASS_REQUIRED: &str = "PassRequired";

#[derive(Clone, Debug)]
pub enum SecretType {
    Hidden(String),
    Partial(String),
    View(String),
    PassRequired(EncryptedData),
}

impl PartialEq for SecretType {
    fn eq(&self, other: &Self) -> bool {
        self.dumps() == other.dumps()
    }
}

impl Eq for SecretType {}

impl Serialized for SecretType {
    type Error = DataError;

    fn dumps(&self) -> String {
        match self {
            SecretType::Hidden(s) => {
                format!(
                    "{STYPE_TAG_HIDDEN}{FIELD}{}",
                    BASE64_STANDARD.encode(s.as_bytes())
                )
            }
            SecretType::Partial(s) => {
                format!(
                    "{STYPE_TAG_PARTIAL}{FIELD}{}",
                    BASE64_STANDARD.encode(s.as_bytes())
                )
            }
            SecretType::View(s) => {
                format!(
                    "{STYPE_TAG_VIEW}{FIELD}{}",
                    BASE64_STANDARD.encode(s.as_bytes())
                )
            }
            SecretType::PassRequired(enc) => {
                format!("{STYPE_TAG_PASS_REQUIRED}{FIELD}{}", enc.dumps())
            }
        }
    }

    fn parse(data: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = data.splitn(2, FIELD).collect();
        if parts.len() != 2 {
            return Err(DataError::Parse(format!(
                "SecretType: expected tag{}value, got {} part(s)",
                FIELD,
                parts.len()
            )));
        }
        let tag: &str = parts[0];
        let value: &str = parts[1];
        match tag {
            STYPE_TAG_HIDDEN => {
                let s: String = String::from_utf8(
                    BASE64_STANDARD
                        .decode(value)
                        .map_err(|e| DataError::Parse(format!("Hidden value base64: {e}")))?,
                )
                .map_err(|e| DataError::Parse(format!("Hidden value utf8: {e}")))?;
                Ok(SecretType::Hidden(s))
            }
            STYPE_TAG_PARTIAL => {
                let s: String = String::from_utf8(
                    BASE64_STANDARD
                        .decode(value)
                        .map_err(|e| DataError::Parse(format!("Partial value base64: {e}")))?,
                )
                .map_err(|e| DataError::Parse(format!("Partial value utf8: {e}")))?;
                Ok(SecretType::Partial(s))
            }
            STYPE_TAG_VIEW => {
                let s: String = String::from_utf8(
                    BASE64_STANDARD
                        .decode(value)
                        .map_err(|e| DataError::Parse(format!("View value base64: {e}")))?,
                )
                .map_err(|e| DataError::Parse(format!("View value utf8: {e}")))?;
                Ok(SecretType::View(s))
            }
            STYPE_TAG_PASS_REQUIRED => {
                let enc: EncryptedData = EncryptedData::parse(value).map_err(DataError::Cipher)?;
                Ok(SecretType::PassRequired(enc))
            }
            _ => Err(DataError::Parse(format!("unknown SecretType tag: {tag}"))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Secret {
    pub name: String,
    pub secret: SecretType,
}

impl Secret {
    // For `preview_secret`, return a string:
    // - For Partial: show start and end (e.g., "abc...xyz" for "abcxyz")
    // - For Hidden: "***"
    // - For View: return as is
    // - For PassRequired: "**pass required**"
    pub fn preview_secret(&self) -> String {
        match &self.secret {
            SecretType::Partial(s) => {
                let len = s.len();
                if len <= 6 {
                    s.clone()
                } else {
                    let start = &s[..3.min(len)];
                    let end = &s[len - 3.min(len)..];
                    format!("{start}...{end}")
                }
            }
            SecretType::Hidden(_) => "***".to_string(),
            SecretType::View(s) => s.clone(),
            SecretType::PassRequired(_) => "**pass required**".to_string(),
        }
    }

    // For `get_data`:
    // - Partial/Hidden/View: return the string value
    // - PassRequired: use password to decrypt
    pub fn get_data(&self, password: Option<&str>) -> Result<String, DataError> {
        match &self.secret {
            SecretType::Partial(s) | SecretType::Hidden(s) | SecretType::View(s) => Ok(s.clone()),
            SecretType::PassRequired(enc) => {
                let pw = password
                    .ok_or_else(|| DataError::Parse("Password required for secret".to_string()))?;
                let decrypted = enc.decrypt(pw)?;
                Ok(decrypted)
            }
        }
    }
}

impl Serialized for Secret {
    type Error = DataError;

    fn dumps(&self) -> String {
        format!(
            "{}{FIELD}{}",
            BASE64_STANDARD.encode(self.name.as_bytes()),
            self.secret.dumps()
        )
    }

    fn parse(data: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = data.splitn(2, FIELD).collect();
        if parts.len() != 2 {
            return Err(DataError::Parse(format!(
                "Secret: expected name{}value, got {}",
                FIELD,
                parts.len()
            )));
        }
        let name: String = String::from_utf8(
            BASE64_STANDARD
                .decode(parts[0])
                .map_err(|e| DataError::Parse(format!("name base64: {e}")))?,
        )
        .map_err(|e| DataError::Parse(format!("name utf8: {e}")))?;
        let secret: SecretType = SecretType::parse(parts[1])?;
        Ok(Self { name, secret })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretEntry {
    pub secrets: Vec<Secret>,
    pub id: IdType,
}

impl Serialized for SecretEntry {
    type Error = DataError;

    fn dumps(&self) -> String {
        let parts: Vec<String> = self.secrets.iter().map(Serialized::dumps).collect();
        format!("{}{SECRET_SEP}{}", parts.join(SECRET_SEP), self.id)
    }

    fn parse(data: &str) -> Result<Self, Self::Error> {
        let v: Vec<&str> = data.split(SECRET_SEP).collect();
        if v.is_empty() {
            return Err(DataError::Parse("SecretEntry: empty".to_string()));
        }
        let last: &str = v[v.len() - 1];
        let id: IdType = last
            .parse()
            .map_err(|_| DataError::Parse("SecretEntry id must be u32".to_string()))?;
        let secret_strs: &[&str] = &v[..v.len() - 1];
        let secrets: Vec<Secret> = secret_strs
            .iter()
            .map(|s| Secret::parse(s))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { secrets, id })
    }
}

impl Encrypted for SecretEntry {}

pub struct Credentials {
    keys: PassRing,
    v2fa: TwoFAData,
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

pub struct ScrtStore {
    pub checksum: String,
    pub creds: Credentials,
    pub data: EncryptedData,
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

pub struct Vault {
    pub metadata: Vec<MetadataEntry>,
    secrets: HashMap<IdType, EncryptedData>,
}

impl Default for Vault {
    fn default() -> Self {
        Vault {
            metadata: vec![],
            secrets: HashMap::new(),
        }
    }
}

impl Serialized for Vault {
    type Error = DataError;

    fn dumps(&self) -> String {
        let mut out: Vec<String> = Vec::new();
        for m in &self.metadata {
            out.push(m.dumps());
        }
        out.push("--secrets--".to_string());
        for (id, enc) in &self.secrets {
            out.push(format!("{}:{}", id, enc.dumps()));
        }
        out.join("\n")
    }

    fn parse(data: &str) -> Result<Self, Self::Error> {
        let lines: Vec<&str> = data.lines().collect();
        let sep_idx: usize = lines
            .iter()
            .position(|&l| l == "--secrets--")
            .ok_or(DataError::Parse("Vault: missing --secrets--".to_string()))?;
        let metadata: Vec<MetadataEntry> = lines[..sep_idx]
            .iter()
            .map(|l| MetadataEntry::parse(l))
            .collect::<Result<Vec<_>, _>>()?;
        let mut secrets: HashMap<IdType, EncryptedData> = HashMap::new();
        for line in &lines[sep_idx + 1..] {
            let line: &str = line.trim();
            if line.is_empty() {
                continue;
            }
            let colon: usize = line.find(':').ok_or(DataError::Parse(
                "Vault secret line: expected id:data".to_string(),
            ))?;
            let id: IdType = line[..colon]
                .parse()
                .map_err(|_| DataError::Parse("Vault secret id must be u32".to_string()))?;
            let enc: EncryptedData =
                EncryptedData::parse(&line[colon + 1..]).map_err(DataError::Cipher)?;
            secrets.insert(id, enc);
        }
        Ok(Self { metadata, secrets })
    }
}

impl Encrypted for Vault {}

impl MetadataEntry {
    pub fn new(
        name: String,
        group: Option<String>,
        info: Option<String>,
        tags: Vec<String>,
        id: IdType,
    ) -> Self {
        MetadataEntry {
            name,
            info,
            key: rand_arr!(12),
            group,
            tags,
            id,
        }
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

impl Vault {
    pub fn get_secret(&self, id: IdType, key: &str) -> Option<SecretEntry> {
        let data = self.secrets.get(&id)?;
        SecretEntry::decrypt(key, data).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Credentials, DataError, MetadataEntry, ScrtStore, Secret, SecretEntry, SecretType, Vault,
    };
    use crate::auth::cipher::{Encrypted, EncryptedData, Serialized};
    use crate::auth::twofa::TwoFAMethod;

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
    fn metadata_entry_serialize_deserialize() {
        let e: MetadataEntry = MetadataEntry {
            name: "entry1".to_string(),
            info: Some("info".to_string()),
            key: [1u8; 12],
            group: Some("g1".to_string()),
            tags: vec!["a".to_string(), "b".to_string()],
            id: 42,
        };
        let s: String = e.dumps();
        let parsed: MetadataEntry = MetadataEntry::parse(&s).expect("parse");
        assert_eq!(e, parsed);
    }

    #[test]
    fn secret_serialize_deserialize() {
        let s: Secret = Secret {
            name: "n".to_string(),
            secret: SecretType::Hidden("val".to_string()),
        };
        let out: String = s.dumps();
        let parsed: Secret = Secret::parse(&out).expect("parse");
        assert_eq!(s, parsed);
    }

    #[test]
    fn secret_entry_serialize_deserialize() {
        let e: SecretEntry = SecretEntry {
            secrets: vec![
                Secret {
                    name: "a".to_string(),
                    secret: SecretType::View("x".to_string()),
                },
                Secret {
                    name: "b".to_string(),
                    secret: SecretType::PassRequired(EncryptedData::encrypt("key", b"y")),
                },
            ],
            id: 1,
        };
        let out: String = e.dumps();
        let parsed: SecretEntry = SecretEntry::parse(&out).expect("parse");
        assert_eq!(e, parsed);
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

    #[test]
    fn vault_get_secret_pass_required_and_decrypt_with_password() {
        let vault_key: &str = "vault_key";
        let pass_password: &str = "mypass";
        let secret_payload: &[u8] = b"secret_data";

        let enc_for_pass: EncryptedData = EncryptedData::encrypt(pass_password, secret_payload);
        let secret_with_pass: Secret = Secret {
            name: "mysecret".to_string(),
            secret: SecretType::PassRequired(enc_for_pass),
        };
        let secret_entry: SecretEntry = SecretEntry {
            secrets: vec![secret_with_pass],
            id: 1,
        };
        let enc_entry: EncryptedData = secret_entry.encrypt(vault_key);

        let mut secrets_map: std::collections::HashMap<super::IdType, EncryptedData> =
            std::collections::HashMap::new();
        secrets_map.insert(1, enc_entry);
        let vault = Vault {
            metadata: vec![],
            secrets: secrets_map,
        };
        let decrypted_entry: SecretEntry = vault
            .get_secret(1, vault_key)
            .expect("get_secret should return Some");
        assert_eq!(decrypted_entry.secrets.len(), 1);
        let first: &Secret = &decrypted_entry.secrets[0];
        match &first.secret {
            SecretType::PassRequired(enc) => {
                let data: String = enc.decrypt(pass_password).expect("decrypt with password");
                assert_eq!(data.as_bytes(), secret_payload);
            }
            _ => panic!("expected PassRequired"),
        }
    }
}
