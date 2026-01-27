# LSP 開発エージェント

k1s0-lsp (Language Server Protocol) の開発を支援するエージェント。

## 対象領域

- `CLI/crates/k1s0-lsp/` - LSP 実装

## 概要

k1s0-lsp はエディタ統合用の Language Server で、以下の機能を提供:

- Lint 結果をエディタの診断情報として表示
- ファイル保存時に自動検査
- UTF-16 オフセット対応の増分テキスト更新
- 500ms デバウンス付き lint 実行
- Completion（補完）機能
- Hover（ホバー）機能

## 起動モード

### stdio モード

```bash
k1s0-lsp --stdio
```

エディタが標準入出力で通信する場合に使用。

### TCP モード

```bash
k1s0-lsp --tcp --port 9257
```

デバッグや複数クライアント接続時に使用。

## ビルド・テスト

```bash
cd CLI

# ビルド
cargo build -p k1s0-lsp

# テスト
cargo test -p k1s0-lsp

# リリースビルド
cargo build -p k1s0-lsp --release
```

## 依存関係

- `tower-lsp`: LSP サーバーフレームワーク
- `k1s0-generator`: Lint エンジン（診断情報の生成）

## LSP 機能

### 診断 (Diagnostics)

k1s0 Lint ルール (K001-K032) をエディタ診断として表示:

- Error: manifest.json 問題、必須ファイル/ディレクトリ不足、規約違反
- Warning: gRPC リトライ設定の問題

### ファイル監視

- `textDocument/didOpen`: ファイルオープン時に検査
- `textDocument/didChange`: 変更時に検査（500ms デバウンス）
- `textDocument/didSave`: 保存時に検査

### Completion（補完）

manifest.json や config ファイルのキー補完を提供。

### Hover（ホバー）

k1s0 固有のキーや設定にホバー情報を表示。

## エディタ設定例

### VS Code

```json
{
  "k1s0.lsp.path": "/path/to/k1s0-lsp",
  "k1s0.lsp.args": ["--stdio"]
}
```

### Neovim (nvim-lspconfig)

```lua
require('lspconfig').k1s0.setup {
  cmd = { 'k1s0-lsp', '--stdio' },
  filetypes = { 'yaml', 'json' },
  root_dir = function(fname)
    return require('lspconfig').util.find_git_ancestor(fname)
  end,
}
```

## 開発・デバッグ

### ログ出力

```bash
# 詳細ログを有効化
RUST_LOG=k1s0_lsp=debug k1s0-lsp --stdio
```

### テスト用クライアント

```bash
# TCP モードで起動してテスト
k1s0-lsp --tcp --port 9257

# 別ターミナルで接続テスト
nc localhost 9257
```

## アーキテクチャ

```
┌─────────────┐     ┌─────────────┐     ┌────────────────┐
│   Editor    │────▶│  k1s0-lsp   │────▶│ k1s0-generator │
│  (Client)   │◀────│  (Server)   │◀────│   (Lint Engine)│
└─────────────┘     └─────────────┘     └────────────────┘
      LSP Protocol        Internal API
```
