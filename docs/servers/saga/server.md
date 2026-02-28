# system-saga-server 設計

system tier の Saga Orchestrator 設計を定義する。YAML ベースのワークフロー定義に基づく分散トランザクションオーケストレーション（5 ステップ以上）を担い、Rust で実装する。

## 概要

system tier の Saga Orchestrator は以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| Saga の開始・実行 | YAML ワークフロー定義に基づき、gRPC 経由で各サービスのステップを順次実行する |
| 補償トランザクション | ステップ失敗時に実行済みステップを逆順で補償（ロールバック）する |
| 状態永続化 | Saga の状態とステップログを PostgreSQL にトランザクショナルに記録する |
| 起動時リカバリ | サーバー再起動時に未完了の Saga を自動検出・再開する |
| ワークフロー管理 | YAML ファイルからの起動時ロードおよび REST/gRPC API 経由の動的登録 |
| Kafka イベント発行 | Saga の状態遷移イベントを Kafka に非同期配信する |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

### 配置パス

配置: `regions/system/server/rust/saga/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) の Saga パターンに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| 外部エンジン | 不使用（Temporal 等は導入しない、自前の軽量 Orchestrator） |
| サービス呼び出し | gRPC（静的サービスレジストリ by config.yaml） |
| 補償順序 | 実行の逆順（current_step - 1 → 0） |
| タイムアウト | `tokio::time::timeout`（ステップ毎、デフォルト 30 秒） |
| リトライ | Exponential backoff、最大 3 回（ステップ毎設定可） |
| 状態永続化 | PostgreSQL トランザクション（saga_states UPDATE + saga_step_logs INSERT を原子的に） |
| 起動時リカバリ | `status IN ('STARTED','RUNNING','COMPENSATING')` を検索し自動再開 |
| ワークフロー定義 | YAML ファイル（起動時ロード + API 経由の動的登録） |
| 並行実行 | `tokio::spawn` でバックグラウンド実行 |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_SAGA_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/sagas` | Saga 開始 | `sys_operator` 以上 |
| GET | `/api/v1/sagas` | Saga 一覧取得 | `sys_auditor` 以上 |
| GET | `/api/v1/sagas/:saga_id` | Saga 詳細取得（ステップログ含む） | `sys_auditor` 以上 |
| POST | `/api/v1/sagas/:saga_id/cancel` | Saga キャンセル | `sys_operator` 以上 |
| POST | `/api/v1/sagas/:saga_id/compensate` | Saga キャンセル（`/cancel` のエイリアス） | `sys_operator` 以上 |
| POST | `/api/v1/sagas/workflows` | ワークフロー登録 | `sys_operator` 以上 |
| GET | `/api/v1/sagas/workflows` | ワークフロー一覧取得 | `sys_auditor` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要（公開） |
| GET | `/readyz` | レディネスチェック | 不要（公開） |
| GET | `/metrics` | Prometheus メトリクス | 不要（公開） |

#### POST /api/v1/sagas

指定されたワークフローで新しい Saga を開始する。Saga は非同期で実行され、即座に saga_id が返却される。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `workflow_name` | string | Yes | 実行するワークフロー名 |
| `payload` | object | No | 各ステップに渡す JSON ペイロード |
| `correlation_id` | string | No | 業務相関 ID（トレーサビリティ用） |
| `initiated_by` | string | No | 呼び出し元のサービス名 |

**レスポンスフィールド（201 Created）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `saga_id` | string (UUID) | 作成された Saga の ID |
| `status` | string | 初期ステータス（`STARTED`） |

#### GET /api/v1/sagas

Saga の一覧をページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |
| `workflow_name` | string | No | - | ワークフロー名でフィルタ |
| `status` | string | No | - | ステータスでフィルタ（`STARTED`, `RUNNING`, `COMPLETED`, `COMPENSATING`, `FAILED`, `CANCELLED`） |
| `correlation_id` | string | No | - | 業務相関 ID でフィルタ |

