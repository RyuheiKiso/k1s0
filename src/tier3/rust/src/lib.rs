// k1s0 tier3 Rust crate エントリポイント
// 4 layer client state reducer の Rust 等価強度実装

// state モジュールを公開する
pub mod state;
// 添付ファイルストア trait モジュールを公開する
pub mod attachments;
// outbox モジュールを公開する（HLC ベース TTL / PII strip / Idempotency-Key 生成）
pub mod outbox;
// R3-5: purge モジュールを公開する（client_purge_event emit / PurgeTrigger 型）
pub mod purge;
// Y-tier3-3: concurrency_guard モジュールを公開する（aggregate 単位 max_one_in_flight 強制）
pub mod concurrency_guard;
// Y-tier3-4: pii_annotation モジュールを公開する（field_pii annotation compile-time 強制）
pub mod pii_annotation;
