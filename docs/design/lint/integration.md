# CLI 統合・LSP 統合

← [Lint 設計書](./)

---

## CLI 統合

### 使用例

```bash
# 基本的な lint 実行
k1s0 lint

# 特定のルールのみ実行
k1s0 lint --rules K001,K002,K003

# 特定のルールを除外
k1s0 lint --exclude-rules K030,K031,K032

# 警告をエラーとして扱う（CI 向け）
k1s0 lint --strict

# 自動修正を試みる
k1s0 lint --fix

# JSON 出力
k1s0 lint --json

# 環境変数参照を許可するファイルを指定
k1s0 lint --env-var-allowlist "tests/**/*,scripts/**/*"
```

### JSON 出力形式

```json
{
  "error": true,
  "path": "feature/backend/rust/user-service",
  "violation_count": 2,
  "warning_count": 1,
  "violations": [
    {
      "rule": "K001",
      "severity": "error",
      "message": "manifest.json が見つかりません",
      "path": ".k1s0/manifest.json",
      "line": null
    },
    {
      "rule": "K030",
      "severity": "warning",
      "message": "gRPC リトライ設定が検出されました",
      "path": "config/default.yaml",
      "line": 42
    }
  ]
}
```

---

## LSP 統合

### 概要

k1s0-lsp は Language Server Protocol を実装し、エディタ/IDE と連携して lint 結果をリアルタイムで提供します。

### モジュール構成

```
CLI/crates/k1s0-lsp/
├── Cargo.toml
├── src/
│   ├── lib.rs           # LSP サーバ本体
│   ├── main.rs          # エントリポイント
│   ├── schema.rs        # manifest.json スキーマ定義
│   ├── completion.rs    # 補完機能
│   ├── hover.rs         # ホバー情報機能
│   ├── definition.rs    # 定義ジャンプ
│   ├── references.rs    # 参照検索
│   └── symbols.rs       # シンボル機能
```

### サポート機能

| 機能 | 説明 |
|------|------|
| `textDocument/publishDiagnostics` | lint 結果を診断情報として送信 |
| `textDocument/didOpen` | ファイル開時に lint 実行 |
| `textDocument/didSave` | ファイル保存時に lint 実行 |
| `textDocument/didChange` | ファイル変更時に lint 実行（デバウンス付き） |
| `textDocument/completion` | manifest.json の入力補完 |
| `textDocument/hover` | manifest.json キーのホバー情報 |

### 起動方法

```bash
# stdio モードで起動
k1s0-lsp --stdio

# TCP モードで起動（デバッグ用）
k1s0-lsp --tcp --port 9257
```

### 補完機能

manifest.json 編集時に以下の補完を提供：

**キー補完:**
- ルートレベルのキー（`schema_version`, `template`, `service` 等）
- ネストされたキー（`template.name`, `service.language` 等）

**値補完:**
- enum 型の値（`rust`, `go`, `backend`, `frontend` 等）
- 例に基づく値の提案

### ホバー情報

manifest.json のキーにカーソルを合わせると以下の情報を表示：

- キーの説明
- 必須/オプションの区別
- 型情報（enum の場合は有効な値一覧）
- 使用例

### 設定

```rust
pub struct LspConfig {
    /// lint 設定
    pub lint: LintConfig,
    /// ファイル変更時の lint を有効にするか
    pub lint_on_change: bool,
    /// デバウンス間隔（ミリ秒）
    pub debounce_ms: u64,  // デフォルト: 500
}
```

### VS Code 統合例

```json
// .vscode/settings.json
{
  "k1s0.lsp.path": "/path/to/k1s0-lsp",
  "k1s0.lsp.args": ["--stdio"]
}
```
