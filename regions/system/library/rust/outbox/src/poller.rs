use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::time;
use uuid::Uuid;

use crate::event::OutboxEvent;

/// OutboxEventSource はアウトボックスイベントの取得元インターフェース。
///
/// 各サービスのリポジトリトレイトがこのトレイトを実装する。
/// fetch と mark を分離することで、at-least-once（最低1回）配信を実現する。
#[async_trait]
pub trait OutboxEventSource: Send + Sync {
    /// 未パブリッシュのイベントを取得する（mark は行わない）。
    /// FOR UPDATE SKIP LOCKED により並行ポーラー間の排他を保証する。
    async fn fetch_unpublished_events(&self, limit: i64) -> anyhow::Result<Vec<OutboxEvent>>;

    /// 指定した ID のイベントをパブリッシュ済みとしてマークする。
    /// publish 成功後のみ呼び出すことで at-least-once セマンティクスを実現する。
    async fn mark_events_published(&self, ids: &[Uuid]) -> anyhow::Result<()>;
}

/// OutboxEventHandler はイベント種別ごとの変換・パブリッシュロジックを抽象化する。
///
/// 各サービスが自身の Protobuf 型へ変換し Kafka へ publish するロジックを実装する。
#[async_trait]
pub trait OutboxEventHandler: Send + Sync {
    /// イベントを処理する。変換+パブリッシュを行い、結果を返す。
    /// 未知のイベント種別の場合は Ok(false) を返してスキップする。
    async fn handle_event(&self, event: &OutboxEvent) -> anyhow::Result<bool>;
}

/// OutboxEventFetcher はリポジトリ層の outbox 操作を抽象化するトレイト。
///
/// 各サービスのリポジトリトレイトには他のメソッド（find_by_id 等）も含まれるため、
/// OutboxEventSource を直接実装するのは困難。
/// このトレイトを使えば、リポジトリを OutboxEventSource として簡単にラップできる。
#[async_trait]
pub trait OutboxEventFetcher: Send + Sync {
    /// 未パブリッシュのイベントを取得する（mark は行わない）。
    async fn fetch_unpublished_events(&self, limit: i64) -> anyhow::Result<Vec<OutboxEvent>>;

    /// 指定した ID のイベントをパブリッシュ済みとしてマークする。
    async fn mark_events_published(&self, ids: &[Uuid]) -> anyhow::Result<()>;
}

/// RepositoryOutboxSource は OutboxEventFetcher を OutboxEventSource にアダプトする。
///
/// 各サービスで重複していた *OutboxSource 構造体と OutboxEventSource impl を共通化する。
/// リポジトリが OutboxEventFetcher を実装していれば、このアダプタで OutboxEventSource に変換できる。
/// dyn Trait を受け取れるよう ?Sized バウンドを指定。
pub struct RepositoryOutboxSource<R: OutboxEventFetcher + ?Sized> {
    /// イベント取得元リポジトリ
    repo: Arc<R>,
}

impl<R: OutboxEventFetcher + ?Sized> RepositoryOutboxSource<R> {
    /// 新しい RepositoryOutboxSource を生成する。
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R: OutboxEventFetcher + ?Sized + 'static> OutboxEventSource for RepositoryOutboxSource<R> {
    /// リポジトリの fetch_unpublished_events に委譲する
    async fn fetch_unpublished_events(&self, limit: i64) -> anyhow::Result<Vec<OutboxEvent>> {
        self.repo.fetch_unpublished_events(limit).await
    }

    /// リポジトリの mark_events_published に委譲する
    async fn mark_events_published(&self, ids: &[Uuid]) -> anyhow::Result<()> {
        self.repo.mark_events_published(ids).await
    }
}

/// OutboxEventPoller のファクトリ関数。
/// OutboxEventFetcher を実装するリポジトリから簡単にポーラーを構築する。
/// dyn Trait を受け取れるよう ?Sized バウンドを指定。
pub fn new_poller<R: OutboxEventFetcher + ?Sized + 'static>(
    repo: Arc<R>,
    handler: Arc<dyn OutboxEventHandler>,
    poll_interval: Duration,
    batch_size: i64,
) -> OutboxEventPoller {
    let source = Arc::new(RepositoryOutboxSource::new(repo));
    OutboxEventPoller::new(source, handler, poll_interval, batch_size)
}

