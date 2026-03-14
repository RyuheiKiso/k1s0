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
default = []
http = ["reqwest"]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
k1s0-bb-core = { path = "../bb-core" }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
tokio = { version = "1", features = ["sync"] }
tracing = "0.1"
mockall = { version = "0.13", optional = true }
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
│   ├── traits.rs       # InputBinding・OutputBinding トレイト定義・BindingRequest・BindingResponse
│   ├── error.rs        # BindingError
│   ├── http.rs         # HttpOutputBinding（feature = "http"）
│   └── memory.rs       # InMemoryInputBinding・InMemoryOutputBinding
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
| HttpBinding | `GET`・`POST`・`PUT`・`DELETE`・`PATCH` | No | Yes |

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

**配置先**: `regions/system/library/go/building-blocks/`

**主要インターフェース**:

```go
package binding

import "context"

// BindingData は InputBinding の受信データを表す。
type BindingData struct {
    Data     []byte
    Metadata map[string]string
}

// BindingResponse は OutputBinding の応答を表す。
type BindingResponse struct {
    Data     []byte
    Metadata map[string]string
}

// InputBinding は外部リソースからのイベント受信インターフェース。
type InputBinding interface {
    buildingblocks.Component
    Read(ctx context.Context) (*BindingData, error)
}

// OutputBinding は外部リソースへの操作実行インターフェース。
type OutputBinding interface {
    buildingblocks.Component
    Invoke(ctx context.Context, operation string, data []byte, metadata map[string]string) (*BindingResponse, error)
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

// HTTPOutputBinding: net/http を使用。operation = HTTP メソッド。metadata["url"] 必須。
type HTTPOutputBinding struct{}
func NewHTTPOutputBinding(client *http.Client) *HTTPOutputBinding

// FileOutputBinding: FileClientIface 経由で注入（k1s0-file-client 互換）。
// operations: "upload-url"・"download-url"・"delete"・"list"・"copy"
type FileOutputBinding struct{}
func NewFileOutputBinding(name string, client FileClientIface) *FileOutputBinding

// InMemoryOutputBinding: 呼び出し履歴を記録するテスト用 OutputBinding。
// LastInvocation() / SetResponse() / Reset() ヘルパーを提供する。
type InMemoryOutputBinding struct{}
func NewInMemoryOutputBinding() *InMemoryOutputBinding

// InMemoryInputBinding: FIFO キューからデータを読み取るテスト用 InputBinding。
// Push() ヘルパーでキューにデータを追加する。
type InMemoryInputBinding struct{}
func NewInMemoryInputBinding() *InMemoryInputBinding
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/building-blocks/`

> **現在の実装**: `InMemoryInputBinding`・`InMemoryOutputBinding` のみ。本番バックエンド（HTTP・File）は Go/Rust 側で提供する。

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

export class InMemoryInputBinding implements InputBinding { /* テスト・開発用（push() ヘルパー付き） */ }
export class InMemoryOutputBinding implements OutputBinding { /* テスト・開発用（lastInvocation() / setResponse() / reset() 付き） */ }
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/building-blocks/`

> **現在の実装**: `InMemoryInputBinding`・`InMemoryOutputBinding` のみ。本番バックエンドは Go/Rust 側で提供する。

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

class InMemoryInputBinding implements InputBinding { /* テスト・開発用（push() ヘルパー付き） */ }
class InMemoryOutputBinding implements OutputBinding { /* テスト・開発用（lastInvocation() / setResponse() / reset() 付き） */ }
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
