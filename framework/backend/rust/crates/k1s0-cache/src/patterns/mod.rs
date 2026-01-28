//! キャッシュパターン
//!
//! 一般的なキャッシュパターンの実装を提供する。
//!
//! ## サポートされているパターン
//!
//! - **Cache-Aside**: アプリケーションがキャッシュとDBの両方を管理
//! - **Write-Through**: キャッシュとDBに同時書き込み
//! - **Write-Behind**: キャッシュに書き込み後、非同期でDBに反映
//! - **TTL Refresh**: アクセス時にTTLを更新

pub mod cache_aside;
pub mod ttl_refresh;
pub mod write_behind;
pub mod write_through;

pub use cache_aside::{CacheAside, CacheAsideConfig, CacheAsideLoader};
pub use ttl_refresh::{TtlRefresh, TtlRefreshConfig};
pub use write_behind::{WriteBehind, WriteBehindConfig, WriteBehindStats, WriteBehindStatsSnapshot};
pub use write_through::{WriteThrough, WriteThroughConfig, WriteThroughStore};