#### GET /api/v1/sagas/:saga_id

Saga の詳細情報とステップログを取得する。

#### POST /api/v1/sagas/:saga_id/cancel

実行中の Saga をキャンセルする。終端状態（COMPLETED / FAILED / CANCELLED）の Saga はキャンセルできない。

#### POST /api/v1/sagas/workflows

YAML 形式のワークフロー定義を登録する。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `workflow_yaml` | string | Yes | YAML 形式のワークフロー定義文字列 |

#### GET /api/v1/sagas/workflows

登録済みワークフロー定義の一覧を取得する。

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_SAGA_NOT_FOUND` | 404 | 指定された Saga が見つからない |
| `SYS_SAGA_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_SAGA_CONFLICT` | 409 | 終端状態の Saga に対する不正な操作 |
| `SYS_SAGA_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

proto ファイルは [API設計.md](../../architecture/api/API設計.md) D-009 の命名規則に従い、以下に配置する。

```
api/proto/k1s0/system/saga/v1/saga.proto
```

```protobuf
// api/proto/k1s0/system/saga/v1/saga.proto
syntax = "proto3";
package k1s0.system.saga.v1;

import "k1s0/system/common/v1/types.proto";

service SagaService {
  rpc StartSaga(StartSagaRequest) returns (StartSagaResponse);
  rpc GetSaga(GetSagaRequest) returns (GetSagaResponse);
  rpc ListSagas(ListSagasRequest) returns (ListSagasResponse);
  rpc CancelSaga(CancelSagaRequest) returns (CancelSagaResponse);
  rpc RegisterWorkflow(RegisterWorkflowRequest) returns (RegisterWorkflowResponse);
  rpc ListWorkflows(ListWorkflowsRequest) returns (ListWorkflowsResponse);
}

message SagaStateProto {
  string id = 1;
  string workflow_name = 2;
  int32 current_step = 3;
  string status = 4;
  bytes payload = 5;
  string correlation_id = 6;
  string initiated_by = 7;
  string error_message = 8;
  k1s0.system.common.v1.Timestamp created_at = 9;
  k1s0.system.common.v1.Timestamp updated_at = 10;
}

message SagaStepLogProto {
  string id = 1;
  string saga_id = 2;
  int32 step_index = 3;
  string step_name = 4;
  string action = 5;       // EXECUTE, COMPENSATE
  string status = 6;       // SUCCESS, FAILED, TIMEOUT, SKIPPED
  bytes request_payload = 7;
  bytes response_payload = 8;
  string error_message = 9;
  k1s0.system.common.v1.Timestamp started_at = 10;
  k1s0.system.common.v1.Timestamp completed_at = 11;
}

message WorkflowSummary {
  string name = 1;
  int32 step_count = 2;
  repeated string step_names = 3;
}
```

---

## Saga 状態遷移

### ステータス一覧

| ステータス | 説明 |
| --- | --- |
| `STARTED` | Saga が作成された初期状態 |
| `RUNNING` | ステップが実行中 |
| `COMPLETED` | 全ステップが正常完了（終端状態） |
| `COMPENSATING` | ステップ失敗により補償処理を実行中 |
| `FAILED` | 補償処理完了後の失敗状態（終端状態） |
| `CANCELLED` | ユーザーによるキャンセル（終端状態） |

### 状態遷移図

```
              ┌─────────────────────────────────────────────┐
              │                                             │
  STARTED ──▶ RUNNING ──▶ COMPLETED (終端)                 │
              │                                             │
              │ ステップ失敗                                 │
              ▼                                             │
          COMPENSATING ──▶ FAILED (終端)                    │
              │                                             │
              │          ユーザーキャンセル                   │
              └────────────────────▶ CANCELLED (終端) ◀─────┘
```

---

## ワークフロー定義

### フィールド定義

