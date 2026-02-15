use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::prompt::{self, ConfirmResult};

/// プロジェクト初期化の設定。
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

const ALL_TIERS: &[Tier] = &[Tier::System, Tier::Business, Tier::Service];
const TIER_LABELS: &[&str] = &["system", "business", "service"];

/// 初期化ウィザードのステップ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    ProjectName,
    GitInit,
    SparseCheckout,
    TierSelection,
    Confirm,
}

/// プロジェクト初期化コマンドを実行する。
///
/// CLIフロー.md の「プロジェクト初期化」セクションに準拠:
/// [1] プロジェクト名入力
/// [2] Git リポジトリ初期化 (はい/いいえ)
/// [3] sparse-checkout 有効化 (はい/いいえ)
/// [4] (sparse-checkout時) Tier選択
/// [確認] 確認画面
///
/// 各ステップで Esc を押すと前のステップに戻る。
/// 最初のステップで Esc を押すとメインメニューに戻る。
pub fn run() -> Result<()> {
    println!("\n--- プロジェクト初期化 ---\n");

    let mut step = Step::ProjectName;
    let mut project_name = String::new();
    let mut git_init = false;
    let mut sparse_checkout = false;
    let mut tiers: Vec<Tier> = vec![];

    loop {
        match step {
            Step::ProjectName => match step_project_name()? {
                Some(name) => {
                    project_name = name;
                    step = Step::GitInit;
                }
                None => return Ok(()), // 最初のステップ → メインメニューに戻る
            },
            Step::GitInit => match step_git_init()? {
                Some(g) => {
                    git_init = g;
                    step = Step::SparseCheckout;
                }
                None => {
                    step = Step::ProjectName; // 前のステップに戻る
                }
            },
            Step::SparseCheckout => match step_sparse_checkout()? {
                Some(s) => {
                    sparse_checkout = s;
                    if sparse_checkout {
                        step = Step::TierSelection;
                    } else {
                        tiers = vec![Tier::System, Tier::Business, Tier::Service];
                        step = Step::Confirm;
                    }
                }
                None => {
                    step = Step::GitInit; // 前のステップに戻る
                }
            },
            Step::TierSelection => match step_tier_selection()? {
                Some(t) => {
                    tiers = t;
                    step = Step::Confirm;
                }
                None => {
                    step = Step::SparseCheckout; // 前のステップに戻る
                }
            },
            Step::Confirm => {
                let config = InitConfig {
                    project_name: project_name.clone(),
                    git_init,
                    sparse_checkout,
                    tiers: tiers.clone(),
                };
                print_confirmation(&config);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        execute_init(&config)?;
                        println!(
                            "\nプロジェクト '{}' の初期化が完了しました。",
                            config.project_name
                        );
                        return Ok(());
                    }
                    ConfirmResult::GoBack => {
                        // 前のステップに戻る
                        if sparse_checkout {
                            step = Step::TierSelection;
                        } else {
                            step = Step::SparseCheckout;
                        }
                    }
                    ConfirmResult::Cancel => {
                        println!("キャンセルしました。");
                        return Ok(());
                    }
                }
            }
        }
    }
}

/// ステップ1: プロジェクト名入力
///
/// 入力されたプロジェクト名が既にカレントディレクトリに存在する場合は
/// エラーメッセージを表示して再入力を促す。
fn step_project_name() -> Result<Option<String>> {
    loop {
        let name = match prompt::input_prompt("プロジェクト名を入力してください") {
            Ok(n) => n,
            Err(e) => {
                if is_dialoguer_escape(&e) {
                    return Ok(None);
                }
                return Err(e);
            }
        };
        if Path::new(&name).exists() {
            println!(
                "'{}' は既に存在します。別の名前を入力してください。",
                name
            );
            continue;
        }
        return Ok(Some(name));
    }
}

/// ステップ2: Git初期化
fn step_git_init() -> Result<Option<bool>> {
    prompt::yes_no_prompt("Git リポジトリを初期化しますか？")
}

/// ステップ3: sparse-checkout
fn step_sparse_checkout() -> Result<Option<bool>> {
    prompt::yes_no_prompt("sparse-checkout を有効にしますか？")
}

/// ステップ4: Tier選択
fn step_tier_selection() -> Result<Option<Vec<Tier>>> {
    let selected = prompt::multi_select_prompt(
        "チェックアウトするTierを選択してください（複数選択可）",
        TIER_LABELS,
    )?;

    match selected {
        None => Ok(None),
        Some(indices) => {
            if indices.is_empty() {
                println!("少なくとも1つのTierを選択してください。");
                // 再帰的にリトライ
                step_tier_selection()
            } else {
                let tiers: Vec<Tier> = indices.iter().map(|&i| ALL_TIERS[i]).collect();
                Ok(Some(tiers))
            }
        }
    }
}

/// 確認内容を表示する。
fn print_confirmation(config: &InitConfig) {
    let tiers_str: Vec<&str> = config.tiers.iter().map(|t| t.display()).collect();
    println!("\n[確認] 以下の内容で初期化します。よろしいですか？");
    println!("    プロジェクト名: {}", config.project_name);
    println!(
        "    Git 初期化:     {}",
        if config.git_init { "はい" } else { "いいえ" }
    );
    if config.sparse_checkout {
        println!(
            "    sparse-checkout: はい ({})",
            tiers_str.join(", ")
        );
    } else {
        println!("    sparse-checkout: いいえ");
    }
}

/// 実際の初期化処理を実行する。
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
        let status = Command::new("git")
            .arg("init")
            .current_dir(base)
            .status();
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

/// dialoguer のエスケープ/Ctrl+C を検出するヘルパー
fn is_dialoguer_escape(e: &anyhow::Error) -> bool {
    let msg = format!("{}", e);
    msg.contains("interrupted") || msg.contains("Escape")
}

// --- ファイル生成ヘルパー ---

fn generate_devcontainer_json(project_name: &str) -> String {
    format!(
        r#"{{
  "name": "{}",
  "dockerComposeFile": "../docker-compose.yaml",
  "service": "app",
  "workspaceFolder": "/workspace"
}}
"#,
        project_name
    )
}

fn generate_ci_yaml(project_name: &str) -> String {
    format!(
        r#"name: CI - {}

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
        run: echo "Build step for {}"
      - name: Test
        run: echo "Test step for {}"
"#,
        project_name, project_name, project_name
    )
}

fn generate_deploy_yaml(project_name: &str) -> String {
    format!(
        r#"name: Deploy - {}

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
        run: echo "Deploying {} to ${{{{ github.event.inputs.environment }}}}"
"#,
        project_name, project_name
    )
}

fn generate_docker_compose(project_name: &str) -> String {
    format!(
        r#"version: "3.8"

services:
  app:
    build: .
    container_name: {}-app
    volumes:
      - .:/workspace
    working_dir: /workspace
"#,
        project_name
    )
}

fn generate_readme(project_name: &str) -> String {
    format!(
        r#"# {}

k1s0 で生成されたプロジェクトです。

## ディレクトリ構成

- `regions/` - リージョン構成 (system / business / service)
- `api/` - API 定義 (proto / OpenAPI)
- `infra/` - インフラ設定
- `e2e/` - E2E テスト
- `docs/` - ドキュメント
"#,
        project_name
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
        let project_name = tmp
            .path()
            .join("my-project")
            .to_string_lossy()
            .to_string();
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
        let project_name = tmp
            .path()
            .join("git-project")
            .to_string_lossy()
            .to_string();
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
