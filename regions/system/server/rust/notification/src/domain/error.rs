//! Notification サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Notification ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    /// 通知が見つからない
    #[error("notification '{0}' not found")]
    NotFound(String),

    /// チャンネルが見つからない
    #[error("channel '{0}' not found")]
    ChannelNotFound(String),

    /// テンプレートが見つからない
    #[error("template '{0}' not found")]
    TemplateNotFound(String),

    /// 既に送信済み
    #[error("already sent: {0}")]
    AlreadySent(String),

    /// チャンネルが無効化されている
    #[error("channel disabled: {0}")]
    ChannelDisabled(String),

    /// 送信に失敗
    #[error("send failed: {0}")]
    SendFailed(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// NotificationError から ServiceError への変換実装
impl From<NotificationError> for ServiceError {
    fn from(err: NotificationError) -> Self {
        match err {
            NotificationError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_NOTIFY_NOT_FOUND"),
                message: msg,
            },
            NotificationError::ChannelNotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_NOTIFY_CHANNEL_NOT_FOUND"),
                message: msg,
            },
            NotificationError::TemplateNotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_NOTIFY_TEMPLATE_NOT_FOUND"),
                message: msg,
            },
            NotificationError::AlreadySent(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_NOTIFY_ALREADY_SENT"),
                message: msg,
                details: vec![],
            },
            NotificationError::ChannelDisabled(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_NOTIFY_CHANNEL_DISABLED"),
                message: msg,
                details: vec![],
            },
            NotificationError::SendFailed(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_NOTIFY_SEND_FAILED"),
                message: msg,
            },
            NotificationError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_NOTIFY_VALIDATION_ERROR"),
                message: msg,
                details: vec![],
            },
            NotificationError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_NOTIFY_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
