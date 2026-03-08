use std::fs;
use std::path::{Path, PathBuf};

use crate::auth::cipher::{Encrypted, Serialized};
use crate::auth::twofa::TwoFAMethod;
use crate::data::secret::{Secret, SecretType};
use crate::data::store::ScrtStore;
use crate::data::utils::{DataError, IdType};

/// Create a new empty store
pub fn new_scrtstore<P: AsRef<Path>>(
    path: P,
    password: String,
    pass_phrase: String,
) -> Result<(), DataError> {
    let scrtstore = ScrtStore::new(password, pass_phrase);
    save_scrtstore(path, scrtstore)?;
    Ok(())
}

/// Load scrt file to memory
pub fn load_scrtstore<P: AsRef<Path>>(path: P) -> Result<ScrtStore, DataError> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(DataError::LoadError(format!(
            "File does not exist: {}",
            path.display()
        )));
    }

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| DataError::LoadError(format!("Invalid filename: {}", path.display())))?;

    if !filename.ends_with(".scrt") {
        return Err(DataError::LoadError(format!(
            "File extension must be .scrt: {filename}"
        )));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| DataError::LoadError(format!("Failed to read file: {e}")))?;

    ScrtStore::parse(&content)
}

/// Save scrtstore as .scrt file in disk
pub fn save_scrtstore<P: AsRef<Path>>(path: P, scrtstore: ScrtStore) -> Result<(), DataError> {
    let mut pathbuf = PathBuf::from(path.as_ref());
    let file_name = pathbuf.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if !file_name.ends_with(".scrt") {
        let stem = pathbuf
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        pathbuf.set_file_name(format!("{stem}.scrt"));
    }

    let content = scrtstore.dumps();
    fs::write(&pathbuf, content)
        .map_err(|e| DataError::SaveError(format!("Failed to write file: {e}")))
}

pub struct VaultUpdatePayload {
    id: Option<IdType>,
    name: String,
    info: Option<String>,
    group: Option<String>,
    tags: Vec<String>,
    secrets: Vec<Secret>,
}

/// Update Single Vault Entry
pub fn update_vault_entry<P: AsRef<Path> + Clone>(
    path: P,
    entry: VaultUpdatePayload,
    master_key: &str,
) -> Result<(), DataError> {
    let mut scrtstore = load_scrtstore(path.clone())?;
    let mut vault = scrtstore.unlock(master_key)?;

    vault.atomic_update(
        entry.id,
        entry.name,
        entry.info,
        entry.group,
        entry.tags,
        entry.secrets,
        master_key,
    );
    scrtstore.data = vault.encrypt(master_key);
    save_scrtstore(path, scrtstore)?;
    Ok(())
}

/// Update any of the 2FA method using any other existing 2FA method
pub fn update_store_2fa<P: AsRef<Path> + Clone>(
    path: P,

    password: String,

    new_method: TwoFAMethod,
    new_verification_data: String,

    method: TwoFAMethod,
    verification_data: String,
) -> Result<(), DataError> {
    let mut scrtstore = load_scrtstore(path.clone())?;
    scrtstore.add_2fa_method(
        password,
        new_method,
        new_verification_data,
        method,
        verification_data,
    )?;
    save_scrtstore(path, scrtstore)?;
    Ok(())
}

