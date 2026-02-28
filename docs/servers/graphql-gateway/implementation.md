# system-graphql-gateway 実装設計

> **ガイド**: 実装コード例・設定ファイル例・テスト例は [implementation.guide.md](./implementation.guide.md) を参照。

概要・API 定義・アーキテクチャは [system-graphql-gateway設計.md](server.md) を参照。

---

## ディレクトリ構成

```
regions/system/server/rust/graphql-gateway/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── model/
│   │   │   ├── mod.rs
│   │   │   ├── tenant.rs          # Tenant, TenantStatus, TenantConnection
│   │   │   ├── feature_flag.rs    # FeatureFlag
│   │   │   ├── config_entry.rs    # ConfigEntry
│   │   │   └── graphql_context.rs # GraphqlContext (user_id, roles, request_id)
│   │   └── loader/
│   │       └── mod.rs             # DataLoader trait 定義
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── tenant_query.rs        # TenantQueryResolver
│   │   ├── feature_flag_query.rs  # FeatureFlagQueryResolver
│   │   ├── config_query.rs        # ConfigQueryResolver
│   │   ├── tenant_mutation.rs     # TenantMutationResolver
│   │   └── subscription.rs        # SubscriptionResolver
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── graphql_handler.rs     # POST/GET /graphql, /graphql/ws
│   │   └── middleware/
│   │       ├── mod.rs
│   │       └── auth_middleware.rs  # JWT 検証 axum layer
│   └── infra/
│       ├── mod.rs
│       ├── config/
│       │   └── mod.rs             # Config struct
│       ├── grpc/
│       │   ├── mod.rs
│       │   ├── tenant_client.rs   # TenantGrpcClient
│       │   ├── feature_flag_client.rs
│       │   └── config_client.rs
│       └── auth/
│           └── jwks.rs            # JWKS 取得・JWT 検証
├── api/
│   └── graphql/
│       └── schema.graphql
├── config/
│   ├── config.yaml
│   ├── config.dev.yaml
│   ├── config.staging.yaml
│   └── config.prod.yaml
└── build.rs
```

---

## 依存クレート

> 共通依存は [Rust共通実装.md](../_common/Rust共通実装.md#共通cargo依存) を参照。

| クレート | バージョン | 用途 |
| --- | --- | --- |
| `axum` | 0.7 | HTTP フレームワーク（`macros`, `ws` feature） |
| `async-graphql` | 7 | GraphQL サーバー（`dataloader` feature） |
| `async-graphql-axum` | 7 | axum 統合 |
| `jsonwebtoken` | 9 | JWT 検証 |
| `reqwest` | 0.12 | JWKS 取得用 HTTP クライアント（`json`, `rustls-tls` feature） |
| `async-trait` | 0.1 | 非同期トレイト |
| `k1s0-telemetry` | path | テレメトリ（`full` feature） |
| `axum-test` | 16 | テスト（dev-dependency） |

### build.rs

gRPC クライアント側のため `build_server(false)` / `build_client(true)`。proto パス: `tenant.proto`, `featureflag.proto`, `config.proto`。

---

## テスト構成

| レイヤー | テスト種別 | 手法 |
| --- | --- | --- |
| domain/model | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/graphql_handler | 統合テスト（HTTP） | `axum-test` + `tokio::test` |
| adapter/middleware | 単体テスト | `tokio::test` + モック JWT |
| infra/auth | 単体テスト | `tokio::test` + `wiremock` |
| infra/grpc | 統合テスト | `tonic` mock + `tokio::test` |

---

## 関連ドキュメント

- [system-graphql-gateway設計.md](server.md) -- 概要・API 定義・アーキテクチャ
- [system-graphql-gateway-deploy.md](deploy.md) -- Dockerfile・Helm values・デプロイ設計
- [proto設計.md](../../architecture/api/proto設計.md) -- ConfigService / TenantService / FeatureFlagService proto 定義
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- JWT Claims 構造・RBAC ロール定義
- [GraphQL設計.md](../../architecture/api/GraphQL設計.md) -- GraphQL 設計ガイドライン
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート仕様
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) -- コーディング規約
