use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use std::collections::HashMap;

use crate::auth::cipher::{Encrypted, EncryptedData, Serialized};
use crate::data::secret::{Secret, SecretEntry};
use crate::data::utils::{DataError, FIELD_SEP, IdType};
use crate::{rand_arr, rand_key_hex};

const PUBLIC_SALT_KEY_SIZE: usize = 12;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataEntry {
    pub name: String,
    pub info: Option<String>,
    pub key: String,
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
            key: rand_key_hex!(PUBLIC_SALT_KEY_SIZE),
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
            BASE64_STANDARD.encode(self.key.as_bytes()),
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
        let key: String =
            String::from_utf8(key_bytes).map_err(|e| DataError::Parse(format!("key utf8: {e}")))?;
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

fn decryption_key(master_key: &str, pub_key: &str) -> String {
    format!("{master_key}{pub_key}")
}

impl Vault {
    pub fn secret_count(&self) -> usize {
        self.secrets.len()
    }

    pub fn get_secret(&self, id: IdType, master_key: &str) -> Option<SecretEntry> {
        let mentry = self.metadata.iter().find(|x| x.id == id)?;

        let data = self.secrets.get(&id)?;
        SecretEntry::decrypt(&decryption_key(master_key, &mentry.key), data).ok()
    }

    pub fn add_entry(
        &mut self,
        name: String,
        info: Option<String>,
        group: Option<String>,
        tags: Vec<String>,
        secrets: Vec<Secret>,
        master_key: &str,
    ) {
        let max_key = self.secrets.keys().max().unwrap_or(&0);
        let id: IdType = *max_key + 1;

        let metadata = MetadataEntry::new(name, group, info, tags, id);
        let secret_entry = SecretEntry { secrets, id };

        self.secrets.insert(
            id,
            secret_entry.encrypt(&decryption_key(master_key, &metadata.key)),
        );
        self.metadata.push(metadata);
    }

    pub fn remove_entry(&mut self, id: IdType) -> Result<(), String> {
        let meta_index = self.metadata.iter().position(|entry| entry.id == id);
        if meta_index.is_none() && !self.secrets.contains_key(&id) {
            return Err(format!("No entry with id {id}"));
        }

        if let Some(idx) = meta_index {
            self.metadata.remove(idx);
        }
        self.secrets.remove(&id);

        Ok(())
    }

    /// Update/Insert
    pub fn atomic_update(
        &mut self,
        id: Option<IdType>,
        name: String,
        info: Option<String>,
        group: Option<String>,
        tags: Vec<String>,
        secrets: Vec<Secret>,
        master_key: &str,
    ) {
        if let Some(id) = id {
            let _ = self.remove_entry(id);
        }
        self.add_entry(name, info, group, tags, secrets, master_key);
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
            key: "0123456789ab0123456789ab".to_string(),
            // the data will be converted to
            // base64 so it should work fine
            group: Some("#g1$%|".to_string()),
            tags: vec!["a".to_string(), "b".to_string()],
            id: 42,
        };
        let s: String = e.dumps();
        let parsed: MetadataEntry = MetadataEntry::parse(&s).expect("parse");
        assert_eq!(e, parsed);
    }

    #[test]
    fn vault_atomic_update_get_secret_and_counts() {
        let master_key: &str = "master_key";
        let pass_for_secret: &str = "pass1";

        let mut vault: Vault = Vault::default();
        assert_eq!(vault.metadata.len(), 0);
        assert_eq!(vault.secret_count(), 0);

        // Add first entry with None (new)
        vault.atomic_update(
            None,
            "entry1".to_string(),
            Some("info1".to_string()),
            Some("group1".to_string()),
            vec!["t1".to_string()],
            vec![Secret {
                name: "k1".to_string(),
                secret: SecretType::new_view("value1".to_string()),
            }],
            master_key,
        );
        assert_eq!(vault.metadata.len(), 1);
        assert_eq!(vault.metadata.len(), vault.secret_count());

        // Add second and third with None
        vault.atomic_update(
            None,
            "entry2".to_string(),
            None,
            None,
            vec![],
            vec![Secret {
                name: "k2".to_string(),
                secret: SecretType::new_hidden("hidden_val".to_string()),
            }],
            master_key,
        );
        vault.atomic_update(
            None,
            "entry3".to_string(),
            Some("info3".to_string()),
            None,
            vec!["a".to_string(), "b".to_string()],
            vec![Secret {
                name: "pw".to_string(),
                secret: SecretType::new_passrequired("secret_payload".to_string(), pass_for_secret),
            }],
            master_key,
        );
        assert_eq!(vault.metadata.len(), 3);
        assert_eq!(vault.metadata.len(), vault.secret_count());

        // Get by id (ids are 1, 2, 3 from add_entry)
        let id1: super::IdType = 1;
        let entry1: SecretEntry = vault
            .get_secret(id1, master_key)
            .expect("get_secret(1) should return Some");
        assert_eq!(entry1.id, id1);
        assert_eq!(entry1.secrets.len(), 1);
        let s1: &Secret = &entry1.secrets[0];
        assert_eq!(s1.name, "k1");
        let data1: String = s1.get_data(None).expect("View secret needs no password");
        assert_eq!(data1, "value1");

        // Modify existing entry via atomic_update(Some(id), ...)
        let id2: super::IdType = 2;
        vault.atomic_update(
            Some(id2),
            "entry2_updated".to_string(),
            Some("info2_new".to_string()),
            Some("group2".to_string()),
            vec!["tag2".to_string()],
            vec![Secret {
                name: "k2".to_string(),
                secret: SecretType::new_partial("partial_visible".to_string()),
            }],
            master_key,
        );
        assert_eq!(vault.metadata.len(), 3);
        assert_eq!(vault.metadata.len(), vault.secret_count());

        // Old id 2 is gone; new entry got id 4
        assert!(vault.get_secret(id2, master_key).is_none());
        let id4: super::IdType = 4;
        let entry4: SecretEntry = vault
            .get_secret(id4, master_key)
            .expect("get_secret(4) should return Some");
        assert_eq!(entry4.secrets.len(), 1);
        let s4: &Secret = &entry4.secrets[0];
        assert_eq!(
            s4.get_data(None).expect("Partial needs no password"),
            "partial_visible"
        );
        let meta4: &MetadataEntry = vault
            .metadata
            .iter()
            .find(|m| m.id == id4)
            .expect("metadata 4");
        assert_eq!(meta4.name, "entry2_updated");
        assert_eq!(meta4.info.as_deref(), Some("info2_new"));
    }

    #[test]
    fn vault_get_secret_pass_required_and_decrypt_with_password() {
        let vault_key: &str = "vault_key";
        let pass_password: &str = "mypass";
        let secret_payload: &[u8] = b"secret_data";
        let entry_key: &str = "0123456789ab0123456789ab";
        let dec_key: String = format!("{vault_key}{entry_key}");

        let enc_for_pass: EncryptedData = EncryptedData::encrypt(pass_password, secret_payload);
        let secret_with_pass: Secret = Secret {
            name: "mysecret".to_string(),
            secret: SecretType::PassRequired(enc_for_pass),
        };
        let secret_entry: SecretEntry = SecretEntry {
            secrets: vec![secret_with_pass],
            id: 1,
        };
        let enc_entry: EncryptedData = secret_entry.encrypt(&dec_key);

        let mut secrets_map: std::collections::HashMap<super::IdType, EncryptedData> =
            std::collections::HashMap::new();
        secrets_map.insert(1, enc_entry);
        let meta1: MetadataEntry = MetadataEntry {
            name: "entry1".to_string(),
            info: None,
            key: entry_key.to_string(),
            group: None,
            tags: vec![],
            id: 1,
        };
        let vault = Vault {
            metadata: vec![meta1],
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
