use dirs;
use std::io::ErrorKind;
use std::path::Path;
use std::{fs, path::PathBuf, vec};

use chrono::{DateTime, Local};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct StoreMetaData {
    path: String,
    last_accessed: i64,
}
pub fn get_project_dirs() -> ProjectDirs {
    ProjectDirs::from("com", "satz", "scrtstore").expect("Unable to get project directories")
}

impl StoreMetaData {
    pub fn new(path: &Path) -> Self {
        StoreMetaData {
            path: path.to_str().unwrap().to_string(),
            last_accessed: chrono::Utc::now().timestamp(),
        }
    }

    pub fn save(&self) {
        let project_dir = get_project_dirs();
        let mut all_stores = get_known_stores(&project_dir);
        all_stores.push(self.clone());
        save_known_stores(&all_stores, &project_dir);
    }

    pub fn get_last_accessed_timestamp(&self) -> DateTime<Local> {
        let ts = chrono::DateTime::from_timestamp(self.last_accessed, 0).unwrap_or_default();
        ts.with_timezone(&Local)
    }

    pub fn get_display_path(&self) -> String {
        let path = Path::new(&self.path);
        if let Some(home) = dirs::home_dir()
            && let Ok(relative) = path.strip_prefix(&home)
        {
            if relative.as_os_str().is_empty() {
                return "~".to_string();
            }

            return format!("~/{}", relative.display());
        }
        path.display().to_string()
    }

    pub fn get_path_buf(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }
}

fn get_store_state_path(project_dir: &ProjectDirs) -> Option<PathBuf> {
    let store_state_file = project_dir.state_dir().map(|p| p.join("stores_state.json"));

    if store_state_file.is_none() {
        eprintln!("Unable to find app state directory.");
    }
    store_state_file
}

pub fn get_known_stores(project_dir: &ProjectDirs) -> Vec<StoreMetaData> {
    let ss_file = match get_store_state_path(project_dir) {
        Some(p) => p,
        None => return vec![],
    };

    match fs::read_to_string(ss_file) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_else(|e| {
            eprintln!("Failed to parse known stores metadata: {e}");
            vec![]
        }),

        Err(e) if e.kind() == ErrorKind::NotFound => {
            vec![]
        }

        Err(e) => {
            eprintln!("Failed to access known stores metadata: {e}");
            vec![]
        }
    }
}

pub fn save_known_stores(data: &Vec<StoreMetaData>, project_dir: &ProjectDirs) {
    let ss_file = match get_store_state_path(project_dir) {
        Some(p) => p,
        None => return,
    };

    let json = match serde_json::to_string(&data) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to serialize known stores metadata due to error: {e}",);
            return;
        }
    };

    if let Some(parent) = ss_file.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        eprintln!(
            "Failed to create parent directories {}: {e}",
            parent.display()
        );
    }

    if let Err(e) = fs::write(&ss_file, json) {
        eprintln!(
            "Failed to save known stores data to {}: {e}",
            ss_file.display()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_get_known_stores_file_not_exists() {
        let temp = tempdir().unwrap();
        let proj_dir = directories::ProjectDirs::from_path(temp.path().to_path_buf()).unwrap();

        let stores = get_known_stores(&proj_dir);
        assert!(
            stores.is_empty(),
            "Expected empty vector when file does not exist"
        );
    }

    #[test]
    fn test_get_known_stores_invalid_json() {
        let temp = tempdir().unwrap();
        let state_dir = temp.path();
        fs::create_dir_all(state_dir).unwrap();

        fs::write(
            state_dir.join("stores_state.json"),
            "{ invalid_json: true }",
        )
        .unwrap();

        let proj_dir = directories::ProjectDirs::from_path(temp.path().to_path_buf()).unwrap();
        let stores = get_known_stores(&proj_dir);

        assert!(
            stores.is_empty(),
            "Expected empty vector when JSON is malformed"
        );
    }

    #[test]
    fn test_save_and_get_known_stores_normal_case() {
        let temp = tempdir().unwrap();
        let proj_dir = &directories::ProjectDirs::from_path(temp.path().to_path_buf()).unwrap();

        let sample_data = vec![
            StoreMetaData {
                path: "/var/lib/store_one".to_string(),
                last_accessed: 1718000000,
            },
            StoreMetaData {
                path: "/var/lib/store_two".to_string(),
                last_accessed: 1718001111,
            },
        ];

        save_known_stores(&sample_data, proj_dir);
        let loaded_stores = get_known_stores(proj_dir);

        assert_eq!(
            loaded_stores, sample_data,
            "Loaded stores should match saved data"
        );
    }
}
