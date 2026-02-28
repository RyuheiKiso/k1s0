# system-search-server 設計

OpenSearch 連携の全文検索サーバー。インデックス管理・全文検索クエリ・Kafka 非同期インデックスを提供。

> **ガイド**: 実装例・設定ファイル・依存関係図は [server.guide.md](./server.guide.md) を参照。

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
| ポート | ホスト側 8094（内部 8080）、gRPC 9090 |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_SEARCH_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/search/indices` | インデックス作成 | `sys_admin` のみ |
| GET | `/api/v1/search/indices` | インデックス一覧取得 | `sys_auditor` 以上 |
| POST | `/api/v1/search/index` | ドキュメント登録（`index_name` はリクエストボディで指定） | `sys_operator` 以上 |
| DELETE | `/api/v1/search/index/:index_name/:id` | ドキュメント削除 | `sys_operator` 以上 |
| POST | `/api/v1/search` | 全文検索 | `sys_auditor` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### POST /api/v1/indices

新しいインデックスを作成する。`mapping` フィールドで任意の JSON マッピング定義を指定可能（省略時はデフォルト空オブジェクト）。

#### GET /api/v1/indices

登録済みインデックスの一覧を取得する。

#### POST /api/v1/search/index

ドキュメントをインデックスに登録する。`index_name` はリクエストボディで指定する。

#### POST /api/v1/search

指定されたクエリで全文検索を実行する。`index_name` でインデックスを指定し、`from` / `size` でページネーション制御する。

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `index_name` | String | （必須） | 検索対象インデックス名 |
| `query` | String | （必須） | 検索キーワード |
| `from` | u32 | 0 | オフセット（スキップ件数） |
| `size` | u32 | 10 | 取得件数 |

#### DELETE /api/v1/search/index/:index_name/:id

指定インデックスから特定ドキュメントを削除する。成功時は 204 No Content（レスポンスボディなし）。

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