/// OutboxEventPoller はアウトボックスイベントの汎用ポーリングエンジン。
///
/// 各サービスで重複していた run ループ + poll_and_publish ロジックを共通化する。
/// イベントの取得は OutboxEventSource、変換・パブリッシュは OutboxEventHandler に委譲する。
pub struct OutboxEventPoller {
    /// イベント取得元（リポジトリ）
    source: Arc<dyn OutboxEventSource>,
    /// イベント変換・パブリッシュハンドラ
    handler: Arc<dyn OutboxEventHandler>,
    /// ポーリング間隔
    poll_interval: Duration,
    /// 1回のポーリングで取得するイベント数
    batch_size: i64,
}

impl OutboxEventPoller {
    /// 新しい OutboxEventPoller を生成する。
    pub fn new(
        source: Arc<dyn OutboxEventSource>,
        handler: Arc<dyn OutboxEventHandler>,
        poll_interval: Duration,
        batch_size: i64,
    ) -> Self {
        Self {
            source,
            handler,
            poll_interval,
            batch_size,
        }
    }

    /// バックグラウンドタスクとしてポーリングを開始する。
    /// shutdown_rx でシャットダウンシグナルを受信したら停止する。
    pub async fn run(&self, mut shutdown_rx: tokio::sync::watch::Receiver<bool>) {
        tracing::info!(
            poll_interval_ms = self.poll_interval.as_millis() as u64,
            batch_size = self.batch_size,
            "outbox poller started"
        );

        let mut interval = time::interval(self.poll_interval);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(err) = self.poll_and_publish().await {
                        tracing::error!(error = %err, "outbox poller failed to process events");
                    }
                }
                _ = shutdown_rx.changed() => {
                    tracing::info!("outbox poller shutting down");
                    break;
                }
            }
        }
    }

    /// 未パブリッシュイベントを取得し、ハンドラに委譲してパブリッシュする。
    ///
    /// # At-Least-Once 配信セマンティクス
    ///
    /// このメソッドは at-least-once（最低1回）配信を採用している。
    /// イベントはディスパッチ（Kafka publish）の **成功後に** パブリッシュ済みとしてマークされる。
    /// これにより、以下の動作となる：
    ///
    /// - ディスパッチ成功時：イベントはマークされ、次回ポーリングでスキップされる
    /// - ディスパッチ失敗時：イベントはマークされず、次回ポーリングでリトライされる
    ///
    /// ## トレードオフ
    ///
    /// - メリット：Kafka 障害時にイベントが失われない（少なくとも1回配信）
    /// - デメリット：重複イベントが発生する可能性がある（コンシューマー側での冪等性が必要）
    ///
    /// コンシューマーは outbox の `id` または `idempotency_key` を使って重複排除すること。
    async fn poll_and_publish(&self) -> anyhow::Result<()> {
        // fetch のみ（mark はここでは行わない）
        let events = self
            .source
            .fetch_unpublished_events(self.batch_size)
            .await?;

        if events.is_empty() {
            return Ok(());
        }

        tracing::debug!(count = events.len(), "processing outbox events");

        let mut published_ids: Vec<Uuid> = Vec::new();

        for event in &events {
            // ハンドラにイベント変換・パブリッシュを委譲する
            match self.handler.handle_event(event).await {
                Ok(true) => {
                    tracing::debug!(
                        event_id = %event.id,
                        event_type = %event.event_type,
                        "outbox event published successfully"
                    );
                    // publish 成功のみ mark 対象に追加する
                    published_ids.push(event.id);
                }
                Ok(false) => {
                    // ハンドラが処理をスキップした場合（未知のイベント種別など）
                    // スキップしたイベントも mark して再処理しないようにする
                    tracing::warn!(
                        event_type = %event.event_type,
                        event_id = %event.id,
                        "unknown outbox event type, skipping"
                    );
                    published_ids.push(event.id);
                }
                Err(err) => {
                    // publish 失敗時：mark しないことで次回ポーリングにてリトライされる。
                    // これは at-least-once 保証のためのトレードオフ（重複配信の可能性）。
                    tracing::warn!(
                        error = %err,
                        event_id = %event.id,
                        event_type = %event.event_type,
                        "failed to publish outbox event, will retry on next poll"
                    );
                }
            }
        }

        // publish 成功したイベントのみを一括で mark する
        if !published_ids.is_empty() {
            self.source.mark_events_published(&published_ids).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    /// テスト用のイベントソース（常に空リストを返す）
    struct EmptySource;

    #[async_trait]
    impl OutboxEventSource for EmptySource {
        async fn fetch_unpublished_events(&self, _limit: i64) -> anyhow::Result<Vec<OutboxEvent>> {
            Ok(vec![])
        }

        async fn mark_events_published(&self, _ids: &[Uuid]) -> anyhow::Result<()> {
            Ok(())
        }
    }

    /// テスト用のイベントソース（指定されたイベントを返す）
    struct FixedSource {
        events: Vec<OutboxEvent>,
    }

    #[async_trait]
    impl OutboxEventSource for FixedSource {
        async fn fetch_unpublished_events(&self, _limit: i64) -> anyhow::Result<Vec<OutboxEvent>> {
            Ok(self.events.clone())
        }

        async fn mark_events_published(&self, _ids: &[Uuid]) -> anyhow::Result<()> {
            Ok(())
        }
    }

    /// テスト用のイベントハンドラ（常に成功を返す）
    struct AlwaysSuccessHandler;

    #[async_trait]
    impl OutboxEventHandler for AlwaysSuccessHandler {
        async fn handle_event(&self, _event: &OutboxEvent) -> anyhow::Result<bool> {
            Ok(true)
        }
    }

    /// テスト用のイベントハンドラ（常にスキップを返す）
    struct AlwaysSkipHandler;

    #[async_trait]
    impl OutboxEventHandler for AlwaysSkipHandler {
        async fn handle_event(&self, _event: &OutboxEvent) -> anyhow::Result<bool> {
            Ok(false)
        }
    }

    /// テスト用のイベントハンドラ（常にエラーを返す）
    struct AlwaysFailHandler;

    #[async_trait]
    impl OutboxEventHandler for AlwaysFailHandler {
        async fn handle_event(&self, _event: &OutboxEvent) -> anyhow::Result<bool> {
            Err(anyhow::anyhow!("kafka unavailable"))
        }
    }

    /// サンプルの OutboxEvent を生成するヘルパー関数
    fn sample_event(event_type: &str) -> OutboxEvent {
        OutboxEvent {
            id: Uuid::new_v4(),
            aggregate_type: "test".to_string(),
            aggregate_id: Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            payload: serde_json::json!({"test": true}),
            created_at: Utc::now(),
            published_at: None,
        }
    }

    // 空のイベントソースの場合、poll_and_publish が正常終了することを確認する。
    #[tokio::test]
    async fn test_poll_and_publish_empty() {
        let poller = OutboxEventPoller::new(
            Arc::new(EmptySource),
            Arc::new(AlwaysSuccessHandler),
            Duration::from_secs(1),
            10,
        );
        let result = poller.poll_and_publish().await;
        assert!(result.is_ok());
    }

    // イベントが正常にパブリッシュされる場合を確認する。
    #[tokio::test]
    async fn test_poll_and_publish_success() {
        let source = FixedSource {
            events: vec![sample_event("test.created")],
        };
        let poller = OutboxEventPoller::new(
            Arc::new(source),
            Arc::new(AlwaysSuccessHandler),
            Duration::from_secs(1),
            10,
        );
        let result = poller.poll_and_publish().await;
        assert!(result.is_ok());
    }

    // ハンドラがスキップした場合でもポーラー自体はエラーにならないことを確認する。
    #[tokio::test]
    async fn test_poll_and_publish_skip() {
        let source = FixedSource {
            events: vec![sample_event("unknown.type")],
        };
        let poller = OutboxEventPoller::new(
            Arc::new(source),
            Arc::new(AlwaysSkipHandler),
            Duration::from_secs(1),
            10,
        );
        let result = poller.poll_and_publish().await;
        assert!(result.is_ok());
    }

    // パブリッシュ失敗時もポーラー自体はエラーにならないことを確認する。
    // at-least-once: 失敗したイベントは mark されず、次回ポーリングでリトライされる。
    #[tokio::test]
    async fn test_poll_and_publish_failure_does_not_fail_poller() {
        let source = FixedSource {
            events: vec![sample_event("test.created")],
        };
        let poller = OutboxEventPoller::new(
            Arc::new(source),
            Arc::new(AlwaysFailHandler),
            Duration::from_secs(1),
            10,
        );
        let result = poller.poll_and_publish().await;
        assert!(result.is_ok());
    }

    // シャットダウンシグナルを受信すると run が正常終了することを確認する。
    #[tokio::test]
    async fn test_run_shutdown() {
        let poller = Arc::new(OutboxEventPoller::new(
            Arc::new(EmptySource),
            Arc::new(AlwaysSuccessHandler),
            Duration::from_millis(50),
            10,
        ));

        let poller_clone = poller.clone();
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let handle = tokio::spawn(async move {
            poller_clone.run(shutdown_rx).await;
        });

        // 少なくとも1回ポーリングさせる
        tokio::time::sleep(Duration::from_millis(100)).await;

        // シャットダウンシグナルを送信
        shutdown_tx.send(true).unwrap();
        handle.await.unwrap();
    }

    /// テスト用の OutboxEventFetcher 実装（常に空リストを返す）
    struct EmptyFetcher;

    #[async_trait]
    impl OutboxEventFetcher for EmptyFetcher {
        async fn fetch_unpublished_events(&self, _limit: i64) -> anyhow::Result<Vec<OutboxEvent>> {
            Ok(vec![])
        }

        async fn mark_events_published(&self, _ids: &[Uuid]) -> anyhow::Result<()> {
            Ok(())
        }
    }

    /// テスト用の OutboxEventFetcher 実装（指定されたイベントを返す）
    struct FixedFetcher {
        events: Vec<OutboxEvent>,
    }

    #[async_trait]
    impl OutboxEventFetcher for FixedFetcher {
        async fn fetch_unpublished_events(&self, _limit: i64) -> anyhow::Result<Vec<OutboxEvent>> {
            Ok(self.events.clone())
        }

        async fn mark_events_published(&self, _ids: &[Uuid]) -> anyhow::Result<()> {
            Ok(())
        }
    }

    // RepositoryOutboxSource が OutboxEventFetcher を通じてイベントを取得できることを確認する
    #[tokio::test]
    async fn test_repository_outbox_source_empty() {
        let source = RepositoryOutboxSource::new(Arc::new(EmptyFetcher));
        let events = source.fetch_unpublished_events(10).await.unwrap();
        assert!(events.is_empty());
    }

    // RepositoryOutboxSource がイベントを正しく取得できることを確認する
    #[tokio::test]
    async fn test_repository_outbox_source_with_events() {
        let fetcher = FixedFetcher {
            events: vec![sample_event("test.created")],
        };
        let source = RepositoryOutboxSource::new(Arc::new(fetcher));
        let events = source.fetch_unpublished_events(10).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "test.created");
    }

    // new_poller ファクトリ関数でポーラーが正常に構築されることを確認する
    #[tokio::test]
    async fn test_new_poller_factory() {
        let poller = Arc::new(new_poller(
            Arc::new(EmptyFetcher),
            Arc::new(AlwaysSuccessHandler),
            Duration::from_millis(50),
            10,
        ));

        let poller_clone = poller.clone();
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let handle = tokio::spawn(async move {
            poller_clone.run(shutdown_rx).await;
        });

        // 少なくとも1回ポーリングさせる
        tokio::time::sleep(Duration::from_millis(100)).await;

        // シャットダウンシグナルを送信
        shutdown_tx.send(true).unwrap();
        handle.await.unwrap();
    }

    // new_poller で作ったポーラーがイベントを正常に処理することを確認する
    #[tokio::test]
    async fn test_new_poller_with_events() {
        let fetcher = FixedFetcher {
            events: vec![sample_event("test.created")],
        };
        let poller = Arc::new(new_poller(
            Arc::new(fetcher),
            Arc::new(AlwaysSuccessHandler),
            Duration::from_millis(50),
            10,
        ));

        let poller_clone = poller.clone();
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let handle = tokio::spawn(async move {
            poller_clone.run(shutdown_rx).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        shutdown_tx.send(true).unwrap();
        handle.await.unwrap();
    }
}
