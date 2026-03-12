/// テンプレートマイグレーション CLI。
///
/// ステートマシンパターンでフローを管理する。
/// 各ステップで Esc（None）→ 前のステップに戻る。
/// 最初のステップで Esc → メインメニュー復帰。
use anyhow::Result;

use crate::prompt::{self, ConfirmResult};
use k1s0_core::commands::template_migrate::{
    differ, executor,
    types::{ChangeType, MergeResult, MigrationPlan, MigrationTarget},
};

/// フローのステップ。
enum State {
    /// 対象選択
    SelectTarget,
    /// プレビュー
    Preview(MigrationTarget),
    /// コンフリクト解決
    ConflictResolution(MigrationPlan),
    /// 確認
    Confirm(MigrationPlan),
    /// 実行
    Execute(MigrationPlan),
    /// 完了
    Done,
}

/// テンプレートマイグレーションコマンドを実行する。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、またはマイグレーション操作に失敗した場合にエラーを返す。
pub fn run() -> Result<()> {
    println!("\n--- テンプレートマイグレーション ---\n");

    let mut state = State::SelectTarget;

    loop {
        match state {
            State::SelectTarget => match step_select_target()? {
                Some(target) => state = State::Preview(target),
                None => return Ok(()),
            },
            State::Preview(target) => match step_preview(&target)? {
                Some(plan) => {
                    if plan.has_conflicts() {
                        state = State::ConflictResolution(plan);
                    } else {
                        state = State::Confirm(plan);
                    }
                }
                None => state = State::SelectTarget,
            },
            State::ConflictResolution(mut plan) => match step_resolve_conflicts(&mut plan)? {
                Some(()) => state = State::Confirm(plan),
                None => state = State::SelectTarget,
            },
            State::Confirm(plan) => match step_confirm(&plan)? {
                Some(()) => state = State::Execute(plan),
                None => state = State::SelectTarget,
            },
            State::Execute(plan) => {
                step_execute(&plan)?;
                state = State::Done;
            }
            State::Done => {
                println!("\nマイグレーションが完了しました。");
                return Ok(());
            }
        }
    }
}

// ============================================================================
// ステップ関数
// ============================================================================

/// 対象選択。.k1s0-template.yaml を走査してリストを表示する。
fn step_select_target() -> Result<Option<MigrationTarget>> {
    let root = std::path::Path::new(".");
    let targets = k1s0_core::commands::template_migrate::scanner::scan_targets(root)?;

    if targets.is_empty() {
        println!("テンプレートマイグレーション対象が見つかりません。");
        println!("（.k1s0-template.yaml を持つプロジェクトが必要です）");
        return Ok(None);
    }

    let labels: Vec<String> = targets
        .iter()
        .map(|(path, manifest)| {
            format!(
                "{} ({} / {} v{})",
                path.display(),
                manifest.template_type,
                manifest.language,
                manifest.version,
            )
        })
        .collect();
    let label_refs: Vec<&str> = labels.iter().map(String::as_str).collect();

    match prompt::select_prompt("マイグレーション対象を選択してください", &label_refs)?
    {
        Some(idx) => {
            let (path, manifest) = targets.into_iter().nth(idx).unwrap();
            // TODO: 実際にはレジストリから最新バージョンを取得する
            let available_version = format_next_version(&manifest.version);
            Ok(Some(MigrationTarget {
                path,
                manifest,
                available_version,
            }))
        }
        None => Ok(None),
    }
}

/// プレビュー。差分計画を生成して表示する。
fn step_preview(target: &MigrationTarget) -> Result<Option<MigrationPlan>> {
    println!("\n現在のバージョン: {}", target.manifest.version);
    println!("利用可能バージョン: {}", target.available_version);

    // TODO: 実際にはテンプレートレジストリから新旧テンプレートを取得して差分を計算する
    // ここではプレースホルダーとして空の変更リストを返す
    let plan = MigrationPlan {
        target: target.clone(),
        changes: Vec::new(),
    };

    if plan.changes.is_empty() {
        println!("\n変更はありません。");
    } else {
        println!("\n--- 変更プレビュー ---");
        for change in &plan.changes {
            let type_label = match &change.change_type {
                ChangeType::Added => "[追加]",
                ChangeType::Modified => "[変更]",
                ChangeType::Deleted => "[削除]",
            };
            println!("  {} {}", type_label, change.path.display());

            if let MergeResult::Clean(new_content) = &change.merge_result {
                // 既存ファイルの差分を表示
                let old_path = target.path.join(&change.path);
                if old_path.exists() {
                    let old_content = std::fs::read_to_string(&old_path).unwrap_or_default();
                    let diff = differ::format_diff(&old_content, new_content);
                    for line in diff.lines().take(20) {
                        println!("    {line}");
                    }
                }
            }
        }
    }

    match prompt::yes_no_prompt("マイグレーションを続行しますか？")? {
        Some(true) => Ok(Some(plan)),
        _ => Ok(None),
    }
}

