# system-search-server 設計

OpenSearch 連携の全文検索サーバー。インデックス管理・全文検索クエリ・Kafka 非同期インデックスを提供。

## 概要

system tier の全文検索サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| インデックス管理 | インデックス定義の作成・更新・削除 |
| ドキュメント管理 | ドキュメントのインデックス登録・更新・削除（REST / Kafka 非同期） |
| 全文検索 | キーワード・フィルタ・ファセットによる全文検索クエリ |
| インデックス状態確認 | ドキュメント数・サイズ等のインデックスステータス取得 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| キャッシュ | moka v0.12 |
| OpenSearch クライアント | opensearch-rs |

### 配置パス

配置: `regions/system/server/rust/search/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| 検索エンジン | OpenSearch（opensearch-rs クライアント経由でアクセス） |
| 非同期インデックス | Kafka トピック `k1s0.system.search.index.requested.v1` を Consumer し非同期にインデックス登録 |
| 同期インデックス | REST POST `/api/v1/search/index` により即座にインデックス登録（`index_name` はリクエストボディで指定） |
| キャッシュ | インデックス一覧・ドキュメント数等のステータスを moka で TTL 30 秒キャッシュ |
| 認可 | インデックス管理は `sys_admin`、ドキュメント操作は `sys_operator`、検索・参照は `sys_auditor` |
| ポート | ホスト側 8094（内部 8080）、gRPC 50051 |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_SEARCH_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/indices` | インデックス作成 | `sys_admin` のみ |
| GET | `/api/v1/indices` | インデックス一覧取得 | `sys_auditor` 以上 |
| POST | `/api/v1/search/index` | ドキュメント登録（`index_name` はリクエストボディで指定） | `sys_operator` 以上 |
| DELETE | `/api/v1/search/index/:index_name/:id` | ドキュメント削除 | `sys_operator` 以上 |
| POST | `/api/v1/search` | 全文検索 | `sys_auditor` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### POST /api/v1/indices

新しいインデックスを作成する。`mapping` フィールドで任意の JSON マッピング定義を指定可能（省略時はデフォルト空オブジェクト）。

**リクエスト例**

```json
{
  "name": "k1s0-products",
  "mapping": {
    "properties": {
      "title": { "type": "text" },
      "description": { "type": "text" },
      "tenant_id": { "type": "keyword" }
    }
  }
}
```

**レスポンス例（201 Created）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "k1s0-products",
  "mapping": { "..." : "..." },
  "created_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス例（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_SEARCH_INDEX_ALREADY_EXISTS",
    "message": "index already exists: k1s0-products",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/indices

登録済みインデックスの一覧を取得する。

**レスポンス例（200 OK）**

```json
{
  "indices": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "k1s0-products",
      "mapping": { "..." : "..." },
      "created_at": "2026-02-20T10:00:00.000+00:00"
    }
  ]
}
```

#### POST /api/v1/search/index

ドキュメントをインデックスに登録する。`index_name` はリクエストボディで指定する。

**リクエスト例**

```json
{
  "id": "product-001",
  "index_name": "k1s0-products",
  "content": {
    "title": "高性能ノートPC",
    "description": "最新世代プロセッサー搭載の高性能ノートパソコン",
    "tenant_id": "tenant-abc",
    "created_at": "2026-02-20T10:00:00.000+00:00"
  }
}
```

**レスポンス例（201 Created）**

