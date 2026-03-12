# Building Blocks 抽象化

## 概要

Building Block（以下 BB）は、k1s0 の分散システム基盤において、外部リソースへのアクセスを統一インターフェースで抽象化する設計パターンである。Dapr のコンポーネントモデルに着想を得つつ、k1s0 ネイティブなアプローチとして再設計している。

BB の設計思想は以下の通り:

- **実装非依存**: アプリケーションコードは BB トレイト/インターフェースのみに依存し、具体的な実装（Kafka・Redis・Vault 等）を意識しない
- **テスト容易性**: 全 BB に InMemory 実装を提供し、外部依存なしのユニットテストを実現
- **宣言的構成**: `components.yaml` でサービスが利用するコンポーネントを宣言的に定義
- **既存ライブラリ活用**: 新規実装ではなく、k1s0-kafka・k1s0-cache・k1s0-vault-client 等の既存ライブラリをラップして統一 API を提供

## コンポーネントモデル

全 BB 実装は `Component` トレイト/インターフェースを満たす必要がある。`Component` はライフサイクル管理（初期化・終了）と識別情報（名前・バージョン）を統一的に提供する。

各 BB トレイト（PubSub・StateStore・SecretStore・Binding）は `Component` を継承し、それぞれの操作固有メソッドを追加定義する。

## Building Block 一覧

以下の 4 つのライブラリは、設計当初はスタンドアロンとして計画されましたが、実装では Building Block パッケージ (`bb-*` / `building-blocks/`) に統合されています。実装パスは `regions/system/library/{go,rust,typescript,dart}/building-blocks/` または `bb-*` です。

| Building Block | 用途 | 内部実装 | 統合パッケージ | 詳細設計 |
|---------------|------|---------|--------------|---------|
| PubSub | メッセージング抽象化（トピックベースの Pub/Sub） | k1s0-kafka + k1s0-messaging ラッパー、Redis Pub/Sub、InMemory | `building-blocks/pubsub.*` | [PubSub 設計](../messaging/pubsub.md) |
| StateStore | 状態管理抽象化（キーバリューストア + ETag 楽観ロック） | k1s0-cache ラッパー（Redis）、PostgreSQL、InMemory | `bb-statestore` | [StateStore 設計](../data/statestore.md) |
| SecretStore | シークレット抽象化（統一的なシークレット取得） | k1s0-vault-client ラッパー、環境変数、ファイル | `bb-secretstore` | [SecretStore 設計](../auth-security/secret-store.md) |
| Binding | リソースバインディング（外部リソースへの入出力） | PostgreSQL、S3、HTTP | `building-blocks/binding.*` | [Binding 設計](../config/binding.md) |

## 共通インターフェース

### Rust

```rust
use async_trait::async_trait;
use std::collections::HashMap;

/// コンポーネントメタデータ（components.yaml から読み込み）
pub struct Metadata {
    pub properties: HashMap<String, String>,
}

/// コンポーネントエラー
#[derive(Debug, thiserror::Error)]
pub enum ComponentError {
    #[error("initialization failed: {0}")]
    InitFailed(String),
    #[error("operation failed: {0}")]
    OperationFailed(String),
    #[error("component closed")]
    Closed,
}

/// 全 Building Block が実装すべき基底トレイト
#[async_trait]
pub trait Component: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn init(&mut self, metadata: Metadata) -> Result<(), ComponentError>;
    async fn close(&self) -> Result<(), ComponentError>;
}
```

### Go

```go
package buildingblocks

import "context"

// Metadata はコンポーネント設定を保持する。
type Metadata struct {
    Properties map[string]string
}

// Component は全 Building Block が実装すべき基底インターフェース。
type Component interface {
    Name() string
    Version() string
    Init(ctx context.Context, metadata Metadata) error
    Close(ctx context.Context) error
}
```

### TypeScript

```typescript
export interface Metadata {
  readonly properties: Record<string, string>;
}

export interface Component {
  readonly name: string;
  readonly version: string;
  init(metadata: Metadata): Promise<void>;
  close(): Promise<void>;
}

export class ComponentError extends Error {
  constructor(
    message: string,
    public readonly code: 'INIT_FAILED' | 'OPERATION_FAILED' | 'CLOSED',
  ) {
    super(message);
  }
}
```

