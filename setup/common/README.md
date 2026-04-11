# common

`cli` と `gui` の両モジュールから共通で利用するロジックをまとめたクレートです。

---

## 技術スタック

| 項目 | 内容 |
|------|------|
| 言語 | Rust Edition 2024 |

---

## ディレクトリ構成

```
common/
├── src/
│   └── lib.rs      # ライブラリのエントリーポイント
└── Cargo.toml      # パッケージ設定
```

---

## 利用方法

`cli` の `Cargo.toml` に依存関係を追記します。

```toml
[dependencies]
common = { path = "../common" }
```

`gui` の `src-tauri/Cargo.toml` に依存関係を追記します。

```toml
[dependencies]
common = { path = "../../common" }
```

コード内では以下のようにインポートします。

```rust
use common::install_check;
```

---

## 提供する機能（随時追加予定）

| モジュール | 説明 |
|-----------|------|
| `install_check` | 必要なソフトウェアのインストール確認ロジック |

---

## テスト

```bash
cargo test
```