/// コンフリクト解決。各コンフリクトに対して解決方針を選択する。
fn step_resolve_conflicts(plan: &mut MigrationPlan) -> Result<Option<()>> {
    println!("\n--- コンフリクト解決 ---\n");

    let conflict_items = &["テンプレートを優先", "ユーザーの変更を優先", "スキップ"];

    for change in &mut plan.changes {
        if let MergeResult::Conflict(hunks) = &change.merge_result {
            println!("コンフリクト: {}", change.path.display());

            for (i, hunk) in hunks.iter().enumerate() {
                println!("\n  ハンク #{}", i + 1);
                println!("  --- テンプレート ---");
                for line in hunk.theirs.lines().take(10) {
                    println!("  + {line}");
                }
                println!("  --- ユーザー ---");
                for line in hunk.ours.lines().take(10) {
                    println!("  - {line}");
                }
            }

            match prompt::select_prompt("解決方針を選択してください", conflict_items)?
            {
                Some(0) => {
                    // テンプレート優先
                    let content = hunks.first().map(|h| h.theirs.clone()).unwrap_or_default();
                    change.merge_result = MergeResult::Clean(content);
                }
                Some(1) => {
                    // ユーザー優先
                    let content = hunks.first().map(|h| h.ours.clone()).unwrap_or_default();
                    change.merge_result = MergeResult::Clean(content);
                }
                Some(2) => {
                    // スキップ
                    change.merge_result = MergeResult::NoChange;
                }
                None => return Ok(None),
                _ => unreachable!(),
            }
        }
    }

    Ok(Some(()))
}

/// 確認。変更サマリーを表示して実行を確認する。
fn step_confirm(plan: &MigrationPlan) -> Result<Option<()>> {
    let added = plan
        .changes
        .iter()
        .filter(|c| matches!(c.change_type, ChangeType::Added))
        .count();
    let modified = plan
        .changes
        .iter()
        .filter(|c| matches!(c.change_type, ChangeType::Modified))
        .count();
    let deleted = plan
        .changes
        .iter()
        .filter(|c| matches!(c.change_type, ChangeType::Deleted))
        .count();

    println!("\n[確認] 以下の内容でテンプレートマイグレーションを実行します。よろしいですか？");
    println!("    対象:     {}", plan.target.path.display());
    println!(
        "    バージョン: {} -> {}",
        plan.target.manifest.version, plan.target.available_version
    );
    println!("    追加: {added} 件, 変更: {modified} 件, 削除: {deleted} 件");

    match prompt::confirm_prompt()? {
        ConfirmResult::Yes => Ok(Some(())),
        ConfirmResult::GoBack | ConfirmResult::Cancel => Ok(None),
    }
}

/// 実行。バックアップを作成してマイグレーションを適用する。
fn step_execute(plan: &MigrationPlan) -> Result<()> {
    println!("\nマイグレーションを実行中...");
    executor::execute_migration(plan)?;
    println!("バックアップを作成しました。");
    Ok(())
}

// ============================================================================
// ヘルパー
// ============================================================================

/// バージョン文字列から次のバージョンを推測する（簡易実装）。
fn format_next_version(current: &str) -> String {
    // セマンティックバージョニング (x.y.z) の場合、パッチバージョンをインクリメント
    let parts: Vec<&str> = current.split('.').collect();
    if parts.len() == 3 {
        if let Ok(patch) = parts[2].parse::<u32>() {
            return format!("{}.{}.{}", parts[0], parts[1], patch + 1);
        }
    }
    // その他の形式の場合はそのまま返す
    format!("{current}-next")
}
