# system-file-server 設計

ローカルファイルシステム（PV ベース）を統一 API で抽象化するファイルサーバー。メタデータ管理・ストレージURL・テナント分離を提供。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| sys_auditor 以上 | files/read |
| sys_operator 以上 | files/write |
| sys_admin のみ | files/admin |


system tier のファイルストレージ抽象化サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| ストレージ抽象化 | ローカルファイルシステム（PV）への統一アクセスレイヤー |
| メタデータ管理 | ファイル名・サイズ・MIME type・タグ・所有者情報を PostgreSQL で管理 |
| ストレージURL発行 | アップロード・ダウンロード用の一時的な URL を発行（file-server 自身のエンドポイント） |
| テナント分離 | テナント ID によるプレフィックス分離とアクセス制御 |
| ファイルイベント通知 | Kafka `k1s0.system.file.events.v1` の単一トピックでイベントを通知 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| ストレージバックエンド | ローカルファイルシステム（tokio::fs） |

### 配置パス

配置: `regions/system/server/rust/file/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| ストレージバックエンド | ローカルファイルシステム（PV マウント）。`LocalFsStorageRepository` で tokio::fs を使用。AWS 依存なし |
| メタデータ永続化 | PostgreSQL の `file` スキーマ（file_metadata テーブル）でメタデータを管理 |
| テナント分離 | ストレージパスプレフィックスにテナント ID を付与（例: `tenant-abc/path/to/file`）。テナント ID はリクエストヘッダー `x-tenant-id` から取得し、`storage_path` のプレフィックスと照合してアクセス制御を行う（`FileMetadata` エンティティに `tenant_id` フィールドは存在しない） |
| ストレージURL | file-server 自身のエンドポイント URL を発行（`{base_url}/internal/storage/{storage_key}`） |
| アップロード完了通知 | クライアントがコールバック API を呼び出した時点で Kafka イベントを発行 |
| 認可 | 参照・ダウンロードは `sys_auditor`、アップロード・タグ更新は `sys_operator`、削除は `sys_operator`（所有者）/ `sys_admin`（全体） |
| ポート | 8098（REST）/ 50051（gRPC） |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_FILE_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/files` | ファイル一覧取得 | `sys_auditor` 以上 |
| POST | `/api/v1/files` | ファイルアップロード開始（プリサインドURL発行） | `sys_operator` 以上 |
| GET | `/api/v1/files/{id}` | ファイルメタデータ取得 | `sys_auditor` 以上 |
| POST | `/api/v1/files/{id}/complete` | アップロード完了通知 | `sys_operator` 以上 |
| DELETE | `/api/v1/files/{id}` | ファイル削除 | `sys_operator` 以上 |
| DELETE | `/api/v1/files/admin/{id}` | 管理者用ファイル削除（files/admin 権限が必要） | `sys_admin` のみ |
| PUT | `/api/v1/files/{id}/tags` | タグ更新 | `sys_operator` 以上 |
| GET | `/api/v1/files/{id}/download-url` | ダウンロードプリサインドURL発行 | `sys_auditor` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### GET /api/v1/files

ファイルメタデータ一覧をページネーション付きで取得する。テナント ID・MIME type・タグでフィルタリングできる。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `tenant_id` | string | No | - | テナント ID でフィルタ |
| `uploaded_by` | string | No | - | アップロード実行ユーザー ID でフィルタ（別名: `owner_id`） |
| `mime_type` | string | No | - | MIME type でフィルタ（例: `image/`、別名: `content_type`） |
| `tag` | string | No | - | タグフィルタ（`key:value` または `key=value`） |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス例（200 OK）**

```json
{
  "files": [
    {
      "id": "file_01JABCDEF1234567890",
      "filename": "report-2026-02.pdf",
      "size_bytes": 2097152,
      "content_type": "application/pdf",
      "tenant_id": "tenant-abc",
      "uploaded_by": "user-001",
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
  "total_count": 42,
  "page": 1,
  "page_size": 20,
  "has_next": true
}
```

