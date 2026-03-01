use crate::domain::entity::navigation::NavigationConfig;

/// NavigationConfigLoader はナビゲーション設定の読み込みを抽象化するトレイト。
#[cfg_attr(test, mockall::automock)]
pub trait NavigationConfigLoader: Send + Sync {
    fn load(&self) -> anyhow::Result<NavigationConfig>;
}

/// YAML ファイルからナビゲーション設定を読み込むローダー。
pub struct YamlNavigationConfigLoader {
    path: String,
}

impl YamlNavigationConfigLoader {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

impl NavigationConfigLoader for YamlNavigationConfigLoader {
    fn load(&self) -> anyhow::Result<NavigationConfig> {
        let content = std::fs::read_to_string(&self.path)?;
        let config: NavigationConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn yaml_loader_success() {
        let yaml = r#"
version: 1
guards: []
routes:
  - id: root
    path: /
    redirect_to: /dashboard
"#;
        let dir = std::env::temp_dir().join("k1s0_nav_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_nav.yaml");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(yaml.as_bytes()).unwrap();

        let loader = YamlNavigationConfigLoader::new(path.to_str().unwrap());
        let config = loader.load().unwrap();
        assert_eq!(config.version, 1);
        assert_eq!(config.routes.len(), 1);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn yaml_loader_file_not_found() {
        let loader = YamlNavigationConfigLoader::new("/nonexistent/path.yaml");
        let result = loader.load();
        assert!(result.is_err());
    }
}
