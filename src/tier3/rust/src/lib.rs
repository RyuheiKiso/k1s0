// k1s0-impl: IMPL-tier3-0001 realizes=FR-tier3-001
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

// 型エイリアス: HLC タイムスタンプ（u64 の monotonic HLC 値）を全モジュールで共有する
pub type HlcTimestamp = u64;

// 型エイリアス: クライアント識別子（UUID 文字列）を全モジュールで共有する
pub type ClientId = String;

// tier3 全体で使用する共通エラー型を定義する（spec 11_クライアント状態適合仕様.md §error_model）
#[derive(Debug)]
pub enum ClientStateError {
    // field_pii annotation が付与されていないフィールドに PII が混入した場合のエラー
    MissingPiiAnnotation { field: String },
    // aggregate 単位の max_one_in_flight 制約違反（concurrency_guard.rs が物理強制する）
    ConcurrencyGuardViolation,
    // purge_event emit 時の整合性エラー（HLC 単調性違反 / tenant_id 不一致等）
    PurgeIntegrityError { message: String },
    // BusinessConflict subtype が未知の場合のエラー（4 subtype 外の subtype は不正）
    UnknownConflictSubtype { subtype: String },
}

// ClientStateError の標準 Display 実装（エラーメッセージの人間可読形式）
impl std::fmt::Display for ClientStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // PII annotation エラーメッセージを整形する
            Self::MissingPiiAnnotation { field } => {
                write!(f, "PII annotation missing for field: {field}")
            }
            // 同時実行違反エラーメッセージを整形する
            Self::ConcurrencyGuardViolation => {
                write!(f, "concurrency guard violation: max_one_in_flight exceeded")
            }
            // purge 整合性エラーメッセージを整形する
            Self::PurgeIntegrityError { message } => {
                write!(f, "purge event integrity error: {message}")
            }
            // 未知 subtype エラーメッセージを整形する
            Self::UnknownConflictSubtype { subtype } => {
                write!(f, "unknown conflict subtype: {subtype}")
            }
        }
    }
}

// ClientStateError を標準 Error trait に適合させる
impl std::error::Error for ClientStateError {}

// よく使用する型を prelude として再エクスポートする（各モジュールの import を簡略化する）
pub mod prelude {
    // HLC タイムスタンプ型を再エクスポートする
    pub use crate::HlcTimestamp;
    // クライアント識別子型を再エクスポートする
    pub use crate::ClientId;
    // 共通エラー型を再エクスポートする
    pub use crate::ClientStateError;
}