| フィールド | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `name` | string | Yes | - | ワークフロー名（一意） |
| `steps` | array | Yes | - | ステップ定義の配列（1 個以上） |
| `steps[].name` | string | Yes | - | ステップ名 |
| `steps[].service` | string | Yes | - | サービス名（config.yaml の services セクションで解決） |
| `steps[].method` | string | Yes | - | gRPC メソッド名（`ServiceName.MethodName` 形式） |
| `steps[].compensate` | string | No | null | 補償メソッド名（未設定時はスキップ） |
| `steps[].timeout_secs` | int | No | 30 | ステップのタイムアウト秒数 |
| `steps[].retry.max_attempts` | int | No | 3 | 最大リトライ回数 |
| `steps[].retry.backoff` | string | No | exponential | バックオフ方式 |
| `steps[].retry.initial_interval_ms` | int | No | 1000 | 初回リトライ間隔（ミリ秒） |

### リトライ・バックオフ計算

Exponential backoff の遅延: `delay_ms = initial_interval_ms * 2^attempt`

| リトライ回数 | initial_interval_ms=1000 の場合 |
| --- | --- |
| 1 回目 | 1,000 ms |
| 2 回目 | 2,000 ms |
| 3 回目 | 4,000 ms |

---

## データベース設計

### スキーマ

データベースは `saga` スキーマに配置する。マイグレーションファイルは `regions/system/database/saga-db/migrations/` に格納する。

#### saga_states テーブル

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| `id` | UUID | PK, DEFAULT gen_random_uuid() | Saga ID |
| `workflow_name` | VARCHAR(255) | NOT NULL | ワークフロー名 |
| `current_step` | INT | NOT NULL, DEFAULT 0 | 現在のステップインデックス |
| `status` | VARCHAR(50) | NOT NULL, DEFAULT 'STARTED', CHECK制約 | Saga ステータス |
| `payload` | JSONB | - | 実行ペイロード |
| `correlation_id` | VARCHAR(255) | - | 業務相関 ID |
| `initiated_by` | VARCHAR(255) | - | 呼び出し元 |
| `error_message` | TEXT | - | エラーメッセージ |
| `created_at` | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| `updated_at` | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時（トリガー自動更新） |

**インデックス:**
- `idx_saga_states_workflow_name` -- workflow_name
- `idx_saga_states_status` -- status
- `idx_saga_states_correlation_id` -- correlation_id（WHERE IS NOT NULL）
- `idx_saga_states_created_at` -- created_at

#### saga_step_logs テーブル

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| `id` | UUID | PK, DEFAULT gen_random_uuid() | ログ ID |
| `saga_id` | UUID | NOT NULL, FK → saga_states(id) ON DELETE CASCADE | 所属する Saga |
| `step_index` | INT | NOT NULL | ステップインデックス |
| `step_name` | VARCHAR(255) | NOT NULL | ステップ名 |
| `action` | VARCHAR(50) | NOT NULL, CHECK ('EXECUTE', 'COMPENSATE') | 実行アクション |
| `status` | VARCHAR(50) | NOT NULL, CHECK ('SUCCESS', 'FAILED', 'TIMEOUT', 'SKIPPED') | 実行結果 |
| `request_payload` | JSONB | - | リクエストペイロード |
| `response_payload` | JSONB | - | レスポンスペイロード |
| `error_message` | TEXT | - | エラーメッセージ |
| `started_at` | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 開始日時 |
| `completed_at` | TIMESTAMPTZ | - | 完了日時 |

