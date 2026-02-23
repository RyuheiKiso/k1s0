# k1s0-validation ライブラリ設計

## 概要

共通バリデーションルール実装ライブラリ。メールアドレス・UUID・URL・日時範囲・ページネーション・テナント ID 等のバリデーションを多言語で統一実装する。API 境界でのリクエスト検証に利用する。

`Validator` トレイトにより独自バリデーションルールを拡張可能とし、`ValidationErrors` による複数エラーの一括収集をサポートする。`validate!` マクロによる簡潔な複数フィールド検証と、serde との連携で gRPC / REST リクエスト構造体への直接適用が可能。

**配置先**: `regions/system/library/rust/validation/`

## 公開 API

| 型・関数 | 種別 | 説明 |
|---------|------|------|
| `Validator` | トレイト | バリデーション実行インターフェース |
| `validate_email` | 関数 | RFC 5321 準拠メールアドレス検証 |
| `validate_uuid` | 関数 | UUID v4 形式検証 |
| `validate_url` | 関数 | HTTP/HTTPS URL 形式検証 |
| `validate_pagination` | 関数 | ページ番号・ページサイズの範囲検証（page >= 1, page_size 1-200） |
| `validate_date_range` | 関数 | 日時範囲（from <= to）検証 |
| `validate_tenant_id` | 関数 | テナント ID 形式検証 |
| `ValidationError` | 構造体 | フィールド名・エラーコード・メッセージ |
| `ValidationErrors` | 構造体 | 複数 ValidationError のコレクション |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-validation"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror = "2"
serde = { version = "1", features = ["derive"] }
regex = "1"
url = "2"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**Cargo.toml への追加行**:

```toml
k1s0-validation = { path = "../../system/library/rust/validation" }
```

**モジュール構成**:

```
validation/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── rules.rs        # 各バリデーション関数
│   ├── error.rs        # ValidationError・ValidationErrors
│   └── macros.rs       # validate! マクロ
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_validation::{
    validate_email, validate_uuid, validate_pagination, validate_date_range,
    ValidationErrors,
};
use chrono::Utc;

// 単一フィールドのバリデーション
validate_email("email", "user@example.com")?;
validate_uuid("user_id", "550e8400-e29b-41d4-a716-446655440000")?;

// 複数フィールドの一括バリデーション（validate! マクロ）
let mut errors = ValidationErrors::new();
validate!(errors,
    validate_email("email", &req.email),
    validate_uuid("tenant_id", &req.tenant_id),
    validate_pagination("page", req.page, req.page_size),
);
if !errors.is_empty() {
    return Err(errors.into());
}

// 日時範囲バリデーション
let from = Utc::now() - chrono::Duration::days(30);
let to = Utc::now();
validate_date_range("date_range", from, to)?;
```

## Go 実装

**配置先**: `regions/system/library/go/validation/`

```
validation/
├── validation.go
├── rules.go
├── errors.go
├── validation_test.go
├── go.mod
└── go.sum
```

**依存関係**: `github.com/google/uuid v1.6.0`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type ValidationError struct {
    Field   string
    Code    string
    Message string
}

type ValidationErrors []ValidationError

func (e ValidationErrors) Error() string

func ValidateEmail(field, value string) *ValidationError
func ValidateUUID(field, value string) *ValidationError
func ValidateURL(field, value string) *ValidationError
func ValidatePagination(field string, page, pageSize int) *ValidationError
func ValidateDateRange(field string, from, to time.Time) *ValidationError
func ValidateTenantID(field, value string) *ValidationError

// 複数バリデーションの一括実行
func Collect(validators ...func() *ValidationError) ValidationErrors
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/validation/`

```
validation/
├── package.json        # "@k1s0/validation", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # ValidationError, ValidationErrors, validate* 関数群
└── __tests__/
    └── validation.test.ts
```

**主要 API**:

```typescript
export interface ValidationError {
  field: string;
  code: string;
  message: string;
}

export class ValidationErrors extends Error {
  constructor(public readonly errors: ValidationError[]);
  get isEmpty(): boolean;
  add(error: ValidationError): void;
  throw(): never;
}

export function validateEmail(field: string, value: string): ValidationError | null;
export function validateUuid(field: string, value: string): ValidationError | null;
export function validateUrl(field: string, value: string): ValidationError | null;
export function validatePagination(field: string, page: number, pageSize: number): ValidationError | null;
export function validateDateRange(field: string, from: Date, to: Date): ValidationError | null;
export function validateTenantId(field: string, value: string): ValidationError | null;

// 複数バリデーションの一括実行
export function collect(...validators: Array<ValidationError | null>): ValidationErrors;
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/validation/`

```
validation/
├── pubspec.yaml        # k1s0_validation
├── analysis_options.yaml
├── lib/
│   ├── validation.dart
│   └── src/
│       ├── rules.dart       # 各バリデーション関数
│       ├── error.dart       # ValidationError・ValidationErrors
│       └── validator.dart   # Validator abstract
└── test/
    └── validation_test.dart
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/validation/`

```
validation/
├── src/
│   ├── Validation.csproj
│   ├── IValidator.cs               # バリデーションインターフェース
│   ├── ValidationRules.cs          # 各バリデーション静的メソッド
│   ├── ValidationError.cs          # フィールド名・エラーコード・メッセージ
│   ├── ValidationErrors.cs         # 複数 ValidationError コレクション
│   └── ValidationException.cs      # 公開例外型
├── tests/
│   ├── Validation.Tests.csproj
│   ├── Unit/
│   │   ├── ValidationRulesTests.cs
│   │   └── ValidationErrorsTests.cs
│   └── Integration/
│       └── ValidatorIntegrationTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| System.ComponentModel.Annotations | バリデーション属性との統合 |