#### POST /api/v1/files

アップロード用のプリサインドURLを発行する。クライアントはこの URL に対して直接 HTTP PUT でファイルをアップロードする。アップロード完了後、`/api/v1/files/{id}/complete` を呼び出してサーバーに完了を通知する。

**リクエスト例**

```json
{
  "filename": "report-2026-02.pdf",
  "size_bytes": 2097152,
  "content_type": "application/pdf",
  "tenant_id": "tenant-abc",
  "uploaded_by": "user-001",
  "tags": {
    "category": "report",
    "year": "2026"
  },
  "expires_in_seconds": 3600
}
```

**レスポンス例（201 Created）**

```json
{
  "file_id": "file_01JABCDEF1234567890",
  "upload_url": "https://storage.example.com/k1s0-files/tenant-abc/reports/report-2026-02.pdf?X-Amz-Signature=...",
  "expires_in_seconds": 3600
}
```

**レスポンス例（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_FILE_VALIDATION",
    "message": "validation failed"
  }
}
```

#### GET /api/v1/files/{id}

ファイルのメタデータを取得する。ファイルが `available` 状態の場合はダウンロード URL も合わせて返す。

**レスポンス例（200 OK）**

```json
{
  "file": {
    "id": "file_01JABCDEF1234567890",
    "filename": "report-2026-02.pdf",
    "size_bytes": 2097152,
    "content_type": "application/pdf",
    "tenant_id": "tenant-abc",
    "uploaded_by": "user-001",
    "tags": {
      "category": "report",
      "year": "2026"
    },
    "storage_key": "tenant-abc/reports/report-2026-02.pdf",
    "checksum_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "status": "available",
    "created_at": "2026-02-20T10:00:00.000+00:00",
    "updated_at": "2026-02-20T10:05:00.000+00:00"
  },
  "download_url": "https://storage.example.com/k1s0-files/tenant-abc/reports/report-2026-02.pdf?X-Amz-Signature=..."
}
```

**レスポンス例（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_FILE_NOT_FOUND",
    "message": "file not found: file_01JABCDEF1234567890"
  }
}
```

#### POST /api/v1/files/{id}/complete

クライアントがストレージへの直接アップロード完了後に呼び出す。サーバーはファイルの状態を `pending` から `available` に更新し、Kafka イベントを発行する。

**リクエスト例**

```json
{
  "checksum_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `checksum_sha256` | string | No | アップロード後のファイルチェックサム（任意） |

**レスポンス例（200 OK）**

```json
{
  "id": "file_01JABCDEF1234567890",
  "filename": "report-2026-02.pdf",
  "size_bytes": 2097152,
  "content_type": "application/pdf",
  "tenant_id": "tenant-abc",
  "uploaded_by": "user-001",
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

> 後方互換の別名入力も受け付ける: `name`→`filename`, `size`→`size_bytes`, `mime_type`→`content_type`, `owner_id`→`uploaded_by`。

| Canonical | Alias |
| --- | --- |
| `filename` | `name` |
| `size_bytes` | `size` |
| `content_type` | `mime_type` |
| `uploaded_by` | `owner_id` |

**レスポンス例（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_FILE_NOT_FOUND",
    "message": "file not found: file_01JABCDEF1234567890"
  }
}
```

**レスポンス例（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_FILE_ALREADY_COMPLETED",
    "message": "already completed"
  }
}
```

#### GET /api/v1/files/{id}/download-url

ダウンロード用のプリサインドURLを発行する。クライアントはこの URL に対して直接 HTTP GET でファイルをダウンロードする。

`expires_in_seconds` はレスポンス上は `u32` で返却し、gRPC/proto は `int32` として表現する（実装で `u32 -> i32` 変換）。

**レスポンス例（200 OK）**

