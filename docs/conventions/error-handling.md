# エラーハンドリング規約

本ドキュメントは、k1s0 におけるエラー表現と出力の規約を定義する。

## 1. 目的

- チーム間でエラー表現がバラつくことを防ぐ
- 運用（調査/監視/アラート/顧客対応）を標準化する

## 2. エラーの層別責務

### 2.1 domain 層

- **業務上の失敗** を表す
- HTTP/gRPC に依存しない
- 外部 I/O（DB/HTTP/Redis 等）の具体エラーを直接露出しない

```rust
// 例: Rust
pub enum DomainError {
    UserNotFound { user_id: UserId },
    EmailAlreadyExists { email: Email },
    InsufficientPermission { required: Permission },
}
```

```go
// 例: Go
type DomainError struct {
    Code    string
    Message string
}

var (
    ErrUserNotFound          = &DomainError{Code: "user.not_found", Message: "User not found"}
    ErrEmailAlreadyExists    = &DomainError{Code: "user.already_exists", Message: "Email already exists"}
)
```

```csharp
// 例: C#
public abstract class DomainException : Exception
{
    public string ErrorCode { get; }
    protected DomainException(string errorCode, string message) : base(message)
        => ErrorCode = errorCode;
}

public class UserNotFoundException : DomainException
{
    public UserNotFoundException(string userId)
        : base("user.not_found", $"User {userId} was not found") { }
}
```

```python
# 例: Python
class DomainError(Exception):
    def __init__(self, error_code: str, message: str):
        self.error_code = error_code
        super().__init__(message)

class UserNotFoundError(DomainError):
    def __init__(self, user_id: str):
        super().__init__("user.not_found", f"User {user_id} was not found")
```

```kotlin
// 例: Kotlin
sealed class DomainError(val errorCode: String, override val message: String) : Exception(message) {
    class UserNotFound(userId: String) : DomainError("user.not_found", "User $userId was not found")
    class EmailAlreadyExists(email: String) : DomainError("user.already_exists", "Email $email already exists")
}
```

### 2.2 application 層

- domain エラーと外部 I/O エラーを受け取る
- ユーザーに返す失敗の **分類/再試行可否/影響範囲** を決める
- `error_code` を付与する

```rust
// 例: Rust
pub struct ApplicationError {
    pub error_code: String,
    pub kind: ErrorKind,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error>>,
}
```

```go
// 例: Go
type ApplicationError struct {
    ErrorCode string
    Kind      ErrorKind
    Message   string
    Cause     error
}
```

```csharp
// 例: C#
public class ApplicationException : Exception
{
    public string ErrorCode { get; }
    public ErrorKind Kind { get; }
    public ApplicationException(string errorCode, ErrorKind kind, string message, Exception? inner = null)
        : base(message, inner) { ErrorCode = errorCode; Kind = kind; }
}
```

```python
# 例: Python
@dataclass
class ApplicationError(Exception):
    error_code: str
    kind: ErrorKind
    message: str
    cause: Exception | None = None
```

```kotlin
// 例: Kotlin
data class ApplicationError(
    val errorCode: String,
    val kind: ErrorKind,
    override val message: String,
    override val cause: Throwable? = null,
) : Exception(message, cause)
```

### 2.3 presentation 層

- application の失敗を **REST/gRPC の表現** へ変換する
- 変換ルールは framework が提供する共通実装を使用する

## 3. エラーコード（error_code）

### 3.1 規則

- 外部へ返すエラーには **安定した `error_code`** を必ず付与する
- `error_code` は変更しない（名称変更が必要な場合は段階移行）

### 3.2 命名規則

```
{service_name}.{category}.{reason}
```

- 小文字 + 数字 + アンダースコア
- ドット区切り

### 3.3 例

| error_code | 説明 |
|-----------|------|
| `auth.invalid_credentials` | 認証情報が不正 |
| `user.not_found` | ユーザーが見つからない |
| `user.already_exists` | ユーザーが既に存在 |
| `db.conflict` | DB での競合 |
| `config.fetch_failed` | 設定取得失敗 |

## 4. REST エラーレスポンス

### 4.1 形式

`application/problem+json`（RFC 7807 互換）

### 4.2 必須フィールド

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `status` | integer | HTTP ステータスコード |
| `title` | string | 短い要約（固定文言でよい） |
| `detail` | string | 人間向け詳細（機密情報は含めない） |
| `error_code` | string | エラーコード |
| `trace_id` | string | トレース ID |

### 4.3 オプションフィールド

| フィールド | 説明 |
|-----------|------|
| `request_id` | リクエスト ID（採用する場合） |
| `errors` | バリデーションエラーの詳細（配列） |

### 4.4 レスポンス例

```json
{
  "status": 404,
  "title": "Not Found",
  "detail": "User with ID 12345 was not found",
  "error_code": "user.not_found",
  "trace_id": "abc123def456"
}
```

### 4.5 バリデーションエラー例

```json
{
  "status": 400,
  "title": "Bad Request",
  "detail": "Validation failed",
  "error_code": "validation.failed",
  "trace_id": "abc123def456",
  "errors": [
    {"field": "email", "reason": "required"},
    {"field": "password", "reason": "too_short", "min": 8}
  ]
}
```

## 5. gRPC エラー表現

### 5.1 ステータスコード

gRPC は **Canonical Status Code** を使用し、アプリ独自コードは作らない。

| 状況 | Status Code |
|------|------------|
| 入力不正/バリデーション | `INVALID_ARGUMENT` |
| 認証失敗 | `UNAUTHENTICATED` |
| 認可失敗 | `PERMISSION_DENIED` |
| リソースなし | `NOT_FOUND` |
| 競合 | `ALREADY_EXISTS` / `FAILED_PRECONDITION` |
| 外部依存の一時障害 | `UNAVAILABLE` |
| タイムアウト | `DEADLINE_EXCEEDED` |
| 想定外 | `INTERNAL` |

### 5.2 error_code の伝達

`error_code` は gRPC メタデータとして付与し、クライアントが機械判定できるようにする。

```
metadata:
  error_code: user.not_found
```

実装方法は framework が統一する。

## 6. エラー分類（error.kind）

ログ/メトリクスで使用するエラー分類：

| kind | 説明 |
|------|------|
| `validation` | 入力バリデーション失敗 |
| `authz` | 認可失敗 |
| `authn` | 認証失敗 |
| `not_found` | リソースが見つからない |
| `conflict` | 競合（既存リソースとの衝突） |
| `dependency` | 外部依存の失敗（DB/Redis/他サービス） |
| `internal` | 想定外のエラー |

## 7. メトリクス

エラー発生時は以下のメトリクスを集計：

| メトリクス | ラベル |
|-----------|--------|
| `k1s0.{service}.request.failures` | protocol, route/method, status_code, error_code |
| `k1s0.{service}.dependency.failures` | dependency, error_kind |

## 8. 禁止事項

| 禁止事項 | 理由 |
|----------|------|
| スタックトレースを外部レスポンスに含める | セキュリティリスク |
| 機密情報をエラーメッセージに含める | 情報漏洩リスク |
| gRPC で独自ステータスコードを定義 | クライアント互換性 |
| error_code の変更（移行期間なし） | クライアント破壊 |

## 関連ドキュメント

- [観測性規約](observability.md)
- [構想.md](../../work/構想.md): 全体方針（12. エラー規約）
