# k1s0-pagination ライブラリ設計

## 概要

共通ページネーション実装ライブラリ。オフセットベース（page/page_size）とカーソルベース（cursor/limit）の両方式をサポートし、API 設計の統一レスポンス（D-007）に準拠した `PaginatedResponse` を提供する。各 Tier のサーバーが一貫したページネーション形式を返せるよう、多言語で統一実装する。

**配置先**: `regions/system/library/rust/pagination/`

## 公開 API

| 型・関数 | 種別 | 説明 |
|---------|------|------|
| `PageRequest` | 構造体 | page・per_page |
| `PageResponse<T>` | 構造体 | items・total・page・per_page・total_pages |
| `PaginationMeta` | 構造体 | オフセットページネーションのメタデータ（total・page・per_page・total_pages） |
| `CursorRequest` | 構造体 | カーソルベースのリクエスト（cursor?・limit） |
| `CursorMeta` | 構造体 | カーソルベースのレスポンスメタ（next_cursor?・has_more） |
| `encode_cursor(sort_key, id)` | 関数 | sort_key と id を結合して Base64 エンコード |
| `decode_cursor(cursor)` | 関数 | カーソルを (sort_key, id) のタプルに復元 |
| `validate_per_page(per_page)` | 関数 | per_page が 1〜100 であることを検証（範囲外はエラー） |
| `default_page_request()` | 関数 | デフォルト値（page: 1, per_page: 20）の PageRequest を返す |
| `offset()` | メソッド | ページネーションのオフセット値を返す（`(page - 1) * per_page`） |
| `has_next(total)` | メソッド | 次のページが存在するかを返す（`page * per_page < total`） |
| `PaginationError` | enum | `InvalidCursor`・`InvalidPerPage` |

> **注意: Base64 エンコーディング方式の差異**
>
> Dart 実装のみ `base64Url`（URL-safe Base64）を使用しており、他の3言語（Rust: `base64::STANDARD`、Go: `base64.StdEncoding`、TypeScript: `btoa`）は標準 Base64 を使用している。そのため、**Dart で生成したカーソルを他言語でデコードする場合（またはその逆）は互換性がない可能性がある**。言語間でカーソルを受け渡す場合は、この差異に注意すること。

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-pagination"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
base64 = "0.22"
thiserror = "2"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

**依存追加**: `k1s0-pagination = { path = "../../system/library/rust/pagination" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
pagination/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── page.rs         # PageRequest・PageResponse
│   ├── cursor.rs       # encode_cursor・decode_cursor
│   └── error.rs        # PaginationError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_pagination::{PageRequest, PageResponse, encode_cursor, decode_cursor, validate_per_page};

// per_page バリデーション（1〜100 の範囲）
validate_per_page(20)?;

// オフセットベースページネーション
let req = PageRequest { page: 2, per_page: 20 };

let users: Vec<User> = db.fetch_users(req.page, req.per_page).await?;
let total = db.count_users().await?;

let response: PageResponse<User> = PageResponse::new(users, total, &req);
// response.total_pages は自動計算
let meta = response.meta(); // PaginationMeta

// カーソルベースページネーション（sort_key + id の 2 引数）
let cursor = encode_cursor("2024-01-15T10:30:00Z", "some-record-id");
let (sort_key, id) = decode_cursor(&cursor)?;
```

## Go 実装

**配置先**: `regions/system/library/go/pagination/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: （標準ライブラリのみ、外部依存なし）

**主要インターフェース**:

```go
type PageRequest struct {
    Page    uint32
    PerPage uint32
}

func NewPageRequest(page, perPage uint32) PageRequest
func DefaultPageRequest() PageRequest                  // page: 1, perPage: 20
func (r PageRequest) Offset() uint64                   // (Page-1) * PerPage
func (r PageRequest) HasNext(total uint64) bool        // Page * PerPage < total

type PageResponse[T any] struct {
    Items      []T
    Total      uint64
    Page       uint32
    PerPage    uint32
    TotalPages uint32
}

// 注意: Go では PageResponse に Meta() メソッドは存在しない。
// 各フィールド（Total, Page, PerPage, TotalPages）へ直接アクセスして使用する。
func NewPageResponse[T any](items []T, total uint64, req PageRequest) PageResponse[T]

