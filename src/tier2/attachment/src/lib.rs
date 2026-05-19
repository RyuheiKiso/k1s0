// k1s0-tier2-attachment クレートのルートモジュール（設計方針 28 / 業務添付帳票資産）
// AttachmentStore トレイトとエンベロープ暗号化を公開 API として提供する

// 添付ファイルストアモジュール（AttachmentStore トレイト / ObjectAttachmentStore 実装）
pub mod attachment_store;
// エンベロープ暗号化モジュール（EnvelopeEncryptor / CiphertextEnvelope）
pub mod envelope_encryption;

// 公開型の再エクスポート（tier2-attachment 公開 API 表面を最小化する）
// AttachmentStore トレイトと AttachmentMetadata を公開する
pub use attachment_store::{AttachmentMetadata, AttachmentStore, ObjectAttachmentStore, ObjectStorageConfig};
// EnvelopeEncryptor と CiphertextEnvelope を公開する（生 DEK は含まない）
pub use envelope_encryption::{CiphertextEnvelope, EnvelopeEncryptor};
