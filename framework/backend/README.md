# Framework Backend

バックエンド共通部品（crate/ライブラリ）および共通マイクロサービス。

## ディレクトリ構成

```
backend/
├── rust/
│   ├── crates/       # 共通 crate 群
│   └── services/     # 共通マイクロサービス
└── go/
    └── pkg/          # 共通パッケージ（予定）
```

## Rust

### 共通 crate（Tier 1-3）

| crate | 説明 | Tier | ステータス |
|-------|------|:----:|:--------:|
| `k1s0-error` | 統一エラーハンドリング | 1 | ✅ |
| `k1s0-config` | 設定ファイル管理 | 1 | ✅ |
| `k1s0-validation` | 入力バリデーション | 1 | ✅ |
| `k1s0-observability` | ロギング/トレーシング/メトリクス | 2 | ✅ |
| `k1s0-grpc-server` | gRPC サーバー基盤 | 2 | ✅ |
| `k1s0-grpc-client` | gRPC クライアント | 2 | ✅ |
| `k1s0-resilience` | リトライ/サーキットブレーカー | 2 | ✅ |
| `k1s0-health` | ヘルスチェック | 2 | ✅ |
| `k1s0-db` | DB 接続/トランザクション | 2 | ✅ |
| `k1s0-cache` | Redis キャッシュ | 2 | ✅ |
| `k1s0-auth` | 認証/認可 | 3 | ✅ |

### 共通マイクロサービス

| サービス | 説明 | ステータス |
|----------|------|:--------:|
| `auth-service` | 認証/認可サービス | ✅ |
| `config-service` | 設定管理サービス | ✅ |
| `endpoint-service` | エンドポイント管理サービス | ✅ |

## Go

置き場のみ固定。実装は後続フェーズで行う。
