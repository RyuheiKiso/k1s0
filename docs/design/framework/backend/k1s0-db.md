# k1s0-db

## 目的

データベース接続、プール管理、トランザクション、リポジトリパターンの標準化を提供する。

## 設計原則

1. **Clean Architecture 対応**: domain/application 層用インターフェース
2. **トランザクション境界**: Unit of Work パターン
3. **リポジトリパターン**: CRUD 抽象化、ページング対応
4. **PostgreSQL 重視**: SQLx による実装

## 主要な型

### DbConfig

```rust
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password_file: Option<String>,
    pub ssl_mode: SslMode,
    pub pool: PoolConfig,
    pub timeout: TimeoutConfig,
}

impl DbConfig {
    pub fn builder() -> DbConfigBuilder;
}
```

### PoolConfig

```rust
pub struct PoolConfig {
    pub max_connections: u32,      // デフォルト: 10
    pub min_connections: u32,      // デフォルト: 1
    pub idle_timeout_secs: u64,    // デフォルト: 600
    pub max_lifetime_secs: u64,    // デフォルト: 1800
}
```

## コネクションプール詳細

### プールサイズの設定指針

| 環境 | max_connections | min_connections | 備考 |
|------|-----------------|-----------------|------|
| 開発 | 5 | 1 | リソース節約 |
| ステージング | 10 | 2 | 本番に近い設定 |
| 本番 | 20-50 | 5-10 | トラフィックに応じて調整 |

**計算式の目安:**
```
max_connections = (CPU コア数 * 2) + effective_spindle_count
```

### タイムアウト設定

| パラメータ | 説明 | 推奨値 |
|-----------|------|--------|
| `idle_timeout_secs` | アイドル接続のタイムアウト | 300-600秒 |
| `max_lifetime_secs` | 接続の最大生存時間 | 1800秒（30分） |
| `connection_timeout_secs` | 接続確立のタイムアウト | 5-10秒 |
| `acquire_timeout_secs` | プールからの取得タイムアウト | 5-30秒 |

### プール監視

```rust
// プールの状態を取得
let pool_state = pool.state();
println!("接続数: {} / {}", pool_state.connections, pool_state.max_connections);
println!("アイドル接続: {}", pool_state.idle_connections);
println!("待機中リクエスト: {}", pool_state.pending_requests);

// メトリクス出力（OpenTelemetry 連携）
pool.export_metrics(&meter);
```

### 枯渇対策

1. **接続リーク検知**: `acquire_timeout` を設定し、取得できない場合にエラーログを出力
2. **ヘルスチェック**: 定期的に `pool.health_check()` を実行
3. **アラート設定**: `pending_requests > 0` が続く場合に通知

```rust
// 接続ヘルスチェック
if let Err(e) = pool.health_check().await {
    tracing::error!(error = %e, "Pool health check failed");
}
```

## TransactionOptions

```rust
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,      // デフォルト
    RepeatableRead,
    Serializable,
}

pub enum TransactionMode {
    ReadWrite,          // デフォルト
    ReadOnly,
}

pub struct TransactionOptions {
    pub isolation_level: IsolationLevel,
    pub mode: TransactionMode,
}

impl TransactionOptions {
    pub fn new() -> Self;
    pub fn read_only() -> Self;
    pub fn serializable() -> Self;
    pub fn with_isolation_level(self, level: IsolationLevel) -> Self;
}
```

## 分離レベル選択ガイド

| 分離レベル | ユースケース | 注意点 |
|-----------|-------------|--------|
| `ReadUncommitted` | ダーティリードを許容する集計処理 | 本番では非推奨 |
| `ReadCommitted` | 一般的な CRUD 操作（デフォルト） | ファントムリードの可能性あり |
| `RepeatableRead` | レポート生成、一貫性が必要な読み取り | 更新競合に注意 |
| `Serializable` | 金融取引、在庫管理 | デッドロックリスク、性能低下 |

### ユースケース別の推奨設定

**1. 通常の CRUD 操作:**
```rust
// デフォルト（ReadCommitted）を使用
let options = TransactionOptions::new();
```

**2. 残高更新などの金融取引:**
```rust
// Serializable + リトライロジック
let options = TransactionOptions::serializable();
let result = retry_with_backoff(3, || async {
    uow.execute_with_options(&options, |tx| async {
        let balance = tx.get_balance(user_id).await?;
        tx.update_balance(user_id, balance - amount).await
    }).await
}).await;
```

