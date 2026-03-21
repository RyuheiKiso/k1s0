/// ローカル開発コマンドの CLI UI。
///
/// ステートマシンパターンで対話フローを実装する。
use anyhow::Result;

use crate::prompt::{self, ConfirmResult};

pub use k1s0_core::commands::dev::*;

/// 操作選択のラベル。
const OPERATION_LABELS: &[&str] = &["起動", "停止", "状態確認", "ログ表示"];

/// 認証モード選択のラベル。
const AUTH_MODE_LABELS: &[&str] = &["スキップ（Keycloakなし）", "Keycloak（フル認証）"];

/// クリーンアップレベル選択のラベル。
const CLEANUP_LABELS: &[&str] = &[
    "コンテナのみ停止（ボリューム保持）",
    "コンテナとボリュームを削除",
];

/// ローカル開発フローのステップ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// 操作選択
    Operation,
    // --- 起動フロー ---
    /// サービス選択
    ServiceSelect,
    /// 認証モード選択
    AuthModeSelect,
    /// 起動確認
    ConfirmUp,
    // --- 停止フロー ---
    /// クリーンアップレベル選択
    CleanupLevel,
    /// 停止確認
    ConfirmDown,
    // --- ログフロー ---
    /// ログ対象選択
    LogsTarget,
}

/// ローカル開発コマンドを実行する。
///
/// # Errors
///
/// プロンプトの入出力またはコマンド実行に失敗した場合にエラーを返す。
pub fn run() -> Result<()> {
    println!("\n--- ローカル開発 ---\n");

    let mut step = Step::Operation;
    let mut selected_services: Vec<String> = Vec::new();
    let mut selected_service_names: Vec<String> = Vec::new();
    let mut auth_mode = AuthMode::Skip;
    let mut cleanup_level = CleanupLevel::ContainersOnly;
    let mut deps = detect::DetectedDependencies::default();

    loop {
        match step {
            Step::Operation => {
                match step_operation()? {
                    Some(op) => match op {
                        DevOperation::Up => step = Step::ServiceSelect,
                        DevOperation::Down => step = Step::CleanupLevel,
                        DevOperation::Status => {
                            execute_dev_status()?;
                            return Ok(());
                        }
                        DevOperation::Logs => step = Step::LogsTarget,
                    },
                    None => return Ok(()), // Esc → メインメニューに戻る
                }
            }

            // === 起動フロー ===
            Step::ServiceSelect => {
                match step_service_select()? {
                    Some((names, paths)) => {
                        if paths.is_empty() {
                            println!("開発対象のサービスが見つかりません。");
                            return Ok(());
                        }
                        selected_service_names = names;
                        selected_services = paths;

                        // 依存検出
                        let deps_list: Vec<detect::DetectedDependencies> = selected_services
                            .iter()
                            .map(|s| detect::detect_dependencies(s))
                            .collect::<Result<Vec<_>>>()?;
                        deps = detect::merge_dependencies(&deps_list);

                        step = Step::AuthModeSelect;
                    }
                    None => step = Step::Operation,
                }
            }

            Step::AuthModeSelect => match step_auth_mode()? {
                Some(mode) => {
                    auth_mode = mode;
                    step = Step::ConfirmUp;
                }
                None => step = Step::ServiceSelect,
            },

            Step::ConfirmUp => {
                print_up_confirmation(&selected_service_names, &deps, &auth_mode);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        let config = DevUpConfig {
                            services: selected_services.clone(),
                            auth_mode: auth_mode.clone(),
                        };
                        execute_dev_up(&config)?;
                        return Ok(());
                    }
                    ConfirmResult::GoBack => step = Step::AuthModeSelect,
                    ConfirmResult::Cancel => {
                        println!("キャンセルしました。");
                        return Ok(());
                    }
                }
            }

            // === 停止フロー ===
            Step::CleanupLevel => match step_cleanup_level()? {
                Some(level) => {
                    cleanup_level = level;
                    step = Step::ConfirmDown;
                }
                None => step = Step::Operation,
            },

            Step::ConfirmDown => {
                print_down_confirmation(&cleanup_level);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        let config = DevDownConfig {
                            cleanup: cleanup_level.clone(),
                        };
                        execute_dev_down(&config)?;
                        return Ok(());
                    }
                    ConfirmResult::GoBack => step = Step::CleanupLevel,
                    ConfirmResult::Cancel => {
                        println!("キャンセルしました。");
                        return Ok(());
                    }
                }
            }

            // === ログフロー ===
            Step::LogsTarget => match step_logs_target()? {
                Some(service) => {
                    execute_dev_logs(service.as_deref())?;
                    return Ok(());
                }
                None => step = Step::Operation,
            },
        }
    }
}

/// ステップ: 操作選択。
fn step_operation() -> Result<Option<DevOperation>> {
    let idx = prompt::select_prompt("操作を選択してください", OPERATION_LABELS)?;
    Ok(idx.map(|i| match i {
        0 => DevOperation::Up,
        1 => DevOperation::Down,
        2 => DevOperation::Status,
        3 => DevOperation::Logs,
        _ => unreachable!(),
    }))
}

