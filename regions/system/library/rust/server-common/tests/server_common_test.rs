//! Integration tests for k1s0-server-common.

use k1s0_server_common::{
    ApiResponse, ErrorBody, ErrorCode, ErrorDetail, ErrorResponse, PaginatedResponse,
    PaginationResponse, ServiceError,
};

// ============================================================
// ErrorCode tests
// ============================================================

// カスタム文字列で ErrorCode が作成されることを確認する。
#[test]
fn error_code_new_creates_custom_code() {
    let code = ErrorCode::new("SYS_CUSTOM_ERROR");
    assert_eq!(code.as_str(), "SYS_CUSTOM_ERROR");
}

// not_found で生成したエラーコードが命名規則に従うことを確認する。
#[test]
fn error_code_not_found_follows_naming_pattern() {
    let code = ErrorCode::not_found("AUTH");
    assert_eq!(code.as_str(), "SYS_AUTH_NOT_FOUND");
}

// validation で生成したエラーコードが命名規則に従うことを確認する。
#[test]
fn error_code_validation_follows_naming_pattern() {
    let code = ErrorCode::validation("CONFIG");
    assert_eq!(code.as_str(), "SYS_CONFIG_VALIDATION_FAILED");
}

// internal で生成したエラーコードが命名規則に従うことを確認する。
#[test]
fn error_code_internal_follows_naming_pattern() {
    let code = ErrorCode::internal("DLQ");
    assert_eq!(code.as_str(), "SYS_DLQ_INTERNAL_ERROR");
}

// unauthorized で生成したエラーコードが命名規則に従うことを確認する。
#[test]
fn error_code_unauthorized_follows_naming_pattern() {
    let code = ErrorCode::unauthorized("SESSION");
    assert_eq!(code.as_str(), "SYS_SESSION_UNAUTHORIZED");
}

// forbidden で生成したエラーコードが命名規則に従うことを確認する。
#[test]
fn error_code_forbidden_follows_naming_pattern() {
    let code = ErrorCode::forbidden("TENANT");
    assert_eq!(code.as_str(), "SYS_TENANT_PERMISSION_DENIED");
}

// conflict で生成したエラーコードが命名規則に従うことを確認する。
#[test]
fn error_code_conflict_follows_naming_pattern() {
    let code = ErrorCode::conflict("APIREG");
    assert_eq!(code.as_str(), "SYS_APIREG_CONFLICT");
}

// unprocessable で生成したエラーコードが命名規則に従うことを確認する。
#[test]
fn error_code_unprocessable_follows_naming_pattern() {
    let code = ErrorCode::unprocessable("ORDER");
    assert_eq!(code.as_str(), "SYS_ORDER_BUSINESS_RULE_VIOLATION");
}

// rate_exceeded で生成したエラーコードが命名規則に従うことを確認する。
#[test]
fn error_code_rate_exceeded_follows_naming_pattern() {
    let code = ErrorCode::rate_exceeded("API");
    assert_eq!(code.as_str(), "SYS_API_RATE_EXCEEDED");
}

// service_unavailable で生成したエラーコードが命名規則に従うことを確認する。
#[test]
fn error_code_service_unavailable_follows_naming_pattern() {
    let code = ErrorCode::service_unavailable("AUTH");
    assert_eq!(code.as_str(), "SYS_AUTH_SERVICE_UNAVAILABLE");
}

// ビジネス層の not_found エラーコードが BIZ_ プレフィックスで生成されることを確認する。
#[test]
fn error_code_biz_not_found() {
    let code = ErrorCode::biz_not_found("ORDER");
    assert_eq!(code.as_str(), "BIZ_ORDER_NOT_FOUND");
}

// ビジネス層の validation エラーコードが BIZ_ プレフィックスで生成されることを確認する。
#[test]
fn error_code_biz_validation() {
    let code = ErrorCode::biz_validation("PAYMENT");
    assert_eq!(code.as_str(), "BIZ_PAYMENT_VALIDATION_FAILED");
}

// サービス層の not_found エラーコードが SVC_ プレフィックスで生成されることを確認する。
#[test]
fn error_code_svc_not_found() {
    let code = ErrorCode::svc_not_found("INVENTORY");
    assert_eq!(code.as_str(), "SVC_INVENTORY_NOT_FOUND");
}

// サービス層の validation エラーコードが SVC_ プレフィックスで生成されることを確認する。
#[test]
fn error_code_svc_validation() {
    let code = ErrorCode::svc_validation("SHIPPING");
    assert_eq!(code.as_str(), "SVC_SHIPPING_VALIDATION_FAILED");
}

