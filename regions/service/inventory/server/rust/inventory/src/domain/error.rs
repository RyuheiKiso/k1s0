//! Inventory サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Inventory ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum InventoryError {
    #[error("Inventory item '{0}' not found")]
    NotFound(String),

    #[error("insufficient stock: available={available}, requested={requested}")]
    InsufficientStock { available: i32, requested: i32 },

    #[error("insufficient reserved stock: reserved={reserved}, requested={requested}")]
    InsufficientReserved { reserved: i32, requested: i32 },

    #[error("validation failed: {0}")]
    ValidationFailed(String),

    #[error("version conflict for inventory item '{0}'")]
    VersionConflict(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<InventoryError> for ServiceError {
    fn from(err: InventoryError) -> Self {
        match err {
            InventoryError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SVC_INVENTORY_NOT_FOUND"),
                message: msg,
            },
            InventoryError::InsufficientStock {
                available,
                requested,
            } => ServiceError::BadRequest {
                code: ErrorCode::new("SVC_INVENTORY_INSUFFICIENT_STOCK"),
                message: format!(
                    "insufficient stock: available={}, requested={}",
                    available, requested
                ),
                details: vec![],
            },
            InventoryError::InsufficientReserved {
                reserved,
                requested,
            } => ServiceError::BadRequest {
                code: ErrorCode::new("SVC_INVENTORY_INSUFFICIENT_RESERVED"),
                message: format!(
                    "insufficient reserved stock: reserved={}, requested={}",
                    reserved, requested
                ),
                details: vec![],
            },
            InventoryError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SVC_INVENTORY_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            InventoryError::VersionConflict(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SVC_INVENTORY_VERSION_CONFLICT"),
                message: msg,
                details: vec![],
            },
            InventoryError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SVC_INVENTORY_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