/// ステップ: サービス選択。
///
/// 空選択の場合は再入力を促すループを使用する（再帰呼び出しを避ける）。
fn step_service_select() -> Result<Option<(Vec<String>, Vec<String>)>> {
    let targets = detect::scan_dev_targets(std::path::Path::new("."));
    if targets.is_empty() {
        return Ok(Some((Vec::new(), Vec::new())));
    }

    let mut items: Vec<&str> = vec!["すべて"];
    let display_names: Vec<String> = targets.iter().map(|(name, _)| name.clone()).collect();
    for name in &display_names {
        items.push(name.as_str());
    }

    loop {
        let selected = prompt::multi_select_prompt(
            "開発対象のサービスを選択してください（複数選択可）",
            &items,
        )?;

        match selected {
            // Esc が押された場合は前のステップに戻る
            None => return Ok(None),
            Some(indices) => {
                if indices.is_empty() {
                    // 未選択の場合は警告を出して再入力を促す
                    println!("少なくとも1つのサービスを選択してください。");
                } else if indices.contains(&0) {
                    // 「すべて」が選択された
                    let names: Vec<String> = targets.iter().map(|(n, _)| n.clone()).collect();
                    let paths: Vec<String> = targets.iter().map(|(_, p)| p.clone()).collect();
                    return Ok(Some((names, paths)));
                } else {
                    let names: Vec<String> =
                        indices.iter().map(|&i| targets[i - 1].0.clone()).collect();
                    let paths: Vec<String> =
                        indices.iter().map(|&i| targets[i - 1].1.clone()).collect();
                    return Ok(Some((names, paths)));
                }
            }
        }
    }
}

/// ステップ: 認証モード選択。
fn step_auth_mode() -> Result<Option<AuthMode>> {
    let idx = prompt::select_prompt("認証モードを選択してください", AUTH_MODE_LABELS)?;
    Ok(idx.map(|i| match i {
        0 => AuthMode::Skip,
        1 => AuthMode::Keycloak,
        _ => unreachable!(),
    }))
}

/// ステップ: クリーンアップレベル選択。
fn step_cleanup_level() -> Result<Option<CleanupLevel>> {
    let idx = prompt::select_prompt("クリーンアップレベルを選択してください", CLEANUP_LABELS)?;
    Ok(idx.map(|i| match i {
        0 => CleanupLevel::ContainersOnly,
        1 => CleanupLevel::ContainersAndVolumes,
        _ => unreachable!(),
    }))
}

/// ステップ: ログ表示対象選択。
#[allow(clippy::option_option)]
fn step_logs_target() -> Result<Option<Option<String>>> {
    let mut items = vec!["すべてのサービス"];

    // 稼働中のサービス名を取得（state.json から）
    let service_names: Vec<String> = if let Some(state) = state::load_state() {
        state
            .services
            .iter()
            .filter_map(|s| {
                std::path::Path::new(s)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(std::string::ToString::to_string)
            })
            .collect()
    } else {
        // docker compose のサービス名を静的に提示
        vec![
            "postgres".to_string(),
            "kafka".to_string(),
            "redis".to_string(),
        ]
    };

    let name_refs: Vec<&str> = service_names
        .iter()
        .map(std::string::String::as_str)
        .collect();
    for name in &name_refs {
        items.push(name);
    }

    let idx = prompt::select_prompt("ログ表示対象を選択してください", &items)?;
    match idx {
        None => Ok(None),
        Some(0) => Ok(Some(None)), // すべて
        Some(i) => Ok(Some(Some(service_names[i - 1].clone()))),
    }
}

/// 起動確認の表示。
fn print_up_confirmation(
    service_names: &[String],
    deps: &detect::DetectedDependencies,
    auth_mode: &AuthMode,
) {
    println!("\n[確認] 以下の構成で起動します。よろしいですか？");
    for name in service_names {
        println!("    対象サービス: {name}");
    }
    println!("    検出された依存:");
    if !deps.databases.is_empty() {
        for db in &deps.databases {
            println!("      PostgreSQL  {}", db.name);
        }
    }
    if deps.has_kafka {
        println!("      Kafka       {} topics", deps.kafka_topics.len());
    }
    if deps.has_redis {
        println!("      Redis       cache");
    }
    if deps.has_redis_session {
        println!("      Redis       session");
    }
    let auth_label = match auth_mode {
        AuthMode::Skip => "スキップ",
        AuthMode::Keycloak => "Keycloak",
    };
    println!("    認証モード: {auth_label}");
}

/// 停止確認の表示。
fn print_down_confirmation(cleanup: &CleanupLevel) {
    println!("\n[確認] ローカル開発環境を停止します。よろしいですか？");
    match cleanup {
        CleanupLevel::ContainersOnly => {
            println!("    クリーンアップ: コンテナのみ停止（ボリューム保持）");
        }
        CleanupLevel::ContainersAndVolumes => {
            println!("    クリーンアップ: コンテナとボリュームを削除");
        }
    }
}
