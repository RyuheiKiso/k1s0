# system-search-server 設計

system tier の全文検索サーバー設計を定義する。OpenSearch と連携し、全 Tier のサービスからインデックス書き込み・全文検索クエリを集約する。Kafka トピック `k1s0.system.search.index.requested.v1` でインデックス要求を受け付ける。Rust での実装を定義する。

## 概要

system tier の全文検索サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| インデックス管理 | インデックス定義の作成・更新・削除 |
| ドキュメント管理 | ドキュメントのインデックス登録・更新・削除（REST / Kafka 非同期） |
| 全文検索 | キーワード・フィルタ・ファセットによる全文検索クエリ |
| インデックス状態確認 | ドキュメント数・サイズ等のインデックスステータス取得 |

### 技術スタック

| コンポーネント | Rust |
| --- | --- |
| HTTP フレームワーク | axum + tokio |
| gRPC | tonic v0.12 |
| DB アクセス | sqlx v0.8 |
| Kafka | rdkafka (rust-rdkafka) |
| OTel | opentelemetry v0.27 |
| 設定管理 | serde_yaml |
| バリデーション | validator v0.18 |
| キャッシュ | moka v0.12 |
| シリアライゼーション | serde + serde_json |
| 非同期ランタイム | tokio 1 (full) |
| OpenSearch クライアント | opensearch-rs |

### 配置パス

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
| Rust | `regions/system/server/rust/search/` |

---

## 設計方針

[認証認可設計.md](認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| 検索エンジン | OpenSearch。opensearch-rs クライアント経由でアクセス |
| 非同期インデックス | Kafka トピック `k1s0.system.search.index.requested.v1` を Consumer し非同期にインデックス登録 |
| 同期インデックス | REST POST `/api/v1/indices/:name/documents` により即座にインデックス登録 |
| キャッシュ | インデックス一覧・ドキュメント数等のステータスを moka で TTL 30 秒キャッシュ |
| 認可 | インデックス管理は `sys_admin`、ドキュメント操作は `sys_operator`、検索・参照は `sys_auditor` |
| ポート | ホスト側 8094（内部 8080）、gRPC 9090 |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_SEARCH_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/indices` | インデックス作成 | `sys_admin` のみ |
| DELETE | `/api/v1/indices/:name` | インデックス削除 | `sys_admin` のみ |
| GET | `/api/v1/indices` | インデックス一覧取得 | `sys_auditor` 以上 |
| POST | `/api/v1/indices/:name/documents` | ドキュメント登録 | `sys_operator` 以上 |
| PUT | `/api/v1/indices/:name/documents/:id` | ドキュメント更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/indices/:name/documents/:id` | ドキュメント削除 | `sys_operator` 以上 |
| POST | `/api/v1/search` | 全文検索 | `sys_auditor` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### POST /api/v1/indices

新しいインデックスを OpenSearch に作成する。マッピング定義を指定することで、フィールド型・アナライザー設定を行う。

**リクエスト**

```json
{
  "name": "k1s0-products",
  "settings": {
    "number_of_shards": 1,
    "number_of_replicas": 1
  },
  "mappings": {
    "properties": {
      "title": { "type": "text", "analyzer": "kuromoji" },
      "description": { "type": "text", "analyzer": "kuromoji" },
      "tenant_id": { "type": "keyword" },
      "created_at": { "type": "date" }
    }
  }
}
```

**レスポンス（201 Created）**

```json
{
  "name": "k1s0-products",
  "status": "created",
  "created_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス（409 Conflict）**

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

登録済みインデックスの一覧とステータス（ドキュメント数・サイズ）を取得する。

**レスポンス（200 OK）**

```json
{
  "indices": [
    {
      "name": "k1s0-products",
      "doc_count": 12450,
      "size_bytes": 5242880,
      "status": "green",
      "created_at": "2026-02-20T10:00:00.000+00:00"
    }
  ],
  "total_count": 1
}
```

#### POST /api/v1/indices/:name/documents

指定インデックスにドキュメントを同期登録する。`id` を指定した場合は upsert として動作する。

**リクエスト**

```json
{
  "id": "product-001",
  "document": {
    "title": "高性能ノートPC",
    "description": "最新世代プロセッサー搭載の高性能ノートパソコン",
    "tenant_id": "tenant-abc",
    "created_at": "2026-02-20T10:00:00.000+00:00"
  }
}
```

**レスポンス（201 Created）**

```json
{
  "id": "product-001",
  "index": "k1s0-products",
  "result": "created"
}
```

**レスポンス（404 Not Found）**

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

指定されたクエリで全文検索を実行する。インデックス名・キーワード・フィルタ・ファセット・ページネーションをサポートする。

**リクエスト**

```json
{
  "index": "k1s0-products",
  "query": "高性能ノートPC",
  "filters": {
    "tenant_id": "tenant-abc"
  },
  "facets": ["tenant_id"],
  "page": 1,
  "page_size": 20
}
```

**レスポンス（200 OK）**

```json
{
  "hits": [
    {
      "id": "product-001",
      "score": 1.8752,
      "document": {
        "title": "高性能ノートPC",
        "description": "最新世代プロセッサー搭載の高性能ノートパソコン",
        "tenant_id": "tenant-abc",
        "created_at": "2026-02-20T10:00:00.000+00:00"
      }
    }
  ],
  "facets": {
    "tenant_id": [
      { "value": "tenant-abc", "count": 350 }
    ]
  },
  "pagination": {
    "total_count": 1,
    "page": 1,
    "page_size": 20,
    "has_next": false
  }
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_SEARCH_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "index", "message": "index is required and must be non-empty"}
    ]
  }
}
```

#### DELETE /api/v1/indices/:name/documents/:id

指定インデックスから特定ドキュメントを削除する。

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "document product-001 deleted from index k1s0-products"
}
```

