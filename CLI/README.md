# k1s0 CLI

k1s0 の雛形生成・導入・アップグレードを担う CLI ツール。

## ディレクトリ構成

```
CLI/
├── crates/
│   ├── k1s0-cli/           # 実行 CLI (clap)
│   │   ├── src/
│   │   │   ├── commands/   # サブコマンド実装
│   │   │   ├── doctor/     # 環境診断モジュール
│   │   │   ├── prompts/    # 対話式プロンプト
│   │   │   ├── main.rs
│   │   │   └── lib.rs
│   │   └── tests/          # 統合テスト
│   ├── k1s0-generator/     # テンプレ展開・差分適用・lint ロジック
│   │   └── src/
│   │       ├── lint/       # lint ルール実装
│   │       ├── analyzer/   # 移行分析エンジン
│   │       ├── diff/       # 差分計算
│   │       └── lib.rs
│   └── k1s0-lsp/           # Language Server Protocol 実装
│       └── src/
│           └── lib.rs
├── templates/              # 生成テンプレ群
│   ├── backend-rust/
│   ├── backend-go/
│   ├── backend-csharp/
│   ├── backend-python/
│   ├── backend-kotlin/
│   ├── frontend-react/
│   ├── frontend-flutter/
│   ├── frontend-android/
│   └── playground/         # playground 用テンプレート
└── schemas/                # JSON Schema 定義
```

## コマンド一覧

| コマンド | 説明 |
|---------|------|
| `k1s0 init` | リポジトリ初期化（`.k1s0/` 作成等） |
| `k1s0 new-feature` | 新規サービスの雛形生成 |
| `k1s0 new-domain` | 新規 domain の雛形生成 |
| `k1s0 new-screen` | 画面（Screen）の雛形生成（React/Flutter） |
| `k1s0 lint` | 規約違反の検査 |
| `k1s0 upgrade` | テンプレート更新の適用 |
| `k1s0 doctor` | 開発環境の健全性チェック |
| `k1s0 completions` | シェル補完スクリプトの生成 |
| `k1s0 registry` | テンプレートレジストリの操作 |
| `k1s0 domain-list` | 全 domain の一覧表示 |
| `k1s0 domain-version` | domain バージョンの表示・更新 |
| `k1s0 domain-dependents` | domain に依存する feature の表示 |
| `k1s0 domain-impact` | domain バージョンアップの影響分析 |
| `k1s0 domain-catalog` | domain カタログの表示 |
| `k1s0 domain-graph` | domain 依存グラフ出力（Mermaid/DOT） |
| `k1s0 feature-update-domain` | Feature のドメイン依存更新 |
| `k1s0 docker build` | Docker イメージをビルド |
| `k1s0 docker compose up` | docker compose サービスを起動 |
| `k1s0 docker compose down` | docker compose サービスを停止 |
| `k1s0 docker compose logs` | docker compose ログを表示 |
| `k1s0 docker status` | コンテナ状態を表示 |
| `k1s0 playground start` | playground 環境を起動 |
| `k1s0 playground stop` | playground 環境を停止 |
| `k1s0 playground status` | playground 環境の状態を表示 |
| `k1s0 playground list` | 利用可能な playground テンプレート一覧 |
| `k1s0 migrate analyze` | 既存プロジェクトの準拠状況を分析 |
| `k1s0 migrate plan` | 移行計画を生成 |
| `k1s0 migrate apply` | 移行計画を適用 |
| `k1s0 migrate status` | 移行の進捗状況を表示 |
| `k1s0 log` | Git コミット履歴を表示 |
| `k1s0 diff` | Git diff を表示 |

## lint ルール

### Manifest & Structure (K001-K011)

| ルール ID | 深刻度 | 説明 | 自動修正 |
|-----------|--------|------|:--------:|
| K001 | Error | manifest.json が存在しない | - |
| K002 | Error | manifest.json の必須キーが不足 | - |
| K003 | Error | manifest.json の値が不正 | - |
| K010 | Error | 必須ディレクトリが存在しない | Yes |
| K011 | Error | 必須ファイルが存在しない | Yes |

### Code Quality (K020-K029)

| ルール ID | 深刻度 | 説明 | 自動修正 |
|-----------|--------|------|:--------:|
| K020 | Error | 環境変数参照の禁止 | - |
| K021 | Error | config YAML への機密直書き禁止 | - |
| K022 | Error | Clean Architecture 依存方向違反 | - |
| K025 | Error | 設定ファイル命名規約違反（default/dev/stg/prod のみ） | - |
| K026 | Error | Domain 層でのプロトコル型使用（HTTP/gRPC 依存） | - |
| K028 | Warning | 未使用 domain 依存宣言 | - |
| K029 | Error | 本番コードでの panic/unwrap/expect | - |

### gRPC Retry (K030-K032)

| ルール ID | 深刻度 | 説明 | 自動修正 |
|-----------|--------|------|:--------:|
| K030 | Warning | gRPC リトライ設定の検出 | - |
| K031 | Warning | gRPC リトライ設定に ADR 参照がない | - |
| K032 | Warning | gRPC リトライ設定が不完全 | - |

### Layer Dependency (K040-K047)

| ルール ID | 深刻度 | 説明 | 自動修正 |
|-----------|--------|------|:--------:|
| K040 | Error | 層間依存ルール違反 | - |
| K041 | Error | 参照先 domain が見つからない | - |
| K042 | Error | domain バージョン制約の不一致 | - |
| K043 | Error | domain 間の循環依存 | - |
| K044 | Warning | 非推奨 domain の使用 | - |
| K045 | Warning | min_framework_version 未満 | - |
| K046 | Warning | 破壊的変更の影響検出 | - |
| K047 | Error | domain 層に必須の version フィールドがない | - |

### Security (K050-K053)

| ルール ID | 深刻度 | 説明 | 自動修正 |
|-----------|--------|------|:--------:|
| K050 | Error | SQL インジェクションリスク（文字列結合） | - |
| K053 | Warning | 機密データのログ出力 | - |

### Infrastructure (K060)

| ルール ID | 深刻度 | 説明 | 自動修正 |
|-----------|--------|------|:--------:|
| K060 | Warning | Dockerfile ベースイメージ未固定（:latest またはタグなし） | - |

## LSP サーバー

エディタ連携用の Language Server を提供。

```bash
# stdio モードで起動
k1s0-lsp --stdio

# TCP モードで起動
k1s0-lsp --tcp --port 9257
```

### LSP 機能

| 機能 | 説明 |
|------|------|
| 補完（Completion） | キー/値の自動補完、スニペット |
| ホバー（Hover） | キーの説明、値の型情報 |
| 診断（Diagnostics） | JSON 構文エラー、スキーマバリデーション、lint 結果 |
| 定義ジャンプ（Go to Definition） | テンプレート/crate への移動 |
| 参照検索（Find References） | 値の使用箇所を検索 |
| ドキュメントシンボル（Document Symbol） | ファイル内のシンボル一覧 |
| ワークスペースシンボル（Workspace Symbol） | プロジェクト全体のシンボル検索 |
| コードアクション（Code Action） | 診断に対する QuickFix 自動修正 |
| ドキュメントリンク（Document Link） | manifest.json 内パスのクリック可能リンク |

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

- [CLI 設計書](../docs/design/cli/): CLI アーキテクチャ詳細
- [crates/README.md](crates/README.md): Crate 詳細
