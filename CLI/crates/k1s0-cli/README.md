# k1s0-cli

k1s0 CLI の実行バイナリ。clap を使用したコマンドライン引数解析。

## サブコマンド

| コマンド | 説明 | 状態 |
|---------|------|------|
| `init` | リポジトリ初期化 | ✅ 実装済み |
| `new-feature` | 新規サービスの雛形生成 | ✅ 実装済み |
| `new-screen` | 画面の雛形生成（React/Flutter） | ✅ 実装済み |
| `lint` | 規約違反の検査（11ルール） | ✅ 実装済み |
| `upgrade` | テンプレート更新（差分提示・適用） | ✅ 実装済み |
| `registry` | テンプレートレジストリ管理 | 部分実装 |
| `completions` | シェル補完生成（bash/zsh/fish/powershell） | ✅ 実装済み |

## サポートするテンプレート

| テンプレート | 説明 |
|-------------|------|
| `backend-rust` | Rust バックエンドサービス |
| `backend-go` | Go バックエンドサービス |
| `frontend-react` | React フロントエンド |
| `frontend-flutter` | Flutter フロントエンド |

## ディレクトリ構成

```
k1s0-cli/
├── src/
│   ├── commands/
│   │   ├── init.rs
│   │   ├── new_feature.rs
│   │   ├── new_screen.rs
│   │   ├── lint.rs
│   │   ├── upgrade.rs
│   │   ├── registry.rs
│   │   ├── completions.rs
│   │   └── mod.rs
│   ├── main.rs
│   └── lib.rs
└── tests/
    └── cli_integration_tests.rs   # 33テスト
```

## 使用例

```bash
# リポジトリ初期化
k1s0 init

# Rust バックエンドサービス生成
k1s0 new-feature -t backend-rust -n user-management

# Go バックエンドサービス生成
k1s0 new-feature -t backend-go -n payment-service

# React 画面生成
k1s0 new-screen -t react -s users.list -T "ユーザー一覧"

# Flutter 画面生成
k1s0 new-screen -t flutter -s settings.profile -T "プロフィール設定"

# Lint 実行
k1s0 lint feature/backend/rust/user-management

# シェル補完生成
k1s0 completions bash > ~/.bash_completion.d/k1s0
```
