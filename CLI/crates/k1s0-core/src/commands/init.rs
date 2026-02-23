use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

/// プロジェクト初期化の設定。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitConfig {
    /// プロジェクト名
    pub project_name: String,
    /// Git リポジトリを初期化するか
    pub git_init: bool,
    /// sparse-checkout を有効にするか
    pub sparse_checkout: bool,
    /// sparse-checkout 対象のTier一覧
    pub tiers: Vec<Tier>,
}

/// Tier種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    System,
    Business,
    Service,
}

impl Tier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::System => "system",
            Tier::Business => "business",
            Tier::Service => "service",
        }
    }

    pub fn display(&self) -> &'static str {
        match self {
            Tier::System => "system",
            Tier::Business => "business",
            Tier::Service => "service",
        }
    }
}

/// 実際の初期化処理を実行する。
///
/// # Errors
/// エラーが発生した場合。
pub fn execute_init(config: &InitConfig) -> Result<()> {
    let base = Path::new(&config.project_name);

    // regions/ ディレクトリ作成
    for tier in &config.tiers {
        fs::create_dir_all(base.join("regions").join(tier.as_str()))?;
    }

    // api/ ディレクトリ
    fs::create_dir_all(base.join("api/proto"))?;

    // infra/ ディレクトリ
    fs::create_dir_all(base.join("infra"))?;

    // e2e/ ディレクトリ
    fs::create_dir_all(base.join("e2e"))?;

    // docs/ ディレクトリ
    fs::create_dir_all(base.join("docs"))?;

    // .devcontainer/
    fs::create_dir_all(base.join(".devcontainer"))?;
    fs::write(
        base.join(".devcontainer/devcontainer.json"),
        generate_devcontainer_json(&config.project_name),
    )?;

    // .github/workflows/
    fs::create_dir_all(base.join(".github/workflows"))?;
    fs::write(
        base.join(".github/workflows/ci.yaml"),
        generate_ci_yaml(&config.project_name),
    )?;
    fs::write(
        base.join(".github/workflows/deploy.yaml"),
        generate_deploy_yaml(&config.project_name),
    )?;

    // docker-compose.yaml
    fs::write(
        base.join("docker-compose.yaml"),
        generate_docker_compose(&config.project_name),
    )?;

    // README.md
    fs::write(
        base.join("README.md"),
        generate_readme(&config.project_name),
    )?;

    // Git初期化
    if config.git_init {
        let status = Command::new("git").arg("init").current_dir(base).status();
        match status {
            Ok(s) if s.success() => {
                // sparse-checkout
                if config.sparse_checkout {
                    let _ = Command::new("git")
                        .args(["sparse-checkout", "init", "--cone"])
                        .current_dir(base)
                        .status();
                    let tier_paths: Vec<String> = config
                        .tiers
                        .iter()
                        .map(|t| format!("regions/{}", t.as_str()))
                        .collect();
                    let _ = Command::new("git")
                        .args(["sparse-checkout", "set"])
                        .args(&tier_paths)
                        .current_dir(base)
                        .status();
                }
            }
            _ => {
                eprintln!("警告: git init に失敗しました。手動で初期化してください。");
            }
        }
    }

    Ok(())
}

// --- ファイル生成ヘルパー ---

fn generate_devcontainer_json(project_name: &str) -> String {
    format!(
        r#"{{
  "name": "{project_name}",
  "dockerComposeFile": "../docker-compose.yaml",
  "service": "app",
  "workspaceFolder": "/workspace"
}}
"#
    )
}

fn generate_ci_yaml(project_name: &str) -> String {
    format!(
        r#"name: CI - {project_name}

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: echo "Build step for {project_name}"
      - name: Test
        run: echo "Test step for {project_name}"
"#
    )
}

fn generate_deploy_yaml(project_name: &str) -> String {
    format!(
        r#"name: Deploy - {project_name}

on:
  workflow_dispatch:
    inputs:
      environment:
        description: "Deploy environment"
        required: true
        type: choice
        options:
          - dev
          - staging
          - prod

jobs:
  deploy:
    runs-on: ubuntu-latest
    environment: ${{{{ github.event.inputs.environment }}}}
    steps:
      - uses: actions/checkout@v4
      - name: Deploy
        run: echo "Deploying {project_name} to ${{{{ github.event.inputs.environment }}}}"
"#
    )
}

fn generate_docker_compose(project_name: &str) -> String {
    format!(
        r#"version: "3.8"

services:
  app:
    build: .
    container_name: {project_name}-app
    volumes:
      - .:/workspace
    working_dir: /workspace
"#
    )
}