```json
{
  "file_id": "file_01JABCDEF1234567890",
  "download_url": "https://storage.example.com/k1s0-files/tenant-abc/reports/report-2026-02.pdf?X-Amz-Signature=...",
  "expires_in_seconds": 3600
}
```

#### PUT /api/v1/files/{id}/tags

ファイルのタグを更新する。既存タグは上書きされる（全置換）。

**リクエスト例**

```json
{
  "tags": {
    "category": "report",
    "year": "2026",
    "reviewed": "true"
  }
}
```

**レスポンス例（200 OK）**

```json
{
  "file_id": "file_01JABCDEF1234567890",
  "tags": {
    "category": "report",
    "year": "2026",
    "reviewed": "true"
  },
  "message": "tags updated"
}
```

#### DELETE /api/v1/files/{id}

ファイルをメタデータとストレージの両方から削除する。

**レスポンス例（204 No Content）**

レスポンスボディなし。

#### DELETE /api/v1/files/admin/{id}

管理者権限（`files/admin`）でファイルを削除する。所有者チェックをスキップし、監査対象として強制削除を実行する。

**レスポンス例（204 No Content）**

レスポンスボディなし。

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_FILE_NOT_FOUND` | 404 | 指定されたファイルが見つからない |
| `SYS_FILE_ALREADY_COMPLETED` | 409 | アップロードが既に完了済み |
| `SYS_FILE_VALIDATION` | 400 | リクエストのバリデーションエラー |
| `SYS_FILE_NOT_AVAILABLE` | 400 | ファイルが利用可能状態でない（ダウンロードURL発行時） |
| `SYS_FILE_ACCESS_DENIED` | 403 | 別テナントのファイルへのアクセス拒否 |
| `SYS_FILE_STORAGE_ERROR` | 502 | ストレージバックエンドへの接続・操作エラー |
| `SYS_FILE_SIZE_EXCEEDED` | 413 | ファイルサイズ上限超過 |
| `SYS_FILE_UPLOAD_FAILED` | 500 | アップロードURL発行エラー |
| `SYS_FILE_GET_FAILED` | 500 | メタデータ取得エラー |
| `SYS_FILE_LIST_FAILED` | 500 | ファイル一覧取得エラー |
| `SYS_FILE_DELETE_FAILED` | 500 | ファイル削除エラー |
| `SYS_FILE_COMPLETE_FAILED` | 500 | アップロード完了処理エラー |
| `SYS_FILE_DOWNLOAD_URL_FAILED` | 500 | ダウンロードURL発行エラー |
| `SYS_FILE_TAGS_UPDATE_FAILED` | 500 | タグ更新エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.file.v1;

service FileService {
  rpc GetFileMetadata(GetFileMetadataRequest) returns (GetFileMetadataResponse);
  rpc ListFiles(ListFilesRequest) returns (ListFilesResponse);
  rpc GenerateUploadUrl(GenerateUploadUrlRequest) returns (GenerateUploadUrlResponse);
  rpc CompleteUpload(CompleteUploadRequest) returns (CompleteUploadResponse);
  rpc GenerateDownloadUrl(GenerateDownloadUrlRequest) returns (GenerateDownloadUrlResponse);
  rpc UpdateFileTags(UpdateFileTagsRequest) returns (UpdateFileTagsResponse);
  rpc DeleteFile(DeleteFileRequest) returns (DeleteFileResponse);
}

message FileMetadata {
  string id = 1;
  string filename = 2;
  string content_type = 3;
  int64 size_bytes = 4;
  string tenant_id = 5;
  string uploaded_by = 6;
  string status = 7;
  string created_at = 8;
  string updated_at = 9;
  map<string, string> tags = 10;
  string storage_key = 11;
  optional string checksum_sha256 = 12;
}

message GetFileMetadataRequest {
  string id = 1;
}

message GetFileMetadataResponse {
  FileMetadata metadata = 1;
}

message ListFilesRequest {
  string tenant_id = 1;
  int32 page = 2;
  int32 page_size = 3;
  optional string uploaded_by = 4;
  optional string mime_type = 5;
  optional string tag = 6;
}

message ListFilesResponse {
  repeated FileMetadata files = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

message GenerateUploadUrlRequest {
  string filename = 1;
  string content_type = 2;
  string tenant_id = 3;
  string uploaded_by = 4;
  map<string, string> tags = 5;
  optional uint32 expires_in_seconds = 6;
  int64 size_bytes = 7;
}

// REST -> gRPC フィールドマッピング:
// name -> filename
// mime_type -> content_type
// owner_id -> uploaded_by
// size -> size_bytes

message GenerateUploadUrlResponse {
  string file_id = 1;
  string upload_url = 2;
  uint32 expires_in_seconds = 3;
}

message CompleteUploadRequest {
  string file_id = 1;
  optional string checksum_sha256 = 3;
}

message CompleteUploadResponse {
  FileMetadata metadata = 1;
}

message GenerateDownloadUrlRequest {
  string id = 1;
}

message GenerateDownloadUrlResponse {
  string download_url = 1;
  int32 expires_in_seconds = 2;
}

message UpdateFileTagsRequest {
  string id = 1;
  map<string, string> tags = 2;
}

message UpdateFileTagsResponse {
  FileMetadata metadata = 1;
}

message DeleteFileRequest {
  string id = 1;
}

message DeleteFileResponse {}
```

