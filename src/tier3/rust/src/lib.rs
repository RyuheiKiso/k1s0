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
