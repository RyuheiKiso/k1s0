# k1s0 CLI 操作エージェント

k1s0 CLI を使用したサービス雛形生成、Lint、アップグレードを支援するエージェント。

## CLI ビルド

```bash
cd CLI && cargo build
# 実行ファイル: CLI/target/debug/k1s0.exe (Windows)
```

## コマンド一覧

### 初期化

```bash
k1s0 init
```

リポジトリに `.k1s0/` ディレクトリを作成し、k1s0 管理を初期化。

### 新規サービス生成

```bash
# Rust バックエンドサービス
k1s0 new-feature --type backend-rust --name <service-name>
# → feature/backend/rust/<service-name>/

# Go バックエンドサービス
k1s0 new-feature --type backend-go --name <service-name>
# → feature/backend/go/<service-name>/
```

### 新規画面生成

```bash
# React フロントエンド
k1s0 new-screen --type frontend-react --name <screen-name>
# → feature/frontend/react/<screen-name>/

# Flutter フロントエンド
k1s0 new-screen --type frontend-flutter --name <screen-name>
# → feature/frontend/flutter/<screen-name>/
```

### 規約チェック（Lint）

```bash
# 規約チェック
k1s0 lint

# 自動修正
k1s0 lint --fix

# 特定ルールのみ実行
k1s0 lint --rules K001,K002

# 警告をエラーとして扱う
k1s0 lint --strict

# JSON 形式で出力
k1s0 lint --json
```

### テンプレート更新

```bash
# 更新の確認（差分表示）
k1s0 upgrade --check

# テンプレート更新の適用
k1s0 upgrade
```

### その他

```bash
# テンプレートレジストリ操作
k1s0 registry list
k1s0 registry add <name> <url>

# シェル補完スクリプト生成
k1s0 completions bash
k1s0 completions zsh
k1s0 completions powershell
```

## 共通オプション

- `-v, --verbose`: 詳細出力
- `--no-color`: カラー出力無効化
- `--json`: JSON 形式出力

## Lint ルール一覧

| ルール ID | 重要度 | 説明 | 自動修正 |
|-----------|--------|------|----------|
| K001 | Error | manifest.json が存在しない | - |
| K002 | Error | manifest.json の必須キーが不足 | - |
| K003 | Error | manifest.json の値が不正 | - |
| K010 | Error | 必須ディレクトリが存在しない | ✓ |
| K011 | Error | 必須ファイルが存在しない | ✓ |
| K020 | Error | 環境変数参照の禁止 | - |
| K021 | Error | config YAML への機密直書き禁止 | - |
| K022 | Error | Clean Architecture 依存方向違反 | - |
| K030 | Warning | gRPC リトライ設定の検出 | - |
| K031 | Warning | gRPC リトライ設定に ADR 参照なし | - |
| K032 | Warning | gRPC リトライ設定が不完全 | - |

## テンプレート種類

- `backend-rust`: Rust マイクロサービス
- `backend-go`: Go マイクロサービス
- `frontend-react`: React 画面
- `frontend-flutter`: Flutter モバイル画面
