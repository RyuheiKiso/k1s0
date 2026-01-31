# k1s0-cache

## 目的

Redis キャッシュクライアントの標準化を提供する。Cache-Aside パターン、TTL 管理をサポート。

## 主要な型

### CacheConfig

```rust
pub struct CacheConfig {
    pub host: String,
    pub port: u16,
    pub key_prefix: String,
    pub default_ttl_secs: Option<u64>,
}

impl CacheConfig {
    pub fn builder() -> CacheConfigBuilder;
}
```

## TTL 管理戦略

### TTL 設定のベストプラクティス

| データ種別 | 推奨 TTL | 理由 |
|-----------|---------|------|
| セッション | 30分-24時間 | セキュリティ要件に依存 |
| ユーザープロファイル | 1-24時間 | 更新頻度が低い |
| 商品カタログ | 5-60分 | 更新時に明示的に無効化 |
| API レスポンス | 1-5分 | リアルタイム性とのバランス |
| 計算結果キャッシュ | 処理時間に比例 | 再計算コストに応じて |

### TTL 設定パターン

```rust
/// TTL 設定の定数定義（推奨）
pub mod cache_ttl {
    use std::time::Duration;

    pub const SESSION: Duration = Duration::from_secs(30 * 60);        // 30分
    pub const USER_PROFILE: Duration = Duration::from_secs(60 * 60);   // 1時間
    pub const CATALOG: Duration = Duration::from_secs(5 * 60);         // 5分
    pub const API_RESPONSE: Duration = Duration::from_secs(60);        // 1分
}

// 使用例
client.set("user:123", &user, Some(cache_ttl::USER_PROFILE)).await?;
```

### スライディング TTL

アクセスごとに TTL をリセットするパターン。セッション管理に有効。

```rust
/// アクセス時に TTL を延長
pub async fn get_with_refresh<T>(&self, key: &str, ttl: Duration) -> CacheResult<Option<T>> {
    if let Some(value) = self.get::<T>(key).await? {
        // TTL をリセット
        self.expire(key, ttl).await?;
        Ok(Some(value))
    } else {
        Ok(None)
    }
}
```

## エビクションポリシー

### Redis のエビクションポリシー設定

| ポリシー | 説明 | ユースケース |
|---------|------|-------------|
| `noeviction` | メモリ上限でエラー | データ損失を許容しない |
| `allkeys-lru` | LRU で削除 | 一般的なキャッシュ |
| `volatile-lru` | TTL 付きキーを LRU で削除 | 永続データとキャッシュの混在 |
| `allkeys-lfu` | LFU で削除 | アクセス頻度重視 |
| `volatile-ttl` | TTL が近いキーを優先削除 | 時間ベースの優先度 |

**推奨設定（redis.conf）:**
```
maxmemory 1gb
maxmemory-policy allkeys-lru
maxmemory-samples 10
```

### メモリ使用量の監視

```rust
// Redis INFO コマンドでメモリ使用量を取得
let info = client.info("memory").await?;
let used_memory: u64 = info.get("used_memory")?;
let maxmemory: u64 = info.get("maxmemory")?;

let usage_ratio = used_memory as f64 / maxmemory as f64;
if usage_ratio > 0.8 {
    tracing::warn!(usage = %usage_ratio, "Cache memory usage high");
}
```

### キャッシュウォーミング

コールドスタート時のキャッシュミス急増を防ぐ。

```rust
/// 起動時にキャッシュをプリロード
pub async fn warm_cache(&self, keys: &[String]) -> CacheResult<()> {
    let batch_size = 100;
    for chunk in keys.chunks(batch_size) {
        let futures: Vec<_> = chunk.iter()
            .map(|key| self.load_to_cache(key))
            .collect();
        futures::future::join_all(futures).await;
    }
    Ok(())
}
```

## CacheOperations トレイト

