// k1s0 tier2 ライブラリクレートのルート
// テナント分離 Repository abstraction + atomic 三表書込を提供する

// テナントコンテキスト（4 GUC 自動注入）モジュール
pub mod tenant_context;
// atomic 三表書込（P1-P4 invariant）モジュール
pub mod atomic_triple_write;
// Repository abstraction（生 SQL 封鎖）モジュール
pub mod repository;
// Outbox relay モジュール
pub mod outbox;

// 公開型の再エクスポート（各行コメント: tier2 公開 API 表面を最小化する）
pub use tenant_context::{TenantContext, SessionPurpose};
pub use atomic_triple_write::{AtomicTripleWrite, TripleWriteResult, AtomicWriteError};
pub use repository::{RepositoryContext, TableClass};
pub use outbox::{OutboxEntry, OutboxPayload};
