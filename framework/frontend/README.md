# Framework Frontend

フロントエンド共通パッケージ。

## ディレクトリ構成

```
frontend/
├── react/
│   └── packages/     # React 共通パッケージ
└── flutter/
    └── packages/     # Flutter 共通パッケージ
```

## React 共通パッケージ

| パッケージ | 説明 | ステータス |
|-----------|------|:--------:|
| `@k1s0/config` | YAML 設定読み込み | ✅ |
| `@k1s0/api-client` | API クライアント | ✅ |
| `@k1s0/auth-client` | 認証クライアント | ✅ |
| `@k1s0/observability` | OTel/ログ | ✅ |
| `@k1s0/ui` | 共通 UI（Design System） | ✅ |
| `@k1s0/navigation` | Config-driven ルーティング | ✅ |
| `@k1s0/shell` | AppShell（Header/Sidebar/Footer） | ✅ |
| `eslint-config-k1s0` | ESLint 設定 | ✅ |
| `tsconfig-k1s0` | TypeScript 設定 | ✅ |

## Flutter 共通パッケージ

| パッケージ | 説明 | ステータス |
|-----------|------|:--------:|
| `k1s0_config` | YAML 設定読み込み | ✅ |
| `k1s0_http` | API クライアント | ✅ |
| `k1s0_auth` | 認証クライアント | ✅ |
| `k1s0_observability` | OTel/ログ | ✅ |
| `k1s0_ui` | 共通 UI | ✅ |
| `k1s0_state` | 状態管理（Riverpod） | ✅ |

## ステータス

初期実装完了。テストカバレッジ拡充は継続的に実施。
