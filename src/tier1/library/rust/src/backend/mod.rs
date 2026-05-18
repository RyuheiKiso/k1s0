// mod.rs — k1s0 tier1 Library backend: backend 専用モジュール群
// cache / storage / config / rpc / messaging / schema / db / db_distributed / vector / workflow / rules
// の 11 サブモジュールを提供する。
// 全モジュールは backend 専用（frontend ビルドには含まない）。
// 各モジュールの公開 API は OSS 型を露出しない。

// cache モジュール: KeyValue/Cache L3（backend 専用）
// Cache trait / CacheEntry / CacheGetResult / CacheSetOptions を提供する
pub mod cache;

// storage モジュール: Object Storage L3（backend 専用）
// ObjectStorage trait / ObjectMetadata / UploadOptions / ListResult を提供する
pub mod storage;

// config モジュール: Configuration/Feature Flag L2*（backend 専用）
// ConfigStore trait / FeatureFlagClient trait / ConfigValue / FeatureFlag を提供する
pub mod config;

// rpc モジュール: RPC/Gateway L3（backend 向け）
// RpcClient trait / RpcRequest / RpcResponse / RpcStatus を提供する
pub mod rpc;

// messaging モジュール: Messaging/EventBus L1+（backend 専用）
// MessageProducer trait / MessageConsumer trait / Message / ProduceResult を提供する
pub mod messaging;

// schema モジュール: Schema Registry L2*（backend 専用）
// SchemaRegistry trait / SchemaRecord / SchemaFormat / CompatibilityLevel を提供する
pub mod schema;

// db モジュール: Relational Store/Single-leader L1+（backend 専用）
// Repository<T> / PgRepository<T> / QueryableRepository<T> / QueryPage を提供する
pub mod db;

// db_distributed モジュール: Relational Store/Distributed SQL L1+（backend 専用）
// DistributedSqlRepository<T> / DistributedTransaction / ReadConsistency を提供する
pub mod db_distributed;

// vector モジュール: Vector Search L1+（backend 専用）
// VectorStore trait / VectorPoint / SearchRequest / SearchResult を提供する
pub mod vector;

// workflow モジュール: Workflow/Long-running Saga L1+（backend 専用）
// WorkflowOrchestrator trait / WorkflowInstance / WorkflowStatus / StepResult を提供する
pub mod workflow;

// rules モジュール: Rule Engine L1+（backend 専用）
// RuleEngine trait / FactSet / RuleSetEvaluationResult / RuleMetadata を提供する
pub mod rules;

// 頻繁に使用する型を backend 名前空間から直接参照できるように re-export する
// cache 関連の主要型を re-export する
pub use cache::{Cache, CacheEntry, CacheGetResult, CacheSetOptions};
// storage 関連の主要型を re-export する
pub use storage::{ListResult, ObjectMetadata, ObjectStorage, UploadOptions};
// config 関連の主要型を re-export する
pub use config::{ConfigStore, ConfigValue, EvaluationContext, FeatureFlag, FeatureFlagClient};
// rpc 関連の主要型を re-export する
pub use rpc::{RpcClient, RpcRequest, RpcResponse, RpcStatus};
// messaging 関連の主要型を re-export する
pub use messaging::{
    // プロデューサー関連
    MessageProducer,
    ProduceResult,
    // コンシューマー関連
    ConsumeOptions,
    MessageConsumer,
    // 共通型
    Message,
    MessageHeaders,
};
// schema 関連の主要型を re-export する
pub use schema::{CompatibilityLevel, SchemaFormat, SchemaLookupResult, SchemaRecord, SchemaRegistry};
// db 関連の主要型を re-export する（後方互換 pub use も維持する）
pub use db::{FilterOperator, PgRepository, QueryFilter, QueryPage, QueryableRepository, Repository};
// db_distributed 関連の主要型を re-export する
pub use db_distributed::{
    // トランザクション関連
    DistributedSqlRepository,
    DistributedTransaction,
    // ポリシー関連
    GeoPartitionPolicy,
    IsolationLevel,
    ReadConsistency,
};
// vector 関連の主要型を re-export する
pub use vector::{
    // 主要 trait
    VectorStore,
    // 設定型
    CollectionConfig,
    SimilarityMetric,
    // 操作型
    SearchRequest,
    SearchResult,
    VectorPoint,
};
// workflow 関連の主要型を re-export する
pub use workflow::{
    // 主要 trait
    WorkflowOrchestrator,
    // データ型
    StepResult,
    WorkflowInstance,
    WorkflowSignal,
    WorkflowStatus,
};
// rules 関連の主要型を re-export する
pub use rules::{
    // 主要 trait
    RuleEngine,
    // データ型
    FactSet,
    RuleEvaluationResult,
    RuleMetadata,
    RuleSetEvaluationResult,
};
