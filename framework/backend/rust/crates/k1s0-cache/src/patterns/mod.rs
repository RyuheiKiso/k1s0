//! キャッシュパターン
//!
//! 一般的なキャッシュパターンの実装を提供する。

pub mod cache_aside;
pub mod ttl_refresh;

pub use cache_aside::{CacheAside, CacheAsideConfig};
pub use ttl_refresh::{TtlRefresh, TtlRefreshConfig};
