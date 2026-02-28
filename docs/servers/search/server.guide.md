# system-search-server ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

---

## API リクエスト・レスポンス例

### POST /api/v1/indices

**リクエスト**

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

**レスポンス（201 Created）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "k1s0-products",
  "mapping": { "..." : "..." },
  "created_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": "index already exists: k1s0-products"
}
```

### GET /api/v1/indices

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

### POST /api/v1/search/index

**リクエスト**

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

**レスポンス（201 Created）**

```json
{
  "id": "product-001",
  "index_name": "k1s0-products",
  "indexed_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": "index not found: k1s0-products"
}
```

### POST /api/v1/search

**リクエスト**

```json
{
  "index_name": "k1s0-products",
  "query": "高性能ノートPC",
  "from": 0,
  "size": 20
}
```

**レスポンス（200 OK）**

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

**レスポンス（404 Not Found）**

```json
{
  "error": "index not found: k1s0-products"
}
```

### DELETE /api/v1/search/index/:index_name/:id

**レスポンス（404 Not Found）**

```json
{
  "error": "document not found: product-001"
}
```

---

## Kafka メッセージ例

### インデックス要求メッセージ

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
