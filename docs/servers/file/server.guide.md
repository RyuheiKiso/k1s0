# system-file-server ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

---

## API リクエスト・レスポンス例

### GET /api/v1/files

```json
{
  "files": [
    {
      "id": "file_01JABCDEF1234567890",
      "name": "report-2026-02.pdf",
      "size_bytes": 2097152,
      "mime_type": "application/pdf",
      "tenant_id": "tenant-abc",
      "owner_id": "user-001",
      "tags": {
        "category": "report",
        "year": "2026"
      },
      "storage_key": "tenant-abc/reports/report-2026-02.pdf",
      "status": "available",
      "created_at": "2026-02-20T10:00:00.000+00:00",
      "updated_at": "2026-02-20T10:05:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 42,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}
```

### POST /api/v1/files/upload-url

**リクエスト**

```json
{
  "name": "report-2026-02.pdf",
  "size_bytes": 2097152,
  "mime_type": "application/pdf",
  "tenant_id": "tenant-abc",
  "tags": {
    "category": "report",
    "year": "2026"
  },
  "expires_in_seconds": 3600
}
```

**レスポンス（201 Created）**

```json
{
  "file_id": "file_01JABCDEF1234567890",
  "upload_url": "https://storage.example.com/k1s0-files/tenant-abc/reports/report-2026-02.pdf?X-Amz-Signature=...",
  "upload_method": "PUT",
  "expires_at": "2026-02-20T11:00:00.000+00:00",
  "required_headers": {
    "Content-Type": "application/pdf",
    "Content-Length": "2097152"
  }
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_FILE_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "size_bytes", "message": "size_bytes must be greater than 0"},
      {"field": "mime_type", "message": "mime_type is required"}
    ]
  }
}
```

### GET /api/v1/files/:id

**レスポンス（200 OK）**

```json
{
  "id": "file_01JABCDEF1234567890",
  "name": "report-2026-02.pdf",
  "size_bytes": 2097152,
  "mime_type": "application/pdf",
  "tenant_id": "tenant-abc",
  "owner_id": "user-001",
  "tags": {
    "category": "report",
    "year": "2026"
  },
  "storage_key": "tenant-abc/reports/report-2026-02.pdf",
  "checksum_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "status": "available",
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T10:05:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_FILE_NOT_FOUND",
    "message": "file not found: file_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### GET /api/v1/files/:id/download-url

```json
{
  "file_id": "file_01JABCDEF1234567890",
  "download_url": "https://storage.example.com/k1s0-files/tenant-abc/reports/report-2026-02.pdf?X-Amz-Signature=...",
  "expires_at": "2026-02-20T11:00:00.000+00:00"
}
```

### PUT /api/v1/files/:id/tags

**リクエスト**

```json
{
  "tags": {
    "category": "report",
    "year": "2026",
    "reviewed": "true"
  }
}
```

**レスポンス（200 OK）**

```json
{
  "id": "file_01JABCDEF1234567890",
  "tags": {
    "category": "report",
    "year": "2026",
    "reviewed": "true"
  },
  "updated_at": "2026-02-23T15:00:00.000+00:00"
}
```

### DELETE /api/v1/files/:id

```json
{
  "success": true,
  "message": "file file_01JABCDEF1234567890 deleted"
}
```

---

## Kafka メッセージ例

### ファイルアップロード完了イベント

クライアントが `/api/v1/files/:id/complete` を呼び出した時点で、`k1s0.system.file.uploaded.v1` トピックへ発行する。ダウンストリームのサービスはこのイベントを Consumer してファイル処理（ウイルススキャン・サムネイル生成等）を実施できる。

```json
{
  "event_type": "FILE_UPLOADED",
  "file_id": "file_01JABCDEF1234567890",
  "name": "report-2026-02.pdf",
  "size_bytes": 2097152,
  "mime_type": "application/pdf",
  "tenant_id": "tenant-abc",
  "owner_id": "user-001",
  "storage_key": "tenant-abc/reports/report-2026-02.pdf",
  "tags": {
    "category": "report",
    "year": "2026"
  },
  "uploaded_at": "2026-02-20T10:05:00.000+00:00"
}
```

### ファイル削除イベント

```json
{
  "event_type": "FILE_DELETED",
  "file_id": "file_01JABCDEF1234567890",
  "tenant_id": "tenant-abc",
  "storage_key": "tenant-abc/reports/report-2026-02.pdf",
  "deleted_at": "2026-02-23T15:00:00.000+00:00",
  "deleted_by": "admin@example.com"
}
```

---

## 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (file_handler.rs)           │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  list_files / generate_upload_url /      │   │
                    │  │  complete_upload / get_metadata /        │   │
                    │  │  generate_download_url / delete_file /   │   │
                    │  │  update_tags                             │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (file_grpc.rs)              │   │
                    │  │  GetFileMetadata / GenerateUploadUrl /   │   │
                    │  │  GenerateDownloadUrl / DeleteFile        │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  ListFilesUsecase /                             │
                    │  GenerateUploadUrlUsecase /                     │
                    │  CompleteUploadUsecase /                        │
                    │  GetFileMetadataUsecase /                       │
                    │  GenerateDownloadUrlUsecase /                   │
                    │  DeleteFileUsecase /                            │
                    │  UpdateFileTagsUsecase                          │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  FileMetadata   │              │ FileMetadataRepository     │   │
    └────────────────┘              │ FileStorageRepository      │   │
              │                     │ (trait)                    │   │
              │  ┌────────────────┐  └──────────┬─────────────────┘   │
              └──▶ domain/service │             │                     │
                 │ FileDomain    │             │                     │
                 │ Service       │             │                     │
                 └────────────────┘             │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ FileMetadataPostgres   │  │
                    │  │ Producer     │  │ Repository             │  │
                    │  │ (uploaded/   │  └────────────────────────┘  │
                    │  │  deleted)    │  ┌────────────────────────┐  │
                    │  └──────────────┘  │ S3FileStorage          │  │
                    │  ┌──────────────┐  │ Repository             │  │
                    │  │ Config       │  │ (aws-sdk-s3)           │  │
                    │  │ Loader       │  └────────────────────────┘  │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル例

### config.yaml（本番）

```yaml
app:
  name: "file"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 9090

database:
  url: "postgresql://app:@postgres.k1s0-system.svc.cluster.local:5432/k1s0_system"
  schema: "file"
  max_connections: 10
  min_connections: 2
  connect_timeout_seconds: 5

storage:
  backend: "s3"
  endpoint: "https://s3.ap-northeast-1.amazonaws.com"
  region: "ap-northeast-1"
  bucket: "k1s0-files"
  access_key_id: ""
  secret_access_key: ""
  presigned_url_max_expires_seconds: 86400
  max_file_size_bytes: 104857600

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"
  topic_uploaded: "k1s0.system.file.uploaded.v1"
  topic_deleted: "k1s0.system.file.deleted.v1"

auth:
  jwks_url: "http://auth-server.k1s0-system.svc.cluster.local:8080/.well-known/jwks.json"
```

### Helm values

```yaml
# values-file.yaml（infra/helm/services/system/file/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/file
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
    - path: "secret/data/k1s0/system/file/database"
      key: "password"
      mountPath: "/vault/secrets/database-password"
    - path: "secret/data/k1s0/system/file/storage"
      key: "secret_access_key"
      mountPath: "/vault/secrets/storage-secret-key"
```
