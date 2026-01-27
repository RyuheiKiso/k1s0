# React 開発エージェント

React/TypeScript コードの開発、ビルド、型チェックを支援するエージェント。

## 対象領域

- `framework/frontend/react/` - 共通 React パッケージ（モノレポ）
- `feature/frontend/react/` - 個別 React フロントエンド

## パッケージ構成

### 共通パッケージ (framework/frontend/react/packages/)

- `k1s0-api-client`: API クライアント（REST）
- `k1s0-config`: YAML 設定の読み込み・型付け
- `k1s0-navigation`: 設定駆動ナビゲーション（実装済み）
- `k1s0-shell`: AppShell（Header/Footer/Menu）
- `k1s0-ui`: Design System（MUI ベース）

## 主な操作

### 依存関係

```bash
cd framework/frontend/react

# 依存関係インストール
pnpm install

# 依存関係更新
pnpm update
```

### ビルド・型チェック

```bash
cd framework/frontend/react

# ビルド
pnpm build

# 型チェック
pnpm typecheck

# 開発サーバー起動
pnpm dev
```

### テスト

```bash
cd framework/frontend/react

# テスト実行
pnpm test

# テスト（watch モード）
pnpm test --watch

# カバレッジ付きテスト
pnpm test --coverage
```

### Lint・フォーマット

```bash
cd framework/frontend/react

# ESLint
pnpm lint

# Prettier
pnpm format
```

## 技術スタック

- **React**: Web フロントエンド
- **TypeScript**: 型安全な開発
- **Material UI (MUI)**: Design System
- **React Router**: ナビゲーション
- **zod**: スキーマバリデーション
- **pnpm**: パッケージマネージャー（ワークスペース対応）

## 設定ファイル

- `tsconfig.base.json`: 共通 TypeScript 設定
- `pnpm-workspace.yaml`: ワークスペース定義
- `package.json`: ルートパッケージ設定
