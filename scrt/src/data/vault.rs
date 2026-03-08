use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use std::collections::HashMap;

use crate::auth::cipher::{Encrypted, EncryptedData, Serialized};
use crate::data::secret::SecretEntry;
use crate::data::utils::{DataError, FIELD_SEP, IdType};
use crate::rand_arr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataEntry {
    pub name: String,
    pub info: Option<String>,
    pub key: [u8; 12],
    pub group: Option<String>,
    pub tags: Vec<String>,
    pub id: IdType,
}

#[derive(Default)]
pub struct Vault {
    pub metadata: Vec<MetadataEntry>,
    secrets: HashMap<IdType, EncryptedData>,
}

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
            "{}{FIELD_SEP}{}{FIELD_SEP}{}{FIELD_SEP}{}{FIELD_SEP}{}{FIELD_SEP}{}",
            BASE64_STANDARD.encode(self.name.as_bytes()),
            info_b64,
            BASE64_STANDARD.encode(self.key.as_slice()),
            group_b64,
            tags_b64,
            self.id
        )
    }

    fn parse(data: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = data.splitn(6, FIELD_SEP).collect();
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

impl Vault {
    pub fn get_secret(&self, id: IdType, key: &str) -> Option<SecretEntry> {
        let data = self.secrets.get(&id)?;
        SecretEntry::decrypt(key, data).ok()
    }
}

#[cfg(test)]
mod tests {

    use super::{Encrypted, EncryptedData, MetadataEntry, SecretEntry, Serialized, Vault};
    use crate::data::secret::{Secret, SecretType};
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