**レスポンス（404 Not Found）**

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

**メッセージフォーマット**

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

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.search.index.requested.v1` |
| Consumer グループ | `search-server-consumer` |
| auto.offset.reset | `earliest` |
| max.poll.records | `100` |
| キー | インデックス名（例: `k1s0-products`） |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の 4 レイヤー構成に従う。

```
domain（エンティティ・リポジトリインターフェース・ドメインサービス）
  ^
usecase（ビジネスロジック）
  ^
adapter（REST ハンドラー・gRPC ハンドラー・Kafka Consumer）
  ^
infra（OpenSearch クライアント・Kafka Producer/Consumer・moka キャッシュ・設定ローダー）
```

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/model | `SearchIndex`, `SearchDocument`, `SearchQuery`, `SearchResult` | エンティティ定義 |
| domain/repository | `SearchIndexRepository`, `SearchDocumentRepository` | リポジトリトレイト |
| domain/service | `SearchDomainService` | 検索クエリ構築・ファセット集計ロジック |
| usecase | `CreateIndexUsecase`, `DeleteIndexUsecase`, `ListIndicesUsecase`, `IndexDocumentUsecase`, `UpdateDocumentUsecase`, `DeleteDocumentUsecase`, `SearchUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic）, Kafka Consumer ハンドラー | プロトコル変換 |
| infra/config | Config ローダー | config.yaml の読み込み |
| infra/persistence | `OpenSearchIndexRepository`, `OpenSearchDocumentRepository` | OpenSearch リポジトリ実装 |
| infra/cache | `SearchCacheService` | moka キャッシュ実装（インデックスステータスキャッシュ） |
| infra/messaging | `SearchIndexKafkaConsumer` | Kafka Consumer（非同期インデックス要求） |

### ドメインモデル

#### SearchIndex

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `name` | String | インデックス名（例: `k1s0-products`） |
| `doc_count` | u64 | 登録ドキュメント数 |
| `size_bytes` | u64 | インデックスサイズ（バイト） |
| `status` | String | インデックス状態（green / yellow / red） |
| `created_at` | DateTime\<Utc\> | 作成日時 |