```rust
#[async_trait]
pub trait CacheOperations: Send + Sync {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> CacheResult<Option<T>>;
    async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> CacheResult<()>;
    async fn delete(&self, key: &str) -> CacheResult<bool>;
    async fn exists(&self, key: &str) -> CacheResult<bool>;
    async fn get_or_set<T, F, Fut>(&self, key: &str, f: F, ttl: Option<Duration>) -> CacheResult<T>;
}

/// パターンマッチング削除 (CacheOperationsExt)
#[async_trait]
pub trait CacheOperationsExt: CacheOperations {
    /// パターンに一致するキーを削除
    /// Redis の SCAN + DEL を使用
    async fn delete_pattern(&self, pattern: &str) -> CacheResult<u64>;
}

impl CacheClient {
    /// パターンに一致するキーを削除
    ///
    /// パターン構文:
    /// - `*` - 任意の文字列にマッチ
    /// - `?` - 任意の1文字にマッチ
    /// - `[abc]` - a, b, c のいずれかにマッチ
    pub async fn delete_by_pattern(&self, pattern: &str) -> CacheResult<u64>;

    /// パターンに一致するキーをスキャン
    pub async fn scan_keys(&self, pattern: &str) -> CacheResult<Vec<String>>;
}
```

## Go 版（k1s0-cache）

```go
// PatternDeleter is an interface for caches that support pattern-based deletion.
type PatternDeleter interface {
    // DeletePattern deletes all keys matching the pattern.
    DeletePattern(ctx context.Context, pattern string) (int64, error)

    // Scan iterates over keys matching a pattern.
    Scan(ctx context.Context, pattern string, count int64) ([]string, error)
}

// InvalidatePattern deletes all keys matching the pattern.
func InvalidatePattern(ctx context.Context, client *CacheClient, pattern string) (int64, error)

// ScanKeys returns all keys matching the pattern.
func ScanKeys(ctx context.Context, client *CacheClient, pattern string) ([]string, error)

// DeletePattern is a method on CacheClient.
func (c *CacheClient) DeletePattern(ctx context.Context, pattern string) (int64, error)
```

## Features

```toml
[features]
default = []
redis = ["dep:redis", "dep:bb8", "dep:bb8-redis"]
health = ["dep:k1s0-health"]
full = ["redis", "health"]
```

## 使用例

```rust
use k1s0_cache::{CacheConfig, CacheClient, CacheOperations};
use std::time::Duration;

let config = CacheConfig::builder()
    .host("localhost")
    .port(6379)
    .key_prefix("myapp")
    .build()?;

let client = CacheClient::new(config).await?;

// 値の設定
client.set("user:123", &user, Some(Duration::from_secs(3600))).await?;

// 値の取得
let user: Option<User> = client.get("user:123").await?;

// Cache-Aside パターン
let user = client.get_or_set(
    "user:123",
    || async { db.find_user("123").await },
    Some(Duration::from_secs(3600)),
).await?;
```

## Write-Through パターン

DB と Cache に同時に書き込むパターン。データの一貫性を重視する場合に使用。

```rust
pub struct WriteThrough<T, D, C>
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync,
    D: DbOperations<T> + Send + Sync,
    C: CacheOperations + Send + Sync,
{
    db: Arc<D>,
    cache: Arc<C>,
    key_prefix: String,
    default_ttl: Option<Duration>,
}

impl<T, D, C> WriteThrough<T, D, C> {
    pub fn new(db: Arc<D>, cache: Arc<C>, key_prefix: impl Into<String>) -> Self;
    pub fn with_ttl(self, ttl: Duration) -> Self;

    /// DB に書き込み、成功後キャッシュにも書き込み
    pub async fn write(&self, key: &str, value: &T) -> CacheResult<()>;

    /// DB に書き込み、成功後キャッシュにも書き込み（TTL指定）
    pub async fn write_with_ttl(&self, key: &str, value: &T, ttl: Duration) -> CacheResult<()>;

    /// Cache-Aside フォールバック付きで読み取り
    pub async fn read(&self, key: &str) -> CacheResult<Option<T>>;

    /// 複数エントリを一括書き込み
    pub async fn write_batch(&self, entries: &[(String, T)]) -> CacheResult<()>;
}
```

## Go 版

