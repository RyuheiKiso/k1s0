#![allow(clippy::unwrap_used)]
// GraphQL Gateway のスキーマレベルテスト。
// ドメインモデルの構築、カーソルのエンコード・デコード、
// エラー型の変換を検証する。

use k1s0_graphql_gateway_server::domain::error::GraphqlGatewayError;
use k1s0_graphql_gateway_server::domain::model::{
    decode_cursor, encode_cursor, ConfigEntry, FeatureFlag, PageInfo, Session, SessionStatus,
    Tenant, TenantConnection, TenantEdge, TenantStatus,
};
use k1s0_server_common::error::ServiceError;

// --- カーソルエンコード・デコードテスト ---

#[test]
fn test_cursor_encode_decode_roundtrip() {
    // エンコードしたカーソルを正しくデコードできることを検証
    for offset in [0, 1, 10, 100, 9999] {
        let cursor = encode_cursor(offset);
        let decoded = decode_cursor(&cursor);
        assert_eq!(decoded, Some(offset), "offset={offset} の往復変換に失敗");
    }
}

#[test]
fn test_cursor_decode_invalid_base64() {
    // 不正な base64 文字列に対して None を返すことを検証
    let result = decode_cursor("not-valid-base64!!!");
    assert!(result.is_none());
}

#[test]
fn test_cursor_decode_invalid_prefix() {
    // base64 は有効だが "cursor:" プレフィックスがない場合に None を返すことを検証
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode("invalid:42");
    let result = decode_cursor(&encoded);
    assert!(result.is_none());
}

#[test]
fn test_cursor_decode_non_numeric() {
    // "cursor:abc" のようにオフセットが数値でない場合に None を返すことを検証
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode("cursor:abc");
    let result = decode_cursor(&encoded);
    assert!(result.is_none());
}

// --- TenantStatus 変換テスト ---

#[test]
fn test_tenant_status_from_string() {
    // 既知のステータス文字列が正しく変換されることを検証
    assert_eq!(
        TenantStatus::from("ACTIVE".to_string()),
        TenantStatus::Active
    );
    assert_eq!(
        TenantStatus::from("SUSPENDED".to_string()),
        TenantStatus::Suspended
    );
    assert_eq!(
        TenantStatus::from("DELETED".to_string()),
        TenantStatus::Deleted
    );
}

#[test]
fn test_tenant_status_unknown_defaults_to_active() {
    // 未知のステータス文字列はデフォルトで Active になることを検証
    assert_eq!(
        TenantStatus::from("UNKNOWN".to_string()),
        TenantStatus::Active
    );
    assert_eq!(TenantStatus::from("".to_string()), TenantStatus::Active);
}

// --- ドメインモデル構築テスト ---

#[test]
fn test_tenant_construction() {
    // Tenant ドメインモデルが正しく構築できることを検証
    let tenant = Tenant {
        id: "t-001".to_string(),
        name: "Test Tenant".to_string(),
        status: TenantStatus::Active,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };
    assert_eq!(tenant.id, "t-001");
    assert_eq!(tenant.status, TenantStatus::Active);
}

#[test]
fn test_tenant_connection_construction() {
    // TenantConnection（Relay-style ページネーション）が正しく構築できることを検証
    let connection = TenantConnection {
        edges: vec![TenantEdge {
            node: Tenant {
                id: "t-001".to_string(),
                name: "Tenant 1".to_string(),
                status: TenantStatus::Active,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            },
            cursor: encode_cursor(0),
        }],
        page_info: PageInfo {
            has_next_page: false,
            has_previous_page: false,
            start_cursor: Some(encode_cursor(0)),
            end_cursor: Some(encode_cursor(0)),
        },
        total_count: 1,
    };
    assert_eq!(connection.edges.len(), 1);
    assert_eq!(connection.total_count, 1);
    assert!(!connection.page_info.has_next_page);
}

#[test]
fn test_config_entry_construction() {
    // ConfigEntry ドメインモデルが正しく構築できることを検証
    let entry = ConfigEntry {
        key: "app/feature.enabled".to_string(),
        value: "true".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };
    assert_eq!(entry.key, "app/feature.enabled");
    assert_eq!(entry.value, "true");
}

#[test]
fn test_feature_flag_construction() {
    // FeatureFlag ドメインモデルが正しく構築できることを検証
    let flag = FeatureFlag {
        key: "dark-mode".to_string(),
        name: "Dark mode toggle".to_string(),
        enabled: true,
        rollout_percentage: 100,
        target_environments: vec!["production".to_string()],
    };
    assert_eq!(flag.key, "dark-mode");
    assert!(flag.enabled);
}

