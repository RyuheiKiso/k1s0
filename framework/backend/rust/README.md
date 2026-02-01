# Framework Backend Rust

Rust バックエンド共通部品および共通マイクロサービス。

## ディレクトリ構成

```
rust/
├── crates/                   # 共通 crate 群
│   ├── k1s0-auth/
│   ├── k1s0-cache/
│   ├── k1s0-config/
│   ├── k1s0-db/
│   ├── k1s0-error/
│   ├── k1s0-grpc-client/
│   ├── k1s0-grpc-server/
│   ├── k1s0-health/
│   ├── k1s0-observability/
│   ├── k1s0-rate-limit/
│   ├── k1s0-resilience/
│   ├── k1s0-domain-event/
│   ├── k1s0-consensus/
│   └── k1s0-validation/
└── services/                 # 共通マイクロサービス
    ├── auth-service/
    ├── config-service/
    └── endpoint-service/
```

## 共通 Crates

### Tier 1: コア基盤（実装済み）

| Crate | 説明 | 状態 |
|-------|------|------|
| `k1s0-error` | エラー表現の統一（層別責務、HTTP/gRPC変換） | ✅ 実装済み |
| `k1s0-config` | 設定読み込み（CLI引数 + YAML + secrets） | ✅ 実装済み |
| `k1s0-validation` | 入力バリデーション（REST + gRPC対応） | ✅ 実装済み |

### Tier 2: インフラ統合（実装済み）

| Crate | 説明 | 状態 |
|-------|------|------|
| `k1s0-observability` | ログ/トレース/メトリクス統合（OTel対応） | ✅ 実装済み |
| `k1s0-grpc-server` | gRPCサーバ基盤（deadline検知、error_code必須化） | ✅ 実装済み |
| `k1s0-grpc-client` | gRPCクライアント基盤（deadline必須、retry原則禁止） | ✅ 実装済み |
| `k1s0-resilience` | 耐障害性パターン（Timeout/Bulkhead/CircuitBreaker） | ✅ 実装済み |
| `k1s0-rate-limit` | レート制限（トークンバケット、スライディングウィンドウ） | ✅ 実装済み |
| `k1s0-health` | Kubernetesプローブ対応（readyz/livez/startupz） | ✅ 実装済み |
| `k1s0-db` | DB接続プール、トランザクション、リポジトリパターン | ✅ 実装済み |
| `k1s0-cache` | Redisクライアント、Cache-Asideパターン | ✅ 実装済み |
| `k1s0-domain-event` | ドメインイベント publish/subscribe/outbox | ✅ 実装済み |
| `k1s0-consensus` | リーダー選出、分散ロック、Saga オーケストレーション | ✅ 実装済み |

### Tier 3: 業務特化（実装済み）

| Crate | 説明 | 状態 |
|-------|------|------|
| `k1s0-auth` | JWT/OIDC検証、ポリシー評価、監査ログ | ✅ 実装済み |

## 共通サービス

| サービス | 説明 | 状態 |
|---------|------|------|
| auth-service | 認証・認可 | 計画中 |
| config-service | 動的設定 | 計画中 |
| endpoint-service | エンドポイント管理 | 計画中 |

## ビルド

```bash
cd framework/backend/rust
cargo build --workspace
```
