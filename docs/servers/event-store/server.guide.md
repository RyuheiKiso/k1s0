# system-event-store-server 設計ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

---

## API リクエスト・レスポンス例

### POST /api/v1/events

**リクエスト例**

```json
{
  "events": [
    {
      "event_type": "OrderPlaced",
      "payload": {
        "order_id": "order-001",
        "tenant_id": "tenant-abc",
        "items": [
          {"product_id": "prod-001", "quantity": 2, "unit_price": 1500}
        ],
        "total_amount": 3000
      },
      "metadata": {
        "actor_id": "user-001",
        "correlation_id": "corr_01JABCDEF1234567890",
        "causation_id": null
      }
    }
  ],
  "expected_version": 0
}
```

**レスポンス（201 Created）**

```json
{
  "stream_id": "order-order-001",
  "events": [
    {
      "stream_id": "order-order-001",
      "sequence": 1,
      "event_type": "OrderPlaced",
      "version": 1,
      "payload": {
        "order_id": "order-001",
        "tenant_id": "tenant-abc",
        "items": [
          {"product_id": "prod-001", "quantity": 2, "unit_price": 1500}
        ],
        "total_amount": 3000
      },
      "metadata": {
        "actor_id": "user-001",
        "correlation_id": "corr_01JABCDEF1234567890",
        "causation_id": null
      },
      "occurred_at": "2026-02-23T10:00:00.000+00:00",
      "stored_at": "2026-02-23T10:00:00.012+00:00"
    }
  ],
  "current_version": 1
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_EVSTORE_VERSION_CONFLICT",
    "message": "version conflict for stream order-order-001: expected 0, actual 3",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "expected_version", "message": "0"},
      {"field": "actual_version", "message": "3"}
    ]
  }
}
```

### GET /api/v1/events/:stream_id

**レスポンス（200 OK）**

```json
{
  "stream_id": "order-order-001",
  "events": [
    {
      "stream_id": "order-order-001",
      "sequence": 1,
      "event_type": "OrderPlaced",
      "version": 1,
      "payload": {
        "order_id": "order-001",
        "tenant_id": "tenant-abc",
        "total_amount": 3000
      },
      "metadata": {
        "actor_id": "user-001",
        "correlation_id": "corr_01JABCDEF1234567890",
        "causation_id": null
      },
      "occurred_at": "2026-02-23T10:00:00.000+00:00",
      "stored_at": "2026-02-23T10:00:00.012+00:00"
    },
    {
      "stream_id": "order-order-001",
      "sequence": 2,
      "event_type": "OrderShipped",
      "version": 2,
      "payload": {
        "order_id": "order-001",
        "tracking_number": "TRK-12345"
      },
      "metadata": {
        "actor_id": "user-002",
        "correlation_id": "corr_02JABCDEF1234567890",
        "causation_id": "corr_01JABCDEF1234567890"
      },
      "occurred_at": "2026-02-23T14:00:00.000+00:00",
      "stored_at": "2026-02-23T14:00:00.008+00:00"
    }
  ],
  "current_version": 2,
  "pagination": {
    "total_count": 2,
    "page": 1,
    "page_size": 50,
    "has_next": false
  }
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_EVSTORE_STREAM_NOT_FOUND",
    "message": "stream not found: order-order-999",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### GET /api/v1/events

**レスポンス（200 OK）**

```json
{
  "events": [
    {
      "stream_id": "order-order-001",
      "sequence": 1,
      "event_type": "OrderPlaced",
      "version": 1,
      "payload": { "..." : "..." },
      "metadata": { "..." : "..." },
      "occurred_at": "2026-02-23T10:00:00.000+00:00",
      "stored_at": "2026-02-23T10:00:00.012+00:00"
    }
  ],
  "pagination": {
    "total_count": 100,
    "page": 1,
    "page_size": 50,
    "has_next": true
  }
}
```

### GET /api/v1/streams

**レスポンス（200 OK）**

```json
{
  "streams": [
    {
      "id": "order-order-001",
      "aggregate_type": "Order",
      "current_version": 2,
      "created_at": "2026-02-23T10:00:00.000+00:00",
      "updated_at": "2026-02-23T14:00:00.000+00:00"
    }
  ]
}
```

### POST /api/v1/streams/:stream_id/snapshot

**リクエスト例**

```json
{
  "snapshot_version": 2,
  "aggregate_type": "Order",
  "state": {
    "order_id": "order-001",
    "status": "shipped",
    "tenant_id": "tenant-abc",
    "total_amount": 3000,
    "tracking_number": "TRK-12345"
  }
}
```

**レスポンス（201 Created）**

```json
{
  "id": "snap_01JABCDEF1234567890",
  "stream_id": "order-order-001",
  "snapshot_version": 2,
  "aggregate_type": "Order",
  "created_at": "2026-02-23T15:00:00.000+00:00"
}
```

### GET /api/v1/streams/:stream_id/snapshot

**レスポンス（200 OK）**

```json
{
  "id": "snap_01JABCDEF1234567890",
  "stream_id": "order-order-001",
  "snapshot_version": 2,
  "aggregate_type": "Order",
  "state": {
    "order_id": "order-001",
    "status": "shipped",
    "tenant_id": "tenant-abc",
    "total_amount": 3000,
    "tracking_number": "TRK-12345"
  },
  "created_at": "2026-02-23T15:00:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_EVSTORE_SNAPSHOT_NOT_FOUND",
    "message": "no snapshot found for stream: order-order-001",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### DELETE /api/v1/streams/:stream_id

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "stream order-order-001 and all related data deleted"
}
```

---

## Kafka メッセージフォーマット

イベント追記後、バックグラウンドタスクが `k1s0.system.eventstore.event.published.v1` トピックへ非同期転送する。

```json
{
  "event_type": "EVENT_PUBLISHED",
  "stream_id": "order-order-001",
  "sequence": 1,
  "domain_event_type": "OrderPlaced",
  "version": 1,
  "payload": {
    "order_id": "order-001",
    "tenant_id": "tenant-abc",
    "total_amount": 3000
  },
  "metadata": {
    "actor_id": "user-001",
    "correlation_id": "corr_01JABCDEF1234567890",
    "causation_id": null
  },
  "occurred_at": "2026-02-23T10:00:00.000+00:00",
  "stored_at": "2026-02-23T10:00:00.012+00:00"
}
```

---

## 設定ファイル例

### config.yaml（本番）

```yaml
app:
  name: "event-store"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 9090

