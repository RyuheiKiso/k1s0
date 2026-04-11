# gui

グラフィカルな画面から開発環境のセットアップを行うアプリケーションです。

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
| Node.js | 22以上（LTS） |
| Tauri CLI | 最新 |

Tauri CLI のインストール:

```bash
cargo install tauri-cli
```

---

## 技術スタック

| 項目 | 内容 |
|------|------|
| フレームワーク | Tauri |
| 言語（バックエンド） | Rust Edition 2024 |
| フロントエンド | React / TypeScript |
| UIコンポーネント | MUI (Material UI) |
| ビルドツール | Vite |
| 共通ロジック | `common` クレート |

---

## ディレクトリ構成

```
gui/
├── src/                        # フロントエンド（React）
│   ├── main.tsx                # Reactエントリーポイント
│   ├── App.tsx                 # ルートコンポーネント
│   └── components/             # UIコンポーネント
├── src-tauri/
│   ├── src/
│   │   └── main.rs             # Tauriバックエンドのエントリーポイント
│   ├── Cargo.toml              # パッケージ設定
│   └── tauri.conf.json         # Tauriアプリケーション設定
├── index.html                  # HTMLエントリーポイント
├── package.json                # フロントエンド依存関係
└── vite.config.ts              # Viteビルド設定
```

---

## ビルド・実行

```bash
# 依存関係のインストール（gui/ ディレクトリで実行）
npm install

# 開発モードで起動（ホットリロードあり）
cargo tauri dev

# リリースビルド
cargo tauri build
```

---

## テスト

```bash
# フロントエンドのテスト
npm test

# バックエンド（Rust）のテスト
cd src-tauri && cargo test
```

---

## ダウンロード

[GitHub Releases](https://github.com/RyuheiKiso/k1s0/releases) から最新バージョンをダウンロードしてください。

| OS | ファイル名 |
|----|-----------|
| Windows | `k1s0-gui_<バージョン>_x64.msi` または `k1s0-gui_<バージョン>_x64-setup.exe` |
| macOS | `k1s0-gui_<バージョン>_aarch64.dmg` |
| Linux | `k1s0-gui_<バージョン>_amd64.AppImage` または `k1s0-gui_<バージョン>_amd64.deb` |
