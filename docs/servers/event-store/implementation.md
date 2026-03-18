# system-event-store-server 実装設計

> **注記**: 本ドキュメントは event-store-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../_common/Rust共通実装.md) を参照。

system-event-store-server（イベントストアサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（イベント追記・取得・スナップショット管理） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア | usecase, domain |
| infrastructure | 設定・DB接続・永続化実装・Kafka・InMemory・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/event-store/)

### ディレクトリ構成

```
regions/system/server/rust/event-store/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   └── event.rs                     # Event / EventStream / Snapshot エンティティ
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   └── event_repository.rs          # EventRepository / EventStreamRepository / SnapshotRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── event_store_domain_service.rs # 楽観的ロック・バージョン管理ロジック
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── append_events.rs                 # イベント追記（楽観的ロック付き）
│   │   ├── read_events.rs                   # ストリーム別イベント取得
│   │   ├── read_event_by_sequence.rs        # シーケンス指定の単一イベント取得
│   │   ├── list_events.rs                   # 全イベント一覧取得
│   │   ├── list_streams.rs                  # ストリーム一覧取得
│   │   ├── create_snapshot.rs               # スナップショット作成
│   │   ├── get_latest_snapshot.rs           # 最新スナップショット取得
│   │   └── delete_stream.rs                 # ストリーム削除
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── event_handler.rs             # axum REST ハンドラー
│   │   │   ├── error.rs                     # エラーレスポンス定義
│   │   │   └── health.rs                    # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── event_store_grpc.rs          # gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   └── middleware/
│   │       ├── mod.rs
│   │       ├── auth.rs                      # JWT 認証ミドルウェア
│   │       └── rbac.rs                      # RBAC ミドルウェア
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                        # 設定構造体・読み込み
│   │   ├── database.rs                      # DB 接続プール
│   │   ├── in_memory.rs                     # InMemory リポジトリ（dev/test 用）
│   │   ├── kafka.rs                         # Kafka プロデューサー（イベント転送）+ NoopPublisher
│   │   ├── persistence/
│   │   │   ├── mod.rs
│   │   │   ├── event_postgres.rs            # EventRepository PostgreSQL 実装
│   │   │   ├── stream_postgres.rs           # EventStreamRepository PostgreSQL 実装
│   │   │   └── snapshot_postgres.rs         # SnapshotRepository PostgreSQL 実装
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

- **EventStoreDomainService**: 楽観的ロック（`expected_version` による競合検出）とバージョン管理ロジックを提供する。events テーブルは Append-only（UPDATE/DELETE 禁止）

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `AppendEventsUseCase` | イベント追記。`expected_version` で楽観的ロックを実現する |
| `ReadEventsUseCase` | ストリーム別イベント取得（バージョン範囲指定可） |
| `ReadEventBySequenceUseCase` | シーケンス番号指定の単一イベント取得 |
| `CreateSnapshotUseCase` | 集約状態のスナップショット作成 |
| `GetLatestSnapshotUseCase` | 最新スナップショット取得 |
| `DeleteStreamUseCase` | ストリーム削除（監査・テスト用途に限定） |

#### 永続化

- **persistence/** ディレクトリに PostgreSQL の `event_store` スキーマ（event_streams, events, snapshots テーブル）用のリポジトリ実装を配置する
- DB 未設定時は `in_memory.rs` の InMemory 実装にフォールバックする（dev/test 環境のみ。本番環境では `require_infra` で強制）

#### Kafka 転送

- イベント追記後、バックグラウンドタスクが `k1s0.system.eventstore.event.published.v1` トピックへ非同期転送する（at-least-once 保証）
- Kafka 未設定時は `NoopEventPublisher` にフォールバックする

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_EVSTORE_`
- 楽観的ロック競合時は `SYS_EVSTORE_VERSION_CONFLICT`（409 Conflict）を返す

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | 楽観的ロック・バージョン管理 | mockall によるリポジトリモック |
| InMemory テスト | リポジトリ | `in_memory.rs` による DB 不要テスト |
| 統合テスト | REST/gRPC ハンドラー | axum-test / tonic テストクライアント |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義
- [Rust共通実装.md](../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