database:
  url: "postgresql://app:@postgres.k1s0-system.svc.cluster.local:5432/k1s0_system"
  schema: "event_store"
  max_connections: 20
  min_connections: 5
  connect_timeout_seconds: 5

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"
  topic_published: "k1s0.system.eventstore.event.published.v1"
  producer_acks: "all"
  producer_retries: 3

auth:
  jwks_url: "http://auth-server.k1s0-system.svc.cluster.local:8080/.well-known/jwks.json"

event_store:
  max_events_per_append: 100
  max_page_size: 200
```

---

## Helm values 例

```yaml
# values-event-store.yaml（infra/helm/services/system/event-store/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/event-store
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
  maxReplicas: 8
  targetCPUUtilizationPercentage: 70

kafka:
  enabled: true
  brokers: []

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/event-store/database"
      key: "password"
      mountPath: "/vault/secrets/database-password"
```

---

## DB スキーマ DDL

```sql
-- event_store スキーマ
CREATE SCHEMA IF NOT EXISTS event_store;

-- ストリームテーブル
CREATE TABLE event_store.event_streams (
    id              TEXT        NOT NULL PRIMARY KEY,
    aggregate_type  TEXT        NOT NULL,
    current_version BIGINT      NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- イベントテーブル（Append-only: UPDATE/DELETE 禁止）
CREATE TABLE event_store.events (
    sequence        BIGINT      NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    stream_id       TEXT        NOT NULL REFERENCES event_store.event_streams(id),
    version         BIGINT      NOT NULL,
    event_type      TEXT        NOT NULL,
    payload         JSONB       NOT NULL,
    metadata        JSONB       NOT NULL DEFAULT '{}',
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    stored_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (stream_id, version)
);

CREATE INDEX idx_events_stream_id ON event_store.events (stream_id, version);

-- スナップショットテーブル
CREATE TABLE event_store.snapshots (
    id                TEXT        NOT NULL PRIMARY KEY,
    stream_id         TEXT        NOT NULL REFERENCES event_store.event_streams(id),
    snapshot_version  BIGINT      NOT NULL,
    aggregate_type    TEXT        NOT NULL,
    state             JSONB       NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_snapshots_stream_id ON event_store.snapshots (stream_id, snapshot_version DESC);
```