```go
type WriteThrough[T any] struct {
    db        DbOperations[T]
    cache     CacheOperations
    keyPrefix string
    ttl       time.Duration
}

func NewWriteThrough[T any](db DbOperations[T], cache CacheOperations, keyPrefix string) *WriteThrough[T]
func (w *WriteThrough[T]) WithTTL(ttl time.Duration) *WriteThrough[T]
func (w *WriteThrough[T]) Write(ctx context.Context, key string, value *T) error
func (w *WriteThrough[T]) Read(ctx context.Context, key string) (*T, error)
func (w *WriteThrough[T]) WriteBatch(ctx context.Context, entries map[string]*T) error
```

## Write-Behind パターン

キャッシュに即座に書き込み、DB への書き込みを非同期で行うパターン。書き込み性能を重視する場合に使用。

```rust
pub struct WriteBehind<T, D, C>
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    D: DbOperations<T> + Send + Sync + 'static,
    C: CacheOperations + Send + Sync + 'static,
{
    db: Arc<D>,
    cache: Arc<C>,
    key_prefix: String,
    sender: mpsc::Sender<WriteOperation<T>>,
    stats: Arc<WriteBehindStats>,
}

pub struct WriteOperation<T> {
    pub key: String,
    pub value: T,
    pub operation_type: WriteOperationType,
}

pub enum WriteOperationType {
    Insert,
    Update,
    Delete,
}

pub struct WriteBehindConfig {
    pub batch_size: usize,          // デフォルト: 100
    pub flush_interval: Duration,   // デフォルト: 1秒
    pub max_retries: u32,           // デフォルト: 3
    pub retry_delay: Duration,      // デフォルト: 100ms
}

pub struct WriteBehindStats {
    pub total_writes: AtomicU64,
    pub successful_writes: AtomicU64,
    pub failed_writes: AtomicU64,
    pub pending_writes: AtomicU64,
}

impl<T, D, C> WriteBehind<T, D, C> {
    pub async fn new(
        db: Arc<D>,
        cache: Arc<C>,
        key_prefix: impl Into<String>,
        config: WriteBehindConfig,
    ) -> CacheResult<Self>;

    /// キャッシュに即座に書き込み、DB への書き込みをキューに追加
    pub async fn write(&self, key: &str, value: T) -> CacheResult<()>;

    /// 削除をキューに追加
    pub async fn delete(&self, key: &str) -> CacheResult<()>;

    /// キャッシュから読み取り（DB フォールバックあり）
    pub async fn read(&self, key: &str) -> CacheResult<Option<T>>;

    /// 統計情報を取得
    pub fn stats(&self) -> &WriteBehindStats;

    /// バックグラウンドワーカーを停止
    pub async fn shutdown(&self) -> CacheResult<()>;
}
```

## Go 版

```go
type WriteBehind[T any] struct {
    db         DbOperations[T]
    cache      CacheOperations
    keyPrefix  string
    writeChan  chan WriteOperation[T]
    stats      *WriteBehindStats
    stopCh     chan struct{}
}

type WriteBehindConfig struct {
    BatchSize     int           // default: 100
    FlushInterval time.Duration // default: 1s
    MaxRetries    int           // default: 3
    RetryDelay    time.Duration // default: 100ms
}

type WriteBehindStats struct {
    TotalWrites      atomic.Uint64
    SuccessfulWrites atomic.Uint64
    FailedWrites     atomic.Uint64
    PendingWrites    atomic.Uint64
}

func NewWriteBehind[T any](db DbOperations[T], cache CacheOperations, keyPrefix string, config WriteBehindConfig) *WriteBehind[T]
func (w *WriteBehind[T]) Write(ctx context.Context, key string, value *T) error
func (w *WriteBehind[T]) Delete(ctx context.Context, key string) error
func (w *WriteBehind[T]) Read(ctx context.Context, key string) (*T, error)
func (w *WriteBehind[T]) Stats() WriteBehindStats
func (w *WriteBehind[T]) Shutdown(ctx context.Context) error
```

## C# 版（K1s0.Cache）

StackExchange.Redis ベースのキャッシュライブラリ。

### 主要な型

