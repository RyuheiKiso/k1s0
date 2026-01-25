# Framework Frontend React

React 共通パッケージ。

## 概要

個別機能チームが「画面の中身」以外（ナビゲーション/レイアウト/デザイン/権限制御/設定読込/観測）を再実装せずに済む状態を提供する。

## ディレクトリ構成

```
react/
├── package.json
├── pnpm-workspace.yaml
├── tsconfig.base.json
└── packages/
    ├── k1s0-navigation/     # 設定駆動ナビゲーション（実装済み）
    ├── k1s0-config/         # YAML設定の読み込み/型付け/バリデーション
    ├── k1s0-http/           # APIクライアント
    ├── k1s0-auth-client/    # 認証クライアント
    ├── k1s0-observability/  # OTel/ログ/trace_id 相関
    ├── k1s0-ui/             # 共通UI（Design System）
    ├── k1s0-shell/          # AppShell（Header/Footer/Menu）
    ├── k1s0-state/          # 状態管理
    ├── eslint-config-k1s0/  # ESLint ルール
    └── tsconfig-k1s0/       # TypeScript 設定
```

## パッケージ一覧

### @k1s0/navigation（実装済み）

設定駆動ナビゲーションライブラリ。

- `config/{env}.yaml` の `ui.navigation` からルート/メニュー/フローを読み込み
- React Router への自動反映
- 権限/feature flag による表示/遷移制御
- zod によるスキーマバリデーション

詳細は [packages/k1s0-navigation/README.md](./packages/k1s0-navigation/README.md) を参照。

### その他パッケージ

後続フェーズで実装予定。

## 開発

```bash
# 依存関係のインストール
pnpm install

# ビルド
pnpm build

# 型チェック
pnpm typecheck

# テスト
pnpm test
```

## 規約

- UI コンポーネントは MUI（Material UI）を標準とする
- Header / Footer / Menu は framework が提供する
- 画面遷移（routes/menus/flows）は設定で制御する
- ナビゲーション設定は `config/{env}.yaml` を正本とする