**インデックス:**
- `idx_saga_step_logs_saga_id_step_index` -- (saga_id, step_index)

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `SagaState`, `SagaStepLog`, `WorkflowDefinition` | エンティティ定義・状態遷移 |
| domain/repository | `SagaRepository`, `WorkflowRepository` | リポジトリトレイト |
| usecase | `StartSagaUseCase`, `ExecuteSagaUseCase`, `GetSagaUseCase`, `ListSagasUseCase`, `CancelSagaUseCase`, `RegisterWorkflowUseCase`, `ListWorkflowsUseCase`, `RecoverSagasUseCase` | ユースケース |
| adapter/handler | REST ハンドラー | プロトコル変換（axum） |
| adapter/grpc | gRPC サービス | プロトコル変換（tonic） |
| adapter/repository | `SagaPostgresRepository`, `InMemoryWorkflowRepository` | リポジトリ実装 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/database | DatabaseConfig | DB 接続設定 |
| infrastructure/grpc_caller | `GrpcStepCaller`, `ServiceRegistry`, `TonicGrpcCaller` | gRPC 動的呼び出し |
| infrastructure/kafka_producer | `SagaEventPublisher`, `KafkaProducer` | Kafka イベント発行 |

### ドメインモデル

#### SagaState

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `saga_id` | UUID | Saga の一意識別子 |
| `workflow_name` | string | ワークフロー名 |
| `current_step` | i32 | 現在のステップインデックス |
| `status` | SagaStatus | Saga ステータス（STARTED / RUNNING / COMPLETED / COMPENSATING / FAILED / CANCELLED） |
| `payload` | JSON | 各ステップに渡すペイロード |
| `correlation_id` | Option\<string\> | 業務相関 ID |
| `initiated_by` | Option\<string\> | 呼び出し元サービス名 |
| `error_message` | Option\<string\> | エラーメッセージ |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

**メソッド:**
- `new()` -- 初期状態（STARTED）で作成
- `advance_step()` -- ステップを進める（status=RUNNING）
- `complete()` -- 正常完了（status=COMPLETED）
- `start_compensation(error)` -- 補償開始（status=COMPENSATING）
- `fail(error)` -- 失敗確定（status=FAILED）
- `cancel()` -- キャンセル（status=CANCELLED）
- `is_terminal()` -- 終端状態かどうか

#### SagaStepLog

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | ログの一意識別子 |
| `saga_id` | UUID | 所属する Saga ID |
| `step_index` | i32 | ステップインデックス |
| `step_name` | string | ステップ名 |
| `action` | StepAction | EXECUTE / COMPENSATE |
| `status` | StepStatus | SUCCESS / FAILED / TIMEOUT / SKIPPED |
| `request_payload` | Option\<JSON\> | リクエストペイロード |
| `response_payload` | Option\<JSON\> | レスポンスペイロード |
| `error_message` | Option\<string\> | エラーメッセージ |
| `started_at` | DateTime\<Utc\> | 開始日時 |
| `completed_at` | Option\<DateTime\<Utc\>\> | 完了日時 |

#### WorkflowDefinition

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `name` | string | ワークフロー名 |
| `steps` | Vec\<WorkflowStep\> | ステップ定義の配列 |

### ディレクトリ構成

