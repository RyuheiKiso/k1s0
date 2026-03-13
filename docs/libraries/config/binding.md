# k1s0-binding ライブラリ設計

> **実装形態**: Building Block 統合
> 本ライブラリは設計当初スタンドアロンとして計画されましたが、実装では Building Block パッケージ (`bb-*` / `building-blocks/`) に統合されています。
> 実装パス: `regions/system/library/{go,rust,typescript,dart}/building-blocks/` または `bb-*`

## 概要

Binding Building Block ライブラリ。外部リソースへの入出力を統一インターフェースで抽象化する。InputBinding（外部からのイベント受信）と OutputBinding（外部への操作実行）の 2 つのトレイトを提供し、PostgreSQL・S3・HTTP 等のリソースを同一の API で扱える。

**配置先**: `regions/system/library/rust/bb-binding/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `InputBinding` | トレイト | 入力バインディング（外部リソースからのイベント受信、Component 継承） |
| `OutputBinding` | トレイト | 出力バインディング（外部リソースへの操作実行、Component 継承） |
| `BindingRequest` | 構造体 | バインディングリクエスト（operation・data・metadata） |
| `BindingResponse` | 構造体 | バインディングレスポンス（data・metadata） |
| `BindingError` | enum | `OperationFailed`・`ConnectionFailed`・`UnsupportedOperation`・`TimeoutError` |
| `PostgresBinding` | 構造体 | PostgreSQL 入出力バインディング |
| `S3Binding` | 構造体 | S3 出力バインディング |
| `HttpBinding` | 構造体 | HTTP 出力バインディング |
| `InMemoryBinding` | 構造体 | InMemory 実装（テスト用、入出力両対応） |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "bb-binding"
version = "0.1.0"
edition = "2021"

[features]
postgres = ["sqlx"]
s3 = ["aws-sdk-s3"]
http = ["reqwest"]
mock = []

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"], optional = true }
aws-sdk-s3 = { version = "1", optional = true }
reqwest = { version = "0.12", features = ["json"], optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `bb-binding = { path = "../../system/library/rust/bb-binding" }`

**モジュール構成**:

```
bb-binding/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── traits.rs       # InputBinding・OutputBinding トレイト定義
│   ├── request.rs      # BindingRequest・BindingResponse
│   ├── error.rs        # BindingError
│   ├── postgres.rs     # PostgresBinding（feature = "postgres"）
│   ├── s3.rs           # S3Binding（feature = "s3"）
│   ├── http.rs         # HttpBinding（feature = "http"）
│   └── in_memory.rs    # InMemoryBinding
└── Cargo.toml
```

**InputBinding / OutputBinding トレイト**:

```rust
use async_trait::async_trait;
use tokio_stream::Stream;
use std::pin::Pin;

/// 外部リソースからのイベント受信。
#[async_trait]
pub trait InputBinding: Component + Send + Sync {
    /// 外部リソースからのイベントストリームを返す。
    async fn read(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<BindingRequest, BindingError>> + Send>>, BindingError>;

    /// リソースを解放する。
    async fn close(&self) -> Result<(), BindingError>;
}

/// 外部リソースへの操作実行。
#[async_trait]
pub trait OutputBinding: Component + Send + Sync {
    /// 指定オペレーションを実行する。
    async fn invoke(
        &self,
        request: &BindingRequest,
    ) -> Result<BindingResponse, BindingError>;

    /// サポートするオペレーション一覧を返す。
    fn operations(&self) -> &[&str];

