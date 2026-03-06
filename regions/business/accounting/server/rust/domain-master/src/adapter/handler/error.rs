use k1s0_server_common::ServiceError;

/// anyhow::Error から ServiceError への変換ヘルパー。
/// エラーメッセージからドメイン固有のエラーコードにマッピングする。
pub fn from_anyhow(err: anyhow::Error) -> ServiceError {
    let msg = err.to_string();
    let lower = msg.to_ascii_lowercase();

    if lower.contains("not found") {
        if lower.contains("category") {
            return ServiceError::NotFound {
                code: k1s0_server_common::ErrorCode::new("BIZ_DOMAINMASTER_CATEGORY_NOT_FOUND"),
                message: msg,
            };
        }
        if lower.contains("item") {
            return ServiceError::NotFound {
                code: k1s0_server_common::ErrorCode::new("BIZ_DOMAINMASTER_ITEM_NOT_FOUND"),
                message: msg,
            };
        }
        return ServiceError::NotFound {
            code: k1s0_server_common::ErrorCode::new("BIZ_DOMAINMASTER_NOT_FOUND"),
            message: msg,
        };
    }
    if lower.contains("duplicate code") || lower.contains("already exists") {
        return ServiceError::Conflict {
            code: k1s0_server_common::ErrorCode::new("BIZ_DOMAINMASTER_DUPLICATE_CODE"),
            message: msg,
            details: vec![],
        };
    }
    if lower.contains("validation error") {
        return ServiceError::BadRequest {
            code: k1s0_server_common::ErrorCode::new("BIZ_DOMAINMASTER_VALIDATION_ERROR"),
            message: msg,
            details: vec![],
        };
    }

    ServiceError::Internal {
        code: k1s0_server_common::ErrorCode::new("BIZ_DOMAINMASTER_INTERNAL_ERROR"),
        message: msg,
    }
}