#[test]
fn test_session_status_construction() {
    // Session モデルが正しく構築できることを検証
    let session = Session {
        session_id: "sess-001".to_string(),
        user_id: "user-001".to_string(),
        device_id: "device-001".to_string(),
        device_name: Some("Test Device".to_string()),
        device_type: Some("desktop".to_string()),
        user_agent: Some("test-agent".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
        status: SessionStatus::Active,
        expires_at: "2024-01-02T00:00:00Z".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        last_accessed_at: Some("2024-01-01T12:00:00Z".to_string()),
    };
    assert_eq!(session.session_id, "sess-001");
}

// --- エラー型変換テスト ---

#[test]
fn test_graphql_gateway_error_not_found_to_service_error() {
    // NotFound エラーが ServiceError::NotFound に変換されることを検証
    let err = GraphqlGatewayError::NotFound("test schema".to_string());
    let service_err: ServiceError = err.into();
    match service_err {
        ServiceError::NotFound { code, message } => {
            assert_eq!(code.as_str(), "SYS_GQLGW_NOT_FOUND");
            assert!(message.contains("test schema"));
        }
        _ => panic!("NotFound エラーが期待と異なる型に変換された"),
    }
}

#[test]
fn test_graphql_gateway_error_query_parse_failed_to_service_error() {
    // QueryParseFailed エラーが ServiceError::BadRequest に変換されることを検証
    let err = GraphqlGatewayError::QueryParseFailed("syntax error".to_string());
    let service_err: ServiceError = err.into();
    match service_err {
        ServiceError::BadRequest {
            code,
            message,
            details,
        } => {
            assert_eq!(code.as_str(), "SYS_GQLGW_QUERY_PARSE_FAILED");
            assert!(message.contains("syntax error"));
            assert!(details.is_empty());
        }
        _ => panic!("QueryParseFailed エラーが期待と異なる型に変換された"),
    }
}

#[test]
fn test_graphql_gateway_error_upstream_to_service_error() {
    // UpstreamError エラーが ServiceError::ServiceUnavailable に変換されることを検証
    let err = GraphqlGatewayError::UpstreamError("connection refused".to_string());
    let service_err: ServiceError = err.into();
    match service_err {
        ServiceError::ServiceUnavailable { code, message } => {
            assert_eq!(code.as_str(), "SYS_GQLGW_UPSTREAM_ERROR");
            assert!(message.contains("connection refused"));
        }
        _ => panic!("UpstreamError エラーが期待と異なる型に変換された"),
    }
}

#[test]
fn test_graphql_gateway_error_validation_to_service_error() {
    // ValidationFailed エラーが ServiceError::BadRequest に変換されることを検証
    let err = GraphqlGatewayError::ValidationFailed("name is required".to_string());
    let service_err: ServiceError = err.into();
    match service_err {
        ServiceError::BadRequest { code, message, .. } => {
            assert_eq!(code.as_str(), "SYS_GQLGW_VALIDATION_FAILED");
            assert!(message.contains("name is required"));
        }
        _ => panic!("ValidationFailed エラーが期待と異なる型に変換された"),
    }
}

#[test]
fn test_graphql_gateway_error_internal_to_service_error() {
    // Internal エラーが ServiceError::Internal に変換されることを検証
    let err = GraphqlGatewayError::Internal("unexpected panic".to_string());
    let service_err: ServiceError = err.into();
    match service_err {
        ServiceError::Internal { code, message } => {
            assert_eq!(code.as_str(), "SYS_GQLGW_INTERNAL_ERROR");
            assert!(message.contains("unexpected panic"));
        }
        _ => panic!("Internal エラーが期待と異なる型に変換された"),
    }
}

// --- ドメインモデルのCloneとDebugトレイト検証 ---

#[test]
fn test_tenant_clone_and_debug() {
    // Tenant が Clone と Debug を実装していることを検証
    let tenant = Tenant {
        id: "t-001".to_string(),
        name: "Test".to_string(),
        status: TenantStatus::Active,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };
    let cloned = tenant.clone();
    assert_eq!(cloned.id, tenant.id);
    // Debug 出力が空でないことを検証
    let debug_str = format!("{:?}", tenant);
    assert!(!debug_str.is_empty());
}

#[test]
fn test_page_info_empty_cursors() {
    // カーソルが None の場合のPageInfo構築を検証
    let page_info = PageInfo {
        has_next_page: false,
        has_previous_page: false,
        start_cursor: None,
        end_cursor: None,
    };
    assert!(page_info.start_cursor.is_none());
    assert!(page_info.end_cursor.is_none());
}