**3. 大量データの読み取り専用レポート:**
```rust
// ReadOnly + RepeatableRead
let options = TransactionOptions::read_only()
    .with_isolation_level(IsolationLevel::RepeatableRead);
```

**4. 楽観的ロックとの組み合わせ:**
```rust
// ReadCommitted + version チェック
let entity = repository.find_by_id(id).await?;
entity.version += 1;
match repository.update_with_version(&entity).await {
    Err(DbError::OptimisticLockError) => {
        // リトライまたはエラー
    }
    Ok(_) => {}
}
```

### デッドロック対策

1. **一貫したロック順序**: テーブル/行のロック順序を統一
2. **タイムアウト設定**: `statement_timeout` を設定
3. **リトライロジック**: デッドロック検出時に自動リトライ

```rust
// デッドロック検出とリトライ
pub async fn with_deadlock_retry<F, T>(f: F) -> DbResult<T>
where
    F: Fn() -> Future<Output = DbResult<T>>,
{
    for attempt in 0..3 {
        match f().await {
            Err(DbError::Deadlock) => {
                tracing::warn!(attempt, "Deadlock detected, retrying");
                tokio::time::sleep(Duration::from_millis(100 * (attempt + 1))).await;
            }
            result => return result,
        }
    }
    Err(DbError::MaxRetriesExceeded)
}
```

## Repository トレイト

```rust
#[async_trait]
pub trait Repository<T, ID: ?Sized>: Send + Sync {
    async fn find_by_id(&self, id: &ID) -> DbResult<Option<T>>;
    async fn find_all(&self) -> DbResult<Vec<T>>;
    async fn save(&self, entity: &T) -> DbResult<T>;
    async fn delete(&self, id: &ID) -> DbResult<bool>;
}

#[async_trait]
pub trait PagedRepository<T, ID>: Repository<T, ID> {
    async fn find_paginated(&self, pagination: &Pagination) -> DbResult<PagedResult<T>>;
}
```

## Pagination

```rust
pub struct Pagination {
    pub page: u64,          // 1から開始
    pub page_size: u64,     // 1-1000
}

pub struct PagedResult<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

impl<T> PagedResult<T> {
    pub fn has_next_page(&self) -> bool;
    pub fn has_prev_page(&self) -> bool;
}
```

## Unit of Work

```rust
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn begin(&self) -> DbResult<()>;
    async fn commit(&self) -> DbResult<()>;
    async fn rollback(&self) -> DbResult<()>;
}

pub async fn execute_in_transaction<F, T, E>(
    uow: &impl UnitOfWork,
    f: F,
) -> DbResult<T>
where
    F: FnOnce() -> Future<Output = Result<T, E>>,
    E: Into<DbError>;
```

## UnitOfWork 拡張機能

```rust
/// 完全な UnitOfWork 実装
pub struct UnitOfWorkImpl {
    pool: DbPool,
    tx: Option<Transaction<'static, Postgres>>,
    savepoints: Vec<String>,
    isolation_level: IsolationLevel,
}

impl UnitOfWorkImpl {
    pub fn new(pool: DbPool) -> Self;

    /// トランザクション分離レベルを設定
    pub fn with_isolation_level(mut self, level: IsolationLevel) -> Self;

    /// セーブポイントを作成
    pub async fn savepoint(&mut self, name: &str) -> DbResult<()>;

    /// セーブポイントまでロールバック
    pub async fn rollback_to_savepoint(&mut self, name: &str) -> DbResult<()>;

    /// セーブポイントを解放
    pub async fn release_savepoint(&mut self, name: &str) -> DbResult<()>;

    /// トランザクション内で処理を実行
    pub async fn execute<F, T, E>(&mut self, f: F) -> DbResult<T>
    where
        F: FnOnce(&mut Self) -> Future<Output = Result<T, E>> + Send,
        E: Into<DbError>;
}
```

## ScopedUnitOfWork

スコープ終了時に自動でロールバックする Unit of Work。

```rust
/// スコープベースの UnitOfWork
pub struct ScopedUnitOfWork {
    inner: UnitOfWorkImpl,
    committed: bool,
}

impl ScopedUnitOfWork {
    pub fn new(pool: DbPool) -> Self;

    /// トランザクションをコミット（呼び出さない場合はドロップ時にロールバック）
    pub async fn commit(mut self) -> DbResult<()>;
}

impl Drop for ScopedUnitOfWork {
    fn drop(&mut self) {
        // committed が false の場合、自動でロールバック
    }
}
```

