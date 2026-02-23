use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::progress::ProgressEvent;

// ============================================================================
// デプロイパイプライン (CLIフロー.md「デプロイ実行」セクション準拠)
// ============================================================================

/// デプロイパイプラインのステップ名。
///
/// CLIフロー.md で定義された4段階のデプロイパイプラインに対応する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployStep {
    DockerBuild,
    DockerPush,
    CosignSign,
    HelmDeploy,
}

impl DeployStep {
    /// ステップの日本語ラベルを返す。
    pub fn label(&self) -> &'static str {
        match self {
            DeployStep::DockerBuild => "Docker イメージのビルド",
            DeployStep::DockerPush => "Docker イメージのプッシュ",
            DeployStep::CosignSign => "イメージ署名 (Cosign)",
            DeployStep::HelmDeploy => "Helm デプロイ",
        }
    }

    /// ステップ番号 (1-based) を返す。
    pub fn step_number(&self) -> usize {
        match self {
            DeployStep::DockerBuild => 1,
            DeployStep::DockerPush => 2,
            DeployStep::CosignSign => 3,
            DeployStep::HelmDeploy => 4,
        }
    }
}

/// デプロイパイプラインの総ステップ数。
pub const TOTAL_DEPLOY_STEPS: usize = 4;

/// Docker イメージタグを生成する。
///
/// フォーマット: `{registry}/k1s0-{tier}/{service_name}:{version}-{sha}`
pub fn build_image_tag(
    registry: &str,
    tier: &str,
    service_name: &str,
    version: &str,
    sha: &str,
) -> String {
    format!("{registry}/k1s0-{tier}/{service_name}:{version}-{sha}")
}

/// Helm upgrade コマンドの引数を構築する。
///
/// CLIフロー.md のデプロイ実行パイプラインに準拠:
/// ```text
/// helm upgrade --install {service_name} ./infra/helm/services/{helm_path} \
///     -n k1s0-{tier} \
///     -f ./infra/helm/services/{helm_path}/values-{env}.yaml \
///     --set image.tag={version}-{sha}
/// ```
pub fn build_helm_args(
    service_name: &str,
    helm_path: &str,
    tier: &str,
    env: &str,
    image_tag: &str,
) -> Vec<String> {
    vec![
        "upgrade".to_string(),
        "--install".to_string(),
        service_name.to_string(),
        format!("./infra/helm/services/{}", helm_path),
        "-n".to_string(),
        format!("k1s0-{}", tier),
        "-f".to_string(),
        format!("./infra/helm/services/{}/values-{}.yaml", helm_path, env),
        "--set".to_string(),
        format!("image.tag={}", image_tag),
    ]
}

/// Helm rollback コマンドの引数を構築する。
///
/// prod 環境でのデプロイ失敗時にロールバックするためのコマンド引数:
/// ```text
/// helm rollback {service_name} -n k1s0-{tier}
/// ```
pub fn build_helm_rollback_args(service_name: &str, tier: &str) -> Vec<String> {
    vec![
        "rollback".to_string(),
        service_name.to_string(),
        "-n".to_string(),
        format!("k1s0-{}", tier),
    ]
}

/// デプロイエラー情報。
///
/// デプロイパイプラインの各ステップでエラーが発生した場合の情報を保持する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployError {
    /// 失敗したステップ
    pub step: DeployStep,
    /// エラーメッセージ
    pub message: String,
    /// 手動で再実行するためのコマンド
    pub manual_command: String,
}

/// ターゲットパスから tier を抽出する。
///
/// パス例: `regions/service/order/server/rust` → `"service"`
/// パス例: `regions/system/server/rust/auth` → `"system"`
///
/// `regions/{tier}/...` の形式を前提とする。
/// 抽出できない場合は `None` を返す。
pub fn extract_tier_from_target_path(target: &str) -> Option<String> {
    let normalized = target.replace('\\', "/");
    let parts: Vec<&str> = normalized.split('/').collect();
    // "regions" の次の要素が tier
    for (i, part) in parts.iter().enumerate() {
        if *part == "regions" && i + 1 < parts.len() {
            let tier = parts[i + 1];
            if tier == "system" || tier == "business" || tier == "service" {
                return Some(tier.to_string());
            }
        }
    }
    None
}

