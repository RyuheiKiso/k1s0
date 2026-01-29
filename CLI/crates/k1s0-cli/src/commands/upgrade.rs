//! `k1s0 upgrade` コマンド
//!
//! テンプレートの更新を確認・適用する。
//!
//! ## フェーズ32: upgrade --check
//! - manifest.json と新テンプレートの fingerprint 差分を計算
//! - managed_paths の変更対象を一覧表示
//! - 衝突（手動変更されたファイル）を検知
//! - MAJOR 変更の場合、ADR/UPGRADE.md の存在を確認
//!
//! ## フェーズ33: upgrade
//! - managed 領域のみパッチを適用
//! - protected 領域は差分提示のみ
//! - 衝突時は手動解決を促して停止
//! - manifest.json を更新

use std::path::{Path, PathBuf};

use clap::Args;
use k1s0_generator::upgrade::{apply_migrations, apply_upgrade, check_upgrade, list_pending_migrations, VersionChange};

use crate::error::{CliError, Result};
use crate::output::output;

/// `k1s0 upgrade` の引数
#[derive(Args, Debug)]
#[command(after_long_help = r#"例:
  k1s0 upgrade --check
  k1s0 upgrade --yes
  k1s0 upgrade --to-version 0.2.0 --backup

--check で差分のみ表示、--yes で対話的な確認をスキップします。
"#)]
pub struct UpgradeArgs {
    /// 更新するサービスのディレクトリ（デフォルト: カレントディレクトリ）
    #[arg(default_value = ".")]
    pub path: String,

    /// 差分のみ表示し、実際には適用しない
    #[arg(long)]
    pub check: bool,

    /// 対話的な確認なしで適用する
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// managed 領域のみ更新（protected 領域の差分は提示のみ）
    #[arg(long)]
    pub managed_only: bool,

    /// 特定のバージョンにアップグレード
    #[arg(long)]
    pub to_version: Option<String>,

    /// 衝突時にバックアップを作成
    #[arg(long, default_value = "true")]
    pub backup: bool,

    /// DB マイグレーションを自動適用（dev 環境のみ）
    #[arg(long)]
    pub apply_migrations: bool,

    /// テンプレートディレクトリを直接指定（テスト・開発用）
    #[arg(long, hide = true)]
    pub template_path: Option<PathBuf>,
}

