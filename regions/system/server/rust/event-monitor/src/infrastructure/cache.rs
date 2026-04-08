use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::entity::flow_definition::FlowDefinition;
use crate::domain::entity::flow_kpi::FlowKpi;

/// KPI 集計結果のインメモリキャッシュ。
/// 高頻度アクセスされる KPI データの DB 負荷を軽減する。
pub struct KpiCache {
    inner: Cache<String, Arc<FlowKpi>>,
}

impl KpiCache {
    #[must_use] 
    pub fn new(max_capacity: u64, ttl_secs: u64) -> Self {
        let inner = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();
        Self { inner }
    }

    pub async fn get(&self, key: &str) -> Option<Arc<FlowKpi>> {
        self.inner.get(key).await
    }

    pub async fn insert(&self, key: String, kpi: Arc<FlowKpi>) {
        self.inner.insert(key, kpi).await;
    }

    #[allow(dead_code)]
    pub async fn invalidate(&self, key: &str) {
        self.inner.invalidate(key).await;
    }

    #[allow(dead_code)]
    pub async fn invalidate_all(&self) {
        self.inner.invalidate_all();
        self.inner.run_pending_tasks().await;
    }
}

/// フロー定義のインメモリキャッシュ。
/// Kafka メッセージ処理ごとの `find_all()` による N+1 クエリを防止する。
/// TTL 経過後に自動的にキャッシュが無効化され、次のアクセスで再取得する。
pub struct FlowDefinitionCache {
    /// "`all_flows`" キーで全フロー定義リストをキャッシュする
    inner: Cache<String, Arc<Vec<FlowDefinition>>>,
}

/// フロー定義キャッシュの全件取得に使用する固定キー
const ALL_FLOWS_KEY: &str = "all_flows";

impl FlowDefinitionCache {
    /// 新しいフロー定義キャッシュを生成する。
    /// `max_capacity`: キャッシュエントリの最大数
    /// `ttl_secs`: エントリの有効期限（秒）
    #[must_use] 
    pub fn new(max_capacity: u64, ttl_secs: u64) -> Self {
        let inner = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();
        Self { inner }
    }

    /// キャッシュから全フロー定義を取得する。キャッシュミスの場合は None を返す。
    pub async fn get_all(&self) -> Option<Arc<Vec<FlowDefinition>>> {
        self.inner.get(ALL_FLOWS_KEY).await
    }

    /// 全フロー定義をキャッシュに格納する。
    pub async fn set_all(&self, flows: Vec<FlowDefinition>) {
        self.inner
            .insert(ALL_FLOWS_KEY.to_string(), Arc::new(flows))
            .await;
    }

    /// フロー定義の変更時にキャッシュを無効化する。
    /// create/update/delete 操作後に呼び出すことで、次回アクセス時に最新データを取得する。
    #[allow(dead_code)]
    pub async fn invalidate(&self) {
        self.inner.invalidate(ALL_FLOWS_KEY).await;
    }
}