```json
{
  "id": "product-001",
  "index_name": "k1s0-products",
  "indexed_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス例（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_SEARCH_INDEX_NOT_FOUND",
    "message": "index not found: k1s0-products",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/search

指定されたクエリで全文検索を実行する。`index_name` でインデックスを指定し、`from` / `size` でページネーション制御する。

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `index_name` | String | （必須） | 検索対象インデックス名 |
| `query` | String | （必須） | 検索キーワード |
| `from` | u32 | 0 | オフセット（スキップ件数） |
| `size` | u32 | 10 | 取得件数 |

**リクエスト例**

```json
{
  "index_name": "k1s0-products",
  "query": "高性能ノートPC",
  "from": 0,
  "size": 20
}
```

**レスポンス例（200 OK）**

```json
{
  "total": 1,
  "hits": [
    {
      "id": "product-001",
      "index_name": "k1s0-products",
      "content": {
        "title": "高性能ノートPC",
        "description": "最新世代プロセッサー搭載の高性能ノートパソコン",
        "tenant_id": "tenant-abc"
      },
      "indexed_at": "2026-02-20T10:00:00.000+00:00"
    }
  ]
}
```

**レスポンス例（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_SEARCH_INDEX_NOT_FOUND",
    "message": "index not found: k1s0-products",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### DELETE /api/v1/search/index/:index_name/:id

指定インデックスから特定ドキュメントを削除する。成功時は 204 No Content（レスポンスボディなし）。

**レスポンス例（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_SEARCH_DOCUMENT_NOT_FOUND",
    "message": "document not found: product-001",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_SEARCH_INDEX_NOT_FOUND` | 404 | 指定されたインデックスが見つからない |
| `SYS_SEARCH_DOCUMENT_NOT_FOUND` | 404 | 指定されたドキュメントが見つからない |
| `SYS_SEARCH_INDEX_ALREADY_EXISTS` | 409 | 同一名のインデックスが既に存在する |
| `SYS_SEARCH_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_SEARCH_OPENSEARCH_ERROR` | 502 | OpenSearch への接続・クエリエラー |
| `SYS_SEARCH_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.search.v1;

service SearchService {
  rpc IndexDocument(IndexDocumentRequest) returns (IndexDocumentResponse);
  rpc Search(SearchRequest) returns (SearchResponse);
  rpc DeleteDocument(DeleteDocumentRequest) returns (DeleteDocumentResponse);
}

message IndexDocumentRequest {
  string index = 1;
  string document_id = 2;
  bytes document_json = 3;
}

message IndexDocumentResponse {
  string document_id = 1;
  string index = 2;
  string result = 3;
}

message SearchRequest {
  string index = 1;
  string query = 2;
  bytes filters_json = 3;
  uint32 page = 4;
  uint32 page_size = 5;
}

message SearchResponse {
  repeated SearchHit hits = 1;
  uint64 total_count = 2;
  uint32 page = 3;
  uint32 page_size = 4;
  bool has_next = 5;
}

message SearchHit {
  string id = 1;
  float score = 2;
  bytes document_json = 3;
}

message DeleteDocumentRequest {
  string index = 1;
  string document_id = 2;
}

message DeleteDocumentResponse {
  bool success = 1;
  string message = 2;
}
```

---

## Kafka メッセージング設計

### インデックス要求メッセージ

Kafka トピック `k1s0.system.search.index.requested.v1` を Consumer し、以下のフォーマットのメッセージを非同期でインデックス登録する。

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.search.index.requested.v1` |
| Consumer グループ | `search-server-consumer` |
| auto.offset.reset | `earliest` |
| max.poll.records | `100` |
| キー | インデックス名（例: `k1s0-products`） |

**メッセージ例**

```json
{
  "event_type": "INDEX_REQUESTED",
  "index": "k1s0-products",
  "document_id": "product-001",
  "operation": "upsert",
  "document": {
    "title": "高性能ノートPC",
    "description": "最新世代プロセッサー搭載の高性能ノートパソコン",
    "tenant_id": "tenant-abc",
    "created_at": "2026-02-20T10:00:00.000+00:00"
  },
  "timestamp": "2026-02-20T10:00:00.000+00:00",
  "actor_service": "product-service"
}
```

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `SearchIndex`, `SearchDocument`, `SearchQuery`, `SearchResult` | エンティティ定義 |
| domain/repository | `SearchRepository`（単一トレイト） | リポジトリトレイト（`create_index`, `find_index`, `list_indices`, `index_document`, `search`, `delete_document`） |
| domain/service | `SearchDomainService` | 検索クエリ構築・ファセット集計ロジック |
| usecase | `CreateIndexUseCase`, `ListIndicesUseCase`, `IndexDocumentUseCase`, `SearchUseCase`, `DeleteDocumentUseCase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `OpenSearchRepository` | OpenSearch リポジトリ実装 |
| infrastructure/cache | `SearchCacheService` | moka キャッシュ実装 |
| infrastructure/messaging | `SearchIndexKafkaConsumer` | Kafka Consumer（非同期インデックス要求） |

