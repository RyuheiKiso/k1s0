# system-file-server 設計

system tier のファイルストレージ抽象化サーバー設計を定義する。S3/GCS/Ceph 互換ストレージを統一 API で抽象化し、ファイルメタデータ管理・プリサインドURL発行・テナント分離アクセス制御を提供する。ファイルアップロード完了時に Kafka トピック `k1s0.system.file.uploaded.v1` でイベントを発行する。Rust での実装を定義する。

## 概要

system tier のファイルストレージ抽象化サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| ストレージ抽象化 | S3/GCS/Ceph 互換ストレージへの統一アクセスレイヤー |
| メタデータ管理 | ファイル名・サイズ・MIME type・タグ・所有者情報を PostgreSQL で管理 |
| プリサインドURL発行 | アップロード・ダウンロード用の一時的な署名付き URL を発行 |
| テナント分離 | テナント ID によるバケット/プレフィックス分離とアクセス制御 |
| アップロード完了イベント | Kafka `k1s0.system.file.uploaded.v1` でアップロード完了を通知 |

### 技術スタック

| コンポーネント | Rust |
| --- | --- |
| HTTP フレームワーク | axum + tokio |
| DB アクセス | sqlx v0.8 |
| S3 クライアント | aws-sdk-s3（S3/GCS/Ceph 互換） |
| Kafka | rdkafka (rust-rdkafka) |
| OTel | opentelemetry v0.27 |
| 設定管理 | serde_yaml |
| バリデーション | validator v0.18 |
| シリアライゼーション | serde + serde_json |
| 非同期ランタイム | tokio 1 (full) |

### 配置パス

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
| Rust | `regions/system/server/rust/file/` |

---

## 設計方針

