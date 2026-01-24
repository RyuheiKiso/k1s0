//! `k1s0 upgrade` コマンド
//!
//! テンプレートの更新を確認・適用する。

use anyhow::Result;
use clap::Args;

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
    if args.check {
        println!("k1s0 upgrade --check");
    } else {
        println!("k1s0 upgrade");
    }

    println!("  path: {}", args.path);
    println!("  check: {}", args.check);
    println!("  yes: {}", args.yes);
    println!("  managed_only: {}", args.managed_only);
    if let Some(version) = &args.to_version {
        println!("  to_version: {}", version);
    }
    println!("  backup: {}", args.backup);
    println!("  apply_migrations: {}", args.apply_migrations);
    println!();

    if args.check {
        println!("TODO: 実装予定（フェーズ32）");
        println!();
        println!("実行内容:");
        println!("  1. manifest.json と新テンプレートの差分を計算");
        println!("  2. managed_paths の変更対象を一覧表示");
        println!("  3. 衝突（手動変更されたファイル）を検知");
        println!("  4. MAJOR 変更の場合、ADR/UPGRADE.md の存在を確認");
    } else {
        println!("TODO: 実装予定（フェーズ33）");
        println!();
        println!("実行内容:");
        println!("  1. upgrade --check を実行");
        println!("  2. managed_paths のみパッチを適用");
        println!("  3. protected_paths は差分提示のみ");
        println!("  4. 衝突時は手動解決を促して停止");
        println!("  5. manifest.json を更新");
    }

    Ok(())
}
