# system-auth-server 実装設計

> **注記**: 本ドキュメントは auth-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../../_common/Rust共通実装.md) を参照。

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

#### Keycloak 認証方式（STATIC-MEDIUM-003 監査対応 / ADR-0061）

Keycloak Admin API への認証方式を **Client Credentials Grant のみ** に統一した（ROPC廃止）。

| 項目 | 変更前 | 変更後 |
|-----|--------|--------|
| 認証フロー | ROPC (admin_username + admin_password) または Client Credentials | Client Credentials Grant のみ |
| 設定フィールド | `admin_username`, `admin_password`, `admin_realm`, `admin_client_id` | 廃止（`client_id` + `client_secret` のみ） |
| Keycloak 側設定 | admin アカウントを直接使用 | `auth-rust-admin` Service Account を作成し `realm-management` ロールを付与 |

**理由**: ROPC は OAuth 2.1 草案で廃止予定。Client Credentials Grant は M2M 通信の標準フローであり、認証情報漏洩リスクが低い（詳細は `docs/architecture/adr/0061-ropc-to-client-credentials-migration.md` を参照）。

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

## /jwks エンドポイントと鍵ローテーション設計（M-06 監査対応）

### /jwks エンドポイントの実装

auth-rust は以下の 2 つのエンドポイントで JWKS（JSON Web Key Set）を公開する。

| エンドポイント | 説明 |
|--------------|------|
| `GET /jwks` | JWKS 取得（短縮パス） |
| `GET /.well-known/jwks.json` | RFC 準拠の標準パス |

実装ファイル: `adapter/handler/jwks_handler.rs`

### JwksProvider の動作（鍵キャッシュ）

`infrastructure/jwks_provider.rs` が JWKS を Keycloak から取得してキャッシュする。

```
Keycloak JWKS エンドポイント
  └─ GET {KEYCLOAK_URL}/realms/{realm}/protocol/openid-connect/certs
       ↓
JwksProvider（TTL: config.keycloak.jwks_cache_ttl_secs、デフォルト 300 秒）
  ├─ キャッシュ有効 → RwLock（読み取り）でキャッシュ値を返す（排他ロック不要）
  └─ キャッシュ失効 → fetch_lock（Mutex）で排他制御し、1 リクエストのみ Keycloak にフェッチ
       ↓（ダブルチェック: 他のリクエストが先に更新済みなら待機なし）
  Keycloak から最新 JWKS を取得して RwLock（書き込み）でキャッシュを更新
```

サンダリングハード（多数の同時リクエストが一斉に Keycloak を叩く）を防ぐため、
`fetch_lock: Mutex<()>` で排他制御している（`jwks_provider.rs` 参照）。

### 鍵ローテーション対応

Keycloak が公開鍵をローテーションした場合の動作:

| ケース | 動作 |
|--------|------|
| キャッシュ TTL 内 | 旧鍵がキャッシュされているため、一時的に古い JWKS が返される |
| キャッシュ TTL 超過（デフォルト 300 秒） | 次のリクエスト時に Keycloak から最新 JWKS を取得してキャッシュを更新 |
| 強制ローテーション | auth-rust の再起動またはキャッシュ TTL を短縮することで対応 |

**鍵ローテーション時の影響**:
- キャッシュ TTL の間（最大 300 秒）は旧鍵での署名検証が成功し続ける（後方互換）
- 新鍵での署名済みトークンは TTL 経過後から正常に検証される
- Keycloak のデフォルトでは古い鍵も一定期間公開鍵セットに残るため、
  移行期間中のトークン検証は正常に機能する

### 設定

```yaml
# config.yaml（auth サーバー設定）
keycloak:
  url: "http://keycloak:8080"
  realm: "k1s0"
  jwks_cache_ttl_secs: 300  # デフォルト 5 分。緊急ローテーション時は 60 に短縮する
```

### 緊急ローテーション手順

Keycloak 管理コンソールで公開鍵を強制ローテーションした場合:

1. `config.yaml` の `jwks_cache_ttl_secs` を 60（秒）に変更
2. auth-rust を再デプロイ（またはローリングアップデート）
3. 1 分後に BFF-Proxy の `/auth/session` で新しい鍵での署名検証が成功することを確認
4. 安定後に `jwks_cache_ttl_secs` を 300 に戻す

参考: 外部技術監査報告書 M-06 "/jwks 鍵ローテーション設計の記録を求める"

---

## Keycloak フォールバック挙動と障害対応

### フォールバック挙動の概要

auth-rust の起動シーケンス（`infrastructure/startup.rs`）において、Keycloak との連携に以下の 2 パターンで失敗した場合、静的 RBAC フォールバックへ移行する。

| ケース | 発生条件 | フォールバック動作 |
|--------|----------|------------------|
| Keycloak 初期同期失敗 | `table.sync_once()` が Err を返した場合（401/503 等） | 静的 RBAC テーブルを使用してサービスを継続起動する |
| Keycloak 未設定 | `config.yaml` に `keycloak` セクションがない場合 | 静的 RBAC テーブルを使用してサービスを継続起動する |

**フォールバック時のリスク**: Keycloak 側で更新されたロール・パーミッションが反映されず、旧い静的テーブルの権限で動作し続ける。本番環境では速やかに Keycloak 接続を復旧させること。

### 構造化ログフィールドと監視設定

フォールバック発生時、以下の構造化フィールドを含む `WARN` ログが出力される。

**Keycloak 初期同期失敗時:**

```json
{
  "level": "WARN",
  "alert": true,
  "keycloak_fallback": true,
  "error": "<エラー詳細>",
  "message": "initial keycloak role-permission sync failed; static RBAC fallback will be used"
}
```

**Keycloak 未設定時:**

```json
{
  "level": "WARN",
  "alert": true,
  "keycloak_not_configured": true,
  "message": "keycloak is not configured; static RBAC fallback will be used"
}
```

### Grafana / Loki でのアラート設定例

`alert=true` フィールドを使ってログベースのアラートを設定できる。

**Loki クエリ例（フォールバック発生を検知）:**

```logql
{service="k1s0-auth-server"} | json | alert = "true"
```

**Keycloak フォールバック限定の検知:**

```logql
{service="k1s0-auth-server"} | json | keycloak_fallback = "true"
```

**推奨アラートルール（Grafana Alerting）:**

| アラート名 | クエリ条件 | 重要度 | 通知先 |
|-----------|-----------|--------|--------|
| `AuthKeycloakFallback` | `keycloak_fallback = "true"` が 1 件以上 | Warning | 担当チーム Slack |
| `AuthKeycloakNotConfigured` | `keycloak_not_configured = "true"` が 1 件以上（本番環境のみ） | Critical | PagerDuty |

参考: 外部技術監査 MEDIUM-003 "Keycloak フォールバック時のアラートログ強化"

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義
- [Rust共通実装.md](../../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
- [認証設計.md](../../architecture/auth/認証設計.md) -- 認証アーキテクチャ
