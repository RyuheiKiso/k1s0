# system-session-server 実装設計

> **注記**: 本ドキュメントは session-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../_common/Rust共通実装.md) を参照。

system-session-server（セッション管理サーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（セッション CRUD・TTL 延長・失効・マルチデバイス管理） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装・Kafka コンシューマー | usecase, domain |
| infrastructure | 設定・DB接続・Redis接続・Kafka・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/session/)

### ディレクトリ構成

```
regions/system/server/rust/session/
├── src/
│   ├── main.rs                                          # エントリポイント
│   ├── lib.rs                                           # ライブラリルート
│   ├── error.rs                                         # アプリケーションエラー型定義
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   └── session.rs                               # Session エンティティ（is_valid/is_expired/revoke/refresh メソッド）
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   └── session_repository.rs                    # SessionRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── session_domain_service.rs                # TTL 計算・デバイス数制限ロジック
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── create_session.rs                            # セッション作成（デバイス数制限・最古セッション自動失効）
│   │   ├── get_session.rs                               # セッション取得
│   │   ├── refresh_session.rs                           # セッション TTL 延長（スライディング有効期限）
│   │   ├── revoke_session.rs                            # セッション失効（ログアウト）
│   │   ├── revoke_all_sessions.rs                       # 全デバイスセッション一括失効
│   │   └── list_user_sessions.rs                        # ユーザーのアクティブセッション一覧
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── session_handler.rs                       # axum REST ハンドラー
│   │   │   └── health.rs                                # ヘルスチェック（Redis/PostgreSQL/Kafka 疎通確認）
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── session_grpc.rs                          # gRPC サービス実装
│   │   │   └── tonic_service.rs                         # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                                  # JWT 認証ミドルウェア
│   │   │   └── rbac.rs                                  # RBAC ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── session_redis.rs                         # SessionRepository Redis 実装（TTL 付き）
│   │       └── session_metadata_postgres.rs             # セッションメタデータ PostgreSQL 実装
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                                    # 設定構造体・読み込み
│   │   ├── database.rs                                  # PostgreSQL 接続プール
│   │   ├── kafka_consumer.rs                            # Kafka Consumer（revoke_all イベント受信）
│   │   ├── kafka_producer.rs                            # Kafka Producer（created/revoked イベント配信）
│   │   └── startup.rs                                   # 起動シーケンス・DI
│   └── proto/                                           # tonic-build 生成コード
│       ├── mod.rs
│       ├── k1s0.system.session.v1.rs
│       └── k1s0.system.common.v1.rs
├── tests/
│   ├── integration_test.rs                              # 統合テスト
│   └── usecase_test.rs                                  # ユースケーステスト
├── config/
│   └── config.yaml
├── build.rs
├── Cargo.toml
└── Dockerfile
```

### 主要コンポーネント

#### ドメインサービス

- **SessionDomainService**: TTL 計算（デフォルト 3600 秒、最大 86400 秒の制限バリデーション）とデバイス数制限ロジック（1 ユーザー最大 10 デバイス）を担当する。デバイス数超過時は最古セッションの自動失効を判定する

#### Session エンティティメソッド

- `is_valid()` -- セッションが有効か判定（未失効かつ未期限切れ）
- `is_expired()` -- 有効期限切れか判定
- `revoke()` -- セッションを失効状態にする
- `refresh(new_expires_at)` -- 有効期限を延長する

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `CreateSessionUseCase` | セッション作成・Redis TTL 設定・デバイス数制限チェック・PostgreSQL メタデータ記録 |
| `GetSessionUseCase` | セッション ID による取得・有効性確認 |
| `RefreshSessionUseCase` | TTL 延長（スライディング有効期限） |
| `RevokeSessionUseCase` | 単一セッション即時失効・Redis 削除・Kafka イベント発行 |
| `RevokeAllSessionsUseCase` | ユーザー全セッション一括失効 |
| `ListUserSessionsUseCase` | ユーザーのアクティブセッション一覧 |

#### 外部連携

- **Redis** (`adapter/repository/session_redis.rs`): redis-rs クレートで Redis 7 に接続。セッションデータを TTL 付きで保存する
- **PostgreSQL** (`adapter/repository/session_metadata_postgres.rs`): デバイス情報・作成日時等のメタデータを記録する
- **Kafka Consumer** (`infrastructure/kafka_consumer.rs`): `k1s0.system.session.revoke_all.v1` トピックから全セッション失効リクエストを受信する
- **Kafka Producer** (`infrastructure/kafka_producer.rs`): `k1s0.system.session.created.v1` および `k1s0.system.session.revoked.v1` にイベントを配信する

### エラーハンドリング方針

- `error.rs` でアプリケーション固有のエラー型を定義し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_SESSION_`
- セッション期限切れは `SYS_SESSION_EXPIRED`（410 Gone）で返却する
- 既に失効済みセッションへの再失効は `SYS_SESSION_ALREADY_REVOKED`（409 Conflict）を返却する

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | Session エンティティ（is_valid/is_expired/revoke/refresh） | ドメインモデルの直接テスト |
| ユースケーステスト | セッション CRUD・TTL 延長・デバイス制限 | `usecase_test.rs` でモックリポジトリを使用 |
| 統合テスト | REST/gRPC ハンドラー | `integration_test.rs` で axum-test / tonic テストクライアント |
| Redis テスト | TTL 動作・一括失効 | テスト用 Redis でセッション有効期限の動作を検証 |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・Kafka メッセージング設計
- [Rust共通実装.md](../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