## MultiTableUnitOfWork

複数テーブルの操作をまとめて管理する UnitOf Work。

```rust
/// 複数テーブル操作用 UnitOfWork
pub struct MultiTableUnitOfWork {
    inner: UnitOfWorkImpl,
    operations: Vec<Operation>,
}

pub enum Operation {
    Insert { table: String, data: Value },
    Update { table: String, id: String, data: Value },
    Delete { table: String, id: String },
}

impl MultiTableUnitOfWork {
    pub fn new(pool: DbPool) -> Self;

    /// 操作を追加
    pub fn add_operation(&mut self, op: Operation);

    /// すべての操作を実行してコミット
    pub async fn execute_all(&mut self) -> DbResult<()>;

    /// 特定のテーブルのみ実行
    pub async fn execute_for_table(&mut self, table: &str) -> DbResult<()>;
}
```

## UnitOfWorkFactory

```rust
/// UnitOfWork ファクトリー
pub struct UnitOfWorkFactory {
    pool: DbPool,
}

impl UnitOfWorkFactory {
    pub fn new(pool: DbPool) -> Self;

    /// 基本の UnitOfWork を作成
    pub fn create(&self) -> UnitOfWorkImpl;

    /// スコープベースの UnitOfWork を作成
    pub fn create_scoped(&self) -> ScopedUnitOfWork;

    /// 複数テーブル用 UnitOfWork を作成
    pub fn create_multi_table(&self) -> MultiTableUnitOfWork;

    /// 特定の分離レベルで UnitOfWork を作成
    pub fn create_with_isolation(&self, level: IsolationLevel) -> UnitOfWorkImpl;
}
```

## Features

```toml
[features]
default = []
postgres = ["sqlx"]
full = ["postgres"]
```

## 使用例

```rust
use k1s0_db::{DbConfig, DbPoolBuilder, Repository, Pagination};

// 接続設定
let config = DbConfig::builder()
    .host("localhost")
    .database("myapp")
    .username("app_user")
    .password_file("/run/secrets/db_password")
    .build()?;

// プール作成
let pool = DbPoolBuilder::new()
    .host(&config.host)
    .database(&config.database)
    .build()
    .await?;

// ページネーション
let pagination = Pagination { page: 1, page_size: 20 };
let result = repository.find_paginated(&pagination).await?;
```

## Go 版（k1s0-db）

### 主要な型

```go
// DbConfig はデータベース接続設定。
type DbConfig struct {
    Host         string
    Port         int
    Database     string
    Username     string
    PasswordFile string
    SSLMode      string
    Pool         PoolConfig
}

type PoolConfig struct {
    MaxConnections  int // デフォルト: 10
    MinConnections  int // デフォルト: 1
    IdleTimeoutSecs int // デフォルト: 600
}

// Repository はリポジトリインターフェース。
type Repository[T any, ID any] interface {
    FindByID(ctx context.Context, id ID) (*T, error)
    FindAll(ctx context.Context) ([]T, error)
    Save(ctx context.Context, entity *T) (*T, error)
    Delete(ctx context.Context, id ID) (bool, error)
}

// UnitOfWork はトランザクション管理。
type UnitOfWork interface {
    Begin(ctx context.Context) error
    Commit(ctx context.Context) error
    Rollback(ctx context.Context) error
}

// Pagination はページネーション。
type Pagination struct {
    Page     uint64
    PageSize uint64
}

type PagedResult[T any] struct {
    Data       []T
    Total      uint64
    Page       uint64
    PageSize   uint64
    TotalPages uint64
}
```

### 使用例

```go
import k1s0db "github.com/k1s0/framework/backend/go/k1s0-db"

config := k1s0db.DbConfig{
    Host: "localhost", Database: "myapp", Username: "app_user",
    PasswordFile: "/run/secrets/db_password",
}
pool, err := k1s0db.NewPool(config)

pagination := k1s0db.Pagination{Page: 1, PageSize: 20}
result, err := repo.FindPaginated(ctx, pagination)
```

## C# 版（K1s0.Db）

EF Core ベースのデータベースライブラリ。

### 主要な型

