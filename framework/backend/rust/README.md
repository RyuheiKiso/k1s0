# Framework Backend Rust

Rust バックエンド共通部品および共通マイクロサービス。

## ディレクトリ構成

```
rust/
├── crates/                   # 共通 crate 群
│   ├── k1s0-config/
│   ├── k1s0-error/
│   ├── k1s0-observability/
│   ├── k1s0-grpc-client/
│   └── ...
└── services/                 # 共通マイクロサービス
    ├── auth-service/
    ├── config-service/
    └── endpoint-service/
```

## 共通 Crates

| Crate | 説明 | 実装フェーズ |
|-------|------|-------------|
| `k1s0-config` | 設定読み込み | フェーズ18 |
| `k1s0-error` | エラー表現の統一 | フェーズ19 |
| `k1s0-observability` | ログ/トレース/メトリクス | フェーズ20 |
| `k1s0-grpc-client` | gRPC クライアント共通 | フェーズ21 |

## 共通サービス

| サービス | 説明 | 実装フェーズ |
|---------|------|-------------|
| auth-service | 認証・認可 | フェーズ27 |
| config-service | 動的設定 | フェーズ24-25 |
| endpoint-service | エンドポイント管理 | フェーズ26 |

## ビルド

```bash
cd framework/backend/rust
cargo build --workspace
```
