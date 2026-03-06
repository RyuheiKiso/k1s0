/// マイグレーション管理 CLI。
///
/// ステートマシンパターンでフローを管理する。
/// 各ステップで Esc（None）→ 前のステップに戻る。
/// 最初のステップで Esc → メインメニュー復帰。
use anyhow::Result;

use crate::prompt::{self, ConfirmResult};

pub use k1s0_core::commands::migrate::*;

/// フローのステップ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// 操作選択
    Operation,

    // --- 新規作成フロー ---
    CreateTarget,
    CreateName,
    CreateConfirm,

    // --- 適用 (up) フロー ---
    UpTarget,
    UpRange,
    UpConnection,
    UpConfirm,

    // --- ロールバック (down) フロー ---
    DownTarget,
    DownRange,
    DownConnection,
    DownConfirm,

    // --- 状態確認フロー ---
    StatusTarget,

    // --- 修復フロー ---
    RepairTarget,
    RepairOperation,
    RepairConfirm,
}

/// マイグレーション管理コマンドを実行する。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、またはマイグレーション操作に失敗した場合にエラーを返す。
pub fn run() -> Result<()> {
    println!("\n--- マイグレーション管理 ---\n");

    let mut step = Step::Operation;
    let mut targets: Vec<MigrateTarget> = Vec::new();
    let mut selected_target_idx: usize = 0;
    let mut migration_name = String::new();
    let mut range = MigrateRange::All;
    let mut connection = DbConnection::LocalDev;
    let mut repair_op = RepairOperation::ClearDirty;

    loop {
        match step {
            // ================================================================
            // 操作選択
            // ================================================================
            Step::Operation => {
                match step_select_operation()? {
                    Some(op) => {
                        // 対象走査
                        targets = scan_migrate_targets(std::path::Path::new("."));
                        step = match op {
                            MigrateOperation::Create => Step::CreateTarget,
                            MigrateOperation::Up => Step::UpTarget,
                            MigrateOperation::Down => Step::DownTarget,
                            MigrateOperation::Status => Step::StatusTarget,
                            MigrateOperation::Repair => Step::RepairTarget,
                        };
                    }
                    None => return Ok(()), // メインメニューに戻る
                }
            }

            // ================================================================
            // 新規作成フロー
            // ================================================================
            Step::CreateTarget => {
                match step_select_target(&targets)? {
                    Some(idx) => {
                        selected_target_idx = idx;
                        step = Step::CreateName;
                    }
                    None => step = Step::Operation,
                }
            }
            Step::CreateName => {
                match step_input_migration_name()? {
                    Some(name) => {
                        migration_name = name;
                        step = Step::CreateConfirm;
                    }
                    None => step = Step::CreateTarget,
                }
            }
            Step::CreateConfirm => {
                let target = &targets[selected_target_idx];
                print_create_confirmation(target, &migration_name);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        let config = MigrateCreateConfig {
                            target: target.clone(),
                            migration_name: migration_name.clone(),
                        };
                        let (up, down) = create_migration(&config)?;
                        println!("\nマイグレーションファイルを作成しました:");
                        println!("  up:   {}", up.display());
                        println!("  down: {}", down.display());
                        return Ok(());
                    }
                    ConfirmResult::GoBack => step = Step::CreateName,
                    ConfirmResult::Cancel => {
                        println!("キャンセルしました。");
                        return Ok(());
                    }
                }
            }

            // ================================================================
            // 適用 (up) フロー
            // ================================================================
            Step::UpTarget => {
                match step_select_target(&targets)? {
                    Some(idx) => {
                        selected_target_idx = idx;
                        step = Step::UpRange;
                    }
                    None => step = Step::Operation,
                }
            }
            Step::UpRange => {
                match step_select_range()? {
                    Some(r) => {
                        range = r;
                        step = Step::UpConnection;
                    }
                    None => step = Step::UpTarget,
                }
            }
            Step::UpConnection => {
                match step_select_connection()? {
                    Some(c) => {
                        connection = c;
                        step = Step::UpConfirm;
                    }
                    None => step = Step::UpRange,
                }
            }
            Step::UpConfirm => {
                let target = &targets[selected_target_idx];
                print_up_confirmation(target, &range, &connection);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        // ツール確認
                        if !check_tool_installed(&target.language) {
                            println!(
                                "\n{} がインストールされていません。",
                                tool_name(&target.language)
                            );
                            match prompt::yes_no_prompt("インストールしますか？")? {
                                Some(true) => install_tool(&target.language)?,
                                _ => {
                                    println!("キャンセルしました。");
                                    return Ok(());
                                }
                            }
                        }
                        let config = MigrateUpConfig {
                            target: target.clone(),
                            range: range.clone(),
                            connection: connection.clone(),
                        };
                        execute_migrate_up(&config)?;
                        println!("\nマイグレーションの適用が完了しました。");
                        return Ok(());
                    }
                    ConfirmResult::GoBack => step = Step::UpConnection,
                    ConfirmResult::Cancel => {
                        println!("キャンセルしました。");
                        return Ok(());
                    }
                }
            }

            // ================================================================
            // ロールバック (down) フロー
            // ================================================================
            Step::DownTarget => {
                match step_select_target(&targets)? {
                    Some(idx) => {
                        selected_target_idx = idx;
                        step = Step::DownRange;
                    }
                    None => step = Step::Operation,
                }
            }
            Step::DownRange => {
                match step_select_range()? {
                    Some(r) => {
                        range = r;
                        step = Step::DownConnection;
                    }
                    None => step = Step::DownTarget,
                }
            }
            Step::DownConnection => {
                match step_select_connection()? {
                    Some(c) => {
                        connection = c;
                        step = Step::DownConfirm;
                    }
                    None => step = Step::DownRange,
                }
            }
            Step::DownConfirm => {
                let target = &targets[selected_target_idx];
                print_down_confirmation(target, &range, &connection);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        // ツール確認
                        if !check_tool_installed(&target.language) {
                            println!(
                                "\n{} がインストールされていません。",
                                tool_name(&target.language)
                            );
                            match prompt::yes_no_prompt("インストールしますか？")? {
                                Some(true) => install_tool(&target.language)?,
                                _ => {
                                    println!("キャンセルしました。");
                                    return Ok(());
                                }
                            }
                        }
                        let config = MigrateDownConfig {
                            target: target.clone(),
                            range: range.clone(),
                            connection: connection.clone(),
                        };
                        execute_migrate_down(&config)?;
                        println!("\nロールバックが完了しました。");
                        return Ok(());
                    }
                    ConfirmResult::GoBack => step = Step::DownConnection,
                    ConfirmResult::Cancel => {
                        println!("キャンセルしました。");
                        return Ok(());
                    }
                }
            }

            // ================================================================
            // 状態確認フロー
            // ================================================================
            Step::StatusTarget => {
                match step_select_target_or_all(&targets)? {
                    Some(None) => {
                        // 全対象
                        get_all_migration_status(&targets)?;
                        return Ok(());
                    }
                    Some(Some(idx)) => {
                        let target = &targets[idx];
                        let statuses =
                            get_migration_status(target, &DbConnection::LocalDev)?;
                        println!(
                            "\n=== {} ({}) ===",
                            target.service_name, target.tier
                        );
                        if statuses.is_empty() {
                            println!("  マイグレーションファイルはありません。");
                        } else {
                            for s in &statuses {
                                let mark = if s.applied { "[x]" } else { "[ ]" };
                                let at = s.applied_at.as_deref().unwrap_or("-");
                                println!(
                                    "  {} {:03}_{} (適用日時: {})",
                                    mark, s.number, s.description, at
                                );
                            }
                        }
                        return Ok(());
                    }
                    None => step = Step::Operation,
                }
            }

            // ================================================================
            // 修復フロー
            // ================================================================
            Step::RepairTarget => {
                match step_select_target(&targets)? {
                    Some(idx) => {
                        selected_target_idx = idx;
                        step = Step::RepairOperation;
                    }
                    None => step = Step::Operation,
                }
            }
            Step::RepairOperation => {
                match step_select_repair_operation()? {
                    Some(op) => {
                        repair_op = op;
                        step = Step::RepairConfirm;
                    }
                    None => step = Step::RepairTarget,
                }
            }
            Step::RepairConfirm => {
                let target = &targets[selected_target_idx];
                print_repair_confirmation(target, &repair_op);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        execute_repair(target, &repair_op, &DbConnection::LocalDev)?;
                        println!("\n修復が完了しました。");
                        return Ok(());
                    }
                    ConfirmResult::GoBack => step = Step::RepairOperation,
                    ConfirmResult::Cancel => {
                        println!("キャンセルしました。");
                        return Ok(());
                    }
                }
            }
        }
    }
}

