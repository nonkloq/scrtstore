use std::fs;
use std::path::{Path, PathBuf};

use crate::auth::cipher::Serialized;
use crate::data::ScrtStore;

pub fn load_scrtstore<P: AsRef<Path>>(path: P) -> Result<ScrtStore, String> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("Invalid filename: {}", path.display()))?;

    if !filename.ends_with(".scrt") {
        return Err(format!("File extension must be .scrt: {}", filename));
    }

    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;

    ScrtStore::parse(&content).map_err(|e| format!("Parse error: {:?}", e))
}

pub fn save_scrtstore<P: AsRef<Path>>(path: P, scrtstore: &ScrtStore) -> Result<(), String> {
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
    fs::write(&pathbuf, content).map_err(|e| format!("Failed to write file: {e}"))
}

