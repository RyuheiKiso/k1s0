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