/// `k1s0 upgrade` を実行する
pub fn execute(args: UpgradeArgs) -> Result<()> {
    let out = output();

    let service_path = PathBuf::from(&args.path);
    let manifest_path = service_path.join(".k1s0/manifest.json");

    // manifest が存在するか確認
    if !manifest_path.exists() {
        let err = CliError::manifest_not_found(&manifest_path);
        out.error(&err);
        return Err(err);
    }

    // upgrade --check を実行
    let check_result = check_upgrade(&service_path, args.template_path.as_deref())
        .map_err(|e| CliError::internal(e.to_string()))?;

    if args.check {
        // --check モード: 差分を表示して終了
        out.header("k1s0 upgrade --check");
        out.newline();

        println!("{}", check_result.format_check_output());

        if check_result.has_conflicts {
            out.warning("衝突が検出されました。手動で解決してから再実行してください。");
            return Ok(());
        }

        if !check_result.needs_upgrade {
            out.success("最新の状態です。更新は不要です。");
            return Ok(());
        }

        out.newline();
        out.info("実際に適用するには `k1s0 upgrade` を実行してください。");

        return Ok(());
    }

    // 適用モード
    out.header("k1s0 upgrade");
    out.newline();

    // 更新が不要な場合
    if !check_result.needs_upgrade {
        out.success("最新の状態です。更新は不要です。");
        return Ok(());
    }

    // 衝突がある場合は停止
    if check_result.has_conflicts {
        let err = CliError::conflict("衝突が検出されました")
            .with_hint("衝突を解決してから再実行してください");
        out.error(&err);
        out.newline();
        println!("{}", check_result.format_check_output());
        out.newline();
        out.hint("  1. 手動で変更をマージする");
        out.hint("  2. または、チェックサムを更新して上書きを許可する");
        return Err(err);
    }

    // MAJOR 変更の場合は警告
    if check_result.version_change == VersionChange::Major {
        out.warning("MAJOR バージョン変更です。破壊的変更が含まれる可能性があります。");
        if check_result.has_upgrade_adr || check_result.has_upgrade_md {
            out.info("移行ガイドを確認してください:");
            if check_result.has_upgrade_adr {
                out.hint("  - ADR/UPGRADE.md");
            }
            if check_result.has_upgrade_md {
                out.hint("  - UPGRADE.md");
            }
        }
        out.newline();
    }

    // 差分を表示
    out.header("変更内容:");
    println!("{}", check_result.format_check_output());

    // 確認
    if !args.yes {
        out.newline();
        out.info("上記の変更を適用しますか？");
        out.hint("  --yes オプションで確認をスキップできます");

        // 対話的な確認（現在は簡易実装）
        use std::io::{self, Write};
        print!("続行しますか？ [y/N] ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if !input.trim().eq_ignore_ascii_case("y") {
            out.info("キャンセルしました。");
            return Ok(());
        }
    }

    // 適用
    out.newline();
    let spinner = out.spinner("変更を適用しています...");

    let apply_result = apply_upgrade(&service_path, &check_result, args.managed_only, args.backup)
        .map_err(|e| CliError::internal(e.to_string()))?;

    spinner.finish_and_clear();

    // 結果を表示
    out.newline();
    out.success("アップグレードが完了しました。");
    out.newline();

    if !apply_result.applied.is_empty() {
        out.header("適用されたファイル:");
        for path in &apply_result.applied {
            out.list_item("  ✓", path);
        }
        out.newline();
    }

    if !apply_result.skipped.is_empty() {
        out.header("スキップされたファイル (protected):");
        for path in &apply_result.skipped {
            out.list_item("  -", path);
        }
        out.hint("protected 領域は手動で確認・更新してください。");
        out.newline();
    }

    if !apply_result.backed_up.is_empty() {
        out.header("バックアップ:");
        for path in &apply_result.backed_up {
            out.list_item("  →", &format!("{}.bak", path));
        }
        out.newline();
    }

    // マイグレーション適用
    if args.apply_migrations {
        out.newline();
        out.header("DB マイグレーション");
        out.newline();

        // 環境を検出（dev のみ許可）
        let env = detect_environment(&service_path);

        if env != "dev" && env != "development" && env != "local" {
            out.warning(&format!(
                "自動マイグレーションは dev 環境でのみ実行可能です（現在: {}）",
                env
            ));
            out.hint("開発環境で実行するか、手動でマイグレーションを適用してください。");
        } else {
            // 保留中のマイグレーションを一覧表示
            match list_pending_migrations(&service_path) {
                Ok(pending) => {
                    if pending.is_empty() {
                        out.success("保留中のマイグレーションはありません。");
                    } else {
                        out.info(&format!("保留中のマイグレーション: {} 件", pending.len()));
                        for migration in &pending {
                            out.list_item("  -", &migration.name);
                        }
                        out.newline();

                        // マイグレーションを適用
                        match apply_migrations(&service_path, &env, false) {
                            Ok(result) => {
                                if result.is_success() {
                                    out.success(&format!("マイグレーション完了: {}", result.summary()));
                                    if !result.applied.is_empty() {
                                        out.header("適用したマイグレーション:");
                                        for name in &result.applied {
                                            out.list_item("  ✓", name);
                                        }
                                    }
                                } else {
                                    out.warning(&format!("一部のマイグレーションが失敗しました: {}", result.summary()));
                                    for (name, error) in &result.failed {
                                        out.warning(&format!("  ✗ {}: {}", name, error));
                                    }
                                }
                            }
                            Err(e) => {
                                out.warning(&format!("マイグレーション実行エラー: {}", e));
                            }
                        }
                    }
                }
                Err(e) => {
                    out.warning(&format!("マイグレーションファイルの取得に失敗: {}", e));
                }
            }
        }
    }

    Ok(())
}

/// 環境を検出する
fn detect_environment(service_path: &Path) -> String {
    // 環境変数から取得
    if let Ok(env) = std::env::var("K1S0_ENV") {
        return env;
    }
    if let Ok(env) = std::env::var("ENV") {
        return env;
    }
    if let Ok(env) = std::env::var("APP_ENV") {
        return env;
    }

    // manifest.json から取得を試みる
    let manifest_path = service_path.join(".k1s0/manifest.json");
    if let Ok(content) = std::fs::read_to_string(&manifest_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(env) = json.get("settings")
                .and_then(|s| s.get("env"))
                .and_then(|e| e.as_str())
            {
                return env.to_string();
            }
        }
    }

    // デフォルトは dev
    "dev".to_string()
}
