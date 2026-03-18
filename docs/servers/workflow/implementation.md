# system-workflow-server 実装設計

> **注記**: 本ドキュメントは workflow-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../_common/Rust共通実装.md) を参照。

system-workflow-server（ワークフローオーケストレーションサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（ワークフロー定義管理・インスタンス管理・人間タスク管理・期日監視） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・Kafka・scheduler 連携・通知連携・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/workflow/)

### ディレクトリ構成

```
regions/system/server/rust/workflow/
├── src/
│   ├── main.rs                                          # エントリポイント
│   ├── lib.rs                                           # ライブラリルート
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── workflow_definition.rs                   # WorkflowDefinition エンティティ
│   │   │   ├── workflow_step.rs                         # WorkflowStep エンティティ
│   │   │   ├── workflow_instance.rs                     # WorkflowInstance エンティティ（状態機械）
│   │   │   └── workflow_task.rs                         # WorkflowTask エンティティ（人間タスク）
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── workflow_definition_repository.rs        # WorkflowDefinitionRepository トレイト
│   │   │   ├── workflow_instance_repository.rs          # WorkflowInstanceRepository トレイト
│   │   │   └── workflow_task_repository.rs              # WorkflowTaskRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── workflow_domain_service.rs               # ステップ遷移計算・タスク生成・期日計算・状態機械ロジック
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── create_workflow.rs                           # ワークフロー定義作成
│   │   ├── update_workflow.rs                           # ワークフロー定義更新
│   │   ├── delete_workflow.rs                           # ワークフロー定義削除
│   │   ├── get_workflow.rs                              # ワークフロー定義取得
│   │   ├── list_workflows.rs                            # ワークフロー定義一覧
│   │   ├── start_instance.rs                            # インスタンス起動
│   │   ├── get_instance.rs                              # インスタンス取得
│   │   ├── list_instances.rs                            # インスタンス一覧
│   │   ├── cancel_instance.rs                           # インスタンスキャンセル
│   │   ├── list_tasks.rs                                # タスク一覧
│   │   ├── approve_task.rs                              # タスク承認
│   │   ├── reject_task.rs                               # タスク却下
│   │   ├── reassign_task.rs                             # タスク再割当
│   │   ├── check_overdue_tasks.rs                       # 期日超過タスク検出と通知発行
│   │   └── postgres_support.rs                          # PostgreSQL 固有のサポートユーティリティ
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── dto.rs                                   # AppState + DTO 構造体
│   │   │   ├── workflow_handler.rs                      # ワークフロー定義 REST ハンドラー
│   │   │   ├── instance_handler.rs                      # インスタンス REST ハンドラー
│   │   │   ├── task_handler.rs                          # タスク REST ハンドラー
│   │   │   └── health.rs                                # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── workflow_grpc.rs                         # gRPC サービス実装
│   │   │   └── tonic_service.rs                         # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                                  # JWT 認証ミドルウェア
│   │   │   ├── rbac.rs                                  # RBAC ミドルウェア
│   │   │   └── grpc_auth.rs                             # gRPC 認証インターセプター
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── definition_postgres.rs                   # WorkflowDefinitionRepository PostgreSQL 実装
│   │       ├── instance_postgres.rs                     # WorkflowInstanceRepository PostgreSQL 実装
│   │       └── task_postgres.rs                         # WorkflowTaskRepository PostgreSQL 実装
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                                    # 設定構造体・読み込み
│   │   ├── database.rs                                  # PostgreSQL 接続プール
│   │   ├── in_memory.rs                                 # テスト用インメモリリポジトリ
│   │   ├── kafka_producer.rs                            # Kafka プロデューサー（状態変化通知）
│   │   ├── notification_request_producer.rs             # 通知リクエスト Kafka プロデューサー
│   │   ├── scheduler_registration.rs                    # scheduler-server へのジョブ登録
│   │   └── startup.rs                                   # 起動シーケンス・DI
│   └── proto/                                           # tonic-build 生成コード
│       ├── mod.rs
│       ├── k1s0.system.workflow.v1.rs
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

- **WorkflowDomainService**: ステップ遷移計算（`on_approve`/`on_reject` による分岐）、タスク生成（`human_task` ステップで担当者・期日を設定）、期日計算（`timeout_hours` からの算出）、状態機械ロジック（`pending -> running -> completed/cancelled/failed`）を担当する

#### 状態機械

**インスタンス状態**: `pending` -> `running` -> `completed` / `cancelled` / `failed`

**タスク状態**: `pending` -> `assigned` -> `approved` / `rejected`

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `CreateWorkflowUseCase` / `UpdateWorkflowUseCase` / `DeleteWorkflowUseCase` | ワークフロー定義 CRUD |
| `GetWorkflowUseCase` / `ListWorkflowsUseCase` | ワークフロー定義取得・一覧 |
| `StartInstanceUseCase` | インスタンス起動・最初のステップのタスク生成 |
| `GetInstanceUseCase` / `ListInstancesUseCase` | インスタンス取得・一覧 |
| `CancelInstanceUseCase` | インスタンスキャンセル（completed/cancelled 済みは拒否） |
| `ListTasksUseCase` | タスク一覧（担当者・ステータス・期日超過フィルタ） |
| `ApproveTaskUseCase` | タスク承認・次ステップ遷移・最終ステップ承認時のインスタンス完了 |
| `RejectTaskUseCase` | タスク却下・`on_reject` に基づく差し戻しまたはインスタンス失敗 |
| `ReassignTaskUseCase` | タスク再割当（`pending`/`assigned` 状態のみ） |
| `CheckOverdueTasksUseCase` | 期日超過タスク検出・notification-server 向けイベント発行 |

#### 外部連携

- **Kafka Producer** (`infrastructure/kafka_producer.rs`): `k1s0.system.workflow.state_changed.v1` にインスタンス・タスク状態変化イベントを配信する
- **Notification Request Producer** (`infrastructure/notification_request_producer.rs`): 期日超過タスク検出時に `k1s0.system.notification.requested.v1` へ通知リクエストを送信する
- **Scheduler Registration** (`infrastructure/scheduler_registration.rs`): 起動時に scheduler-server へ期日超過チェックジョブ（15 分ごと）を登録する
- **InMemory Repository** (`infrastructure/in_memory.rs`): テスト用のインメモリリポジトリ実装

#### REST ハンドラー分割

workflow-server は REST ハンドラーを機能ごとに 3 ファイルに分割している。

- `workflow_handler.rs`: ワークフロー定義の CRUD
- `instance_handler.rs`: インスタンスの起動・取得・キャンセル
- `task_handler.rs`: タスクの承認・却下・再割当

`dto.rs` で AppState と共通 DTO 構造体を定義し、各ハンドラーから参照する。

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_WORKFLOW_`
- ワークフロー定義のステップ参照不正は `SYS_WORKFLOW_STEP_REF_ERROR`（400）を返却する
- インスタンスの不正なステータス遷移は `SYS_WORKFLOW_INSTANCE_INVALID_STATUS`（409）を返却する
- タスクの不正な状態での操作は `SYS_WORKFLOW_TASK_INVALID_STATUS`（409）を返却する

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | ステップ遷移計算・状態機械ロジック | ドメインサービスの直接テスト |
| ユースケーステスト | ワークフロー CRUD・承認/却下フロー | `usecase_test.rs` でモック/インメモリリポジトリを使用 |
| 統合テスト | REST/gRPC ハンドラー | `integration_test.rs` で axum-test / tonic テストクライアント |
| フロー E2E テスト | 定義作成 -> インスタンス起動 -> 承認 -> 完了 | PostgreSQL を使用した E2E フロー検証 |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・Kafka メッセージング設計・DB スキーマ
- [Rust共通実装.md](../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