#### SearchDocument

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | ドキュメント ID |
| `index` | String | 所属インデックス名 |
| `document` | serde_json::Value | ドキュメント本体（任意の JSON） |

#### SearchQuery

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `index` | String | 検索対象インデックス名 |
| `query` | String | 全文検索キーワード |
| `filters` | HashMap\<String, String\> | フィールドフィルタ |
| `facets` | Vec\<String\> | ファセット集計対象フィールド |
| `page` | u32 | ページ番号 |
| `page_size` | u32 | 1 ページあたりの件数 |

### 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (search_handler.rs)         │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  create_index / delete_index             │   │
                    │  │  list_indices                            │   │
                    │  │  index_document / update_document        │   │
                    │  │  delete_document / search                │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (search_grpc.rs)            │   │
                    │  │  IndexDocument / Search / DeleteDocument │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ Kafka Consumer (index_consumer.rs)       │   │
                    │  │  k1s0.system.search.index.requested.v1   │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  CreateIndexUsecase / DeleteIndexUsecase /      │
                    │  ListIndicesUsecase / IndexDocumentUsecase /    │
                    │  UpdateDocumentUsecase / DeleteDocumentUsecase  │
                    │  SearchUsecase                                  │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/model   │              │ domain/repository          │   │
    │  SearchIndex,   │              │ SearchIndexRepository      │   │
    │  SearchDocument,│              │ SearchDocumentRepository   │   │
    │  SearchQuery,   │              │ (trait)                    │   │
    │  SearchResult   │              └──────────┬─────────────────┘   │
    └────────────────┘                         │                     │
              │                                │                     │
              │  ┌────────────────┐            │                     │
              └──▶ domain/service │            │                     │
                 │ SearchDomain   │            │                     │
                 │ Service        │            │                     │
                 └────────────────┘            │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │                  infra 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ OpenSearchIndex        │  │
                    │  │ Consumer     │  │ Repository             │  │
                    │  └──────────────┘  ├────────────────────────┤  │
                    │  ┌──────────────┐  │ OpenSearchDocument     │  │
                    │  │ moka Cache   │  │ Repository             │  │
                    │  │ Service      │  └────────────────────────┘  │
                    │  └──────────────┘  ┌────────────────────────┐  │
                    │  ┌──────────────┐  │ OpenSearch             │  │
                    │  │ Config       │  │ Config                 │  │
                    │  │ Loader       │  └────────────────────────┘  │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル

### config.yaml（本番）

```yaml
app:
  name: "search"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 9090

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

---

## デプロイ

### Helm values

[helm設計.md](helm設計.md) のサーバー用 Helm Chart を使用する。search 固有の values は以下の通り。

```yaml
# values-search.yaml（infra/helm/services/system/search/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/search
  tag: ""

replicaCount: 2

container:
  port: 8080
  grpcPort: 9090

service:
  type: ClusterIP
  port: 80
  grpcPort: 9090

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

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| OpenSearch パスワード | `secret/data/k1s0/system/search/opensearch` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## 詳細設計ドキュメント

- [system-search-server-実装設計.md](system-search-server-実装設計.md) -- 実装設計の詳細
- [system-search-server-デプロイ設計.md](system-search-server-デプロイ設計.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

- [RBAC設計.md](RBAC設計.md) -- RBAC ロールモデル
- [認証認可設計.md](認証認可設計.md) -- RBAC 認可モデル
- [API設計.md](API設計.md) -- REST API 設計ガイドライン
- [メッセージング設計.md](メッセージング設計.md) -- Kafka メッセージング設計
- [可観測性設計.md](可観測性設計.md) -- メトリクス・トレース設計
- [テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) -- サーバーテンプレート仕様
- [コーディング規約.md](コーディング規約.md) -- コーディング規約
- [system-server設計.md](system-server設計.md) -- system tier サーバー一覧
- [tier-architecture.md](tier-architecture.md) -- Tier アーキテクチャ
- [helm設計.md](helm設計.md) -- Helm Chart・Vault Agent Injector
