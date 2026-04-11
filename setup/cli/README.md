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
