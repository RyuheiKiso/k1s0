# 出力制御・UX 機能

← [CLI 設計書](./)

## 出力制御

### OutputConfig

```rust
pub struct OutputConfig {
    /// 出力モード
    pub mode: OutputMode,
    /// カラー出力
    pub color: bool,
    /// 詳細出力
    pub verbose: bool,
}

pub enum OutputMode {
    /// 人間向け出力
    Human,
    /// JSON 出力
    Json,
}
```

### Output トレイト

```rust
impl Output {
    pub fn header(&self, text: &str);
    pub fn info(&self, text: &str);
    pub fn success(&self, text: &str);
    pub fn warning(&self, text: &str);
    pub fn error(&self, err: &CliError);
    pub fn list_item(&self, key: &str, value: &str);
    pub fn file_added(&self, path: &str);
    pub fn hint(&self, text: &str);
    pub fn newline(&self);
    pub fn print_json<T: Serialize>(&self, value: &T);
}
```

---

## UX 機能 (v0.2.2)

### エラーリカバリ提案

エラー発生時に、復旧のために実行可能なコマンドを自動提案します。

```
error: manifest.json が見つかりません

  試してみてください:
    $ k1s0 init  # プロジェクトを初期化
```

対応するエラー:
- `manifest_not_found`: `k1s0 init` を提案
- `directory_exists`: `--force` 付きコマンドを提案
- `FileConflict`: `k1s0 upgrade --force` を提案
- lint K010/K011 違反: `k1s0 lint --fix` を提案

### ヘルプテキスト拡充

全コマンドの `--help` に使用例と生成物の説明を追加しました（`after_long_help`）。

### 実行前プレビュー & 確認

`new-feature`、`new-domain` で生成前にファイル一覧をプレビュー表示し、確認プロンプトを表示します。

- `--yes` / `-y`: 確認をスキップ
- `--json` / 非TTY: 自動スキップ

### リアルタイム進捗フィードバック

`ProgressCallback` トレイトにより、テンプレート展開中のファイル処理進捗をプログレスバーで表示します。

### doctor 自動実行統合

- `new-feature --skip-doctor`: doctor チェックをスキップ
- `init --skip-doctor`: 初期化後の doctor チェックをスキップ
- サービスタイプに応じた関連ツールのみチェック（例: `backend-rust` → Rust カテゴリ）