**名前空間**: `K1s0.System.Validation`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `IValidator<T>` | interface | バリデーション実行インターフェース |
| `ValidationRules` | static class | 各バリデーション静的メソッド |
| `ValidationError` | record | フィールド名・エラーコード・メッセージ |
| `ValidationErrors` | class | 複数 ValidationError のコレクション |
| `ValidationException` | class | バリデーション失敗の例外型 |

**主要 API**:

```csharp
namespace K1s0.System.Validation;

public record ValidationError(string Field, string Code, string Message);

public class ValidationErrors : Exception
{
    public IReadOnlyList<ValidationError> Errors { get; }
    public bool IsEmpty => Errors.Count == 0;
    public void Add(ValidationError error);
    public void ThrowIfNotEmpty();
}

public static class ValidationRules
{
    public static ValidationError? ValidateEmail(string field, string value);
    public static ValidationError? ValidateUuid(string field, string value);
    public static ValidationError? ValidateUrl(string field, string value);
    public static ValidationError? ValidatePagination(string field, int page, int pageSize);
    public static ValidationError? ValidateDateRange(string field, DateTimeOffset from, DateTimeOffset to);
    public static ValidationError? ValidateTenantId(string field, string value);

    // 複数バリデーションの一括実行
    public static ValidationErrors Collect(params Func<ValidationError?>[] validators);
}
```

**カバレッジ目標**: 90%以上

---

## Swift

### パッケージ構成
- ターゲット: `K1s0Validation`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開API

```swift
public struct ValidationError: Sendable {
    public let field: String
    public let code: String
    public let message: String
}

public struct ValidationErrors: Error, Sendable {
    public private(set) var errors: [ValidationError]
    public var isEmpty: Bool { errors.isEmpty }
    public mutating func add(_ error: ValidationError)
}

public enum ValidationRules {
    public static func validateEmail(field: String, value: String) -> ValidationError?
    public static func validateUUID(field: String, value: String) -> ValidationError?
    public static func validateURL(field: String, value: String) -> ValidationError?
    public static func validatePagination(field: String, page: Int, pageSize: Int) -> ValidationError?
    public static func validateDateRange(field: String, from: Date, to: Date) -> ValidationError?
    public static func validateTenantId(field: String, value: String) -> ValidationError?

    // 複数バリデーションの一括実行
    public static func collect(_ validators: () -> ValidationError?...) -> ValidationErrors
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上

---

## Python 実装

**配置先**: `regions/system/library/python/validation/`

### パッケージ構造

```
validation/
├── pyproject.toml
├── src/
│   └── k1s0_validation/
│       ├── __init__.py       # 公開 API（再エクスポート）
│       ├── rules.py          # 各バリデーション関数
│       ├── error.py          # ValidationError・ValidationErrors
│       └── py.typed
└── tests/
    └── test_validation.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `ValidationError` | dataclass | フィールド名・エラーコード・メッセージ |
| `ValidationErrors` | Exception | 複数 ValidationError のコレクション |
| `validate_email` | 関数 | RFC 5321 準拠メールアドレス検証 |
| `validate_uuid` | 関数 | UUID v4 形式検証 |
| `validate_url` | 関数 | HTTP/HTTPS URL 形式検証 |
| `validate_pagination` | 関数 | ページ番号・ページサイズの範囲検証 |
| `validate_date_range` | 関数 | 日時範囲（from <= to）検証 |
| `validate_tenant_id` | 関数 | テナント ID 形式検証 |

### 使用例

```python
from k1s0_validation import (
    validate_email, validate_uuid, validate_pagination,
    validate_date_range, ValidationErrors,
)
from datetime import datetime, timezone

# 単一フィールドのバリデーション
error = validate_email("email", "user@example.com")
if error:
    raise ValidationErrors([error])

# 複数フィールドの一括バリデーション
errors = ValidationErrors.collect(
    validate_email("email", request.email),
    validate_uuid("tenant_id", request.tenant_id),
    validate_pagination("page", request.page, request.page_size),
)
errors.raise_if_not_empty()

# 日時範囲バリデーション
from_dt = datetime(2024, 1, 1, tzinfo=timezone.utc)
to_dt = datetime(2024, 12, 31, tzinfo=timezone.utc)
error = validate_date_range("date_range", from_dt, to_dt)
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| email-validator | >=2.2 | RFC 5321 準拠メールアドレス検証 |
| pydantic | >=2.10 | 設定バリデーション統合 |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 90%以上
- 実行: `pytest` / `ruff check .`

---

## テスト戦略

| テスト種別 | 対象 | ツール |
|-----------|------|--------|
| ユニットテスト（`#[cfg(test)]`） | 各バリデーション関数の正常系・異常系（境界値・無効形式・空文字・特殊文字） | `#[test]` |
| パラメトリックテスト | メールアドレス・UUID・URL の多様な入力パターン一括検証 | rstest |
| 結合テスト | `validate!` マクロ・`ValidationErrors` の複数エラー収集動作 | `#[test]` |
| モックテスト | `Validator` トレイト実装の差し替えテスト | 手動モック |

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [API設計](API設計.md) — API リクエスト検証での利用
- [system-library-idempotency設計](system-library-idempotency設計.md) — リクエスト ID 検証での活用
- [proto設計](proto設計.md) — gRPC リクエスト構造体のバリデーション
