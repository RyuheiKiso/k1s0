use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::application::port::ConfigStore;
use crate::domain::workspace::WorkspacePath;

#[derive(Debug, Serialize, Deserialize, Default)]
struct ConfigData {
    workspace_path: Option<String>,
}

pub struct TomlConfigStore {
    path: PathBuf,
}

impl TomlConfigStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default_path() -> PathBuf {
        let config_dir = dirs::config_dir().expect("設定ディレクトリを取得できません");
        config_dir.join("k1s0").join("config.toml")
    }
}

impl ConfigStore for TomlConfigStore {
    fn load_workspace_path(&self) -> Option<WorkspacePath> {
        let content = fs::read_to_string(&self.path).ok()?;
        let data: ConfigData = toml::from_str(&content).ok()?;
        let raw = data.workspace_path?;
        WorkspacePath::new(&raw).ok()
    }

    fn save_workspace_path(&self, path: &WorkspacePath) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = ConfigData {
            workspace_path: Some(path.to_string_lossy()),
        };
        let content = toml::to_string_pretty(&data)?;
        fs::write(&self.path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let store = TomlConfigStore::new(config_path);

        let ws = WorkspacePath::new(r"C:\my\workspace").unwrap();
        store.save_workspace_path(&ws).unwrap();

        let loaded = store.load_workspace_path();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().to_string_lossy(), r"C:\my\workspace");
    }

    #[test]
    fn load_returns_none_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("nonexistent.toml");
        let store = TomlConfigStore::new(config_path);

        assert!(store.load_workspace_path().is_none());
    }

    #[test]
    fn creates_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("nested").join("dir").join("config.toml");
        let store = TomlConfigStore::new(config_path.clone());

        let ws = WorkspacePath::new(r"C:\test").unwrap();
        store.save_workspace_path(&ws).unwrap();

        assert!(config_path.exists());
    }
}
