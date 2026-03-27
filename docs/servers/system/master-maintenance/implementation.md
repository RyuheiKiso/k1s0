# system-master-maintenance-server 実装設計

> **注記**: 本ドキュメントは master-maintenance-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../../_common/Rust共通実装.md) を参照。

system-master-maintenance-server（マスタメンテナンスサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。メタデータ駆動型の動的 CRUD を特徴とする。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス・値オブジェクト | なし（最内層） |
| usecase | ビジネスロジック（テーブル定義管理・動的CRUD・整合性チェック・インポート/エクスポート） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・プレゼンター | usecase, domain |
| infrastructure | 設定・DB接続・永続化・ルールエンジン・スキーマ生成・Kafka・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/master-maintenance/)

### ディレクトリ構成

```
regions/system/server/rust/master-maintenance/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── table_definition.rs          # TableDefinition エンティティ
│   │   │   ├── column_definition.rs         # ColumnDefinition エンティティ（型・制約・表示設定）
│   │   │   ├── table_relationship.rs        # TableRelationship エンティティ
│   │   │   ├── consistency_rule.rs          # ConsistencyRule エンティティ
│   │   │   ├── rule_condition.rs            # RuleCondition エンティティ
│   │   │   ├── display_config.rs            # DisplayConfig エンティティ
│   │   │   ├── change_log.rs                # ChangeLog エンティティ（before/after JSONB）
│   │   │   └── import_job.rs                # ImportJob エンティティ
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── table_definition_repository.rs
│   │   │   ├── column_definition_repository.rs
│   │   │   ├── table_relationship_repository.rs
│   │   │   ├── consistency_rule_repository.rs
│   │   │   ├── display_config_repository.rs
│   │   │   ├── dynamic_record_repository.rs # 動的レコード CRUD リポジトリトレイト
│   │   │   ├── change_log_repository.rs
│   │   │   └── import_job_repository.rs
│   │   ├── service/
│   │   │   ├── mod.rs
│   │   │   ├── metadata_service.rs          # メタデータ駆動のスキーマ解析
│   │   │   ├── query_builder_service.rs     # メタデータから動的 SQL を生成
│   │   │   ├── rule_engine_service.rs       # 整合性ルール評価サービス
│   │   │   └── schema_generator_service.rs  # JSON Schema 生成（フォーム自動生成用）
│   │   └── value_object/
│   │       ├── mod.rs
│   │       ├── data_type.rs                 # DataType 値オブジェクト
│   │       ├── domain_filter.rs             # DomainFilter 値オブジェクト
│   │       ├── operator.rs                  # Operator 値オブジェクト
│   │       ├── relationship_type.rs         # RelationshipType 値オブジェクト
│   │       └── rule_result.rs               # RuleResult 値オブジェクト
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── manage_table_definitions.rs      # テーブル定義管理
│   │   ├── manage_column_definitions.rs     # カラム定義管理
│   │   ├── manage_relationships.rs          # テーブル間リレーション管理
│   │   ├── manage_rules.rs                  # 整合性ルール管理
│   │   ├── manage_display_configs.rs        # 表示設定管理
│   │   ├── crud_records.rs                  # 動的レコード CRUD
│   │   ├── check_consistency.rs             # 整合性チェック
│   │   ├── rule_evaluator.rs                # ルール評価
│   │   ├── import_export.rs                 # CSV/Excel インポート・エクスポート
│   │   └── get_audit_logs.rs                # 変更監査ログ取得
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── table_handler.rs             # テーブル定義 REST ハンドラー
│   │   │   ├── record_handler.rs            # 動的レコード CRUD REST ハンドラー
│   │   │   ├── rule_handler.rs              # 整合性ルール REST ハンドラー
│   │   │   ├── relationship_handler.rs      # リレーション REST ハンドラー
│   │   │   ├── display_config_handler.rs    # 表示設定 REST ハンドラー
│   │   │   ├── import_export_handler.rs     # インポート/エクスポート REST ハンドラー
│   │   │   ├── audit_handler.rs             # 監査ログ REST ハンドラー
│   │   │   ├── error.rs                     # エラーレスポンス定義
│   │   │   └── integration_tests.rs         # 統合テスト用ヘルパー
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── master_maintenance_grpc.rs   # gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                      # JWT 認証ミドルウェア
│   │   │   ├── grpc_auth.rs                 # gRPC 認証ミドルウェア
│   │   │   └── rbac.rs                      # RBAC ミドルウェア
│   │   └── presenter/
│   │       ├── mod.rs
│   │       └── response.rs                  # レスポンスプレゼンター
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config/
│   │   │   └── mod.rs                       # 設定構造体・読み込み
│   │   ├── persistence/
│   │   │   ├── mod.rs
│   │   │   ├── table_definition_repo_impl.rs
│   │   │   ├── column_definition_repo_impl.rs
│   │   │   ├── consistency_rule_repo_impl.rs
│   │   │   ├── display_config_repo_impl.rs
│   │   │   ├── dynamic_record_repo_impl.rs  # 動的 SQL 実行
│   │   │   ├── table_relationship_repo_impl.rs
│   │   │   ├── change_log_repo_impl.rs
│   │   │   └── import_job_repo_impl.rs
│   │   ├── messaging/
│   │   │   ├── mod.rs
│   │   │   └── kafka_producer.rs            # Kafka プロデューサー
│   │   ├── rule_engine/
│   │   │   ├── mod.rs
│   │   │   └── zen_engine_adapter.rs        # ZEN Engine アダプター（整合性ルール評価）
│   │   ├── schema/
│   │   │   └── mod.rs                       # DB スキーマ管理
│   │   └── startup.rs                       # 起動シーケンス・DI
│   └── proto/                               # tonic-build 生成コード
├── config/
│   └── config.yaml
├── build.rs
├── Cargo.toml
└── Dockerfile
```