```
regions/system/server/rust/saga/
├── src/
│   ├── main.rs                              # エントリポイント + InMemorySagaRepository
│   ├── lib.rs                               # ライブラリクレート
│   ├── proto/
│   │   └── mod.rs                           # Proto include（codegen後に有効化）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── saga_state.rs                # SagaState / SagaStatus
│   │   │   ├── saga_step_log.rs             # SagaStepLog / StepAction / StepStatus
│   │   │   └── workflow.rs                  # WorkflowDefinition / WorkflowStep / RetryConfig
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── saga_repository.rs           # SagaRepository トレイト
│   │       └── workflow_repository.rs       # WorkflowRepository トレイト
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── start_saga.rs                    # Saga 開始
│   │   ├── execute_saga.rs                  # Saga 実行エンジン（核心）
│   │   ├── get_saga.rs                      # Saga 詳細取得
│   │   ├── list_sagas.rs                    # Saga 一覧取得
│   │   ├── cancel_saga.rs                   # Saga キャンセル
│   │   ├── register_workflow.rs             # ワークフロー登録
│   │   ├── list_workflows.rs                # ワークフロー一覧
│   │   └── recover_sagas.rs                 # 起動時リカバリ
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs                       # AppState / router() / ErrorResponse
│   │   │   ├── saga_handler.rs              # REST ハンドラー
│   │   │   └── error.rs                     # SagaError / SYS_SAGA_* エラーコード
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── saga_grpc.rs                 # gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── saga_postgres.rs             # PostgreSQL リポジトリ
│   │       └── workflow_in_memory.rs        # InMemory ワークフローリポジトリ
│   └── infrastructure/
│       ├── mod.rs
│       ├── config.rs                        # Config / AppConfig / ServerConfig
│       ├── database.rs                      # DatabaseConfig
│       ├── kafka_producer.rs                # SagaEventPublisher / KafkaProducer
│       └── grpc_caller.rs                   # GrpcStepCaller / ServiceRegistry / TonicGrpcCaller
├── config/
│   ├── config.yaml                          # 本番設定
│   └── config.dev.yaml                      # 開発設定
├── workflows/
│   └── order-fulfillment.yaml               # サンプルワークフロー
├── tests/
│   ├── integration_test.rs                  # REST API 統合テスト
│   ├── workflow_engine_test.rs              # ワークフローエンジンテスト
│   ├── postgres_repository_test.rs          # DB テスト
│   └── kafka_integration_test.rs            # Kafka テスト
├── build.rs                                 # tonic-build（proto codegen）
├── Cargo.toml
└── Cargo.lock
```

### 依存関係図

```
                    ┌─────────────────────────────────────────────────────┐
                    │                    adapter 層                       │
                    │  ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │
                    │  │ REST Handler │  │ gRPC Handler │  │Repository│ │
                    │  └──────┬───────┘  └──────┬───────┘  └─────┬────┘ │
                    │         │                  │                │      │
                    └─────────┼──────────────────┼────────────────┼──────┘
                              │                  │                │
                    ┌─────────▼──────────────────▼────────────────│──────┐
                    │                   usecase 層                │      │
                    │  StartSaga / ExecuteSaga / GetSaga /        │      │
                    │  ListSagas / CancelSaga /                   │      │
                    │  RegisterWorkflow / ListWorkflows /         │      │
                    │  RecoverSagas                               │      │
                    └─────────┬──────────────────────────────────┘──────┘
                              │
              ┌───────────────┼───────────────────────┐
              │               │                       │
    ┌─────────▼──────┐  ┌────▼───────────┐  ┌───────▼─────────────┐
    │  domain/entity  │  │ domain/        │  │ domain/repository   │
    │  SagaState,     │  │ (no domain     │  │ SagaRepository      │
    │  SagaStepLog,   │  │  service)      │  │ WorkflowRepository  │
    │  Workflow        │  │               │  │ (trait)              │
    └────────────────┘  └────────────────┘  └──────────┬──────────┘
                                                       │
                    ┌──────────────────────────────────┼──────────────┐
                    │             infrastructure 層         │              │
                    │  ┌──────────────┐  ┌─────────────▼──────────┐  │
                    │  │ ServiceReg + │  │ PostgreSQL Repository  │  │
                    │  │ GrpcCaller   │  │ InMemoryWorkflowRepo   │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────┐  ┌────────────────────────┐  │
                    │  │ Config       │  │ Kafka Producer         │  │
                    │  │ Loader       │  │ (saga events)          │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    └────────────────────────────────────────────────┘
```

---

## Kafka イベント

Saga の状態遷移時に以下のイベントを Kafka トピック `k1s0.system.saga.events.v1` に発行する。

| イベント | 発行タイミング |
| --- | --- |
| `SAGA_RUNNING` | Saga 実行開始時 |
| `SAGA_COMPLETED` | 全ステップ正常完了時 |
| `SAGA_COMPENSATING` | 補償処理開始時 |
| `SAGA_FAILED` | 補償処理完了（Saga 失敗確定）時 |

---

## SDK ライブラリ

他サービスから Saga Orchestrator を呼び出すためのクライアントライブラリを提供する。