### ドメインモデル

#### SearchIndex

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | Uuid | インデックス ID（自動生成） |
| `name` | String | インデックス名（例: `k1s0-products`） |
| `mapping` | serde_json::Value | マッピング定義（任意の JSON） |
| `created_at` | DateTime\<Utc\> | 作成日時 |

#### SearchDocument

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | ドキュメント ID |
| `index_name` | String | 所属インデックス名 |
| `content` | serde_json::Value | ドキュメント本体（任意の JSON） |
| `indexed_at` | DateTime\<Utc\> | インデックス登録日時 |

#### SearchQuery

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `index_name` | String | 検索対象インデックス名 |
| `query` | String | 全文検索キーワード |
| `from` | u32 | オフセット（スキップ件数、デフォルト 0） |
| `size` | u32 | 取得件数（デフォルト 10） |
| `filters` | HashMap\<String, String\> | フィールドフィルタ |
| `facets` | Vec\<String\> | ファセット集計対象フィールド |

#### SearchResult

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `total` | u64 | ヒット件数 |
| `hits` | Vec\<SearchDocument\> | 検索結果ドキュメント一覧 |

---

## 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (search_handler.rs)         │   │
                    │  │  healthz / readyz                        │   │
                    │  │  create_index / list_indices             │   │
                    │  │  index_document                          │   │
                    │  │  delete_document_from_index / search     │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (search_grpc.rs)            │   │
                    │  │  IndexDocument / Search / DeleteDocument │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  CreateIndexUseCase / ListIndicesUseCase /      │
                    │  IndexDocumentUseCase / SearchUseCase /         │
                    │  DeleteDocumentUseCase                          │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  SearchIndex,   │              │ SearchRepository           │   │
    │  SearchDocument,│              │ (単一トレイト)              │   │
    │  SearchQuery,   │              └──────────┬─────────────────┘   │
    │  SearchResult   │                         │                     │
    └────────────────┘                         │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Config       │  │ OpenSearch             │  │
                    │  │ Loader       │  │ Repository             │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル例

### config.yaml（本番）

```yaml
app:
  name: "search"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 50051

opensearch:
  url: "https://opensearch.k1s0-system.svc.cluster.local:9200"
  username: "app"
  password: ""
  index_prefix: "k1s0-"

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"
  consumer_group: "search-server-consumer"
  topic: "k1s0.system.search.index.requested.v1"

cache:
  max_entries: 1000
  ttl_seconds: 30
```

### Helm values

```yaml
# values-search.yaml（infra/helm/services/system/search/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/search
  tag: ""

replicaCount: 2

container:
  port: 8080
  grpcPort: 50051

service:
  type: ClusterIP
  port: 80
  grpcPort: 50051

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 5
  targetCPUUtilizationPercentage: 70

kafka:
  enabled: true
  brokers: []

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/search/opensearch"
      key: "password"
      mountPath: "/vault/secrets/opensearch-password"
```

---

## デプロイ

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| OpenSearch パスワード | `secret/data/k1s0/system/search/opensearch` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## 詳細設計ドキュメント

- [system-search-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-search-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [RBAC設計.md](../../architecture/auth/RBAC設計.md) -- RBAC ロールモデル
- [system-server.md](../auth/server.md) -- system tier サーバー一覧
