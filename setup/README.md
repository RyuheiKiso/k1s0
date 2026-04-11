# k1s0 セットアップツール

TypeScript を使用した CLI / GUI 統合ツールです。

## 構成

| モジュール | 技術スタック | 説明 |
|-----------|------------|------|
| `cli`     | TypeScript | コマンドラインインターフェース |
| `gui`     | React Native (Windows / macOS) | グラフィカルユーザーインターフェース |
| `common`  | TypeScript | CLI・GUI 双方から呼び出す共通ロジック |

## ビルド

`pkg` を使用して、Node.js ランタイムを同梱した単一の実行ファイル（`.exe`）を生成する。

```bash
# 依存パッケージのインストール
npm install

# TypeScript をコンパイル
npm run build

# 実行ファイルを生成
npx pkg ./dist/index.js --targets node22-win-x64 --output k1s0.exe
```

| 対象プラットフォーム | `--targets` の値 |
|-------------------|----------------|
| Windows (x64)     | `node22-win-x64` |
| macOS (x64)       | `node22-macos-x64` |
| Linux (x64)       | `node22-linux-x64` |

> Node.js のインストールが不要なため、エンドユーザーへの配布が容易。  
> ただし、ランタイムを同梱するためファイルサイズは 30〜80MB 程度になる。

## コマンド一覧

| コマンド | 説明 |
|---------|------|
| `k1s0 install-check` | Node.js のインストール状況を確認する |


## ディレクトリ構成

npm workspaces を使用して、各モジュールを独立したパッケージとして管理する。  
`cli` および `gui` は `@k1s0/common` をパッケージ参照で利用する。

```
setup/
├── package.json          # ルートパッケージ（workspaces 設定）
├── package-lock.json
├── tsconfig.base.json    # 全モジュール共通の TypeScript 設定
├── cli/
│   ├── package.json      # name: "@k1s0/cli"、dependencies に "@k1s0/common" を指定
│   ├── tsconfig.json     # tsconfig.base.json を継承
│   └── src/
│       └── index.ts      # CLI エントリーポイント
├── common/
│   ├── package.json      # name: "@k1s0/common"
│   ├── tsconfig.json     # tsconfig.base.json を継承
│   └── src/
│       └── index.ts      # 共通ロジックのエクスポート
├── gui/
│   ├── package.json      # name: "@k1s0/gui"、dependencies に "@k1s0/common" を指定
│   ├── tsconfig.json     # tsconfig.base.json を継承
│   └── src/
│       └── index.ts      # GUI エントリーポイント
└── README.md             # このファイル
```

### common の参照方法

`cli` および `gui` から `@k1s0/common` を以下のようにインポートする。

```ts
import { xxx } from "@k1s0/common";
```