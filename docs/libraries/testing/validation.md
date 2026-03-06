# k1s0-validation ライブラリ設計

## 概要

共通バリデーションルール実装ライブラリ。メールアドレス・UUID・URL・日時範囲・ページネーション・テナント ID 等のバリデーションを多言語で統一実装する。API 境界でのリクエスト検証に利用する。

`Validator` トレイトにより独自バリデーションルールを拡張可能とし、`ValidationErrors` による複数エラーの一括収集をサポートする。`validate!` マクロにより複数フィールド検証を簡潔に記述できる。

**配置先**: `regions/system/library/rust/validation/`

## 公開 API

| 型・関数 | 種別 | 説明 |
|---------|------|------|
| `Validator` | トレイト | バリデーション実行インターフェース |
| `validate_email` | 関数 | RFC 5321 準拠メールアドレス検証 |
| `validate_uuid` | 関数 | UUID v4 形式検証 |
| `validate_url` | 関数 | HTTP/HTTPS URL 形式検証 |
| `validate_pagination` | 関数 | ページ番号・ページサイズの範囲検証（page >= 1, page_size 1-200）。TS/Dart は 1-100 |
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
func (v *DefaultValidator) ValidateEmail(email string) error
func (v *DefaultValidator) ValidateUUID(id string) error
func (v *DefaultValidator) ValidateURL(rawURL string) error
func (v *DefaultValidator) ValidateTenantID(tenantID string) error
func (v *DefaultValidator) ValidatePagination(page, perPage int) error
func (v *DefaultValidator) ValidateDateRange(startDate, endDate time.Time) error
```

> Go は standalone 関数ではなく `DefaultValidator` メソッドとして API を提供する。

## TypeScript 実装

**配置先**: `regions/system/library/typescript/validation/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export class ValidationError extends Error {
  readonly field: string;
  readonly code: string;        // 未指定時は field 名から自動生成
  constructor(field: string, message: string, code?: string);
}

export class ValidationErrors {
  hasErrors(): boolean;
  getErrors(): ReadonlyArray<ValidationError>;
  add(error: ValidationError): void;
}

// 各関数は検証失敗時に ValidationError を throw する（成功時は void）
export function validateEmail(email: string): void;
export function validateUUID(id: string): void;
export function validateURL(url: string): void;
export function validatePagination(page: number, perPage: number): void;  // perPage: 1-100
export function validateDateRange(startDate: Date, endDate: Date): void;
export function validateTenantId(tenantId: string): void;
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/validation/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**カバレッジ目標**: 90%以上

## 言語別 API パラダイムの差異

| 言語 | パラダイム | 例 |
|------|-----------|-----|
| Go | interface + method（`DefaultValidator` 構造体のメソッド） | `v.ValidateEmail(email)` |
| Rust | standalone 関数 + `Validator` トレイト（拡張用） | `validate_email("email", value)?` |
| TypeScript | standalone 関数（失敗時 throw） | `validateEmail(email)` |
| Dart | （TypeScript と同等、失敗時 throw） | `validateEmail(email)` |

> Go のみ `Validator` インターフェースのメソッドとして API を提供する（`NewDefaultValidator()` でインスタンスを生成）。Rust は `(field, value)` 引数で `Result` を返す standalone 関数。TypeScript・Dart は `(value)` 引数で失敗時に `ValidationError` を throw する standalone 関数。Rust の `Validator` トレイトはカスタムルール拡張用であり、組み込みバリデーション関数とは独立している。
>
> **pagination 上限の言語差異**: Go/Rust は `per_page` 上限 200、TypeScript/Dart は上限 100。

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
