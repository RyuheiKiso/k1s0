# gui

グラフィカルな画面から開発環境のセットアップを行うアプリケーションです。

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
# 依存関係のインストール
npm install

# 開発モードで起動（ホットリロードあり）
cargo tauri dev

# リリースビルド
cargo tauri build
```
