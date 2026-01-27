# k1s0

高速な開発サイクルを実現する統合開発基盤。Framework / Templates / CLI を含むモノレポ。

## 概要

k1s0 は以下の 3 つのコア機能を提供します：

- **サービス雛形の自動生成**: テンプレートから一貫したディレクトリ構造を生成
- **開発規約の自動チェック**: 11 個のルールで規約違反を検出・自動修正
- **テンプレート更新の安全な管理**: managed/protected 領域の分離で破壊的変更を回避

## ディレクトリ構成

```
k1s0/
├── CLI/                    # 雛形生成・導入・アップグレード CLI（Rust）
│   ├── crates/             # CLI を構成する Rust crate 群
│   │   ├── k1s0-cli/       # 実行 CLI（clap ベース）
│   │   └── k1s0-generator/ # テンプレ展開・差分適用・Lint エンジン
│   └── templates/          # 生成テンプレ群
│       ├── backend-rust/
│       ├── backend-go/
│       ├── frontend-react/
│       └── frontend-flutter/
├── framework/              # 共通部品・共通サービス
│   ├── backend/
│   │   ├── rust/
│   │   │   ├── crates/     # 共通 crate 群
│   │   │   └── services/   # 共通マイクロサービス
│   │   └── go/
│   ├── frontend/
│   │   ├── react/
│   │   └── flutter/
│   └── database/
│       └── table/          # 共通テーブル定義（DDL 正本）
├── feature/                # 個別機能チームのサービス実装
│   ├── backend/
│   │   ├── rust/
│   │   └── go/
│   ├── frontend/
│   │   ├── react/
│   │   └── flutter/
│   └── database/
├── bff/                    # フロントエンド向け集約 API 層（任意）
├── docs/                   # ドキュメント
│   ├── adr/                # Architecture Decision Records
│   ├── design/             # 設計書
│   ├── conventions/        # 規約
│   └── operations/         # 運用
└── work/                   # 検討中の草案
```

## クイックスタート

```bash
# リポジトリ初期化
k1s0 init

# 新規バックエンドサービスの生成
k1s0 new-feature --type backend-rust --name user-management

# 新規フロントエンド画面の生成
k1s0 new-screen --type frontend-react --name dashboard

# 規約チェック
k1s0 lint

# 規約違反の自動修正
k1s0 lint --fix

# テンプレート更新の確認
k1s0 upgrade --check

# テンプレート更新の適用
k1s0 upgrade
```

詳細は [Getting Started](docs/GETTING_STARTED.md) を参照してください。

## CLI コマンド

| コマンド | 説明 |
|---------|------|
| `k1s0 init` | リポジトリ初期化（`.k1s0/` ディレクトリ作成） |
| `k1s0 new-feature` | 新規サービス雛形生成 |
| `k1s0 new-screen` | 画面（フロントエンド）雛形生成 |
| `k1s0 lint` | 規約違反検査 |
| `k1s0 upgrade` | テンプレート更新 |
| `k1s0 completions` | シェル補完スクリプト生成 |

### 共通オプション

```bash
-v, --verbose     # 詳細出力
--no-color        # カラー出力無効化
--json            # JSON 形式出力
```

## Lint 機能

11 個のルールで開発規約を自動検査します。

| ID | 重要度 | 説明 | 自動修正 |
|----|--------|------|:--------:|
| K001 | Error | manifest.json が存在しない | - |
| K002 | Error | manifest.json の必須キーが不足 | - |
| K003 | Error | manifest.json の値が不正 | - |
| K010 | Error | 必須ディレクトリが存在しない | ✓ |
| K011 | Error | 必須ファイルが存在しない | ✓ |
| K020 | Error | 環境変数参照の禁止 | - |
| K021 | Error | config YAML への機密直書き禁止 | - |
| K022 | Error | Clean Architecture 依存方向違反 | - |
| K030 | Warning | gRPC リトライ設定の検出 | - |
| K031 | Warning | リトライ設定に ADR 参照なし | - |
| K032 | Warning | リトライ設定が不完全 | - |

```bash
# 特定ルールのみ実行
k1s0 lint --rules K001,K002

# 特定ルールを除外
k1s0 lint --exclude-rules K030,K031

# 警告もエラーとして扱う
k1s0 lint --strict
```

詳細は [Lint 設計書](docs/design/lint.md) を参照してください。

## テンプレート

4 種類のサービステンプレートを提供します。

| テンプレート | サブテンプレート | 出力先 |
|-------------|-----------------|--------|
| backend-rust | feature, project | `feature/backend/rust/{name}/` |
| backend-go | feature | `feature/backend/go/{name}/` |
| frontend-react | feature, screen | `feature/frontend/react/{name}/` |
| frontend-flutter | feature, screen | `feature/frontend/flutter/{name}/` |

詳細は [テンプレート設計書](docs/design/template.md) を参照してください。

## ドキュメント

### 入門

- [Getting Started](docs/GETTING_STARTED.md): 環境セットアップと基本操作

### 設計書

- [CLI 設計書](docs/design/cli.md): k1s0-cli の設計
- [Generator 設計書](docs/design/generator.md): k1s0-generator の設計
- [Lint 設計書](docs/design/lint.md): Lint 機能の詳細設計
- [テンプレート設計書](docs/design/template.md): テンプレートシステム設計
- [Framework 設計書](docs/design/framework.md): 共通ライブラリ設計

### 規約・方針

- [ADR](docs/adr/README.md): Architecture Decision Records
- [規約](docs/conventions/README.md): 開発規約

## Framework パッケージ

### Backend（Rust）

11 個の共通 crate を提供します。

| Crate | 説明 |
|-------|------|
| k1s0-error | エラー表現の統一 |
| k1s0-config | 設定読み込み |
| k1s0-validation | 入力バリデーション |
| k1s0-observability | ログ/トレース/メトリクス |
| k1s0-grpc-server | gRPC サーバ共通基盤 |
| k1s0-grpc-client | gRPC クライアント共通 |
| k1s0-resilience | レジリエンスパターン |
| k1s0-health | ヘルスチェック |
| k1s0-db | DB 接続・トランザクション |
| k1s0-cache | Redis キャッシュ |
| k1s0-auth | 認証・認可 |

### Frontend（React）

5 個の共通パッケージを提供します（実装済み）。

| Package | 説明 |
|---------|------|
| @k1s0/navigation | 設定駆動ナビゲーション |
| @k1s0/config | YAML 設定管理 |
| @k1s0/api-client | API 通信クライアント |
| @k1s0/ui | Design System（Material-UI） |
| @k1s0/shell | AppShell（Header/Sidebar/Footer） |

詳細は [Framework 設計書](docs/design/framework.md) を参照してください。

## 技術スタック

| レイヤー | 技術 |
|---------|------|
| **バックエンド** | Rust (axum + tokio), Go |
| **フロントエンド** | React (Material-UI, Zod), Flutter |
| **CLI** | Rust 1.85+ (clap 4.5, Tera 1.19) |
| **データベース** | PostgreSQL |
| **キャッシュ** | Redis |
| **観測性** | OpenTelemetry |
| **API** | gRPC (内部), REST/OpenAPI (外部) |

## 守るべき重要ルール

- **環境変数は使わない**: 設定ファイル（`config/*.yaml`）経由で取得
- **機密情報は直書きしない**: `*_file` サフィックスで外部参照
- **Clean Architecture に従う**: 依存方向を厳密に守る
- **リトライ設定は文書化**: ADR で設計決定を記録

## ライセンス

MIT
