# k1s0-tenant-client ライブラリ設計

## 概要

system-tenant-server（ポート 8089）へのテナント情報取得クライアントライブラリ。テナント情報の取得・TTL 付きキャッシュ・テナントコンテキストの伝播（リクエストヘッダー `X-Tenant-ID` 管理）・テナント存在確認とアクティブ状態チェック・テナント設定値の取得を統一インターフェースで提供する。全 Tier のサービスからマルチテナント制御を共通利用するための基盤ライブラリである。

> **ポート注記**: ポート `8089` は Docker Compose 環境でのホスト側ポートである。本番環境では Kubernetes Service 経由（`tenant-server:8080`）で接続する。

**配置先**: `regions/system/library/rust/tenant-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `TenantClient` | トレイト | テナント操作インターフェース |
| `HttpTenantClient` | 構造体 | tenant-server HTTP 接続実装（TTL 付きキャッシュ内蔵）|
| `InMemoryTenantClient` | 構造体 | テスト用インメモリ実装 |
| `Tenant` | 構造体 | テナント情報（ID・名称・ステータス・プラン・設定・作成日時）|
| `TenantStatus` | enum | `Active`・`Suspended`・`Deleted` |
| `TenantFilter` | 構造体 | テナント一覧取得フィルター（ステータス・プラン）|
| `TenantSettings` | 構造体 | テナント固有設定値（`values: HashMap<String, String>` フィールドを持つ構造体）|
| `TenantClientConfig` | 構造体 | サーバー URL・キャッシュ TTL・最大キャッシュサイズ |
| `TenantError` | enum | `NotFound`・`Suspended`・`ServerError`・`Timeout` |
| `CreateTenantRequest` | 構造体 | テナント作成リクエスト（名称・プラン・管理者ユーザー ID）|
| `TenantMember` | 構造体 | テナントメンバー（ユーザー ID・ロール・参加日時）|
| `ProvisioningStatus` | enum | `Pending`・`InProgress`・`Completed`・`Failed(String)` |
| `HttpTenantClient::close` | メソッド | HTTP クライアントのリソース解放（TypeScript・Dart のみ実装。Go・Rust は GC/Drop で自動解放のため不要）|
| `InMemoryTenantClient::new` | コンストラクタ | テスト用インメモリ実装の生成（全4言語で実装済み）|
| `InMemoryTenantClient::with_tenants` | ファクトリ | 初期テナント一覧を指定して生成（全4言語で実装済み）|
| `InMemoryTenantClient::add_tenant` | メソッド | テナントをインメモリストアへ追加（全4言語で実装済み）|

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-tenant-client"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
reqwest = { version = "0.12", features = ["json"] }
moka = { version = "0.12", features = ["future"] }
tokio = { version = "1", features = ["rt", "time"] }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `k1s0-tenant-client = { path = "../../system/library/rust/tenant-client" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
tenant-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── client.rs       # TenantClient トレイト・InMemoryTenantClient・HttpTenantClient（TTL キャッシュ内蔵）
│   ├── tenant.rs       # Tenant・TenantStatus・TenantSettings・TenantFilter
│   ├── config.rs       # TenantClientConfig
│   └── error.rs        # TenantError
└── Cargo.toml
```

**主要 API**:

```rust
// トレイト定義
#[async_trait]
pub trait TenantClient: Send + Sync {
    async fn get_tenant(&self, tenant_id: &str) -> Result<Tenant, TenantError>;
    async fn list_tenants(&self, filter: TenantFilter) -> Result<Vec<Tenant>, TenantError>;
    async fn is_active(&self, tenant_id: &str) -> Result<bool, TenantError>;
    async fn get_settings(&self, tenant_id: &str) -> Result<TenantSettings, TenantError>;
    async fn create_tenant(&self, req: CreateTenantRequest) -> Result<Tenant, TenantError>;
    async fn add_member(&self, tenant_id: &str, user_id: &str, role: &str) -> Result<TenantMember, TenantError>;
    async fn remove_member(&self, tenant_id: &str, user_id: &str) -> Result<(), TenantError>;
    async fn list_members(&self, tenant_id: &str) -> Result<Vec<TenantMember>, TenantError>;
    async fn get_provisioning_status(&self, tenant_id: &str) -> Result<ProvisioningStatus, TenantError>;
}

// 型定義
pub enum TenantStatus { Active, Suspended, Deleted }

pub struct Tenant {
    pub id: String,
    pub name: String,
    pub status: TenantStatus,
    pub plan: String,
    pub settings: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

pub struct TenantFilter {
    pub status: Option<TenantStatus>,
    pub plan: Option<String>,
}

impl TenantFilter {
    pub fn new() -> Self
    pub fn status(mut self, status: TenantStatus) -> Self
    pub fn plan(mut self, plan: impl Into<String>) -> Self
}

pub struct TenantSettings {
    pub values: HashMap<String, String>,
}

impl TenantSettings {
    pub fn new(values: HashMap<String, String>) -> Self
    pub fn get(&self, key: &str) -> Option<&str>
}

pub struct CreateTenantRequest {
    pub name: String,
    pub plan: String,
    pub admin_user_id: Option<String>,
}

pub struct TenantMember {
    pub user_id: String,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

pub enum ProvisioningStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

pub struct TenantClientConfig {
    pub server_url: String,
    pub cache_ttl: Duration,
    pub cache_max_capacity: u64,
}

impl TenantClientConfig {
    pub fn new(server_url: impl Into<String>) -> Self
    pub fn cache_ttl(self, ttl: Duration) -> Self
    pub fn cache_max_capacity(self, capacity: u64) -> Self
}

// HTTP 接続実装
pub struct HttpTenantClient { /* ... */ }

impl HttpTenantClient {
    pub fn new(config: TenantClientConfig) -> Result<Self, TenantError>
    // TypeScript / Dart: close() でリソースを明示解放する
    // Go / Rust: GC / Drop による自動解放のため close 相当メソッドは不要
}

// TenantClient トレイトの全メソッドを実装

// テスト用インメモリ実装
pub struct InMemoryTenantClient { /* ... */ }

impl InMemoryTenantClient {
    pub fn new() -> Self
    pub fn with_tenants(tenants: Vec<Tenant>) -> Self
    pub fn add_tenant(&self, tenant: Tenant)
}

// TenantClient トレイトの全メソッドを実装
```

**使用例**:

```rust
use k1s0_tenant_client::{
    HttpTenantClient, TenantClient, TenantClientConfig, TenantFilter, TenantStatus,
};
use std::time::Duration;

// クライアントの構築（TTL 5 分・最大 1000 件キャッシュ）
let config = TenantClientConfig::new("http://tenant-server:8080")
    .cache_ttl(Duration::from_secs(300))
    .cache_max_capacity(1000);

let client = HttpTenantClient::new(config).await?;

// テナント情報の取得（キャッシュヒット時はサーバーへの呼び出しをスキップ）
let tenant = client.get_tenant("TENANT-001").await?;
tracing::info!(
    tenant_id = %tenant.id,
    plan = %tenant.plan,
    status = ?tenant.status,
    "テナント情報取得"
);

// アクティブ状態チェック（ゲートウェイでのリクエスト受付可否判定）
if !client.is_active("TENANT-001").await? {
    return Err("Tenant is not active".into());
}

// テナント設定値の取得
let settings = client.get_settings("TENANT-001").await?;
let max_users = settings.get("max_users").unwrap_or("100");

// テナント一覧の取得（アクティブのみ、特定プランでフィルタ）
let filter = TenantFilter::new().status(TenantStatus::Active).plan("enterprise");
let tenants = client.list_tenants(filter).await?;
tracing::info!(count = tenants.len(), "アクティブテナント一覧取得");
```

## Go 実装

**配置先**: `regions/system/library/go/tenant-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/stretchr/testify v1.11.1`

**主要インターフェース**:

```go
type TenantClient interface {
    GetTenant(ctx context.Context, tenantID string) (Tenant, error)
    ListTenants(ctx context.Context, filter TenantFilter) ([]Tenant, error)
    IsActive(ctx context.Context, tenantID string) (bool, error)
    GetSettings(ctx context.Context, tenantID string) (TenantSettings, error)
    CreateTenant(ctx context.Context, req CreateTenantRequest) (Tenant, error)
    AddMember(ctx context.Context, tenantID, userID, role string) (TenantMember, error)
    RemoveMember(ctx context.Context, tenantID, userID string) error
    ListMembers(ctx context.Context, tenantID string) ([]TenantMember, error)
    GetProvisioningStatus(ctx context.Context, tenantID string) (ProvisioningStatus, error)
}

type TenantStatus string

const (
    TenantStatusActive    TenantStatus = "active"
    TenantStatusSuspended TenantStatus = "suspended"
    TenantStatusDeleted   TenantStatus = "deleted"
)

// NOTE: TenantStatus の言語別表現
//   Rust:       PascalCase enum  — Active / Suspended / Deleted
//   Go:         小文字文字列      — "active" / "suspended" / "deleted"
//   TypeScript: 小文字文字列      — 'active' | 'suspended' | 'deleted'
//   Dart:       lowerCamelCase enum — TenantStatus.active / .suspended / .deleted

type Tenant struct {
    ID        string            `json:"id"`
    Name      string            `json:"name"`
    Status    TenantStatus      `json:"status"`
    Plan      string            `json:"plan"`
    Settings  map[string]string `json:"settings"`
    CreatedAt time.Time         `json:"created_at"`
}

// NOTE: Go には TenantFilter / TenantSettings / TenantClientConfig のビルダーメソッドは存在しない。
// struct リテラルで直接初期化する（例: TenantFilter{Status: &status}）。
// Rust はビルダーメソッド（TenantFilter::new().status(...)）を提供する。
// TypeScript / Dart はオブジェクト / named parameters で初期化する。
type TenantFilter struct {
    Status *TenantStatus
    Plan   *string
}

type TenantSettings struct {
    Values map[string]string
}

// NOTE: Go の Get は Go 慣用の (value string, ok bool) 2値返却。
// Rust は Option<&str>、TypeScript は string | undefined、Dart は String? を返す。
func (s TenantSettings) Get(key string) (string, bool)

type CreateTenantRequest struct {
    Name        string `json:"name"`
    Plan        string `json:"plan"`
    AdminUserID string `json:"admin_user_id,omitempty"` // NOTE: Go は非ポインタ文字列（空文字でオプショナル扱い）。Rust は Option<String>、TypeScript は string?、Dart は String?
}

type TenantMember struct {
    UserID   string    `json:"user_id"`
    Role     string    `json:"role"`
    JoinedAt time.Time `json:"joined_at"`
}

type ProvisioningStatus string

const (
    ProvisioningStatusPending    ProvisioningStatus = "pending"
    ProvisioningStatusInProgress ProvisioningStatus = "in_progress"
    ProvisioningStatusCompleted  ProvisioningStatus = "completed"
    ProvisioningStatusFailed     ProvisioningStatus = "failed"
)

// NOTE: ProvisioningStatus.Failed の失敗理由文字列は Rust のみサポート（`Failed(String)`）。
// Go / TypeScript / Dart では failed 時の理由は別途エラーメッセージ等で取得する。

type TenantClientConfig struct {
    ServerURL        string
    CacheTTL         time.Duration
    CacheMaxCapacity int // NOTE: Go は int（符号付き）。Rust / Doc は u64（符号なし64bit）、Dart は int（符号付き）
}

type HttpTenantClient struct{ /* ... */ }

// NOTE: Go の NewHttpTenantClient は addr（URL 文字列）と config の2引数を受け取る。
// Rust / TypeScript / Dart は config オブジェクト1つのみ（serverUrl は config 内に含まれる）。
// さらに Go には httpClient をカスタマイズできる NewHttpTenantClientWithHTTPClient もある。
func NewHttpTenantClient(addr string, config TenantClientConfig) (*HttpTenantClient, error)
func NewHttpTenantClientWithHTTPClient(addr string, config TenantClientConfig, httpClient *http.Client) (*HttpTenantClient, error)
func (c *HttpTenantClient) GetTenant(ctx context.Context, tenantID string) (Tenant, error)
func (c *HttpTenantClient) ListTenants(ctx context.Context, filter TenantFilter) ([]Tenant, error)
func (c *HttpTenantClient) IsActive(ctx context.Context, tenantID string) (bool, error)
func (c *HttpTenantClient) GetSettings(ctx context.Context, tenantID string) (TenantSettings, error)
func (c *HttpTenantClient) CreateTenant(ctx context.Context, req CreateTenantRequest) (Tenant, error)
func (c *HttpTenantClient) AddMember(ctx context.Context, tenantID, userID, role string) (TenantMember, error)
func (c *HttpTenantClient) RemoveMember(ctx context.Context, tenantID, userID string) error
func (c *HttpTenantClient) ListMembers(ctx context.Context, tenantID string) ([]TenantMember, error)
func (c *HttpTenantClient) GetProvisioningStatus(ctx context.Context, tenantID string) (ProvisioningStatus, error)

type InMemoryTenantClient struct{ /* ... */ }

func NewInMemoryTenantClient() *InMemoryTenantClient
func NewInMemoryTenantClientWithTenants(tenants []Tenant) *InMemoryTenantClient
func (c *InMemoryTenantClient) AddTenant(t Tenant)
func (c *InMemoryTenantClient) GetTenant(ctx context.Context, tenantID string) (Tenant, error)
func (c *InMemoryTenantClient) ListTenants(ctx context.Context, filter TenantFilter) ([]Tenant, error)
func (c *InMemoryTenantClient) IsActive(ctx context.Context, tenantID string) (bool, error)
func (c *InMemoryTenantClient) GetSettings(ctx context.Context, tenantID string) (TenantSettings, error)
func (c *InMemoryTenantClient) CreateTenant(ctx context.Context, req CreateTenantRequest) (Tenant, error)
func (c *InMemoryTenantClient) AddMember(ctx context.Context, tenantID, userID, role string) (TenantMember, error)
func (c *InMemoryTenantClient) RemoveMember(ctx context.Context, tenantID, userID string) error
func (c *InMemoryTenantClient) ListMembers(ctx context.Context, tenantID string) ([]TenantMember, error)
func (c *InMemoryTenantClient) GetProvisioningStatus(ctx context.Context, tenantID string) (ProvisioningStatus, error)
```

**使用例**:

```go
config := TenantClientConfig{
    ServerURL: "tenant-server:8080",
    CacheTTL:  5 * time.Minute,
}
client, err := NewHttpTenantClient("tenant-server:8080", config)
if err != nil {
    log.Fatal(err)
}

// X-Tenant-ID ヘッダーからテナント ID を取得して検証
tenantID := r.Header.Get("X-Tenant-ID")
active, err := client.IsActive(ctx, tenantID)
if err != nil || !active {
    http.Error(w, "Tenant not found or inactive", http.StatusForbidden)
    return
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/tenant-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export type TenantStatus = 'active' | 'suspended' | 'deleted';

export interface Tenant {
  id: string;
  name: string;
  status: TenantStatus;
  plan: string;
  settings: Record<string, string>;
  createdAt: Date;
}

export interface TenantFilter {
  status?: TenantStatus;
  plan?: string;
}

export interface TenantSettings {
  values: Record<string, string>;
  get(key: string): string | undefined;
}

export interface TenantClientConfig {
  serverUrl: string;
  cacheTtlMs?: number;  // NOTE: TypeScript のみ Duration ではなくミリ秒を表す number 型（フィールド名に Ms サフィックスあり）。Go / Rust / Dart は Duration 型
  cacheMaxCapacity?: number;
}

export interface CreateTenantRequest {
  name: string;
  plan: string;
  adminUserId?: string;
}

export interface TenantMember {
  userId: string;
  role: string;
  joinedAt: Date;
}

export type ProvisioningStatus = 'pending' | 'in_progress' | 'completed' | 'failed';

export interface TenantClient {
  getTenant(tenantId: string): Promise<Tenant>;
  listTenants(filter?: TenantFilter): Promise<Tenant[]>;
  isActive(tenantId: string): Promise<boolean>;
  getSettings(tenantId: string): Promise<TenantSettings>;
  createTenant(req: CreateTenantRequest): Promise<Tenant>;
  addMember(tenantId: string, userId: string, role: string): Promise<TenantMember>;
  removeMember(tenantId: string, userId: string): Promise<void>;
  listMembers(tenantId: string): Promise<TenantMember[]>;
  getProvisioningStatus(tenantId: string): Promise<ProvisioningStatus>;
}

export class HttpTenantClient implements TenantClient {
  constructor(config: TenantClientConfig);
  getTenant(tenantId: string): Promise<Tenant>;
  listTenants(filter?: TenantFilter): Promise<Tenant[]>;
  isActive(tenantId: string): Promise<boolean>;
  getSettings(tenantId: string): Promise<TenantSettings>;
  createTenant(req: CreateTenantRequest): Promise<Tenant>;
  addMember(tenantId: string, userId: string, role: string): Promise<TenantMember>;
  removeMember(tenantId: string, userId: string): Promise<void>;
  listMembers(tenantId: string): Promise<TenantMember[]>;
  getProvisioningStatus(tenantId: string): Promise<ProvisioningStatus>;
  close(): Promise<void>;
}

export class InMemoryTenantClient implements TenantClient {
  constructor(tenants?: Tenant[]);
  addTenant(tenant: Tenant): void;
  getTenant(tenantId: string): Promise<Tenant>;
  listTenants(filter?: TenantFilter): Promise<Tenant[]>;
  isActive(tenantId: string): Promise<boolean>;
  getSettings(tenantId: string): Promise<TenantSettings>;
  createTenant(req: CreateTenantRequest): Promise<Tenant>;
  addMember(tenantId: string, userId: string, role: string): Promise<TenantMember>;
  removeMember(tenantId: string, userId: string): Promise<void>;
  listMembers(tenantId: string): Promise<TenantMember[]>;
  getProvisioningStatus(tenantId: string): Promise<ProvisioningStatus>;
}

export class TenantError extends Error {
  constructor(
    message: string,
    public readonly code: 'NOT_FOUND' | 'SUSPENDED' | 'SERVER_ERROR' | 'TIMEOUT'
  );
}

// NOTE: TenantError コード命名規則の言語別対応表
// | 概念          | Rust                       | Go (標準 error) | TypeScript              | Dart                          |
// |---------------|----------------------------|-----------------|-------------------------|-------------------------------|
// | 未発見        | TenantError::NotFound(_)   | errors.New(...) | code: 'NOT_FOUND'       | TenantErrorCode.notFound      |
// | 停止中        | TenantError::Suspended(_)  | errors.New(...) | code: 'SUSPENDED'       | TenantErrorCode.suspended     |
// | サーバーエラー | TenantError::ServerError(_)| errors.New(...) | code: 'SERVER_ERROR'    | TenantErrorCode.serverError   |
// | タイムアウト  | TenantError::Timeout(_)    | errors.New(...) | code: 'TIMEOUT'         | TenantErrorCode.timeout       |
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/tenant_client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**pubspec.yaml 主要依存**:

```yaml
dependencies: {}
```

**主要 API**:

```dart
// 設定
class TenantClientConfig {
  const TenantClientConfig({
    required String serverUrl,
    Duration cacheTtl = const Duration(minutes: 5),
    int cacheMaxCapacity = 1000,
  });
  final String serverUrl;
  final Duration cacheTtl;
  final int cacheMaxCapacity;
}

// テナント情報
enum TenantStatus { active, suspended, deleted }

class Tenant {
  const Tenant({
    required this.id,
    required this.name,
    required this.status,
    required this.plan,
    required this.settings,
    required this.createdAt,
  });
  final String id;
  final String name;
  final TenantStatus status;
  final String plan;
  final Map<String, String> settings;
  final DateTime createdAt;
}

class TenantFilter {
  const TenantFilter({this.status, this.plan});
  final TenantStatus? status;
  final String? plan;
}

class TenantSettings {
  const TenantSettings(this.values);
  final Map<String, String> values;
  String? get(String key)
}

class CreateTenantRequest {
  const CreateTenantRequest({
    required this.name,
    required this.plan,
    this.adminUserId,
  });
  final String name;
  final String plan;
  final String? adminUserId;
}

class TenantMember {
  const TenantMember({
    required this.userId,
    required this.role,
    required this.joinedAt,
  });
  final String userId;
  final String role;
  final DateTime joinedAt;
}

enum ProvisioningStatus { pending, inProgress, completed, failed }

// クライアントインターフェース
abstract class TenantClient {
  Future<Tenant> getTenant(String tenantId);
  Future<List<Tenant>> listTenants(TenantFilter filter);
  Future<bool> isActive(String tenantId);
  Future<TenantSettings> getSettings(String tenantId);
  Future<Tenant> createTenant(CreateTenantRequest req);
  Future<TenantMember> addMember(String tenantId, String userId, String role);
  Future<void> removeMember(String tenantId, String userId);
  Future<List<TenantMember>> listMembers(String tenantId);
  Future<ProvisioningStatus> getProvisioningStatus(String tenantId);
}

// HTTP 接続実装
class HttpTenantClient implements TenantClient {
  HttpTenantClient(TenantClientConfig config);
  Future<Tenant> getTenant(String tenantId);
  Future<List<Tenant>> listTenants(TenantFilter filter);
  Future<bool> isActive(String tenantId);
  Future<TenantSettings> getSettings(String tenantId);
  Future<Tenant> createTenant(CreateTenantRequest req);
  Future<TenantMember> addMember(String tenantId, String userId, String role);
  Future<void> removeMember(String tenantId, String userId);
  Future<List<TenantMember>> listMembers(String tenantId);
  Future<ProvisioningStatus> getProvisioningStatus(String tenantId);
  Future<void> close();
}

// インメモリ実装（テスト用）
class InMemoryTenantClient implements TenantClient {
  InMemoryTenantClient([List<Tenant>? tenants]);
  void addTenant(Tenant tenant);
  Future<Tenant> getTenant(String tenantId);
  Future<List<Tenant>> listTenants(TenantFilter filter);
  Future<bool> isActive(String tenantId);
  Future<TenantSettings> getSettings(String tenantId);
  Future<Tenant> createTenant(CreateTenantRequest req);
  Future<TenantMember> addMember(String tenantId, String userId, String role);
  Future<void> removeMember(String tenantId, String userId);
  Future<List<TenantMember>> listMembers(String tenantId);
  Future<ProvisioningStatus> getProvisioningStatus(String tenantId);
}

// エラー型
enum TenantErrorCode { notFound, suspended, serverError, timeout }

class TenantError implements Exception {
  const TenantError(this.message, this.code);
  final String message;
  final TenantErrorCode code;
  String toString()
}
```

**使用例**:

```dart
import 'package:k1s0_tenant_client/tenant_client.dart';

final config = TenantClientConfig(
  serverUrl: 'tenant-server:8080',
  cacheTtl: Duration(minutes: 5),
  cacheMaxCapacity: 1000,
);
final client = HttpTenantClient(config);

// テナント一覧の取得（アクティブのみ）
final tenants = await client.listTenants(
  const TenantFilter(status: TenantStatus.active),
);

// 特定テナントの取得
final tenant = await client.getTenant('TENANT-001');

// X-Tenant-ID ヘッダーのテナント検証
final tenantId = request.headers['X-Tenant-ID'];
final isActive = await client.isActive(tenantId);
if (!isActive) {
  throw TenantError('Tenant not active', TenantErrorCode.suspended);
}

final settings = await client.getSettings(tenantId);
final maxUsers = settings.get('max_users') ?? '100';
```

**エラーハンドリング**:

```dart
try {
  final tenant = await client.getTenant(tenantId);
} on TenantError catch (e) {
  switch (e.code) {
    case TenantErrorCode.notFound:
      // テナントが存在しない
      break;
    case TenantErrorCode.suspended:
      // テナントが停止中
      break;
    case TenantErrorCode.serverError:
      // サーバーエラー
      break;
    case TenantErrorCode.timeout:
      // タイムアウト
      break;
  }
}
```

**カバレッジ目標**: 90%以上

## テスト戦略

### ユニットテスト（`#[cfg(test)]`）

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_status_active() {
        let status = TenantStatus::Active;
        assert!(matches!(status, TenantStatus::Active));
    }

    #[test]
    fn test_tenant_filter_builder() {
        let filter = TenantFilter::new().status(TenantStatus::Active);
        assert_eq!(filter.status, Some(TenantStatus::Active));
    }

    #[test]
    fn test_tenant_error_not_found() {
        let err = TenantError::NotFound("TENANT-999".to_string());
        assert!(matches!(err, TenantError::NotFound(_)));
    }
}
```

### 統合テスト

- `testcontainers` で tenant-server コンテナを起動して実際の get/list フローを検証
- キャッシュ TTL 経過後に再取得が発生することを確認
- 存在しないテナント ID で `NotFound` エラーが返ることを確認
- 停止テナントで `is_active` が `false` を返すことを確認

### モックテスト

```rust
use mockall::mock;

mock! {
    pub TestTenantClient {}
    #[async_trait]
    impl TenantClient for TestTenantClient {
        async fn get_tenant(&self, tenant_id: &str) -> Result<Tenant, TenantError>;
        async fn list_tenants(&self, filter: TenantFilter) -> Result<Vec<Tenant>, TenantError>;
        async fn is_active(&self, tenant_id: &str) -> Result<bool, TenantError>;
        async fn get_settings(&self, tenant_id: &str) -> Result<TenantSettings, TenantError>;
    }
}

#[tokio::test]
async fn test_middleware_rejects_inactive_tenant() {
    let mut mock = MockTestTenantClient::new();
    mock.expect_is_active()
        .once()
        .returning(|_| Ok(false));

    let middleware = TenantMiddleware::new(Arc::new(mock));
    let result = middleware.check("TENANT-SUSPENDED").await;
    assert!(result.is_err());
}
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-tenant-server設計](../../servers/tenant/server.md) — テナントサーバー設計
- [system-library-ratelimit-client設計](ratelimit-client.md) — レート制限クライアント（テナント ID キー連携）
- [system-library-cache設計](../data/cache.md) — k1s0-cache ライブラリ（キャッシュ基盤）
- [system-library-correlation設計](../observability/correlation.md) — トレース ID 伝播（X-Tenant-ID ヘッダー連携）