---

## Kafka メッセージング設計

### ファイルアップロード完了イベント

クライアントが `/api/v1/files/{id}/complete` を呼び出した時点で、`k1s0.system.file.events.v1` トピックへ発行する設計。
イベントは usecase 層から発行され、`event_type=file.upload.completed` として送信される。

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.file.events.v1` |
| キー | file_id |
| パーティション戦略 | tenant_id によるハッシュ分散 |

**メッセージ例**

```json
{
  "event_type": "file.upload.completed",
  "file_id": "file_01JABCDEF1234567890",
  "tenant_id": "tenant-abc",
  "uploaded_by": "user-001",
  "status": "available",
  "actor_user_id": "user-001",
  "before": null,
  "after": {
    "file_id": "file_01JABCDEF1234567890",
    "status": "available",
    "checksum_sha256": "b1946ac92492d2347c6235b4d2611184"
  },
  "checksum_sha256": "b1946ac92492d2347c6235b4d2611184",
  "updated_at": "2026-02-20T10:05:00.000+00:00",
  "timestamp": "2026-02-20T10:05:01.000+00:00"
}
```

### ファイル削除イベント

ファイル削除時も同じ `k1s0.system.file.events.v1` トピックへ発行する。

**メッセージ例**

```json
{
  "event_type": "file.deleted",
  "file_id": "file_01JABCDEF1234567890",
  "tenant_id": "tenant-abc",
  "storage_key": "tenant-abc/reports/report-2026-02.pdf",
  "deleted_at": "2026-02-23T15:00:00.000+00:00",
  "timestamp": "2026-02-23T15:00:00.100+00:00"
}
```

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `FileMetadata` | エンティティ定義 |
| domain/repository | `FileMetadataRepository`, `FileStorageRepository` | リポジトリトレイト |
| domain/service | `FileDomainService` | テナント分離・ストレージキー生成ロジック |
| usecase | `ListFilesUsecase`, `GenerateUploadUrlUsecase`, `CompleteUploadUsecase`, `GetFileMetadataUsecase`, `GenerateDownloadUrlUsecase`, `DeleteFileUsecase`, `UpdateFileTagsUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `FileMetadataPostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/storage | `LocalFsStorageRepository` | ストレージ実装（ローカルFS） |
| infrastructure/messaging | `FileUploadedKafkaProducer`, `FileDeletedKafkaProducer` | Kafka プロデューサー |

### ドメインモデル

#### FileMetadata

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | ファイルの一意識別子 |
| `filename` | String | ファイル名（元のファイル名） |
| `size_bytes` | u64 | ファイルサイズ（バイト、`size` は後方互換 alias） |
| `content_type` | String | Content-Type（例: `application/pdf`、`mime_type` は後方互換 alias） |
| `uploaded_by` | String | アップロードしたユーザー ID（`owner_id` は後方互換 alias） |
| `tags` | HashMap\<String, String\> | 任意のタグ（最大 10 件） |
| `storage_path` | String | ストレージ上のオブジェクトパス（形式: `{tenant_id}/{filename}`、旧 `storage_key` から改名）。テナント ID はこのプレフィックスから取得する |
| `checksum` | Option\<String\> | SHA-256 チェックサム（アップロード完了後に記録、旧 `checksum_sha256` から改名） |
| `status` | String | ファイル状態（pending / available / deleted） |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

> **注意（C-01 監査対応）**: `FileMetadata` エンティティには `tenant_id` フィールドが存在しない。テナント ID は `storage_path` のプレフィックス（`/` より前の部分）から取得する。アクセス制御においては、リクエストヘッダー `x-tenant-id` の値と `storage_path` プレフィックスを `FileDomainService::can_access_tenant_resource()` で照合する。

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
                    │  │  GetFileMetadata / ListFiles /           │   │
                    │  │  GenerateUploadUrl / CompleteUpload /    │   │
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
                    │  └──────────────┘  │ LocalFsStorage         │  │
                    │  ┌──────────────┐  │ Repository             │  │
                    │  │ Config       │  │ (tokio::fs)            │  │
                    │  │ Loader       │  └────────────────────────┘  │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル例

### config.yaml（本番）
> ※ dev環境では省略可能なセクションがあります。


```yaml
app:
  name: "file"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8098
  grpc_port: 50051

