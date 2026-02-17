# k1s0 CLI / GUI

k1s0 プロジェクトの開発ツール群。対話式 CLI と Tauri デスクトップ GUI の 2 つのインターフェースで、プロジェクト初期化・ひな形生成・ビルド・テスト・デプロイを実行する。

## アーキテクチャ

```
CLI/
├── Cargo.toml                 # Workspace ルート
├── crates/
│   ├── k1s0-core/             # 共有ライブラリ（ビジネスロジック）
│   ├── k1s0-cli/              # 対話式 CLI（dialoguer）
│   └── k1s0-gui/              # Tauri v2 デスクトップ GUI
│       ├── src/               #   Rust バックエンド（Tauri コマンド）
│       └── ui/                #   React フロントエンド
└── rustfmt.toml
```

```
k1s0-cli ──→ k1s0-core ←── k1s0-gui (Tauri)
(TUI)         (共有)         (GUI)
```

CLI と GUI は同一の `k1s0-core` を共有し、機能の等価性を保証する。

## 前提条件

| ツール | バージョン | 用途 |
|--------|-----------|------|
| Rust | 2021 edition | CLI / Core / GUI バックエンド |
| Node.js | 20+ | GUI フロントエンド |
| npm | 10+ | パッケージ管理 |
| Tauri CLI | 2.x | GUI ビルド（`cargo install tauri-cli --version "^2"` で導入） |

## ビルド

### CLI のビルド

```bash
cd CLI

# デバッグビルド
cargo build -p k1s0-cli

# リリースビルド
cargo build -p k1s0-cli --release

# 実行ファイルの場所
# デバッグ: target/debug/k1s0-cli
# リリース: target/release/k1s0-cli
```

### Core ライブラリのビルド

```bash
cargo build -p k1s0-core
```

### GUI のビルド

```bash
cd CLI

# フロントエンドの依存インストール（初回のみ）
cd crates/k1s0-gui/ui && npm install && cd ../../../

# Tauri 開発サーバー起動（ホットリロード対応）
cargo tauri dev

# リリースビルド（インストーラー生成）
cargo tauri build
```

フロントエンド単体のビルド:

```bash
cd CLI/crates/k1s0-gui/ui

npm run build    # TypeScript コンパイル + Vite バンドル
npm run dev      # Vite 開発サーバー (http://localhost:5173)
```

### ワークスペース全体のビルド

```bash
cd CLI
cargo build --workspace
```

## テスト

### Rust テスト

```bash
cd CLI

# 全テスト実行
cargo test --workspace

# クレート別
cargo test -p k1s0-core       # Core ライブラリ
cargo test -p k1s0-cli        # CLI
cargo test -p k1s0-gui        # GUI バックエンド（Tauri コマンド）

# 特定テストの実行
cargo test -p k1s0-core test_validate_name

# スナップショットテスト更新（insta）
cargo insta test --workspace
cargo insta review             # 差分を確認して承認
```

### React テスト（GUI フロントエンド）

```bash
cd CLI/crates/k1s0-gui/ui

npm test             # Vitest 実行（ウォッチモード）
npm test -- --run    # 一回実行して終了

# 特定ファイルのテスト
npx vitest run src/pages/__tests__/GeneratePage.test.tsx

# カバレッジ
npx vitest run --coverage
```

### 全テスト一括実行

```bash
cd CLI
cargo test --workspace && cd crates/k1s0-gui/ui && npm test -- --run && cd ../../..
```

## デバッグ

### CLI のデバッグ

**VS Code (launch.json)**

```jsonc
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug k1s0-cli",
      "cargo": {
        "args": ["build", "--bin=k1s0-cli", "--package=k1s0-cli"],
        "filter": { "name": "k1s0-cli", "kind": "bin" }
      },
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

> CodeLLDB 拡張機能（`vadimcn.vscode-lldb`）が必要。

**ログ出力**

```bash
# RUST_LOG 環境変数でログレベルを制御
RUST_LOG=debug cargo run -p k1s0-cli

