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
    ├── k1s0-config/         # YAML設定の読み込み/型付け/バリデーション（実装済み）
    ├── k1s0-api-client/     # APIクライアント（実装済み）
    ├── k1s0-ui/             # 共通UI（Design System）（実装済み）
    ├── k1s0-shell/          # AppShell（Header/Footer/Menu）（実装済み）
    ├── k1s0-auth-client/    # 認証クライアント（未実装）
    ├── k1s0-observability/  # OTel/ログ/trace_id 相関（未実装）
    ├── eslint-config-k1s0/  # ESLint ルール（未実装）
    └── tsconfig-k1s0/       # TypeScript 設定（未実装）
```

## パッケージ一覧

### @k1s0/navigation（実装済み）

設定駆動ナビゲーションライブラリ。

- `config/{env}.yaml` の `ui.navigation` からルート/メニュー/フローを読み込み
- React Router への自動反映
- 権限/feature flag による表示/遷移制御
- zod によるスキーマバリデーション

詳細は [packages/k1s0-navigation/README.md](./packages/k1s0-navigation/README.md) を参照。

### @k1s0/ui（実装済み）

k1s0 Design/UX 標準コンポーネントライブラリ。

| モジュール | 内容 |
|-----------|------|
| `theme/` | K1s0ThemeProvider, createK1s0Theme, palette, typography, spacing, components |
| `form/` | FormContainer, FormField, validation, types |
| `feedback/` | Toast, ConfirmDialog, FeedbackProvider |
| `state/` | Loading, EmptyState |

詳細は [packages/k1s0-ui/README.md](./packages/k1s0-ui/README.md) を参照。

### @k1s0/config（実装済み）

YAML設定の読み込み/型付け/バリデーション。

| 機能 | 内容 |
|-----|------|
| Schema | apiConfigSchema, authConfigSchema, appConfigSchema, validateConfig |
| Loader | ConfigLoader, loadConfigFromUrl, parseConfig, resolveConfigPaths |
| Merge | deepMerge, mergeConfigs, mergeEnvironmentConfig |

### @k1s0/shell（実装済み）

AppShell（Header/Footer/Menu）コンポーネント。

| 機能 | 内容 |
|-----|------|
| Components | AppShell, Header, Sidebar, Footer |
| Hooks | useResponsiveLayout |
| Types | AppShellProps, HeaderProps, SidebarProps, FooterProps, NavItem等 |

### @k1s0/api-client（実装済み）

API通信クライアント。

詳細は [packages/k1s0-api-client/README.md](./packages/k1s0-api-client/README.md) を参照。

### 未実装パッケージ

以下のパッケージは後続フェーズで実装予定。

- `@k1s0/auth-client` - 認証クライアント
- `@k1s0/observability` - OTel/ログ/trace_id 相関
- `eslint-config-k1s0` - ESLint ルール
- `tsconfig-k1s0` - TypeScript 設定

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
