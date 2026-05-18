// lib.rs — k1s0 tier1 Library Rust エントリーポイント
// 17 機能カテゴリ × L3/L2*/L1+ 階層の facade モジュールを提供する。
// core（auth / observability / profiling / secret / error / policy）/
// backend（cache / storage / config / rpc / messaging / schema / db / db_distributed / vector / workflow / rules）/
// frontend（auth / config / rpc）の 3 サブ名前空間で構成する。
// 後方互換のため既存 3 モジュールを pub use で再エクスポートする。

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
pub mod frontend;

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
