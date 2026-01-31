# k1s0-validation

## 目的

API 境界での入力バリデーションを統一するライブラリ。REST（problem+json）と gRPC（INVALID_ARGUMENT）の両方に対応。

## 主要な型

### FieldError

```rust
pub struct FieldError {
    field: String,
    kind: FieldErrorKind,
    message: String,
}

pub enum FieldErrorKind {
    Required,
    InvalidFormat,
    MinLength(usize),
    MaxLength(usize),
    MinValue(i64),
    MaxValue(i64),
    Pattern(String),
    Custom(String),
}

impl FieldError {
    pub fn required(field: impl Into<String>) -> Self;
    pub fn invalid_format(field: impl Into<String>, message: impl Into<String>) -> Self;
    pub fn min_length(field: impl Into<String>, min: usize) -> Self;
    pub fn max_length(field: impl Into<String>, max: usize) -> Self;
}
```

### ValidationErrors

```rust
pub struct ValidationErrors {
    errors: HashMap<String, Vec<FieldError>>,
}

impl ValidationErrors {
    pub fn new() -> Self;
    pub fn add_field_error(&mut self, error: FieldError);
    pub fn is_empty(&self) -> bool;
    pub fn len(&self) -> usize;
    pub fn to_problem_details(&self, instance: &str, title: &str) -> ProblemDetails;
    pub fn to_grpc_details(&self) -> GrpcErrorDetails;
}
```

### Validate トレイト

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValidationErrors>;
}
```

## 使用例

```rust
use k1s0_validation::{ValidationErrors, FieldError, Validate};

#[derive(Debug)]
struct CreateUserRequest {
    name: String,
    email: String,
    password: String,
}

impl Validate for CreateUserRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if self.name.is_empty() {
            errors.add_field_error(FieldError::required("name"));
        }

        if !self.email.contains('@') {
            errors.add_field_error(
                FieldError::invalid_format("email", "有効なメールアドレスを入力してください")
            );
        }

        if self.password.len() < 8 {
            errors.add_field_error(FieldError::min_length("password", 8));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

## Go 版（k1s0-validation）

### 主要な型

```go
// FieldErrorKind はフィールドエラーの種類。
type FieldErrorKind string

const (
    Required      FieldErrorKind = "required"
    InvalidFormat FieldErrorKind = "invalid_format"
    MinLength     FieldErrorKind = "min_length"
    MaxLength     FieldErrorKind = "max_length"
)

// FieldError は個別フィールドのバリデーションエラー。
type FieldError struct {
    Field   string
    Kind    FieldErrorKind
    Message string
}

// ValidationErrors はバリデーションエラーの集合。
type ValidationErrors struct {
    Errors map[string][]FieldError
}

func (v *ValidationErrors) AddFieldError(err FieldError)
func (v *ValidationErrors) IsEmpty() bool
func (v *ValidationErrors) ToProblemDetails(instance, title string) *ProblemDetails
func (v *ValidationErrors) ToGRPCDetails() *GRPCErrorDetails

// Validatable はバリデーション可能な型のインターフェース。
type Validatable interface {
    Validate() *ValidationErrors
}
```

### 使用例

```go
import k1s0val "github.com/k1s0/framework/backend/go/k1s0-validation"

type CreateUserRequest struct {
    Name     string
    Email    string
    Password string
}

func (r *CreateUserRequest) Validate() *k1s0val.ValidationErrors {
    errs := &k1s0val.ValidationErrors{}
    if r.Name == "" {
        errs.AddFieldError(k1s0val.FieldError{Field: "name", Kind: k1s0val.Required})
    }
    if len(r.Password) < 8 {
        errs.AddFieldError(k1s0val.FieldError{Field: "password", Kind: k1s0val.MinLength, Message: "8文字以上"})
    }
    if errs.IsEmpty() {
        return nil
    }
    return errs
}
```

## C# 版（K1s0.Validation）

### 主要な型

```csharp
public enum FieldErrorKind
{
    Required, InvalidFormat, MinLength, MaxLength, MinValue, MaxValue, Pattern, Custom
}

public record FieldError(string Field, FieldErrorKind Kind, string Message)
{
    public static FieldError Required(string field);
    public static FieldError InvalidFormat(string field, string message);
    public static FieldError MinLength(string field, int min);
}

public class ValidationErrors
{
    public void AddFieldError(FieldError error);
    public bool IsEmpty { get; }
    public ProblemDetails ToProblemDetails(string instance, string title);
    public GrpcErrorDetails ToGrpcDetails();
}

public interface IValidatable
{
    ValidationErrors? Validate();
}
```

### 使用例

```csharp
using K1s0.Validation;

public record CreateUserRequest(string Name, string Email, string Password) : IValidatable
{
    public ValidationErrors? Validate()
    {
        var errors = new ValidationErrors();
        if (string.IsNullOrEmpty(Name))
            errors.AddFieldError(FieldError.Required("name"));
        if (Password.Length < 8)
            errors.AddFieldError(FieldError.MinLength("password", 8));
        return errors.IsEmpty ? null : errors;
    }
}
```

## Python 版（k1s0-validation）

Pydantic ベースのバリデーションライブラリ。

### 主要な型

```python
from pydantic import BaseModel, field_validator

class FieldError:
    field: str
    kind: str
    message: str

    @classmethod
    def required(cls, field: str) -> "FieldError": ...
    @classmethod
    def min_length(cls, field: str, min: int) -> "FieldError": ...

class ValidationErrors(Exception):
    errors: dict[str, list[FieldError]]

    def add_field_error(self, error: FieldError) -> None: ...
    def is_empty(self) -> bool: ...
    def to_problem_details(self, instance: str, title: str) -> dict: ...
    def to_grpc_details(self) -> "GrpcErrorDetails": ...
```

### 使用例

```python
from k1s0_validation import ValidationErrors, FieldError
from pydantic import BaseModel, field_validator

class CreateUserRequest(BaseModel):
    name: str
    email: str
    password: str

    @field_validator("name")
    @classmethod
    def name_required(cls, v: str) -> str:
        if not v:
            raise ValueError("name is required")
        return v

    @field_validator("password")
    @classmethod
    def password_min_length(cls, v: str) -> str:
        if len(v) < 8:
            raise ValueError("8文字以上必要です")
        return v
```

## Kotlin 版（k1s0-validation）

### 主要な型

```kotlin
enum class FieldErrorKind {
    Required, InvalidFormat, MinLength, MaxLength, MinValue, MaxValue, Pattern, Custom
}

data class FieldError(
    val field: String,
    val kind: FieldErrorKind,
    val message: String
) {
    companion object {
        fun required(field: String): FieldError
        fun invalidFormat(field: String, message: String): FieldError
        fun minLength(field: String, min: Int): FieldError
    }
}

class ValidationErrors {
    fun addFieldError(error: FieldError)
    fun isEmpty(): Boolean
    fun toProblemDetails(instance: String, title: String): ProblemDetails
    fun toGrpcDetails(): GrpcErrorDetails
}

interface Validatable {
    fun validate(): ValidationErrors?
}
```

### 使用例

```kotlin
import com.k1s0.validation.*

data class CreateUserRequest(
    val name: String,
    val email: String,
    val password: String
) : Validatable {
    override fun validate(): ValidationErrors? {
        val errors = ValidationErrors()
        if (name.isEmpty()) errors.addFieldError(FieldError.required("name"))
        if (password.length < 8) errors.addFieldError(FieldError.minLength("password", 8))
        return if (errors.isEmpty()) null else errors
    }
}
```
