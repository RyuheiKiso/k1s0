# Framework Frontend

フロントエンド共通パッケージ。

## ディレクトリ構成

```
frontend/
├── react/
│   └── packages/     # React 共通パッケージ
├── flutter/
│   └── packages/     # Flutter 共通パッケージ
└── android/
    └── packages/     # Android 共通パッケージ
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
| `@k1s0/realtime` | WebSocket/SSE クライアント（再接続・ハートビート・オフラインキュー） | ✅ |
| `eslint-config-k1s0` | ESLint 設定 | ✅ |
| `tsconfig-k1s0` | TypeScript 設定 | ✅ |

## Flutter 共通パッケージ

| パッケージ | 説明 | ステータス |
|-----------|------|:--------:|
| `k1s0_navigation` | Config-driven ルーティング（go_router） | ✅ |
| `k1s0_config` | YAML 設定読み込み | ✅ |
| `k1s0_http` | API クライアント（Dio） | ✅ |
| `k1s0_auth` | 認証クライアント | ✅ |
| `k1s0_observability` | OTel/ログ | ✅ |
| `k1s0_ui` | 共通 UI（Material 3） | ✅ |
| `k1s0_state` | 状態管理（Riverpod） | ✅ |
| `k1s0_realtime` | WebSocket/SSE クライアント（再接続・ハートビート・オフラインキュー） | ✅ |

## Android 共通パッケージ

| パッケージ | 説明 | ステータス |
|-----------|------|:--------:|
| `k1s0-navigation` | Navigation Compose ルーティング | ✅ |
| `k1s0-config` | YAML 設定管理 | ✅ |
| `k1s0-http` | Ktor Client HTTP | ✅ |
| `k1s0-ui` | Material 3 デザインシステム | ✅ |
| `k1s0-auth` | JWT 認証クライアント | ✅ |
| `k1s0-observability` | ログ・トレーシング | ✅ |
| `k1s0-state` | ViewModel + StateFlow ユーティリティ | ✅ |
| `k1s0-realtime` | WebSocket/SSE クライアント | ✅ |
