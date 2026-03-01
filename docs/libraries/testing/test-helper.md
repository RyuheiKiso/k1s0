# k1s0-test-helper ライブラリ設計

## 概要

テスト基盤ユーティリティライブラリ。Testcontainers 統合（PostgreSQL・Redis・Kafka・Keycloak のコンテナ起動ヘルパー）、system tier 各サーバーの HTTP モックサーバー、テスト用エンティティのフィクスチャビルダー、テスト用署名済み JWT 生成、JSON 部分一致・イベントアサーション等のアサーションヘルパーを提供する。

各 system tier サーバー（notification・ratelimit・tenant・scheduler 等）の実際の依存を排除し、単体テスト・統合テストを高速かつ安定して実行できる環境を構築する。

**配置先**:

| 言語 | パス |
|------|------|
| Go | `regions/system/library/go/test-helper/` |
| Rust | `regions/system/library/rust/test-helper/` |
| TypeScript | `regions/system/library/typescript/test-helper/` |
| Dart | `regions/system/library/dart/test_helper/` |

## 公開 API

### 言語別対応状況

| 機能 | Rust | Go | TypeScript | Dart |
|------|------|-----|-----------|------|
| TestContainerBuilder（DB/Kafka/Keycloak コンテナ） | ✓ (feature: containers) | ✓ | — | — |
| JwtTestHelper | ✓ (feature: jwt) | ✓ | ✓ | ✓ |
| MockServerBuilder | ✓ (feature: mock-server) | ✓ | ✓ | ✓ |
| FixtureBuilder | ✓ (feature: fixtures) | ✓ | ✓ | ✓ |
| AssertionHelper | ✓ | ✓ | ✓ | ✓ |
| MockServer | ✓ (feature: mock-server) | ✓ | ✓ | ✓ |

> **注記**: Testcontainers 統合（TestContainerBuilder）は Rust と Go のみ提供。TypeScript/Dart では Docker コンテナの直接制御は行わず、モックサーバーと JWT ヘルパーに特化した構成。

### 全言語共通 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `JwtTestHelper` | クラス/構造体 | テスト用 JWT トークン生成（HS256） |
| `TestClaims` | 構造体/interface | テスト用 JWT クレーム（sub・roles・tenant_id 等） |
| `MockServerBuilder` | 構造体/クラス | system tier 各サーバーの HTTP モック構築 |
| `MockServer` | 構造体/クラス | モックサーバー（レスポンス設定・検証） |
| `FixtureBuilder` | トレイト/クラス | テスト用エンティティ生成・ランダム値注入 |
| `AssertionHelper` | 構造体/クラス | JSON 部分一致・イベントアサーション等 |

### Rust / Go 専用 API（コンテナ統合）

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `TestContainerBuilder` | 構造体 | PostgreSQL・Redis・Kafka・Keycloak のコンテナ起動ヘルパー |
| `PostgresContainer` | 構造体 | PostgreSQL コンテナ（接続 URL 提供） |
| `RedisContainer` | 構造体 | Redis コンテナ（接続 URL 提供） |
| `KafkaContainer` | 構造体 | Kafka コンテナ（ブローカー URL 提供） |
| `KeycloakContainer` | 構造体 | Keycloak コンテナ（認証 URL・レルム設定） |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-test-helper"
version = "0.1.0"
edition = "2021"