fn generate_readme(project_name: &str) -> String {
    format!(
        r"# {project_name}

k1s0 で生成されたプロジェクトです。

## ディレクトリ構成

- `regions/` - リージョン構成 (system / business / service)
- `api/` - API 定義 (proto / OpenAPI)
- `infra/` - インフラ設定
- `e2e/` - E2E テスト
- `docs/` - ドキュメント
"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_tier_as_str() {
        assert_eq!(Tier::System.as_str(), "system");
        assert_eq!(Tier::Business.as_str(), "business");
        assert_eq!(Tier::Service.as_str(), "service");
    }

    #[test]
    fn test_tier_display() {
        assert_eq!(Tier::System.display(), "system");
        assert_eq!(Tier::Business.display(), "business");
        assert_eq!(Tier::Service.display(), "service");
    }

    #[test]
    fn test_init_config_creation() {
        let config = InitConfig {
            project_name: "test-project".to_string(),
            git_init: true,
            sparse_checkout: true,
            tiers: vec![Tier::System, Tier::Business],
        };
        assert_eq!(config.project_name, "test-project");
        assert!(config.git_init);
        assert!(config.sparse_checkout);
        assert_eq!(config.tiers.len(), 2);
    }

    #[test]
    fn test_execute_init_creates_directories() {
        let tmp = TempDir::new().unwrap();
        let project_name = tmp.path().join("my-project").to_string_lossy().to_string();
        let config = InitConfig {
            project_name,
            git_init: false,
            sparse_checkout: false,
            tiers: vec![Tier::System, Tier::Business, Tier::Service],
        };
        execute_init(&config).unwrap();

        let base = Path::new(&config.project_name);
        assert!(base.join("regions/system").is_dir());
        assert!(base.join("regions/business").is_dir());
        assert!(base.join("regions/service").is_dir());
        assert!(base.join("api/proto").is_dir());
        assert!(base.join("infra").is_dir());
        assert!(base.join("e2e").is_dir());
        assert!(base.join("docs").is_dir());
        assert!(base.join(".devcontainer/devcontainer.json").is_file());
        assert!(base.join(".github/workflows/ci.yaml").is_file());
        assert!(base.join(".github/workflows/deploy.yaml").is_file());
        assert!(base.join("docker-compose.yaml").is_file());
        assert!(base.join("README.md").is_file());
    }

    #[test]
    fn test_execute_init_partial_tiers() {
        let tmp = TempDir::new().unwrap();
        let project_name = tmp
            .path()
            .join("partial-project")
            .to_string_lossy()
            .to_string();
        let config = InitConfig {
            project_name,
            git_init: false,
            sparse_checkout: true,
            tiers: vec![Tier::System],
        };
        execute_init(&config).unwrap();

        let base = Path::new(&config.project_name);
        assert!(base.join("regions/system").is_dir());
        assert!(!base.join("regions/business").exists());
        assert!(!base.join("regions/service").exists());
    }

    #[test]
    fn test_execute_init_with_git() {
        let tmp = TempDir::new().unwrap();
        let project_name = tmp.path().join("git-project").to_string_lossy().to_string();
        let config = InitConfig {
            project_name,
            git_init: true,
            sparse_checkout: false,
            tiers: vec![Tier::System, Tier::Business, Tier::Service],
        };
        // Git が利用可能であればテスト成功
        let result = execute_init(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_devcontainer_json() {
        let json = generate_devcontainer_json("my-app");
        assert!(json.contains("\"name\": \"my-app\""));
        assert!(json.contains("docker-compose.yaml"));
    }

    #[test]
    fn test_generate_ci_yaml() {
        let yaml = generate_ci_yaml("my-app");
        assert!(yaml.contains("CI - my-app"));
        assert!(yaml.contains("actions/checkout"));
    }

    #[test]
    fn test_generate_deploy_yaml() {
        let yaml = generate_deploy_yaml("my-app");
        assert!(yaml.contains("Deploy - my-app"));
        assert!(yaml.contains("workflow_dispatch"));
    }

    #[test]
    fn test_generate_docker_compose() {
        let yaml = generate_docker_compose("my-app");
        assert!(yaml.contains("my-app-app"));
        assert!(yaml.contains("/workspace"));
    }

    #[test]
    fn test_generate_readme() {
        let md = generate_readme("my-app");
        assert!(md.contains("# my-app"));
        assert!(md.contains("regions/"));
    }
}
