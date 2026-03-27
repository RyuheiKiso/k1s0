# system-search-server 実装設計

> **注記**: 本ドキュメントは search-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../../_common/Rust共通実装.md) を参照。

system-search-server（全文検索サーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（インデックス管理・ドキュメント登録・全文検索） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・キャッシュ・Kafka・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/search/)

### ディレクトリ構成

```
regions/system/server/rust/search/
├── src/
│   ├── main.rs                                          # エントリポイント
│   ├── lib.rs                                           # ライブラリルート
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   └── search_index.rs                          # SearchIndex・SearchDocument・SearchQuery・SearchResult エンティティ
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   └── search_repository.rs                     # SearchRepository トレイト（単一トレイト）
│   │   └── service/
│   │       ├── mod.rs
│   │       └── search_domain_service.rs                 # 検索クエリ構築・ファセット集計ロジック
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── create_index.rs                              # インデックス作成
│   │   ├── list_indices.rs                              # インデックス一覧
│   │   ├── index_document.rs                            # ドキュメント登録
│   │   ├── search.rs                                    # 全文検索
│   │   └── delete_document.rs                           # ドキュメント削除
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── search_handler.rs                        # axum REST ハンドラー
│   │   │   └── health.rs                                # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── search_grpc.rs                           # gRPC サービス実装
│   │   │   └── tonic_service.rs                         # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                                  # JWT 認証ミドルウェア
│   │   │   └── rbac.rs                                  # RBAC ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── search_opensearch.rs                     # SearchRepository OpenSearch 実装
│   │       ├── search_postgres.rs                       # SearchRepository PostgreSQL フォールバック実装
│   │       └── cached_search_repository.rs              # キャッシュ付き SearchRepository ラッパー
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                                    # 設定構造体・読み込み
│   │   ├── database.rs                                  # DB 接続プール
│   │   ├── cache.rs                                     # moka キャッシュ（インデックス一覧 TTL 30秒）
│   │   ├── kafka_consumer.rs                            # Kafka Consumer（非同期インデックス要求）
│   │   ├── kafka_producer.rs                            # Kafka Producer（インデックス完了通知）
│   │   └── startup.rs                                   # 起動シーケンス・DI
│   └── proto/                                           # tonic-build 生成コード
│       ├── mod.rs
│       ├── k1s0.system.search.v1.rs
│       └── k1s0.system.common.v1.rs
├── tests/
│   ├── integration_test.rs                              # 統合テスト
│   └── usecase_test.rs                                  # ユースケーステスト
├── config/
│   └── config.yaml
├── build.rs
├── Cargo.toml
└── Dockerfile
```

### 主要コンポーネント

#### ドメインサービス

- **SearchDomainService**: 検索クエリの構築・ファセット集計ロジックを担当する。OpenSearch クエリ DSL への変換は infrastructure 層で行う

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `CreateIndexUseCase` | インデックス作成（重複チェック含む） |
| `ListIndicesUseCase` | 登録済みインデックス一覧取得 |
| `IndexDocumentUseCase` | ドキュメントのインデックス登録・更新 |
| `SearchUseCase` | キーワード・フィルタ・ファセットによる全文検索 |
| `DeleteDocumentUseCase` | インデックスからのドキュメント削除 |

#### 外部連携

- **OpenSearch Repository** (`adapter/repository/search_opensearch.rs`): opensearch-rs を使用して OpenSearch クラスターと通信する
- **PostgreSQL Repository** (`adapter/repository/search_postgres.rs`): OpenSearch 障害時のフォールバック実装。`plainto_tsquery('simple', ...)` による全文検索、`QueryBuilder` を使った動的 WHERE 句によるフィルタ適用、GROUP BY によるファセット集計を提供する。`to_tsquery` は使用しない（ユーザー入力をそのまま渡すとシンタックスエラーになるため）
- **Kafka Consumer** (`infrastructure/kafka_consumer.rs`): `k1s0.system.search.index.requested.v1` トピックから非同期インデックス要求を受信する
- **Kafka Producer** (`infrastructure/kafka_producer.rs`): `k1s0.system.search.indexed.v1` トピックにインデックス完了通知を配信する

#### キャッシュ戦略

- moka でインデックス一覧・ドキュメント数等のステータスを TTL 30 秒キャッシュする
- `cached_search_repository.rs` で SearchRepository をラップし、透過的にキャッシュを適用する

#### リポジトリフォールバック

Repository の選択順序は `OpenSearch -> PostgreSQL -> InMemory`。OpenSearch 接続失敗時に PostgreSQL、さらに PostgreSQL 未接続時に InMemory へフォールバックする。

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_SEARCH_`
- OpenSearch 接続障害時は `SYS_SEARCH_OPENSEARCH_ERROR`（502）を返却し、フォールバックリポジトリへ切り替える

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | 検索クエリ構築・ファセット集計 | mockall によるリポジトリモック |
| ユースケーステスト | インデックス CRUD・検索 | `usecase_test.rs` でモックリポジトリを使用 |
| 統合テスト | REST/gRPC ハンドラー | `integration_test.rs` で axum-test / tonic テストクライアント |
| フォールバックテスト | リポジトリ切り替え | OpenSearch 未接続状態での PostgreSQL フォールバック検証 |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・Kafka メッセージング設計
- [Rust共通実装.md](../../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