type PaginationMeta struct {
    Total      uint64
    Page       uint32
    PerPage    uint32
    TotalPages uint32
}

type CursorRequest struct {
    Cursor *string
    Limit  uint32
}

type CursorMeta struct {
    NextCursor *string
    HasMore    bool
}

func ValidatePerPage(perPage uint32) error
func EncodeCursor(sortKey, id string) string
func DecodeCursor(cursor string) (sortKey string, id string, err error)
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/pagination/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export interface PageRequest {
  page: number;
  perPage: number;
}

export interface PageResponse<T> {
  items: T[];
  total: number;
  page: number;
  perPage: number;
  totalPages: number;
}

export interface PaginationMeta {
  total: number;
  page: number;
  perPage: number;
  totalPages: number;
}

export interface CursorRequest {
  cursor?: string;
  limit: number;
}

export interface CursorMeta {
  nextCursor?: string;
  hasMore: boolean;
}

export class PerPageValidationError extends Error {
  constructor(value: number);
}

// 注意: TypeScript では PageResponse に meta() 関数は存在しない。
// PageResponse の各フィールド（total, page, perPage, totalPages）へ直接アクセスして使用する。
export function createPageResponse<T>(items: T[], total: number, req: PageRequest): PageResponse<T>;
export function validatePerPage(perPage: number): number;  // 範囲外は PerPageValidationError をスロー
export function defaultPageRequest(): PageRequest;         // page: 1, perPage: 20
export function pageOffset(req: PageRequest): number;      // (page - 1) * perPage
export function hasNextPage(req: PageRequest, total: number): boolean;  // page * perPage < total

export function encodeCursor(sortKey: string, id: string): string;
export function decodeCursor(cursor: string): { sortKey: string; id: string };
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/pagination/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```dart
class PageRequest {
  final int page;
  final int perPage;

  factory PageRequest.defaultRequest() => PageRequest(page: 1, perPage: 20);
  int get offset => (page - 1) * perPage;
  bool hasNext(int total) => page * perPage < total;
}

class PaginationMeta {
  final int total;
  final int page;
  final int perPage;
  final int totalPages;
}

class PageResponse<T> {
  final List<T> items;
  final int total;
  final int page;
  final int perPage;
  final int totalPages;

  factory PageResponse.create(List<T> items, int total, PageRequest req);
  PaginationMeta get meta;
}

class CursorRequest {
  final String? cursor;
  final int limit;
}

class CursorMeta {
  final String? nextCursor;
  final bool hasMore;
}

int validatePerPage(int perPage);  // 範囲外は PerPageValidationException をスロー
String encodeCursor(String sortKey, String id);
({String sortKey, String id}) decodeCursor(String cursor);
```

**カバレッジ目標**: 90%以上

## テスト戦略

### ユニットテスト（`#[cfg(test)]`）

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_response_new() {
        let req = PageRequest { page: 0, per_page: 10 };
        let response = PageResponse::new(vec![1, 2, 3], 25, &req);
        assert_eq!(response.total, 25);
        assert_eq!(response.total_pages, 3);
    }

    #[test]
    fn test_cursor_encode_decode_roundtrip() {
        let sort_key = "2026-02-23T10:00:00Z";
        let id = "user-123";
        let encoded = encode_cursor(sort_key, id);
        let (decoded_key, decoded_id) = decode_cursor(&encoded).unwrap();
        assert_eq!(decoded_key, sort_key);
        assert_eq!(decoded_id, id);
    }

    #[test]
    fn test_invalid_cursor_returns_error() {
        let result = decode_cursor("not-valid-base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_per_page_valid() {
        assert!(validate_per_page(1).is_ok());
        assert!(validate_per_page(100).is_ok());
    }

    #[test]
    fn test_validate_per_page_over_max() {
        assert!(validate_per_page(101).is_err()); // 最大 100、超過はバリデーションエラー
    }
}
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [REST-API設計](../../architecture/api/REST-API設計.md) — D-007 統一レスポンス形式定義
- [API設計](../../architecture/api/API設計.md) — ページネーション方針（オフセット vs カーソル選択基準）
- [system-library-cache設計](cache.md) — ページネーション結果のキャッシュ
- [system-library-serviceauth設計](../auth-security/serviceauth.md) — ページネーション API への認証統合
