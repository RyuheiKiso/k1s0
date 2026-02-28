# system-file-server 設計

S3 互換ストレージを統一 API で抽象化するファイルサーバー。メタデータ管理・プリサインドURL・テナント分離を提供。

> **ガイド**: 実装例・設定ファイル・依存関係図は [server.guide.md](./server.guide.md) を参照。

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

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| S3 クライアント | aws-sdk-s3（S3/GCS/Ceph 互換） |

### 配置パス

配置: `regions/system/server/rust/file/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| ストレージバックエンド | aws-sdk-s3 クライアントで S3/GCS/Ceph 互換エンドポイントに接続。バックエンドは config で切り替え |
| メタデータ永続化 | PostgreSQL の `file` スキーマ（file_metadata テーブル）でメタデータを管理 |
| テナント分離 | バケット名またはオブジェクトキープレフィックスにテナント ID を付与（例: `tenant-abc/path/to/file`） |
| プリサインドURL | aws-sdk-s3 の presigned request 機能で TTL 付き署名 URL を発行 |
| アップロード完了通知 | クライアントがコールバック API を呼び出した時点で Kafka イベントを発行 |
| 認可 | 参照・ダウンロードは `sys_auditor`、アップロード・タグ更新は `sys_operator`、削除は `sys_operator`（所有者）/ `sys_admin`（全体） |
| ポート | ホスト側 8098（内部 8080） |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_FILE_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/files` | ファイル一覧取得 | `sys_auditor` 以上 |
| POST | `/api/v1/files` | ファイルアップロード（プリサインドURL発行） | `sys_operator` 以上 |
| GET | `/api/v1/files/:id` | ファイルメタデータ取得 | `sys_auditor` 以上 |
| POST | `/api/v1/files/:id/complete` | アップロード完了通知 | `sys_operator` 以上 |
| DELETE | `/api/v1/files/:id` | ファイル削除 | `sys_operator` 以上 |
| PUT | `/api/v1/files/:id/tags` | タグ更新 | `sys_operator` 以上 |
| GET | `/api/v1/files/:id/download-url` | ダウンロードプリサインドURL発行 | `sys_auditor` 以上 |
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

#### POST /api/v1/files/upload-url

アップロード用のプリサインドURLを発行する。クライアントはこの URL に対して直接 HTTP PUT でファイルをアップロードする。アップロード完了後、`/api/v1/files/:id/complete` を呼び出してサーバーに完了を通知する。

#### GET /api/v1/files/:id

ファイルのメタデータを取得する。ストレージへの直接アクセスは行わない。

#### GET /api/v1/files/:id/download-url

ダウンロード用のプリサインドURLを発行する。クライアントはこの URL に対して直接 HTTP GET でファイルをダウンロードする。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `expires_in_seconds` | int | No | 3600 | URLの有効期限（秒）。最大 86400 |

#### PUT /api/v1/files/:id/tags

ファイルのタグを更新する。既存タグは上書きされる（全置換）。

#### DELETE /api/v1/files/:id

ファイルをメタデータとストレージの両方から削除する。

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

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.file.uploaded.v1` |
| キー | file_id |
| パーティション戦略 | tenant_id によるハッシュ分散 |

### ファイル削除イベント

ファイル削除時に `k1s0.system.file.deleted.v1` トピックへ発行する。

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
| infrastructure/storage | `S3FileStorageRepository` | ストレージ実装 |
| infrastructure/messaging | `FileUploadedKafkaProducer`, `FileDeletedKafkaProducer` | Kafka プロデューサー |

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

---

## デプロイ

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/file/database` |
| S3 シークレットキー | `secret/data/k1s0/system/file/storage` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## 詳細設計ドキュメント

- [system-file-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-file-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [RBAC設計.md](../../architecture/auth/RBAC設計.md) -- RBAC ロールモデル
- [system-server.md](../auth/server.md) -- system tier サーバー一覧
- [system-server-implementation.md](../_common/implementation.md) -- system tier 実装設計