[認証認可設計.md](認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| ストレージバックエンド | aws-sdk-s3 クライアントで S3/GCS/Ceph 互換エンドポイントに接続。バックエンドはconfig で切り替え |
| メタデータ永続化 | PostgreSQL の `file` スキーマ（file_metadata テーブル）でメタデータを管理 |
| テナント分離 | バケット名またはオブジェクトキープレフィックスにテナント ID を付与（例: `tenant-abc/path/to/file`） |
| プリサインドURL | aws-sdk-s3 の presigned request 機能で TTL 付き署名 URL を発行 |
| アップロード完了通知 | クライアントがコールバック API を呼び出した時点で Kafka イベントを発行 |
| 認可 | 参照・ダウンロードは `sys_auditor`、アップロード・タグ更新は `sys_operator`、削除は `sys_operator`（所有者）/ `sys_admin`（全体） |
| ポート | ホスト側 8098（内部 8080） |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_FILE_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/files` | ファイル一覧取得 | `sys_auditor` 以上 |
| POST | `/api/v1/files/upload-url` | アップロードプリサインドURL発行 | `sys_operator` 以上 |
| POST | `/api/v1/files/:id/complete` | アップロード完了通知 | `sys_operator` 以上 |
| GET | `/api/v1/files/:id` | ファイルメタデータ取得 | `sys_auditor` 以上 |
| GET | `/api/v1/files/:id/download-url` | ダウンロードプリサインドURL発行 | `sys_auditor` 以上 |
| DELETE | `/api/v1/files/:id` | ファイル削除 | `sys_operator` 以上 |
| PUT | `/api/v1/files/:id/tags` | タグ更新 | `sys_operator` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### GET /api/v1/files

ファイルメタデータ一覧をページネーション付きで取得する。テナント ID・タグ・MIME type でフィルタリングできる。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `tenant_id` | string | No | - | テナント ID でフィルタ |
| `owner_id` | string | No | - | 所有者 ID でフィルタ |
| `mime_type` | string | No | - | MIME type でフィルタ（例: `image/`） |
| `tag` | string | No | - | タグでフィルタ（キー=値 形式） |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス（200 OK）**

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

#### POST /api/v1/files/upload-url

アップロード用のプリサインドURLを発行する。クライアントはこの URL に対して直接 HTTP PUT でファイルをアップロードする。アップロード完了後、`/api/v1/files/:id/complete` を呼び出してサーバーに完了を通知する。

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

#### GET /api/v1/files/:id

ファイルのメタデータを取得する。ストレージへの直接アクセスは行わない。

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

#### GET /api/v1/files/:id/download-url

ダウンロード用のプリサインドURLを発行する。クライアントはこの URL に対して直接 HTTP GET でファイルをダウンロードする。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `expires_in_seconds` | int | No | 3600 | URLの有効期限（秒）。最大 86400 |

**レスポンス（200 OK）**

```json
{
  "file_id": "file_01JABCDEF1234567890",
  "download_url": "https://storage.example.com/k1s0-files/tenant-abc/reports/report-2026-02.pdf?X-Amz-Signature=...",
  "expires_at": "2026-02-20T11:00:00.000+00:00"
}
```

#### PUT /api/v1/files/:id/tags

ファイルのタグを更新する。既存タグは上書きされる（全置換）。

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

#### DELETE /api/v1/files/:id

ファイルをメタデータとストレージの両方から削除する。

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "file file_01JABCDEF1234567890 deleted"
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_FILE_NOT_FOUND` | 404 | 指定されたファイルが見つからない |
| `SYS_FILE_ALREADY_EXISTS` | 409 | 同一パスのファイルが既に存在する |
| `SYS_FILE_UPLOAD_PENDING` | 409 | アップロードがまだ完了していない |
| `SYS_FILE_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_FILE_ACCESS_DENIED` | 403 | 別テナントのファイルへのアクセス拒否 |
| `SYS_FILE_STORAGE_ERROR` | 502 | ストレージバックエンドへの接続・操作エラー |
| `SYS_FILE_SIZE_EXCEEDED` | 413 | ファイルサイズ上限超過 |
| `SYS_FILE_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.file.v1;

service FileService {
  rpc GetFileMetadata(GetFileMetadataRequest) returns (GetFileMetadataResponse);
  rpc GenerateUploadUrl(GenerateUploadUrlRequest) returns (GenerateUploadUrlResponse);
  rpc GenerateDownloadUrl(GenerateDownloadUrlRequest) returns (GenerateDownloadUrlResponse);
  rpc DeleteFile(DeleteFileRequest) returns (DeleteFileResponse);
}

message GetFileMetadataRequest {
  string file_id = 1;
}

message GetFileMetadataResponse {
  FileMetadata file = 1;
}

message GenerateUploadUrlRequest {
  string name = 1;
  uint64 size_bytes = 2;
  string mime_type = 3;
  string tenant_id = 4;
  map<string, string> tags = 5;
  uint32 expires_in_seconds = 6;
}

message GenerateUploadUrlResponse {
  string file_id = 1;
  string upload_url = 2;
  string expires_at = 3;
}

message GenerateDownloadUrlRequest {
  string file_id = 1;
  uint32 expires_in_seconds = 2;
}

message GenerateDownloadUrlResponse {
  string file_id = 1;
  string download_url = 2;
  string expires_at = 3;
}

message DeleteFileRequest {
  string file_id = 1;
}

message DeleteFileResponse {
  bool success = 1;
  string message = 2;
}

message FileMetadata {
  string id = 1;
  string name = 2;
  uint64 size_bytes = 3;
  string mime_type = 4;
  string tenant_id = 5;
  string owner_id = 6;
  map<string, string> tags = 7;
  string storage_key = 8;
  optional string checksum_sha256 = 9;
  string status = 10;
  string created_at = 11;
  string updated_at = 12;
}
```

---

## Kafka メッセージング設計

### ファイルアップロード完了イベント

クライアントが `/api/v1/files/:id/complete` を呼び出した時点で、`k1s0.system.file.uploaded.v1` トピックへ発行する。ダウンストリームのサービスはこのイベントを Consumer してファイル処理（ウイルススキャン・サムネイル生成等）を実施できる。

**メッセージフォーマット**

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

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.file.uploaded.v1` |
| キー | file_id |
| パーティション戦略 | tenant_id によるハッシュ分散 |

### ファイル削除イベント

ファイル削除時に `k1s0.system.file.deleted.v1` トピックへ発行する。

**メッセージフォーマット**

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

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の 4 レイヤー構成に従う。

```
domain（エンティティ・リポジトリインターフェース・ドメインサービス）
  ^
usecase（ビジネスロジック）
  ^
adapter（REST ハンドラー・gRPC ハンドラー）
  ^
infra（DB接続・S3クライアント・Kafka Producer・設定ローダー）
```

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/model | `FileMetadata` | エンティティ定義 |
| domain/repository | `FileMetadataRepository`, `FileStorageRepository` | リポジトリトレイト |
| domain/service | `FileDomainService` | テナント分離・ストレージキー生成ロジック |
| usecase | `ListFilesUsecase`, `GenerateUploadUrlUsecase`, `CompleteUploadUsecase`, `GetFileMetadataUsecase`, `GenerateDownloadUrlUsecase`, `DeleteFileUsecase`, `UpdateFileTagsUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| infra/config | Config ローダー | config.yaml の読み込み |
| infra/persistence | `FileMetadataPostgresRepository` | PostgreSQL リポジトリ実装 |
| infra/storage | `S3FileStorageRepository` | aws-sdk-s3 ストレージ実装 |
| infra/messaging | `FileUploadedKafkaProducer`, `FileDeletedKafkaProducer` | Kafka プロデューサー |

### ドメインモデル

#### FileMetadata

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | ファイルの一意識別子 |
| `name` | String | ファイル名（元のファイル名） |
| `size_bytes` | u64 | ファイルサイズ（バイト） |
| `mime_type` | String | MIME type（例: `application/pdf`） |
| `tenant_id` | String | 所属テナント ID |
| `owner_id` | String | アップロードした ユーザー ID |
| `tags` | HashMap\<String, String\> | 任意のタグ（最大 10 件） |
| `storage_key` | String | ストレージ上のオブジェクトキー |
| `checksum_sha256` | Option\<String\> | SHA-256 チェックサム（アップロード完了後に記録） |
| `status` | String | ファイル状態（pending / available / deleted） |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

### 依存関係図

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
    │  domain/model   │              │ domain/repository          │   │
    │  FileMetadata   │              │ FileMetadataRepository     │   │
    └────────────────┘              │ FileStorageRepository      │   │
              │                     │ (trait)                    │   │
              │  ┌────────────────┐  └──────────┬─────────────────┘   │
              └──▶ domain/service │             │                     │
                 │ FileDomain    │             │                     │
                 │ Service       │             │                     │
                 └────────────────┘             │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │                  infra 層  │
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

## 設定ファイル

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

---

## デプロイ

### Helm values

[helm設計.md](helm設計.md) のサーバー用 Helm Chart を使用する。file 固有の values は以下の通り。

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

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/file/database` |
| S3 シークレットキー | `secret/data/k1s0/system/file/storage` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## 詳細設計ドキュメント

- [system-file-server-実装設計.md](system-file-server-実装設計.md) -- 実装設計の詳細
- [system-file-server-デプロイ設計.md](system-file-server-デプロイ設計.md) -- デプロイ設計の詳細

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
- [system-server-実装設計.md](system-server-実装設計.md) -- system tier 実装設計
- [tier-architecture.md](tier-architecture.md) -- Tier アーキテクチャ
- [helm設計.md](helm設計.md) -- Helm Chart・Vault Agent Injector