[features]
containers = ["testcontainers", "testcontainers-modules"]
jwt = ["jsonwebtoken"]
mock-server = ["wiremock"]
fixtures = ["fake"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
testcontainers = { version = "0.23", optional = true }
testcontainers-modules = { version = "0.11", features = ["postgres", "redis", "kafka"], optional = true }
jsonwebtoken = { version = "9", optional = true }
wiremock = { version = "0.6", optional = true }
fake = { version = "2.10", features = ["derive", "chrono"], optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `[dev-dependencies]`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
test-helper/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── containers.rs   # TestContainerBuilder・各コンテナ型
│   ├── jwt.rs          # JwtTestHelper・TestClaims
│   ├── mock_server.rs  # MockServerBuilder・MockServer・各サーバーモック
│   ├── fixture.rs      # FixtureBuilder トレイト・ランダム値生成
│   └── assertion.rs    # AssertionHelper（JSON 部分一致等）
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_test_helper::{
    TestContainerBuilder, JwtTestHelper, MockServerBuilder,
    AssertionHelper,
};

// PostgreSQL コンテナ起動
let pg = TestContainerBuilder::postgres()
    .with_db("test_db")
    .with_user("test_user")
    .start()
    .await;
let db_url = pg.connection_url();

// Redis コンテナ起動
let redis = TestContainerBuilder::redis().start().await;
let redis_url = redis.connection_url();

// Keycloak コンテナ起動
let kc = TestContainerBuilder::keycloak()
    .with_realm("k1s0-test")
    .with_admin("admin", "admin123")
    .start()
    .await;
let auth_url = kc.auth_url();

// JWT テストトークン生成
let jwt_helper = JwtTestHelper::new_hs256("test-secret");

let admin_token = jwt_helper.create_admin_token();
let user_token = jwt_helper.create_user_token("user-123", vec!["user".to_string()]);
let custom_token = jwt_helper.create_token(TestClaims {
    sub: "service-account".to_string(),
    roles: vec!["service".to_string()],
    tenant_id: Some("tenant-456".to_string()),
    ..Default::default()
});

// system tier サーバーのモック
let mock = MockServerBuilder::notification_server()
    .with_success_response()
    .start()
    .await;

let ratelimit_mock = MockServerBuilder::ratelimit_server()
    .with_allow_response(remaining: 99)
    .start()
    .await;

// JSON 部分一致アサーション
let response_body = r#"{"id":"123","status":"ok","extra":"ignored"}"#;
AssertionHelper::assert_json_contains(response_body, r#"{"id":"123","status":"ok"}"#);
```

## Go 実装

**配置先**: `regions/system/library/go/test-helper/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/testcontainers/testcontainers-go v0.35.0`, `github.com/golang-jwt/jwt/v5 v5.2.0`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
// コンテナビルダー
type ContainerBuilder struct{}

func NewContainerBuilder() *ContainerBuilder

func (b *ContainerBuilder) Postgres(ctx context.Context, opts ...PostgresOption) (*PostgresContainer, error)
func (b *ContainerBuilder) Redis(ctx context.Context, opts ...RedisOption) (*RedisContainer, error)
func (b *ContainerBuilder) Kafka(ctx context.Context, opts ...KafkaOption) (*KafkaContainer, error)
func (b *ContainerBuilder) Keycloak(ctx context.Context, opts ...KeycloakOption) (*KeycloakContainer, error)

// JWT ヘルパー
type JwtTestHelper struct{}

func NewJwtTestHelper(secret string) *JwtTestHelper
func (h *JwtTestHelper) CreateAdminToken() string
func (h *JwtTestHelper) CreateUserToken(userID string, roles []string) string
func (h *JwtTestHelper) CreateToken(claims TestClaims) string

type TestClaims struct {
    Sub      string
    Roles    []string
    TenantID string
    Expiry   time.Duration
}

// モックサーバービルダー
type MockServerBuilder struct{}

func NewMockServerBuilder() *MockServerBuilder
func (b *MockServerBuilder) NotificationServer() *MockServer
func (b *MockServerBuilder) RatelimitServer() *MockServer
func (b *MockServerBuilder) TenantServer() *MockServer
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/test-helper/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
// コンテナビルダー
export class TestContainerBuilder {
  static postgres(opts?: PostgresOptions): PostgresContainerHelper;
  static redis(opts?: RedisOptions): RedisContainerHelper;
  static kafka(opts?: KafkaOptions): KafkaContainerHelper;
  static keycloak(opts?: KeycloakOptions): KeycloakContainerHelper;
}

export interface PostgresContainerHelper {
  start(): Promise<{ connectionUrl: string; stop(): Promise<void> }>;
}

// JWT ヘルパー
export interface TestClaims {
  sub: string;
  roles?: string[];
  tenantId?: string;
  expiresInMs?: number;
  extra?: Record<string, unknown>;
}

export class JwtTestHelper {
  constructor(secret: string, algorithm?: 'HS256' | 'RS256');
  createToken(claims: TestClaims): string;
  createAdminToken(): string;
  createUserToken(userId: string, roles: string[]): string;
}

// モックサーバービルダー
export class MockServerBuilder {
  static notificationServer(): MockServerHelper;
  static ratelimitServer(): MockServerHelper;
  static tenantServer(): MockServerHelper;
}

export interface MockServerHelper {
  withSuccessResponse(): this;
  withErrorResponse(status: number, body?: object): this;
  start(): Promise<{ baseUrl: string; verify(): void; stop(): Promise<void> }>;
}

// フィクスチャビルダー
export class FixtureBuilder {
  static uuid(): string;
  static email(): string;
  static name(): string;
  static int(min?: number, max?: number): number;
}

// アサーションヘルパー
export class AssertionHelper {
  static assertJsonContains(actual: unknown, expected: unknown): void;
  static assertEventEmitted(events: unknown[], eventType: string): void;
}
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/test_helper/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**pubspec.yaml 主要依存**:

```yaml
dev_dependencies:
  test: ^1.25.0
  mocktail: ^1.0.4
  dart_jsonwebtoken: ^2.14.0
  http: ^1.2.0
```

**主要インターフェース**:

```dart
class TestContainerBuilder {
  static Future<PostgresContainer> postgres({String? db, String? user}) async;
  static Future<RedisContainer> redis() async;
  static Future<KeycloakContainer> keycloak({String realm = 'k1s0-test'}) async;
}

class JwtTestHelper {
  JwtTestHelper({required String secret, String algorithm = 'HS256'});
  String createToken(TestClaims claims);
  String createAdminToken();
  String createUserToken(String userId, List<String> roles);
}

class MockServerBuilder {
  static MockServer notificationServer();
  static MockServer ratelimitServer();
  static MockServer tenantServer();
}

class AssertionHelper {
  static void assertJsonContains(dynamic actual, dynamic expected);
  static void assertEventEmitted(List<dynamic> events, String eventType);
}
```

**カバレッジ目標**: 85%以上

## テスト戦略

| テスト種別 | 対象 | ツール |
|-----------|------|--------|
| ユニットテスト（`#[cfg(test)]`） | JWT 生成・クレーム検証・フィクスチャ値の範囲確認・JSON 部分一致ロジック | tokio::test |
| コンテナテスト | PostgreSQL・Redis・Kafka・Keycloak の正常起動・接続 URL 取得 | testcontainers（CI 環境で有効化） |
| モックサーバーテスト | 各 system tier サーバーモックへのリクエスト・レスポンス検証 | wiremock |
| JWT 検証テスト | 生成済みトークンのクレーム内容・有効期限・署名検証 | jsonwebtoken |

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-migration設計](../data/migration.md) — テスト用 DB マイグレーション
- [system-library-serviceauth設計](../auth-security/serviceauth.md) — JWT 認証テスト