// ============================================================================
// ステップ関数
// ============================================================================

/// 操作選択。
fn step_select_operation() -> Result<Option<MigrateOperation>> {
    let idx = prompt::select_prompt("操作を選択してください", OPERATION_LABELS)?;
    Ok(idx.map(|i| ALL_OPERATIONS[i].clone()))
}

/// サービス選択。
fn step_select_target(targets: &[MigrateTarget]) -> Result<Option<usize>> {
    if targets.is_empty() {
        println!("マイグレーション対象のサービスが見つかりません。");
        println!("（migrations/ ディレクトリを持つサービスが必要です）");
        return Ok(None);
    }

    let labels: Vec<String> = targets.iter().map(|t| t.display_label()).collect();
    let label_refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
    prompt::select_prompt("サービスを選択してください", &label_refs)
}

/// サービス選択（「すべて」オプション付き）。
/// 戻り値: Some(None) = すべて, Some(Some(idx)) = 個別, None = Esc
fn step_select_target_or_all(
    targets: &[MigrateTarget],
) -> Result<Option<Option<usize>>> {
    if targets.is_empty() {
        println!("マイグレーション対象のサービスが見つかりません。");
        return Ok(None);
    }

    let mut labels: Vec<String> = vec!["すべて".to_string()];
    for t in targets {
        labels.push(t.display_label());
    }
    let label_refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();

    match prompt::select_prompt("サービスを選択してください", &label_refs)? {
        None => Ok(None),
        Some(0) => Ok(Some(None)),         // 「すべて」
        Some(i) => Ok(Some(Some(i - 1))),  // 個別サービス
    }
}

