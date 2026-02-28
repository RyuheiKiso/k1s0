# system-config-server 実装設計

system-config-server の Rust 実装構成を定義する。概要・API 定義・アーキテクチャは [system-config-server.md](server.md) を参照。

> **ガイド**: 設計背景・実装例は [implementation.guide.md](./implementation.guide.md) を参照。

---

## Rust 実装 (regions/system/server/rust/config/)

### ディレクトリ構成

```
regions/system/server/rust/config/
├── src/
│   ├── main.rs                          # エントリポイント
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── config_entry.rs          # ConfigEntry エンティティ
│   │   │   └── config_change_log.rs     # ConfigChangeLog エンティティ
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── config_repository.rs     # ConfigRepository トレイト
│   │   │   └── config_change_log_repository.rs  # ConfigChangeLogRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── config_domain_service.rs # namespace バリデーション・バージョン検証
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── get_config.rs
│   │   ├── list_configs.rs
│   │   ├── update_config.rs
│   │   ├── delete_config.rs
│   │   └── get_service_config.rs
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── rest_handler.rs          # axum REST ハンドラー
│   │   │   ├── grpc_handler.rs          # tonic gRPC ハンドラー
│   │   │   └── error.rs                 # エラーレスポンス
│   │   ├── presenter/
│   │   │   ├── mod.rs
│   │   │   └── response.rs
│   │   └── middleware/
│   │       ├── mod.rs
│   │       ├── auth.rs                  # JWT 認証ミドルウェア
│   │       └── rbac.rs                  # RBAC ミドルウェア
│   └── infrastructure/
│       ├── mod.rs
│       ├── config/
│       │   ├── mod.rs
│       │   └── logger.rs
│       ├── persistence/
│       │   ├── mod.rs
│       │   ├── db.rs
│       │   ├── config_repository.rs
│       │   └── config_change_log_repository.rs
│       ├── cache/
│       │   ├── mod.rs
│       │   └── config_cache.rs          # インメモリキャッシュ
│       └── messaging/
│           ├── mod.rs
│           └── producer.rs              # Kafka プロデューサー
├── api/
│   └── proto/
│       └── k1s0/system/config/v1/
│           └── config.proto
├── migrations/
│   ├── 001_create_config_entries.sql
│   └── 002_create_config_change_logs.sql
├── config/
│   ├── config.yaml
│   ├── config.dev.yaml
│   ├── config.staging.yaml
│   └── config.prod.yaml
├── build.rs                             # tonic-build（proto コンパイル）
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
└── README.md
```

### Cargo.toml

> 共通依存は [Rust共通実装.md](../_common/Rust共通実装.md#共通cargo依存) を参照。サービス固有の追加依存:

```toml
# キャッシュ
moka = { version = "0.12", features = ["future"] }
```

### build.rs

> build.rs パターンは [Rust共通実装.md](../_common/Rust共通実装.md#共通buildrs) を参照。proto パス: `api/proto/k1s0/system/config/v1/config.proto`

---

## config.yaml サービス固有設定

| セクション | フィールド | 型 | デフォルト | 説明 |
|-----------|-----------|-----|-----------|------|
| `config_server.cache` | `ttl` | string | `"60s"` | キャッシュの TTL |
| `config_server.cache` | `max_entries` | int | `10000` | キャッシュの最大エントリ数 |
| `config_server.cache` | `refresh_on_miss` | bool | `true` | キャッシュミス時にバックグラウンドリフレッシュ |
| `config_server.audit` | `kafka_enabled` | bool | `true` | Kafka への非同期配信を有効化 |
| `config_server.audit` | `retention_days` | int | `365` | DB 内の保持日数 |
| `config_server.namespace` | `allowed_tiers` | string[] | `["system","business","service"]` | 許可される Tier |
| `config_server.namespace` | `max_depth` | int | `4` | namespace の最大階層数 |

---

## 関連ドキュメント

- [system-config-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [system-config-server-deploy.md](deploy.md) -- キャッシュ戦略・DB マイグレーション・テスト・Dockerfile・Helm values
- [config.md](../../cli/config/config設計.md) -- config.yaml スキーマと環境別管理
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) -- Linter・Formatter・命名規則
