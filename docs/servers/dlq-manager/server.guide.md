# system-dlq-manager-server 設計ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

---

## API リクエスト・レスポンス例

### GET /api/v1/dlq/:topic

**レスポンス（200 OK）**

```json
{
  "messages": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "original_topic": "orders.events.v1",
      "error_message": "processing failed",
      "retry_count": 0,
      "max_retries": 3,
      "payload": {"order_id": "123"},
      "status": "PENDING",
      "created_at": "2026-02-20T10:30:00.000+00:00",
      "updated_at": "2026-02-20T10:30:00.000+00:00",
      "last_retry_at": null
    }
  ],
  "pagination": {
    "total_count": 150,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}
```

### GET /api/v1/dlq/messages/:id

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "original_topic": "orders.events.v1",
  "error_message": "processing failed",
  "retry_count": 1,
  "max_retries": 3,
  "payload": {"order_id": "123"},
  "status": "RETRYING",
  "created_at": "2026-02-20T10:30:00.000+00:00",
  "updated_at": "2026-02-20T10:31:00.000+00:00",
  "last_retry_at": "2026-02-20T10:31:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_DLQ_NOT_FOUND",
    "message": "dlq message not found: invalid-uuid",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_DLQ_VALIDATION_ERROR",
    "message": "invalid message id: not-a-uuid",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/dlq/messages/:id/retry

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "RESOLVED",
  "message": "message retry initiated"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_DLQ_NOT_FOUND",
    "message": "dlq message not found: 550e8400-e29b-41d4-a716-446655440000",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_DLQ_CONFLICT",
    "message": "message is not retryable: status=DEAD, retry_count=3/3",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### DELETE /api/v1/dlq/messages/:id

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "message 550e8400-e29b-41d4-a716-446655440000 deleted"
}
```

### POST /api/v1/dlq/:topic/retry-all

**レスポンス（200 OK）**

```json
{
  "retried": 15,
  "message": "15 messages retried in topic orders.dlq.v1"
}
```

---

## Kafka コンシューマー設計

### メッセージ取り込みフロー

```
1. Kafka コンシューマーがメッセージを受信
2. ペイロードを JSON にデシリアライズ（失敗時は null）
3. Kafka ヘッダーの "error" キーからエラーメッセージを取得（未設定時は "unknown error"）
4. DlqMessage エンティティを作成（status=PENDING, max_retries=3）
5. リポジトリ経由で DB に永続化
6. ログ出力（message_id, topic）
```

---

## 設定ファイル例

### config.yaml（本番）

```yaml
app:
  name: "dlq-manager"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_dlq"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  consumer_group: "dlq-manager.default"
  security_protocol: "PLAINTEXT"
  dlq_topic_pattern: "*.dlq.v1"
```

---

## Helm values 例

```yaml
# values-dlq-manager.yaml（infra/helm/services/system/dlq-manager/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/dlq-manager
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
    - path: "secret/data/k1s0/system/dlq-manager/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
```

---

## 統合テスト一覧

- `test_healthz_returns_ok` -- ヘルスチェック正常
- `test_readyz_returns_ok` -- レディネスチェック正常
- `test_list_messages_empty_topic` -- 空トピックの一覧取得
- `test_list_messages_returns_stored_message` -- メッセージありの一覧取得
- `test_get_message_returns_404_when_not_found` -- 未存在メッセージ取得時 404
- `test_get_message_returns_message` -- メッセージ取得成功
- `test_get_message_returns_400_for_invalid_id` -- 不正 UUID 時 400
- `test_retry_message_returns_404_when_not_found` -- 未存在メッセージリトライ時 404
- `test_retry_message_resolves_pending_message` -- PENDING メッセージのリトライ成功
- `test_delete_message_returns_ok` -- メッセージ削除成功
- `test_retry_all_returns_retried_count` -- 一括リトライの件数返却
