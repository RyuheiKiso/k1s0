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

use std::path::PathBuf;

use clap::Args;
use k1s0_generator::upgrade::{apply_upgrade, check_upgrade, VersionChange};

use crate::error::{CliError, Result};
use crate::output::output;

/// `k1s0 upgrade` の引数
#[derive(Args, Debug)]
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
    out.info("変更を適用しています...");

    let apply_result = apply_upgrade(&service_path, &check_result, args.managed_only, args.backup)
        .map_err(|e| CliError::internal(e.to_string()))?;

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

    // マイグレーション適用（将来実装）
    if args.apply_migrations {
        out.warning("--apply-migrations は現在実装されていません。");
        out.hint("手動で DB マイグレーションを実行してください。");
    }

    Ok(())
}
