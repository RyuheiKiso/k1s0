/// テンプレートマイグレーション CLI。
use std::path::Path;

use anyhow::Result;

use crate::prompt::{self, ConfirmResult};
use k1s0_core::commands::template_migrate::{
    differ, executor, planner, rollback, scanner,
    types::{ChangeType, MergeResult, MigrationPlan, MigrationTarget},
};

enum TemplateMigrationOperation {
    Execute,
    Rollback,
}

/// テンプレートマイグレーションコマンドを実行する。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、またはマイグレーション操作に失敗した場合にエラーを返す。
pub fn run() -> Result<()> {
    println!("\n--- テンプレートマイグレーション ---\n");

    match step_select_operation()? {
        Some(TemplateMigrationOperation::Execute) => run_execute_flow(),
        Some(TemplateMigrationOperation::Rollback) => run_rollback_flow(),
        None => Ok(()),
    }
}

fn run_execute_flow() -> Result<()> {
    let targets = scanner::scan_targets(Path::new("."))?;
    let Some(target) = step_select_target(&targets)? else {
        return Ok(());
    };

    let Some(mut plan) = step_preview(&target)? else {
        return Ok(());
    };

    if plan.has_conflicts() {
        let Some(()) = step_resolve_conflicts(&mut plan)? else {
            return Ok(());
        };
    }

    let Some(()) = step_confirm(&plan)? else {
        return Ok(());
    };

    println!("\nマイグレーションを実行中...");
    executor::execute_migration(&plan)?;
    println!("マイグレーションが完了しました。");
    Ok(())
}

fn run_rollback_flow() -> Result<()> {
    let targets = scanner::scan_targets(Path::new("."))?;
    let Some(target) = step_select_target(&targets)? else {
        return Ok(());
    };

    let backups = rollback::list_backups(&target.path)?;
    if backups.is_empty() {
        println!("利用可能なバックアップがありません。");
        return Ok(());
    }

    let Some(backup_id) = step_select_backup(&backups)? else {
        return Ok(());
    };

    println!("\n[確認] 以下のバックアップへロールバックします。よろしいですか？");
    println!("    対象:       {}", target.path.display());
    println!("    バックアップ: {backup_id}");
    match prompt::confirm_prompt()? {
        ConfirmResult::Yes => {
            rollback::rollback(
                &target.path,
                &rollback::backup_dir(&target.path, &backup_id),
            )?;
            println!("ロールバックが完了しました。");
        }
        ConfirmResult::GoBack | ConfirmResult::Cancel => {}
    }

    Ok(())
}

fn step_select_operation() -> Result<Option<TemplateMigrationOperation>> {
    let items = &["マイグレーション実行", "ロールバック"];
    match prompt::select_prompt("マイグレーション操作を選択してください", items)?
    {
        Some(0) => Ok(Some(TemplateMigrationOperation::Execute)),
        Some(1) => Ok(Some(TemplateMigrationOperation::Rollback)),
        None => Ok(None),
        _ => unreachable!(),
    }
}

fn step_select_target(targets: &[MigrationTarget]) -> Result<Option<MigrationTarget>> {
    if targets.is_empty() {
        println!("テンプレートマイグレーション対象が見つかりません。");
        println!("（.k1s0-template.yaml を持つプロジェクトが必要です）");
        return Ok(None);
    }

    let labels: Vec<String> = targets.iter().map(target_label).collect();
    let label_refs: Vec<&str> = labels.iter().map(String::as_str).collect();
    prompt::select_prompt("マイグレーション対象を選択してください", &label_refs)
        .map(|selection| selection.map(|index| targets[index].clone()))
}

fn step_preview(target: &MigrationTarget) -> Result<Option<MigrationPlan>> {
    println!("\n現在のバージョン: v{}", target.manifest.version());
    println!("利用可能バージョン: v{}", target.available_version);

    let plan = planner::build_plan(target)?;
    if plan.changes.is_empty() {
        println!("\n変更はありません。");
        return Ok(None);
    }

    println!("\n--- 変更プレビュー ---");
    for change in &plan.changes {
        let type_label = match (&change.change_type, &change.merge_result) {
            (ChangeType::Added, _) => "A",
            (ChangeType::Modified, MergeResult::Conflict(_)) => "C",
            (ChangeType::Modified, _) => "M",
            (ChangeType::Deleted, _) => "D",
            (ChangeType::Skipped, _) => "S",
        };
        println!("  {type_label}  {}", change.path.display());

        match (&change.change_type, &change.merge_result) {
            (ChangeType::Skipped, _) => {
                println!("     スキップ: カスタマイズ保護");
            }
            (_, MergeResult::Conflict(_)) => {
                println!("     コンフリクト: ユーザー変更とテンプレート変更が競合");
            }
            (ChangeType::Deleted, _) | (_, MergeResult::NoChange) => {}
            (_, MergeResult::Clean(new_content)) => {
                let old_path = target.path.join(&change.path);
                let old_content = if old_path.exists() {
                    std::fs::read_to_string(&old_path).unwrap_or_default()
                } else {
                    String::new()
                };
                let diff = differ::format_diff(&old_content, new_content);
                for line in diff.lines().take(20) {
                    println!("     {line}");
                }
            }
        }
    }

    match prompt::yes_no_prompt("マイグレーションを続行しますか？")? {
        Some(true) => Ok(Some(plan)),
        _ => Ok(None),
    }
}

