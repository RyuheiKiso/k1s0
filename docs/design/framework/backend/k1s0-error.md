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

## Go 版（k1s0-error）

### 主要な型

```go
// ErrorKind はエラー分類を表す。
type ErrorKind int

const (
    InvalidInput      ErrorKind = iota
    NotFound
    Conflict
    Unauthorized
    Forbidden
    DependencyFailure
    Transient
    Internal
)

// DomainError は transport 非依存のドメインエラー。
type DomainError struct {
    Kind    ErrorKind
    Message string
    Cause   error
}

func NewNotFound(resource, id string) *DomainError
func NewConflict(message string) *DomainError
func NewInvalidInput(message string) *DomainError

// AppError は error_code 付きのアプリケーションエラー。
type AppError struct {
    DomainError *DomainError
    ErrorCode   string
    TraceID     string
    RequestID   string
}

func NewAppError(domainErr *DomainError, code string) *AppError
func (e *AppError) ToHTTPError() *HTTPError
func (e *AppError) ToGRPCError() *GRPCError
```

### 使用例

```go
import "github.com/k1s0/framework/backend/go/k1s0-error"

// domain 層
domainErr := k1s0error.NewNotFound("User", "user-123")

// application 層
appErr := k1s0error.NewAppError(domainErr, "USER_NOT_FOUND")
appErr.TraceID = "trace-abc123"

// presentation 層
httpErr := appErr.ToHTTPError()   // -> 404 + problem+json
grpcErr := appErr.ToGRPCError()   // -> NOT_FOUND + metadata
```

## C# 版（K1s0.Error）

### 主要な型

```csharp
// エラー分類
public enum ErrorKind
{
    InvalidInput, NotFound, Conflict, Unauthorized,
    Forbidden, DependencyFailure, Transient, Internal
}

// transport 非依存のドメインエラー
public class DomainError : Exception
{
    public ErrorKind Kind { get; }
    public static DomainError NotFound(string resource, string id);
    public static DomainError Conflict(string message);
    public static DomainError InvalidInput(string message);
}

// error_code 付きアプリケーションエラー
public class AppError
{
    public DomainError DomainError { get; }
    public string ErrorCode { get; }
    public string? TraceId { get; set; }
    public string? RequestId { get; set; }

    public static AppError FromDomain(DomainError err, string code);
    public HttpError ToHttpError();
    public GrpcError ToGrpcError();
}
```

### 使用例

```csharp
using K1s0.Error;

// domain 層
var domainErr = DomainError.NotFound("User", "user-123");

// application 層
var appErr = AppError.FromDomain(domainErr, "USER_NOT_FOUND");
appErr.TraceId = "trace-abc123";

// presentation 層
var httpErr = appErr.ToHttpError();   // -> 404 + problem+json
var grpcErr = appErr.ToGrpcError();   // -> NOT_FOUND + metadata
```

## Python 版（k1s0-error）

### 主要な型

```python
from enum import Enum
from dataclasses import dataclass

class ErrorKind(Enum):
    INVALID_INPUT = "invalid_input"
    NOT_FOUND = "not_found"
    CONFLICT = "conflict"
    UNAUTHORIZED = "unauthorized"
    FORBIDDEN = "forbidden"
    DEPENDENCY_FAILURE = "dependency_failure"
    TRANSIENT = "transient"
    INTERNAL = "internal"

class DomainError(Exception):
    kind: ErrorKind
    message: str

    @classmethod
    def not_found(cls, resource: str, id: str) -> "DomainError": ...
    @classmethod
    def conflict(cls, message: str) -> "DomainError": ...
    @classmethod
    def invalid_input(cls, message: str) -> "DomainError": ...

@dataclass
class AppError:
    domain_error: DomainError
    error_code: str
    trace_id: str | None = None
    request_id: str | None = None

    @classmethod
    def from_domain(cls, err: DomainError, code: str) -> "AppError": ...
    def to_http_error(self) -> "HttpError": ...
    def to_grpc_error(self) -> "GrpcError": ...
```

### 使用例

```python
from k1s0_error import DomainError, AppError

# domain 層
domain_err = DomainError.not_found("User", "user-123")

# application 層
app_err = AppError.from_domain(domain_err, "USER_NOT_FOUND")
app_err.trace_id = "trace-abc123"

# presentation 層
http_err = app_err.to_http_error()   # -> 404 + problem+json
grpc_err = app_err.to_grpc_error()   # -> NOT_FOUND + metadata
```

## Kotlin 版（k1s0-error）

### 主要な型

```kotlin
enum class ErrorKind {
    InvalidInput, NotFound, Conflict, Unauthorized,
    Forbidden, DependencyFailure, Transient, Internal
}

// transport 非依存のドメインエラー
class DomainError(
    val kind: ErrorKind,
    override val message: String,
    override val cause: Throwable? = null
) : Exception(message, cause) {
    companion object {
        fun notFound(resource: String, id: String): DomainError
        fun conflict(message: String): DomainError
        fun invalidInput(message: String): DomainError
    }
}

// error_code 付きアプリケーションエラー
data class AppError(
    val domainError: DomainError,
    val errorCode: String,
    val traceId: String? = null,
    val requestId: String? = null
) {
    companion object {
        fun fromDomain(err: DomainError, code: String): AppError
    }
    fun toHttpError(): HttpError
    fun toGrpcError(): GrpcError
}
```

### 使用例

```kotlin
import com.k1s0.error.*

// domain 層
val domainErr = DomainError.notFound("User", "user-123")

// application 層
val appErr = AppError.fromDomain(domainErr, "USER_NOT_FOUND")
    .copy(traceId = "trace-abc123")

// presentation 層
val httpErr = appErr.toHttpError()   // -> 404 + problem+json
val grpcErr = appErr.toGrpcError()   // -> NOT_FOUND + metadata
```
