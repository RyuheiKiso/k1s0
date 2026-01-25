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

## React 共通パッケージ（予定）

| パッケージ | 説明 |
|-----------|------|
| `k1s0-config` | YAML 設定読み込み |
| `k1s0-http` | API クライアント |
| `k1s0-auth-client` | 認証クライアント |
| `k1s0-observability` | OTel/ログ |
| `k1s0-ui` | 共通 UI（Design System） |

## Flutter 共通パッケージ（予定）

| パッケージ | 説明 |
|-----------|------|
| `k1s0_config` | YAML 設定読み込み |
| `k1s0_http` | API クライアント |
| `k1s0_auth` | 認証クライアント |
| `k1s0_observability` | OTel/ログ |
| `k1s0_ui` | 共通 UI |

## ステータス

置き場のみ固定。実装は後続フェーズで行う。