// サービス名が自動的に大文字変換されることを確認する。
#[test]
fn error_code_uppercases_service_name() {
    let code = ErrorCode::not_found("my_service");
    assert_eq!(code.as_str(), "SYS_MY_SERVICE_NOT_FOUND");
}

// ErrorCode の Display 出力が as_str の値と一致することを確認する。
#[test]
fn error_code_display_matches_as_str() {
    let code = ErrorCode::new("SYS_TEST_CODE");
    assert_eq!(format!("{}", code), "SYS_TEST_CODE");
}

// &str から ErrorCode への変換が正しく機能することを確認する。
#[test]
fn error_code_from_str() {
    let code: ErrorCode = "SYS_CUSTOM".into();
    assert_eq!(code.as_str(), "SYS_CUSTOM");
}

// String から ErrorCode への変換が正しく機能することを確認する。
#[test]
fn error_code_from_string() {
    let code: ErrorCode = String::from("SYS_DYNAMIC").into();
    assert_eq!(code.as_str(), "SYS_DYNAMIC");
}

// 同じ文字列で作成した ErrorCode が等価であり、異なる文字列では非等価であることを確認する。
#[test]
fn error_code_equality() {
    let a = ErrorCode::new("SYS_X");
    let b = ErrorCode::new("SYS_X");
    let c = ErrorCode::new("SYS_Y");
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ErrorCode がクローンされて元と等価になることを確認する。
#[test]
fn error_code_clone() {
    let original = ErrorCode::new("SYS_CLONE_TEST");
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// ErrorCode が JSON 文字列としてシリアライズされることを確認する。
#[test]
fn error_code_serializes_as_string() {
    let code = ErrorCode::new("SYS_TEST_SERIALIZE");
    let json = serde_json::to_value(&code).unwrap();
    assert_eq!(json, serde_json::json!("SYS_TEST_SERIALIZE"));
}

// ============================================================
// ErrorDetail tests
// ============================================================

// ErrorDetail がフィールド・理由・メッセージで正しく構築されることを確認する。
#[test]
fn error_detail_construction() {
    let detail = ErrorDetail::new("email", "format", "invalid email address");
    assert_eq!(detail.field, "email");
    assert_eq!(detail.reason, "format");
    assert_eq!(detail.message, "invalid email address");
}

// ErrorDetail が String 型の引数を受け入れることを確認する。
#[test]
fn error_detail_accepts_string_types() {
    let detail = ErrorDetail::new(
        String::from("quantity"),
        String::from("range"),
        String::from("must be positive"),
    );
    assert_eq!(detail.field, "quantity");
    assert_eq!(detail.reason, "range");
    assert_eq!(detail.message, "must be positive");
}

// ErrorDetail の JSON シリアライズで各フィールドが正しく出力されることを確認する。
#[test]
fn error_detail_serialization() {
    let detail = ErrorDetail::new("name", "required", "must not be empty");
    let json = serde_json::to_value(&detail).unwrap();
    assert_eq!(json["field"], "name");
    assert_eq!(json["reason"], "required");
    assert_eq!(json["message"], "must not be empty");
}

// ============================================================
// ErrorBody tests
// ============================================================

// details が空の場合に JSON シリアライズで details フィールドが省略されることを確認する。
#[test]
fn error_body_serialization_omits_empty_details() {
    let body = ErrorBody {
        code: ErrorCode::new("SYS_TEST"),
        message: "test error".to_string(),
        request_id: "req-123".to_string(),
        details: vec![],
    };
    let json = serde_json::to_value(&body).unwrap();
    assert!(json.get("details").is_none());
}

// details が存在する場合に JSON シリアライズで details フィールドが含まれることを確認する。
#[test]
fn error_body_serialization_includes_nonempty_details() {
    let body = ErrorBody {
        code: ErrorCode::new("SYS_TEST"),
        message: "test error".to_string(),
        request_id: "req-123".to_string(),
        details: vec![ErrorDetail::new("field1", "reason1", "msg1")],
    };
    let json = serde_json::to_value(&body).unwrap();
    assert!(json.get("details").is_some());
    assert_eq!(json["details"][0]["field"], "field1");
}

// ============================================================
// ErrorResponse tests
// ============================================================

// ErrorResponse::new でコードとメッセージが正しく設定されることを確認する。
#[test]
fn error_response_new_sets_code_and_message() {
    let resp = ErrorResponse::new("SYS_AUTH_UNAUTHORIZED", "not authorized");
    assert_eq!(resp.error.code.as_str(), "SYS_AUTH_UNAUTHORIZED");
    assert_eq!(resp.error.message, "not authorized");
    assert!(resp.error.details.is_empty());
}

// ErrorResponse::new で request_id が自動生成されることを確認する。
#[test]
fn error_response_new_generates_request_id() {
    let resp = ErrorResponse::new("SYS_TEST", "test");
    assert!(!resp.error.request_id.is_empty());
    // request_id should be a valid UUID format (36 chars with hyphens)
    assert_eq!(resp.error.request_id.len(), 36);
}

// ErrorResponse::with_details で詳細情報が含まれることを確認する。
#[test]
fn error_response_with_details_includes_details() {
    let details = vec![
        ErrorDetail::new("field_a", "required", "missing"),
        ErrorDetail::new("field_b", "format", "invalid"),
    ];
    let resp =
        ErrorResponse::with_details("SYS_CONFIG_VALIDATION_FAILED", "validation failed", details);
    assert_eq!(resp.error.details.len(), 2);
    assert_eq!(resp.error.details[0].field, "field_a");
    assert_eq!(resp.error.details[1].field, "field_b");
}

// with_request_id でデフォルトの request_id が上書きされることを確認する。
#[test]
fn error_response_with_request_id_overrides_default() {
    let resp = ErrorResponse::new("SYS_TEST", "test").with_request_id("custom-correlation-id-123");
    assert_eq!(resp.error.request_id, "custom-correlation-id-123");
}

// ErrorResponse が error キーでラップされた JSON にシリアライズされることを確認する。
#[test]
fn error_response_serialization_envelope() {
    let resp = ErrorResponse::new("SYS_CONFIG_KEY_NOT_FOUND", "key not found")
        .with_request_id("test-req-id");
    let json = serde_json::to_value(&resp).unwrap();

    // Verify envelope structure: { "error": { ... } }
    assert!(json.get("error").is_some());
    assert_eq!(json["error"]["code"], "SYS_CONFIG_KEY_NOT_FOUND");
    assert_eq!(json["error"]["message"], "key not found");
    assert_eq!(json["error"]["request_id"], "test-req-id");
    // details should be omitted when empty
    assert!(json["error"].get("details").is_none());
}

// 詳細情報付き ErrorResponse が JSON に正しくシリアライズされることを確認する。
#[test]
fn error_response_with_details_serialization() {
    let details = vec![ErrorDetail::new(
        "namespace",
        "required",
        "must not be empty",
    )];
    let resp = ErrorResponse::with_details("SYS_CONFIG_VALIDATION_FAILED", "invalid", details)
        .with_request_id("req-456");
    let json = serde_json::to_value(&resp).unwrap();

    assert_eq!(json["error"]["details"][0]["field"], "namespace");
    assert_eq!(json["error"]["details"][0]["reason"], "required");
    assert_eq!(json["error"]["details"][0]["message"], "must not be empty");
}

// ============================================================
// ServiceError tests
// ============================================================

// ServiceError::not_found の Display 出力がメッセージと一致することを確認する。
#[test]
fn service_error_not_found_display() {
    let err = ServiceError::not_found("CONFIG", "key 'db.host' not found");
    assert_eq!(format!("{}", err), "key 'db.host' not found");
}

// ServiceError::not_found から ErrorResponse への変換が正しいことを確認する。
#[test]
fn service_error_not_found_to_error_response() {
    let err = ServiceError::not_found("CONFIG", "not found");
    let resp = err.to_error_response();
    assert_eq!(resp.error.code.as_str(), "SYS_CONFIG_NOT_FOUND");
    assert_eq!(resp.error.message, "not found");
    assert!(resp.error.details.is_empty());
}

// ServiceError::bad_request から ErrorResponse への変換が正しいことを確認する。
#[test]
fn service_error_bad_request_to_error_response() {
    let err = ServiceError::bad_request("DLQ", "invalid input");
    let resp = err.to_error_response();
    assert_eq!(resp.error.code.as_str(), "SYS_DLQ_VALIDATION_FAILED");
    assert_eq!(resp.error.message, "invalid input");
    assert!(resp.error.details.is_empty());
}

// ServiceError::bad_request_with_details から ErrorResponse に詳細情報が含まれることを確認する。
#[test]
fn service_error_bad_request_with_details_to_error_response() {
    let details = vec![
        ErrorDetail::new("page", "range", "must be >= 1"),
        ErrorDetail::new("size", "range", "must be <= 100"),
    ];
    let err = ServiceError::bad_request_with_details("CONFIG", "validation failed", details);
    let resp = err.to_error_response();
    assert_eq!(resp.error.code.as_str(), "SYS_CONFIG_VALIDATION_FAILED");
    assert_eq!(resp.error.details.len(), 2);
    assert_eq!(resp.error.details[0].field, "page");
    assert_eq!(resp.error.details[1].field, "size");
}

// ServiceError::unauthorized から ErrorResponse への変換が正しいことを確認する。
#[test]
fn service_error_unauthorized_to_error_response() {
    let err = ServiceError::unauthorized("AUTH", "missing token");
    let resp = err.to_error_response();
    assert_eq!(resp.error.code.as_str(), "SYS_AUTH_UNAUTHORIZED");
    assert_eq!(resp.error.message, "missing token");
}

// ServiceError::forbidden から ErrorResponse への変換が正しいことを確認する。
#[test]
fn service_error_forbidden_to_error_response() {
    let err = ServiceError::forbidden("TENANT", "not allowed");
    let resp = err.to_error_response();
    assert_eq!(resp.error.code.as_str(), "SYS_TENANT_PERMISSION_DENIED");
    assert_eq!(resp.error.message, "not allowed");
}

// ServiceError::conflict から ErrorResponse への変換が正しいことを確認する。
#[test]
fn service_error_conflict_to_error_response() {
    let err = ServiceError::conflict("APIREG", "version already exists");
    let resp = err.to_error_response();
    assert_eq!(resp.error.code.as_str(), "SYS_APIREG_CONFLICT");
    assert_eq!(resp.error.message, "version already exists");
    assert!(resp.error.details.is_empty());
}

// ServiceError::unprocessable_entity から ErrorResponse への変換が正しいことを確認する。
#[test]
fn service_error_unprocessable_entity_to_error_response() {
    let err = ServiceError::unprocessable_entity("ACCT", "ledger is closed");
    let resp = err.to_error_response();
    assert_eq!(resp.error.code.as_str(), "SYS_ACCT_BUSINESS_RULE_VIOLATION");
    assert_eq!(resp.error.message, "ledger is closed");
}

// ServiceError::too_many_requests から ErrorResponse への変換が正しいことを確認する。
#[test]
fn service_error_too_many_requests_to_error_response() {
    let err = ServiceError::too_many_requests("API", "rate limit exceeded");
    let resp = err.to_error_response();
    assert_eq!(resp.error.code.as_str(), "SYS_API_RATE_EXCEEDED");
    assert_eq!(resp.error.message, "rate limit exceeded");
}

// ServiceError::internal から ErrorResponse への変換が正しいことを確認する。
#[test]
fn service_error_internal_to_error_response() {
    let err = ServiceError::internal("DB", "connection pool exhausted");
    let resp = err.to_error_response();
    assert_eq!(resp.error.code.as_str(), "SYS_DB_INTERNAL_ERROR");
    assert_eq!(resp.error.message, "connection pool exhausted");
}

// ServiceError::service_unavailable から ErrorResponse への変換が正しいことを確認する。
#[test]
fn service_error_service_unavailable_to_error_response() {
    let err = ServiceError::service_unavailable("AUTH", "maintenance mode");
    let resp = err.to_error_response();
    assert_eq!(resp.error.code.as_str(), "SYS_AUTH_SERVICE_UNAVAILABLE");
    assert_eq!(resp.error.message, "maintenance mode");
}

// すべての ServiceError バリアントの Display 出力がメッセージと一致することを確認する。
#[test]
fn service_error_display_shows_message() {
    let variants: Vec<ServiceError> = vec![
        ServiceError::not_found("S", "msg_not_found"),
        ServiceError::bad_request("S", "msg_bad_request"),
        ServiceError::unauthorized("S", "msg_unauthorized"),
        ServiceError::forbidden("S", "msg_forbidden"),
        ServiceError::conflict("S", "msg_conflict"),
        ServiceError::unprocessable_entity("S", "msg_unprocessable"),
        ServiceError::too_many_requests("S", "msg_too_many"),
        ServiceError::internal("S", "msg_internal"),
        ServiceError::service_unavailable("S", "msg_unavailable"),
    ];
    let expected_messages = vec![
        "msg_not_found",
        "msg_bad_request",
        "msg_unauthorized",
        "msg_forbidden",
        "msg_conflict",
        "msg_unprocessable",
        "msg_too_many",
        "msg_internal",
        "msg_unavailable",
    ];
    for (err, expected) in variants.into_iter().zip(expected_messages) {
        assert_eq!(format!("{}", err), expected);
    }
}

// ServiceError が std::error::Error トレイトを実装していることを確認する。
#[test]
fn service_error_is_std_error() {
    let err = ServiceError::internal("TEST", "something went wrong");
    let _: &dyn std::error::Error = &err;
}

// ============================================================
// ApiResponse tests
// ============================================================

// ApiResponse がデータを data キーでラップして JSON 出力することを確認する。
#[test]
fn api_response_wraps_data() {
    let resp = ApiResponse {
        data: "hello world",
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["data"], "hello world");
}

// ApiResponse が構造体データを正しく JSON シリアライズすることを確認する。
#[test]
fn api_response_with_struct() {
    #[derive(serde::Serialize)]
    struct User {
        id: String,
        name: String,
    }

    let resp = ApiResponse {
        data: User {
            id: "u-1".to_string(),
            name: "Taro".to_string(),
        },
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["data"]["id"], "u-1");
    assert_eq!(json["data"]["name"], "Taro");
}

// ApiResponse がベクター型データを正しく JSON シリアライズすることを確認する。
#[test]
fn api_response_with_vec() {
    let resp = ApiResponse {
        data: vec![1, 2, 3],
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["data"], serde_json::json!([1, 2, 3]));
}

// ============================================================
// PaginatedResponse tests
// ============================================================

// PaginatedResponse がページネーション情報とアイテムを正しく JSON シリアライズすることを確認する。
#[test]
fn paginated_response_serialization() {
    let resp = PaginatedResponse {
        items: vec!["item1", "item2"],
        pagination: PaginationResponse {
            total_count: 50,
            page: 1,
            page_size: 10,
            has_next: true,
        },
    };
    let json = serde_json::to_value(&resp).unwrap();

    assert_eq!(json["items"], serde_json::json!(["item1", "item2"]));
    assert_eq!(json["pagination"]["total_count"], 50);
    assert_eq!(json["pagination"]["page"], 1);
    assert_eq!(json["pagination"]["page_size"], 10);
    assert_eq!(json["pagination"]["has_next"], true);
}

// 最終ページの PaginatedResponse で has_next が false になることを確認する。
#[test]
fn paginated_response_last_page() {
    let resp = PaginatedResponse {
        items: vec!["last_item"],
        pagination: PaginationResponse {
            total_count: 11,
            page: 2,
            page_size: 10,
            has_next: false,
        },
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["pagination"]["has_next"], false);
    assert_eq!(json["pagination"]["page"], 2);
}

// 空のアイテムリストを持つ PaginatedResponse が正しく JSON 出力されることを確認する。
#[test]
fn paginated_response_empty_items() {
    let resp: PaginatedResponse<String> = PaginatedResponse {
        items: vec![],
        pagination: PaginationResponse {
            total_count: 0,
            page: 1,
            page_size: 10,
            has_next: false,
        },
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["items"], serde_json::json!([]));
    assert_eq!(json["pagination"]["total_count"], 0);
}

// 構造体アイテムを持つ PaginatedResponse が正しく JSON シリアライズされることを確認する。
#[test]
fn paginated_response_with_struct_items() {
    #[derive(serde::Serialize)]
    struct Config {
        key: String,
        value: String,
    }

    let resp = PaginatedResponse {
        items: vec![
            Config {
                key: "db.host".to_string(),
                value: "localhost".to_string(),
            },
            Config {
                key: "db.port".to_string(),
                value: "5432".to_string(),
            },
        ],
        pagination: PaginationResponse {
            total_count: 2,
            page: 1,
            page_size: 10,
            has_next: false,
        },
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["items"][0]["key"], "db.host");
    assert_eq!(json["items"][1]["key"], "db.port");
}

// PaginationResponse がクローン後も同じ値を保持することを確認する。
#[test]
fn pagination_response_clone() {
    let original = PaginationResponse {
        total_count: 100,
        page: 3,
        page_size: 20,
        has_next: true,
    };
    let cloned = original.clone();
    assert_eq!(cloned.total_count, 100);
    assert_eq!(cloned.page, 3);
    assert_eq!(cloned.page_size, 20);
    assert!(cloned.has_next);
}

// ============================================================
// auth module tests (allow_insecure_no_auth, require_auth_state)
// ============================================================

// 本番環境では ALLOW_INSECURE_NO_AUTH が true であっても認証なしが拒否されることを確認する。
#[test]
fn allow_insecure_no_auth_rejects_production() {
    // Even if env var is set, production should be rejected
    std::env::set_var("ALLOW_INSECURE_NO_AUTH", "true");
    assert!(!k1s0_server_common::allow_insecure_no_auth("production"));
    assert!(!k1s0_server_common::allow_insecure_no_auth("staging"));
    std::env::remove_var("ALLOW_INSECURE_NO_AUTH");
}

// 認証設定が存在する場合に require_auth_state が Some を返すことを確認する。
#[test]
fn require_auth_state_returns_some_when_auth_present() {
    let result =
        k1s0_server_common::require_auth_state("svc", "production", Some("auth-config")).unwrap();
    assert_eq!(result, Some("auth-config"));
}

// ============================================================
// Well-known error code module tests
// ============================================================

// Auth サービスの既知エラーコードが正しい文字列を返すことを確認する。
#[test]
fn well_known_error_codes_auth() {
    use k1s0_server_common::error::auth;
    assert_eq!(auth::missing_claims().as_str(), "SYS_AUTH_MISSING_CLAIMS");
    assert_eq!(
        auth::permission_denied().as_str(),
        "SYS_AUTH_PERMISSION_DENIED"
    );
    assert_eq!(auth::unauthorized().as_str(), "SYS_AUTH_UNAUTHORIZED");
    assert_eq!(auth::token_expired().as_str(), "SYS_AUTH_TOKEN_EXPIRED");
    assert_eq!(auth::invalid_token().as_str(), "SYS_AUTH_INVALID_TOKEN");
    assert_eq!(
        auth::jwks_fetch_failed().as_str(),
        "SYS_AUTH_JWKS_FETCH_FAILED"
    );
    assert_eq!(
        auth::audit_validation().as_str(),
        "SYS_AUTH_AUDIT_VALIDATION"
    );
}

// Config サービスの既知エラーコードが正しい文字列を返すことを確認する。
#[test]
fn well_known_error_codes_config() {
    use k1s0_server_common::error::config;
    assert_eq!(config::key_not_found().as_str(), "SYS_CONFIG_KEY_NOT_FOUND");
    assert_eq!(
        config::service_not_found().as_str(),
        "SYS_CONFIG_SERVICE_NOT_FOUND"
    );
    assert_eq!(
        config::schema_not_found().as_str(),
        "SYS_CONFIG_SCHEMA_NOT_FOUND"
    );
    assert_eq!(
        config::version_conflict().as_str(),
        "SYS_CONFIG_VERSION_CONFLICT"
    );
    assert_eq!(
        config::validation_failed().as_str(),
        "SYS_CONFIG_VALIDATION_FAILED"
    );
    assert_eq!(
        config::internal_error().as_str(),
        "SYS_CONFIG_INTERNAL_ERROR"
    );
}

// Event Store サービスの既知エラーコードが正しい文字列を返すことを確認する。
#[test]
fn well_known_error_codes_event_store() {
    use k1s0_server_common::error::event_store;
    assert_eq!(
        event_store::stream_not_found().as_str(),
        "SYS_EVSTORE_STREAM_NOT_FOUND"
    );
    assert_eq!(
        event_store::event_not_found().as_str(),
        "SYS_EVSTORE_EVENT_NOT_FOUND"
    );
    assert_eq!(
        event_store::version_conflict().as_str(),
        "SYS_EVSTORE_VERSION_CONFLICT"
    );
    assert_eq!(
        event_store::stream_already_exists().as_str(),
        "SYS_EVSTORE_STREAM_ALREADY_EXISTS"
    );
}

// Feature Flag サービスの既知エラーコードが正しい文字列を返すことを確認する。
#[test]
fn well_known_error_codes_featureflag() {
    use k1s0_server_common::error::featureflag;
    assert_eq!(
        featureflag::internal_error().as_str(),
        "SYS_FF_INTERNAL_ERROR"
    );
    assert_eq!(featureflag::not_found().as_str(), "SYS_FF_NOT_FOUND");
    assert_eq!(
        featureflag::already_exists().as_str(),
        "SYS_FF_ALREADY_EXISTS"
    );
}
