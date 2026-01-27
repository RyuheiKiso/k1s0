# k1s0 CLI

k1s0 の雛形生成・導入・アップグレードを担う CLI ツール。

## ディレクトリ構成

```
CLI/
├── crates/
│   ├── k1s0-cli/           # 実行 CLI (clap)
│   │   ├── src/
│   │   │   ├── commands/   # サブコマンド実装
│   │   │   ├── main.rs
│   │   │   └── lib.rs
│   │   └── tests/          # 統合テスト
│   ├── k1s0-generator/     # テンプレ展開・差分適用・lint ロジック
│   │   └── src/
│   │       ├── lint/       # lint ルール実装
│   │       ├── diff/       # 差分計算
│   │       └── lib.rs
│   └── k1s0-lsp/           # Language Server Protocol 実装
│       └── src/
│           └── lib.rs
├── templates/              # 生成テンプレ群
│   ├── backend-rust/
│   ├── backend-go/
│   ├── frontend-react/
│   └── frontend-flutter/
└── schemas/                # JSON Schema 定義
```

## コマンド一覧

| コマンド | 説明 |
|---------|------|
| `k1s0 init` | リポジトリ初期化（`.k1s0/` 作成等） |
| `k1s0 new-feature` | 新規サービスの雛形生成 |
| `k1s0 new-screen` | 画面（Screen）の雛形生成（React/Flutter） |
| `k1s0 lint` | 規約違反の検査 |
| `k1s0 upgrade --check` | 差分提示と衝突検知 |
| `k1s0 upgrade` | テンプレート更新の適用 |
| `k1s0 registry` | テンプレートレジストリの操作 |
| `k1s0 completions` | シェル補完スクリプトの生成 |

## lint ルール

| ルール ID | 説明 |
|-----------|------|
| K001 | manifest.json が存在しない |
| K002 | manifest.json の必須キーが不足 |
| K003 | manifest.json の値が不正 |
| K010 | 必須ディレクトリが存在しない |
| K011 | 必須ファイルが存在しない |
| K020 | 環境変数参照の禁止 |
| K021 | config YAML への機密直書き禁止 |
| K022 | Clean Architecture 依存方向違反 |
| K030 | gRPC リトライ設定の検出（可視化） |
| K031 | gRPC リトライ設定に ADR 参照がない |
| K032 | gRPC リトライ設定が不完全 |

## LSP サーバー

エディタ連携用の Language Server を提供。

```bash
# stdio モードで起動
k1s0-lsp --stdio

# TCP モードで起動
k1s0-lsp --tcp --port 9257
```

### LSP 機能

- `textDocument/publishDiagnostics`: lint 結果を診断情報として送信
- `textDocument/didOpen`: ファイルを開いたときに lint 実行
- `textDocument/didSave`: ファイルを保存したときに lint 実行
- `textDocument/didChange`: ファイルを変更したときに lint 実行（デバウンス付き）

### 実装済み機能

- 増分テキスト更新（UTF-16 オフセット対応）
- デバウンス付き lint 実行（既定 500ms）
- URI 毎のタスク管理と既存タスクのキャンセル

## 開発

```bash
# ビルド
cd CLI
cargo build

# 実行
cargo run -- --help

# テスト
cargo test --all

# lint
cargo clippy --all-targets --all-features -- -D warnings
```

## 関連ドキュメント

- [プラン.md](../work/初期開発/プラン.md): CLI 実装計画（フェーズ3〜5, 11〜13, 32〜33）
- [crates/README.md](crates/README.md): Crate 詳細