fn step_resolve_conflicts(plan: &mut MigrationPlan) -> Result<Option<()>> {
    println!("\n--- コンフリクト解決 ---\n");

    let items = &["テンプレート版を採用", "ユーザー版を維持", "スキップ"];
    for change in &mut plan.changes {
        let MergeResult::Conflict(hunks) = &change.merge_result else {
            continue;
        };

        println!("コンフリクト: {}", change.path.display());
        for (index, hunk) in hunks.iter().enumerate() {
            let base_preview = hunk.base_preview.as_deref().unwrap_or(&hunk.base);
            let theirs_preview = hunk.theirs_preview.as_deref().unwrap_or(&hunk.theirs);
            let ours_preview = hunk.ours_preview.as_deref().unwrap_or(&hunk.ours);
            println!("\n  ハンク #{}", index + 1);
            if !base_preview.is_empty() {
                println!("  --- Base ---");
                for line in base_preview.lines().take(5) {
                    println!("    {line}");
                }
            }
            println!("  --- Template ---");
            for line in theirs_preview.lines().take(10) {
                println!("    + {line}");
            }
            println!("  --- User ---");
            for line in ours_preview.lines().take(10) {
                println!("    - {line}");
            }
        }

        match prompt::select_prompt("解決方法を選択してください", items)? {
            Some(0) => {
                let content = hunks
                    .first()
                    .map(|hunk| hunk.theirs.clone())
                    .unwrap_or_default();
                change.merge_result = MergeResult::Clean(content);
            }
            Some(1) => {
                let content = hunks
                    .first()
                    .map(|hunk| hunk.ours.clone())
                    .unwrap_or_default();
                change.merge_result = MergeResult::Clean(content);
            }
            Some(2) => {
                change.merge_result = MergeResult::NoChange;
            }
            None => return Ok(None),
            _ => unreachable!(),
        }
    }

    Ok(Some(()))
}

fn step_confirm(plan: &MigrationPlan) -> Result<Option<()>> {
    let added = plan
        .changes
        .iter()
        .filter(|change| matches!(change.change_type, ChangeType::Added))
        .count();
    let modified = plan
        .changes
        .iter()
        .filter(|change| matches!(change.change_type, ChangeType::Modified))
        .count();
    let deleted = plan
        .changes
        .iter()
        .filter(|change| matches!(change.change_type, ChangeType::Deleted))
        .count();
    let conflicts = plan
        .changes
        .iter()
        .filter(|change| matches!(change.merge_result, MergeResult::Conflict(_)))
        .count();

    println!("\n[確認] 以下の内容でテンプレートマイグレーションを実行します。よろしいですか？");
    println!("    対象:       {}", plan.target.path.display());
    println!(
        "    バージョン: v{} → v{}",
        plan.target.manifest.version(),
        plan.target.available_version,
    );
    println!(
        "    変更:       {modified} ファイル更新, {added} ファイル追加, {deleted} ファイル削除"
    );
    println!("    コンフリクト: {conflicts} 件");

    match prompt::confirm_prompt()? {
        ConfirmResult::Yes => Ok(Some(())),
        ConfirmResult::GoBack | ConfirmResult::Cancel => Ok(None),
    }
}

fn step_select_backup(backups: &[String]) -> Result<Option<String>> {
    let items: Vec<&str> = backups.iter().map(String::as_str).collect();
    prompt::select_prompt("ロールバック対象のバックアップを選択してください", &items)
        .map(|selection| selection.map(|index| backups[index].clone()))
}

fn target_label(target: &MigrationTarget) -> String {
    if target.manifest.version() == target.available_version {
        format!(
            "{} (v{}, 最新)",
            target.path.display(),
            target.manifest.version()
        )
    } else {
        format!(
            "{} (v{} → v{})",
            target.path.display(),
            target.manifest.version(),
            target.available_version,
        )
    }
}