/// マイグレーション名入力。
fn step_input_migration_name() -> Result<Option<String>> {
    let name = prompt::input_with_validation(
        "マイグレーション名を入力してください（英小文字・数字・アンダースコアのみ）",
        |input: &String| validate_migration_name(input),
    )?;
    if name.is_empty() {
        Ok(None)
    } else {
        Ok(Some(name))
    }
}

/// 適用範囲の選択。
fn step_select_range() -> Result<Option<MigrateRange>> {
    let items = &["すべて", "指定バージョンまで"];
    let idx = prompt::select_prompt("適用範囲を選択してください", items)?;

    match idx {
        None => Ok(None),
        Some(0) => Ok(Some(MigrateRange::All)),
        Some(1) => {
            let version_str = prompt::input_with_validation(
                "バージョン番号を入力してください",
                |input: &String| {
                    input
                        .parse::<u32>()
                        .map(|_| ())
                        .map_err(|_| "正の整数を入力してください".to_string())
                },
            )?;
            let version: u32 = version_str.parse().unwrap_or(1);
            Ok(Some(MigrateRange::UpTo(version)))
        }
        _ => unreachable!(),
    }
}

/// DB接続先の選択。
fn step_select_connection() -> Result<Option<DbConnection>> {
    let items = &["ローカル開発環境", "カスタム接続文字列"];
    let idx = prompt::select_prompt("DB接続先を選択してください", items)?;

    match idx {
        None => Ok(None),
        Some(0) => Ok(Some(DbConnection::LocalDev)),
        Some(1) => {
            let url = prompt::input_prompt_raw(
                "接続文字列を入力してください (例: postgresql://user:pass@host:5432/db)",
            )?;
            Ok(Some(DbConnection::Custom(url)))
        }
        _ => unreachable!(),
    }
}

