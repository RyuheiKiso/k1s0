use serde::{Deserialize, Serialize};
use std::path::Path;

/// CLI全体の設定を保持する構造体。
///
/// プロジェクトルートの設定ファイル (k1s0.yaml) から読み込む。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CliConfig {
    /// プロジェクト名
    pub project_name: String,
    /// リージョンのルートパス
    pub regions_root: String,
    /// Docker レジストリ
    pub docker_registry: String,
    /// Go モジュールのベースパス
    pub go_module_base: String,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            project_name: String::new(),
            regions_root: "regions".to_string(),
            docker_registry: "harbor.internal.example.com".to_string(),
            go_module_base: "github.com/org/k1s0".to_string(),
        }
    }
}

/// 設定ファイルを読み込む。
///
/// 指定されたパスから YAML 形式の設定ファイルを読み込む。
/// ファイルが存在しない場合はデフォルト値を返す。
pub fn load_config(path: &str) -> anyhow::Result<CliConfig> {
    let config_path = Path::new(path);
    if !config_path.exists() {
        return Ok(CliConfig::default());
    }
    let content = std::fs::read_to_string(config_path)
        .map_err(|e| anyhow::anyhow!("設定ファイルの読み込みに失敗: {}", e))?;
    let config: CliConfig = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("設定ファイルのパースに失敗: {}", e))?;
    Ok(config)
}

/// 環境別設定をマージする。
///
/// ベース設定に環境別設定を上書きマージする。
/// config設計.md のマージ順序: config.yaml < config.{env}.yaml < Vault
pub fn merge_config(base: &mut CliConfig, override_path: &str) -> anyhow::Result<()> {
    let path = Path::new(override_path);
    if !path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("環境別設定の読み込みに失敗: {}", e))?;
    let override_config: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("環境別設定のパースに失敗: {}", e))?;

    if let serde_yaml::Value::Mapping(map) = override_config {
        if let Some(serde_yaml::Value::String(name)) = map.get(&serde_yaml::Value::String("project_name".to_string())) {
            base.project_name = name.clone();
        }
        if let Some(serde_yaml::Value::String(root)) = map.get(&serde_yaml::Value::String("regions_root".to_string())) {
            base.regions_root = root.clone();
        }
        if let Some(serde_yaml::Value::String(registry)) = map.get(&serde_yaml::Value::String("docker_registry".to_string())) {
            base.docker_registry = registry.clone();
        }
        if let Some(serde_yaml::Value::String(go_base)) = map.get(&serde_yaml::Value::String("go_module_base".to_string())) {
            base.go_module_base = go_base.clone();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = CliConfig::default();
        assert_eq!(config.project_name, "");
        assert_eq!(config.regions_root, "regions");
        assert_eq!(config.docker_registry, "harbor.internal.example.com");
        assert_eq!(config.go_module_base, "github.com/org/k1s0");
    }

    #[test]
    fn test_load_config_nonexistent_returns_default() {
        let config = load_config("nonexistent.yaml").unwrap();
        assert_eq!(config.regions_root, "regions");
        assert_eq!(config.docker_registry, "harbor.internal.example.com");
    }

    #[test]
    fn test_load_config_from_yaml() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "project_name: my-project\nregions_root: custom-regions\ndocker_registry: my-registry.io\ngo_module_base: github.com/myorg/myrepo"
        )
        .unwrap();
        let config = load_config(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.project_name, "my-project");
        assert_eq!(config.regions_root, "custom-regions");
        assert_eq!(config.docker_registry, "my-registry.io");
        assert_eq!(config.go_module_base, "github.com/myorg/myrepo");
    }

    #[test]
    fn test_load_config_partial_yaml_uses_defaults() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "project_name: partial-project").unwrap();
        let config = load_config(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.project_name, "partial-project");
        assert_eq!(config.regions_root, "regions");
        assert_eq!(config.docker_registry, "harbor.internal.example.com");
    }

    #[test]
    fn test_load_config_invalid_yaml_returns_error() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{{invalid yaml").unwrap();
        assert!(load_config(file.path().to_str().unwrap()).is_err());
    }

    #[test]
    fn test_merge_config_overrides_values() {
        let mut base = CliConfig::default();
        base.project_name = "base-project".to_string();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "project_name: overridden\ndocker_registry: custom-registry.io").unwrap();

        merge_config(&mut base, file.path().to_str().unwrap()).unwrap();
        assert_eq!(base.project_name, "overridden");
        assert_eq!(base.docker_registry, "custom-registry.io");
        assert_eq!(base.regions_root, "regions"); // not overridden
    }

    #[test]
    fn test_merge_config_nonexistent_file_noop() {
        let mut base = CliConfig::default();
        base.project_name = "original".to_string();
        merge_config(&mut base, "nonexistent.yaml").unwrap();
        assert_eq!(base.project_name, "original");
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = CliConfig {
            project_name: "test-project".to_string(),
            regions_root: "regions".to_string(),
            docker_registry: "harbor.internal.example.com".to_string(),
            go_module_base: "github.com/org/k1s0".to_string(),
        };
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: CliConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.project_name, deserialized.project_name);
        assert_eq!(config.regions_root, deserialized.regions_root);
        assert_eq!(config.docker_registry, deserialized.docker_registry);
        assert_eq!(config.go_module_base, deserialized.go_module_base);
    }
}