### Dart

```dart
class Metadata {
  final Map<String, String> properties;
  const Metadata({required this.properties});
}

abstract class Component {
  String get name;
  String get version;
  Future<void> init(Metadata metadata);
  Future<void> close();
}

enum ComponentErrorCode { initFailed, operationFailed, closed }

class ComponentError implements Exception {
  final String message;
  final ComponentErrorCode code;
  const ComponentError(this.message, this.code);
}
```

## components.yaml 仕様

サービスが利用する BB コンポーネントを宣言的に定義する YAML ファイル。サービス起動時にこのファイルを読み込み、指定されたコンポーネントを初期化する。

```yaml
apiVersion: k1s0/v1
kind: Components
metadata:
  name: my-service
spec:
  pubsub:
    name: kafka
    metadata:
      brokers: "${KAFKA_BROKERS}"
      consumerGroup: "${SERVICE_NAME}"
  statestore:
    name: redis
    metadata:
      host: "${REDIS_HOST}"
  secretstore:
    name: vault
    metadata:
      address: "${VAULT_ADDR}"
  bindings:
    - name: postgres-orders
      type: postgres
      metadata:
        connectionString: "${DATABASE_URL}"
```

### フィールド定義

| フィールド | 必須 | 説明 |
|-----------|------|------|
| `apiVersion` | Yes | API バージョン（`k1s0/v1`） |
| `kind` | Yes | リソース種別（`Components`） |
| `metadata.name` | Yes | サービス名 |
| `spec.pubsub` | No | PubSub コンポーネント設定 |
| `spec.statestore` | No | StateStore コンポーネント設定 |
| `spec.secretstore` | No | SecretStore コンポーネント設定 |
| `spec.bindings` | No | Binding コンポーネント設定（複数指定可） |

### 環境変数展開

`${ENV_VAR}` 形式の値は実行時に環境変数から展開される。未定義の環境変数が参照された場合はコンポーネント初期化エラーとなる。

### テスト用構成

テスト時は InMemory コンポーネントを指定して外部依存を排除する。

```yaml
apiVersion: k1s0/v1
kind: Components
metadata:
  name: my-service-test
spec:
  pubsub:
    name: in-memory
  statestore:
    name: in-memory
  secretstore:
    name: in-memory
```

## テスト方針

全 BB に対して InMemory 実装を提供し、以下のテスト戦略を採用する。

| テストレベル | InMemory 利用 | 外部依存 | 用途 |
|-------------|--------------|---------|------|
| ユニットテスト | Yes | なし | BB トレイトを通じたビジネスロジックのテスト |
| 統合テスト | No | testcontainers | 実コンポーネント（Redis・Kafka・Vault）との結合テスト |
| コントラクトテスト | Yes | なし | BB トレイトの振る舞い仕様を全実装で検証 |

InMemory 実装は本番コードと同一の BB トレイトを実装するため、テストコードと本番コードの乖離を防止する。

---

## 関連ドキュメント

- [system-library-概要](概要.md) — ライブラリ一覧・テスト方針
- [PubSub 設計](../messaging/pubsub.md) — PubSub Building Block 詳細
- [StateStore 設計](../data/statestore.md) — StateStore Building Block 詳細
- [SecretStore 設計](../auth-security/secret-store.md) — SecretStore Building Block 詳細
- [Binding 設計](../config/binding.md) — Binding Building Block 詳細
- [Building Blocks アーキテクチャ図](../../diagrams/building-blocks-architecture.drawio) — BB 全体構成図
- [コンポーネントモデル図](../../diagrams/building-blocks-component-model.drawio) — コンポーネント継承関係図
- [system-library-messaging設計](../messaging/messaging.md) — Kafka メッセージングライブラリ
- [system-library-cache設計](../data/cache.md) — Redis キャッシュライブラリ
- [system-library-vault-client設計](../auth-security/vault-client.md) — Vault クライアントライブラリ
