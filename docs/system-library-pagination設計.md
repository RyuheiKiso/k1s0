# k1s0-pagination ライブラリ設計

## 概要

共通ページネーション実装ライブラリ。オフセットベース（page/page_size）とカーソルベース（cursor/limit）の両方式をサポートし、API 設計の統一レスポンス（D-007）に準拠した `PaginatedResponse` を提供する。各 Tier のサーバーが一貫したページネーション形式を返せるよう、多言語で統一実装する。

**配置先**: `regions/system/library/rust/pagination/`

## 公開 API

| 型・関数 | 種別 | 説明 |
|---------|------|------|
| `PageRequest` | 構造体 | page・per_page |
| `PageResponse<T>` | 構造体 | items・total・page・per_page・total_pages |
| `encode_cursor` | 関数 | ID を Base64 エンコード |
| `decode_cursor` | 関数 | カーソル文字列を復元（単一文字列） |
| `PaginationError` | enum | `InvalidCursor` |

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

**Cargo.toml への追加行**:

```toml
k1s0-pagination = { path = "../../system/library/rust/pagination" }
```

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
use k1s0_pagination::{PageRequest, PageResponse, encode_cursor, decode_cursor};

// オフセットベースページネーション
let req = PageRequest { page: 2, per_page: 20 };

let users: Vec<User> = db.fetch_users(req.page, req.per_page).await?;
let total = db.count_users().await?;

let response: PageResponse<User> = PageResponse::new(users, total, &req);
// response.total_pages は自動計算

// カーソルベースページネーション
let cursor = encode_cursor("last-record-id");
let decoded_id = decode_cursor(&cursor)?;
```

## Go 実装

**配置先**: `regions/system/library/go/pagination/`

```
pagination/
├── pagination.go
├── pagination_test.go
├── go.mod
└── go.sum
```

**依存関係**: （標準ライブラリのみ、外部依存なし）

**主要インターフェース**:

```go
type PageRequest struct {
    Page    uint32
    PerPage uint32
}

type PageResponse[T any] struct {
    Items      []T
    Total      uint64
    Page       uint32
    PerPage    uint32
    TotalPages uint32
}

func NewPageResponse[T any](items []T, total uint64, req PageRequest) PageResponse[T]

func EncodeCursor(id string) string
func DecodeCursor(cursor string) (string, error)
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/pagination/`

```
pagination/
├── package.json        # "@k1s0/pagination", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # PageRequest, CursorRequest, PaginatedResponse, PaginationMeta, CursorMeta, encodeCursor, decodeCursor
└── __tests__/
    ├── page.test.ts
    └── cursor.test.ts
```

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

export function createPageResponse<T>(items: T[], total: number, req: PageRequest): PageResponse<T>;

export function encodeCursor(id: string): string;
export function decodeCursor(cursor: string): string;
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/pagination/`

```
pagination/
├── pubspec.yaml        # k1s0_pagination
├── analysis_options.yaml
├── lib/
│   ├── pagination.dart
│   └── src/
│       ├── page.dart           # PageRequest, PaginatedResponse, PaginationMeta
│       ├── cursor.dart         # CursorRequest, CursorMeta, encodeCursor, decodeCursor
│       └── error.dart          # PaginationError
└── test/
    ├── page_test.dart
    └── cursor_test.dart
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/pagination/`

```
pagination/
├── src/
│   ├── Pagination.csproj
│   ├── PageRequest.cs              # ページリクエスト（バリデーション付き）
│   ├── CursorRequest.cs            # カーソルリクエスト
│   ├── PaginatedResponse.cs        # ページネーションレスポンス型
│   ├── PaginationMeta.cs           # オフセット方式メタデータ
│   ├── CursorMeta.cs               # カーソル方式メタデータ
│   ├── CursorEncoder.cs            # カーソルエンコード・デコード
│   └── PaginationException.cs      # 公開例外型
├── tests/
│   ├── Pagination.Tests.csproj
│   ├── Unit/
│   │   ├── PageRequestTests.cs
│   │   ├── CursorRequestTests.cs
│   │   └── CursorEncoderTests.cs
│   └── Integration/
│       └── PaginationIntegrationTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| （標準ライブラリのみ、外部依存なし） | Convert.ToBase64String / Convert.FromBase64String 使用 |

**名前空間**: `K1s0.System.Pagination`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `PageRequest` | record | page（>=1）・page_size（1-200）バリデーション付き |
| `CursorRequest` | record | cursor（opaque token）・limit（1-100）|
| `PaginatedResponse<T>` | record | items + PaginationMeta |
| `PaginationMeta` | record | total_count・page・page_size・has_next |
| `CursorMeta` | record | next_cursor・has_next |
| `CursorEncoder` | static class | エンコード・デコード関数 |
| `PaginationException` | class | バリデーションエラーの公開例外型 |

**主要 API**:

```csharp
namespace K1s0.System.Pagination;

