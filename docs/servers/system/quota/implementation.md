# system-quota-server 実装設計

> **注記**: 本ドキュメントは quota-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../_common/Rust共通実装.md) を参照。

system-quota-server（クォータサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（クォータポリシー管理・使用量操作・超過検知） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・Redis・Kafka・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/quota/)

### ディレクトリ構成

```
regions/system/server/rust/quota/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   └── quota.rs                     # QuotaPolicy / QuotaUsage エンティティ
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   └── quota_repository.rs          # QuotaRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── quota_domain_service.rs      # 超過判定・リセットロジック
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── create_quota_policy.rs           # クォータポリシー作成
│   │   ├── update_quota_policy.rs           # クォータポリシー更新
│   │   ├── delete_quota_policy.rs           # クォータポリシー削除
│   │   ├── get_quota_policy.rs              # クォータポリシー取得
│   │   ├── list_quota_policies.rs           # クォータポリシー一覧
│   │   ├── get_quota_usage.rs               # 使用量照会
│   │   ├── increment_quota_usage.rs         # 使用量インクリメント + 超過検知
│   │   └── reset_quota_usage.rs             # 使用量リセット
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── quota_handler.rs             # axum REST ハンドラー
│   │   │   └── health.rs                    # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── quota_grpc.rs                # gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                      # JWT 認証ミドルウェア
│   │   │   └── rbac.rs                      # RBAC ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── quota_policy_postgres.rs     # QuotaPolicyRepository PostgreSQL 実装
│   │       └── quota_usage_postgres.rs      # QuotaUsageRepository PostgreSQL 実装
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                        # 設定構造体・読み込み
│   │   ├── database.rs                      # DB 接続プール
│   │   ├── redis_store.rs                   # Redis 使用量カウンター（INCR + EXPIRE）
│   │   ├── kafka_producer.rs                # Kafka プロデューサー（超過通知）
│   │   └── startup.rs                       # 起動シーケンス・DI
│   └── proto/                               # tonic-build 生成コード
├── config/
│   └── config.yaml
├── build.rs
├── Cargo.toml
└── Dockerfile
```

### 主要コンポーネント

#### ドメインサービス

- **QuotaDomainService**: 超過判定（使用量 vs 閾値比較）とリセットロジック。日次・月次リセットは tokio スケジューラーで自動実行する

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `CreateQuotaPolicyUseCase` 等 | クォータポリシーの CRUD |
| `GetQuotaUsageUseCase` | Redis を活用した低レイテンシな残量照会 |
| `IncrementQuotaUsageUseCase` | Redis INCR によるアトミック加算 + 超過時 Kafka イベント発行 |
| `ResetQuotaUsageUseCase` | 使用量リセット（手動/自動） |

#### Redis 連携

- **Redis Store** (`infrastructure/redis_store.rs`): `quota:{policy_id}:{period}` キー形式で使用量カウンターを管理する。INCR + EXPIRE によるアトミック加算を行う
- deadpool-redis を使用した接続プール管理

#### Kafka 連携

- **Producer** (`infrastructure/kafka_producer.rs`): 超過検知時に `k1s0.system.quota.exceeded.v1` を発行し、notification-server 経由で通知する

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_QUOTA_`
- Redis 障害時はフェイルオープン（使用量チェックをパスさせる）

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | 超過判定・リセットロジック | mockall によるリポジトリモック |
| 統合テスト | REST/gRPC ハンドラー | axum-test / tonic テストクライアント |
| Redis テスト | カウンター操作 | テスト用 Redis インスタンス |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・Redis 設計
- [Rust共通実装.md](../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
