# lint コマンド

← [CLI 設計書](./)

## 目的

k1s0 の開発規約に対する違反を検査する。

## 引数

```rust
pub struct LintArgs {
    /// 検査するディレクトリ（デフォルト: カレントディレクトリ）
    #[arg(default_value = ".")]
    pub path: String,

    /// 特定のルールのみ実行（カンマ区切り）
    #[arg(long)]
    pub rules: Option<String>,

    /// 特定のルールを除外（カンマ区切り）
    #[arg(long)]
    pub exclude_rules: Option<String>,

    /// 警告をエラーとして扱う
    #[arg(long)]
    pub strict: bool,

    /// 自動修正を試みる
    #[arg(long)]
    pub fix: bool,

    /// JSON 形式で出力
    #[arg(long)]
    pub json: bool,

    /// 環境変数参照を許可するファイルパス（カンマ区切り、glob パターン対応）
    #[arg(long)]
    pub env_var_allowlist: Option<String>,

    /// ファイル変更を監視して自動 lint
    #[arg(long)]
    pub watch: bool,

    /// Git diff ベースで変更ファイルのみ検査（ブランチ名指定）
    #[arg(long)]
    pub diff: Option<String>,

    /// watch モードのデバウンス間隔（ミリ秒）
    #[arg(long, default_value = "500")]
    pub debounce_ms: u64,

    /// 設定ファイルを無視する
    #[arg(long)]
    pub no_config: bool,
}
```

## 処理フロー

```
1. パスの存在確認
2. LintConfig の構築
3. Linter 実行
4. --fix 指定時: 自動修正実行
   └─ 修正後に再検査
5. 結果出力
   ├─ --json: JSON 形式
   └─ なし: 人間向け形式
6. 終了コード決定
```

## 詳細

Lint 機能の詳細は [lint 設計書](../lint/) を参照。