public record PageRequest
{
    public int Page { get; init; } = 1;
    public int PageSize { get; init; } = 20;

    public int Offset => (Page - 1) * PageSize;
    public bool HasNext(long totalCount) => (long)Page * PageSize < totalCount;

    public static PageRequest From(int? page, int? pageSize) =>
        new() { Page = page ?? 1, PageSize = Math.Clamp(pageSize ?? 20, 1, 200) };
}

public record CursorRequest(string? Cursor, int Limit = 20)
{
    public int ClampedLimit => Math.Clamp(Limit, 1, 100);
}

public record PaginationMeta(
    long TotalCount,
    int Page,
    int PageSize,
    bool HasNext);

public record CursorMeta(string? NextCursor, bool HasNext);

public record PaginatedResponse<T>(IReadOnlyList<T> Items, PaginationMeta Meta);

public record CursorResponse<T>(IReadOnlyList<T> Items, CursorMeta Meta);

public static class CursorEncoder
{
    public static string Encode(string sortKey, string id);
    public static (string SortKey, string Id) Decode(string cursor);
}
```

**カバレッジ目標**: 90%以上

---

## Swift

### パッケージ構成
- ターゲット: `K1s0Pagination`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開API

```swift
public struct PageRequest: Sendable {
    public let page: Int     // >= 1
    public let pageSize: Int // 1-200

    public var offset: Int { (page - 1) * pageSize }
    public func hasNext(totalCount: Int) -> Bool { page * pageSize < totalCount }

    public init(page: Int = 1, pageSize: Int = 20)
}

public struct CursorRequest: Sendable {
    public let cursor: String?
    public let limit: Int  // 1-100
    public init(cursor: String? = nil, limit: Int = 20)
}

public struct PaginationMeta: Sendable, Codable {
    public let totalCount: Int
    public let page: Int
    public let pageSize: Int
    public let hasNext: Bool
}

public struct CursorMeta: Sendable, Codable {
    public let nextCursor: String?
    public let hasNext: Bool
}

public struct PaginatedResponse<T: Sendable & Codable>: Sendable, Codable {
    public let items: [T]
    public let meta: PaginationMeta
}

