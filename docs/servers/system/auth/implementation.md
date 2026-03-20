# system-auth-server 実装設計

> **注記**: 本ドキュメントは auth-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../_common/Rust共通実装.md) を参照。

system-auth-server（認証サーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（トークン検証・ユーザー管理・権限チェック・監査ログ） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・JWKS・Keycloak・Kafka・キャッシュ・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/auth/)

### ディレクトリ構成

```
regions/system/server/rust/auth/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── user.rs                      # User エンティティ
│   │   │   ├── role.rs                      # Role エンティティ
│   │   │   ├── permission.rs                # Permission エンティティ
│   │   │   ├── claims.rs                    # TokenClaims エンティティ
│   │   │   ├── audit_log.rs                 # AuditLog エンティティ
│   │   │   └── api_key.rs                   # ApiKey エンティティ
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── user_repository.rs           # UserRepository トレイト
│   │   │   ├── audit_log_repository.rs      # AuditLogRepository トレイト
│   │   │   └── api_key_repository.rs        # ApiKeyRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       ├── auth_domain_service.rs       # パーミッション解決ロジック
│   │       └── role_permission_table.rs     # ロール→パーミッション変換テーブル
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── validate_token.rs                # JWT トークン検証
│   │   ├── validate_api_key.rs              # API キー検証
│   │   ├── get_user.rs                      # ユーザー情報取得
│   │   ├── list_users.rs                    # ユーザー一覧取得
│   │   ├── get_user_roles.rs                # ユーザーロール取得
│   │   ├── check_permission.rs              # 権限チェック
│   │   ├── record_audit_log.rs              # 監査ログ記録
│   │   ├── search_audit_logs.rs             # 監査ログ検索
│   │   ├── create_api_key.rs                # API キー作成
│   │   ├── get_api_key.rs                   # API キー取得
│   │   ├── list_api_keys.rs                 # API キー一覧取得
│   │   └── revoke_api_key.rs                # API キー無効化
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── auth_handler.rs              # 認証系 REST ハンドラー
│   │   │   ├── audit_handler.rs             # 監査ログ REST ハンドラー
│   │   │   ├── api_key_handler.rs           # API キー REST ハンドラー
│   │   │   └── jwks_handler.rs              # JWKS 公開鍵 REST ハンドラー
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── auth_grpc.rs                 # Auth gRPC サービス実装
│   │   │   ├── audit_grpc.rs                # Audit gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                      # JWT 認証ミドルウェア
│   │   │   └── rbac.rs                      # RBAC ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── user_postgres.rs             # UserRepository PostgreSQL 実装
│   │       ├── cached_user_repository.rs    # キャッシュ付き UserRepository
│   │       ├── audit_log_postgres.rs        # AuditLogRepository PostgreSQL 実装
│   │       └── api_key_postgres.rs          # ApiKeyRepository PostgreSQL 実装
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                        # 設定構造体・読み込み（AuthConfig, AuthServerConfig）
│   │   ├── database.rs                      # DB 接続プール
│   │   ├── jwks_provider.rs                 # JWKS プロバイダー
│   │   ├── jwks_adapter.rs                  # JWKS 検証アダプター
│   │   ├── keycloak_client.rs               # Keycloak Admin API クライアント
│   │   ├── keycloak_role_permission_source.rs # Keycloak ロール→パーミッションソース
│   │   ├── kafka_producer.rs                # Kafka プロデューサー
│   │   ├── permission_cache.rs              # パーミッションキャッシュ
│   │   ├── user_cache.rs                    # ユーザーキャッシュ
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

- **AuthDomainService**: ロール→パーミッション変換テーブルに基づくパーミッション解決ロジック
- **RolePermissionTable**: ロールとパーミッションのマッピングテーブル

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `ValidateTokenUseCase` | JWKS による JWT トークン検証。issuer・audience の追加検証を含む |
| `ValidateApiKeyUseCase` | API キーの検証 |
| `GetUserUseCase` / `ListUsersUseCase` | Keycloak 連携によるユーザー情報取得 |
| `CheckPermissionUseCase` | ロールベースの権限チェック判定 |
| `RecordAuditLogUseCase` | 監査ログの記録（DB + Kafka 非同期配信） |
| `SearchAuditLogsUseCase` | 監査ログの検索（ページネーション対応） |

#### 外部連携

- **Keycloak Client** (`infrastructure/keycloak_client.rs`): Keycloak Admin API からユーザー情報・ロールを取得する。Admin API トークンをキャッシュ付きで管理する
- **JWKS Provider** (`infrastructure/jwks_provider.rs`): JWKS エンドポイントから公開鍵を取得・キャッシュする

### エラーハンドリング方針

- ユースケース層で `AuthError` 型を返却し、adapter 層で `ErrorResponse` に変換する
- エラーコードプレフィックス: `SYS_AUTH_`
- 認証失敗は 401、権限不足は 403 を返す

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | トークン検証・権限チェック | mockall によるリポジトリモック |
| JWKS テスト | JWKS キャッシュ・TTL | wiremock によるモックエンドポイント |
| 統合テスト | REST/gRPC ハンドラー | axum-test / tonic テストクライアント |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義
- [Rust共通実装.md](../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
- [認証設計.md](../../architecture/auth/認証設計.md) -- 認証アーキテクチャ