database:
  url: "postgresql://app:@postgres.k1s0-system.svc.cluster.local:5432/k1s0_system"
  schema: "file"
  max_connections: 10
  min_connections: 2
  connect_timeout_seconds: 5

storage:
  backend: "local"
  path: "/data/files"
  base_url: "https://file-server.k1s0-system.svc.cluster.local:8098"
  max_file_size_bytes: 104857600

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"
  topic_events: "k1s0.system.file.events.v1"

auth:
  jwks_url: "http://auth-server.k1s0-system.svc.cluster.local:8080/.well-known/jwks.json"
  issuer: "https://auth.k1s0.example.com/realms/system"
  audience: "k1s0-system"
  jwks_cache_ttl_secs: 3600
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
  port: 8098
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
    - path: "secret/data/k1s0/system/file/database"
      key: "password"
      mountPath: "/vault/secrets/database-password"
    - path: "secret/data/k1s0/system/file/storage"
      key: "secret_access_key"
      mountPath: "/vault/secrets/storage-secret-key"
```

---

## デプロイ

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/file/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## 詳細設計ドキュメント

- [system-file-server-implementation.md](../../_common/implementation.md) -- 実装設計の詳細
- [system-file-server-deploy.md](../../_common/deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../../_common/deploy.md#共通関連ドキュメント) を参照。

- [RBAC設計.md](../../architecture/auth/RBAC設計.md) -- RBAC ロールモデル
- [system-server.md](../auth/server.md) -- system tier サーバー一覧
- [system-server-implementation.md](../../_common/implementation.md) -- system tier 実装設計

## Doc Sync (2026-03-03)

### Message/Field Corrections
- `FileMetadata.checksum_sha256` is available.
- `GenerateUploadUrlResponse.expires_in_seconds` is available.

### REST/gRPC Response Alignment
- `CompleteUploadResponse`: REST/gRPC ともに `FileMetadata` 形状。
- `DeleteFileResponse`: REST は `204 No Content`（ボディなし）、gRPC は空 message `{}`。
---

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。