```
regions/system/library/rust/saga/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── types.rs       # SagaStatus, SagaState, StartSagaRequest/Response 等の DTO
    └── client.rs      # SagaClient（gRPC クライアントラッパー）
```

---

## テスト

### ユニットテスト

各モジュール内の `#[cfg(test)]` ブロックで実装。mockall を使用してリポジトリ・gRPC caller をモック化。

| テスト対象 | テスト数 | 内容 |
| --- | --- | --- |
| domain/entity/saga_state | 10 | 状態遷移、ステータス変換、終端判定 |
| domain/entity/saga_step_log | 7 | ログ作成、成功/失敗/タイムアウトマーク |
| domain/entity/workflow | 6 | YAML 解析、バリデーション、バックオフ計算 |
| infrastructure/config | 2 | 設定デシリアライズ、デフォルト値 |
| infrastructure/database | 2 | 接続 URL 生成 |
| infrastructure/kafka_producer | 5 | KafkaConfig 解析 |
| infrastructure/grpc_caller | 5 | サービスレジストリ、エンドポイント解決 |
| adapter/repository/workflow_in_memory | 4 | 登録・取得・一覧 |
| usecase/execute_saga | 3 | 正常実行、ステップ失敗→補償、終端状態スキップ |
| usecase/start_saga | 2 | 正常開始、ワークフロー未登録エラー |
| usecase/recover_sagas | 2 | 未完了 Saga の自動再開 |
| usecase/get_saga | 2 | 取得成功、未存在 |
| usecase/list_sagas | 1 | 一覧取得 |
| usecase/cancel_saga | 3 | 正常キャンセル、終端状態エラー、未存在エラー |
| usecase/register_workflow | 2 | 正常登録、無効 YAML |
| usecase/list_workflows | 1 | 一覧取得 |
| **合計** | **56** | |

### 統合テスト

`tests/` ディレクトリに配置。外部依存を要するテストは `#[ignore]` でマークし、CI で選択的に実行する。

| テストファイル | 要件 | 内容 |
| --- | --- | --- |
| `integration_test.rs` | InMemory | REST API の統合テスト |
| `workflow_engine_test.rs` | モック | ワークフロー実行パス |
| `postgres_repository_test.rs` | PostgreSQL | DB 操作の検証 |
| `kafka_integration_test.rs` | Kafka | イベント発行の検証 |

---

## デプロイ

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/saga/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## API リクエスト・レスポンス例

### POST /api/v1/sagas

**リクエスト例**

```json
{
  "workflow_name": "order-fulfillment",
  "payload": {
    "order_id": "ord-12345",
    "customer_id": "cust-67890",
    "items": [
      {"product_id": "prod-001", "quantity": 2}
    ],
    "total_amount": 5000
  },
  "correlation_id": "req-abc-123",
  "initiated_by": "order-service"
}
```

**レスポンス（201 Created）**

