// db_distributed.rs — k1s0 tier1 Library backend: Relational Store/Distributed SQL L1+ trait
// backend 専用カテゴリ（frontend には提供しない）。
// L1+: OSS の全機能を表現 + tier1 横断要素（auth context 伝播 / retry / tracing）を強制。
// 公開 API に OSS 型（CockroachDB / YugabyteDB 固有型）を露出しない。
// 分散 SQL 特有の概念（分散 transaction / follower_read / geo-partition）を Library 独自型で表現する。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: スナップショット設定のシリアライズに使用する
use serde::{Deserialize, Serialize};
// AuthContext: DB 操作に認証コンテキストを伝播する
use crate::core::auth::AuthContext;
// SpanContext: DB 操作に tracing コンテキストを伝播する
use crate::core::observability::SpanContext;
// ServicePolicy: retry / timeout を強制する
use crate::core::policy::ServicePolicy;
// db モジュールの型を再利用する
use crate::backend::db::{QueryFilter, QueryPage};
// serde のシリアライズ制約
use serde::de::DeserializeOwned;

// ReadConsistency は分散 SQL の読み取り一貫性レベルを表す Library 独自型。
// 分散 DB の Read Concern を Library 独自語彙で分類する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadConsistency {
    // StrongConsistency: 最強一貫性（全ノードで同一の最新値を保証）
    StrongConsistency,
    // FollowerRead: フォロワーノードから読み取る（レイテンシ優先 / わずかなラグを許容）
    FollowerRead,
    // BoundedStaleness: 境界ある古さ（許容する古さを HLC tick 数で指定する）
    BoundedStaleness { max_staleness_ticks: u64 },
}

// GeoPartitionPolicy は geo-partitioning ポリシーを表す Library 独自型。
// テナントデータを特定のリージョンに物理的に配置する規則を定義する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoPartitionPolicy {
    // region: データを配置するリージョン（例: "ap-northeast-1"）
    pub region: String,
    // replicas_per_region: リージョン内レプリカ数
    pub replicas_per_region: u32,
    // enforce_data_residency: データ居住地規制を強制するかどうか（GDPR 等）
    pub enforce_data_residency: bool,
}

// DistributedTransaction は分散 SQL のトランザクションコンテキストを表す struct。
// 分散 transaction の ID とメタデータを保持する（生の DB transaction オブジェクトは露出しない）。
#[derive(Debug, Clone)]
pub struct DistributedTransaction {
    // transaction_id: 分散トランザクションの識別子（UUID v7 形式）
    pub transaction_id: String,
    // tenant_id: このトランザクションが属するテナントの識別子
    pub tenant_id: String,
    // read_consistency: このトランザクション内の読み取り一貫性レベル
    pub read_consistency: ReadConsistency,
    // isolation_level: トランザクション分離レベル（Library 独自語彙）
    pub isolation_level: IsolationLevel,
}

// IsolationLevel はトランザクション分離レベルを表す Library 独自型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationLevel {
    // Serializable: 直列化可能分離レベル（分散 DB の最強分離）
    Serializable,
    // SnapshotIsolation: スナップショット分離（高スループット / ファントム許容）
    SnapshotIsolation,
    // ReadCommitted: コミット済み読み取り分離
    ReadCommitted,
}

// DistributedSqlRepository は Relational Store/Distributed SQL の L1+ 抽象 trait。
// CockroachDB / YugabyteDB / Spanner 等の分散 SQL を実装で切り替えられる。
// 公開 API に OSS 型を露出しない。
#[async_trait]
pub trait DistributedSqlRepository<T>: Send + Sync
where
    // T はシリアライズ / デシリアライズ可能なドメイン型
    T: Serialize + DeserializeOwned + Send + Sync,
{
    // begin は分散トランザクションを開始する。
    // auth_ctx は tenant_id の境界を保証するために使用する。
    async fn begin(
        &self,
        isolation: IsolationLevel,
        consistency: ReadConsistency,
        auth_ctx: &AuthContext,
    ) -> Result<DistributedTransaction>;

    // commit はトランザクションをコミットする。
    async fn commit(&self, tx: DistributedTransaction) -> Result<()>;

    // rollback はトランザクションをロールバックする。
    async fn rollback(&self, tx: DistributedTransaction) -> Result<()>;

    // find_by_id はトランザクション内で id のエンティティを取得する。
    // auth_ctx は tenant_id 境界を保証する。
    // span_ctx は tracing コンテキストの伝播に使用する。
    async fn find_by_id(
        &self,
        id: &str,
        tx: &DistributedTransaction,
        span_ctx: Option<&SpanContext>,
    ) -> Result<Option<T>>;

    // save はトランザクション内でエンティティを保存する（UPSERT セマンティクス）。
    async fn save(
        &self,
        entity: &T,
        tx: &DistributedTransaction,
        span_ctx: Option<&SpanContext>,
    ) -> Result<()>;

    // query はトランザクション内でフィルター条件付きクエリを実行する。
    async fn query(
        &self,
        filters: Vec<QueryFilter>,
        page_size: u64,
        cursor: Option<String>,
        tx: &DistributedTransaction,
        span_ctx: Option<&SpanContext>,
    ) -> Result<QueryPage<T>>;

    // bulk_save は複数エンティティを 1 バッチで保存する（スループット最適化）。
    // auth_ctx は tenant_id 境界を保証する。
    // policy は retry / timeout を適用する。
    async fn bulk_save(
        &self,
        entities: Vec<T>,
        auth_ctx: &AuthContext,
        span_ctx: Option<&SpanContext>,
        policy: &ServicePolicy,
    ) -> Result<u64>;
}