### 主要コンポーネント

#### ドメインサービス

- **MetadataService**: テーブル定義・カラム定義のメタデータを解析し、動的 CRUD に必要な情報を提供する
- **QueryBuilderService**: メタデータから動的 SQL（SELECT/INSERT/UPDATE/DELETE）を生成する。sqlx のプリペアドステートメントを使用する
- **RuleEngineService**: ZEN Engine（Rust ネイティブ）を使用した整合性ルール評価。JSON Decision Table でルールを管理する
- **SchemaGeneratorService**: カラム定義の型・制約・表示設定から JSON Schema を生成し、React クライアントのフォーム自動生成に使用する

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `ManageTableDefinitionsUseCase` | テーブル定義の CRUD |
| `ManageColumnDefinitionsUseCase` | カラム定義の CRUD |
| `CrudRecordsUseCase` | メタデータ駆動の動的レコード CRUD |
| `CheckConsistencyUseCase` | 整合性チェック（ZEN Engine 評価） |
| `ImportExportUseCase` | CSV/Excel インポート・エクスポート |

#### 外部連携

- **ZEN Engine** (`infrastructure/rule_engine/zen_engine_adapter.rs`): Rust ネイティブのルールエンジンで整合性ルールを評価する
- **Kafka Producer** (`infrastructure/messaging/kafka_producer.rs`): `k1s0.system.mastermaintenance.data_changed.v1` トピックにデータ変更を通知する

### SQLクエリ安全性

- **パラメータ化クエリ必須**: 全ての動的フィルタ条件はプレースホルダ（`$1`, `$2`, ...）を使用し、文字列結合による SQL 組み立てを禁止する
- **参照実装**: `consistency_rule_repo_impl.rs` の `find_all()` メソッドがパラメータ化クエリのリファレンス実装
- **明示的カラム指定**: `SELECT *` を使用せず、`FromRow` 構造体のフィールドに対応するカラムを明示的に列挙する
- **監査履歴**: C-1（SQLインジェクション脆弱性）の修正として `table_definition_repo_impl.rs` の `find_all()` をパラメータ化クエリに移行済み（2026-03-19）

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_MM_`
- 整合性ルール違反時は 422 Unprocessable Entity を返す

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | SQL生成・ルール評価 | mockall によるリポジトリモック |
| 統合テスト | REST/gRPC ハンドラー・動的 CRUD | axum-test / tonic テストクライアント |
| ルールエンジンテスト | ZEN Engine | テスト用 Decision Table による検証 |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・メタデータ駆動設計
- [Rust共通実装.md](../../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
