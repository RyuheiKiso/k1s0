# upgrade コマンド

← [CLI 設計書](./)

## 目的

テンプレートの更新を確認・適用する。

## 引数

```rust
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
```

## 処理フロー（--check モード）

```
1. manifest.json の存在確認
2. check_upgrade() 実行
   ├─ manifest 読み込み
   ├─ テンプレートパス決定
   ├─ 新 fingerprint 計算
   ├─ 差分計算
   └─ ADR/UPGRADE.md 確認
3. 差分表示
4. 次のアクション提示
```

## 処理フロー（適用モード）

```
1. manifest.json の存在確認
2. check_upgrade() 実行
3. 更新が不要な場合: 終了
4. 衝突がある場合: エラー
5. MAJOR 変更の場合: 警告
6. 差分表示
7. 確認（--yes でスキップ）
8. apply_upgrade() 実行
   ├─ managed 領域の変更適用
   ├─ バックアップ作成
   ├─ manifest.json 更新
   └─ checksums 更新
9. 結果表示
10. --apply-migrations: マイグレーション適用
```
