# k1s0-cli

k1s0 CLI の実行バイナリ。clap を使用したコマンドライン引数解析。

## サブコマンド

| コマンド | 説明 | 状態 |
|---------|------|------|
| `init` | リポジトリ初期化 | ✅ 実装済み |
| `new-feature` | 新規サービスの雛形生成 | ✅ 実装済み |
| `new-domain` | 新規 domain の雛形生成 | ✅ 実装済み |
| `new-screen` | 画面の雛形生成（React/Flutter） | ✅ 実装済み |
| `lint` | 規約違反の検査（26ルール） | ✅ 実装済み |
| `upgrade` | テンプレート更新（差分提示・適用） | ✅ 実装済み |
| `doctor` | 環境診断 | ✅ 実装済み |
| `completions` | シェル補完生成（bash/zsh/fish/powershell） | ✅ 実装済み |
| `domain-list` | 全 domain の一覧表示 | ✅ 実装済み |
| `domain-version` | domain バージョンの表示・更新 | ✅ 実装済み |
| `domain-dependents` | domain に依存する feature の一覧表示 | ✅ 実装済み |
| `domain-impact` | domain バージョンアップの影響分析 | ✅ 実装済み |
| `domain-catalog` | domain カタログ（依存状況付き）の表示 | ✅ 実装済み |
| `domain-graph` | domain 依存グラフ出力（Mermaid/DOT） | ✅ 実装済み |
| `feature-update-domain` | feature のドメイン依存更新 | ✅ 実装済み |
| `docker` | Docker ビルド・Compose 操作 | ✅ 実装済み |
| `playground` | サンプル付き一時環境の生成・起動・停止 | ✅ 実装済み |
| `migrate` | 既存プロジェクトの k1s0 移行支援 | ✅ 実装済み |
| `log` | Git コミット履歴表示 | ✅ 実装済み |
| `diff` | Git diff 表示 | ✅ 実装済み |
| `registry` | テンプレートレジストリ管理 | 部分実装 |

## サポートするテンプレート

| テンプレート | 説明 |
|-------------|------|
| `backend-rust` | Rust バックエンドサービス |
| `backend-go` | Go バックエンドサービス |
| `backend-csharp` | C# バックエンドサービス |
| `backend-python` | Python バックエンドサービス |
| `backend-kotlin` | Kotlin バックエンドサービス |
| `frontend-react` | React フロントエンド |
| `frontend-flutter` | Flutter フロントエンド |
| `frontend-android` | Android フロントエンド |

## ディレクトリ構成

```
k1s0-cli/
├── src/
│   ├── commands/
│   │   ├── init.rs
│   │   ├── new_feature.rs
│   │   ├── new_domain.rs
│   │   ├── new_screen.rs
│   │   ├── lint.rs
│   │   ├── upgrade.rs
│   │   ├── doctor.rs
│   │   ├── registry.rs
│   │   ├── completions.rs
│   │   ├── domain_list.rs
│   │   ├── domain_version.rs
│   │   ├── domain_dependents.rs
│   │   ├── domain_impact.rs
│   │   ├── domain_catalog.rs
│   │   ├── domain_graph.rs
│   │   ├── feature_update_domain.rs
│   │   ├── docker.rs
│   │   ├── playground.rs
│   │   ├── migrate.rs
│   │   ├── log.rs
│   │   ├── diff.rs
│   │   └── mod.rs
│   ├── main.rs
│   └── lib.rs
└── tests/
    └── cli_integration_tests.rs
```

## 使用例

```bash
# リポジトリ初期化
k1s0 init

# Rust バックエンドサービス生成
k1s0 new-feature -t backend-rust -n user-management

# Kotlin バックエンドサービス生成
k1s0 new-feature -t backend-kotlin -n payment-service

# Android フロントエンド生成
k1s0 new-feature -t frontend-android -n mobile-app

# React 画面生成
k1s0 new-screen -t react -s users.list -T "ユーザー一覧"

# Lint 実行
k1s0 lint feature/backend/rust/user-management

# Docker ビルド
k1s0 docker build

# Playground 起動
k1s0 playground start --type backend-rust

# 既存プロジェクトの移行
k1s0 migrate analyze --path ./existing-project --type backend-go

# シェル補完生成
k1s0 completions bash > ~/.bash_completion.d/k1s0
```