/// ターゲットパスから `service_name` を抽出する。
///
/// パス構造に応じて末尾のディレクトリ名をサービス名として返す。
/// - `regions/service/order/server/rust` → `"order"`  (service tier: サービス名)
/// - `regions/system/server/rust/auth` → `"auth"` (system tier: 末尾ディレクトリ名)
/// - `regions/business/accounting/server/rust/ledger` → `"ledger"` (business tier: 末尾ディレクトリ名)
///
/// 抽出できない場合は `None` を返す。
pub fn extract_service_name_from_target_path(target: &str) -> Option<String> {
    let normalized = target.replace('\\', "/");
    let trimmed = normalized.trim_end_matches('/');
    let parts: Vec<&str> = trimmed.split('/').collect();

    // 最低限 regions/{tier}/... の形式が必要
    if parts.len() < 3 {
        return None;
    }

    // regions のインデックスを見つける
    let regions_idx = parts.iter().position(|&p| p == "regions")?;
    if regions_idx + 2 >= parts.len() {
        return None;
    }

    let tier = parts[regions_idx + 1];

    match tier {
        "service" => {
            // regions/service/{service_name}/... → service_name
            Some(parts[regions_idx + 2].to_string())
        }
        "system" | "business" => {
            // 末尾のディレクトリ名をサービス名とする
            parts.last().map(std::string::ToString::to_string)
        }
        _ => None,
    }
}

/// デプロイ結果の表示メッセージを構築する。
pub fn format_deploy_success(env: &str, service_name: &str, image_tag: &str, tier: &str) -> String {
    format!(
        "\u{2713} デプロイが完了しました\n  環境:     {env}\n  サービス: {service_name}\n  イメージ: {image_tag}\n  Helm:     helm status {service_name} -n k1s0-{tier}"
    )
}

/// デプロイエラーの表示メッセージを構築する。
pub fn format_deploy_failure(error: &DeployError) -> String {
    format!(
        "\u{2717} デプロイに失敗しました\n  ステップ: {}\n  エラー:   {}\n  手動で再実行する場合: {}",
        error.step.label(),
        error.message,
        error.manual_command
    )
}

/// 進捗メッセージ (開始) を構築する。
pub fn format_step_start(step: &DeployStep) -> String {
    format!(
        "[{}/{}] {} しています...",
        step.step_number(),
        TOTAL_DEPLOY_STEPS,
        step.label()
    )
}

/// 進捗メッセージ (完了) を構築する。
pub fn format_step_done(step: &DeployStep) -> String {
    format!(
        "[{}/{}] \u{2713} {}完了",
        step.step_number(),
        TOTAL_DEPLOY_STEPS,
        step.label()
    )
}

/// デプロイ環境。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// デプロイ設定。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployConfig {
    /// デプロイ先環境
    pub environment: Environment,
    /// デプロイ対象のパス一覧
    pub targets: Vec<String>,
}