```csharp
public class CacheConfig
{
    public string Host { get; set; } = "localhost";
    public int Port { get; set; } = 6379;
    public string KeyPrefix { get; set; } = "";
    public TimeSpan? DefaultTtl { get; set; }
    public static CacheConfigBuilder Builder();
}

public interface ICacheOperations
{
    Task<T?> GetAsync<T>(string key);
    Task SetAsync<T>(string key, T value, TimeSpan? ttl = null);
    Task<bool> DeleteAsync(string key);
    Task<bool> ExistsAsync(string key);
    Task<T> GetOrSetAsync<T>(string key, Func<Task<T>> factory, TimeSpan? ttl = null);
    Task<long> DeletePatternAsync(string pattern);
}

public class CacheClient : ICacheOperations
{
    public CacheClient(CacheConfig config);
}
```

### 使用例

```csharp
using K1s0.Cache;

var config = CacheConfig.Builder()
    .Host("localhost").Port(6379).KeyPrefix("myapp").Build();
var client = new CacheClient(config);

await client.SetAsync("user:123", user, TimeSpan.FromHours(1));
var user = await client.GetAsync<User>("user:123");
var user = await client.GetOrSetAsync("user:123",
    () => db.FindUserAsync("123"), TimeSpan.FromHours(1));
```

## Python 版（k1s0-cache）

### 主要な型

```python
@dataclass
class CacheConfig:
    host: str = "localhost"
    port: int = 6379
    key_prefix: str = ""
    default_ttl: timedelta | None = None

class CacheOperations(ABC):
    @abstractmethod
    async def get(self, key: str, model: type[T]) -> T | None: ...
    @abstractmethod
    async def set(self, key: str, value: Any, ttl: timedelta | None = None) -> None: ...
    @abstractmethod
    async def delete(self, key: str) -> bool: ...
    @abstractmethod
    async def exists(self, key: str) -> bool: ...
    @abstractmethod
    async def get_or_set(self, key: str, factory: Callable, ttl: timedelta | None = None) -> Any: ...
    @abstractmethod
    async def delete_pattern(self, pattern: str) -> int: ...

class CacheClient(CacheOperations):
    def __init__(self, config: CacheConfig) -> None: ...
```

### 使用例

```python
from k1s0_cache import CacheClient, CacheConfig
from datetime import timedelta

config = CacheConfig(host="localhost", port=6379, key_prefix="myapp")
client = CacheClient(config)

await client.set("user:123", user, ttl=timedelta(hours=1))
user = await client.get("user:123", User)
user = await client.get_or_set("user:123",
    lambda: db.find_user("123"), ttl=timedelta(hours=1))
```

## Kotlin 版（k1s0-cache）

Lettuce ベースのキャッシュライブラリ。

### 主要な型

```kotlin
data class CacheConfig(
    val host: String = "localhost",
    val port: Int = 6379,
    val keyPrefix: String = "",
    val defaultTtl: Duration? = null
) {
    class Builder {
        fun host(host: String): Builder
        fun port(port: Int): Builder
        fun keyPrefix(prefix: String): Builder
        fun build(): CacheConfig
    }
}

interface CacheOperations {
    suspend fun <T> get(key: String, type: KClass<T>): T?
    suspend fun <T : Any> set(key: String, value: T, ttl: Duration? = null)
    suspend fun delete(key: String): Boolean
    suspend fun exists(key: String): Boolean
    suspend fun <T : Any> getOrSet(key: String, ttl: Duration? = null, factory: suspend () -> T): T
    suspend fun deletePattern(pattern: String): Long
}

class CacheClient(config: CacheConfig) : CacheOperations
```

### 使用例

```kotlin
import com.k1s0.cache.*
import kotlin.time.Duration.Companion.hours

val config = CacheConfig.Builder()
    .host("localhost").port(6379).keyPrefix("myapp").build()
val client = CacheClient(config)

client.set("user:123", user, ttl = 1.hours)
val user = client.get("user:123", User::class)
val user = client.getOrSet("user:123", ttl = 1.hours) { db.findUser("123") }
```

## キャッシュパターン使い分け

| パターン | 一貫性 | 書き込み性能 | 読み取り性能 | 適用場面 |
|---------|:-----:|:----------:|:----------:|---------|
| Cache-Aside | 中 | 低 | 高 | 読み取り中心のワークロード |
| Write-Through | 高 | 低 | 高 | データ一貫性が最重要 |
| Write-Behind | 低 | 高 | 高 | 書き込み中心、一時的な不整合許容 |
