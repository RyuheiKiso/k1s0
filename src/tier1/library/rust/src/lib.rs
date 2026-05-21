// lib.rs — k1s0 tier1 Library Rust エントリーポイント
// 17 機能カテゴリ × L3/L2*/L1+ 階層の facade モジュールを提供する。
// core（auth / observability / profiling / secret / error / policy）/
// backend（cache / storage / config / rpc / messaging / schema / db / db_distributed / vector / workflow / rules）/
// frontend（auth / config / rpc）の 3 サブ名前空間で構成する。
// 後方互換のため既存 3 モジュールを pub use で再エクスポートする。
// フラット 14 モジュール（cache / config / db / db_distributed / messaging /
// observability / profiling / rpc / rules / schema / secret / storage / vector / workflow）
// を crate ルートで直接公開する（4 言語等価強度 API 規約準拠）。

// ---- crate 共通 Result 型 ----

// Result<T> は全 facade trait が使用する共通エラー型エイリアス。
// Box<dyn std::error::Error + Send + Sync> によりエラー型の具体化を実装層に委ねる。
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// ---- フラット 14 モジュール（crate ルート直接公開） ----

// cache モジュール: キャッシュ操作の L2* facade trait（Cache）
// Valkey/Redis を L2* ラップする。wall-clock TTL 禁止（HLC ミリ秒使用）。
pub mod cache;

// config モジュール: フィーチャーフラグ操作の L2* facade trait（FlagdClient）
// flagd/OpenFeature を L2* ラップする。
pub mod config;

// db モジュール: 単一ノード PostgreSQL 操作の L1+ facade trait（Db / DbTransaction）
// sqlx を L1+ ラップする。tenant_id は GUC 経由で注入済み。
pub mod db;

// db_distributed モジュール: 分散 PostgreSQL 互換 DB の L1+ facade trait（DistributedDb）
// CockroachDB を L1+ ラップする。begin_serializable のみを提供する。
pub mod db_distributed;

// messaging モジュール: メッセージング L1+ facade trait（MessagingProducer / MessagingConsumer）
// Kafka を L1+ ラップする。タイムスタンプは HLC 形式文字列。
pub mod messaging;

// observability モジュール: 可観測性 L1+ facade trait（Tracer / Span / MetricRecorder / LogEmitter）
// OTel SDK を L1+ ラップする。5 signal class（Trace / Metric / Log / Profile / CP）対応。
pub mod observability;

// profiling モジュール: プロファイリング L1+ facade trait（Profiler / ProfilingSession）
// Pyroscope を L1+ ラップする。
pub mod profiling;

// rpc モジュール: RPC L1+ facade trait（RpcClient / RpcServer / RpcHandler）
// tonic を L1+ ラップする。
pub mod rpc;

// rules モジュール: ルールエンジン L1+ facade trait（RuleEngine）
// ZEN engine を L1+ ラップする。入出力は JSON bytes。
pub mod rules;

// schema モジュール: スキーマレジストリ L1+ facade trait（SchemaRegistry）
// Apicurio Registry を L1+ ラップする。artifact_id ベースで管理する。
pub mod schema;

// secret モジュール: シークレット管理 L1+ facade trait（SecretStore / SecretHandle）
// OpenBao を L1+ ラップする。生シークレット露出禁止（SecretHandle による opaque ラップ）。
pub mod secret;

// storage モジュール: オブジェクトストレージ L1+ facade trait（ObjectStorage）
// S3 互換 API を L1+ ラップする。put / get / delete の 3 操作を提供する。
pub mod storage;

// vector モジュール: ベクトル検索 L1+ facade trait（VectorSearch）
// pgvector を L1+ ラップする。cosine 距離による k-NN 検索。
pub mod vector;

// workflow モジュール: ワークフローエンジン L1+ facade trait（WorkflowClient）
// Temporal を L1+ ラップする。タイムアウトは HLC ミリ秒（wall-clock 禁止）。
pub mod workflow;

// ---- サブ名前空間モジュール ----

// core モジュール: 横断的・基盤モジュール群（both 向け）
// AuthContext / AuthClass / KeyHandle / LibraryError / ServicePolicy 等を提供する
pub mod core;

// backend モジュール: backend 専用モジュール群
// Cache / ObjectStorage / ConfigStore / RpcClient / MessageProducer /
// SchemaRegistry / Repository / DistributedSqlRepository / VectorStore /
// WorkflowOrchestrator / RuleEngine 等を提供する
pub mod backend;

// frontend モジュール: frontend 向け thin wrapper モジュール群
// FrontendAuthClient / FrontendConfigClient / FrontendRpcClient 等を提供する
// TransportNegotiationClient / BidiChannel / ClientCapabilities 等も提供する
pub mod frontend;

// ---- conformance assertion id デコレータ ----

// conformance_assert モジュール: spec 01 §assertion id の連結
// ConformanceAssertId 型 + conformance_assert! マクロを提供する。
// CI 整合 2「scenarios.yaml の assertion id が全言語で実装されていること」の Rust 物理機構。
pub mod conformance_assert;

// ---- 後方互換: 既存 3 モジュールを維持する ----

// key_handle モジュール: KeyClass enum + KeyHandle trait + OpenBaoKeyHandle struct
// core::secret に統合済み; 後方互換のため pub として残す
pub mod key_handle;

// auth_context モジュール: AuthClass enum + AuthContext struct
// core::auth に統合済み; 後方互換のため pub として残す
pub mod auth_context;

// repository モジュール: Repository<T> trait + PgRepository<T> 実装
// backend::db に統合済み; 後方互換のため pub として残す
pub mod repository;
