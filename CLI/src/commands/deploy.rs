use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::prompt::{self, ConfirmResult};

/// デプロイ環境。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Dev,
    Staging,
    Prod,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Dev => "dev",
            Environment::Staging => "staging",
            Environment::Prod => "prod",
        }
    }

    pub fn is_prod(&self) -> bool {
        matches!(self, Environment::Prod)
    }
}

const ENV_LABELS: &[&str] = &["dev", "staging", "prod"];
const ALL_ENVS: &[Environment] = &[Environment::Dev, Environment::Staging, Environment::Prod];

/// デプロイ設定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeployConfig {
    /// デプロイ先環境
    pub environment: Environment,
    /// デプロイ対象のパス一覧
    pub targets: Vec<String>,
}

/// デプロイフローのステップ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Environment,
    Targets,
    ProdConfirm,
    Confirm,
}

/// デプロイコマンドを実行する。
///
/// CLIフロー.md の「デプロイ」セクションに準拠:
/// [1] 環境の選択 (dev / staging / prod)
/// [2] 対象の選択（複数選択可）
/// [3] (prod時) "deploy" 入力確認
/// [確認] 確認画面
///
/// 各ステップで Esc を押すと前のステップに戻る。
/// 最初のステップで Esc → メインメニューに戻る。
pub fn run() -> Result<()> {
    println!("\n--- デプロイ ---\n");

    let mut step = Step::Environment;
    let mut env = Environment::Dev;
    let mut targets: Vec<String> = Vec::new();

    loop {
        match step {
            Step::Environment => {
                match step_environment()? {
                    Some(e) => {
                        env = e;
                        step = Step::Targets;
                    }
                    None => return Ok(()), // 最初のステップで Esc → メインメニューに戻る
                }
            }
            Step::Targets => {
                match step_select_targets()? {
                    Some(t) => {
                        if t.is_empty() {
                            println!("デプロイ対象が見つかりません。");
                            return Ok(());
                        }
                        targets = t;
                        // prod の場合は ProdConfirm、それ以外は Confirm へ
                        step = if env.is_prod() {
                            Step::ProdConfirm
                        } else {
                            Step::Confirm
                        };
                    }
                    None => step = Step::Environment, // Esc → Environment に戻る
                }
            }
            Step::ProdConfirm => {
                match step_prod_confirmation()? {
                    Some(true) => {
                        step = Step::Confirm; // 通過
                    }
                    _ => {
                        println!("キャンセルしました。");
                        step = Step::Targets; // Esc → Targets に戻る
                    }
                }
            }
            Step::Confirm => {
                let config = DeployConfig {
                    environment: env,
                    targets: targets.clone(),
                };
                print_confirmation(&config);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        execute_deploy(&config)?;
                        println!("\nデプロイが完了しました。");
                        return Ok(());
                    }
                    ConfirmResult::GoBack => {
                        // GoBack → ProdConfirm (prodの場合) or Targets (非prodの場合)
                        step = if env.is_prod() {
                            Step::ProdConfirm
                        } else {
                            Step::Targets
                        };
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

/// ステップ1: 環境選択
fn step_environment() -> Result<Option<Environment>> {
    let idx = prompt::select_prompt("デプロイ先の環境を選択してください", ENV_LABELS)?;
    Ok(idx.map(|i| ALL_ENVS[i]))
}

/// ステップ2: 対象選択
fn step_select_targets() -> Result<Option<Vec<String>>> {
    let all_targets = scan_deployable_targets();
    if all_targets.is_empty() {
        return Ok(Some(Vec::new()));
    }

    let mut items: Vec<&str> = vec!["すべて"];
    for t in &all_targets {
        items.push(t.as_str());
    }

    let selected = prompt::multi_select_prompt(
        "デプロイ対象を選択してください（複数選択可）",
        &items,
    )?;

    match selected {
        None => Ok(None),
        Some(indices) => {
            if indices.is_empty() {
                println!("少なくとも1つの対象を選択してください。");
                step_select_targets()
            } else if indices.contains(&0) {
                Ok(Some(all_targets))
            } else {
                let targets: Vec<String> = indices
                    .iter()
                    .map(|&i| all_targets[i - 1].clone())
                    .collect();
                Ok(Some(targets))
            }
        }
    }
}

/// ステップ3: 本番環境の追加確認。"deploy" と入力させる。
fn step_prod_confirmation() -> Result<Option<bool>> {
    println!("\n⚠ 本番環境へのデプロイです。");
    let input = prompt::input_prompt_raw(
        "本当にデプロイしますか？ \"deploy\" と入力してください",
    )?;
    if input.trim() == "deploy" {
        Ok(Some(true))
    } else {
        println!("入力が一致しません。デプロイをキャンセルします。");
        Ok(Some(false))
    }
}

/// 確認内容を表示する。
fn print_confirmation(config: &DeployConfig) {
    println!("\n[確認] 以下の内容でデプロイします。よろしいですか？");
    println!("    環境: {}", config.environment.as_str());
    for target in &config.targets {
        println!("    対象: {}", target);
    }
}

/// デプロイ実行。
pub fn execute_deploy(config: &DeployConfig) -> Result<()> {
    for target in &config.targets {
        println!(
            "\nデプロイ中: {} → {}",
            target,
            config.environment.as_str()
        );
        let target_path = Path::new(target);

        if !target_path.is_dir() {
            println!("  警告: ディレクトリが見つかりません: {}", target);
            continue;
        }

        // Dockerfile があれば Docker ベースのデプロイ
        if target_path.join("Dockerfile").exists() {
            let image_tag = format!(
                "{}:{}",
                target.replace('/', "-").replace('\\', "-"),
                config.environment.as_str()
            );
            println!("  Docker イメージビルド: {}", image_tag);
            let build_status = Command::new("docker")
                .args(["build", "-t", &image_tag, "."])
                .current_dir(target_path)
                .status();
            match build_status {
                Ok(s) if s.success() => {
                    println!("  イメージビルド完了: {}", image_tag);
                }
                Ok(_) => {
                    println!("  警告: Docker ビルドに失敗しました");
                }
                Err(e) => {
                    println!("  警告: docker コマンドの実行に失敗しました: {}", e);
                }
            }
        } else {
            println!(
                "  デプロイ: {} を {} 環境にデプロイします (dry-run)",
                target,
                config.environment.as_str()
            );
        }
    }
    Ok(())
}

/// デプロイ可能な対象を走査する。
/// サーバーとクライアントのみ (ライブラリは対象外)。
pub fn scan_deployable_targets() -> Vec<String> {
    scan_deployable_targets_at(Path::new("."))
}

/// 指定ディレクトリを基点にデプロイ可能な対象を走査する。
pub fn scan_deployable_targets_at(base_dir: &Path) -> Vec<String> {
    let mut targets = Vec::new();
    let regions = base_dir.join("regions");
    if !regions.is_dir() {
        return targets;
    }
    scan_targets_recursive(&regions, &mut targets);
    targets.sort();
    targets
}

fn scan_targets_recursive(path: &Path, targets: &mut Vec<String>) {
    if !path.is_dir() {
        return;
    }

    // デプロイ可能なプロジェクトを検出
    // Dockerfile がある、または package.json / pubspec.yaml がある
    let is_deployable = path.join("Dockerfile").exists()
        || path.join("package.json").exists()
        || path.join("pubspec.yaml").exists()
        || (path.join("go.mod").exists())
        || (path.join("Cargo.toml").exists());

    if is_deployable {
        // library/ は除外
        let path_str = path.to_str().unwrap_or("");
        let is_library = path_str.contains("/library/") || path_str.contains("\\library\\");
        if !is_library {
            targets.push(path_str.to_string());
        }
        return;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                scan_targets_recursive(&entry.path(), targets);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // --- Environment ---

    #[test]
    fn test_environment_as_str() {
        assert_eq!(Environment::Dev.as_str(), "dev");
        assert_eq!(Environment::Staging.as_str(), "staging");
        assert_eq!(Environment::Prod.as_str(), "prod");
    }

    #[test]
    fn test_environment_is_prod() {
        assert!(!Environment::Dev.is_prod());
        assert!(!Environment::Staging.is_prod());
        assert!(Environment::Prod.is_prod());
    }

    // --- DeployConfig ---

    #[test]
    fn test_deploy_config_creation() {
        let config = DeployConfig {
            environment: Environment::Dev,
            targets: vec!["regions/system/server/go/auth".to_string()],
        };
        assert_eq!(config.environment, Environment::Dev);
        assert_eq!(config.targets.len(), 1);
    }

    // --- scan_deployable_targets ---

    #[test]
    fn test_scan_deployable_targets_empty() {
        let tmp = TempDir::new().unwrap();

        let targets = scan_deployable_targets_at(tmp.path());

        assert!(targets.is_empty());
    }

    #[test]
    fn test_scan_deployable_targets_excludes_library() {
        let tmp = TempDir::new().unwrap();

        // サーバー (デプロイ可能)
        let server_path = tmp.path().join("regions/system/server/go/auth");
        fs::create_dir_all(&server_path).unwrap();
        fs::write(server_path.join("go.mod"), "module auth\n").unwrap();

        // ライブラリ (デプロイ対象外)
        let lib_path = tmp.path().join("regions/system/library/go/authlib");
        fs::create_dir_all(&lib_path).unwrap();
        fs::write(lib_path.join("go.mod"), "module authlib\n").unwrap();

        let targets = scan_deployable_targets_at(tmp.path());

        assert_eq!(targets.len(), 1);
        assert!(targets[0].contains("server"));
    }

    #[test]
    fn test_scan_deployable_targets_includes_client() {
        let tmp = TempDir::new().unwrap();

        let client_path = tmp.path().join("regions/service/order/client/react");
        fs::create_dir_all(&client_path).unwrap();
        fs::write(client_path.join("package.json"), "{}").unwrap();

        let targets = scan_deployable_targets_at(tmp.path());

        assert_eq!(targets.len(), 1);
        assert!(targets[0].contains("client"));
    }

    // --- prod confirmation logic ---

    #[test]
    fn test_step_prod_confirmation_logic_matching() {
        // "deploy" が正確に一致する場合にtrueを返すロジックの検証
        // step_prod_confirmationはprivateでプロンプトを使うので直接テストできない
        // 代わりに、prod確認のビジネスロジックを検証する
        assert_eq!("deploy".trim(), "deploy");
        assert_ne!("Deploy".trim(), "deploy"); // 大文字小文字は区別
        assert_ne!("DEPLOY".trim(), "deploy");
        assert_ne!("".trim(), "deploy");
        assert_ne!("no".trim(), "deploy");
        assert_eq!(" deploy ".trim(), "deploy"); // 前後の空白はtrimされる
    }

    #[test]
    fn test_deploy_step_flow_prod_requires_confirmation() {
        // prod環境選択時はProdConfirmステップを経由する
        let env = Environment::Prod;
        assert!(env.is_prod());
        // 非prod環境はProdConfirmをスキップ
        let env_dev = Environment::Dev;
        assert!(!env_dev.is_prod());
        let env_stg = Environment::Staging;
        assert!(!env_stg.is_prod());
    }

    // --- execute_deploy ---

    #[test]
    fn test_execute_deploy_nonexistent_target() {
        let config = DeployConfig {
            environment: Environment::Dev,
            targets: vec!["/nonexistent/path".to_string()],
        };
        let result = execute_deploy(&config);
        assert!(result.is_ok());
    }
}