```csharp
public class DbConfig
{
    public string Host { get; set; }
    public int Port { get; set; } = 5432;
    public string Database { get; set; }
    public string Username { get; set; }
    public string? PasswordFile { get; set; }
    public PoolConfig Pool { get; set; } = new();
    public static DbConfigBuilder Builder();
}

public interface IRepository<T, TId> where T : class
{
    Task<T?> FindByIdAsync(TId id);
    Task<List<T>> FindAllAsync();
    Task<T> SaveAsync(T entity);
    Task<bool> DeleteAsync(TId id);
}

public interface IPagedRepository<T, TId> : IRepository<T, TId> where T : class
{
    Task<PagedResult<T>> FindPaginatedAsync(Pagination pagination);
}

public interface IUnitOfWork
{
    Task BeginAsync();
    Task CommitAsync();
    Task RollbackAsync();
}

public record Pagination(ulong Page, ulong PageSize);
public record PagedResult<T>(List<T> Data, ulong Total, ulong Page, ulong PageSize, ulong TotalPages);
```

### 使用例

```csharp
using K1s0.Db;

var config = DbConfig.Builder()
    .Host("localhost")
    .Database("myapp")
    .Username("app_user")
    .PasswordFile("/run/secrets/db_password")
    .Build();

var pagination = new Pagination(1, 20);
var result = await repository.FindPaginatedAsync(pagination);
```

## Python 版（k1s0-db）

SQLAlchemy + asyncpg ベースのデータベースライブラリ。

### 主要な型

```python
@dataclass
class DbConfig:
    host: str
    port: int = 5432
    database: str = ""
    username: str = ""
    password_file: str | None = None
    pool: PoolConfig = field(default_factory=PoolConfig)

class Repository(ABC, Generic[T, ID]):
    @abstractmethod
    async def find_by_id(self, id: ID) -> T | None: ...
    @abstractmethod
    async def find_all(self) -> list[T]: ...
    @abstractmethod
    async def save(self, entity: T) -> T: ...
    @abstractmethod
    async def delete(self, id: ID) -> bool: ...

class PagedRepository(Repository[T, ID], ABC):
    @abstractmethod
    async def find_paginated(self, pagination: Pagination) -> PagedResult[T]: ...

class UnitOfWork(ABC):
    @abstractmethod
    async def begin(self) -> None: ...
    @abstractmethod
    async def commit(self) -> None: ...
    @abstractmethod
    async def rollback(self) -> None: ...

@dataclass
class Pagination:
    page: int
    page_size: int

@dataclass
class PagedResult(Generic[T]):
    data: list[T]
    total: int
    page: int
    page_size: int
    total_pages: int
```

### 使用例

```python
from k1s0_db import DbConfig, Pagination

config = DbConfig(host="localhost", database="myapp", username="app_user",
                  password_file="/run/secrets/db_password")
pool = await create_pool(config)

pagination = Pagination(page=1, page_size=20)
result = await repo.find_paginated(pagination)
```

## Kotlin 版（k1s0-db）

Exposed + HikariCP ベースのデータベースライブラリ。

### 主要な型

```kotlin
data class DbConfig(
    val host: String,
    val port: Int = 5432,
    val database: String,
    val username: String,
    val passwordFile: String? = null,
    val pool: PoolConfig = PoolConfig()
) {
    class Builder {
        fun host(host: String): Builder
        fun database(database: String): Builder
        fun username(username: String): Builder
        fun passwordFile(path: String): Builder
        fun build(): DbConfig
    }
}

interface Repository<T, ID> {
    suspend fun findById(id: ID): T?
    suspend fun findAll(): List<T>
    suspend fun save(entity: T): T
    suspend fun delete(id: ID): Boolean
}

interface PagedRepository<T, ID> : Repository<T, ID> {
    suspend fun findPaginated(pagination: Pagination): PagedResult<T>
}

interface UnitOfWork {
    suspend fun begin()
    suspend fun commit()
    suspend fun rollback()
}

data class Pagination(val page: Long, val pageSize: Long)
data class PagedResult<T>(
    val data: List<T>, val total: Long,
    val page: Long, val pageSize: Long, val totalPages: Long
)
```

### 使用例

```kotlin
import com.k1s0.db.*

val config = DbConfig.Builder()
    .host("localhost")
    .database("myapp")
    .username("app_user")
    .passwordFile("/run/secrets/db_password")
    .build()

val pool = createPool(config)
val result = repo.findPaginated(Pagination(page = 1, pageSize = 20))
```