```json
{
  "saga_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "STARTED"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_SAGA_VALIDATION_ERROR",
    "message": "workflow_name is required",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### GET /api/v1/sagas

**レスポンス（200 OK）**

```json
{
  "sagas": [
    {
      "saga_id": "550e8400-e29b-41d4-a716-446655440000",
      "workflow_name": "order-fulfillment",
      "current_step": 3,
      "status": "COMPLETED",
      "payload": {"order_id": "ord-12345"},
      "correlation_id": "req-abc-123",
      "initiated_by": "order-service",
      "error_message": null,
      "created_at": "2026-02-20T10:30:00.000Z",
      "updated_at": "2026-02-20T10:30:05.123Z"
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

### GET /api/v1/sagas/:saga_id

**レスポンス（200 OK）**

```json
{
  "saga": {
    "saga_id": "550e8400-e29b-41d4-a716-446655440000",
    "workflow_name": "order-fulfillment",
    "current_step": 3,
    "status": "COMPLETED",
    "payload": {"order_id": "ord-12345"},
    "correlation_id": "req-abc-123",
    "initiated_by": "order-service",
    "error_message": null,
    "created_at": "2026-02-20T10:30:00.000Z",
    "updated_at": "2026-02-20T10:30:05.123Z"
  },
  "step_logs": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "step_index": 0,
      "step_name": "reserve-inventory",
      "action": "EXECUTE",
      "status": "SUCCESS",
      "request_payload": {"order_id": "ord-12345"},
      "response_payload": {"reservation_id": "res-001"},
      "error_message": null,
      "started_at": "2026-02-20T10:30:00.100Z",
      "completed_at": "2026-02-20T10:30:01.200Z"
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440002",
      "step_index": 1,
      "step_name": "process-payment",
      "action": "EXECUTE",
      "status": "SUCCESS",
      "request_payload": {"order_id": "ord-12345"},
      "response_payload": {"transaction_id": "txn-001"},
      "error_message": null,
      "started_at": "2026-02-20T10:30:01.300Z",
      "completed_at": "2026-02-20T10:30:03.500Z"
    }
  ]
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_SAGA_NOT_FOUND",
    "message": "saga not found: invalid-uuid",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/sagas/:saga_id/cancel

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "saga 550e8400-e29b-41d4-a716-446655440000 cancelled"
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_SAGA_CONFLICT",
    "message": "saga is already in terminal state",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/sagas/workflows

**リクエスト例**

```json
{
  "workflow_yaml": "name: order-fulfillment
steps:
  - name: reserve-inventory
    service: inventory-service
    method: InventoryService.Reserve
    compensate: InventoryService.Release
    timeout_secs: 30
    retry:
      max_attempts: 3
      backoff: exponential
      initial_interval_ms: 1000
"
}
```

**レスポンス（201 Created）**

```json
{
  "name": "order-fulfillment",
  "step_count": 3
}
```

### GET /api/v1/sagas/workflows

**レスポンス（200 OK）**

```json
{
  "workflows": [
    {
      "name": "order-fulfillment",
      "step_count": 3,
      "step_names": ["reserve-inventory", "process-payment", "arrange-shipping"]
    }
  ]
}
```

---

## 実行フロー

### 正常系（全ステップ成功）

```
1. StartSaga API 呼び出し
2. SagaState 作成 (status=STARTED)
3. tokio::spawn で非同期実行開始
4. status → RUNNING
5. Step 0: gRPC 呼び出し → 成功 → step_log 記録
6. Step 1: gRPC 呼び出し → 成功 → step_log 記録
7. Step N: gRPC 呼び出し → 成功 → step_log 記録
8. status → COMPLETED
9. Kafka イベント発行: SAGA_COMPLETED
```

### 異常系（ステップ失敗 → 補償）

```
1. StartSaga API 呼び出し
2. SagaState 作成 (status=STARTED)
3. tokio::spawn で非同期実行開始
4. status → RUNNING
5. Step 0: gRPC 呼び出し → 成功
6. Step 1: gRPC 呼び出し → 成功
7. Step 2: gRPC 呼び出し → 失敗（リトライ上限到達）
8. status → COMPENSATING
9. Kafka イベント発行: SAGA_COMPENSATING
10. Compensate Step 1: compensate メソッド呼び出し → 成功/失敗（best-effort）
11. Compensate Step 0: compensate メソッド呼び出し → 成功/失敗（best-effort）
12. status → FAILED
13. Kafka イベント発行: SAGA_FAILED
```

---

## ワークフロー YAML 定義例

```yaml
name: order-fulfillment
steps:
  - name: reserve-inventory
    service: inventory-service
    method: InventoryService.Reserve
    compensate: InventoryService.Release
    timeout_secs: 30
    retry:
      max_attempts: 3
      backoff: exponential
      initial_interval_ms: 1000

  - name: process-payment
    service: payment-service
    method: PaymentService.Charge
    compensate: PaymentService.Refund
    timeout_secs: 60
    retry:
      max_attempts: 2
      backoff: exponential
      initial_interval_ms: 2000

  - name: arrange-shipping
    service: shipping-service
    method: ShippingService.CreateShipment
    compensate: ShippingService.CancelShipment
    timeout_secs: 30
```

---

## トランザクション設計

`update_with_step_log` メソッドでは、Saga 状態の更新とステップログの挿入を単一のデータベーストランザクションで実行する。

```
BEGIN;
  UPDATE saga.saga_states SET current_step=$2, status=$3, ... WHERE id=$1;
  INSERT INTO saga.saga_step_logs (id, saga_id, step_index, ...) VALUES (...);
COMMIT;
```

---

## Cargo.toml

```toml
[package]
name = "k1s0-saga-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web フレームワーク
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }

# gRPC
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"

# シリアライゼーション
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# DB
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }

# Kafka
rdkafka = { version = "0.36", features = ["cmake-build"] }

# 共通
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
async-trait = "0.1"
tracing = "0.1"

# 内部ライブラリ
k1s0-auth = { path = "../../library/rust/auth" }
k1s0-telemetry = { path = "../../library/rust/telemetry" }

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"
```

---

## サービスレジストリと gRPC 呼び出し

`config.yaml` の `services` セクションでサービスのエンドポイントを静的に定義し、`ServiceRegistry` が名前解決を行う。

```yaml
services:
  inventory-service:
    host: "inventory.k1s0-business.svc.cluster.local"
    port: 50051
  payment-service:
    host: "payment.k1s0-business.svc.cluster.local"
    port: 50051
```

`TonicGrpcCaller` は `ServiceRegistry` から取得したエンドポイントに対して tonic の gRPC チャネルを作成し、ワークフローステップの `method` フィールド（`ServiceName.MethodName` 形式）を gRPC パスに変換して動的に呼び出す。チャネルは `RwLock<HashMap<String, Channel>>` で接続プールとして管理する。

---

## Bootstrap（main.rs）

起動シーケンスは auth-server パターンに従う:

```
1. k1s0-telemetry 初期化
2. config.yaml ロード
3. PostgreSQL 接続プール作成（オプショナル）
4. SagaRepository 構築（Postgres or InMemory）
5. InMemoryWorkflowRepository 構築 + workflows/ ディレクトリからロード
6. ServiceRegistry + TonicGrpcCaller 構築
7. KafkaProducer 構築（オプショナル）
8. ユースケース群を構築（Arc でラップ）
9. RecoverSagasUseCase 実行（起動時リカバリ）
10. AppState 構築
11. REST サーバー（axum）+ gRPC サーバー（tonic）を tokio::select! で並行起動
```

---

## 設定ファイル例

### config.yaml（本番）

```yaml
app:
  name: "saga-server"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_system"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  consumer_group: "saga-server.default"
  security_protocol: "PLAINTEXT"
  topics:
    publish:
      - "k1s0.system.saga.events.v1"
    subscribe: []

services:
  inventory-service:
    host: "inventory.k1s0-business.svc.cluster.local"
    port: 50051
  payment-service:
    host: "payment.k1s0-business.svc.cluster.local"
    port: 50051
  shipping-service:
    host: "shipping.k1s0-business.svc.cluster.local"
    port: 50051

saga:
  max_concurrent: 100
  workflow_dir: "workflows"
```

---

## Helm values 例

```yaml
# values-saga.yaml（infra/helm/services/system/saga/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/saga
  tag: ""

replicaCount: 2

container:
  port: 8080
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
    - path: "secret/data/k1s0/system/saga/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
```

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [system-server.md](../auth/server.md) -- auth-server 設計（同一パターン）
- [system-saga-database.md](database.md) -- Saga データベーススキーマ・状態管理テーブル
- [テンプレート仕様-データベース.md](../../templates/data/データベース.md) -- データベースマイグレーション仕様