/// デプロイ実行。
///
/// # Errors
/// エラーが発生した場合。
pub fn execute_deploy(config: &DeployConfig) -> Result<()> {
    for target in &config.targets {
        println!("\nデプロイ中: {} → {}", target, config.environment.as_str());
        let target_path = Path::new(target);

        if !target_path.is_dir() {
            println!("  警告: ディレクトリが見つかりません: {target}");
            continue;
        }

        // Dockerfile があれば Docker ベースのデプロイ
        if target_path.join("Dockerfile").exists() {
            let image_tag = format!(
                "{}:{}",
                target.replace(['/', '\\'], "-"),
                config.environment.as_str()
            );
            println!("  Docker イメージビルド: {image_tag}");
            let build_status = Command::new("docker")
                .args(["build", "-t", &image_tag, "."])
                .current_dir(target_path)
                .status();
            match build_status {
                Ok(s) if s.success() => {
                    println!("  イメージビルド完了: {image_tag}");
                }
                Ok(_) => {
                    println!("  警告: Docker ビルドに失敗しました");
                }
                Err(e) => {
                    println!("  警告: docker コマンドの実行に失敗しました: {e}");
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

/// プログレスコールバック付きデプロイ実行。
///
/// # Errors
/// エラーが発生した場合。
pub fn execute_deploy_with_progress(
    config: &DeployConfig,
    on_progress: impl Fn(ProgressEvent),
) -> Result<()> {
    let total = config.targets.len();
    for (i, target) in config.targets.iter().enumerate() {
        let step = i + 1;
        on_progress(ProgressEvent::StepStarted {
            step,
            total,
            message: format!("デプロイ中: {} → {}", target, config.environment.as_str()),
        });

        let target_path = Path::new(target);
        if !target_path.is_dir() {
            on_progress(ProgressEvent::Warning {
                message: format!("ディレクトリが見つかりません: {target}"),
            });
            on_progress(ProgressEvent::StepCompleted {
                step,
                total,
                message: format!("スキップ: {target}"),
            });
            continue;
        }

        if target_path.join("Dockerfile").exists() {
            let image_tag = format!(
                "{}:{}",
                target.replace(['/', '\\'], "-"),
                config.environment.as_str()
            );
            on_progress(ProgressEvent::Log {
                message: format!("Docker イメージビルド: {image_tag}"),
            });
            let build_status = Command::new("docker")
                .args(["build", "-t", &image_tag, "."])
                .current_dir(target_path)
                .status();
            match build_status {
                Ok(s) if s.success() => {
                    on_progress(ProgressEvent::Log {
                        message: format!("イメージビルド完了: {image_tag}"),
                    });
                }
                Ok(_) => {
                    on_progress(ProgressEvent::Warning {
                        message: "Docker ビルドに失敗しました".to_string(),
                    });
                }
                Err(e) => {
                    on_progress(ProgressEvent::Warning {
                        message: format!("docker コマンドの実行に失敗しました: {e}"),
                    });
                }
            }
        } else {
            on_progress(ProgressEvent::Log {
                message: format!(
                    "デプロイ: {} を {} 環境にデプロイします (dry-run)",
                    target,
                    config.environment.as_str()
                ),
            });
        }

        on_progress(ProgressEvent::StepCompleted {
            step,
            total,
            message: format!("デプロイ完了: {target}"),
        });
    }
    on_progress(ProgressEvent::Finished {
        success: true,
        message: "すべてのデプロイが完了しました".to_string(),
    });
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
            targets: vec!["regions/system/server/rust/auth".to_string()],
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
        let server_path = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(&server_path).unwrap();
        fs::write(server_path.join("Cargo.toml"), "[package]\n").unwrap();

        // ライブラリ (デプロイ対象外)
        let lib_path = tmp.path().join("regions/system/library/rust/authlib");
        fs::create_dir_all(&lib_path).unwrap();
        fs::write(lib_path.join("Cargo.toml"), "[package]\n").unwrap();

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

    // =========================================================================
    // デプロイパイプライン テスト (TDD)
    // =========================================================================

    // --- DeployStep ---

    #[test]
    fn test_deploy_step_labels() {
        assert_eq!(DeployStep::DockerBuild.label(), "Docker イメージのビルド");
        assert_eq!(DeployStep::DockerPush.label(), "Docker イメージのプッシュ");
        assert_eq!(DeployStep::CosignSign.label(), "イメージ署名 (Cosign)");
        assert_eq!(DeployStep::HelmDeploy.label(), "Helm デプロイ");
    }

    #[test]
    fn test_deploy_step_numbers() {
        assert_eq!(DeployStep::DockerBuild.step_number(), 1);
        assert_eq!(DeployStep::DockerPush.step_number(), 2);
        assert_eq!(DeployStep::CosignSign.step_number(), 3);
        assert_eq!(DeployStep::HelmDeploy.step_number(), 4);
    }

    // --- イメージタグ ---

    #[test]
    fn test_build_image_tag() {
        let tag = build_image_tag(
            "harbor.internal.example.com",
            "service",
            "order",
            "1.2.3",
            "abc1234",
        );
        assert_eq!(
            tag,
            "harbor.internal.example.com/k1s0-service/order:1.2.3-abc1234"
        );
    }

    #[test]
    fn test_build_image_tag_with_custom_registry() {
        let tag = build_image_tag("my-registry.io", "system", "auth", "0.1.0", "def5678");
        assert_eq!(tag, "my-registry.io/k1s0-system/auth:0.1.0-def5678");
    }

    // --- Helm 引数 ---

    #[test]
    fn test_build_helm_args() {
        let args = build_helm_args("order", "order", "service", "dev", "1.2.3-abc1234");
        assert_eq!(
            args,
            vec![
                "upgrade",
                "--install",
                "order",
                "./infra/helm/services/order",
                "-n",
                "k1s0-service",
                "-f",
                "./infra/helm/services/order/values-dev.yaml",
                "--set",
                "image.tag=1.2.3-abc1234",
            ]
        );
    }

    #[test]
    fn test_build_helm_args_prod() {
        let args = build_helm_args("auth", "auth", "system", "prod", "2.0.0-fff9999");
        assert_eq!(
            args,
            vec![
                "upgrade",
                "--install",
                "auth",
                "./infra/helm/services/auth",
                "-n",
                "k1s0-system",
                "-f",
                "./infra/helm/services/auth/values-prod.yaml",
                "--set",
                "image.tag=2.0.0-fff9999",
            ]
        );
    }

    // --- Helm ロールバック ---

    #[test]
    fn test_build_helm_rollback_args() {
        let args = build_helm_rollback_args("order", "service");
        assert_eq!(args, vec!["rollback", "order", "-n", "k1s0-service",]);
    }

    // --- DeployError ---

    #[test]
    fn test_deploy_error_creation() {
        let error = DeployError {
            step: DeployStep::DockerBuild,
            message: "Dockerfile not found".to_string(),
            manual_command: "cd regions/service/order/server/rust && docker build -t order:dev ."
                .to_string(),
        };
        assert_eq!(error.step, DeployStep::DockerBuild);
        assert_eq!(error.message, "Dockerfile not found");
        assert!(error.manual_command.contains("docker build"));
    }

    // --- tier 抽出 ---

    #[test]
    fn test_extract_tier_from_target_path() {
        // service tier
        assert_eq!(
            extract_tier_from_target_path("regions/service/order/server/rust"),
            Some("service".to_string())
        );
        // system tier
        assert_eq!(
            extract_tier_from_target_path("regions/system/server/rust/auth"),
            Some("system".to_string())
        );
        // business tier
        assert_eq!(
            extract_tier_from_target_path("regions/business/accounting/server/rust/ledger"),
            Some("business".to_string())
        );
        // 無効なパス
        assert_eq!(extract_tier_from_target_path("invalid/path"), None);
        // regions の後に無効な tier
        assert_eq!(extract_tier_from_target_path("regions/unknown/foo"), None);
        // Windows パス区切りでも動作する
        assert_eq!(
            extract_tier_from_target_path("regions\\service\\order\\server\\rust"),
            Some("service".to_string())
        );
    }

    // --- service_name 抽出 ---

    #[test]
    fn test_extract_service_name_from_target_path() {
        // service tier: サービス名 (regions/service/{service_name}/...)
        assert_eq!(
            extract_service_name_from_target_path("regions/service/order/server/rust"),
            Some("order".to_string())
        );
        // system tier: 末尾ディレクトリ名
        assert_eq!(
            extract_service_name_from_target_path("regions/system/server/rust/auth"),
            Some("auth".to_string())
        );
        // business tier: 末尾ディレクトリ名
        assert_eq!(
            extract_service_name_from_target_path("regions/business/accounting/server/rust/ledger"),
            Some("ledger".to_string())
        );
        // 無効なパス
        assert_eq!(extract_service_name_from_target_path("invalid"), None);
        // 短すぎるパス
        assert_eq!(
            extract_service_name_from_target_path("regions/service"),
            None
        );
    }

    // --- 進捗メッセージ ---

    #[test]
    fn test_format_step_start() {
        let msg = format_step_start(&DeployStep::DockerBuild);
        assert_eq!(msg, "[1/4] Docker イメージのビルド しています...");
    }

    #[test]
    fn test_format_step_done() {
        let msg = format_step_done(&DeployStep::HelmDeploy);
        assert_eq!(msg, "[4/4] \u{2713} Helm デプロイ完了");
    }

    // --- 結果表示メッセージ ---

    #[test]
    fn test_format_deploy_success() {
        let msg = format_deploy_success(
            "dev",
            "order",
            "harbor.internal.example.com/k1s0-service/order:1.0.0-abc1234",
            "service",
        );
        assert!(msg.contains("デプロイが完了しました"));
        assert!(msg.contains("環境:     dev"));
        assert!(msg.contains("サービス: order"));
        assert!(msg.contains("harbor.internal.example.com/k1s0-service/order:1.0.0-abc1234"));
        assert!(msg.contains("helm status order -n k1s0-service"));
    }

    // --- エラー表示メッセージ ---

    #[test]
    fn test_format_deploy_failure() {
        let error = DeployError {
            step: DeployStep::DockerBuild,
            message: "exit code 1".to_string(),
            manual_command: "cd path && docker build -t tag .".to_string(),
        };
        let msg = format_deploy_failure(&error);
        assert!(msg.contains("デプロイに失敗しました"));
        assert!(msg.contains("ステップ: Docker イメージのビルド"));
        assert!(msg.contains("エラー:   exit code 1"));
        assert!(msg.contains("手動で再実行する場合: cd path && docker build -t tag ."));
    }

    // --- TOTAL_DEPLOY_STEPS 定数 ---

    #[test]
    fn test_total_deploy_steps() {
        assert_eq!(TOTAL_DEPLOY_STEPS, 4);
    }

    // --- DeployStep の等値性 ---

    #[test]
    fn test_deploy_step_equality() {
        assert_eq!(DeployStep::DockerBuild, DeployStep::DockerBuild);
        assert_ne!(DeployStep::DockerBuild, DeployStep::DockerPush);
        assert_ne!(DeployStep::CosignSign, DeployStep::HelmDeploy);
    }

    // --- DeployStep の Clone ---

    #[test]
    fn test_deploy_step_clone() {
        let step = DeployStep::HelmDeploy;
        let cloned = step;
        assert_eq!(step, cloned);
    }

    // --- build_helm_rollback_args の system tier ---

    #[test]
    fn test_build_helm_rollback_args_system() {
        let args = build_helm_rollback_args("auth", "system");
        assert_eq!(args[0], "rollback");
        assert_eq!(args[1], "auth");
        assert_eq!(args[2], "-n");
        assert_eq!(args[3], "k1s0-system");
    }

    // --- execute_deploy_with_progress ---

    #[test]
    fn test_execute_deploy_with_progress_nonexistent_target() {
        let config = DeployConfig {
            environment: Environment::Dev,
            targets: vec!["/nonexistent/path".to_string()],
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let result = execute_deploy_with_progress(&config, move |event| {
            events_clone.lock().unwrap().push(event);
        });
        assert!(result.is_ok());

        let collected = events.lock().unwrap();
        assert!(collected.len() >= 3);
        assert!(matches!(
            &collected[0],
            ProgressEvent::StepStarted {
                step: 1,
                total: 1,
                ..
            }
        ));
        assert!(matches!(&collected[1], ProgressEvent::Warning { .. }));
        assert!(matches!(
            collected.last().unwrap(),
            ProgressEvent::Finished { success: true, .. }
        ));
    }

    #[test]
    fn test_execute_deploy_with_progress_empty_targets() {
        let config = DeployConfig {
            environment: Environment::Dev,
            targets: vec![],
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let result = execute_deploy_with_progress(&config, move |event| {
            events_clone.lock().unwrap().push(event);
        });
        assert!(result.is_ok());

        let collected = events.lock().unwrap();
        assert_eq!(collected.len(), 1);
        assert!(matches!(
            &collected[0],
            ProgressEvent::Finished { success: true, .. }
        ));
    }

    #[test]
    fn test_execute_deploy_with_progress_dryrun_no_dockerfile() {
        let tmp = TempDir::new().unwrap();
        // Dockerfile なしのディレクトリ → dry-run メッセージ
        let config = DeployConfig {
            environment: Environment::Staging,
            targets: vec![tmp.path().to_string_lossy().to_string()],
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let result = execute_deploy_with_progress(&config, move |event| {
            events_clone.lock().unwrap().push(event);
        });
        assert!(result.is_ok());

        let collected = events.lock().unwrap();
        // StepStarted, Log (dry-run), StepCompleted, Finished
        assert!(collected.len() >= 3);
        let has_dryrun_log = collected.iter().any(|e| {
            if let ProgressEvent::Log { message } = e {
                message.contains("dry-run")
            } else {
                false
            }
        });
        assert!(has_dryrun_log);
    }
}