/// 修復操作の選択。
fn step_select_repair_operation() -> Result<Option<RepairOperation>> {
    let items = &["ダーティフラグのクリア", "バージョンの強制設定"];
    let idx = prompt::select_prompt("修復操作を選択してください", items)?;

    match idx {
        None => Ok(None),
        Some(0) => Ok(Some(RepairOperation::ClearDirty)),
        Some(1) => {
            let version_str = prompt::input_with_validation(
                "強制設定するバージョン番号を入力してください",
                |input: &String| {
                    input
                        .parse::<u32>()
                        .map(|_| ())
                        .map_err(|_| "正の整数を入力してください".to_string())
                },
            )?;
            let version: u32 = version_str.parse().unwrap_or(0);
            Ok(Some(RepairOperation::ForceVersion(version)))
        }
        _ => unreachable!(),
    }
}

// ============================================================================
// 確認画面表示
// ============================================================================

/// 新規作成の確認内容を表示する。
fn print_create_confirmation(target: &MigrateTarget, migration_name: &str) {
    println!("\n[確認] 以下の内容でマイグレーションファイルを作成します。よろしいですか？");
    println!("    サービス: {}", target.display_label());
    println!("    名前:     {}", migration_name);
    println!(
        "    保存先:   {}/",
        target.migrations_dir.display()
    );
}

/// 適用 (up) の確認内容を表示する。
fn print_up_confirmation(target: &MigrateTarget, range: &MigrateRange, connection: &DbConnection) {
    println!("\n[確認] 以下の内容でマイグレーションを適用します。よろしいですか？");
    println!("    サービス: {}", target.display_label());
    println!("    範囲:     {}", format_range(range));
    println!("    接続先:   {}", format_connection(connection, &target.db_name));
}

/// ロールバック (down) の確認内容を表示する。
fn print_down_confirmation(
    target: &MigrateTarget,
    range: &MigrateRange,
    connection: &DbConnection,
) {
    println!("\n[確認] 以下の内容でロールバックを実行します。よろしいですか？");
    println!("    サービス: {}", target.display_label());
    println!("    範囲:     {}", format_range(range));
    println!("    接続先:   {}", format_connection(connection, &target.db_name));
}

/// 修復の確認内容を表示する。
fn print_repair_confirmation(target: &MigrateTarget, operation: &RepairOperation) {
    println!("\n[確認] 以下の内容で修復を実行します。よろしいですか？");
    println!("    サービス: {}", target.display_label());
    println!("    操作:     {}", format_repair_operation(operation));
}

// ============================================================================
// フォーマットヘルパー
// ============================================================================

/// 範囲の表示文字列を返す。
fn format_range(range: &MigrateRange) -> String {
    match range {
        MigrateRange::All => "すべて".to_string(),
        MigrateRange::UpTo(n) => format!("バージョン {} まで", n),
    }
}

/// 接続先の表示文字列を返す。
fn format_connection(connection: &DbConnection, db_name: &str) -> String {
    match connection {
        DbConnection::LocalDev => format!("ローカル開発環境 ({})", db_name),
        DbConnection::Custom(url) => url.clone(),
    }
}

/// 修復操作の表示文字列を返す。
fn format_repair_operation(operation: &RepairOperation) -> String {
    match operation {
        RepairOperation::ClearDirty => "ダーティフラグのクリア".to_string(),
        RepairOperation::ForceVersion(v) => format!("バージョン {} に強制設定", v),
    }
}
