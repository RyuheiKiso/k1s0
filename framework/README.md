# k1s0 Framework

開発基盤チームが提供する共通部品（crate/ライブラリ）および共通マイクロサービス。

## ディレクトリ構成

```
framework/
├── backend/
│   ├── rust/
│   │   ├── crates/           # 共通 crate 群
│   │   │   ├── k1s0-config/
│   │   │   ├── k1s0-error/
│   │   │   ├── k1s0-observability/
│   │   │   └── k1s0-grpc-client/
│   │   └── services/         # 共通マイクロサービス
│   │       ├── auth-service/
│   │       ├── config-service/
│   │       └── endpoint-service/
│   └── go/
├── frontend/
│   ├── react/
│   │   └── packages/         # React 共通パッケージ
│   └── flutter/
│       └── packages/         # Flutter 共通パッケージ
└── database/
    └── table/                # 共通テーブル定義（DDL 正本）
```

## 提供物

### Backend Crates（Rust）

| Crate | 説明 |
|-------|------|
| `k1s0-config` | 設定読み込み（`--env`/`--config`/`--secrets-dir`） |
| `k1s0-error` | エラー表現の統一 |
| `k1s0-observability` | ログ/トレース/メトリクス初期化 |
| `k1s0-grpc-client` | gRPC クライアント共通（deadline 必須） |

### 共通サービス

| サービス | 説明 | 所有テーブル |
|---------|------|-------------|
| auth-service | 認証・認可 | `fw_m_user`, `fw_m_role`, `fw_m_permission`, `fw_m_user_role`, `fw_m_role_permission` |
| config-service | 動的設定 | `fw_m_setting` |
| endpoint-service | エンドポイント管理 | `fw_m_endpoint` |

## 依存方向

- `feature` → `framework`: 許可
- `framework` → `feature`: **禁止**

## 関連ドキュメント

- [規約: サービス構成](../docs/conventions/service-structure.md)
- [規約: 設定と秘密情報](../docs/conventions/config-and-secrets.md)
