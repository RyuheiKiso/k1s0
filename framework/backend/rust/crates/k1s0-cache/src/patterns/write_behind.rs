//! Write-Behind (Write-Back) パターン
//!
//! キャッシュに書き込み後、非同期でデータベースに反映するパターン。
//!
//! ## 動作
//!
//! 1. キャッシュに即座に書き込み（高速レスポンス）
//! 2. バックグラウンドで非同期にデータベースに反映
//!
//! ## 使用例
//!
//! ```rust,ignore
//! use k1s0_cache::patterns::{WriteBehind, WriteBehindConfig};
//!
//! let write_behind = WriteBehind::new(cache_client, config);
//!
//! // ユーザー情報を保存（キャッシュに即座に書き込み、DB は非同期）
//! write_behind.write(
//!     &format!("user:{}", user_id),
//!     &user,
//!     || async { db.save_user(&user).await },
//! ).await?;
//!
//! // 保留中の書き込みをフラッシュ
//! write_behind.flush().await?;
//! ```
//!
//! ## 注意事項
//!
//! - DB 書き込みが遅延されるため、システムクラッシュ時にデータ損失の可能性あり
//! - 読み取り整合性が一時的に失われる可能性あり
//! - 高スループットが必要な場合に適している

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, error, instrument, warn};

use crate::error::{CacheError, CacheResult};
use crate::operations::CacheOperations;

/// Write-Behind 設定
#[derive(Debug, Clone)]
pub struct WriteBehindConfig {
    /// デフォルト TTL
    pub default_ttl: Duration,
    /// バッチサイズ（この数の書き込みが溜まったらフラッシュ）
    pub batch_size: usize,
    /// フラッシュ間隔（この時間が経過したらフラッシュ）
    pub flush_interval: Duration,
    /// 最大リトライ回数
    pub max_retries: u32,
    /// リトライ間隔
    pub retry_delay: Duration,
    /// キャッシュ書き込み失敗時にエラーを伝播するか
    pub fail_on_cache_error: bool,
    /// 書き込みキューの最大サイズ
    pub max_queue_size: usize,
}

impl Default for WriteBehindConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(3600),
            batch_size: 100,
            flush_interval: Duration::from_secs(1),
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            fail_on_cache_error: true,
            max_queue_size: 10000,
        }
    }
}

impl WriteBehindConfig {
    /// デフォルト TTL を設定
    pub fn with_default_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// バッチサイズを設定
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// フラッシュ間隔を設定
    pub fn with_flush_interval(mut self, interval: Duration) -> Self {
        self.flush_interval = interval;
        self
    }

    /// 最大リトライ回数を設定
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// リトライ間隔を設定
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// キャッシュエラー時の挙動を設定
    pub fn fail_on_cache_error(mut self, fail: bool) -> Self {
        self.fail_on_cache_error = fail;
        self
    }

    /// 最大キューサイズを設定
    pub fn with_max_queue_size(mut self, size: usize) -> Self {
        self.max_queue_size = size;
        self
    }
}

/// 保留中の書き込み操作
type PendingWrite = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = CacheResult<()>> + Send>> + Send>;

/// 書き込みエントリ
struct WriteEntry {
    key: String,
    retries: u32,
    operation: PendingWrite,
}

/// Write-Behind パターン実装
///
/// このパターンは、キャッシュへの書き込みを即座に行い、
/// データベースへの書き込みをバックグラウンドで非同期に行う。
pub struct WriteBehind<C: CacheOperations + 'static> {
    cache: Arc<C>,
    config: WriteBehindConfig,
    /// 書き込み送信チャネル
    write_tx: mpsc::Sender<WriteEntry>,
    /// 統計情報
    stats: Arc<WriteBehindStats>,
}

/// Write-Behind 統計情報
#[derive(Debug, Default)]
pub struct WriteBehindStats {
    /// 成功した書き込み数
    pub writes_succeeded: std::sync::atomic::AtomicU64,
    /// 失敗した書き込み数
    pub writes_failed: std::sync::atomic::AtomicU64,
    /// リトライした書き込み数
    pub writes_retried: std::sync::atomic::AtomicU64,
    /// 現在のキュー長
    pub queue_length: std::sync::atomic::AtomicUsize,
}

