# cli

コマンドラインから開発環境のセットアップを行うツールです。

---

## 対応プラットフォーム

- Windows 10/11
- macOS
- Linux

---

## 前提条件

| ツール | バージョン |
|-------|-----------|
| Rust | stable（最新） |

---

## 技術スタック

| 項目 | 内容 |
|------|------|
| 言語 | Rust Edition 2024 |
| 共通ロジック | `common` クレート |

---

## ディレクトリ構成

```
cli/
├── src/
│   └── main.rs     # エントリーポイント
└── Cargo.toml      # パッケージ設定
```

---

## ビルド・実行

```bash
# ビルド
cargo build --release

# 実行（Linux / macOS）
./target/release/k1s0-cli <コマンド>

# 実行（Windows）
.\target\release\k1s0-cli.exe <コマンド>
```

---

## テスト

```bash
cargo test
```

---

## デバッグ

### デバッグビルドと実行

```bash
# デバッグビルド（最適化なし・デバッグ情報付き）
cargo build

# デバッグビルドを直接実行する（-- 以降がk1s0-cliへの引数）
cargo run -- install-check
cargo run -- help
```

---

### 環境変数

| 環境変数 | 値 | 説明 |
|---|---|---|
| `RUST_BACKTRACE` | `1` | パニック時にスタックトレースを表示する |
| `RUST_BACKTRACE` | `full` | パニック時に詳細なスタックトレースを表示する |

```bash
# Windows（PowerShell）
$env:RUST_BACKTRACE=1; .\target\debug\k1s0-cli.exe install-check

# Linux / macOS
RUST_BACKTRACE=1 ./target/debug/k1s0-cli install-check
```

---

### テストのデバッグ出力

```bash
# テスト中の println! 出力をそのまま表示する
cargo test -- --nocapture

# 特定のテストのみ実行する
cargo test test_install_check_command_ok -- --nocapture
```

---

### VS Code でのデバッグ

`.vscode/launch.json` に以下を追加することで、ステップ実行が可能です。

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "k1s0-cli: install-check",
      "cargo": {
        "args": ["build", "--bin=k1s0-cli", "--package=k1s0-cli"]
      },
      "args": ["install-check"],
      "cwd": "${workspaceFolder}/setup"
    }
  ]
}
```

> VS Code でのデバッグには [CodeLLDB 拡張機能](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) が必要です。

---

### よくある問題

| 症状 | 原因 | 対処 |
|---|---|---|
| `[NG] Node.js` と表示される | `node` コマンドが PATH にない | Node.js をインストールし PATH を通す |
| `[NG] Go` と表示される | `go` コマンドが PATH にない | Go をインストールし PATH を通す |
| `[NG] Git` と表示される | `git` コマンドが PATH にない | Git をインストールし PATH を通す |
| `エラー: 不明なコマンド` が表示される | コマンド名のタイプミス | `k1s0-cli help` でコマンド一覧を確認する |

---

## ダウンロード

[GitHub Releases](https://github.com/RyuheiKiso/k1s0/releases) から最新バージョンをダウンロードしてください。

| OS | ファイル名 |
|----|-----------|
| Windows | `k1s0-cli.exe` |
| macOS | `k1s0-cli`（実行権限の付与が必要） |
| Linux | `k1s0-cli`（実行権限の付与が必要） |

macOS / Linux の場合は、ダウンロード後に以下のコマンドで実行権限を付与してください。

```bash
chmod +x k1s0-cli
./k1s0-cli <コマンド>
```

---

## 利用可能なコマンド（随時追加予定）

| コマンド | 説明 |
|---------|------|
| `install-check` | Node.js・Rust・Go・Git のインストール状態を確認する |
| `help` | コマンド一覧と使い方を表示する |
