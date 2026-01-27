# CLI Crates

k1s0 CLI を構成する Rust crate 群。

## Crate 一覧

| Crate | 説明 |
|-------|------|
| `k1s0-cli` | 実行 CLI（clap ベース） |
| `k1s0-generator` | テンプレート展開・差分適用・lint ロジック |
| `k1s0-lsp` | Language Server Protocol 実装（エディタ連携） |

## 依存関係

```
k1s0-cli
  └── k1s0-generator

k1s0-lsp
  └── k1s0-generator
```

## 主な機能

### k1s0-cli
- サブコマンド: `init`, `new-feature`, `new-screen`, `lint`, `upgrade`, `registry`, `completions`
- 設定ファイル対応（`.k1s0/settings.yaml`）
- JSON 出力オプション

### k1s0-generator
- テンプレート展開（Tera テンプレートエンジン）
- manifest.json の読み書き・JSON Schema バリデーション
- lint ルール実装（K001〜K032）
- 差分計算・upgrade 支援
- ファイル fingerprint 計算

### k1s0-lsp
- LSP サーバー（stdio / TCP モード）
- 増分テキスト更新（UTF-16 オフセット対応）
- デバウンス付き lint 実行
- `textDocument/publishDiagnostics` による診断情報送信

## ビルド

```bash
cd CLI
cargo build --workspace
```

## テスト

```bash
cd CLI
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
```
