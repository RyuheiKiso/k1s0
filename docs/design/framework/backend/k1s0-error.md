# k1s0-error

## 目的

Clean Architecture に基づいたエラー表現の統一ライブラリ。transport 非依存で層別のエラー設計を提供する。

## 設計方針

- **domain 層**: transport 非依存のエラー型（HTTP/gRPC を意識しない）
- **application 層**: `error_code` を付与し、運用で識別可能にする
- **presentation 層**: REST（problem+json）/ gRPC（status + metadata）へ変換

## エラー分類

| 分類 | 説明 | HTTP | gRPC |
|------|------|------|------|
| InvalidInput | 入力不備 | 400 | INVALID_ARGUMENT |
| NotFound | リソースが見つからない | 404 | NOT_FOUND |
| Conflict | 競合（重複等） | 409 | ALREADY_EXISTS |
| Unauthorized | 認証エラー | 401 | UNAUTHENTICATED |
| Forbidden | 認可エラー | 403 | PERMISSION_DENIED |
| DependencyFailure | 依存障害 | 502 | UNAVAILABLE |
| Transient | 一時障害 | 503 | UNAVAILABLE |
| Internal | 内部エラー | 500 | INTERNAL |

## 主要な型

### DomainError

```rust
pub struct DomainError {
    kind: ErrorKind,
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl DomainError {
    pub fn not_found(resource: &str, id: &str) -> Self;
    pub fn conflict(message: impl Into<String>) -> Self;
    pub fn invalid_input(message: impl Into<String>) -> Self;
    pub fn internal(message: impl Into<String>) -> Self;
    pub fn kind(&self) -> ErrorKind;
}
```

### AppError

```rust
pub struct AppError {
    domain_error: DomainError,
    error_code: ErrorCode,
    trace_id: Option<String>,
    request_id: Option<String>,
}

impl AppError {
    pub fn from_domain(err: DomainError, code: ErrorCode) -> Self;
    pub fn with_trace_id(self, trace_id: impl Into<String>) -> Self;
    pub fn with_request_id(self, request_id: impl Into<String>) -> Self;
    pub fn to_http_error(&self) -> HttpError;
    pub fn to_grpc_error(&self) -> GrpcError;
}
```

### ErrorCode

```rust
pub struct ErrorCode(String);

impl ErrorCode {
    pub fn new(code: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

// 例: ErrorCode::new("USER_NOT_FOUND")
```

## 使用例

```rust
use k1s0_error::{DomainError, AppError, ErrorCode, ErrorKind};

// domain 層: transport 非依存
let domain_err = DomainError::not_found("User", "user-123");

// application 層: error_code 付与
let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"))
    .with_trace_id("trace-abc123")
    .with_request_id("req-xyz789");

// presentation 層: REST/gRPC 変換
let http_err = app_err.to_http_error();  // -> 404 + problem+json
let grpc_err = app_err.to_grpc_error();  // -> NOT_FOUND + metadata
```