/// Full deep copy of the store.
/// It removes the all the existing 2FA methods.
pub fn change_password<P: AsRef<Path> + Clone>(
    path: P,
    password: String,

    new_password: String,
    new_pass_phrase: String,

    method: TwoFAMethod,
    verification_data: String,
) -> Result<(), DataError> {
    let scrtstore = load_scrtstore(path.clone())?;

    let master_key: String =
        scrtstore.get_master_key(method, password.clone(), verification_data)?;
    let vault = scrtstore.unlock(&master_key)?;

    let mut new_store: ScrtStore =
        ScrtStore::new(new_password.clone(), new_pass_phrase.clone());
    let new_master_key: String = new_store.get_master_key(
        TwoFAMethod::PassPhrase,
        new_password.clone(),
        new_pass_phrase.clone(),
    )?;
    let mut new_vault = new_store.unlock(&new_master_key)?;

    for meta in vault.metadata.iter() {
        let data = vault.get_secret(meta.id, &master_key).ok_or_else(|| {
            DataError::SaveError(format!(
                "Decrypt failed for secret {}: {}",
                meta.id, meta.name
            ))
        })?;
        let mut new_scrts: Vec<Secret> = vec![];
        for scrt in data.secrets {
            let content: String = scrt.get_data(Some(&password))?;
            let new_scrt: Secret = Secret {
                name: scrt.name,
                secret: SecretType::new(content, scrt.secret.get_name(), &new_password)?,
            };
            new_scrts.push(new_scrt);
        }
        new_vault.add_entry(
            meta.name.clone(),
            meta.info.clone(),
            meta.group.clone(),
            meta.tags.clone(),
            new_scrts,
            &new_master_key,
        );
    }
    new_store.data = new_vault.encrypt(&new_master_key);

    save_scrtstore(path, new_store)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        change_password, load_scrtstore, new_scrtstore, update_vault_entry,
        update_store_2fa, VaultUpdatePayload,
    };
    use crate::auth::twofa::TwoFAMethod;
    use crate::data::secret::{Secret, SecretType};
    use crate::data::store::ScrtStore;
    use crate::data::utils::DataError;
    use tempfile::Builder;

    #[test]
    fn t1_create_new_store_load_unlock_metadata_empty() {
        let _tmp: tempfile::NamedTempFile =
            Builder::new().suffix(".scrt").tempfile().expect("tmp file");
        let path: std::path::PathBuf = _tmp.path().to_path_buf();
        let password: String = "pw1".to_string();
        let pass_phrase: String = "pp1".to_string();

        new_scrtstore(path.clone(), password.clone(), pass_phrase.clone())
            .expect("new_scrtstore");
        let store: ScrtStore = load_scrtstore(&path).expect("load_scrtstore");
        let master_key: String = store
            .get_master_key(
                TwoFAMethod::PassPhrase,
                password,
                pass_phrase,
            )
            .expect("get_master_key");
        let vault = store.unlock(&master_key).expect("unlock");
        assert!(vault.metadata.is_empty());
        assert_eq!(vault.secret_count(), 0);
    }

    #[test]
    fn t2_create_load_update_entries_reload_state_secrets_match_with_passrequired() {
        let _tmp: tempfile::NamedTempFile =
            Builder::new().suffix(".scrt").tempfile().expect("tmp file");
        let path: std::path::PathBuf = _tmp.path().to_path_buf();
        let password: String = "pw2".to_string();
        let pass_phrase: String = "pp2".to_string();
        let entry_pass: &str = "entry_secret_key";

        new_scrtstore(path.clone(), password.clone(), pass_phrase.clone())
            .expect("new_scrtstore");
        let store: ScrtStore = load_scrtstore(&path).expect("load");
        let master_key: String = store
            .get_master_key(
                TwoFAMethod::PassPhrase,
                password.clone(),
                pass_phrase.clone(),
            )
            .expect("master_key");

        let entry1: VaultUpdatePayload = VaultUpdatePayload {
            id: None,
            name: "entry_a".to_string(),
            info: Some("info a".to_string()),
            group: Some("g1".to_string()),
            tags: vec!["x".to_string()],
            secrets: vec![
                Secret {
                    name: "plain".to_string(),
                    secret: SecretType::new_view("visible".to_string()),
                },
                Secret {
                    name: "protected".to_string(),
                    secret: SecretType::new_passrequired(
                        "hidden_content".to_string(),
                        entry_pass,
                    ),
                },
            ],
        };
        update_vault_entry(path.clone(), entry1, &master_key).expect("update entry1");

        let entry2: VaultUpdatePayload = VaultUpdatePayload {
            id: None,
            name: "entry_b".to_string(),
            info: None,
            group: None,
            tags: vec![],
            secrets: vec![Secret {
                name: "k".to_string(),
                secret: SecretType::new_hidden("h".to_string()),
            }],
        };
        update_vault_entry(path.clone(), entry2, &master_key).expect("update entry2");

        let store2: ScrtStore = load_scrtstore(&path).expect("reload");
        let master_key2: String = store2
            .get_master_key(
                TwoFAMethod::PassPhrase,
                password.clone(),
                pass_phrase.clone(),
            )
            .expect("master_key");
        let vault2 = store2.unlock(&master_key2).expect("unlock");
        assert_eq!(vault2.metadata.len(), 2);
        assert_eq!(vault2.metadata.len(), vault2.secret_count());

        let meta_a: &crate::data::vault::MetadataEntry = vault2
            .metadata
            .iter()
            .find(|m| m.name == "entry_a")
            .expect("entry_a");
        let entry_a_data = vault2
            .get_secret(meta_a.id, &master_key2)
            .expect("get_secret entry_a");
        assert_eq!(entry_a_data.secrets.len(), 2);
        let view_s: &Secret = entry_a_data.secrets.iter().find(|s| s.name == "plain").unwrap();
        assert_eq!(view_s.get_data(None).unwrap(), "visible");
        let pass_s: &Secret = entry_a_data.secrets.iter().find(|s| s.name == "protected").unwrap();
        assert_eq!(
            pass_s.get_data(Some(entry_pass)).unwrap(),
            "hidden_content"
        );
    }

    #[test]
    fn t3_update_pass_phrase_old_fail_new_pass() {
        let _tmp: tempfile::NamedTempFile =
            Builder::new().suffix(".scrt").tempfile().expect("tmp file");
        let path: std::path::PathBuf = _tmp.path().to_path_buf();
        let password: String = "pw3".to_string();
        let old_phrase: String = "old_pp".to_string();
        let new_phrase: String = "new_pp".to_string();

        new_scrtstore(path.clone(), password.clone(), old_phrase.clone())
            .expect("new_scrtstore");
        update_store_2fa(
            path.clone(),
            password.clone(),
            TwoFAMethod::PassPhrase,
            new_phrase.clone(),
            TwoFAMethod::PassPhrase,
            old_phrase.clone(),
        )
        .expect("update_store_2fa");

        let store: ScrtStore = load_scrtstore(&path).expect("load");
        let old_unlock: Result<String, DataError> =
            store.get_master_key(TwoFAMethod::PassPhrase, password.clone(), old_phrase);
        assert!(old_unlock.is_err(), "unlock with old pass_phrase should fail");
        let master_key: String = store
            .get_master_key(TwoFAMethod::PassPhrase, password, new_phrase)
            .expect("unlock with new pass_phrase");
        let _vault = store.unlock(&master_key).expect("unlock");
    }

    #[test]
    fn t4_change_password_reload_unlock_new_password() {
        let _tmp: tempfile::NamedTempFile =
            Builder::new().suffix(".scrt").tempfile().expect("tmp file");
        let path: std::path::PathBuf = _tmp.path().to_path_buf();
        let old_password: String = "old_pw".to_string();
        let new_password: String = "new_pw".to_string();
        let pass_phrase: String = "pp4".to_string();

        new_scrtstore(path.clone(), old_password.clone(), pass_phrase.clone())
            .expect("new_scrtstore");
        change_password(
            path.clone(),
            old_password,
            new_password.clone(),
            pass_phrase.clone(),
            TwoFAMethod::PassPhrase,
            pass_phrase.clone(),
        )
        .expect("change_password");

        let store: ScrtStore = load_scrtstore(&path).expect("reload");
        let master_key: String = store
            .get_master_key(
                TwoFAMethod::PassPhrase,
                new_password,
                pass_phrase,
            )
            .expect("get_master_key with new password");
        let _vault = store.unlock(&master_key).expect("unlock with new password");
    }
}