public enum CursorEncoder {
    public static func encode(sortKey: String, id: String) -> String
    public static func decode(_ cursor: String) throws -> (sortKey: String, id: String)
}
```

### エラー型

```swift
public enum PaginationError: Error, Sendable {
    case invalidPage(page: Int)
    case invalidCursor(reason: String)
    case pageSizeExceeded(requested: Int, maximum: Int)
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上

---

## Python 実装

**配置先**: `regions/system/library/python/pagination/`

### パッケージ構造

```
pagination/
├── pyproject.toml
├── src/
│   └── k1s0_pagination/
│       ├── __init__.py       # 公開 API（再エクスポート）
│       ├── page.py           # PageRequest dataclass・PaginatedResponse・PaginationMeta
│       ├── cursor.py         # CursorRequest・CursorMeta・encode_cursor・decode_cursor
│       ├── exceptions.py     # PaginationError
│       └── py.typed
└── tests/
    ├── test_page.py
    └── test_cursor.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `PageRequest` | dataclass | page（>=1）・page_size（1-200）・offset プロパティ |
| `CursorRequest` | dataclass | cursor（opaque token）・limit（1-100）|
| `PaginatedResponse` | Generic dataclass | items + PaginationMeta |
| `PaginationMeta` | dataclass | total_count・page・page_size・has_next |
| `CursorMeta` | dataclass | next_cursor・has_next |
| `encode_cursor` | 関数 | ソートキー + ID を Base64 エンコード |
| `decode_cursor` | 関数 | カーソル文字列を復元 |
| `PaginationError` | Exception | バリデーションエラー基底クラス |

### 使用例

```python
from k1s0_pagination import (
    CursorMeta,
    CursorRequest,
    PageRequest,
    PaginatedResponse,
    PaginationMeta,
    decode_cursor,
    encode_cursor,
)

# オフセットベースページネーション
req = PageRequest(page=2, page_size=20)

users = await db.fetch_users(offset=req.offset, limit=req.page_size)
total_count = await db.count_users()

response: PaginatedResponse[User] = PaginatedResponse(
    items=users,
    meta=PaginationMeta(
        total_count=total_count,
        page=req.page,
        page_size=req.page_size,
        has_next=req.has_next(total_count),
    ),
)

# カーソルベースページネーション
cursor_req = CursorRequest(cursor=None, limit=50)

if cursor_req.cursor:
    sort_key, last_id = decode_cursor(cursor_req.cursor)
else:
    sort_key, last_id = None, None

users = await db.fetch_users_after(sort_key, last_id, cursor_req.limit + 1)
has_next = len(users) > cursor_req.limit
items = users[:cursor_req.limit]

next_cursor = encode_cursor(items[-1].created_at, items[-1].id) if items and has_next else None

response = PaginatedResponse(
    items=items,
    meta=CursorMeta(next_cursor=next_cursor, has_next=has_next),
)
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| （標準ライブラリのみ、外部依存なし） | — | base64 / dataclasses / typing 使用 |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 90%以上
- 実行: `pytest` / `ruff check .`

## テスト戦略

### ユニットテスト（`#[cfg(test)]`）

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_request_defaults() {
        let req = PageRequest::default();
        assert_eq!(req.page, 1);
        assert_eq!(req.page_size, 20);
        assert_eq!(req.offset(), 0);
    }

    #[test]
    fn test_page_request_offset() {
        let req = PageRequest::new().page(3).page_size(20);
        assert_eq!(req.offset(), 40);
    }

    #[test]
    fn test_has_next_true() {
        let req = PageRequest::new().page(1).page_size(10);
        assert!(req.has_next(25));  // 25 > 1 * 10
    }

    #[test]
    fn test_has_next_false() {
        let req = PageRequest::new().page(3).page_size(10);
        assert!(!req.has_next(25)); // 25 == 3 * 10 - but 3*10=30 > 25, so false
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
    fn test_page_size_clamp() {
        let req = PageRequest::new().page_size(999); // 最大 200 にクランプ
        assert_eq!(req.page_size, 200);
    }
}
```

### 統合テスト

- 実際のデータベース（testcontainers + PostgreSQL）に対してページネーションクエリを実行し、レスポンス形式が D-007 仕様に準拠していることを確認
- カーソルの encode/decode ラウンドトリップ + 次ページ取得の連続フローを検証
- ページ境界値（最終ページ・has_next=false）の正確な判定を確認

### モックテスト

```rust
use async_trait::async_trait;

struct MockUserRepository {
    users: Vec<User>,
}

#[async_trait]
impl Paginator<User> for MockUserRepository {
    async fn paginate(
        &self,
        req: &PageRequest,
    ) -> Result<PaginatedResponse<User>, PaginationError> {
        let total_count = self.users.len() as i64;
        let items = self.users
            .iter()
            .skip(req.offset() as usize)
            .take(req.page_size as usize)
            .cloned()
            .collect();

        Ok(PaginatedResponse::new(items).meta(PaginationMeta {
            total_count,
            page: req.page,
            page_size: req.page_size,
            has_next: req.has_next(total_count),
        }))
    }
}
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [REST-API設計](REST-API設計.md) — D-007 統一レスポンス形式定義
- [API設計](API設計.md) — ページネーション方針（オフセット vs カーソル選択基準）
- [system-library-cache設計](system-library-cache設計.md) — ページネーション結果のキャッシュ
- [system-library-serviceauth設計](system-library-serviceauth設計.md) — ページネーション API への認証統合