# 特定モジュールのみ
RUST_LOG=k1s0_core::template=trace cargo run -p k1s0-cli
```

**cargo-watch で自動再ビルド**

```bash
cargo install cargo-watch
cargo watch -x "run -p k1s0-cli"
```

### GUI のデバッグ

**Tauri 開発モード**

```bash
cd CLI
cargo tauri dev
```

`cargo tauri dev` は以下を同時実行する:
- Vite 開発サーバー (ポート 5173) — フロントエンドのホットリロード
- Rust バックエンドのビルド・起動 — ソース変更時に自動リビルド

**フロントエンド（React）のデバッグ**

- `cargo tauri dev` 起動後、Tauri ウィンドウで右クリック → 「要素を検証」で DevTools を開く
- React コンポーネントの状態確認には [React Developer Tools](https://react.dev/learn/react-developer-tools) をインストール
- `console.log` によるデバッグ出力は DevTools の Console タブに表示される

**バックエンド（Rust）のデバッグ**

```jsonc
// VS Code launch.json — Tauri バックエンドのデバッグ
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug k1s0-gui",
      "cargo": {
        "args": ["build", "--bin=k1s0-gui", "--package=k1s0-gui"],
        "filter": { "name": "k1s0-gui", "kind": "bin" }
      },
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug"
      }
    }
  ]
}
```

> Tauri DevTools を使う場合は `cargo tauri dev` の起動時に自動で WebView の開発者ツールが有効になる。

**React テストのデバッグ**

```bash
cd CLI/crates/k1s0-gui/ui

# デバッグモードでテスト（Node inspector）
node --inspect-brk ./node_modules/.bin/vitest run

# VS Code の JavaScript Debug Terminal でテスト実行
npx vitest run --reporter=verbose
```

### Core ライブラリのデバッグ

```bash
# 特定テストをデバッグ出力付きで実行
RUST_LOG=debug cargo test -p k1s0-core test_name -- --nocapture

# println! の出力を表示
cargo test -p k1s0-core -- --nocapture
```

## リント・フォーマット

```bash
cd CLI

# Rust
cargo fmt --all              # フォーマット
cargo fmt --all -- --check   # フォーマットチェック
cargo clippy --workspace     # リント

# React
cd crates/k1s0-gui/ui
npm run lint                 # ESLint
npx tsc --noEmit             # TypeScript 型チェック
```

## クレート構成

### k1s0-core

共有ライブラリ。CLI と GUI の両方から利用される。

| モジュール | 責務 |
|-----------|------|
| `commands::init` | プロジェクト初期化 |
| `commands::generate` | ひな形生成（種別・Tier・言語の組み合わせ） |
| `commands::build` | ビルド実行 |
| `commands::test_cmd` | テスト実行 |
| `commands::deploy` | デプロイ実行 |
| `config` | CLI 設定の読み込み・バリデーション |
| `template` | Tera テンプレート処理 |
| `progress` | 進捗イベント（`ProgressEvent`） |
| `validation` | 名前バリデーション（`[a-z0-9-]+`） |

### k1s0-cli

対話式 CLI。`dialoguer` によるターミナルプロンプトで `k1s0-core` のコマンドを呼び出す。

### k1s0-gui

Tauri v2 デスクトップ GUI。React フロントエンドから `#[tauri::command]` 経由で `k1s0-core` を呼び出す。

| 技術 | 用途 |
|------|------|
| Tauri 2 | デスクトップフレームワーク |
| React 19 + TypeScript | フロントエンド |
| TanStack Router | ページルーティング |
| Radix UI | アクセシブルな UI コンポーネント |
| Tailwind CSS v4 | スタイリング |
| React Hook Form + Zod | フォーム・バリデーション |
| Vitest + Testing Library | フロントエンドテスト |

## テンプレート

`crates/k1s0-cli/templates/` に 11 種のテンプレートを格納:

| テンプレート | 内容 |
|-------------|------|
| server | バックエンドサーバー（Go / Rust） |
| client | フロントエンド（React / Flutter） |
| library | 共有ライブラリ |
| database | データベース定義 |
| bff | GraphQL BFF |
| helm | Kubernetes Helm チャート |
| cicd | CI/CD パイプライン |
| docker-compose | Docker Compose |
| devcontainer | Dev Container |
| terraform | インフラ（Terraform） |
| service-mesh | サービスメッシュ |

## 関連ドキュメント

- [CLIフロー](../docs/CLIフロー.md) — CLI の対話フロー
- [TauriGUI設計](../docs/TauriGUI設計.md) — GUI の設計仕様
- [tier-architecture](../docs/tier-architecture.md) — 3 階層アーキテクチャ
- [コンセプト](../docs/コンセプト.md) — 技術スタック・設計思想
