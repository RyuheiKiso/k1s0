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
// i18n formatter トレイト（ICU 風 number / date / currency / unit）
pub mod i18n_formatter;
// PII フィールド redaction 機構（spec 10 §PII redact）
pub mod redaction;
// business_conflict detector（4 subtype: stale_write / lost_update / supersede / concurrent_edit）
pub mod business_conflict;
// audit hash chain モジュール（SHA-256 hash chain 整合性検証）
#[path = "../../audit/hash_chain.rs"]
pub mod hash_chain;
// audit relay モジュール（audit_local → ClickHouse 転送エンジン）
#[path = "../../audit/relay.rs"]
pub mod relay;

// 公開型の再エクスポート（各行コメント: tier2 公開 API 表面を最小化する）
pub use tenant_context::{TenantContext, SessionPurpose};
pub use atomic_triple_write::{AtomicTripleWrite, TripleWriteResult, AtomicWriteError};
pub use repository::{RepositoryContext, TableClass};
pub use outbox::{OutboxEntry, OutboxPayload};
// redaction の公開関数を再エクスポートする（Outbox 書込前の PII 除去に使用する）
pub use redaction::redact_pii_fields;
