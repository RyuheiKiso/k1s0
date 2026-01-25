//! `k1s0 upgrade` コマンド
//!
//! テンプレートの更新を確認・適用する。

use clap::Args;

use crate::error::Result;
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
}

/// `k1s0 upgrade` を実行する
pub fn execute(args: UpgradeArgs) -> Result<()> {
    let out = output();

    if args.check {
        out.header("k1s0 upgrade --check");
    } else {
        out.header("k1s0 upgrade");
    }
    out.newline();

    out.list_item("path", &args.path);
    out.list_item("check", &args.check.to_string());
    out.list_item("yes", &args.yes.to_string());
    out.list_item("managed_only", &args.managed_only.to_string());
    if let Some(version) = &args.to_version {
        out.list_item("to_version", version);
    }
    out.list_item("backup", &args.backup.to_string());
    out.list_item("apply_migrations", &args.apply_migrations.to_string());
    out.newline();

    if args.check {
        out.info("TODO: 実装予定（フェーズ32）");
        out.newline();

        out.header("実行内容:");
        out.hint("1. manifest.json と新テンプレートの差分を計算");
        out.hint("2. managed_paths の変更対象を一覧表示");
        out.hint("3. 衝突（手動変更されたファイル）を検知");
        out.hint("4. MAJOR 変更の場合、ADR/UPGRADE.md の存在を確認");
    } else {
        out.info("TODO: 実装予定（フェーズ33）");
        out.newline();

        out.header("実行内容:");
        out.hint("1. upgrade --check を実行");
        out.hint("2. managed_paths のみパッチを適用");
        out.hint("3. protected_paths は差分提示のみ");
        out.hint("4. 衝突時は手動解決を促して停止");
        out.hint("5. manifest.json を更新");
    }

    Ok(())
}