    /// リソースを解放する。
    async fn close(&self) -> Result<(), BindingError>;
}
```

**BindingRequest / BindingResponse**:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BindingRequest {
    pub operation: String,
    pub data: serde_json::Value,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BindingResponse {
    pub data: serde_json::Value,
    pub metadata: std::collections::HashMap<String, String>,
}
```

**各バインディングのサポートオペレーション**:

| バインディング | サポートオペレーション | 入力 | 出力 |
|--------------|---------------------|------|------|
| PostgresBinding | `query`・`exec`・`create`・`close` | Yes（CDC イベント） | Yes |
| S3Binding | `get`・`put`・`delete`・`list` | No | Yes |
| HttpBinding | `get`・`post`・`put`・`delete` | No | Yes |

**使用例**:

```rust
use k1s0_binding::{OutputBinding, BindingRequest, InMemoryBinding};

let binding = InMemoryBinding::new();

// クエリ実行
let request = BindingRequest {
    operation: "query".to_string(),
    data: serde_json::json!({"sql": "SELECT * FROM orders WHERE id = $1", "params": ["ORD-001"]}),
    metadata: Default::default(),
};
let response = binding.invoke(&request).await?;
println!("result: {:?}", response.data);

// S3 アップロード
let request = BindingRequest {
    operation: "put".to_string(),
    data: serde_json::json!({"key": "uploads/file.pdf", "content": "base64data..."}),
    metadata: [("bucket".to_string(), "my-bucket".to_string())].into(),
};
let response = binding.invoke(&request).await?;
```

## Go 実装

**配置先**: `regions/system/library/go/binding/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要インターフェース**:

```go
package binding

import "context"

// InputBinding は外部リソースからのイベント受信インターフェース。
type InputBinding interface {
    buildingblocks.Component
    Read(ctx context.Context) (<-chan *BindingRequest, error)
    Close(ctx context.Context) error
}

// OutputBinding は外部リソースへの操作実行インターフェース。
type OutputBinding interface {
    buildingblocks.Component
    Invoke(ctx context.Context, request *BindingRequest) (*BindingResponse, error)
    Operations() []string
    Close(ctx context.Context) error
}

type BindingRequest struct {
    Operation string            `json:"operation"`
    Data      []byte            `json:"data"`
    Metadata  map[string]string `json:"metadata"`
}

type BindingResponse struct {
    Data     []byte            `json:"data"`
    Metadata map[string]string `json:"metadata"`
}

type ErrorKind int

const (
    OperationFailed ErrorKind = iota
    ConnectionFailed
    UnsupportedOperation
    TimeoutError
)

type BindingError struct {
    Kind    ErrorKind
    Message string
    Err     error
}

func (e *BindingError) Error() string
func (e *BindingError) Unwrap() error

// --- 実装 ---

type PostgresBinding struct { /* PostgreSQL */ }
func NewPostgresBinding() *PostgresBinding

type S3Binding struct { /* S3 */ }
func NewS3Binding() *S3Binding

type HttpBinding struct { /* HTTP */ }
func NewHttpBinding() *HttpBinding

type InMemoryBinding struct { /* テスト用 */ }
func NewInMemoryBinding() *InMemoryBinding
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/binding/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
import { Component } from '@k1s0/building-blocks';

export interface BindingRequest {
  readonly operation: string;
  readonly data: unknown;
  readonly metadata: Record<string, string>;
}

export interface BindingResponse {
  readonly data: unknown;
  readonly metadata: Record<string, string>;
}

export type BindingErrorCode =
  | 'OPERATION_FAILED'
  | 'CONNECTION_FAILED'
  | 'UNSUPPORTED_OPERATION'
  | 'TIMEOUT_ERROR';

export class BindingError extends Error {
  constructor(
    message: string,
    public readonly code: BindingErrorCode,
  ) {
    super(message);
  }
}

export interface InputBinding extends Component {
  read(handler: (request: BindingRequest) => Promise<void>): Promise<void>;
  close(): Promise<void>;
}

export interface OutputBinding extends Component {
  invoke(request: BindingRequest): Promise<BindingResponse>;
  operations(): string[];
  close(): Promise<void>;
}

export class PostgresBinding implements InputBinding, OutputBinding { /* ... */ }
export class S3Binding implements OutputBinding { /* ... */ }
export class HttpBinding implements OutputBinding { /* ... */ }
export class InMemoryBinding implements InputBinding, OutputBinding { /* ... */ }
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/binding/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```dart
import 'package:k1s0_building_blocks/component.dart';

class BindingRequest {
  final String operation;
  final dynamic data;
  final Map<String, String> metadata;
  const BindingRequest({
    required this.operation,
    required this.data,
    this.metadata = const {},
  });
}

class BindingResponse {
  final dynamic data;
  final Map<String, String> metadata;
  const BindingResponse({required this.data, this.metadata = const {}});
}

enum BindingErrorCode {
  operationFailed,
  connectionFailed,
  unsupportedOperation,
  timeoutError,
}

class BindingError implements Exception {
  final String message;
  final BindingErrorCode code;
  const BindingError(this.message, this.code);
}

abstract class InputBinding implements Component {
  Stream<BindingRequest> read();
  Future<void> close();
}

abstract class OutputBinding implements Component {
  Future<BindingResponse> invoke(BindingRequest request);
  List<String> get operations;
  Future<void> close();
}

class PostgresBinding implements InputBinding, OutputBinding { /* PostgreSQL */ }
class S3Binding implements OutputBinding { /* S3 */ }
class HttpBinding implements OutputBinding { /* HTTP */ }
class InMemoryBinding implements InputBinding, OutputBinding { /* テスト用 */ }
```

**カバレッジ目標**: 85%以上

## テスト戦略

### ユニットテスト

InMemoryBinding を活用し、各オペレーションの振る舞いを検証する。

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_invoke_operation() {
        let binding = InMemoryBinding::new();
        let request = BindingRequest {
            operation: "query".to_string(),
            data: serde_json::json!({"sql": "SELECT 1"}),
            metadata: Default::default(),
        };
        let response = binding.invoke(&request).await.unwrap();
        assert!(!response.data.is_null());
    }

    #[tokio::test]
    async fn test_unsupported_operation() {
        let binding = InMemoryBinding::new();
        let request = BindingRequest {
            operation: "unsupported".to_string(),
            data: serde_json::json!({}),
            metadata: Default::default(),
        };
        let result = binding.invoke(&request).await;
        assert!(matches!(result, Err(BindingError::UnsupportedOperation { .. })));
    }

    #[tokio::test]
    async fn test_operations_list() {
        let postgres = PostgresBinding::new();
        assert!(postgres.operations().contains(&"query"));
        assert!(postgres.operations().contains(&"exec"));

        let s3 = S3Binding::new();
        assert!(s3.operations().contains(&"put"));
        assert!(s3.operations().contains(&"get"));
    }
}
```

### 統合テスト

- testcontainers で PostgreSQL・MinIO（S3 互換）を起動し、実装の動作を検証
- 大容量ファイルアップロード・ダウンロードのパフォーマンスを計測
- HTTP バインディングの各メソッド（GET/POST/PUT/DELETE）を wiremock で検証

### コントラクトテスト

全 OutputBinding 実装が `invoke` メソッドの共通振る舞い（サポート外オペレーションで `UnsupportedOperation` を返す等）を満たすことを検証する。

**カバレッジ目標**: 85%以上

---

## 関連ドキュメント

- [Building Blocks 概要](../_common/building-blocks.md) — BB 設計思想・共通インターフェース
- [system-library-file-client設計](../client-sdk/file-client.md) — ファイルストレージクライアント
- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
