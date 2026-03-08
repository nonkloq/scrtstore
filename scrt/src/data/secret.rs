use base64::Engine;
use base64::prelude::BASE64_STANDARD;

use crate::auth::cipher::{Encrypted, EncryptedData, Serialized};
use crate::data::utils::{DataError, FIELD_SEP, IdType, SECRET_SEP};

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Secret {
    pub name: String,
    pub secret: SecretType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretEntry {
    pub secrets: Vec<Secret>,
    pub id: IdType,
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
                    "{STYPE_TAG_HIDDEN}{FIELD_SEP}{}",
                    BASE64_STANDARD.encode(s.as_bytes())
                )
            }
            SecretType::Partial(s) => {
                format!(
                    "{STYPE_TAG_PARTIAL}{FIELD_SEP}{}",
                    BASE64_STANDARD.encode(s.as_bytes())
                )
            }
            SecretType::View(s) => {
                format!(
                    "{STYPE_TAG_VIEW}{FIELD_SEP}{}",
                    BASE64_STANDARD.encode(s.as_bytes())
                )
            }
            SecretType::PassRequired(enc) => {
                format!("{STYPE_TAG_PASS_REQUIRED}{FIELD_SEP}{}", enc.dumps())
            }
        }
    }

    fn parse(data: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = data.splitn(2, FIELD_SEP).collect();
        if parts.len() != 2 {
            return Err(DataError::Parse(format!(
                "SecretType: expected tag{}value, got {} part(s)",
                FIELD_SEP,
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

impl Secret {
    /// Get Preview of the content
    /// - For Hidden: "***"
    /// - For View: return as is
    /// - For Partial: show start and end (e.g., "abc...xyz" for "abcxyz")
    /// - For PassRequired: "**pass required**"
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

    /// Get the secret content
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
            "{}{FIELD_SEP}{}",
            BASE64_STANDARD.encode(self.name.as_bytes()),
            self.secret.dumps()
        )
    }

    fn parse(data: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = data.splitn(2, FIELD_SEP).collect();
        if parts.len() != 2 {
            return Err(DataError::Parse(format!(
                "Secret: expected name{}value, got {}",
                FIELD_SEP,
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

impl Serialized for SecretEntry {
    type Error = DataError;

    fn dumps(&self) -> String {
        let parts: Vec<String> = self.secrets.iter().map(Serialized::dumps).collect();
        format!(
            "{}{SECRET_SEP}{}",
            parts.join(&SECRET_SEP.to_string()),
            self.id
        )
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

impl SecretType {
    pub fn new_hidden(content: String) -> Self {
        Self::Hidden(content)
    }

    pub fn new_partial(content: String) -> Self {
        Self::Partial(content)
    }

    pub fn new_view(content: String) -> Self {
        Self::View(content)
    }

    pub fn new_passrequired(content: String, key: &str) -> Self {
        Self::PassRequired(EncryptedData::encrypt(key, &content.as_bytes()))
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            SecretType::Hidden(_) => STYPE_TAG_HIDDEN,
            SecretType::Partial(_) => STYPE_TAG_PARTIAL,
            SecretType::View(_) => STYPE_TAG_VIEW,
            SecretType::PassRequired(_) => STYPE_TAG_PASS_REQUIRED,
        }
    }

    pub fn new(content: String, name: &'static str, key: &str) -> Result<Self, DataError> {
        match name {
            STYPE_TAG_HIDDEN => Ok(Self::Hidden(content)),
            STYPE_TAG_PARTIAL => Ok(Self::Partial(content)),
            STYPE_TAG_VIEW => Ok(Self::View(content)),
            STYPE_TAG_PASS_REQUIRED => Ok(Self::PassRequired(EncryptedData::encrypt(
                key,
                content.as_bytes(),
            ))),
            _ => Err(DataError::Parse(format!(
                "SecretType::new: unknown type tag '{name}'",
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EncryptedData, Secret, SecretEntry, SecretType, Serialized};

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
}