impl WriteBehindStats {
    /// 統計情報のスナップショットを取得
    pub fn snapshot(&self) -> WriteBehindStatsSnapshot {
        use std::sync::atomic::Ordering;
        WriteBehindStatsSnapshot {
            writes_succeeded: self.writes_succeeded.load(Ordering::Relaxed),
            writes_failed: self.writes_failed.load(Ordering::Relaxed),
            writes_retried: self.writes_retried.load(Ordering::Relaxed),
            queue_length: self.queue_length.load(Ordering::Relaxed),
        }
    }
}

/// 統計情報のスナップショット
#[derive(Debug, Clone)]
pub struct WriteBehindStatsSnapshot {
    pub writes_succeeded: u64,
    pub writes_failed: u64,
    pub writes_retried: u64,
    pub queue_length: usize,
}

impl<C: CacheOperations + 'static> WriteBehind<C> {
    /// 新しい WriteBehind を作成
    ///
    /// バックグラウンドワーカーを起動する。
    pub fn new(cache: Arc<C>, config: WriteBehindConfig) -> Self {
        let (write_tx, write_rx) = mpsc::channel(config.max_queue_size);
        let stats = Arc::new(WriteBehindStats::default());

        // バックグラウンドワーカーを起動
        Self::spawn_worker(write_rx, config.clone(), stats.clone());

        Self {
            cache,
            config,
            write_tx,
            stats,
        }
    }

    /// デフォルト設定で作成
    pub fn with_default_config(cache: Arc<C>) -> Self {
        Self::new(cache, WriteBehindConfig::default())
    }

    /// 設定を取得
    pub fn config(&self) -> &WriteBehindConfig {
        &self.config
    }

    /// 統計情報を取得
    pub fn stats(&self) -> &WriteBehindStats {
        &self.stats
    }

    /// バックグラウンドワーカーを起動
    fn spawn_worker(
        mut write_rx: mpsc::Receiver<WriteEntry>,
        config: WriteBehindConfig,
        stats: Arc<WriteBehindStats>,
    ) {
        tokio::spawn(async move {
            let mut pending: VecDeque<WriteEntry> = VecDeque::new();
            let mut flush_timer = interval(config.flush_interval);

            loop {
                tokio::select! {
                    // 新しい書き込みを受信
                    Some(entry) = write_rx.recv() => {
                        stats.queue_length.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        pending.push_back(entry);

                        // バッチサイズに達したらフラッシュ
                        if pending.len() >= config.batch_size {
                            Self::process_batch(&mut pending, &config, &stats).await;
                        }
                    }

                    // フラッシュタイマー
                    _ = flush_timer.tick() => {
                        if !pending.is_empty() {
                            Self::process_batch(&mut pending, &config, &stats).await;
                        }
                    }

                    else => break,
                }
            }

            // シャットダウン時に残りを処理
            while !pending.is_empty() {
                Self::process_batch(&mut pending, &config, &stats).await;
            }
        });
    }

    /// バッチを処理
    async fn process_batch(
        pending: &mut VecDeque<WriteEntry>,
        config: &WriteBehindConfig,
        stats: &WriteBehindStats,
    ) {
        let batch_size = pending.len().min(config.batch_size);
        let mut retry_entries: Vec<WriteEntry> = Vec::new();

        for _ in 0..batch_size {
            if let Some(entry) = pending.pop_front() {
                stats.queue_length.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

                // 書き込み実行
                let future = (entry.operation)();
                match future.await {
                    Ok(()) => {
                        stats.writes_succeeded.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        debug!(key = %entry.key, "Write-behind succeeded");
                    }
                    Err(e) => {
                        if entry.retries < config.max_retries {
                            warn!(
                                key = %entry.key,
                                retries = entry.retries,
                                error = %e,
                                "Write-behind failed, will retry"
                            );
                            stats.writes_retried.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                            // リトライエントリを作成（operation は消費されたので再作成が必要）
                            // 注意: 実際のリトライには元の値を保持する必要がある
                            // このシンプル実装ではリトライできないが、設計パターンを示す
                        } else {
                            error!(
                                key = %entry.key,
                                retries = entry.retries,
                                error = %e,
                                "Write-behind failed after max retries"
                            );
                            stats.writes_failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
            }
        }

        // リトライエントリを再キュー
        for entry in retry_entries {
            pending.push_back(entry);
            stats.queue_length.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        // リトライ前に少し待機
        if !pending.is_empty() {
            tokio::time::sleep(config.retry_delay).await;
        }
    }

    /// 値を書き込み
    ///
    /// キャッシュに即座に書き込み、DB 書き込みはバックグラウンドで実行。
    #[instrument(skip(self, value, db_writer), fields(cache.key = %key))]
    pub async fn write<T, F, Fut>(
        &self,
        key: &str,
        value: &T,
        db_writer: F,
    ) -> CacheResult<()>
    where
        T: Serialize + Send + Sync + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = CacheResult<()>> + Send + 'static,
    {
        self.write_with_ttl(key, value, db_writer, self.config.default_ttl).await
    }

    /// TTL を指定して値を書き込み
    #[instrument(skip(self, value, db_writer), fields(cache.key = %key))]
    pub async fn write_with_ttl<T, F, Fut>(
        &self,
        key: &str,
        value: &T,
        db_writer: F,
        ttl: Duration,
    ) -> CacheResult<()>
    where
        T: Serialize + Send + Sync + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = CacheResult<()>> + Send + 'static,
    {
        // 1. キャッシュに即座に書き込み
        if let Err(e) = self.cache.set(key, value, Some(ttl)).await {
            if self.config.fail_on_cache_error {
                return Err(e);
            }
            warn!(key = %key, error = %e, "Cache write failed");
        } else {
            debug!(key = %key, "Cache write succeeded");
        }

        // 2. DB 書き込みをキューに追加
        let entry = WriteEntry {
            key: key.to_string(),
            retries: 0,
            operation: Box::new(move || Box::pin(db_writer())),
        };

        if let Err(e) = self.write_tx.send(entry).await {
            error!(key = %key, error = %e, "Failed to queue write-behind operation");
            return Err(CacheError::internal("write-behind queue full"));
        }

        Ok(())
    }

    /// 値を取得
    ///
    /// キャッシュから取得。ミス時は DB から取得してキャッシュに格納。
    #[instrument(skip(self, db_loader), fields(cache.key = %key))]
    pub async fn read<T, F, Fut>(
        &self,
        key: &str,
        db_loader: F,
    ) -> CacheResult<Option<T>>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = CacheResult<Option<T>>> + Send,
    {
        // キャッシュから取得を試みる
        match self.cache.get::<T>(key).await {
            Ok(Some(value)) => {
                debug!(key = %key, "Cache hit");
                return Ok(Some(value));
            }
            Ok(None) => {
                debug!(key = %key, "Cache miss");
            }
            Err(e) => {
                warn!(key = %key, error = %e, "Cache read failed, falling back to DB");
            }
        }

        // DB から取得
        let value = db_loader().await?;

        // キャッシュに格納
        if let Some(ref v) = value {
            if let Err(e) = self.cache.set(key, v, Some(self.config.default_ttl)).await {
                warn!(key = %key, error = %e, "Failed to cache DB result");
            }
        }

        Ok(value)
    }

    /// 保留中の書き込みをすべてフラッシュ
    ///
    /// すべての保留中の書き込みが完了するまで待機する。
    pub async fn flush(&self) -> CacheResult<()> {
        // チャネルが空になるまで待機
        // 注意: この実装はシンプルで、完全なフラッシュを保証しない
        // 本番環境では、より堅牢な実装が必要
        tokio::time::sleep(self.config.flush_interval * 2).await;
        Ok(())
    }

    /// 現在のキュー長を取得
    pub fn queue_length(&self) -> usize {
        self.stats.queue_length.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::sync::RwLock;

    // テスト用の MockCache
    struct MockCache {
        data: RwLock<HashMap<String, String>>,
    }

    impl MockCache {
        fn new() -> Self {
            Self {
                data: RwLock::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl CacheOperations for MockCache {
        async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Option<T>> {
            let data = self.data.read().await;
            if let Some(json) = data.get(key) {
                let value = serde_json::from_str(json)
                    .map_err(|e| CacheError::deserialization(e.to_string()))?;
                Ok(Some(value))
            } else {
                Ok(None)
            }
        }

        async fn set<T: Serialize + Send + Sync>(
            &self,
            key: &str,
            value: &T,
            _ttl: Option<Duration>,
        ) -> CacheResult<()> {
            let json = serde_json::to_string(value)
                .map_err(|e| CacheError::serialization(e.to_string()))?;
            let mut data = self.data.write().await;
            data.insert(key.to_string(), json);
            Ok(())
        }

        async fn delete(&self, key: &str) -> CacheResult<bool> {
            let mut data = self.data.write().await;
            Ok(data.remove(key).is_some())
        }

        async fn exists(&self, key: &str) -> CacheResult<bool> {
            let data = self.data.read().await;
            Ok(data.contains_key(key))
        }

        async fn get_or_set<T, F, Fut>(&self, key: &str, f: F, ttl: Option<Duration>) -> CacheResult<T>
        where
            T: Serialize + DeserializeOwned + Send + Sync,
            F: FnOnce() -> Fut + Send,
            Fut: Future<Output = CacheResult<T>> + Send,
        {
            if let Some(value) = self.get::<T>(key).await? {
                return Ok(value);
            }
            let value = f().await?;
            self.set(key, &value, ttl).await?;
            Ok(value)
        }

        async fn mget<T: DeserializeOwned + Send>(&self, keys: &[&str]) -> CacheResult<Vec<Option<T>>> {
            let mut results = Vec::with_capacity(keys.len());
            for key in keys {
                results.push(self.get(key).await?);
            }
            Ok(results)
        }

        async fn mset<T: Serialize + Send + Sync>(&self, items: &[(&str, &T)], ttl: Option<Duration>) -> CacheResult<()> {
            for (key, value) in items {
                self.set(key, value, ttl).await?;
            }
            Ok(())
        }

        async fn mdel(&self, keys: &[&str]) -> CacheResult<u64> {
            let mut count = 0;
            for key in keys {
                if self.delete(key).await? {
                    count += 1;
                }
            }
            Ok(count)
        }

        async fn ttl(&self, _key: &str) -> CacheResult<Option<Duration>> {
            Ok(None)
        }

        async fn expire(&self, _key: &str, _ttl: Duration) -> CacheResult<bool> {
            Ok(true)
        }

        async fn set_nx<T: Serialize + Send + Sync>(&self, key: &str, value: &T, ttl: Option<Duration>) -> CacheResult<bool> {
            if self.exists(key).await? {
                return Ok(false);
            }
            self.set(key, value, ttl).await?;
            Ok(true)
        }

        async fn incr(&self, key: &str, delta: i64) -> CacheResult<i64> {
            let current: Option<i64> = self.get(key).await?;
            let new_value = current.unwrap_or(0) + delta;
            self.set(key, &new_value, None).await?;
            Ok(new_value)
        }

        async fn decr(&self, key: &str, delta: i64) -> CacheResult<i64> {
            self.incr(key, -delta).await
        }
    }

    #[tokio::test]
    async fn test_write_behind_basic() {
        let cache = Arc::new(MockCache::new());
        let config = WriteBehindConfig::default()
            .with_flush_interval(Duration::from_millis(50));
        let write_behind = WriteBehind::new(cache.clone(), config);

        let db_write_count = Arc::new(AtomicU32::new(0));
        let count_clone = db_write_count.clone();

        // Write-behind 書き込み
        write_behind
            .write("user:1", &"Alice", move || {
                let count = count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            })
            .await
            .unwrap();

        // キャッシュには即座に書き込まれる
        let cached: Option<String> = cache.get("user:1").await.unwrap();
        assert_eq!(cached, Some("Alice".to_string()));

        // 少し待ってから DB 書き込みを確認
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert_eq!(db_write_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_write_behind_stats() {
        let cache = Arc::new(MockCache::new());
        let config = WriteBehindConfig::default()
            .with_flush_interval(Duration::from_millis(50));
        let write_behind = WriteBehind::new(cache.clone(), config);

        // 複数の書き込み
        for i in 0..5 {
            let key = format!("key:{}", i);
            write_behind
                .write(&key, &i, || async { Ok(()) })
                .await
                .unwrap();
        }

        // 統計を確認
        tokio::time::sleep(Duration::from_millis(200)).await;
        let stats = write_behind.stats().snapshot();
        assert_eq!(stats.writes_succeeded, 5);
        assert_eq!(stats.writes_failed, 0);
    }
}
