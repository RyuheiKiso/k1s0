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

**依存追加**: `k1s0-validation = { path = "../../system/library/rust/validation" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

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

**配置先**: `regions/system/library/go/validation/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: なし（標準ライブラリのみ）

**主要インターフェース**:

```go
type Validator interface {
    ValidateEmail(email string) error
    ValidateUUID(id string) error
    ValidateURL(rawURL string) error
    ValidateTenantID(tenantID string) error
    ValidatePagination(page, perPage int) error
    ValidateDateRange(startDate, endDate time.Time) error
}

type ValidationError struct {
    Field   string
    Message string
    Code    string
}

type ValidationErrors struct {}

func NewValidationErrors() *ValidationErrors
func (ve *ValidationErrors) HasErrors() bool
func (ve *ValidationErrors) GetErrors() []*ValidationError
func (ve *ValidationErrors) Add(err *ValidationError)

type DefaultValidator struct{}

func NewDefaultValidator() *DefaultValidator
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/validation/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

**配置先**: `regions/system/library/dart/validation/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**カバレッジ目標**: 90%以上

## テスト戦略

| テスト種別 | 対象 | ツール |
|-----------|------|--------|
| ユニットテスト（`#[cfg(test)]`） | 各バリデーション関数の正常系・異常系（境界値・無効形式・空文字・特殊文字） | `#[test]` |
| パラメトリックテスト | メールアドレス・UUID・URL の多様な入力パターン一括検証 | rstest |
| 結合テスト | `validate!` マクロ・`ValidationErrors` の複数エラー収集動作 | `#[test]` |
| モックテスト | `Validator` トレイト実装の差し替えテスト | 手動モック |

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [API設計](../../architecture/api/API設計.md) — API リクエスト検証での利用
- [system-library-idempotency設計](../resilience/idempotency.md) — リクエスト ID 検証での活用
- [proto設計](../../architecture/api/proto設計.md) — gRPC リクエスト構造体のバリデーション
